# Citta: a network for agents

Citta connects agents into a private network to share access to tools, data and compute. Tasks use CPU and memory on the machine that runs them.

## Product

Citta connects agents so they can use tools, data and available compute across the owner's trusted devices. Agents discover what is available, submit bounded work, inspect progress and collect results. The owner configures access and budgets; setup installs the local service and connects eligible devices. Each receiving device still needs its tools, accounts and permissions. Rust is the implementation language, with macOS and Linux as the first supported platforms.

The acceptance scenario is an agent on device A discovering a capability installed only on device B, submitting work through MCP, continuing its own work, and retrieving B's result. B must respect its configured capacity. Work and results must survive an origin restart. Disconnects must never silently replay potentially side-effecting work.

## Architecture decisions

- One daemon per device; no mandatory cloud service or permanent cluster leader.
- Each task has one durable origin. Idle workers pull matching tasks from their own origin and enrolled peers, rotating through peers for fairness. A SQLite transaction grants one lease. This avoids ambiguous push retries and allows peer-to-peer operation without a consensus system.
- Tailscale provides private connectivity. The daemon binds loopback; Tailscale Serve provides the remote HTTPS endpoint. Automatic enrollment pins the existing Tailscale owner/tailnet/device, discovers same-owner untagged devices, and performs verified HTTPS challenge callbacks to exchange per-peer credentials. Workers adopt newly paired devices without restarts. Existing explicit peer grants remain supported for compatibility. Citta never enables public Funnel.
- Local stdio MCP exposes discovery, submission, inspection, listing, and cancellation to agent harnesses. HTTP/JSON is the versioned inter-device protocol; authenticated WebSockets stream durable task events.
- Workers advertise named capabilities. An operator configures a fixed executable and argument vector per capability; task input travels as JSON on stdin, never interpolated into a shell command. This supports command tools, model wrappers, and agent harness wrappers through the same contract.
- Agents choose the capability and optionally a target device. Available matching workers claim tasks. The scheduler enforces global and per-capability concurrency, maximum wall time, bounded input/output, and fresh CPU/RAM admission with reservations for resource-qualified tasks. It does not claim to enforce GPU memory, CPU quotas, or model token charges; wrappers and OS isolation must enforce those.
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

Bounded JSON and text results are supported first. Large artifact transfer, repository synchronization, automatic worktree creation, active harness-session injection, global DAG scheduling, transitive discovery, nested delegation budgets, global load/energy optimization, and Windows support are later work. Basic device-owned permissions are implemented; OS sandboxing and richer approval rules remain the hardening work described below. Arbitrary agent wrappers must not receive unrestricted mesh credentials or spawn unbounded child tasks. No implicit recursive delegation in v1.

## Discovery and human monitoring — implemented

Manifests now include sampled CPU/RAM/OS telemetry, same-user runtime processes, harness-reported Codex loaded-session states, and Citta-managed executions. Agent profiles launch fresh bounded processes through the durable task machinery. The local authenticated browser panel monitors devices, agents, capacity, and recent failures. Stale/absent telemetry is unknown. Additional harness adapters, reuse of existing idle sessions, and remote software upgrades remain future work. See [the implemented contract](docs/discovery.md).

## Resource-qualified routing — implemented

`citta_delegate` queues bounded agent work with CPU/RAM estimates and optional platform/runtime constraints. Eligible workers reserve observed capacity and enforce device policy before launch. Busy origins can offload to compatible peers; tasks remain queued when none currently fit. The runtime delegation skill guides proactive use within the user’s task authorization. This does not migrate existing sessions, provide OS quotas, rank all devices globally, or implement recursive budgets. Physical rollout evidence is recorded separately.

## Device-owned permissions and unattended execution

Status: basic enforcement is implemented: Restricted capability allowlists, Allow all, Pause, stable origin allowlists, live revocation, revision conflicts, durable audit history, and a cross-device owner panel with separate automatic management credentials. See [the implemented contract](docs/permissions.md). OS sandbox integration, approval holds, per-capability origin matrices, resource-specific grants, and the richer policy design below remain future work.

The receiving device owns the final execution decision. A sending agent can request a capability and narrower permissions; it cannot pick its own trust level, widen a sandbox, grant approval, or activate allow-all through task input. Decide authorization independently of which sandbox backend will enforce resource access.

Two separate settings:

- `restricted`: only locally registered capabilities granted to the authenticated origin can run. Rules can allow unattended execution, hold for local approval, or deny. Unmatched requests are denied. Rule ordering must be deterministic, with explicit deny overriding a broad allow.
- `allow_all`: all locally registered capabilities may run unattended for the locally selected trusted origins. This is an explicit user-selected mode, not a fallback when policy fails. Authentication, quotas, cancellation, and audit history remain enabled.

Sandbox selection is independent: a named restricted profile, or explicit `none` for full host access under the worker's OS account. Thus an owner can allow every task without approval while retaining isolation, or deliberately configure `allow_all` plus `none` for their fully trusted device. The remote request cannot select either. OS privileges and harness restrictions still apply; Citta's allow-all is not an override of external controls. It does not create an arbitrary remote-command endpoint: general shell/agent execution, if desired, must itself be a locally installed capability.

Task categories must correspond to enforceable capability contracts, not labels supplied by an agent. Suggested presets:

| Capability preset | Unattended behavior | Files/network/credentials |
| --- | --- | --- |
| Local inference | Allow | JSON input/output; access only to an approved local model broker; model/token budget set locally |
| Repository inspection | Allow | Read-only repository snapshot plus scratch space; no network or secrets |
| Build/test | Allow | Disposable writable workspace; dependencies preloaded or fetched through an approved broker; no production credentials |
| Patch generation | Allow | Disposable checkout; return a patch artifact; no push/merge permission |
| Publish/deploy/account changes | Approval or deny by local policy | Separate narrowly scoped capability and credentials |
| Trusted general agent | Owner-configured allow-all | Chosen sandbox profile, or explicit full-host execution |

Test/build commands themselves execute repository code. A task named `test` must not receive a broader host grant merely because it sounds harmless. Artifact import, repository staging, and patch return need their own validation before those presets can be shipped.

Rules bind to authenticated origin identity, stable capability IDs, resource IDs resolved on the worker, execution budgets, and a policy revision. Human-readable node IDs or task-supplied `kind`/`origin` fields alone are not authorization. Automatic enrollment now issues identity-bound per-peer credentials, while the local/legacy node token grants task API access. Introduce capability scopes and separate control-plane permissions before advertising isolation between peers. In the pull protocol, check the configured origin channel and local grants before offering a capability, then revalidate the actual task immediately before spawn.

Approval holds must not consume worker slots or running leases. A durable approval binds to the immutable request digest, selected target, capability definition, policy revision, expiry, and requested resource grants. Editing any of those invalidates it. Default behavior for unattended workers is to leave a clear blocked/denied state, never wait indefinitely for hidden terminal prompts. The owner's explicitly configured allow-all choice removes Citta's approval requirement within that scope.

## Sandbox backend evaluation

MXC is the leading candidate: its Rust SDK exposes sandbox execution and lifecycle APIs, with Seatbelt on macOS and Bubblewrap on Linux. Evaluate a pinned stable SDK/schema rather than tracking development interfaces. References: [MXC](https://github.com/microsoft/mxc), [Rust SDK](https://github.com/microsoft/mxc/blob/main/src/mxc-sdk/README.md), [stable releases](https://github.com/microsoft/mxc/releases).

Keep Citta's policy model independent of MXC behind an executor interface. Each job gets a fresh sandbox with explicit filesystem, network, UI, environment, and credential grants. Keep Citta state, tokens, host control sockets, and harness configuration outside writable mounts. Provision a scoped credential/model broker when needed instead of exposing a broad home directory. For GPU inference, prefer a bounded broker to the host model runtime; do not assume a Linux VM can use a Mac's native GPU stack.

Backend support is not uniform. Seatbelt supports filesystem controls and coarse outbound/loopback controls, but not direct hostname/IP allowlists. Destination filtering requires an actually enforced broker/proxy path. Linux host namespace and networking prerequisites also need runtime checks. Reject a policy the available backend cannot enforce; never silently run it with weaker isolation. References: [Seatbelt limits](https://github.com/microsoft/mxc/blob/main/docs/backends/seatbelt/seatbelt-backend.md), [Bubblewrap requirements](https://github.com/microsoft/mxc/blob/main/docs/backends/bwrap/bubblewrap-backend.md).

Integrate sandbox termination with the existing guardian, leases, output bounds, and cancellation. Validate process-tree and container cleanup; a process-group signal alone is not evidence that every sandboxed descendant stopped. Sandbox filesystem paths and launch arguments must be constructed from local definitions and validated resources without interpolating task text into a command line.

Qualification must deliberately attempt forbidden file reads/writes, credential access, direct-network and proxy bypass, child-process escapes, policy self-modification, and a task's attempt to enable allow-all. Test expected allowed operations too. Missing/incompatible backends must produce explicit errors. Test the restricted and allow-all modes on each supported OS before promoting this hardening milestone.

## Failure visibility and build history

Preserve failures as durable records, including after a successful retry. Distinguish command/test failures, timeouts, lease loss, policy rejection, sandbox denial, resource exhaustion, and infrastructure failures. Associate each with its task/build ID, attempt, origin/worker, capability, commit/artifact identity when applicable, policy/backend revision, timestamps, bounded diagnostics, and resolution or superseding attempt. A repository build task should return structured build/test results rather than relying on an exit code alone.

The monitoring panel now filters recent failed/interrupted runs and inspects bounded output. A richer durable failure inbox remains planned: expose it through CLI/MCP and the UI, with filters by device, capability, repository, and cause. Acknowledging a failure should not delete its evidence. Never auto-retry side-effecting tasks or weaken a sandbox to make a failure disappear. Retrying an explicitly repeatable build can be a bounded local policy; a new attempt keeps its relationship to the previous one.

Track Citta's own CI separately from product task failures. Preserve run/attempt URLs and job conclusions; distinguish runner allocation failures from source failures. A new green commit does not automatically establish that an older defect was fixed. The development tracker is an on-demand snapshot, not a claim of continuous background monitoring. Scheduled or push-driven product monitoring is a separate implementation step.

## Release evidence

See `docs/acceptance.md` for the device rollout checklist and recorded verification. A local multi-process run proves protocol behavior, not physical cross-device readiness.

## Repeatable local onboarding

Implemented: `citta setup` configures the installed device's automatic enrollment, persistent user service, Codex MCP/skills and initial bounded read-only profile. `citta doctor` explains peer readiness. Reruns preserve device identity and existing execution policy, and resume incomplete initial grants. Each device still needs Citta installed once and Tailscale connected; no SSH or copied credentials are needed for subsequent pairing. Binary self-update and remote bootstrap on a machine without a deployment agent remain unimplemented. The Mini is verified; the Air needs the matching resource-admission build before receiving automatic agent offloads.

## Full-tool agent execution

General-purpose setup now installs a `codex` profile with full shell/filesystem/network access and the device’s configured tools, MCP servers and skills. Explicit read-only profiles remain selectable. Agents may delegate further; child tasks are independent and must be tracked and collected/cancelled explicitly. Automatic cascading cancellation and shared parent/child budgets remain future work. Resource admission and device/task lifecycle controls are separate from tool availability.
