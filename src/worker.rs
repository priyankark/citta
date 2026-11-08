use crate::{api::App, client::Client, config::Capability, model::*};
use anyhow::{Context, Result, bail};
use std::{process::Stdio, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt},
    process::Command,
    sync::Semaphore,
    task::JoinSet,
};

pub async fn run(app: App) -> Result<()> {
    let global = Arc::new(Semaphore::new(app.config.slots));
    let local = Client::local(&app.config)?;
    let mut workers = JoinSet::new();
    for capability in &app.config.capabilities {
        for slot in 0..capability.slots {
            workers.spawn(worker_loop(
                app.clone(),
                capability.clone(),
                global.clone(),
                local.clone(),
                slot,
            ));
        }
    }
    while let Some(result) = workers.join_next().await {
        if let Err(error) = result {
            app.shutdown.cancel();
            while workers.join_next().await.is_some() {}
            return Err(error.into());
        }
    }
    Ok(())
}
async fn worker_loop(
    app: App,
    cap: Capability,
    global: Arc<Semaphore>,
    local: Client,
    mut next: usize,
) {
    loop {
        let permit = tokio::select! {
            _ = app.shutdown.cancelled() => return,
            permit = global.clone().acquire_owned() => match permit { Ok(p) => p, Err(_) => return },
        };
        let mut origins = vec![local.clone()];
        origins.extend(app.peer_clients());
        let origin = &origins[next % origins.len()];
        next += 1;
        if !(if origin.node_id == app.config.node_id {
            app.permissions
                .snapshot()
                .policy
                .allows_capability(&cap.name)
        } else {
            app.origin_identity(&origin.node_id)
                .is_some_and(|id| app.permissions.allows(&id, &cap.name))
        }) {
            drop(permit);
            tokio::select! {_=app.shutdown.cancelled()=>return,_=tokio::time::sleep(Duration::from_millis(200))=>{}}
            continue;
        }
        let claim_gate = tokio::select! {
            _ = app.shutdown.cancelled() => return,
            gate = app.admission.claims.lock() => gate,
        };
        let readiness = app.admission.readiness(
            &app.config.admission,
            app.inventory
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .as_ref(),
            &cap,
            None,
            now(),
        );
        if app.config.admission.guard_all && !readiness.accepting {
            drop(claim_gate);
            drop(permit);
            tokio::select! {_=app.shutdown.cancelled()=>return,_=tokio::time::sleep(Duration::from_millis(200))=>{}}
            continue;
        }
        let offer = readiness.offer.filter(|_| readiness.accepting);
        let claim = tokio::select! {
            _ = app.shutdown.cancelled() => return,
            result = claim_from(origin, &app.config.node_id, &cap.name, cap.kind == CapabilityKind::Agent, offer) => result,
        };
        let reservation = match &claim {
            Ok(Some(claim)) => Some(
                app.admission.reserve(
                    &claim.task.id,
                    cap.resources.max(
                        claim
                            .task
                            .request
                            .requirements
                            .as_ref()
                            .map_or(Default::default(), |r| r.estimate()),
                    ),
                ),
            ),
            _ => None,
        };
        drop(claim_gate);
        match claim {
            Ok(Some(claim)) => {
                // Validate a peer's response before using its identifiers as filesystem paths.
                if uuid::Uuid::parse_str(&claim.task.id).is_err()
                    || claim.task.origin_node != origin.node_id
                    || claim.task.request.capability != cap.name
                    || (claim.task.request.agent_only && cap.kind != CapabilityKind::Agent)
                    || claim.task.worker_node.as_deref() != Some(&app.config.node_id)
                    || claim.task.status != Status::Running
                    || claim.task.request.validate().is_err()
                    || !(6..=300).contains(&claim.lease_secs)
                {
                    tracing::warn!(node=%origin.node_id, "invalid claim response");
                    continue;
                }
                {
                    let mut activity = app.activity.lock().unwrap_or_else(|e| e.into_inner());
                    *activity.entry(cap.name.clone()).or_default() += 1;
                }
                tracing::info!(task=%claim.task.id, origin=%origin.node_id, capability=%cap.name, "task started");
                if cap.kind == CapabilityKind::Agent {
                    crate::inventory::started(
                        &app.managed_agents,
                        &claim.task.id,
                        &origin.node_id,
                        &cap.name,
                        cap.harness.as_deref(),
                        cap.model.as_deref(),
                    );
                }
                let outcome = if app.execution_allowed(origin, &claim.task, &cap.name) {
                    execute(&app, &cap, origin, &claim).await
                } else {
                    Outcome::error(
                        Status::Interrupted,
                        "device permissions deny this execution",
                    )
                };
                if cap.kind == CapabilityKind::Agent {
                    crate::inventory::finished(&app.managed_agents, &claim.task.id, outcome.status);
                }
                let committed = report(origin, &claim, outcome).await;
                if cap.kind == CapabilityKind::Agent
                    && let Some(status) = committed
                {
                    crate::inventory::finished(&app.managed_agents, &claim.task.id, status);
                }
                {
                    let mut activity = app.activity.lock().unwrap_or_else(|e| e.into_inner());
                    *activity.entry(cap.name.clone()).or_default() -= 1;
                }
            }
            Ok(None) => {}
            Err(_) => {
                tracing::debug!(node=%origin.node_id, "origin unavailable");
            }
        }
        drop(reservation);
        drop(permit);
        tokio::select! { _ = app.shutdown.cancelled() => return, _ = tokio::time::sleep(Duration::from_millis(200)) => {} }
    }
}
async fn claim_from(
    origin: &Client,
    worker: &str,
    capability: &str,
    agent_profile: bool,
    offer: Option<crate::resources::Offer>,
) -> Result<Option<Claim>> {
    let info: NodeInfo = origin.get("/v1/node").await?;
    if info.node_id != origin.node_id || info.protocol != 1 {
        bail!("peer identity or protocol mismatch");
    }
    origin
        .post(
            "/v1/claim",
            &ClaimRequest {
                offer: if info.resource_admission_version == Some(1) {
                    offer
                } else {
                    None
                },
                agent_profile,
                worker_node: worker.into(),
                capability: capability.into(),
            },
        )
        .await
}
async fn report(origin: &Client, claim: &Claim, outcome: Outcome) -> Option<Status> {
    let status = outcome.status;
    let completion = Completion {
        lease_token: claim.lease_token.clone(),
        outcome,
    };
    let path = format!("/v1/tasks/{}/complete", claim.task.id);
    // Completion is idempotent under the same lease; never create a new task here.
    for _ in 0..3 {
        if origin.post::<Task>(&path, &completion).await.is_ok() {
            tracing::info!(task=%claim.task.id, ?status, "task reported");
            return Some(status);
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    tracing::warn!(task=%claim.task.id, "completion unacknowledged; inspect origin before resubmitting");
    origin
        .get::<Task>(&format!("/v1/tasks/{}", claim.task.id))
        .await
        .ok()
        .filter(|task| task.status.terminal())
        .map(|task| task.status)
}
async fn execute(app: &App, cap: &Capability, origin: &Client, claim: &Claim) -> Outcome {
    // Renew before starting: a delayed claim response must not start an expired task.
    let renewal = LeaseRequest {
        lease_token: claim.lease_token.clone(),
    };
    let renew_path = format!("/v1/tasks/{}/renew", claim.task.id);
    if origin.post::<Task>(&renew_path, &renewal).await.is_err() {
        return Outcome::error(
            Status::Interrupted,
            "could not confirm lease before execution",
        );
    }
    let workspace = app
        .config
        .data_dir
        .join("work")
        .join(&origin.node_id)
        .join(&claim.task.id);
    if let Err(e) = crate::config::private_dir(&workspace) {
        return Outcome::error(Status::Failed, format!("workspace creation failed: {e}"));
    }
    let outcome = run_command(app, cap, claim, origin, &renew_path, &renewal, &workspace).await;
    if let Err(error) = tokio::fs::remove_dir_all(&workspace).await {
        tracing::warn!(%error, task=%claim.task.id, "workspace cleanup failed");
    }
    outcome
}
async fn run_command(
    app: &App,
    cap: &Capability,
    claim: &Claim,
    origin: &Client,
    renew_path: &str,
    renewal: &LeaseRequest,
    workspace: &std::path::Path,
) -> Outcome {
    let executable = match std::env::current_exe() {
        Ok(path) => path,
        Err(error) => {
            return Outcome::error(
                Status::Failed,
                format!("locate execution guardian: {error}"),
            );
        }
    };
    let mut command = Command::new(executable);
    command
        .arg("guard")
        .arg("--parent")
        .arg(std::process::id().to_string())
        .arg("--timeout")
        .arg(
            claim
                .task
                .request
                .timeout_secs
                .min(cap.max_timeout_secs)
                .to_string(),
        )
        .arg("--program")
        .arg(&cap.program)
        .arg("--")
        .args(&cap.args)
        .current_dir(workspace)
        .env_clear()
        .envs(&cap.env)
        .env("CITTA_TASK_ID", &claim.task.id)
        .env("CITTA_ORIGIN_NODE", &origin.node_id)
        .env("CITTA_WORKER_NODE", &app.config.node_id)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(false);
    #[cfg(unix)]
    command.process_group(0);
    if !app.execution_allowed(origin, &claim.task, &cap.name) {
        return Outcome::error(
            Status::Interrupted,
            "device permissions deny this execution",
        );
    }
    if app.config.admission.guard_all || claim.task.request.requirements.is_some() {
        let readiness = app.admission.readiness(
            &app.config.admission,
            app.inventory
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .as_ref(),
            cap,
            Some(&claim.task.id),
            now(),
        );
        let allowed =
            readiness.accepting
                && readiness.offer.as_ref().is_some_and(|offer| {
                    claim.task.request.requirements.as_ref().is_none_or(|req| {
                        offer.fits(req, &app.config.node_id, &origin.node_id, now())
                    })
                });
        if !allowed {
            return Outcome::error(
                Status::Interrupted,
                "resource capacity changed before execution; no process started",
            );
        }
    }
    let identity = app
        .execution_identity(origin, &claim.task)
        .unwrap_or_default();
    let (mut child, authorization) = match app
        .permissions
        .launch(&identity, &cap.name, || command.spawn().map_err(Into::into))
    {
        Ok(child) => child,
        Err(e) => return Outcome::error(Status::Failed, format!("spawn failed: {e}")),
    };
    let process_group = ProcessGroup(child.id());
    let mut stdin = child.stdin.take().expect("piped stdin");
    let stdout = child.stdout.take().expect("piped stdout");
    let stderr = child.stderr.take().expect("piped stderr");
    let input = serde_json::to_vec(&claim.task.request.input).expect("JSON input");
    let timeout = Duration::from_secs(claim.task.request.timeout_secs.min(cap.max_timeout_secs));
    let outcome = {
        let execution = async {
            let write = async {
                stdin.write_all(&input).await?;
                stdin.write_all(b"\n").await?;
                stdin.shutdown().await?;
                drop(stdin);
                Ok::<_, anyhow::Error>(())
            };
            let wait = async { child.wait().await.context("wait for worker") };
            let (_, stdout, stderr, status) =
                tokio::try_join!(write, read_bounded(stdout), read_bounded(stderr), wait)?;
            Ok::<_, anyhow::Error>(Outcome {
                status: if status.success() {
                    Status::Succeeded
                } else {
                    Status::Failed
                },
                stdout,
                stderr,
                exit_code: status.code(),
                error: if status.success() {
                    None
                } else {
                    Some("worker exited unsuccessfully".into())
                },
            })
        };
        tokio::pin!(execution);
        let deadline = tokio::time::sleep(timeout);
        tokio::pin!(deadline);
        let mut heartbeat =
            tokio::time::interval(Duration::from_secs((claim.lease_secs / 3).clamp(1, 3)));
        heartbeat.tick().await;
        loop {
            tokio::select! {
                biased;
                _ = authorization.revoked() => break Outcome::error(Status::Interrupted,"device permissions revoked during execution"),
                _ = app.shutdown.cancelled() => break Outcome::error(Status::Interrupted, "worker daemon is shutting down"),
                _ = &mut deadline => break Outcome::error(Status::Failed, "execution timeout exceeded"),
                result = &mut execution => break result.unwrap_or_else(|e| Outcome::error(Status::Failed, e.to_string())),
                _ = heartbeat.tick() => {
                    if !app.execution_allowed(origin,&claim.task,&cap.name) {break Outcome::error(Status::Interrupted,"device permissions or origin trust revoked");}
                    // Fail closed on renewal loss. Don't keep executing through a partition.
                    let renewed = tokio::select! {
                        _ = app.shutdown.cancelled() => false,
                        _ = &mut deadline => false,
                        result = origin.post::<Task>(renew_path, renewal) => result.is_ok(),
                    };
                    if !renewed { break Outcome::error(Status::Interrupted, "lease renewal failed or task cancelled"); }
                }
            }
        }
    };
    drop(process_group);
    if tokio::time::timeout(Duration::from_secs(2), child.wait())
        .await
        .is_err()
    {
        let _ = child.kill().await;
        let _ = child.wait().await;
    }
    outcome
}
async fn read_bounded(mut reader: impl AsyncRead + Unpin) -> Result<String> {
    let mut output = Vec::new();
    let mut buffer = [0u8; 8192];
    loop {
        let size = reader.read(&mut buffer).await?;
        if size == 0 {
            break;
        }
        if output.len() + size > MAX_OUTPUT {
            bail!("worker output exceeds 256 KiB per stream");
        }
        output.extend_from_slice(&buffer[..size]);
    }
    String::from_utf8(output).context("worker output must be UTF-8")
}
struct ProcessGroup(Option<u32>);
impl Drop for ProcessGroup {
    fn drop(&mut self) {
        #[cfg(unix)]
        if let Some(pid) = self.0 {
            // Ask the guardian to terminate and reap its separate job group.
            // The guardian also watches parent death and the wall-time deadline.
            unsafe {
                libc::kill(-(pid as i32), libc::SIGTERM);
            }
        }
    }
}

/// A small execution guardian survives daemon death long enough to terminate its
/// job process group. It receives no mesh credentials. Children must not escape
/// the group; arbitrary/untrusted programs require an OS sandbox or container.
static GUARD_STOP: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
extern "C" fn stop_guard(_: libc::c_int) {
    GUARD_STOP.store(true, std::sync::atomic::Ordering::Relaxed);
}

pub fn guard(parent: u32, timeout: u64, program: &std::path::Path, args: &[String]) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        anyhow::ensure!(
            unsafe { libc::getpgrp() == libc::getpid() },
            "guardian must lead its own process group"
        );
        // The handler only sets a lock-free atomic. Child exec resets caught signal
        // handlers. Install before spawning so cancellation cannot orphan a job.
        unsafe {
            libc::signal(libc::SIGTERM, stop_guard as *const () as libc::sighandler_t);
        }
        if unsafe { libc::getppid() } != parent as i32 {
            bail!("daemon exited before execution");
        }
        let mut child = std::process::Command::new(program)
            .args(args)
            .process_group(0)
            .spawn()
            .context("spawn capability")?;
        let group = JobGroup(child.id());
        let deadline = std::time::Instant::now() + Duration::from_secs(timeout);
        let code = loop {
            if GUARD_STOP.load(std::sync::atomic::Ordering::Relaxed)
                || unsafe { libc::getppid() } != parent as i32
                || std::time::Instant::now() >= deadline
            {
                break 125;
            }
            if let Some(status) = child.try_wait()? {
                break status.code().unwrap_or(1);
            }
            std::thread::sleep(Duration::from_millis(50));
        };
        // Terminate remaining descendants even when the direct child succeeded.
        drop(group);
        let _ = child.wait();
        std::process::exit(code);
    }
    #[cfg(not(unix))]
    bail!("execution guardian requires Unix")
}
struct JobGroup(u32);
impl Drop for JobGroup {
    fn drop(&mut self) {
        #[cfg(unix)]
        unsafe {
            libc::kill(-(self.0 as i32), libc::SIGKILL);
        }
    }
}
