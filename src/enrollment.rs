//! Same-owner enrollment. Neither identity headers nor caller-supplied URLs are trusted.
//! A nonce sent over verified HTTPS must return via a reverse HTTPS callback before
//! either node accepts a credential. Only the local Tailscale netmap chooses URLs.
use crate::{
    client::Client,
    config::{self, Config},
    model::{now, valid_name},
};
use anyhow::{Context, Result, ensure};
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, StatusCode},
    routing::post,
};
use futures_util::{StreamExt, stream};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    net::{IpAddr, SocketAddr},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use subtle::ConstantTimeEq;
use tokio::{io::AsyncReadExt, process::Command, sync::Semaphore};
use tokio_util::sync::CancellationToken;

const LIMIT: usize = 64;
const FRESH: Duration = Duration::from_secs(30);
const MAX_PAIR_BODY: usize = 4096;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub tailscale_bin: PathBuf,
    pub owner_user_id: u64,
    pub tailnet_id: String,
    pub device_id: String,
    #[serde(default = "default_port")]
    pub https_port: u16,
}
fn default_port() -> u16 {
    7332
}
impl Settings {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.tailscale_bin.is_absolute() && self.tailscale_bin.is_file(),
            "tailscale_bin must be an existing absolute executable"
        );
        ensure!(
            self.owner_user_id > 0 && valid_name(&self.tailnet_id) && valid_name(&self.device_id),
            "invalid pinned Tailscale identity"
        );
        ensure!(self.https_port > 0, "HTTPS port must be nonzero");
        Ok(())
    }
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Device {
    #[serde(rename = "ID")]
    id: String,
    #[serde(rename = "UserID")]
    user_id: u64,
    #[serde(rename = "DNSName")]
    dns_name: String,
    #[serde(rename = "TailscaleIPs", default)]
    ips: Vec<IpAddr>,
    #[serde(default)]
    tags: Option<Vec<String>>,
    #[serde(default)]
    expired: bool,
    #[serde(default)]
    online: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Tailnet {
    #[serde(rename = "StableID")]
    stable_id: String,
    #[serde(rename = "MagicDNSSuffix")]
    suffix: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Status {
    backend_state: String,
    #[serde(rename = "Self")]
    this: Device,
    current_tailnet: Tailnet,
    #[serde(default)]
    peer: BTreeMap<String, Device>,
}
#[derive(Clone, PartialEq)]
struct Candidate {
    id: String,
    url: String,
    address: IpAddr,
    online: bool,
    #[cfg(test)]
    test_ca: Option<Vec<u8>>,
}
fn tail_address(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            let n = u32::from(ip);
            n & 0xffc00000 == 0x64400000
        }
        IpAddr::V6(ip) => ip.segments()[..3] == [0xfd7a, 0x115c, 0xa1e0],
    }
}
fn candidate(device: &Device, suffix: &str, settings: &Settings) -> Result<Candidate> {
    ensure!(
        valid_name(&device.id)
            && device.user_id == settings.owner_user_id
            && !device.expired
            && device.tags.as_ref().is_none_or(Vec::is_empty),
        "device is not an untagged same-owner device"
    );
    let dns = device.dns_name.trim_end_matches('.');
    let suffix = suffix.trim_end_matches('.');
    ensure!(
        suffix.ends_with(".ts.net") && dns.ends_with(&format!(".{suffix}")),
        "invalid Tailscale DNS suffix"
    );
    ensure!(
        dns.len() <= 253
            && dns.split('.').all(|label| !label.is_empty()
                && label
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-')),
        "invalid Tailscale DNS name"
    );
    let address = device
        .ips
        .iter()
        .copied()
        .find(|ip| ip.is_ipv4() && tail_address(*ip))
        .or_else(|| device.ips.iter().copied().find(|ip| tail_address(*ip)))
        .context("no Tailscale address")?;
    Ok(Candidate {
        id: device.id.clone(),
        url: format!("https://{dns}:{}", settings.https_port),
        address,
        online: device.online,
        #[cfg(test)]
        test_ca: None,
    })
}
fn candidates(status: &Status, settings: &Settings) -> Result<BTreeMap<String, Candidate>> {
    ensure!(
        status.backend_state == "Running"
            && status.current_tailnet.stable_id == settings.tailnet_id
            && status.this.id == settings.device_id,
        "Tailscale identity changed or disconnected"
    );
    candidate(&status.this, &status.current_tailnet.suffix, settings)?;
    let mut result = BTreeMap::new();
    for d in status.peer.values() {
        if d.id == settings.device_id {
            continue;
        }
        if let Ok(c) = candidate(d, &status.current_tailnet.suffix, settings) {
            ensure!(!result.contains_key(&c.id), "duplicate Tailscale identity");
            result.insert(c.id.clone(), c);
        }
    }
    ensure!(
        result.len() <= LIMIT,
        "more than 64 same-owner devices; explicit enrollment scope required"
    );
    Ok(result)
}

// Local subprocess output is bounded and never included in errors (it may contain credentials).
async fn tailscale(bin: &Path, args: &[&str]) -> Result<Vec<u8>> {
    use std::process::Stdio;
    let mut child = Command::new(bin)
        .args(args)
        // The macOS app executable otherwise launches its GUI when launchd has no TERM.
        .env("TAILSCALE_BE_CLI", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .context("start Tailscale CLI")?;
    let mut stdout = child
        .stdout
        .take()
        .context("Tailscale stdout")?
        .take(2 * 1024 * 1024 + 1);
    let operation = async {
        let mut bytes = Vec::new();
        stdout.read_to_end(&mut bytes).await?;
        ensure!(
            bytes.len() <= 2 * 1024 * 1024,
            "Tailscale response too large"
        );
        ensure!(child.wait().await?.success(), "Tailscale command failed");
        Ok(bytes)
    };
    tokio::time::timeout(Duration::from_secs(8), operation)
        .await
        .context("Tailscale command timed out")?
}
async fn status(bin: &Path) -> Result<Status> {
    serde_json::from_slice(&tailscale(bin, &["status", "--json"]).await?)
        .context("invalid Tailscale status")
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    node_id: String,
    url: String,
    inbound: String,
    outbound: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Saved {
    version: u32,
    device_id: String,
    tailnet_id: String,
    owner_user_id: u64,
    peers: BTreeMap<String, Record>,
}
struct Pending {
    challenge: String,
    candidate: Candidate,
    at: Instant,
}
struct Inner {
    records: BTreeMap<String, Record>,
    candidates: BTreeMap<String, Candidate>,
    issued: BTreeMap<String, String>,
    clients: BTreeMap<String, (String, String, Client)>,
    pending: BTreeMap<String, Pending>,
    last_callback: BTreeMap<String, Instant>,
    public_url: Option<String>,
    refreshed: Option<Instant>,
    checked_at: Option<i64>,
    error: Option<&'static str>,
}
pub struct Enrollment {
    control: bool,
    settings: Settings,
    config: Arc<Config>,
    path: PathBuf,
    inner: Mutex<Inner>,
    callbacks: Semaphore,
}
fn secret() -> String {
    format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    )
}
fn valid_secret(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}
fn fresh(inner: &Inner) -> bool {
    inner.refreshed.is_some_and(|at| at.elapsed() < FRESH)
}
fn trusted<'a>(inner: &'a Inner, id: &str) -> Result<&'a Candidate> {
    ensure!(fresh(inner), "Tailscale identity is stale");
    inner.candidates.get(id).context("device not eligible")
}
fn active(inner: &Inner) -> impl Iterator<Item = (&String, &Record)> {
    inner.records.iter().filter(|(id, record)| {
        fresh(inner)
            && inner
                .candidates
                .get(*id)
                .is_some_and(|c| c.url == record.url)
    })
}
impl Enrollment {
    pub fn open(config: Arc<Config>) -> Result<Option<Arc<Self>>> {
        Self::open_channel(config, false)
    }
    pub fn open_control(config: Arc<Config>) -> Result<Option<Arc<Self>>> {
        Self::open_channel(config, true)
    }
    fn open_channel(config: Arc<Config>, control: bool) -> Result<Option<Arc<Self>>> {
        let Some(settings) = config.enrollment.clone() else {
            return Ok(None);
        };
        let path = config.data_dir.join(if control {
            "control-enrollment.json"
        } else {
            "enrollment.json"
        });
        let records = if path.exists() {
            let metadata = std::fs::symlink_metadata(&path)?;
            use std::os::unix::fs::PermissionsExt;
            ensure!(
                metadata.is_file()
                    && metadata.len() <= 128 * 1024
                    && metadata.permissions().mode() & 0o077 == 0,
                "enrollment state must be a private regular file"
            );
            let saved: Saved = serde_json::from_slice(&std::fs::read(&path)?)
                .context("invalid enrollment state")?;
            ensure!(
                saved.version == 1
                    && saved.device_id == settings.device_id
                    && saved.tailnet_id == settings.tailnet_id
                    && saved.owner_user_id == settings.owner_user_id
                    && saved.peers.len() <= LIMIT,
                "enrollment state identity mismatch"
            );
            for (id, r) in &saved.peers {
                ensure!(
                    valid_name(id)
                        && valid_name(&r.node_id)
                        && valid_secret(&r.inbound)
                        && valid_secret(&r.outbound),
                    "invalid enrollment record"
                );
                config::validate_url(&r.url)?;
            }
            saved.peers
        } else {
            BTreeMap::new()
        };
        let issued = records
            .iter()
            .map(|(id, r)| (id.clone(), r.inbound.clone()))
            .collect();
        Ok(Some(Arc::new(Self {
            control,
            settings,
            config,
            path,
            inner: Mutex::new(Inner {
                records,
                issued,
                clients: BTreeMap::new(),
                candidates: BTreeMap::new(),
                pending: BTreeMap::new(),
                last_callback: BTreeMap::new(),
                public_url: None,
                refreshed: None,
                checked_at: None,
                error: Some("waiting_for_tailscale"),
            }),
            callbacks: Semaphore::new(4),
        })))
    }
    pub fn clients(&self) -> Vec<Client> {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let records: Vec<_> = active(&inner)
            .map(|(id, r)| (id.clone(), r.clone()))
            .collect();
        inner
            .clients
            .retain(|id, _| records.iter().any(|(key, _)| key == id));
        let mut result = Vec::new();
        for (id, record) in records {
            let cached = inner
                .clients
                .get(&id)
                .filter(|(url, token, _)| url == &record.url && token == &record.outbound);
            if let Some((_, _, client)) = cached {
                result.push(client.clone());
            } else if let Ok(client) =
                Client::new(&record.url, record.outbound.clone(), record.node_id)
            {
                #[cfg(test)]
                let client = {
                    let mut client = client;
                    if let Some(candidate) = inner.candidates.get(&id)
                        && let Some(ca) = &candidate.test_ca
                    {
                        client.test_endpoint(ca, candidate.address).unwrap();
                    }
                    client
                };
                inner
                    .clients
                    .insert(id, (record.url, record.outbound, client.clone()));
                result.push(client);
            }
        }
        result
    }
    pub fn public_url(&self) -> Option<String> {
        let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        if fresh(&inner) {
            inner.public_url.clone()
        } else {
            None
        }
    }
    pub fn principal(&self, node: &str) -> Option<String> {
        let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        active(&inner)
            .find(|(_, r)| r.node_id == node)
            .map(|(id, _)| format!("tailscale:{id}"))
    }
    pub fn authenticate(&self, token: &str) -> Option<String> {
        let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        active(&inner)
            .find(|(_, r)| bool::from(token.as_bytes().ct_eq(r.inbound.as_bytes())))
            .map(|(_, r)| r.node_id.clone())
    }
    pub fn report(&self) -> Value {
        let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let peers:Vec<Value> = inner.candidates.values().map(|c| {
            let record = inner.records.get(&c.id).filter(|r| fresh(&inner) && r.url == c.url);
            json!({"device_id":c.id,"url":c.url,"online":c.online,"node_id":record.map(|r| &r.node_id),"state":if record.is_some() {"paired"} else {"waiting_for_peer"}})
        }).collect();
        json!({"enabled":true,"trust":"same_tailscale_owner","device_id":self.settings.device_id,"fresh":fresh(&inner),"checked_at":inner.checked_at,"error":inner.error,"peers":peers})
    }
    fn save(&self, records: &BTreeMap<String, Record>) -> Result<()> {
        let saved = Saved {
            version: 1,
            device_id: self.settings.device_id.clone(),
            tailnet_id: self.settings.tailnet_id.clone(),
            owner_user_id: self.settings.owner_user_id,
            peers: records.clone(),
        };
        atomic_private(&self.path, &serde_json::to_vec_pretty(&saved)?)
    }
    fn commit(
        &self,
        inner: &mut Inner,
        remote: &Candidate,
        node_id: String,
        inbound: String,
        outbound: String,
    ) -> Result<()> {
        ensure!(
            trusted(inner, &remote.id)?.url == remote.url,
            "identity changed during enrollment"
        );
        ensure!(
            valid_name(&node_id)
                && node_id != self.config.node_id
                && valid_secret(&inbound)
                && valid_secret(&outbound),
            "invalid enrollment identity or credential"
        );
        ensure!(
            self.config
                .peers
                .iter()
                .all(|p| p.node_id != node_id || p.url.trim_end_matches('/') == remote.url),
            "node ID conflicts with a manual peer"
        );
        ensure!(
            inner
                .records
                .iter()
                .all(|(id, r)| id == &remote.id || r.node_id != node_id),
            "duplicate node ID"
        );
        ensure!(
            inner
                .records
                .get(&remote.id)
                .is_none_or(|r| r.node_id == node_id),
            "enrolled device changed its Citta node ID"
        );
        let all_names: std::collections::HashSet<_> = self
            .config
            .peers
            .iter()
            .map(|p| p.node_id.as_str())
            .chain(inner.records.values().map(|r| r.node_id.as_str()))
            .chain(std::iter::once(node_id.as_str()))
            .collect();
        ensure!(all_names.len() <= LIMIT, "maximum 64 combined peers");
        let mut updated = inner.records.clone();
        updated.insert(
            remote.id.clone(),
            Record {
                node_id,
                url: remote.url.clone(),
                inbound,
                outbound,
            },
        );
        ensure!(updated.len() <= LIMIT, "too many enrolled peers");
        if updated != inner.records {
            self.save(&updated)?;
            inner.records = updated;
        }
        Ok(())
    }
    fn issue(inner: &mut Inner, id: &str) -> String {
        inner.issued.entry(id.into()).or_insert_with(secret).clone()
    }
    async fn refresh(&self) -> Result<()> {
        let result = status(&self.settings.tailscale_bin).await.and_then(|s| {
            Ok((
                candidates(&s, &self.settings)?,
                candidate(&s.this, &s.current_tailnet.suffix, &self.settings)?.url,
            ))
        });
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner.checked_at = Some(now());
        match result {
            Ok((candidates, public_url)) => {
                inner.public_url = Some(public_url);
                // Removal invalidates persisted automatic credentials, including after restart.
                let kept: BTreeMap<_, _> = inner
                    .records
                    .iter()
                    .filter(|(id, r)| candidates.get(*id).is_some_and(|c| c.url == r.url))
                    .map(|(id, r)| (id.clone(), r.clone()))
                    .collect();
                if kept.len() != inner.records.len() {
                    if let Err(e) = self.save(&kept) {
                        inner.refreshed = None;
                        inner.error = Some("state_write_failed");
                        return Err(e);
                    }
                    inner.records = kept;
                }
                inner.issued.retain(|id, _| candidates.contains_key(id));
                inner.pending.retain(|id, p| {
                    p.at.elapsed() < FRESH && candidates.get(id) == Some(&p.candidate)
                });
                inner
                    .last_callback
                    .retain(|id, _| candidates.contains_key(id));
                inner.candidates = candidates;
                inner.refreshed = Some(Instant::now());
                inner.error = None;
                Ok(())
            }
            Err(error) => {
                inner.refreshed = None;
                inner.pending.clear();
                inner.error = Some("tailscale_unavailable_or_identity_changed");
                Err(error)
            }
        }
    }
    async fn pair(&self, remote: Candidate) -> Result<()> {
        let challenge = secret();
        {
            let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            trusted(&inner, &remote.id)?;
            inner.pending.insert(
                remote.id.clone(),
                Pending {
                    challenge: challenge.clone(),
                    candidate: remote.clone(),
                    at: Instant::now(),
                },
            );
        }
        let result: Result<Value> = exchange(
            &remote,
            if self.control {
                "/v1/control/enroll/start"
            } else {
                "/v1/enroll/start"
            },
            &Start {
                device_id: self.settings.device_id.clone(),
                challenge,
            },
        )
        .await;
        self.inner
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .pending
            .remove(&remote.id);
        result?;
        Ok(())
    }
    pub async fn run(self: Arc<Self>, stop: CancellationToken) {
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut offset = 0;
        loop {
            tokio::select! { _=stop.cancelled()=>return, _=interval.tick()=>{} }
            if self.refresh().await.is_err() {
                continue;
            }
            let mut remotes: Vec<_> = {
                let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
                inner
                    .candidates
                    .values()
                    .filter(|c| c.online)
                    .cloned()
                    .collect()
            };
            if !remotes.is_empty() {
                let count = remotes.len();
                remotes.rotate_left(offset % count);
                offset = (offset + 8) % count;
                remotes.truncate(8);
            }
            // Refresh credentials without restarting the daemon; failures are visible as unreachable
            // peers in discovery. Never log HTTP bodies or nonce/token-bearing errors.
            let work = stream::iter(remotes).for_each_concurrent(4, |remote| async {
                let _ = self.pair(remote).await;
            });
            tokio::select! { _=stop.cancelled()=>return, _=work=>{} }
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Start {
    device_id: String,
    challenge: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Confirm {
    device_id: String,
    challenge: String,
    node_id: String,
    token: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Confirmed {
    node_id: String,
    token: String,
}
async fn exchange<T: DeserializeOwned>(
    remote: &Candidate,
    path: &str,
    body: &impl Serialize,
) -> Result<T> {
    let url = reqwest::Url::parse(&remote.url)?;
    let host = url.host_str().context("missing host")?;
    ensure!(url.scheme() == "https", "enrollment requires HTTPS");
    let builder = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(2))
        .timeout(Duration::from_secs(6))
        .resolve(
            host,
            SocketAddr::new(remote.address, url.port_or_known_default().context("port")?),
        );
    #[cfg(test)]
    let builder = match &remote.test_ca {
        Some(cert) => builder.tls_certs_only([reqwest::Certificate::from_der(cert)?]),
        None => builder,
    };
    let client = builder.build()?;
    let response = client
        .post(format!("{}{path}", remote.url))
        .json(body)
        .send()
        .await?;
    ensure!(response.status().is_success(), "enrollment peer not ready");
    let mut bytes = Vec::new();
    let mut chunks = response.bytes_stream();
    while let Some(chunk) = chunks.next().await {
        let chunk = chunk?;
        ensure!(
            bytes.len() + chunk.len() <= MAX_PAIR_BODY,
            "enrollment response too large"
        );
        bytes.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&bytes).context("invalid enrollment reply")
}
pub fn router(enrollment: Arc<Enrollment>) -> Router {
    Router::new()
        .route(
            if enrollment.control {
                "/v1/control/enroll/start"
            } else {
                "/v1/enroll/start"
            },
            post(start),
        )
        .route(
            if enrollment.control {
                "/v1/control/enroll/confirm"
            } else {
                "/v1/enroll/confirm"
            },
            post(confirm),
        )
        .layer(DefaultBodyLimit::max(MAX_PAIR_BODY))
        .with_state(enrollment)
}
async fn start(
    State(enrollment): State<Arc<Enrollment>>,
    headers: HeaderMap,
    Json(request): Json<Start>,
) -> Result<Json<Value>, StatusCode> {
    if headers.contains_key("origin")
        || !valid_name(&request.device_id)
        || !valid_secret(&request.challenge)
    {
        return Err(StatusCode::FORBIDDEN);
    }
    let _permit = enrollment
        .callbacks
        .try_acquire()
        .map_err(|_| StatusCode::TOO_MANY_REQUESTS)?;
    let (remote, token) = {
        let mut inner = enrollment.inner.lock().unwrap_or_else(|e| e.into_inner());
        let remote = trusted(&inner, &request.device_id)
            .map_err(|_| StatusCode::FORBIDDEN)?
            .clone();
        if inner
            .last_callback
            .get(&remote.id)
            .is_some_and(|at| at.elapsed() < Duration::from_secs(3))
        {
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }
        inner
            .last_callback
            .insert(remote.id.clone(), Instant::now());
        let token = Enrollment::issue(&mut inner, &remote.id);
        (remote, token)
    };
    let response: Confirmed = exchange(
        &remote,
        if enrollment.control {
            "/v1/control/enroll/confirm"
        } else {
            "/v1/enroll/confirm"
        },
        &Confirm {
            device_id: enrollment.settings.device_id.clone(),
            challenge: request.challenge,
            node_id: enrollment.config.node_id.clone(),
            token: token.clone(),
        },
    )
    .await
    .map_err(|_| StatusCode::BAD_GATEWAY)?;
    let mut inner = enrollment.inner.lock().unwrap_or_else(|e| e.into_inner());
    enrollment
        .commit(&mut inner, &remote, response.node_id, token, response.token)
        .map_err(|_| StatusCode::CONFLICT)?;
    Ok(Json(json!({"ok":true})))
}
async fn confirm(
    State(enrollment): State<Arc<Enrollment>>,
    headers: HeaderMap,
    Json(request): Json<Confirm>,
) -> Result<Json<Confirmed>, StatusCode> {
    if headers.contains_key("origin") {
        return Err(StatusCode::FORBIDDEN);
    }
    let mut inner = enrollment.inner.lock().unwrap_or_else(|e| e.into_inner());
    let pending = inner
        .pending
        .get(&request.device_id)
        .ok_or(StatusCode::FORBIDDEN)?;
    if pending.at.elapsed() >= FRESH
        || !bool::from(
            request
                .challenge
                .as_bytes()
                .ct_eq(pending.challenge.as_bytes()),
        )
    {
        return Err(StatusCode::FORBIDDEN);
    }
    let remote = pending.candidate.clone();
    inner.pending.remove(&request.device_id); // One use, including malformed confirmations.
    let token = Enrollment::issue(&mut inner, &remote.id);
    enrollment
        .commit(
            &mut inner,
            &remote,
            request.node_id,
            token.clone(),
            request.token,
        )
        .map_err(|_| StatusCode::CONFLICT)?;
    Ok(Json(Confirmed {
        node_id: enrollment.config.node_id.clone(),
        token,
    }))
}

pub fn atomic_private(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().context("file needs parent")?;
    config::private_dir(parent)?;
    let temp = parent.join(format!(".citta-{}.tmp", uuid::Uuid::new_v4()));
    let result = (|| {
        config::write_new_private(&temp, bytes)?;
        std::fs::rename(&temp, path)?;
        std::fs::File::open(parent)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    result
}

/// Agent-facing one-time enablement. The daemon does every subsequent pairing step.
pub async fn enable(path: &Path, bin: PathBuf, port: u16) -> Result<Value> {
    let mut config = Config::load(path)?;
    ensure!(
        bin.is_absolute() && bin.is_file(),
        "Tailscale executable required"
    );
    let state = status(&bin).await?;
    let settings = Settings {
        tailscale_bin: bin.clone(),
        owner_user_id: state.this.user_id,
        tailnet_id: state.current_tailnet.stable_id.clone(),
        device_id: state.this.id.clone(),
        https_port: port,
    };
    settings.validate()?;
    candidates(&state, &settings)?;
    if let Some(old) = &config.enrollment {
        ensure!(
            old.owner_user_id == settings.owner_user_id
                && old.tailnet_id == settings.tailnet_id
                && old.device_id == settings.device_id
                && old.https_port == settings.https_port,
            "existing enrollment identity differs; refusing to repin silently"
        );
    }
    let local = candidate(&state.this, &state.current_tailnet.suffix, &settings)?;
    let current: Value =
        serde_json::from_slice(&tailscale(&bin, &["serve", "status", "--json"]).await?)?;
    ensure_serve(&current, &local.url, &config.local_url(), port)?;
    // Reuse a matching route, otherwise add just this unused port. Never enable Funnel.
    if current
        .get("TCP")
        .and_then(|v| v.get(port.to_string()))
        .is_none()
    {
        tailscale(
            &bin,
            &[
                "serve",
                "--bg",
                &format!("--https={port}"),
                &config.local_url(),
            ],
        )
        .await?;
    }
    config.enrollment = Some(settings);
    config.validate()?;
    atomic_private(path, toml::to_string_pretty(&config)?.as_bytes())?;
    Ok(
        json!({"enabled":true,"url":local.url,"trust":"same_tailscale_owner","next":"Start or restart the daemon. It discovers and pairs devices automatically."}),
    )
}
fn ensure_serve(current: &Value, url: &str, backend: &str, port: u16) -> Result<()> {
    let host = url.strip_prefix("https://").context("HTTPS origin")?;
    ensure!(
        !current
            .get("AllowFunnel")
            .and_then(|v| v.get(host))
            .and_then(Value::as_bool)
            .unwrap_or(false),
        "Citta port has Funnel enabled; refusing enrollment"
    );
    let key = port.to_string();
    if let Some(tcp) = current.get("TCP").and_then(|v| v.get(&key)) {
        ensure!(
            tcp.get("HTTPS").and_then(Value::as_bool) == Some(true),
            "Serve port is already in use"
        );
        let handlers = current
            .get("Web")
            .and_then(|v| v.get(host))
            .and_then(|v| v.get("Handlers"))
            .and_then(Value::as_object)
            .context("Serve port is already in use")?;
        ensure!(
            handlers.len() == 1
                && handlers
                    .get("/")
                    .and_then(|v| v.get("Proxy"))
                    .and_then(Value::as_str)
                    == Some(backend),
            "Serve port has a different backend; preserving it"
        );
    } else {
        ensure!(
            current
                .get("Web")
                .and_then(Value::as_object)
                .is_none_or(|web| web.keys().all(|host| !host.ends_with(&format!(":{port}")))),
            "Serve port is already configured"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests;
