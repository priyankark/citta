# Production release acceptance

Production is the delivery target. A localhost mesh validates implementation behavior; it does not qualify real devices, Tailscale routes, or installed agent harnesses. Do not tag a production release until the device checks below have evidence.

## Automated evidence

Local development host, 2026-11-06, Rust 1.99.1:

| Check | Evidence |
| --- | --- |
| Formatting and warnings | `cargo fmt --check`, Clippy with `-D warnings` |
| Durable state/config/protocol | 9 Rust integration tests: idempotency, claim contention, leases, cancellation, persistence/locking, URL/token validation, WebSocket auth/replay |
| Real daemon mesh | 14 scenarios in `scripts/test_mesh.py`: distinct origin/worker processes, private credentials, remote-only capability, MCP handshake/submission, capacity, environment isolation, cancellation, time/output limits, restart, expiry, disconnection and daemon-crash cleanup |
| Installation/adapters | 5 helper tests: launchd/systemd rendering, non-overwrite, fixed harness argv, fixed local model endpoint/budget (mock model response) |
| Dependency advisories | `cargo audit`; no known vulnerabilities reported at the time of the local run |
| Setup skill | Skill Creator validator passed; installed locally as `citta-setup` |
| Build | Locked debug and optimized release builds passed locally; CI also builds both |

GitHub CI runs these suites on macOS and Linux. Consult the latest successful run for the exact commit. Any failed or unrun platform check remains a release blocker. The model adapter test uses a mock response and does not establish real model/GPU or harness compatibility.

## Automatic enrollment evidence

Enrollment tests exercise a real TLS callback handshake between independent node identities, private state restoration, wrong-owner/tag/expiry exclusion, netmap removal, stale identity failure, forged/replayed/expired challenges, browser-origin rejection, duplicate node names, invalid certificate/hostname rejection, redirect refusal, and preservation of existing Serve routes. TLS fixtures are test-only and are never added to device trust stores. Existing mesh tests cover task execution and MCP. These local tests do not establish physical two-device automatic enrollment.

## Physical device qualification — pending

Record the exact commit and a private device inventory (OS/architecture, node ID, Tailscale HTTPS URL, service lifecycle, harness/version, installed capability and limits). Never record tokens in evidence.

- [ ] Both devices run the same reviewed build and pass `check`.
- [ ] Persistent services start and recover after login/reboot according to the intended lifecycle.
- [ ] Both new-build daemons automatically enroll without manual token exchange, remain paired across restart, and reject devices outside the pinned owner.
- [ ] Tailscale HTTPS discovery works in both directions with valid certificates and intended tailnet access rules.
- [ ] Authentication rejects a wrong token; unrelated devices have no access.
- [ ] A targeted echo task runs on the opposite physical device in each direction; record task IDs and results.
- [ ] An actual agent harness discovers Citta's MCP tools and delegates to a useful capability installed only on the other device.
- [ ] An actual local model/GPU workload completes remotely if model pooling is part of the rollout; verify model-server memory and output budgets.
- [ ] Several concurrent jobs stay within configured global and capability capacity.
- [ ] Cancel and timeout stop the actual harness process tree; model-server or external effects are checked separately.
- [ ] Device sleep/wake and a severed network connection produce interruption or safe resumption of queued tasks, never automatic replay of running tasks.
- [ ] Restart/kill the origin and worker during jobs; inspect durable outcomes and remaining processes/workspaces.
- [ ] Verify a completed command cannot leave detached or lingering compute under the intended adapter's behavior.
- [ ] Backup/restore, retention/pruning, token rotation, and binary rollback have been exercised with test tasks.
- [ ] A representative workload soak has no unexplained interruptions, sustained resource growth, or missing results; record duration, task count, and failure rate.
- [ ] Operator accepts the v1 boundaries below and the tested service configuration.

## V1 boundaries

Single-owner trusted mesh; automatic same-owner enrollment with explicit-peer compatibility; macOS/Linux; fixed configured commands; bounded JSON/text results. No OS sandbox for arbitrary code, large artifact transfer, repository synchronization, active-session injection, recursive delegation budgets, global workflow DAG, GPU-memory enforcement, or multi-tenant authorization. The originating agent can independently submit multiple bounded tasks and combine their results. Treat outputs as untrusted data.

## Release record template

Record date, commit/tag, CI run URL, devices/versions, exercised capabilities, task IDs, observed outcomes, load/soak results, tested recovery procedures, accepted limitations, and unresolved blockers. Publish a version tag only after this record is complete. Do not infer production acceptance from a successful local echo.

## Resource discovery, spawning, and panel evidence — 2026-11-07

Local checks: 25 Rust tests, 16 real-daemon mesh scenarios, five helper tests, six CI-history tests, Clippy, formatting, dependency audit, and locked release build. New cases cover authoritative harness status with no turn collection (including the real Codex socket symlink shape), agent-only claim isolation, remote spawning/idempotent retries/cancellation, panel sessions, cross-origin/Host rejection, read-only browser authority, and denial of panel-session minting to enrolled peers.

On the Mac Mini, the live shared Codex registry reported nine loaded sessions, with one busy and eight idle at the observation time, plus 12 CPU cores and 32 GiB installed memory. Through MCP stdio, `citta_spawn_agent` launched `codex-readonly` task `0365bab4-15fd-4244-8a48-36175160d759`; its busy state was observed and it succeeded under Codex read-only sandboxing with approval policy `never`. This verifies an actual harness launch locally, not physical remote harness execution.

Headless Chrome checks passed for desktop/mobile layout, navigation, agent search/state filtering, failure filtering, task output inspection, logout, and a malicious-label rendering fixture. Screenshots and detailed device evidence remain private in `local/`. The panel never receives mesh tokens.

The Air was unreachable during this feature acceptance and has not been upgraded to this build. Its telemetry/agent state is unknown. Earlier physical echo/model and automatic-enrollment acceptance remains recorded locally; it is not evidence of this build running on both devices. Same-build two-device telemetry/spawning, sleep/wake, soak, and the broader production checklist remain outstanding.
