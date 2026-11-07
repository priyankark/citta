use anyhow::{Context, Result, ensure};
use citta::{
    api::{self, App},
    client::Client,
    config::{self, Config},
    model::*,
    store::Store,
};
use clap::{Parser, Subcommand};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio_util::sync::CancellationToken;

#[derive(Parser)]
#[command(
    version,
    about = "Cross-device capabilities and task delegation for agents"
)]
struct Cli {
    #[arg(long, global = true, default_value = "config.toml")]
    config: PathBuf,
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    /// Create a private device configuration and unique token; never overwrites.
    Init {
        #[arg(long)]
        dir: PathBuf,
        #[arg(long)]
        node: String,
        #[arg(long, default_value_t = 7331)]
        port: u16,
    },
    /// Validate configuration and credential files without starting execution.
    Check,
    /// Enable unattended pairing with devices owned by this Tailscale account.
    Enroll {
        #[arg(long)]
        tailscale_bin: PathBuf,
        #[arg(long, default_value_t = 7332)]
        https_port: u16,
    },
    /// Show automatic enrollment health, waiting devices, and paired identities.
    Enrollment,
    /// List live agent sessions, hardware resources and spawnable profiles across devices.
    Agents,
    /// Issue a short-lived login link for the private local monitoring panel.
    Panel,
    /// Spawn a bounded agent run using a receiver-installed profile.
    SpawnAgent {
        #[arg(long)]
        profile: String,
        #[arg(long)]
        prompt: PathBuf,
        #[arg(long)]
        key: String,
        #[arg(long)]
        target: Option<String>,
        #[arg(long, default_value_t = 300)]
        timeout: u64,
    },
    /// Run the daemon and configured workers.
    Daemon,
    /// Expose Citta tools over MCP stdio to an agent harness.
    Mcp,
    /// Show reachable paired devices, capabilities, and capacity.
    Discover,
    /// Submit JSON from a file, or stdin with --input -.
    Submit {
        #[arg(long)]
        capability: String,
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        key: String,
        #[arg(long)]
        target: Option<String>,
        #[arg(long, default_value_t = 300)]
        timeout: u64,
        #[arg(long, default_value_t = 3600)]
        queue_ttl: u64,
    },
    Inspect {
        id: uuid::Uuid,
    },
    Tasks {
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    Cancel {
        id: uuid::Uuid,
    },
    Events {
        #[arg(long, default_value_t = 0)]
        after: i64,
    },
    /// Delete terminal tasks older than this age. Stop the daemon first. Removes their idempotency keys and events.
    Prune {
        #[arg(long)]
        older_than_days: u32,
    },
    #[command(hide = true)]
    Guard {
        #[arg(long)]
        parent: u32,
        #[arg(long)]
        timeout: u64,
        #[arg(long)]
        program: PathBuf,
        #[arg(last = true)]
        args: Vec<String>,
    },
    #[command(hide = true)]
    Builtin {
        operation: String,
    },
}
fn print(value: impl serde::Serialize) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}
#[tokio::main(worker_threads = 2)]
async fn main() -> Result<()> {
    #[cfg(not(unix))]
    compile_error!(
        "Citta v1 supports macOS and Linux; process containment must be implemented before other targets"
    );
    let cli = Cli::parse();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "citta=info".into()),
        )
        .json()
        .with_writer(std::io::stderr)
        .init();
    match cli.command {
        Commands::Init { dir, node, port } => {
            let path = config::init(&dir, node, port)?;
            return print(
                json!({"config":path,"next":"Run citta --config <path> check, then daemon"}),
            );
        }
        Commands::Enroll {
            tailscale_bin,
            https_port,
        } => {
            return print(citta::enrollment::enable(&cli.config, tailscale_bin, https_port).await?);
        }
        Commands::Guard {
            parent,
            timeout,
            program,
            args,
        } => {
            return citta::worker::guard(parent, timeout, &program, &args);
        }
        Commands::Builtin { operation } => {
            ensure!(operation == "echo", "unknown builtin");
            let value: Value = serde_json::from_slice(&read_input(&PathBuf::from("-"))?)?;
            return print(value);
        }
        _ => {}
    }
    let config = Config::load(&cli.config)?;
    match cli.command {
        Commands::Check => {
            config::read_token(&config.token_file)?;
            Client::peers(&config)?;
            print(
                json!({"ok":true,"node_id":config.node_id,"peers":config.peers.len(),"capabilities":config.capabilities.iter().map(|c| &c.name).collect::<Vec<_>>(),"listen":config.listen}),
            )
        }
        Commands::Daemon => daemon(config).await,
        Commands::Prune { older_than_days } => {
            ensure!(older_than_days >= 1, "minimum retention is one day");
            let store = Store::open(
                &config.data_dir,
                &config.node_id,
                config.lease_secs,
                config.max_pending,
                config.max_history,
            )?;
            let deleted = store.prune(now() - i64::from(older_than_days) * 86400)?;
            print(json!({"deleted":deleted}))
        }
        command => {
            let client = Client::local(&config)?;
            let value: Value = match command {
                Commands::Mcp => return citta::mcp::serve(client).await,
                Commands::Discover => client.get("/v1/discover").await?,
                Commands::Enrollment => client.get("/v1/enrollment").await?,
                Commands::Agents => client.get("/v1/agents").await?,
                Commands::Panel => client.post("/v1/panel/session", &json!({})).await?,
                Commands::SpawnAgent {
                    profile,
                    prompt,
                    key,
                    target,
                    timeout,
                } => {
                    let prompt = String::from_utf8(read_input(&prompt)?)?;
                    client
                        .post(
                            "/v1/agents/spawn",
                            &SpawnAgent {
                                profile,
                                prompt,
                                idempotency_key: key,
                                target_node: target,
                                timeout_secs: timeout,
                                queue_ttl_secs: 3600,
                            },
                        )
                        .await?
                }
                Commands::Submit {
                    capability,
                    input,
                    key,
                    target,
                    timeout,
                    queue_ttl,
                } => {
                    let request = Submission {
                        agent_only: false,
                        capability,
                        input: serde_json::from_slice(&read_input(&input)?)?,
                        idempotency_key: key,
                        target_node: target,
                        timeout_secs: timeout,
                        queue_ttl_secs: queue_ttl,
                    };
                    request.validate()?;
                    client.post("/v1/tasks", &request).await?
                }
                Commands::Inspect { id } => client.get(&format!("/v1/tasks/{id}")).await?,
                Commands::Tasks { limit } => {
                    client
                        .get(&format!("/v1/tasks?limit={}", limit.min(100)))
                        .await?
                }
                Commands::Cancel { id } => {
                    client
                        .post(&format!("/v1/tasks/{id}/cancel"), &json!({}))
                        .await?
                }
                Commands::Events { after } => {
                    client.get(&format!("/v1/events?after={after}")).await?
                }
                _ => unreachable!(),
            };
            print(value)
        }
    }
}
fn read_input(path: &std::path::Path) -> Result<Vec<u8>> {
    use std::io::Read;
    let reader: Box<dyn Read> = if path == std::path::Path::new("-") {
        Box::new(std::io::stdin())
    } else {
        Box::new(std::fs::File::open(path)?)
    };
    let mut data = Vec::new();
    reader.take(MAX_INPUT as u64 + 1).read_to_end(&mut data)?;
    ensure!(data.len() <= MAX_INPUT, "input exceeds 64 KiB");
    Ok(data)
}
async fn daemon(config: Config) -> Result<()> {
    let token = config::read_token(&config.token_file)?;
    let peers = Client::peers(&config)?;
    let store = Store::open(
        &config.data_dir,
        &config.node_id,
        config.lease_secs,
        config.max_pending,
        config.max_history,
    )?;
    store.sweep(now())?;
    let listener = tokio::net::TcpListener::bind(config.listen)
        .await
        .context("bind daemon")?;
    let config = Arc::new(config);
    let enrollment = citta::enrollment::Enrollment::open(config.clone())?;
    let app = App {
        config,
        store: Arc::new(store),
        token: Arc::new(token),
        peers,
        enrollment,
        inventory: Default::default(),
        managed_agents: Default::default(),
        panel: Default::default(),
        activity: Arc::new(Mutex::new(BTreeMap::new())),
        shutdown: CancellationToken::new(),
    };
    let server_stop = CancellationToken::new();
    let server_token = server_stop.clone();
    let router = api::router(app.clone());
    let mut server = tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(server_token.cancelled_owned())
            .await
    });
    let mut workers = tokio::spawn(citta::worker::run(app.clone()));
    let inventory_task = tokio::spawn(citta::inventory::run(
        app.config.inventory.clone(),
        app.inventory.clone(),
        app.shutdown.clone(),
    ));
    let enrollment_task = app
        .enrollment
        .clone()
        .map(|e| tokio::spawn(e.run(app.shutdown.clone())));
    let maintenance_app = app.clone();
    let maintenance = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        loop {
            tokio::select! {
                _ = maintenance_app.shutdown.cancelled() => break,
                _ = interval.tick() => {
                    if let Err(error) = maintenance_app.db(|s| s.sweep(now())).await {
                        tracing::error!(%error, "maintenance failed; shutting down");
                        maintenance_app.shutdown.cancel();
                        break;
                    }
                }
            }
        }
    });
    tracing::info!(node=%app.config.node_id, listen=%app.config.listen, "daemon ready");
    let mut server_done = false;
    let mut workers_done = false;
    let failure: Result<()> = tokio::select! {
        signal = shutdown_signal() => signal,
        _ = app.shutdown.cancelled() => Err(anyhow::anyhow!("daemon stopped after internal failure")),
        result = &mut server => { server_done = true; result.context("HTTP server task failed").and_then(|r| r.context("HTTP server failed")).and_then(|_| Err(anyhow::anyhow!("HTTP server stopped unexpectedly"))) },
        result = &mut workers, if !app.config.capabilities.is_empty() => { workers_done = true; result.context("worker supervisor panicked").and_then(|r| r).and_then(|_| Err(anyhow::anyhow!("worker supervisor stopped unexpectedly"))) },
    };
    app.shutdown.cancel();
    if !workers_done {
        workers.await??;
    }
    maintenance.await?;
    inventory_task.await?;
    if let Some(task) = enrollment_task {
        task.await?;
    }
    server_stop.cancel();
    if !server_done {
        tokio::time::timeout(Duration::from_secs(10), server).await???;
    }
    failure
}
async fn shutdown_signal() -> Result<()> {
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    tokio::select! { result = tokio::signal::ctrl_c() => result?, _ = terminate.recv() => {} }
    Ok(())
}
