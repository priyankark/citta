//! Device-local, repeatable onboarding. Tailscale provides pairing, never an
//! implicit remote installation channel. No SSH, passwords or pairing bundles.
use crate::{
    client::Client,
    config::{self, Capability, Config},
    model::{CapabilityKind, NodeInfo},
    resources::Estimate,
};
use anyhow::{Context, Result, bail, ensure};
use fs2::FileExt;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};

#[derive(Clone, Copy, clap::ValueEnum, Default)]
pub enum AgentAccess {
    #[default]
    Full,
    ReadOnly,
}
impl AgentAccess {
    fn profile(self) -> &'static str {
        match self {
            Self::Full => "codex",
            Self::ReadOnly => "codex-readonly",
        }
    }
    fn sandbox(self) -> &'static str {
        match self {
            Self::Full => "danger-full-access",
            Self::ReadOnly => "read-only",
        }
    }
}

#[derive(clap::Args)]
pub struct Options {
    #[arg(long)]
    pub node: Option<String>,
    #[arg(long)]
    pub tailscale_bin: Option<PathBuf>,
    #[arg(long)]
    pub codex_bin: Option<PathBuf>,
    #[arg(long)]
    pub model: Option<String>,
    /// New profiles default to full tools. Explicit selection adds a missing
    /// profile alongside existing profiles without silently changing them.
    #[arg(long, value_enum, conflicts_with = "no_agent")]
    pub agent_access: Option<AgentAccess>,
    /// Codex config and skills directory; defaults to CODEX_HOME or ~/.codex.
    #[arg(long)]
    pub harness_dir: Option<PathBuf>,
    /// Configure pairing/tools without registering a new agent profile.
    #[arg(long)]
    pub no_agent: bool,
    /// For externally managed daemons; do not install or restart a user service.
    #[arg(long)]
    pub no_service: bool,
    /// Show prerequisites and proposed actions without writing or starting anything.
    #[arg(long)]
    pub dry_run: bool,
    #[arg(long, default_value_t = 7331)]
    pub port: u16,
    #[arg(long, default_value_t = 7332)]
    pub https_port: u16,
}

pub fn default_config() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default()
        .join(".config/citta/config.toml")
}
fn executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.is_absolute()
        && fs::metadata(path).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
}
fn find_program(name: &str, extra: &[PathBuf]) -> Option<PathBuf> {
    extra
        .iter()
        .cloned()
        .chain(
            std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
                .map(|p| p.join(name)),
        )
        .find(|p| executable(p))
}
async fn probe(path: &Path, args: &[&str], harness: &Path) -> Result<std::process::Output> {
    let mut command = tokio::process::Command::new(path);
    command
        .args(args)
        .env("CODEX_HOME", harness)
        .stdin(Stdio::null())
        .kill_on_drop(true);
    tokio::time::timeout(Duration::from_secs(10), command.output())
        .await
        .context("harness probe timed out")?
        .context("harness probe failed")
}
fn read_toml(path: &Path) -> Result<toml::Value> {
    if !path.exists() {
        return Ok(toml::Value::Table(Default::default()));
    }
    toml::from_str(&fs::read_to_string(path)?)
        .context("invalid existing harness config; refusing to overwrite")
}
fn configured_model(config: &toml::Value) -> Option<String> {
    config
        .get("profile")
        .and_then(toml::Value::as_str)
        .and_then(|profile| config.get("profiles")?.get(profile)?.get("model")?.as_str())
        .or_else(|| config.get("model").and_then(toml::Value::as_str))
        .map(str::to_owned)
}
fn mcp_config(mut value: toml::Value, binary: &Path, config: &Path) -> Result<toml::Value> {
    let table = value
        .as_table_mut()
        .context("harness config must be a table")?;
    let servers = table
        .entry("mcp_servers")
        .or_insert_with(|| toml::Value::Table(Default::default()))
        .as_table_mut()
        .context("mcp_servers must be a table")?;
    let server = servers
        .entry("citta")
        .or_insert_with(|| toml::Value::Table(Default::default()))
        .as_table_mut()
        .context("Citta MCP config must be a table")?;
    server.insert("command".into(), binary.to_string_lossy().as_ref().into());
    server.insert(
        "args".into(),
        toml::Value::Array(vec![
            "--config".into(),
            config.to_string_lossy().as_ref().into(),
            "mcp".into(),
        ]),
    );
    Ok(value)
}
fn save(path: &Path, bytes: &[u8], backup: &Path, label: &str) -> Result<bool> {
    if path.exists() {
        let old = fs::read(path)?;
        if old == bytes {
            return Ok(false);
        }
        config::private_dir(backup)?;
        config::write_new_private(&backup.join(label), &old)?;
    }
    crate::enrollment::atomic_private(path, bytes)?;
    Ok(true)
}
fn xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
fn systemd_quote(s: &str) -> Result<String> {
    ensure!(
        !s.contains(['\n', '\r', '\0']),
        "service path contains control characters"
    );
    Ok(format!(
        "\"{}\"",
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('%', "%%")
            .replace('$', "$$")
    ))
}
fn service_definition(binary: &Path, config: &Path, logs: &Path, mac: bool) -> Result<String> {
    let binary = binary.to_string_lossy();
    let config = config.to_string_lossy();
    if mac {
        Ok(format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?><!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\"><plist version=\"1.0\"><dict><key>Label</key><string>io.citta.daemon</string><key>ProgramArguments</key><array><string>{}</string><string>--config</string><string>{}</string><string>daemon</string></array><key>RunAtLoad</key><true/><key>KeepAlive</key><dict><key>SuccessfulExit</key><false/></dict><key>ThrottleInterval</key><integer>10</integer><key>ExitTimeOut</key><integer>30</integer><key>StandardOutPath</key><string>{}</string><key>StandardErrorPath</key><string>{}</string><key>EnvironmentVariables</key><dict><key>RUST_LOG</key><string>citta=info</string></dict></dict></plist>\n",
            xml(&binary),
            xml(&config),
            xml(&logs.join("stdout.log").to_string_lossy()),
            xml(&logs.join("stderr.log").to_string_lossy())
        ))
    } else {
        Ok(format!(
            "[Unit]\nDescription=Citta cross-device agent daemon\nAfter=network-online.target\nWants=network-online.target\n[Service]\nType=simple\nExecStart={} --config {} daemon\nRestart=on-failure\nRestartSec=5\nTimeoutStopSec=30\nKillMode=control-group\nUMask=0077\nNoNewPrivileges=true\nEnvironment=RUST_LOG=citta=info\n[Install]\nWantedBy=default.target\n",
            systemd_quote(&binary)?,
            systemd_quote(&config)?
        ))
    }
}
fn service_path(home: &Path) -> PathBuf {
    home.join(if cfg!(target_os = "macos") {
        "Library/LaunchAgents/io.citta.daemon.plist"
    } else {
        ".config/systemd/user/citta.service"
    })
}
fn check_service(path: &Path, binary: &Path, config: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    if cfg!(target_os = "macos") {
        let output = Command::new("/usr/bin/plutil")
            .args(["-extract", "ProgramArguments", "json", "-o", "-"])
            .arg(path)
            .output()?;
        ensure!(
            output.status.success(),
            "cannot inspect existing Citta service"
        );
        let args: Vec<String> = serde_json::from_slice(&output.stdout)?;
        ensure!(
            args == [
                binary.to_string_lossy(),
                "--config".into(),
                config.to_string_lossy(),
                "daemon".into()
            ],
            "existing service uses another binary/config; upgrade that installation first rather than replacing it"
        );
    } else {
        let wanted = format!(
            "ExecStart={} --config {} daemon",
            systemd_quote(&binary.to_string_lossy())?,
            systemd_quote(&config.to_string_lossy())?
        );
        ensure!(
            fs::read_to_string(path)?.lines().any(|line| line == wanted),
            "existing service uses another binary/config; preserve and upgrade that installation first"
        );
    }
    Ok(())
}
fn service_running() -> bool {
    if cfg!(target_os = "macos") {
        Command::new("launchctl")
            .args([
                "print",
                &format!("gui/{}/io.citta.daemon", unsafe { libc::getuid() }),
            ])
            .output()
            .is_ok_and(|r| r.status.success())
    } else {
        Command::new("systemctl")
            .args(["--user", "is-active", "--quiet", "citta.service"])
            .status()
            .is_ok_and(|r| r.success())
    }
}
async fn stop_service() -> Result<()> {
    let result = if cfg!(target_os = "macos") {
        Command::new("launchctl")
            .args([
                "bootout",
                &format!("gui/{}/io.citta.daemon", unsafe { libc::getuid() }),
            ])
            .output()?
    } else {
        Command::new("systemctl")
            .args(["--user", "stop", "citta.service"])
            .output()?
    };
    ensure!(
        result.status.success(),
        "could not stop existing Citta service"
    );
    for _ in 0..60 {
        if !service_running() {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    bail!("existing Citta service has not stopped; no configuration changed")
}
fn start_service(path: &Path) -> Result<()> {
    let result = if cfg!(target_os = "macos") {
        Command::new("launchctl")
            .args(["bootstrap", &format!("gui/{}", unsafe { libc::getuid() })])
            .arg(path)
            .output()?
    } else {
        ensure!(
            Command::new("systemctl")
                .args(["--user", "daemon-reload"])
                .status()?
                .success(),
            "systemd reload failed"
        );
        Command::new("systemctl")
            .args(["--user", "enable", "--now", "citta.service"])
            .output()?
    };
    ensure!(
        result.status.success(),
        "could not start Citta user service; inspect its local logs"
    );
    Ok(())
}

// If a write fails after stopping a previously running service, restore its
// availability. The atomic config write leaves either the old or new valid file.
struct RestartOnError(Option<PathBuf>);
impl Drop for RestartOnError {
    fn drop(&mut self) {
        if let Some(path) = &self.0
            && let Err(error) = start_service(path)
        {
            eprintln!("Citta service recovery failed: {error}; rerun citta setup");
        }
    }
}

fn ensure_idle(config: &Config) -> Result<()> {
    let db = rusqlite::Connection::open_with_flags(
        config.data_dir.join("citta.db"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .context("cannot verify origin work; existing service left running")?;
    db.busy_timeout(Duration::from_secs(5))?;
    let pending: i64 = db.query_row(
        "SELECT count(*) FROM tasks WHERE status IN ('queued','running')",
        [],
        |r| r.get(0),
    )?;
    ensure!(
        pending == 0,
        "origin has pending work; rerun setup after it completes"
    );
    Ok(())
}

fn permission_guide(binary: &Path, service_binary: Option<&Path>, mac: bool) -> Value {
    let desktop_checks = if mac {
        json!([
            {"permission":"Accessibility / Device Control Data Access","needed_for":"Clicking, typing and controlling apps","action":"In System Settings → Privacy & Security, grant access to the actual Citta service executable. Use service_executable when reported; otherwise inspect the installed service definition. A Terminal grant may not cover the background worker."},
            {"permission":"Screen & System Audio Recording","needed_for":"Seeing the desktop or taking screenshots","action":"Grant screen access to the Citta service executable. Verify an actual screenshot from a delegated worker."},
            {"permission":"Automation","needed_for":"Controlling another app through Apple Events, when the chosen tool uses them","action":"Allow the specific app control prompt for the requested workflow. This is separate from Accessibility."}
        ])
    } else {
        json!([
            {"permission":"Desktop session access","needed_for":"Clicking, typing or capturing the desktop","action":"Use the desktop session or portal supported by your automation tool. A background user service may not have access; verify from a delegated worker."}
        ])
    };
    json!({
        "execution": {
            "full_tool_access":"Full-tool profiles run shell commands, edit files and use network access as your device account, without per-command approval. Installed tools, MCP servers and skills remain available.",
            "owner_policy":"Citta's Allowed capabilities, allowed origins and Pause controls still apply. Setup preserves existing owner choices.",
            "trust":"Automatic pairing trusts eligible devices owned by the same pinned Tailscale account. Install Citta once on each device."
        },
        "desktop": {
            "status":"unverified",
            "needed_for":"Desktop interaction only; shell and ordinary code tasks do not require desktop grants.",
            "invoked_executable":binary,
            "service_executable":service_binary,
            "checks":desktop_checks,
            "verification":"After a permission change, restart the idle Citta service. Test input in an owned window and inspect an actual screenshot from a delegated worker. Setup does not grant or verify OS permissions."
        },
        "accounts": {
            "status":"task_specific",
            "action":"Sign in to each required app or account on the receiving device and enable its required tool permissions. Before an email test, verify the sending account and recipient; verify Sent after sending and inspect an uncertain result before retrying."
        }
    })
}

fn setup_next_steps(
    agent_state: &str,
    no_agent: bool,
    daemon_reachable: bool,
    pending_grants: bool,
    mcp_disabled: bool,
) -> Vec<&'static str> {
    let mut steps = Vec::new();
    if !no_agent {
        match agent_state {
            "harness_not_installed" => steps.push(
                "Install Codex on this device, sign in, then rerun citta setup to add an agent profile.",
            ),
            "harness_login_required" => steps.push(
                "Sign in with codex login using the configured Codex home, then rerun citta setup.",
            ),
            _ => {}
        }
    }
    if !daemon_reachable || pending_grants {
        steps.push("Start or restart the externally managed Citta service after its work finishes, then rerun citta setup to finish configuration and pending profile grants.");
    }
    if mcp_disabled {
        steps.push("Citta MCP is explicitly disabled in your harness config. Enable it there if you want this harness to discover and delegate tasks, then start a new harness session.");
    } else {
        steps.push("Start a new harness session to load the installed Citta tools and skills.");
    }
    steps.push("Run citta doctor. Install and run citta setup once on each other intended device, then verify a task executes on that device; pairing alone does not prove execution.");
    steps.push("For desktop tasks, follow permissions.desktop on the receiving device and verify input and capture from a delegated worker. For account tasks, verify the intended signed-in account and tools there.");
    steps
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct GrantAttempt {
    expected_revision: i64,
    original_policy: crate::permissions::Policy,
    policy: crate::permissions::Policy,
    names: Vec<String>,
}

fn is_false(value: &bool) -> bool {
    !value
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PendingProfiles {
    version: u32,
    profiles: Vec<Capability>,
    grant_attempt: Option<GrantAttempt>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    baselines: BTreeMap<String, i64>,
    #[serde(default, skip_serializing_if = "is_false")]
    legacy: bool,
}
impl PendingProfiles {
    fn load(path: &Path) -> Result<Self> {
        let mut pending = Self {
            version: 1,
            profiles: Vec::new(),
            grant_attempt: None,
            baselines: BTreeMap::new(),
            legacy: false,
        };
        if path.exists() {
            let value: Value = serde_json::from_slice(&fs::read(path)?)?;
            if value.get("version").is_some() {
                pending = serde_json::from_value(value)?;
                ensure!(pending.version == 1, "unsupported setup receipt version");
            } else {
                pending.legacy = true;
                pending.profiles = if value.is_array() {
                    serde_json::from_value(value)?
                } else {
                    vec![serde_json::from_value(value)?]
                };
            }
        }
        Ok(pending)
    }

    fn save(&self, path: &Path) -> Result<()> {
        if self.profiles.is_empty() && self.grant_attempt.is_none() {
            if path.exists() {
                fs::remove_file(path)?;
            }
            return Ok(());
        }
        crate::enrollment::atomic_private(path, &serde_json::to_vec(self)?)
    }

    fn retire(&mut self, names: &[String]) {
        self.profiles
            .retain(|profile| !names.contains(&profile.name));
        self.baselines.retain(|name, _| !names.contains(name));
        self.grant_attempt = None;
    }
}

fn permission_revision(config: &Config) -> Result<i64> {
    use rusqlite::OptionalExtension;
    let path = config.data_dir.join("citta.db");
    if !path.exists() {
        return Ok(0);
    }
    let db =
        rusqlite::Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    db.busy_timeout(Duration::from_secs(5))?;
    Ok(db
        .query_row(
            "SELECT revision FROM device_permissions WHERE id=1",
            [],
            |row| row.get(0),
        )
        .optional()?
        .unwrap_or(0))
}

async fn finish_profile_grants(
    config: &Config,
    live: &NodeInfo,
    path: &Path,
    pending: &mut PendingProfiles,
) -> Result<()> {
    if pending.profiles.is_empty() && pending.grant_attempt.is_none() {
        return Ok(());
    }
    let control = Client::control_local(config)?;
    let state: Value = control.get("/v1/control/permissions").await?;
    let mut snapshot: crate::permissions::Snapshot =
        serde_json::from_value(state["snapshot"].clone())?;
    let history: Vec<crate::permissions::Snapshot> =
        serde_json::from_value(state["history"].clone())?;
    if let Some(attempt) = &pending.grant_attempt {
        let applied_revision = attempt.expected_revision.checked_add(1);
        if history
            .iter()
            .any(|event| Some(event.revision) == applied_revision && event.policy == attempt.policy)
        {
            // The grant happened, even if an owner subsequently revoked it.
            // Retire the intent without changing today's owner policy.
            let names = attempt.names.clone();
            pending.retire(&names);
            pending.save(path)?;
        } else {
            ensure!(
                snapshot.revision == attempt.expected_revision
                    && snapshot.policy == attempt.original_policy,
                "pending setup grant needs manual reconciliation: policy changed or its audit proof is unavailable; existing owner choices and setup-pending-profile.json were preserved"
            );
            ensure!(
                attempt
                    .names
                    .iter()
                    .all(|name| live.capabilities.iter().any(|c| c.name == *name)),
                "pending setup grant needs manual reconciliation: an attempted profile is absent from the live daemon"
            );
            control
                .post::<Value>(
                    "/v1/control/permissions",
                    &crate::permissions::Update {
                        expected_revision: attempt.expected_revision,
                        policy: attempt.policy.clone(),
                    },
                )
                .await?;
            snapshot.revision += 1;
            snapshot.policy = attempt.policy.clone();
            let names = attempt.names.clone();
            pending.retire(&names);
            pending.save(path)?;
        }
    }
    if !pending.profiles.is_empty() {
        // A fresh daemon's initial policy can grant a profile before setup's
        // first POST. Reconcile that grant too, including later revocation.
        // Receipts without creation baselines need complete audit history.
        let previously_granted: Vec<_> = pending
            .profiles
            .iter()
            .filter(|profile| {
                let baseline = pending.baselines.get(&profile.name).copied().unwrap_or(0);
                snapshot.policy.allowed_capabilities.contains(&profile.name)
                    || history.iter().any(|event| {
                        event.revision >= baseline
                            && event.policy.allowed_capabilities.contains(&profile.name)
                    })
            })
            .map(|profile| profile.name.clone())
            .collect();
        pending.retire(&previously_granted);
        ensure!(
            pending.profiles.iter().all(|profile| {
                let baseline = pending
                    .baselines
                    .get(&profile.name)
                    .copied()
                    .unwrap_or(0)
                    .max(1);
                snapshot.revision >= baseline
                    && history.iter().any(|event| event.revision == baseline)
            }),
            "pending setup grant needs manual reconciliation: receipt has no complete audit history since profile creation; existing owner choices and setup-pending-profile.json were preserved"
        );
        pending.legacy = false;
        pending.save(path)?;
    }
    let names: Vec<_> = pending
        .profiles
        .iter()
        .filter(|profile| live.capabilities.iter().any(|c| c.name == profile.name))
        .map(|profile| profile.name.clone())
        .collect();
    if names.is_empty() {
        return Ok(());
    }
    let mut policy = snapshot.policy.clone();
    for name in &names {
        if !policy.allowed_capabilities.contains(name) {
            policy.allowed_capabilities.push(name.clone());
        }
    }
    pending.grant_attempt = Some(GrantAttempt {
        expected_revision: snapshot.revision,
        original_policy: snapshot.policy,
        policy: policy.clone(),
        names: names.clone(),
    });
    // Write intent before sending even an unchanged policy. The next revision
    // is durable evidence for recovery after a lost response or process crash.
    pending.save(path)?;
    control
        .post::<Value>(
            "/v1/control/permissions",
            &crate::permissions::Update {
                expected_revision: snapshot.revision,
                policy,
            },
        )
        .await?;
    pending.retire(&names);
    pending.save(path)
}

pub async fn run(config_path: &Path, options: Options) -> Result<Value> {
    let home = PathBuf::from(std::env::var_os("HOME").context("HOME unavailable")?);
    ensure!(
        home.is_absolute() && config_path.is_absolute(),
        "setup requires absolute home/config paths"
    );
    let binary = std::env::current_exe()?;
    let harness = options.harness_dir.unwrap_or_else(|| {
        std::env::var_os("CODEX_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".codex"))
    });
    ensure!(harness.is_absolute(), "harness directory must be absolute");
    let tailscale = options
        .tailscale_bin
        .or_else(|| {
            find_program(
                "tailscale",
                &[PathBuf::from(
                    "/Applications/Tailscale.app/Contents/MacOS/Tailscale",
                )],
            )
        })
        .context("Install and sign in to Tailscale on this device, then rerun citta setup")?;
    let discovered_node = crate::enrollment::local_node_name(&tailscale).await?;
    let existing = if config_path.exists() {
        Some(Config::load(config_path)?)
    } else {
        None
    };
    if let (Some(c), Some(node)) = (&existing, &options.node) {
        ensure!(
            c.node_id == *node,
            "existing node identity differs; refusing to rename it"
        );
    }
    let harness_config_path = harness.join("config.toml");
    let harness_config = read_toml(&harness_config_path)?;
    let codex = options.codex_bin.or_else(|| {
        find_program(
            "codex",
            &[
                home.join(".local/bin/codex"),
                home.join(".cargo/bin/codex"),
                PathBuf::from("/opt/homebrew/bin/codex"),
            ],
        )
    });
    let mut profile = None;
    let mut agent_state = "not_requested";
    let access = options.agent_access.unwrap_or_default();
    let profile_exists = existing.as_ref().is_some_and(|c| {
        c.capabilities.iter().any(|cap| {
            cap.kind == CapabilityKind::Agent
                && (options.agent_access.is_none() || cap.name == access.profile())
        })
    });
    if !options.no_agent && !profile_exists {
        agent_state = "harness_not_installed";
        if let Some(codex) = codex {
            ensure!(executable(&codex), "Codex executable is unavailable");
            let help = probe(&codex, &["--help"], &harness).await?;
            let exec_help = probe(&codex, &["exec", "--help"], &harness).await?;
            ensure!(
                help.status.success()
                    && exec_help.status.success()
                    && String::from_utf8_lossy(&exec_help.stdout).contains("--sandbox"),
                "installed Codex lacks the supported noninteractive sandbox mode"
            );
            agent_state = "harness_login_required";
            if probe(&codex, &["login", "status"], &harness)
                .await?
                .status
                .success()
            {
                let model = options.model.or_else(|| configured_model(&harness_config));
                let mut args = vec!["-a".to_string(), "never".into()];
                if String::from_utf8_lossy(&help.stdout).contains("--no-daemon") {
                    args.push("--no-daemon".into());
                }
                if let Some(model) = &model {
                    args.extend(["--model".into(), model.clone()]);
                }
                args.extend(
                    [
                        "exec",
                        "--sandbox",
                        access.sandbox(),
                        "--ephemeral",
                        "--skip-git-repo-check",
                        "--color",
                        "never",
                        "-",
                    ]
                    .map(str::to_owned),
                );
                profile = Some(Capability {
                    resources: Estimate {
                        memory_bytes: 512 * 1024 * 1024,
                        cpu_cores: 0.5,
                    },
                    harness: Some("codex".into()),
                    model,
                    name: access.profile().into(),
                    description: match access {
                        AgentAccess::Full => "Codex with full shell, filesystem and network access; installed tools, MCP servers and skills are available.",
                        AgentAccess::ReadOnly => "Explicitly selected read-only Codex profile; installed tools and skills remain available within its sandbox.",
                    }.into(),
                    kind: CapabilityKind::Agent,
                    program: binary.clone(),
                    args: vec!["builtin".into(), "prompt".into()],
                    slots: existing.as_ref().map_or(2, |c| c.slots),
                    max_timeout_secs: 86400,
                    env: BTreeMap::from([
                        ("CITTA_AGENT_PROGRAM".into(), codex.to_string_lossy().into()),
                        ("CITTA_AGENT_ARGS".into(), serde_json::to_string(&args)?),
                        ("HOME".into(), home.to_string_lossy().into()),
                        ("CODEX_HOME".into(), harness.to_string_lossy().into()),
                        (
                            "PATH".into(),
                            std::env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin".into()),
                        ),
                    ]),
                });
                agent_state = "profile_will_be_registered";
            }
        }
    } else if profile_exists {
        agent_state = "existing_profile_preserved";
    }
    let service = service_path(&home);
    if !options.no_service {
        check_service(&service, &binary, config_path)?;
    }
    let merged = mcp_config(harness_config.clone(), &binary, config_path)?;
    let mcp_disabled = merged
        .get("mcp_servers")
        .and_then(|v| v.get("citta"))
        .and_then(|v| v.get("enabled"))
        .and_then(toml::Value::as_bool)
        == Some(false);
    if options.dry_run {
        return Ok(
            json!({"dry_run":true,"config":config_path,"node_id":existing.as_ref().map(|c|c.node_id.as_str()).unwrap_or(options.node.as_deref().unwrap_or(&discovered_node)),"agent":agent_state,"mcp_explicitly_disabled":mcp_disabled,"service":service,"ssh_required":false,"permissions":permission_guide(&binary,(!options.no_service).then_some(binary.as_path()),cfg!(target_os="macos")),"execution_verified":false,"desktop_verified":false,"next_steps":["Review the permission guide, then run citta setup without --dry-run. No files, OS permissions or services were changed by this check."]}),
        );
    }
    let dir = config_path.parent().context("config directory required")?;
    config::private_dir(dir)?;
    let lock = fs::File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(dir.join("setup.lock"))?;
    lock.try_lock_exclusive()
        .context("another setup is already running")?;
    ensure!(
        existing.as_ref().map(serde_json::to_value).transpose()?
            == if config_path.exists() {
                Some(serde_json::to_value(Config::load(config_path)?)?)
            } else {
                None
            },
        "configuration changed during setup; rerun with current settings"
    );
    if existing.is_none() {
        ensure!(
            config_path.file_name().is_some_and(|v| v == "config.toml"),
            "new setup uses config.toml inside its private directory"
        );
        // Publish setup's admission defaults in the initial configuration, so
        // interruption or failed enrollment cannot leave legacy init defaults.
        config::init_for_setup(dir, options.node.unwrap_or(discovered_node), options.port)?;
    }
    let mut config = Config::load(config_path)?;
    let backup = dir
        .join("setup-backups")
        .join(uuid::Uuid::new_v4().to_string());
    let before = serde_json::to_value(&config)?;
    let receipt = dir.join("setup-pending-profile.json");
    let mut pending = PendingProfiles::load(&receipt)?;
    if let Some(profile) = profile {
        ensure!(
            !config.capabilities.iter().any(|c| c.name == profile.name),
            "selected profile name already belongs to another capability"
        );
        if let Some(saved) = pending.profiles.iter().find(|p| p.name == profile.name) {
            ensure!(
                serde_json::to_value(saved)? == serde_json::to_value(&profile)?,
                "pending setup profile was changed; inspect setup-pending-profile.json before restoring it"
            );
        } else {
            pending
                .baselines
                .insert(profile.name.clone(), permission_revision(&config)?);
            pending.profiles.push(profile.clone());
        }
        config.capabilities.push(profile);
    }
    config.validate()?;
    for profile in &pending.profiles {
        if !config.capabilities.iter().any(|c| c.name == profile.name) {
            let action = match profile.name.as_str() {
                "codex" => {
                    "If its config save was interrupted and you still want this profile, repeat setup with --agent-access full using the same model/harness options."
                }
                "codex-readonly" => {
                    "If its config save was interrupted and you still want this profile, repeat setup with --agent-access read-only using the same model/harness options."
                }
                _ => "Review the missing profile and its saved setup receipt before restoring it.",
            };
            bail!(
                "pending setup profile {} is missing from config. {action} If you intentionally removed it, reconcile setup-pending-profile.json instead; setup will not silently restore it.",
                profile.name
            );
        }
        ensure!(
            config
                .capabilities
                .iter()
                .any(|c| serde_json::to_value(c).ok() == serde_json::to_value(profile).ok()),
            "pending setup profile was changed; inspect setup-pending-profile.json before granting it"
        );
    }
    // Validate identity and Serve configuration against a staging file before
    // stopping anything or replacing the live configuration.
    let staged = dir.join(format!(".setup-{}.toml", uuid::Uuid::new_v4()));
    config::write_new_private(&staged, toml::to_string_pretty(&config)?.as_bytes())?;
    let port = config
        .enrollment
        .as_ref()
        .map_or(options.https_port, |e| e.https_port);
    let enrollment_result = crate::enrollment::enable(&staged, tailscale, port).await;
    let staged_config = if enrollment_result.is_ok() {
        Config::load(&staged)
    } else {
        Ok(config.clone())
    };
    fs::remove_file(&staged)?;
    let enrollment = enrollment_result?;
    config = staged_config?;
    let changed = before != serde_json::to_value(&config)?;
    let running = !options.no_service && service_running();
    let mut recovery = RestartOnError(None);
    if running && changed {
        ensure_idle(&config)?;
        let node: NodeInfo = Client::local(&config)?.get("/v1/node").await?;
        ensure!(
            node.active == 0,
            "worker has active work; rerun setup after it completes"
        );
        stop_service().await?;
        recovery.0 = Some(service.clone());
    }
    if !pending.profiles.is_empty() {
        pending.save(&receipt)?;
    }
    save(
        config_path,
        toml::to_string_pretty(&config)?.as_bytes(),
        &backup,
        "config.toml",
    )?;
    save(
        &harness_config_path,
        toml::to_string_pretty(&merged)?.as_bytes(),
        &backup,
        "harness-config.toml",
    )?;
    for (name, skill, metadata) in [
        (
            "citta-setup",
            include_str!("../skills/citta-setup/SKILL.md"),
            include_str!("../skills/citta-setup/agents/openai.yaml"),
        ),
        (
            "citta-delegate",
            include_str!("../skills/citta-delegate/SKILL.md"),
            include_str!("../skills/citta-delegate/agents/openai.yaml"),
        ),
        (
            "citta-desktop",
            include_str!("../skills/citta-desktop/SKILL.md"),
            include_str!("../skills/citta-desktop/agents/openai.yaml"),
        ),
    ] {
        save(
            &harness.join("skills").join(name).join("SKILL.md"),
            skill.as_bytes(),
            &backup,
            &format!("{name}.md"),
        )?;
        save(
            &harness.join("skills").join(name).join("agents/openai.yaml"),
            metadata.as_bytes(),
            &backup,
            &format!("{name}.yaml"),
        )?;
    }
    // Install only portable source/instructions. Native dependencies, compilation
    // and OS approval belong to an explicitly requested desktop task.
    for (relative, contents) in [
        (
            "references/usage.md",
            include_str!("../skills/citta-desktop/references/usage.md"),
        ),
        (
            "scripts/desktop-control.cjs",
            include_str!("../skills/citta-desktop/scripts/desktop-control.cjs"),
        ),
        (
            "scripts/desktop-context.swift",
            include_str!("../skills/citta-desktop/scripts/desktop-context.swift"),
        ),
        (
            "scripts/owned-fixture.swift",
            include_str!("../skills/citta-desktop/scripts/owned-fixture.swift"),
        ),
        (
            "scripts/desktop-session.py",
            include_str!("../skills/citta-desktop/scripts/desktop-session.py"),
        ),
        (
            "scripts/test-session.py",
            include_str!("../skills/citta-desktop/scripts/test-session.py"),
        ),
        (
            "scripts/test-control.cjs",
            include_str!("../skills/citta-desktop/scripts/test-control.cjs"),
        ),
    ] {
        save(
            &harness.join("skills/citta-desktop").join(relative),
            contents.as_bytes(),
            &backup,
            &format!("citta-desktop-{}", relative.replace('/', "-")),
        )?;
    }
    if !options.no_service {
        config::private_dir(&dir.join("logs"))?;
        if !service.exists() {
            save(
                &service,
                service_definition(
                    &binary,
                    config_path,
                    &dir.join("logs"),
                    cfg!(target_os = "macos"),
                )?
                .as_bytes(),
                &backup,
                "service",
            )?;
        }
        if !running || changed {
            start_service(&service)?;
            recovery.0 = None;
        }
        config = Config::load(config_path)?;
        let client = Client::local(&config)?;
        let mut ready = false;
        for _ in 0..40 {
            if client.get::<NodeInfo>("/v1/node").await.is_ok() {
                ready = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
        ensure!(
            ready,
            "service did not start; inspect Citta logs; configuration backups are retained"
        );
    }
    // Externally managed services can finish a pending grant after their owner
    // has restarted them. Never grant a profile absent from the live manifest.
    let live = Client::local(&config)?
        .get::<NodeInfo>("/v1/node")
        .await
        .ok();
    if let Some(live) = &live {
        finish_profile_grants(&config, live, &receipt, &mut pending).await?;
    }
    let agent_configured = config
        .capabilities
        .iter()
        .any(|c| c.kind == CapabilityKind::Agent);
    let next_steps = setup_next_steps(
        agent_state,
        options.no_agent,
        live.is_some(),
        receipt.exists(),
        mcp_disabled,
    );
    Ok(
        json!({"configured":true,"agent_configured":agent_configured,"daemon_reachable":live.is_some(),"profile_grant_pending":receipt.exists(),"config":config_path,"node_id":config.node_id,"agent":agent_state,"mcp_explicitly_disabled":mcp_disabled,"service":if options.no_service{"externally_managed"}else{"running"},"enrollment":enrollment,"ssh_required":false,"next":next_steps.join(" "),"next_steps":next_steps,"permissions":permission_guide(&binary,(!options.no_service).then_some(binary.as_path()),cfg!(target_os="macos")),"execution_verified":false,"desktop_verified":false}),
    )
}

fn summarize(node: &Value) -> Value {
    let profiles: Vec<_> = node["capabilities"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|c| c["kind"] == "agent")
        .collect();
    let policy = &node["permissions"]["policy"];
    let permitted = policy["mode"] == "allow_all"
        || profiles.iter().any(|c| {
            policy["allowed_capabilities"]
                .as_array()
                .is_some_and(|allowed| allowed.contains(&c["name"]))
        });
    let disk = serde_json::from_value::<crate::resources::DiskReadiness>(node["disk"].clone()).ok();
    let disk_reason = disk.as_ref().map_or(Some("disk_unknown"), |d| {
        d.fresh_reason(crate::model::now())
    });
    let state = if node["resource_admission_version"] != 1 {
        "upgrade_required"
    } else if let Some(reason) = disk_reason {
        if reason == "disk_low" {
            "disk_low"
        } else {
            "disk_unknown"
        }
    } else if profiles.is_empty() {
        "agent_profile_missing"
    } else if policy["mode"] == "paused" {
        "execution_paused"
    } else if !permitted {
        "agent_permission_required"
    } else {
        "agent_configured"
    };
    let action = match state {
        "disk_low" => {
            "Free space on this device’s Citta data/work filesystem or move its data using the offline procedure, then rerun doctor. No files are deleted automatically; interrupted tasks are not replayed."
        }
        "disk_unknown" => {
            "Upgrade older workers if disk telemetry is missing; otherwise check access to this device’s Citta data/work filesystem, then rerun doctor."
        }
        "agent_configured" => "inspect_placement_for_current_capacity_and_origin_permissions",
        "execution_paused" | "agent_permission_required" => {
            "review_existing_owner_policy; setup_preserves_it"
        }
        "upgrade_required" => "upgrade_installed_binary_then_run_citta_setup_locally",
        _ => "run_citta_setup_on_that_device",
    };
    json!({"node_id":node["node_id"],"state":state,"action":action,"disk":node["disk"],"execution_verified":false,"desktop_verified":false,"verification":"This checks configuration and policy only. Verify a real delegated task and any required desktop/account access on this device."})
}

pub async fn doctor(config: &Config, config_path: &Path) -> Result<Value> {
    let discovery: Value = Client::local(config)?
        .get("/v1/discover")
        .await
        .with_context(|| format!("Citta daemon is unavailable on this device (config: {}). Run citta setup with this config to restore its user service; if you use --no-service, start the existing daemon through its service manager. Inspect {} on macOS or the user service journal on Linux if it still cannot connect; do not reinitialize its identity.",config_path.display(),config_path.parent().unwrap_or(Path::new(".")).join("logs").display()))?;
    let peers:Vec<Value>=discovery["peers"].as_array().into_iter().flatten().map(|p|if p["reachable"]==true{summarize(&p["node"])}else{json!({"node_id":p["node_id"],"state":"unreachable","action":"check_device_tailscale_and_citta_service"})}).collect();
    let waiting:Vec<Value>=discovery["enrollment"]["peers"].as_array().into_iter().flatten().filter(|p|p["state"]!="paired").map(|p|json!({"device_id":p["device_id"],"online":p["online"],"state":if p["online"]==true{"citta_not_ready"}else{"device_offline"},"action":"run_citta_setup_locally_if_not_already_installed; check_same_port_and_tailnet_policy"})).collect();
    Ok(
        json!({"local":summarize(&discovery["local"]),"tailscale_identity_fresh":discovery["enrollment"]["fresh"],"peers":peers,"waiting_devices":waiting,"ssh_required":false,"permissions":permission_guide(&std::env::current_exe()?,None,cfg!(target_os="macos")),"scope":"Citta must be installed once per device. Pairing is automatic; remote software installation is not implied by Tailscale membership. The permission guide describes this device. invoked_executable is this CLI, not proof of the running service path; setup reports service_executable when it manages that service. Run setup locally on each device."}),
    )
}

pub fn prompt_worker() -> Result<i32> {
    use std::io::Read;
    let mut input = String::new();
    std::io::stdin()
        .take(crate::model::MAX_INPUT as u64 + 1)
        .read_to_string(&mut input)?;
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Prompt {
        prompt: String,
    }
    ensure!(
        input.len() <= crate::model::MAX_INPUT,
        "prompt input exceeds limit"
    );
    let input: Prompt = serde_json::from_str(&input)?;
    let program = PathBuf::from(
        std::env::var_os("CITTA_AGENT_PROGRAM").context("fixed harness program missing")?,
    );
    ensure!(executable(&program), "fixed harness program unavailable");
    let args: Vec<String> = serde_json::from_str(
        &std::env::var("CITTA_AGENT_ARGS").context("fixed harness args missing")?,
    )?;
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .context("harness stdin unavailable")?
        .write_all(input.prompt.as_bytes())?;
    Ok(child.wait()?.code().unwrap_or(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn readiness_distinguishes_legacy_missing_and_denied_profiles() {
        let mut node = json!({"node_id":"a"});
        assert_eq!(summarize(&node)["state"], "upgrade_required");
        node["resource_admission_version"] = json!(1);
        assert_eq!(summarize(&node)["state"], "disk_unknown");
        node["disk"] = serde_json::to_value(crate::resources::DiskReadiness::from_available(
            Some(0),
            Some(100),
            64,
            crate::model::now(),
        ))
        .unwrap();
        assert_eq!(summarize(&node)["state"], "disk_low");
        node["disk"] = serde_json::to_value(crate::resources::DiskReadiness::from_available(
            Some(100),
            Some(100),
            64,
            crate::model::now(),
        ))
        .unwrap();
        assert_eq!(summarize(&node)["state"], "agent_profile_missing");
        node["capabilities"] = json!([{"kind":"agent","name":"analysis"}]);
        node["permissions"] =
            json!({"policy":{"mode":"paused","allowed_capabilities":["analysis"]}});
        assert_eq!(summarize(&node)["state"], "execution_paused");
        node["permissions"]["policy"]["mode"] = json!("restricted");
        assert_eq!(summarize(&node)["state"], "agent_configured");
        node["permissions"]["policy"]["allowed_capabilities"] = json!([]);
        assert_eq!(summarize(&node)["state"], "agent_permission_required");
    }
    #[test]
    fn service_paths_do_not_expand_variables_or_break_xml() {
        assert_eq!(
            systemd_quote("/opt/100%/$bin/a b").unwrap(),
            "\"/opt/100%%/$$bin/a b\""
        );
        assert!(systemd_quote("/a\nb").is_err());
        let service = service_definition(
            Path::new("/a&b/citta"),
            Path::new("/a'b/config"),
            Path::new("/logs"),
            true,
        )
        .unwrap();
        assert!(service.contains("/a&amp;b/citta"));
        assert!(service.contains("/a&apos;b/config"));
    }
    #[test]
    fn idle_check_counts_work_beyond_recent_history_page() {
        let dir = tempfile::tempdir().unwrap();
        let path = config::init(dir.path(), "fixture".into(), 7331).unwrap();
        let config = Config::load(&path).unwrap();
        fs::create_dir_all(&config.data_dir).unwrap();
        let db = rusqlite::Connection::open(config.data_dir.join("citta.db")).unwrap();
        db.execute_batch("CREATE TABLE tasks(status TEXT); INSERT INTO tasks VALUES('running');")
            .unwrap();
        for _ in 0..110 {
            db.execute("INSERT INTO tasks VALUES('succeeded')", [])
                .unwrap();
        }
        assert!(ensure_idle(&config).is_err());
        db.execute("UPDATE tasks SET status='succeeded'", [])
            .unwrap();
        assert!(ensure_idle(&config).is_ok());
    }
}
