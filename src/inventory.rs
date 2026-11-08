//! Hardware telemetry and read-only harness discovery. Never collect argv, environment,
//! prompts or transcripts. A sleeping process is not evidence of an idle agent.
use crate::model::{Status, now};
use anyhow::{Context, Result, ensure};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};
use sysinfo::{CpuRefreshKind, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};
use tokio::{net::UnixStream, time::timeout};
use tokio_tungstenite::{
    WebSocketStream,
    tungstenite::{Message, protocol::WebSocketConfig},
};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    pub enabled: bool,
    pub codex_socket: Option<PathBuf>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: true,
            codex_socket: None,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resources {
    /// OS-specific admission estimate; distinct from sysinfo's raw available-memory value.
    #[serde(default)]
    pub admission_memory_bytes: Option<u64>,
    #[serde(default)]
    pub admission_memory_source: Option<String>,
    pub cpu_model: String,
    pub logical_cpus: usize,
    pub physical_cores: Option<usize>,
    pub cpu_usage_percent: Option<f32>,
    pub load_average: [f64; 3],
    pub memory_total_bytes: u64,
    pub memory_available_bytes: u64,
    pub memory_used_bytes: u64,
    pub swap_used_bytes: u64,
    pub os_version: Option<String>,
    pub uptime_secs: u64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentState {
    Idle,
    Busy,
    Blocked,
    Unknown,
    Completed,
    Failed,
    Interrupted,
    Cancelled,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub harness: String,
    pub model: Option<String>,
    pub label: String,
    pub state: AgentState,
    pub source: String,
    pub observed_at: i64,
    pub managed: bool,
    pub task_id: Option<String>,
    pub origin_node: Option<String>,
    pub profile: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage_percent: f32,
    pub memory_bytes: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub observed_at: i64,
    pub resources: Resources,
    pub agents: Vec<AgentInfo>,
    pub runtimes: Vec<ProcessInfo>,
    pub codex_status: String,
    pub agents_truncated: bool,
}
pub type Inventory = Arc<Mutex<Option<Snapshot>>>;
pub type Managed = Arc<Mutex<BTreeMap<String, AgentInfo>>>;

pub fn started(
    registry: &Managed,
    id: &str,
    origin: &str,
    profile: &str,
    harness: Option<&str>,
    model: Option<&str>,
) {
    let mut agents = registry.lock().unwrap_or_else(|e| e.into_inner());
    agents.insert(
        id.into(),
        AgentInfo {
            id: id.into(),
            harness: harness.unwrap_or("unknown").into(),
            model: model.map(str::to_owned),
            label: profile.into(),
            state: AgentState::Busy,
            source: "citta_worker".into(),
            observed_at: now(),
            managed: true,
            task_id: Some(id.into()),
            origin_node: Some(origin.into()),
            profile: Some(profile.into()),
        },
    );
}
pub fn finished(registry: &Managed, id: &str, status: Status) {
    let mut agents = registry.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(agent) = agents.get_mut(id) {
        agent.state = match status {
            Status::Succeeded => AgentState::Completed,
            Status::Cancelled => AgentState::Cancelled,
            Status::Interrupted => AgentState::Interrupted,
            _ => AgentState::Failed,
        };
        agent.observed_at = now();
    }
    while agents
        .values()
        .filter(|a| a.state != AgentState::Busy)
        .count()
        > 128
    {
        let oldest = agents
            .iter()
            .filter(|(_, a)| a.state != AgentState::Busy)
            .min_by_key(|(_, a)| a.observed_at)
            .map(|(id, _)| id.clone());
        if let Some(id) = oldest {
            agents.remove(&id);
        }
    }
}

fn harness_name(name: &str) -> bool {
    matches!(
        name,
        "codex" | "claude" | "aider" | "gemini" | "opencode" | "ollama"
    )
}
fn sample(system: &mut System, first: bool) -> Snapshot {
    system.refresh_memory();
    system.refresh_cpu_specifics(CpuRefreshKind::nothing().with_cpu_usage());
    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing()
            .with_cpu()
            .with_memory()
            .with_user(UpdateKind::OnlyIfNotSet)
            .without_tasks(),
    );
    let uid = unsafe { libc::geteuid() };
    let mut runtimes: Vec<_> = system
        .processes()
        .iter()
        .filter_map(|(pid, p)| {
            let name = p.name().to_string_lossy().to_lowercase();
            (harness_name(&name) && p.user_id().is_some_and(|id| **id == uid)).then(|| {
                ProcessInfo {
                    pid: pid.as_u32(),
                    name,
                    cpu_usage_percent: p.cpu_usage(),
                    memory_bytes: p.memory(),
                }
            })
        })
        .collect();
    runtimes.sort_by_key(|p| p.pid);
    runtimes.truncate(256);
    let load = System::load_average();
    Snapshot {
        observed_at: now(),
        resources: Resources {
            admission_memory_bytes: admission_memory(system),
            admission_memory_source: Some(
                if cfg!(target_os = "macos") {
                    "macos_memorystatus_estimate"
                } else {
                    "sysinfo_available_memory"
                }
                .into(),
            ),
            cpu_model: system
                .cpus()
                .first()
                .map(|c| c.brand().to_string())
                .unwrap_or_else(|| "Unknown".into()),
            logical_cpus: system.cpus().len(),
            physical_cores: System::physical_core_count(),
            cpu_usage_percent: (!first).then(|| system.global_cpu_usage()),
            load_average: [load.one, load.five, load.fifteen],
            memory_total_bytes: system.total_memory(),
            memory_available_bytes: system.available_memory(),
            memory_used_bytes: system.used_memory(),
            swap_used_bytes: system.used_swap(),
            os_version: System::long_os_version(),
            uptime_secs: System::uptime(),
        },
        agents: vec![],
        runtimes,
        codex_status: "not_connected".into(),
        agents_truncated: false,
    }
}
fn admission_memory(system: &System) -> Option<u64> {
    #[cfg(target_os = "macos")]
    {
        // The kernel's system-wide memory level includes reclaimable capacity.
        // sysinfo 0.37 subtracts compressor pages and can report zero on a healthy Mac.
        let mut percent: libc::c_int = 0;
        let mut size = std::mem::size_of_val(&percent);
        let status = unsafe {
            libc::sysctlbyname(
                c"kern.memorystatus_level".as_ptr(),
                (&mut percent as *mut libc::c_int).cast(),
                &mut size,
                std::ptr::null_mut(),
                0,
            )
        };
        (status == 0 && size == std::mem::size_of_val(&percent) && (0..=100).contains(&percent))
            .then(|| system.total_memory().saturating_mul(percent as u64) / 100)
    }
    #[cfg(not(target_os = "macos"))]
    {
        (system.total_memory() > 0).then(|| system.available_memory())
    }
}
fn socket_path(settings: &Settings) -> Option<PathBuf> {
    settings.codex_socket.clone().or_else(|| {
        std::env::var_os("HOME").map(|home| {
            PathBuf::from(home).join(".codex/app-server-control/app-server-control.sock")
        })
    })
}
pub async fn run(settings: Settings, inventory: Inventory, stop: CancellationToken) {
    if !settings.enabled {
        return;
    }
    let mut system = System::new();
    let mut first = true;
    let mut interval = tokio::time::interval(Duration::from_secs(5));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {_=stop.cancelled()=>return,_=interval.tick()=>{}}
        let result = tokio::task::spawn_blocking(move || {
            let snapshot = sample(&mut system, first);
            (system, snapshot)
        })
        .await;
        let Ok((returned, mut snapshot)) = result else {
            return;
        };
        system = returned;
        first = false;
        if let Some(path) = socket_path(&settings) {
            match timeout(Duration::from_secs(3), codex_agents(path)).await {
                Ok(Ok((agents, truncated))) => {
                    snapshot.agents = agents;
                    snapshot.agents_truncated = truncated;
                    snapshot.codex_status = "connected".into();
                }
                Ok(Err(error)) => {
                    tracing::debug!(%error, "Codex inventory unavailable");
                    snapshot.codex_status = "unavailable".into();
                }
                Err(_) => snapshot.codex_status = "timeout".into(),
            }
        }
        // Processes remain a separate observation: CPU usage never creates an "idle" claim.
        *inventory.lock().unwrap_or_else(|e| e.into_inner()) = Some(snapshot);
    }
}

type Socket = WebSocketStream<UnixStream>;
async fn rpc(socket: &mut Socket, id: u64, method: &str, params: Value) -> Result<Value> {
    socket
        .send(Message::Text(
            json!({"id":id,"method":method,"params":params})
                .to_string()
                .into(),
        ))
        .await?;
    for _ in 0..256 {
        let message = socket.next().await.context("Codex disconnected")??;
        if let Message::Text(text) = message {
            let response: Value = serde_json::from_str(&text)?;
            if response.get("id").and_then(Value::as_u64) == Some(id) {
                ensure!(
                    response.get("error").is_none(),
                    "Codex inventory request rejected"
                );
                return response
                    .get("result")
                    .cloned()
                    .context("missing Codex result");
            }
            // Do not approve server requests or collect streamed task content.
            if response.get("method").is_some() && response.get("id").is_some() {
                socket.send(Message::Text(json!({"id":response["id"],"error":{"code":-32601,"message":"Read-only inventory client"}}).to_string().into())).await?;
            }
        }
    }
    anyhow::bail!("too many unrelated Codex messages")
}
fn state(value: &Value) -> AgentState {
    let kind = value
        .get("type")
        .and_then(Value::as_str)
        .or_else(|| value.as_str())
        .unwrap_or("unknown");
    match kind {
        "idle" => AgentState::Idle,
        "active" => {
            if value
                .get("activeFlags")
                .and_then(Value::as_array)
                .is_some_and(|flags| {
                    flags.iter().any(|f| {
                        matches!(f.as_str(), Some("waitingOnApproval" | "waitingOnUserInput"))
                    })
                })
            {
                AgentState::Blocked
            } else {
                AgentState::Busy
            }
        }
        "systemError" => AgentState::Failed,
        _ => AgentState::Unknown,
    }
}
fn from_thread(thread: &Value) -> Result<AgentInfo> {
    let id = thread
        .get("id")
        .and_then(Value::as_str)
        .context("missing thread ID")?;
    ensure!(id.len() <= 128, "invalid thread ID");
    let label = thread
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("Codex session")
        .chars()
        .take(160)
        .collect();
    Ok(AgentInfo {
        id: format!("codex:{id}"),
        harness: "codex".into(),
        model: thread
            .get("model")
            .and_then(Value::as_str)
            .map(|s| s.chars().take(100).collect()),
        label,
        state: state(&thread["status"]),
        source: "codex_app_server".into(),
        observed_at: now(),
        managed: false,
        task_id: None,
        origin_node: None,
        profile: None,
    })
}
async fn codex_agents(path: PathBuf) -> Result<(Vec<AgentInfo>, bool)> {
    use std::os::unix::fs::{FileTypeExt, MetadataExt};
    let uid = unsafe { libc::geteuid() };
    let link = std::fs::symlink_metadata(&path)?;
    ensure!(
        link.uid() == uid,
        "Codex socket path must belong to this user"
    );
    // Codex publishes a user-owned symlink into its private runtime directory.
    let path = std::fs::canonicalize(path)?;
    let meta = std::fs::symlink_metadata(&path)?;
    ensure!(
        meta.file_type().is_socket() && meta.uid() == uid,
        "Codex socket must belong to this user"
    );
    let stream = UnixStream::connect(path).await?;
    let config = WebSocketConfig::default()
        .max_message_size(Some(4 * 1024 * 1024))
        .max_frame_size(Some(4 * 1024 * 1024));
    let (mut socket, _) =
        tokio_tungstenite::client_async_with_config("ws://localhost/", stream, Some(config))
            .await?;
    rpc(&mut socket,1,"initialize",json!({"clientInfo":{"name":"citta_inventory","version":env!("CARGO_PKG_VERSION")},"capabilities":{"experimentalApi":true}})).await?;
    socket
        .send(Message::Text(
            json!({"method":"initialized"}).to_string().into(),
        ))
        .await?;
    let loaded = rpc(&mut socket, 2, "thread/loaded/list", json!({"limit":128})).await?;
    let ids = loaded
        .get("data")
        .and_then(Value::as_array)
        .context("invalid loaded threads")?;
    let truncated = ids.len() > 128 || loaded.get("nextCursor").is_some_and(|v| !v.is_null());
    let mut agents = Vec::new();
    for (i, id) in ids.iter().take(128).enumerate() {
        let Some(id) = id.as_str() else {
            continue;
        };
        let response = rpc(
            &mut socket,
            3 + i as u64,
            "thread/read",
            json!({"threadId":id,"includeTurns":false}),
        )
        .await?;
        if let Ok(agent) = from_thread(&response["thread"]) {
            agents.push(agent);
        }
    }
    socket.close(None).await?;
    Ok((agents, truncated))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn codex_socket_symlink_collects_status_without_reading_turns() {
        let dir = tempfile::tempdir().unwrap();
        // macOS has a short sockaddr_un limit; keep the socket name compact.
        let socket = dir.path().join("s");
        let link = dir.path().join("l");
        let listener = tokio::net::UnixListener::bind(&socket).unwrap();
        std::os::unix::fs::symlink(&socket, &link).unwrap();
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut ws = tokio_tungstenite::accept_async(stream).await.unwrap();
            for method in [
                "initialize",
                "initialized",
                "thread/loaded/list",
                "thread/read",
            ] {
                let message = ws.next().await.unwrap().unwrap();
                let value: Value = serde_json::from_str(message.to_text().unwrap()).unwrap();
                assert_eq!(value["method"], method);
                if method == "initialized" {
                    continue;
                }
                let result = match method {
                    "initialize" => json!({}),
                    "thread/loaded/list" => json!({"data":["session"],"nextCursor":null}),
                    _ => {
                        assert_eq!(value["params"]["includeTurns"], false);
                        json!({"thread":{"id":"session","name":"Fixture","status":{"type":"idle"},"preview":"secret"}})
                    }
                };
                ws.send(Message::Text(
                    json!({"id":value["id"],"result":result}).to_string().into(),
                ))
                .await
                .unwrap();
            }
            let _ = ws.next().await;
        });
        let (agents, truncated) = codex_agents(link).await.unwrap();
        server.await.unwrap();
        assert!(!truncated);
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].state, AgentState::Idle);
        assert!(!serde_json::to_string(&agents).unwrap().contains("secret"));
    }
    #[test]
    fn harness_status_is_authoritative_and_payload_is_minimal() {
        let thread = json!({"id":"test","name":"Worker","status":{"type":"active","activeFlags":["waitingOnApproval"]},"preview":"secret prompt","cwd":"/private/path","turns":[{"secret":"history"}]});
        let agent = from_thread(&thread).unwrap();
        assert_eq!(agent.state, AgentState::Blocked);
        let output = serde_json::to_string(&agent).unwrap();
        assert!(!output.contains("secret"));
        assert!(!output.contains("private/path"));
        assert_eq!(state(&json!({"type":"idle"})), AgentState::Idle);
        assert_eq!(state(&json!({"type":"notLoaded"})), AgentState::Unknown);
    }
    #[test]
    fn managed_agent_finishes_and_retention_is_bounded() {
        let managed = Managed::default();
        for i in 0..200 {
            let id = i.to_string();
            started(
                &managed,
                &id,
                "origin",
                "codex",
                Some("fixture-harness"),
                Some("fixture-model"),
            );
        }
        assert_eq!(managed.lock().unwrap().len(), 200);
        assert_eq!(managed.lock().unwrap()["0"].harness, "fixture-harness");
        assert_eq!(
            managed.lock().unwrap()["0"].model.as_deref(),
            Some("fixture-model")
        );
        for i in 0..200 {
            finished(&managed, &i.to_string(), Status::Succeeded);
        }
        assert_eq!(managed.lock().unwrap().len(), 128);
        assert_eq!(managed.lock().unwrap()["199"].state, AgentState::Completed);
    }
}
