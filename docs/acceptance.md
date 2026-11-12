# Production release acceptance

## Final candidate `c767ce5` — 2026-11-12

Current artifact acceptance is pinned to clean source `c767ce5b5c1ad283c00afad9684a4f809a7c857e`, which includes the setup-recovery correction. [Full CI run 43160740009](https://github.com/priyankark/citta/actions/runs/43160740009) passed on `59debd4e332e6bbf77eb77ec2242a769996650ba`; the runtime source, Cargo manifests/lockfile, UI, skills and examples are identical to `c767ce5`. [Native packaging run 43162820799](https://github.com/priyankark/citta/actions/runs/43162820799) passed all three platform jobs on `c767ce5`. This distinction is intentional: full CI ran on the equivalent runtime source, while final packaging ran on the artifact source. Source-crate packaging, publish dry-run and fresh-target installation passed before publication.

Both final macOS executables passed Developer ID signing verification, hardened-runtime/timestamp checks, Apple notarization acceptance and explicit online ticket verification. [Native signed Intel run 43164792893](https://github.com/priyankark/citta/actions/runs/43164792893) executed the exact pinned archive on `x86_64` macOS 26.6.1 without rebuilding: setup help, fresh isolated installation/version/help, all 442 manifest files, required assets, existing-install refusal and no service/configuration changes passed. Signed ARM local fresh installation and setup help also passed. See [signing receipts and scope](signing.md#final-candidate-c767ce5--november-12-2026-utc) and [final artifact hashes](release-readiness.md).

The final signed ARM archive passed fresh isolated installation and bounded physical tests on both Macs. Real private same-owner enrollment, bidirectional echo/hostname/input-digest checks, fresh disk/resource admission, a resource-qualified task, same-key identity and incorrect-token rejection passed. An Air Codex worker used Bash, wrote/tested Python and read an installed skill; an Air-origin task ran Gemma 3:4b on the Mini. All 18 candidate tasks reached the expected terminal states: 16 succeeded, one deliberate cancellation was cancelled, and one deliberately interrupted task remained interrupted without replay. The separately counted live-service supervisor succeeded and was collected.

A 20.208-second outage of the owned candidate endpoint left a task queued; it succeeded after reconnect. Receiving-daemon restart and a distinct post-restart task also passed. Two setup-recovery fixture scenarios ran on each physical Mac using the final signed binary: first Serve failure retained fresh defaults, and retry preserved an existing operator's admission choice. These setup fixtures used simulated Tailscale failures; actual mesh pairing used real private HTTPS. Cleanup stopped owned daemons/listeners and the transfer server, restored exact Serve state, preserved live binaries/configuration/service state and left both live nodes idle.

[Version 0.1.0](https://github.com/priyankark/citta/releases/tag/v0.1.0) was published on November 12 at 06:03:10 UTC with the nine verified assets from `c767ce5`. The single upload of the exact prepared source crate completed successfully. All nine anonymous downloads and a fresh registry-only installation subsequently passed. The public ARM browser download/install check also passed with quarantine preserved; see the final distribution follow-up below.

This is bounded final-candidate coding/model/recovery coverage, not a new live rollout or soak. The `f6fd91f` live deployment and 30-minute soak below remain historical. Clean-user provisioning, the full public download/setup path, receiving-worker desktop grants, sleep/reboot, whole-device network loss and representative multi-hour workloads remain open. The older live-service renewal interruption also remains unexplained; it was not replayed. The early release does not establish completion of the broader production checklist.

Later agent-first README/gallery changes are repository documentation updates; the fixed archives and crate retain their `c767ce5` contents. Acceptance must follow the artifact source and hashes rather than whichever documentation commit is currently on `main`.

<a id="current-candidate-evidence--2026-11-12"></a>

## Historical `f6fd91f` candidate evidence — 2026-11-12

This earlier qualification record is pinned to `f6fd91fa7978e16b5aca6dec227c1e303de0daea`. The [native packaging workflow](https://github.com/priyankark/citta/actions/runs/42974812756) passed all three platform jobs and native installer checks; all downloaded manifest hashes were verified. The source crate passed packaging, publish dry-run and isolated installation without upload. The [main CI retry](https://github.com/priyankark/citta/actions/runs/42974236178/attempts/2) completed successfully: Linux, Intel macOS (including ARM compilation) and audit all passed. Its original pre-start billing failure is retained and linked to recovery. This validation belongs to the pinned `f6fd91f` source, even if this evidence is later merged in a documentation-only commit.

The exact CI Apple Silicon artifact ran on both physical Macs in fresh isolated installations and temporary daemons. Checks passed for automatic private pairing, bidirectional shell proof, fresh resource/disk admission, idempotency and incorrect-token rejection. A real receiving Codex agent used Bash, wrote/tested Python and read an installed skill. An actual Air-origin task used the Mini's Gemma 3:4b through the candidate adapter. Twenty lightweight remote tasks completed over 197.38 seconds; this is short batch coverage, not long-soak qualification. Daemon-restart interruption and a separate successful post-restart task verified that the original was not replayed.

Independent cleanup confirmed all owned processes/listeners stopped, exact prior Serve configurations were restored and live executable/configuration/service-plist hashes stayed unchanged. Detailed [artifact and physical evidence](release-readiness.md#current-candidate-artifacts-and-native-checks) preserves the earlier candidate and failure records. No production tag is qualified: clean-user provider/model provisioning, desktop access, sleep/reboot/network-loss recovery, representative multi-hour soak and the intended public download/setup path remain open. Coordinated signed live coding deployment and bounded distribution checks subsequently passed as recorded below; the separate 30-minute signed soak passed within the scope below. The broader production checklist below is not completed by isolated candidate tests.

<a id="signed-distribution-and-live-coding-deployment--november-12"></a>

## Historical `f6fd91f` signed distribution and live coding deployment — November 12

The exact signed Intel archive passed [native verification run 43153509046](https://github.com/priyankark/citta/actions/runs/43153509046) on `x86_64` macOS 26.6.1 without rebuilding. Archive/executable hashes, strict Developer ID identity, hardened runtime/timestamp, designated requirement and online notarization checks passed. Native setup help, fresh isolated installation/version/help, refusal to overwrite and absence of service/configuration changes also passed. The [initial run 43152970631](https://github.com/priyankark/citta/actions/runs/43152970631) failed with HTTP 403 reading the private draft before download/execution; that failure is retained alongside the corrected manual-job permissions and successful run. Intel physical mesh, desktop, browser quarantine and offline ticket behavior remain unverified.

A real signed-in Chrome download of the private draft's exact ARM archive passed native tar extraction and the normal installer under an isolated HOME/prefix on the existing Mini account. The installed executable matched the signed candidate and retained the original browser quarantine value exactly. Version/setup help and repeat-install refusal passed; no quarantine attribute or OS policy was changed. The earlier Archive Utility attempt produced no extraction. This establishes the documented tar/installer path for that existing account, not a clean OS account or public release URL. See [distribution evidence](release-readiness.md#native-signed-intel-and-arm-browser-installation--november-12).

At this historical handoff, both live Macs retained the signed/notarized ARM executable `5551c4eae2c43ebe958d6ad0d79919062f5b20a40e334355e392c6f4597fea76` from pinned source `f6fd91f`. Coordinated idle maintenance used current-state backups and an explicit bounded Air controller, preserving current compatible configuration, schema-4 identity, credentials, policy/audit, original task history and Serve state. Fresh service-launched agents on both devices used Bash, wrote/tested Python and read installed skills; loaded Mini MCP discovery, bidirectional echo and an actual Air-origin Mini Gemma 3:4b task passed. The deployment was committed for coding use, the temporary controller exited/unloaded, all maintenance tasks were collected and both devices were idle at handoff. Previous binaries and rollback guidance remain available.

Two pre-service preparation failures and an unexplained 180-second cleanup-report timeout remain recorded. No mutation was replayed; a distinct read-only inspection confirmed committed state, unloaded maintenance job and the exact healthy signed service. Private controller corrections passed 11 isolated tests and did not change runtime source. The signed Mini worker reported both desktop grants false despite an active display; the prior binary had both true. The Air remained locked with no display and both grants false. No input/capture occurred. Coding rollout succeeded; signed desktop permission migration remains pending. See [deployment scope and rollback constraints](release-readiness.md#coordinated-signed-live-coding-deployment--november-12).

The signed `f6fd91f` live deployment completed a 30-minute workload on each origin: all 136 tasks succeeded (six Air Codex jobs, ten Mini Gemma jobs and 120 bidirectional echoes), with zero failed/interrupted/cancelled tasks, retries or outstanding submissions. The separate Air supervisor completed and all owned processes/workspaces were cleaned up; both services were healthy and idle. Sampled daemon RSS ranges were stable within the measured interval; model-process RAM was not separately measured. This does not establish multi-hour coverage or qualify subsequent runtime changes. Clean-user provisioning, public-URL setup, desktop grants, sleep/reboot, whole-device network-loss recovery and representative multi-hour workload reliability remain open.

<a id="additional-qualification-work--november-11"></a>

## Historical `f6fd91f` additional qualification — November 11

The seven offline-upgrade tests passed against the exact `f6fd91f` CI executable. A separate real old→candidate→old isolated rehearsal preserved the current schema-4 database, configuration, credential and all four task histories, including an intentional failure and work created after upgrade. A new task succeeded after rollback; fixture processes exited and live installation hashes stayed unchanged. This does not qualify a live service upgrade. The old executable rejects a candidate-generated configuration containing `admission.disk_headroom_bytes`, so rollback requires checking the current configuration as well as the database; restoring stale state is not an acceptable substitute.

Both macOS artifacts were separately Developer ID Application signed with `io.citta.daemon`, hardened runtime and secure timestamps, passed strict signature verification and all updated manifest hashes, and were placed in submission ZIPs. The earlier Xcode certificate-creation failure remains recorded; the authorized web portal subsequently supplied a usable certificate. Notarization authentication was validated, both ZIPs were submitted once and Apple accepted both. Explicit online notarization-ticket checks passed. Separate `spctl` assessments rejected the bare CLI as valid code that did not appear to be an app; those results are preserved, with no quarantine/policy changes. Final distribution archives remain unpublished. The signed ARM bundle passed fresh isolated installation, all 442 installed hashes, version/help/setup-help and repeat-install refusal; its final tar archive also passed isolated installer checks. Signed Intel execution was not run locally because the ARM host lacked Rosetta; its architecture mismatch was correctly refused. Original Intel CI does not establish execution of the newly signed bytes. Separate physical signed ARM tests then passed fresh installation/execution on both Macs, automatic pairing, bidirectional shell proof, resource checks, idempotency/authentication, a real full-tool receiving worker and interruption/restart without replay. Cleanup restored exact prior routes and live hashes. At that point public download, signed desktop continuity and coordinated two-device rollout remained open; the signed short batch does not inherit the longer unsigned soak. The later November 12 checks above add bounded distribution and live coding deployment evidence. See [signed staging](release-readiness.md#developer-id-signed-staging).

A preflight on the older live Air service was interrupted after about 18 seconds with a generic renewal error. A distinct read-only diagnostic succeeded, but the logs did not expose the cause. The failed task was preserved without replay. At 17:29 UTC the Air remained locked, with no active displays and both desktop permission flags false; no desktop action occurred. A separate physical soak of the original ad-hoc candidate completed 1,267.61 seconds with 110 forward and 41 reverse lightweight jobs and zero unexpected workload failures. A 35-second owned-endpoint outage preserved a queued task until successful recovery; intentional receiving-daemon restart left the original task interrupted without replay. Independent cleanup verified owned processes/listeners exited, exact Serve restoration and unchanged live hashes. This 21-minute lightweight run does not establish multi-hour agent/model stress, whole-device network-loss or sleep/reboot coverage. These observations alone did not close coordinated two-device rollout, desktop, sleep/reboot/network-loss or signed public-distribution gates.

A subsequent bounded live Mini old→signed→old maintenance rehearsal passed actual launchd stop/start, private current-state backup, signed local echo and a service-launched worker's Python/Bash checks, then exact old-binary rollback. All original and signed-phase task payloads, schema-4 identity, config, credentials, plist, policy and audit records were preserved. Both-direction echo passed after rollback, exact Serve state was restored and both devices were idle. The signed worker lost both desktop permission flags; no grants changed because the desktop became unavailable. A new old-service worker regained both flags but still had no display, so delivery/capture remained unverified. Both live devices retained their original binaries at the end of that rehearsal; the November 12 coordinated coding rollout above subsequently replaced them. This is bounded live lifecycle/rollback evidence, not desktop acceptance. See [the live rehearsal](release-readiness.md#bounded-mini-live-upgrade-and-rollback).

At this earlier preparation snapshot, a private draft release contained nine hash-verified platform archives/manifests/checksum files, and crate-scoped registry publishing was configured. No public release, tag or crate upload had occurred. See [publication staging](release-readiness.md#publication-staging).

## Release preparation — 2026-11-10

The current [release-readiness record](release-readiness.md) lists publication gates and separates prior demonstrations from same-candidate acceptance. No production tag was qualified at this preparation stage. Main CI was blocked before execution by GitHub account billing/spending limits; this was not a new runtime test failure. Later evidence is recorded above.

The portable desktop session implementation from guard branch `b324d5c` is reconciled into the release candidate, including setup embedding and CI tests. Nine isolated session tests, the Node desktop adapter suite, Swift context-helper compilation, five adapter/service helper tests, and six CI-history tests passed locally. Live binaries and compiled installed helpers were not replaced. The amended source passed 42 Rust tests, 25 mesh scenarios, 30 setup scenarios and browser regressions locally. Final clean artifact builds and physical receiving-device checks are separate requirements.

Public release acceptance must record a source commit and per-device artifact checksum, not just version `0.1.0`. Existing physical demonstrations do not close the same-build, clean-install, signing, recovery or soak gates.

## Disk admission regression — 2026-11-10

An existing physical Air installation appeared eligible for agent work but failed before agent startup because workspace creation returned `ENOSPC`. That failed task is preserved in private evidence; it was not replayed or treated as a successful preflight. No automatic cleanup was performed.

The candidate now measures available space on its data and work filesystems. Every worker checks a configurable positive reserve (64 MiB by default), regardless of CPU/memory `guard_all`, before claiming work and again before workspace creation and process launch. Unknown capacity fails closed. Node discovery, placement and doctor carry the disk reason and available bytes; older nodes without the field are reported as unknown by updated placement.

Local validation passed 40 Rust tests, warning-free Clippy, a debug build and a real-process mesh regression. The regression sets an intentionally unattainable reserve rather than consuming host disk, verifies that ordinary tasks remain unclaimed until expiry, checks placement and doctor, and proves legacy work resumes with healthy disk even when optional memory admission would reject it. An owned fixture obstruction verifies unknown work-filesystem handling. Unit checks cover stale observations, default configuration compatibility, both filesystem thresholds, and probes that do not create directories.

Independent review found and closed two additional paths: dangling work-directory symlinks now report unknown capacity, and new origins reject old/unready disk claims for every task at the HTTP endpoint and store. Protocol tests verify HTTP 428 leaves the queued task unclaimed, then a fresh ready claim receives that same task. Updated workers omit the new claim field when serving older origins while still enforcing their own disk gate; older workers must be upgraded to serve newer origins. See [protocol compatibility](protocol.md#disk-readiness-on-claims).

This is prevention and diagnosis, not automatic recovery of the already-full Air. Physical same-candidate acceptance was still pending at this regression stage; later scoped cleanup and candidate checks are recorded above. The reserve is not a disk quota and cannot prevent another process from consuming space after a check.

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

Checked items below identify historical `f6fd91f` live-deployment evidence explicitly. The final `c767ce5` section above records its completed bounded physical smoke; neither record completes this broader production checklist.

Record the exact commit and a private device inventory (OS/architecture, node ID, Tailscale HTTPS URL, service lifecycle, harness/version, installed capability and limits). Never record tokens in evidence.

- [x] Both devices run the same reviewed build and pass `check` (signed `f6fd91f` live coding deployment, November 12).
- [ ] Persistent services start and recover after login/reboot according to the intended lifecycle.
- [ ] Both new-build daemons automatically enroll without manual token exchange, remain paired across restart, and reject devices outside the pinned owner.
- [ ] Tailscale HTTPS discovery works in both directions with valid certificates and intended tailnet access rules.
- [ ] Authentication rejects a wrong token; unrelated devices have no access.
- [x] A targeted echo task runs on the opposite physical device in each direction; task IDs/results are retained for the signed live deployment.
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

## Public distribution follow-up — November 12

All nine published assets passed anonymous download/hash/full-manifest verification. A fresh registry-only Cargo installation passed with no publishing credentials. A new independent Chrome profile downloaded the public ARM archive with its original quarantine; native extraction and isolated installation preserved it, and execution/setup-help/online-notarization checks passed. This used an existing Mini OS account, not a clean OS account or Intel browser, and created no service/configuration. See [public distribution verification](release-readiness.md#public-distribution-verification).
