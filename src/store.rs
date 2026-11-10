use crate::model::*;
use anyhow::{Context, Result, bail, ensure};
use fs2::FileExt;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use sha2::{Digest, Sha256};
use std::{fs::File, path::Path, sync::Mutex, time::Duration};

#[derive(Debug)]
pub enum Rejection {
    NotFound,
    Conflict,
    Capacity,
}
impl std::fmt::Display for Rejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::NotFound => "task not found",
            Self::Conflict => "task state, lease, or idempotency conflict",
            Self::Capacity => "task storage capacity reached; drain or prune history",
        })
    }
}
impl std::error::Error for Rejection {}

pub struct Store {
    db: Mutex<Connection>,
    _lock: File,
    node: String,
    lease_secs: u64,
    max_pending: usize,
    max_history: usize,
}
fn hash(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}
fn state_name(status: Status) -> &'static str {
    match status {
        Status::Queued => "queued",
        Status::Running => "running",
        Status::Succeeded => "succeeded",
        Status::Failed => "failed",
        Status::Cancelled => "cancelled",
        Status::Interrupted => "interrupted",
    }
}
impl Store {
    pub fn open(
        dir: &Path,
        node: &str,
        lease_secs: u64,
        max_pending: usize,
        max_history: usize,
    ) -> Result<Self> {
        crate::config::private_dir(dir)?;
        let lock = File::options()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(dir.join("daemon.lock"))?;
        lock.try_lock_exclusive()
            .context("another daemon or maintenance command owns this data directory")?;
        let db = Connection::open(dir.join("citta.db"))?;
        db.busy_timeout(Duration::from_secs(5))?;
        db.execute_batch(
            "PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; PRAGMA foreign_keys=ON;",
        )?;
        let version: i64 = db.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        ensure!(version <= 4, "database is newer than this binary");
        db.execute_batch(
            "BEGIN IMMEDIATE;
            CREATE TABLE IF NOT EXISTS meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS tasks(
                id TEXT PRIMARY KEY, request_key TEXT NOT NULL UNIQUE,
                status TEXT NOT NULL, capability TEXT NOT NULL, target TEXT,
                created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL,
                deadline INTEGER NOT NULL, lease_expires INTEGER, lease_hash TEXT,
                payload TEXT NOT NULL);
            CREATE INDEX IF NOT EXISTS queue_idx ON tasks(status, capability, created_at);
            CREATE TABLE IF NOT EXISTS events(
                sequence INTEGER PRIMARY KEY AUTOINCREMENT,
                task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
                status TEXT NOT NULL, at INTEGER NOT NULL);
            CREATE TABLE IF NOT EXISTS device_permissions(id INTEGER PRIMARY KEY CHECK(id=1),revision INTEGER NOT NULL,payload TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS permission_events(revision INTEGER PRIMARY KEY,payload TEXT NOT NULL);
            PRAGMA user_version=4; COMMIT;",
        )?;
        db.execute(
            "INSERT OR IGNORE INTO meta(key,value) VALUES('node',?1)",
            [node],
        )?;
        let stored: String =
            db.query_row("SELECT value FROM meta WHERE key='node'", [], |r| r.get(0))?;
        ensure!(stored == node, "data directory belongs to another node");
        Ok(Self {
            db: Mutex::new(db),
            _lock: lock,
            node: node.into(),
            lease_secs,
            max_pending,
            max_history,
        })
    }
    fn transaction<T>(&self, f: impl FnOnce(&Transaction<'_>) -> Result<T>) -> Result<T> {
        let mut db = self
            .db
            .lock()
            .map_err(|_| anyhow::anyhow!("database mutex poisoned"))?;
        let tx = db.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let result = f(&tx)?;
        tx.commit()?;
        Ok(result)
    }
    pub fn initialize_permissions(
        &self,
        policy: crate::permissions::Policy,
    ) -> Result<crate::permissions::Snapshot> {
        self.transaction(|tx| {
            let existing: Option<String> = tx
                .query_row(
                    "SELECT payload FROM device_permissions WHERE id=1",
                    [],
                    |r| r.get(0),
                )
                .optional()?;
            if let Some(value) = existing {
                return Ok(serde_json::from_str(&value)?);
            }
            let initialized: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM meta WHERE key='permissions_initialized')",
                [],
                |r| r.get(0),
            )?;
            ensure!(
                !initialized,
                "saved device permissions are missing; refusing to reset grants"
            );
            let state = crate::permissions::Snapshot {
                revision: 1,
                policy,
                updated_at: now(),
                actor: "migration".into(),
            };
            let payload = serde_json::to_string(&state)?;
            tx.execute("INSERT INTO device_permissions VALUES(1,1,?1)", [&payload])?;
            tx.execute("INSERT INTO permission_events VALUES(1,?1)", [&payload])?;
            tx.execute(
                "INSERT INTO meta(key,value) VALUES('permissions_initialized','true')",
                [],
            )?;
            Ok(state)
        })
    }
    pub fn save_permissions(
        &self,
        state: &crate::permissions::Snapshot,
        expected: i64,
    ) -> Result<()> {
        self.transaction(|tx| {
            let payload = serde_json::to_string(state)?;
            let count = tx.execute(
                "UPDATE device_permissions SET revision=?1,payload=?2 WHERE id=1 AND revision=?3",
                params![state.revision, payload, expected],
            )?;
            if count != 1 {
                bail!(Rejection::Conflict);
            }
            tx.execute(
                "INSERT INTO permission_events VALUES(?1,?2)",
                params![state.revision, payload],
            )?;
            Ok(())
        })
    }
    pub fn permission_history(&self) -> Result<Vec<crate::permissions::Snapshot>> {
        self.transaction(|tx| {
            let mut query = tx
                .prepare("SELECT payload FROM permission_events ORDER BY revision DESC LIMIT 20")?;
            let rows = query.query_map([], |r| r.get::<_, String>(0))?;
            rows.map(|r| Ok(serde_json::from_str(&r?)?)).collect()
        })
    }
    pub fn submit(&self, request: Submission, at: i64) -> Result<Task> {
        self.submit_as(request, at, None)
    }
    pub fn submit_as(
        &self,
        request: Submission,
        at: i64,
        requester: Option<String>,
    ) -> Result<Task> {
        request.validate()?;
        self.transaction(|tx| {
            if let Some(task) = find(tx, "SELECT payload FROM tasks WHERE request_key=?1", &request.idempotency_key)? {
                if task.request != request || (task.requester.is_some() && task.requester != requester) { bail!(Rejection::Conflict); }
                return Ok(task);
            }
            let pending: i64 = tx.query_row("SELECT COUNT(*) FROM tasks WHERE status IN ('queued','running')", [], |r| r.get(0))?;
            let total: i64 = tx.query_row("SELECT COUNT(*) FROM tasks", [], |r| r.get(0))?;
            if pending >= self.max_pending as i64 || total >= self.max_history as i64 { bail!(Rejection::Capacity); }
            let task = Task { requester, id: uuid::Uuid::new_v4().to_string(), origin_node: self.node.clone(), request, status: Status::Queued, worker_node: None, created_at: at, updated_at: at, started_at: None, lease_expires_at: None, outcome: None };
            tx.execute("INSERT INTO tasks(id,request_key,status,capability,target,created_at,updated_at,deadline,payload) VALUES(?1,?2,'queued',?3,?4,?5,?5,?6,?7)", params![task.id, task.request.idempotency_key, task.request.capability, task.request.target_node, at, at + task.request.queue_ttl_secs as i64, serde_json::to_string(&task)?])?;
            event(tx, &task)?;
            Ok(task)
        })
    }
    pub fn existing_submission(
        &self,
        request: &Submission,
        requester: Option<&String>,
    ) -> Result<Option<Task>> {
        self.transaction(|tx| {
            let task = find(
                tx,
                "SELECT payload FROM tasks WHERE request_key=?1",
                &request.idempotency_key,
            )?;
            if task.as_ref().is_some_and(|task| {
                task.request != *request
                    || (task.requester.is_some() && task.requester.as_ref() != requester)
            }) {
                bail!(Rejection::Conflict);
            }
            Ok(task)
        })
    }
    pub fn by_key(&self, key: &str) -> Result<Task> {
        ensure!(valid_name(key), "invalid idempotency key");
        self.transaction(|tx| {
            find(tx, "SELECT payload FROM tasks WHERE request_key=?1", key)?
                .ok_or_else(|| Rejection::NotFound.into())
        })
    }
    pub fn claim(&self, request: ClaimRequest, at: i64) -> Result<Option<Claim>> {
        ensure!(
            valid_name(&request.worker_node) && valid_name(&request.capability),
            "invalid claim"
        );
        if request
            .disk
            .as_ref()
            .is_none_or(|disk| disk.fresh_reason(at).is_some())
        {
            return Ok(None);
        }
        self.transaction(|tx| {
            let mut query = tx.prepare("SELECT payload FROM tasks WHERE status='queued' AND capability=?1 AND (target IS NULL OR target=?2) AND deadline>?3 AND (COALESCE(json_extract(payload, '$.request.agent_only'),0)=0 OR ?4) ORDER BY created_at, rowid")?;
            let rows = query.query_map(params![request.capability, request.worker_node, at, request.agent_profile], |r| r.get::<_, String>(0))?;
            let mut selected = None;
            for raw in rows {
                let task: Task = serde_json::from_str(&raw?)?;
                if task.request.requirements.as_ref().is_none_or(|req| request.offer.as_ref().is_some_and(|offer| offer.fits(req, &request.worker_node, &self.node, at))) {
                    selected = Some(task);
                    break;
                }
            }
            let Some(mut task) = selected else { return Ok(None) };
            task.status = Status::Running;
            task.worker_node = Some(request.worker_node);
            task.started_at = Some(at);
            task.updated_at = at;
            task.lease_expires_at = Some(at + self.lease_secs as i64);
            let token = format!("{}{}", uuid::Uuid::new_v4().simple(), uuid::Uuid::new_v4().simple());
            save(tx, &task, true)?;
            tx.execute("UPDATE tasks SET lease_hash=?1 WHERE id=?2", params![hash(&token), task.id])?;
            Ok(Some(Claim { task, lease_token: token, lease_secs: self.lease_secs }))
        })
    }
    pub fn renew(&self, id: &str, token: &str, at: i64) -> Result<Task> {
        self.transaction(|tx| {
            let mut task = leased(tx, id, token)?;
            check_live(&task, at)?;
            task.lease_expires_at = Some(at + self.lease_secs as i64);
            task.updated_at = at;
            save(tx, &task, false)?;
            Ok(task)
        })
    }
    pub fn complete(&self, id: &str, completion: Completion, at: i64) -> Result<Task> {
        ensure!(
            matches!(
                completion.outcome.status,
                Status::Succeeded | Status::Failed | Status::Interrupted
            ),
            "worker may report only succeeded, failed, or interrupted"
        );
        ensure!(
            completion.outcome.stdout.len() <= MAX_OUTPUT
                && completion.outcome.stderr.len() <= MAX_OUTPUT
                && completion
                    .outcome
                    .error
                    .as_ref()
                    .is_none_or(|s| s.len() <= 4096),
            "output too large"
        );
        self.transaction(|tx| {
            let mut task = leased(tx, id, &completion.lease_token)?;
            if task.status.terminal() && task.outcome.as_ref() == Some(&completion.outcome) {
                return Ok(task);
            }
            check_live(&task, at)?;
            task.status = completion.outcome.status;
            task.outcome = Some(completion.outcome);
            task.updated_at = at;
            task.lease_expires_at = None;
            save(tx, &task, true)?;
            Ok(task)
        })
    }
    pub fn get(&self, id: &str) -> Result<Task> {
        self.transaction(|tx| {
            find(tx, "SELECT payload FROM tasks WHERE id=?1", id)?
                .ok_or_else(|| Rejection::NotFound.into())
        })
    }
    pub fn list(&self, limit: usize, before: i64) -> Result<Vec<Task>> {
        self.transaction(|tx| {
            let mut stmt = tx
                .prepare("SELECT payload FROM tasks WHERE rowid<?1 ORDER BY rowid DESC LIMIT ?2")?;
            let values = stmt.query_map(params![before, limit.min(100) as i64], |r| {
                r.get::<_, String>(0)
            })?;
            values.map(|v| Ok(serde_json::from_str(&v?)?)).collect()
        })
    }
    pub fn cancel(&self, id: &str, at: i64) -> Result<Task> {
        self.transaction(|tx| {
            let mut task = find(tx, "SELECT payload FROM tasks WHERE id=?1", id)?
                .ok_or(Rejection::NotFound)?;
            if task.status.terminal() {
                return Ok(task);
            }
            task.status = Status::Cancelled;
            task.updated_at = at;
            task.lease_expires_at = None;
            task.outcome = Some(Outcome::error(
                Status::Cancelled,
                "cancelled by requester; worker stops on next renewal",
            ));
            save(tx, &task, true)?;
            Ok(task)
        })
    }
    pub fn sweep(&self, at: i64) -> Result<usize> {
        self.transaction(|tx| {
            let mut stmt = tx.prepare("SELECT payload FROM tasks WHERE (status='running' AND lease_expires<=?1) OR (status='queued' AND deadline<=?1)")?;
            let raws = stmt.query_map([at], |r| r.get::<_, String>(0))?.collect::<rusqlite::Result<Vec<_>>>()?;
            let count = raws.len();
            for raw in raws {
                let mut task: Task = serde_json::from_str(&raw)?;
                let reason = if task.status == Status::Queued { "queue deadline expired" } else { "worker lease expired; side effects may have occurred; no automatic retry" };
                task.status = Status::Interrupted;
                task.updated_at = at;
                task.lease_expires_at = None;
                task.outcome = Some(Outcome::error(Status::Interrupted, reason));
                save(tx, &task, true)?;
            }
            Ok(count)
        })
    }
    pub fn events(&self, after: i64) -> Result<Vec<Event>> {
        self.transaction(|tx| {
            let mut stmt = tx.prepare("SELECT sequence,task_id,status,at FROM events WHERE sequence>?1 ORDER BY sequence LIMIT 100")?;
            let rows = stmt.query_map([after], |r| Ok((r.get::<_,i64>(0)?, r.get::<_,String>(1)?, r.get::<_,String>(2)?, r.get::<_,i64>(3)?)))?;
            rows.map(|r| { let (sequence, task_id, status, at) = r?; Ok(Event { sequence, task_id, status: serde_json::from_value(serde_json::Value::String(status))?, at }) }).collect()
        })
    }
    pub fn prune(&self, before: i64) -> Result<usize> {
        self.transaction(|tx| {
            Ok(tx.execute(
                "DELETE FROM tasks WHERE status NOT IN ('queued','running') AND updated_at<?1",
                [before],
            )?)
        })
    }
}
fn find(tx: &Transaction<'_>, sql: &str, key: &str) -> Result<Option<Task>> {
    let raw: Option<String> = tx.query_row(sql, [key], |r| r.get(0)).optional()?;
    raw.map(|s| serde_json::from_str(&s).map_err(Into::into))
        .transpose()
}
fn leased(tx: &Transaction<'_>, id: &str, token: &str) -> Result<Task> {
    let raw: Option<(String, Option<String>)> = tx
        .query_row(
            "SELECT payload,lease_hash FROM tasks WHERE id=?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let Some((raw, digest)) = raw else {
        bail!(Rejection::NotFound)
    };
    if digest.as_deref() != Some(&hash(token)) {
        bail!(Rejection::Conflict);
    }
    Ok(serde_json::from_str(&raw)?)
}
fn check_live(task: &Task, at: i64) -> Result<()> {
    if task.status != Status::Running || task.lease_expires_at.is_none_or(|expiry| expiry <= at) {
        bail!(Rejection::Conflict);
    }
    Ok(())
}
fn save(tx: &Transaction<'_>, task: &Task, changed: bool) -> Result<()> {
    tx.execute(
        "UPDATE tasks SET status=?1,updated_at=?2,lease_expires=?3,payload=?4 WHERE id=?5",
        params![
            state_name(task.status),
            task.updated_at,
            task.lease_expires_at,
            serde_json::to_string(task)?,
            task.id
        ],
    )?;
    if changed {
        event(tx, task)?;
    }
    Ok(())
}
fn event(tx: &Transaction<'_>, task: &Task) -> Result<()> {
    tx.execute(
        "INSERT INTO events(task_id,status,at) VALUES(?1,?2,?3)",
        params![task.id, state_name(task.status), task.updated_at],
    )?;
    Ok(())
}
