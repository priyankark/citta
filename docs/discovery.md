# Devices, agents, and the monitoring panel

`citta --config <path> agents` and MCP `citta_agents` return the local manifest and live manifests from authenticated paired devices. `citta_discover` includes the same information. Discovery is one hop across the owner's paired mesh; no central registry is required.

Each manifest includes OS/architecture, fixed capabilities with separate declared harness/model fields, global/per-capability slots and active counts, hardware telemetry, observed harness sessions, and Citta-managed agent runs. An unreachable peer is explicitly unavailable. Older peers may omit inventory; absence means unknown, never zero agents or idle capacity.

## Hardware and sessions

The daemon samples CPU model, logical/physical cores, aggregate CPU usage, load averages, installed/available/used RAM, swap use, OS version, uptime, and same-user runtime process presence every five seconds. CPU usage needs two samples. Available memory is the OS/library estimate, not a reservation or `total - used`; under macOS compression pressure it can be zero despite reclaimable memory. Resource-qualified tasks additionally use the admission estimates described below; these are not GPU inventory or OS CPU/RAM quotas.

The Codex collector reads the existing same-user app-server Unix socket. It uses `thread/loaded/list` and `thread/read` with `includeTurns:false`; it never starts/resumes a thread or sends a turn. Busy/idle/blocked comes from the harness, not process CPU activity. Waiting for approval or user input is blocked. Other detected harness/model processes are listed separately with unknown session availability until an adapter exists.

Only session IDs, optional names/model names, status, source, and observation times are exported. Commands, environments, previews, working paths, prompts, and transcripts are excluded. Session names themselves may be descriptive, so the inventory shares them only inside the authenticated trusted mesh. Harness collection is bounded to 128 loaded sessions and a three-second request budget; truncation and connection failure are reported. Failure clears the observed session list rather than retaining an old idle claim.

```toml
[inventory]
enabled = true
# Optional; otherwise use $HOME/.codex/app-server-control/app-server-control.sock.
codex_socket = "/absolute/path/to/app-server-control.sock"
```

The installed Codex daemon publishes a symlink; both that link and its resolved socket must belong to the service user. `enabled = false` disables hardware/process/harness sampling. Citta still reports its own configured capabilities and managed executions. Configuration changes require a daemon restart. Codex protocol reference: [app-server](https://learn.chatgpt.com/docs/app-server).

## Start an agent on available compute

A capability with `kind = "agent"` is a spawn profile. Its fixed program/arguments, environment, slot limit, and maximum timeout belong to the receiving device. See [capability configuration](capabilities.md#agent-harness). Remote callers supply a profile, prompt, idempotency key, optional target, and time budgets; they cannot supply executable paths, argv, environment, or permission overrides.

```sh
citta --config "$HOME/.config/citta/config.toml" agents
citta --config "$HOME/.config/citta/config.toml" spawn-agent \
  --profile codex-readonly --prompt /absolute/path/to/prompt.txt \
  --key review-001 --target desktop --timeout 180
citta --config "$HOME/.config/citta/config.toml" inspect TASK_UUID
```

MCP `citta_spawn_agent` accepts `profile`, `prompt`, `idempotency_key`, optional `target_node`, `timeout_secs`, and `queue_ttl_secs`. It returns a durable task immediately. A matching worker starts a fresh bounded harness process, using the existing lease, timeout, cancellation, environment, output, and concurrency controls. Legacy untargeted work is claimed by an available matching worker. `citta_delegate` adds CPU/RAM-aware admission and platform/runtime requirements; see the next section.

Spawn requires a reachable device advertising the profile. Full slots can queue the request. Reusing the same key/request returns the original task even if the worker subsequently disappears. Agent-only tasks cannot be claimed by ordinary or legacy workers with a colliding capability name. Both origin and worker need this build for dedicated agent spawning; ordinary legacy task traffic remains compatible.

Managed agents carry their selected profile’s harness/model metadata and appear while executing and in a bounded recent history (128 completed entries alongside active runs). This in-memory registry resets on daemon restart; durable task history remains at each origin. Process completion can precede origin acknowledgement; inspect the origin task for the authoritative result. A disconnected execution is never automatically replayed.

An observed idle session is **visible, not adopted**: it retains its existing owner, context, and permissions. This version can launch new agents but cannot inject work into arbitrary existing sessions. Nested delegation, parent/child budgets, shared checkouts, and recursive cancellation are not yet implemented.

## Resource-aware delegation

`citta_delegate` (HTTP `POST /v1/agents/delegate`, CLI `delegate`) submits an agent task with durable `requirements`. Defaults are 256 MiB and 0.25 CPU cores; callers should estimate their workload. Optional OS, architecture, harness and model fields require exact matches. `remote_only` excludes the originating device, while `target_node` pins a specific device. Without a target, the first eligible worker claims the task; Citta does not globally rank devices or migrate running jobs.

`citta_placement` (HTTP `POST /v1/placement`, CLI `placement`) explains current per-device eligibility without enqueuing work. It checks installed profiles, advertised capability permissions, slots, fresh telemetry and requirements. Origin-specific permissions are still enforced by the receiving worker. A placement response is not a reservation. Missing/incompatible profiles cause a new delegation request to fail before enqueueing. Temporary pressure leaves compatible work queued until capacity returns or its finite queue TTL expires. Idempotent retries return the existing task even if its worker has gone offline.

Each worker serializes its offer/claim/reservation step across all capability slots, subtracts existing reservations and configured memory headroom, and verifies its local resource estimate again before launching the process. Stale (over 20 seconds), future-dated, or unavailable telemetry is ineligible. A pre-launch loss of capacity interrupts the claimed task without starting a process; it is not automatically replayed. The database skips queued requests that do not fit, so a large task does not prevent a smaller one from running. Old workers cannot claim resource-qualified tasks. Manifests advertise `resource_admission_version: 1` and per-capability admission status.

The `[admission]` config defaults to 512 MiB of memory headroom and a 90% CPU-pressure cutoff. `guard_all = true` applies admission to legacy tasks as well; resource-qualified submissions always enforce it. Per-profile `[capabilities.resources]` sets operator estimates (`memory_bytes`, `cpu_cores`); the reservation uses the maximum of the task estimate and profile estimate for each resource. Slots, timeouts, and execution policy remain enforced separately.

`admission_memory_bytes` is an OS-specific estimate with an explicit `admission_memory_source`. Linux uses the system available-memory estimate. macOS uses the kernel's `kern.memorystatus_level` percentage of total RAM because the raw sysinfo value can become zero when compressed pages are subtracted. The original raw memory values remain visible in discovery. Reservations deliberately subtract their full estimate even when a running process may already be reflected in telemetry. This is conservative admission, not hard memory/CPU isolation or a promise to prevent OOM under external load. Existing running tasks are not killed merely because system pressure increases.

Install the repository's `skills/citta-delegate` in the originating harness for proactive runtime use. Citta does not itself inspect arbitrary user tasks or decide to divide a conversation. The skill guides authorized agents to create bounded, self-contained subtasks, discover capabilities, delegate, and collect results. Recursive delegation, shared child budgets, automatic file transfer, and live-session takeover remain unsupported.

## Human monitoring

```sh
citta --config "$HOME/.config/citta/config.toml" panel
```

Open the returned link on that device. The embedded panel shows device CPU/RAM, session states, spawn profiles and capacity, recent runs, failure filtering, and bounded run output. It refreshes every five seconds and marks session data unknown after twenty seconds without a fresh snapshot. It works without a frontend build service or external assets. The Permissions view also edits the selected device’s execution policy; see [device permissions](permissions.md).

The CLI exchanges a separate local management credential for a single-use login link that expires in two minutes. The fragment is removed before login, and the browser receives an eight-hour HttpOnly, SameSite cookie; mesh credentials are never sent to JavaScript. Automatic peer credentials cannot issue panel sessions. Logout and daemon restart invalidate sessions. Keep the short-lived link private. The default address is `http://127.0.0.1:7331/`; `http://localhost:7331/` is also accepted on the configured listener port. Browser sessions are host-specific, so switching between these addresses requires a fresh login link using that host. Use `panel --network` for a login link on the verified private Tailscale HTTPS origin, accessible from another device. Each updated device’s panel can manage any automatically paired, updated device.

The Agents view includes a discovery coverage card for every paired device, even when its collector returns no sessions. A device filter applies to both session rows and the visible runtime-process cards. Runtime observations include the device, process name/PID, memory, and freshness; their session state and model remain unknown. They are never included in reported live/idle session counts, since one process can host many sessions or support another process. A connected Codex collector with zero loaded sessions does not mean the device has no running agents: separate CLI/desktop instances and other harnesses can be outside that server's scope. Configure a supported collector for those sessions to report names, models, and status; this UI change does not inspect transcripts or adopt existing sessions.

The panel reports missing telemetry, unreachable devices, and stale observations explicitly. It does not upgrade devices, alter enrollment, or imply that a process is available for delegation. Task failure filtering is separate from Citta's own [GitHub build history](build-health.md).
