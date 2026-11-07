use crate::model::valid_name;
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashSet},
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub node_id: String,
    pub listen: SocketAddr,
    pub data_dir: PathBuf,
    pub token_file: PathBuf,
    pub slots: usize,
    #[serde(default = "lease_default")]
    pub lease_secs: u64,
    #[serde(default = "queue_default")]
    pub max_pending: usize,
    #[serde(default = "history_default")]
    pub max_history: usize,
    #[serde(default)]
    pub peers: Vec<Peer>,
    #[serde(default)]
    pub capabilities: Vec<Capability>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enrollment: Option<crate::enrollment::Settings>,
}
fn lease_default() -> u64 {
    30
}
fn queue_default() -> usize {
    1000
}
fn history_default() -> usize {
    10000
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Peer {
    pub node_id: String,
    pub url: String,
    pub token_file: PathBuf,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Capability {
    pub name: String,
    pub description: String,
    pub program: PathBuf,
    #[serde(default)]
    pub args: Vec<String>,
    pub slots: usize,
    pub max_timeout_secs: u64,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}
impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let config: Self = toml::from_str(&fs::read_to_string(path).context("read config")?)
            .context("parse config")?;
        config.validate()?;
        Ok(config)
    }
    pub fn validate(&self) -> Result<()> {
        if let Some(settings) = &self.enrollment {
            settings.validate()?;
        }
        ensure!(valid_name(&self.node_id), "invalid node_id");
        ensure!(
            self.listen.ip().is_loopback() && self.listen.port() != 0,
            "listen must be a loopback address with a nonzero port; use Tailscale Serve for remote access"
        );
        ensure!(
            self.data_dir.is_absolute() && self.token_file.is_absolute(),
            "data_dir and token_file must be absolute"
        );
        ensure!((1..=256).contains(&self.slots), "slots must be 1..256");
        ensure!(
            (6..=300).contains(&self.lease_secs),
            "lease_secs must be 6..300"
        );
        ensure!(
            (1..=10000).contains(&self.max_pending),
            "max_pending must be 1..10000"
        );
        ensure!(
            (self.max_pending..=100000).contains(&self.max_history),
            "max_history must be >= max_pending and <= 100000"
        );
        ensure!(
            self.peers.len() <= 64 && self.capabilities.len() <= 64,
            "maximum 64 peers/capabilities"
        );
        let mut names = HashSet::new();
        for peer in &self.peers {
            ensure!(
                valid_name(&peer.node_id)
                    && peer.node_id != self.node_id
                    && names.insert(&peer.node_id),
                "duplicate or invalid peer node_id"
            );
            validate_url(&peer.url)?;
            ensure!(
                peer.token_file.is_absolute(),
                "peer token_file must be absolute"
            );
        }
        names.clear();
        for cap in &self.capabilities {
            ensure!(
                valid_name(&cap.name) && names.insert(&cap.name),
                "duplicate or invalid capability"
            );
            ensure!(
                cap.description.len() <= 2048,
                "capability description too long"
            );
            ensure!(
                cap.program.is_absolute() && cap.program.is_file(),
                "capability program must be an existing absolute file: {}",
                cap.name
            );
            ensure!(
                (1..=self.slots).contains(&cap.slots),
                "capability slots must be 1..node slots"
            );
            ensure!(
                (1..=86400).contains(&cap.max_timeout_secs),
                "invalid capability timeout"
            );
        }
        Ok(())
    }
    pub fn local_url(&self) -> String {
        format!("http://{}", self.listen)
    }
}
pub fn validate_url(raw: &str) -> Result<()> {
    let url = reqwest::Url::parse(raw)?;
    ensure!(
        url.username().is_empty()
            && url.password().is_none()
            && url.query().is_none()
            && url.fragment().is_none()
            && url.path() == "/",
        "peer URL must be an origin without credentials, query, or path"
    );
    let local = url
        .host_str()
        .and_then(|s| s.trim_matches(['[', ']']).parse::<std::net::IpAddr>().ok())
        .is_some_and(|ip| ip.is_loopback());
    ensure!(
        url.scheme() == "https" || (url.scheme() == "http" && local),
        "remote peers require HTTPS; HTTP is allowed only for numeric loopback addresses"
    );
    Ok(())
}
pub fn read_token(path: &Path) -> Result<String> {
    let metadata = fs::symlink_metadata(path).context("read token metadata")?;
    ensure!(
        metadata.is_file() && metadata.len() <= 1024,
        "token must be a regular file <= 1024 bytes"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        ensure!(
            metadata.permissions().mode() & 0o077 == 0,
            "token permissions must be 0600 or stricter: {}",
            path.display()
        );
    }
    let token = fs::read_to_string(path)?.trim().to_string();
    ensure!(
        token.len() >= 32
            && token.len() <= 256
            && token
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b)),
        "token must be 32..256 URL-safe characters"
    );
    Ok(token)
}
pub fn private_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}
pub fn write_new_private(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}
pub fn init(dir: &Path, node_id: String, port: u16) -> Result<PathBuf> {
    ensure!(valid_name(&node_id), "invalid node id");
    ensure!(port != 0, "port must be nonzero");
    private_dir(dir)?;
    let dir = fs::canonicalize(dir)?;
    let config_path = dir.join("config.toml");
    if config_path.exists() || dir.join("node.token").exists() {
        bail!("configuration or token already exists; refusing to overwrite");
    }
    let config = Config {
        node_id,
        listen: ([127, 0, 0, 1], port).into(),
        data_dir: dir.join("data"),
        token_file: dir.join("node.token"),
        slots: 2,
        lease_secs: 30,
        max_pending: 1000,
        max_history: 10000,
        peers: vec![],
        enrollment: None,
        capabilities: vec![Capability {
            name: "echo".into(),
            description: "Return the supplied JSON; use to validate cross-device connectivity."
                .into(),
            program: std::env::current_exe()?,
            args: vec!["builtin".into(), "echo".into()],
            slots: 2,
            max_timeout_secs: 30,
            env: BTreeMap::new(),
        }],
    };
    config.validate()?;
    let token = format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    );
    write_new_private(&config.token_file, token.as_bytes())?;
    write_new_private(&config_path, toml::to_string_pretty(&config)?.as_bytes())?;
    Ok(config_path)
}
