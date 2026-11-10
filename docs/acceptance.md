# Production release acceptance

## Release preparation — 2026-11-10

The current [release-readiness record](release-readiness.md) lists publication gates and separates prior demonstrations from same-candidate acceptance. No production tag is qualified. Main CI is blocked before execution by GitHub account billing/spending limits; this is not a new runtime test failure.

The portable desktop session implementation from guard branch `b324d5c` is reconciled into the release candidate, including setup embedding and CI tests. Nine isolated session tests, the Node desktop adapter suite, Swift context-helper compilation, five adapter/service helper tests, and six CI-history tests passed locally. Live binaries and compiled installed helpers were not replaced. The amended source passed 42 Rust tests, 25 mesh scenarios, 30 setup scenarios and browser regressions locally. Final clean artifact builds and physical receiving-device checks are separate requirements.

Public release acceptance must record a source commit and per-device artifact checksum, not just version `0.1.0`. Existing physical demonstrations do not close the same-build, clean-install, signing, recovery or soak gates.

## Disk admission regression — 2026-11-10

An existing physical Air installation appeared eligible for agent work but failed before agent startup because workspace creation returned `ENOSPC`. That failed task is preserved in private evidence; it was not replayed or treated as a successful preflight. No automatic cleanup was performed.

The candidate now measures available space on its data and work filesystems. Every worker checks a configurable positive reserve (64 MiB by default), regardless of CPU/memory `guard_all`, before claiming work and again before workspace creation and process launch. Unknown capacity fails closed. Node discovery, placement and doctor carry the disk reason and available bytes; older nodes without the field are reported as unknown by updated placement.

Local validation passed 40 Rust tests, warning-free Clippy, a debug build and a real-process mesh regression. The regression sets an intentionally unattainable reserve rather than consuming host disk, verifies that ordinary tasks remain unclaimed until expiry, checks placement and doctor, and proves legacy work resumes with healthy disk even when optional memory admission would reject it. An owned fixture obstruction verifies unknown work-filesystem handling. Unit checks cover stale observations, default configuration compatibility, both filesystem thresholds, and probes that do not create directories.

Independent review found and closed two additional paths: dangling work-directory symlinks now report unknown capacity, and new origins reject old/unready disk claims for every task at the HTTP endpoint and store. Protocol tests verify HTTP 428 leaves the queued task unclaimed, then a fresh ready claim receives that same task. Updated workers omit the new claim field when serving older origins while still enforcing their own disk gate; older workers must be upgraded to serve newer origins. See [protocol compatibility](protocol.md#disk-readiness-on-claims).

This is prevention and diagnosis, not recovery of the already-full Air. Physical same-candidate acceptance remains pending. The reserve is not a disk quota and cannot prevent another process from consuming space after a check.

## Onboarding permissions and recovery — 2026-11-09

Setup and doctor now distinguish configured profiles from verified execution,
desktop access and account access. Setup provides task-specific permission
guidance; the browser's Permissions view explains receiving-device requirements.
Status and first-login requests have bounded waits with recovery guidance.

The isolated onboarding suite passes 29 scenarios, including empty homes, paths
with spaces, missing or incompatible harnesses, login recovery, concurrent setup,
mocked service-start failure, multi-profile pending grants, legacy receipts,
partial grants, interrupted config writes, and policy changes around interrupted
grant updates. Per-profile creation revisions and durable grant intents preserve
owner revocations, including grants made by the first daemon initialization.
Missing audit proof requires explicit reconciliation rather than regranting.
Service-manager failures are simulated; these fixtures do not reset live devices.

Local validation also passed 36 Rust tests, 19 process-based mesh scenarios,
formatting, Clippy, a release build, skill validation and desktop/mobile browser
checks. Browser tests exercise stalled overview and first-login requests plus
subsequent recovery. These checks do not establish app-account readiness.

A physical Air worker completed preflight task
`2cb9f2af-2646-41ff-adb6-0a78c9e17e0c`: the session was locked, Accessibility
and Screen Recording were unavailable to the worker, and no working authenticated
email/browser path was found. No email was sent or UIUC account accessed.
Retrying the identical request with its original key returned the same completed
task without a new execution. Email acceptance remains pending an unlocked,
authenticated receiving-device session; a successful preflight worker is not a
successful email send. Private evidence is retained under `local/air-onboarding-*`.

Additional physical Air testing exposed two installation prerequisites. Its
default Rust 1.86 was too old; a build with the installed qualified toolchain then
exhausted available disk space. The test's reproducible debug cache was cleaned
after checking for active builds and service references, recovering about 810 MB.
Sources, release artifacts, installed binary, identity, policy and service were
preserved. A matching Mini-built arm64 executable was delivered through Tailscale
and its checksum verified. Gatekeeper denied its execution, confirmed in the
receiving system log; the subsequent suite attempt established no passing Air
scenario coverage. These failures remain in the private evidence. They motivated
compiler/disk checks, isolated build targets, verified-artifact guidance and an
explicit receiving-OS execution check before tests.

## Resource-aware delegation — local verification

The resource-admission build adds 4 Rust behavioral tests (33 total): fresh/compatible offers, pressure and missing telemetry, atomic reservations and release, and durable queued-task filtering/idempotency. The real-process mesh has 19 scenarios, including two daemons with the same agent profile: the origin's configured memory headroom makes it ineligible, so an unpinned task executes on the peer. Impossible capacity expires without a worker; incompatible model requirements fail before submission; same-key retries and lookup recover the original task after its worker disconnects. This is a controlled admission test, not deliberate exhaustion of host memory.

Local formatting, Clippy, Rust tests, mesh tests, 5 adapter/service helper checks, 7 upgrade checks, and both skill validators are required for this change. The runtime skill also received an independent scenario-based evaluation covering stale telemetry, model constraints, uncertain submission, and interrupted partial output. Physical same-build routing requires deployment to both devices and a separate acceptance record. Existing GitHub Linux permission-test failure history remains preserved; local success does not establish new Linux CI success.

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

## Device permissions qualification — 2026-11-07

Automated checks cover Restricted/Allow all/Pause modes, allowed-origin scoping, denied capabilities remaining unclaimed, revocation of running jobs (including rapid re-enable), persistent policies and audit history, optimistic revision conflicts, and a missing saved policy failing closed. Authentication tests reject task credentials at management endpoints, management credentials at task endpoints, remote minting of browser sessions, cross-channel challenge replay, and forged requester attribution.

A real TLS integration test automatically pairs independent task and management channels, logs into one device's HTTPS panel with a Secure cookie, edits the other device, verifies that only the selected device changed, rejects a stale overwrite, and revokes management access when identity freshness is lost. Browser QA exercises capability/origin controls, allow-all, conflict recovery, unsupported-device handling, and desktop/mobile layouts. Physical rollout evidence belongs in the private local acceptance record; a local TLS test is not proof that the Air runs this build.

Local permission/runtime-metadata checks passed: 29 Rust tests, 18 real-process mesh scenarios, helper and CI-history tests, warning-free Clippy, formatting, release compilation, and skill validation. The cross-harness mesh fixture uses two differently declared fake harnesses to validate generic routing and metadata propagation; it is not a live Claude Code compatibility claim. On the Mini, the installed private HTTPS panel authenticated with a Secure owner cookie and persisted an unchanged policy as a new revision. Runtime metadata is configured as `codex / gpt-6-astra` and `ollama / gemma3:4b`. The Air does not yet expose the management channel; its permission rollout remains pending.

## Physical management pairing and upgrade tooling — 2026-11-07

The Air subsequently came online with the permissions API. Independent task and management channels paired automatically. From the Mini's private HTTPS browser panel, selecting the Air and saving its unchanged Restricted policy advanced the Air's revision from 1 to 2 with the Mini's stable Tailscale identity as audit actor. The Mini's policy snapshot remained unchanged. This verifies remote permission editing on the physical Air; it does not establish an Air-hosted browser login or the Air's exact source commit (the version API does not expose a commit). Private evidence and screenshots remain in `local/permissions-acceptance.json` and `local/air-permissions-live.png`. After the Air upgrade, echo succeeded in both directions, and Air-owned Codex task `a7e1165f-b5ad-4acf-b85f-698076b4e567` executed on the Mini with busy state observed before successful completion.

The device-local upgrade helper has seven behavioral checks: private recoverable backups preserving identity/policy/failure history, checksum and validation rejection, queued/running work and newer-schema refusal, identity mismatch, incomplete backup safety, failure after atomic replacement without automatic downgrade, and real-daemon lock interoperability followed by an offline upgrade and restart. The helper leaves service lifecycle and post-start migration verification with the setup agent. It does not add unattended remote installation to an older daemon.


## Agent visibility regression — 2026-11-07

The Air reported four Codex processes but zero loaded sessions from its connected Codex server. The Agents view previously showed only session rows, with process observations collapsed below; this hid the distinction between unavailable session coverage and an empty device. Coverage cards now include every device, and process observations are visible with unknown session state. Live/idle totals count reported sessions only.

The browser regression check `scripts/test_panel_inventory.cjs` exercises a Mini session, a remote Linux session with the same session ID, an Air with four processes and no reported sessions, an offline peer, and an older peer without inventory. It verifies device/state/search filters, stale data, safe text rendering, and mobile layout. Run from the repository root after `cargo build --locked`, with Playwright and its Chromium installed; `CITTA_BROWSER` can select an existing Chrome executable. This is an optional browser check, separate from the Rust/mesh CI suite. Physical Air session names and busy/idle/model reporting remain unavailable while its connected server returns no loaded sessions.

## Repeatable onboarding and resource placement — 2026-11-08

`citta setup` now owns local enrollment, user-service installation, Codex MCP/skills, and initial bounded agent-profile registration. `citta doctor` distinguishes legacy builds, missing profiles, paused/denied execution, unreachable peers, and waiting Tailscale devices. Setup preserves existing identity and policy, stages enrollment checks before replacing a live config, retains private file backups, and resumes unfinished profile grants without regranting profiles subsequently disabled by the owner.

Local verification: 36 Rust tests, seven isolated onboarding scenarios, 19 real-process mesh scenarios, seven offline-upgrade scenarios, five helper tests, six CI-history tests, formatting, warning-free Clippy, release build, both skill validators, and browser inventory/panel regression. Onboarding fixtures cover repeat setup, offline/dry-run behavior, Serve and identity conflicts, login recovery, paused/origin policy preservation, fixed-argv prompt handling, and the standard home config path. The resource-placement fixture forces the origin ineligible and verifies execution on a second local daemon; this is not a physical resource-failover claim.

The Mini was upgraded with a private backup and successfully ran setup twice. Identity, token, config semantics, policy revision, and custom profiles were preserved. Both skills were installed, MCP remained enabled, and the user service stayed available. Fresh Tailscale pairing rediscovered the physical Air; target-specific echo task `155cf90e-4715-4b71-a5c4-0e0d8d18d823` succeeded there. Doctor reports the Air as requiring the resource-admission build. Same-build Air profile execution and physical resource-aware fallback remain pending; Tailscale discovery does not install remote software. Detailed records remain private under `local/seamless-setup-*.json`.


## Delegated desktop permissions — 2026-11-08

A Mini worker initially reported Accessibility=false and ScreenCapture=false while the identical Node/Rusty Robot probe under the interactive session reported both true. Restarting the idle Citta service applied the existing accessibility grant. Citta was absent from Screen & System Audio Recording; adding the installed executable through System Settings and restarting made both worker checks true.

Air-origin task `6aa82a67-a8e3-4c37-a881-e53a679f4d19` then executed on the Mini and delivered a unique marker through Rusty Robot input to an owned native test field. The parent independently verified the field contents and the captured image, then closed the fixture. Rusty Robot capture still timed out; the explicitly recorded macOS screenshot fallback succeeded at 3840×2160. No mail or other external message was sent by this verification. Private evidence, including the earlier failed permission probes, is retained under `local/cross-device-desktop-verification.json` and `local/desktop-permission-probe.json`.
