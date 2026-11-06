# Citta: cross-device agent cooperation

## Product

Citta pools capabilities and spare compute across a person's trusted devices. Agents independently discover what other devices can do, submit bounded work, inspect progress, and consume results. Humans configure the trust boundary and budgets; they do not route each task. Rust is the implementation language. The first supported platforms are macOS and Linux.

The acceptance scenario is an agent on device A discovering a capability installed only on device B, submitting work through MCP, continuing its own work, and retrieving B's result. B must respect its configured capacity. Work and results must survive an origin restart. Disconnects must never silently replay potentially side-effecting work.

## Architecture decisions

- One daemon per device; no mandatory cloud service or permanent cluster leader.
- Each task has one durable origin. Idle workers pull matching tasks from their own origin and explicitly paired peers, rotating through peers for fairness. A SQLite transaction grants one lease. This avoids ambiguous push retries and allows peer-to-peer operation without a consensus system.
- Tailscale provides private connectivity. The daemon binds loopback; Tailscale Serve provides the remote HTTPS endpoint. Pairing is explicit, using peer URLs and per-device bearer secrets. Discovery queries only paired peers, not the entire tailnet. Citta never enables public Funnel.
- Local stdio MCP exposes discovery, submission, inspection, listing, and cancellation to agent harnesses. HTTP/JSON is the versioned inter-device protocol; authenticated WebSockets stream durable task events.
- Workers advertise named capabilities. An operator configures a fixed executable and argument vector per capability; task input travels as JSON on stdin, never interpolated into a shell command. This supports command tools, model wrappers, and agent harness wrappers through the same contract.
- Agents choose the capability and optionally a target device. Available matching workers claim tasks. The initial scheduler enforces global and per-capability concurrency, maximum wall time, and bounded input/output. It does not claim to enforce GPU memory, CPU quotas, or model token charges; wrappers and OS isolation must enforce those.
- Each execution gets a private working directory, cleared environment with an explicit allowlist, bounded output, and a process group terminated on timeout, shutdown, cancellation, or lost lease. A working directory is not an OS security sandbox. Only trusted, operator-installed executables may run.

## Reliability contract

Task states: queued → running → succeeded / failed / cancelled / interrupted. Queued tasks may expire. A running lease is renewed; expiry becomes interrupted, never automatic retry. A partition can leave uncertain external side effects; Citta does not promise exactly-once execution. Explicit resubmission needs a new idempotency key. Repeating the same key and request returns the same task; different content with that key conflicts.

The origin persists task changes and sequenced events atomically in SQLite WAL mode with full synchronization. Database ownership is exclusive per daemon. Claims use random lease credentials; public task views omit them. Completion and renewal require the current credential and an unexpired lease. Worker results are retried only while the same lease is valid. Graceful shutdown stops claiming, terminates active process groups, and attempts terminal reporting.

Jobs have bounded queue lifetimes and execution durations. Queue admission is bounded. Task/event history and work directories need documented retention and maintenance. Authentication is mandatory even locally; remote endpoints require HTTPS; credentials are never passed to task processes by default. This is a single-owner trusted mesh, not a multi-tenant platform.

## Delivery stages

1. Implement the Rust daemon, CLI, durable state machine, peer discovery, worker executor, MCP tools, events, and validated configuration.
2. Ship setup/operations documentation, service definitions for macOS/Linux, and an agent setup skill. Create a private GitHub repository and CI for both platforms, formatting, linting, tests, and dependency audit.
3. Validate two independent daemon processes with distinct databases/tokens: remote-only capability discovery, delegation, concurrent capacity, cancellation, timeout, auth rejection, restart persistence, lease loss, and MCP interoperability.
4. Install on the user's devices through the setup skill. Validate real Tailscale paths, device sleep/wake, network interruption, actual harness/model adapters, and service restart. Record device-specific evidence before a production release.
5. Tag a release only after the device acceptance checklist passes. Keep production readiness claims tied to recorded evidence.

## Deliberate v1 boundaries

Bounded JSON and text results are supported first. Large artifact transfer, repository synchronization, automatic worktree creation, active harness-session injection, global DAG scheduling, transitive discovery, nested delegation budgets, load/energy-aware placement, per-principal authorization, and Windows support are later work. Arbitrary agent wrappers must not receive unrestricted mesh credentials or spawn unbounded child tasks. No implicit recursive delegation in v1.

## Release evidence

See `docs/acceptance.md` for the device rollout checklist and recorded verification. A local multi-process run proves protocol behavior, not physical cross-device readiness.
