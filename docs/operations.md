# Operations

Run one daemon per data directory. Citta takes an exclusive lock, checks the database's node identity and schema version, and uses SQLite WAL with `synchronous=FULL`. Keep state on a local filesystem that supports SQLite locking and durable synchronization. Do not share a live database through a network filesystem or sync client.

## Service lifecycle

SIGTERM/Ctrl-C stop new claims, stop running job groups, attempt interruption reporting, close event streams, and shut down HTTP. A separate per-job guardian watches daemon death and wall time. It terminates the job's process group if its parent disappears; it also removes leftover descendants when the immediate command exits. Capabilities must not detach from their process group. SIGKILL of the guardian itself, an OS crash, detached processes, and external APIs are outside this in-process containment guarantee.

The daemon continues serving API requests while workers drain. A caller may enqueue during that interval; the durable task remains for the next startup. Deployments should stop new submissions before maintenance when a fully drained queue is required.

macOS logs are under the config directory's `logs/`; Linux logs go to the user journal. Logs identify tasks and origins, but omit input, output, bearer tokens, and lease credentials. Rotate macOS logs with the host's log management. `RUST_LOG=citta=debug` enables peer availability diagnostics.

## Capacity, retention, backup

Default admission bounds: 1,000 pending tasks and 10,000 total retained tasks. Default input/output limits are 64 KiB / 256 KiB per output stream. Configure lower history limits for small disks. Monitor free disk space and the authenticated health/node/task endpoints. Full disks can prevent a result commit; workers then stop or report interruption. The daemon stops if periodic database maintenance fails.

History is never silently discarded. To prune terminal tasks older than 30 days, first stop the daemon, then:

```sh
citta --config /absolute/path/config.toml prune --older-than-days 30
```

Pruning removes associated events and idempotency keys. Old retries can then create new tasks, so choose a retention period longer than every client's retry horizon. Pending/running tasks are never pruned. Space becomes reusable in SQLite; pruning does not guarantee the file shrinks. Run SQLite maintenance offline if physical compaction is necessary. Work directories left after hard crashes can be removed only after the service and guardians have stopped and any needed diagnostics are retained.

For a consistent backup, stop the daemon and copy the entire data directory, config, and required token files to protected storage. Avoid copying only `citta.db` while it is running. Restore to the same node identity; never start two copies simultaneously. Token copies remain secrets in backups.

## Upgrade and rollback

Use the same reviewed commit on all participating devices. Stop new submissions, allow active tasks to finish or cancel them deliberately, stop the service, back up state, replace the binary, run `check`, and restart. Confirm discovery and a targeted remote task. The initial schema is version 1; newer schema versions are rejected rather than opened unsafely. Roll back the binary with its compatible backup if a future migration is irreversible. Keep prior binaries until the device acceptance checks pass.

### Build and artifact preflight

Before compiling, inspect `rustc --version`, `cargo --version`, installed toolchains and free space on the build/install/backup volumes. Use this checkout’s qualified toolchain; the manifest’s minimum version is not evidence that a toolchain was tested. Select an installed toolchain per command with `cargo +TOOLCHAIN` rather than changing the global default. Build output and backups need more space than the final executable.

When disk is constrained, prefer an existing qualified binary for the same OS, architecture and reviewed commit from authenticated CI or a trusted device. Record its provenance and SHA-256, verify the digest after transfer, then follow the offline upgrade procedure for an existing installation. File delivery does not install or restart the service.

macOS may require approval before running a downloaded executable, including one
received through Tailscale. After verifying the artifact, follow the system's
per-app approval flow on the receiving device. A denied or unavailable executable
is an installation blocker; it is not evidence that the application's tests ran.
Keep the original diagnostic and verify execution before starting the test suite.

For disposable build tests, use a task-specific `CARGO_TARGET_DIR`. Clean only that task’s reproducible output after confirming no active build or service uses it; preserve the candidate, installed binary, state, backups and unrelated caches. Reassess space after a disk-full failure before retrying.

Tailscale file transfer is an optional artifact delivery channel, not a pairing prerequisite. Use a unique filename, inspect the receiver’s actual inbox/destination and select only that artifact. Do not assume it appears in Downloads or move unrelated pending files.

### macOS signing identity and desktop access

Before replacing a Mac service executable, inspect both installed and candidate files with `codesign -d -r- --verbose=4 /absolute/path/to/citta`. Compare the signing identity and designated requirement, not just the filename. An ad-hoc signature can bind the requirement to a code hash; replacing that binary at the same path can change its identity and invalidate desktop grants. This does not mean every upgrade loses permissions.

Record the existing service worker's Accessibility and Screen Recording checks. If the replacement may require fresh grants for desktop work and OS approval cannot be completed, keep the candidate staged until the receiving device is unlocked and the system approval flow is available. After replacement/restart, check a new service-launched worker and verify actual input/capture before claiming desktop readiness. An interactive shell's flags do not establish the worker's grants; worker flags alone do not establish functional input/capture. Do not edit TCC records or sign with an unrelated application's identity to bypass approval.

If a replacement loses required grants, deliberate recovery can restore only the prior verified binary **when it is compatible with the current database schema and config**. First verify no queued/running or worker jobs remain, stop the service and back up the current state; then replace the binary atomically, restart and recheck the worker. Preserve the current database, config, permissions and task history. Do not restore a stale database merely to recover OS grants. If compatibility or approval is unavailable, retain the evidence and staged candidate for completion on the device.

### Device-local upgrade helper

`scripts/upgrade_device.py` (Python 3.11+) performs the offline backup and binary replacement on macOS/Linux. Run it as the existing service user through a device-local agent or established management access. Automatic pairing does not provide remote software installation. Build the candidate separately with `cargo build --release --locked`, or obtain the matching OS/architecture artifact from a verified successful CI run. Record the source commit, run, and SHA-256; a checksum alone does not authenticate a downloaded binary.

Before stopping the service, run the candidate's `--config <existing-config> check`, inspect local/remote active work, and stop new submissions. Stop the existing user service and wait for it to exit. For launchd, wait until `launchctl print gui/UID/io.citta.daemon` no longer finds the old job; for systemd, use `systemctl --user stop citta.service`. Do not delete the lock file. If shutdown interrupts a racing task, retain and inspect that outcome rather than replaying it.

```sh
python3 scripts/upgrade_device.py \
  --config "$HOME/.config/citta/config.toml" \
  --installed "$HOME/.cargo/bin/citta" \
  --candidate /absolute/path/to/reviewed/citta \
  --sha256 REVIEWED_BINARY_SHA256 --apply
```

Without `--apply`, the helper validates without replacing or creating backups; the service must still be stopped. It holds the daemon's exclusive data-directory lock, refuses queued/running work and unknown storage versions, checks the candidate's digest and config, and saves the old executable, config, data, and referenced peer/node credentials under a private `upgrade-backups/` directory beside the config. The SQLite snapshot is consistent and integrity-checked. Data symlinks or nonregular files cause failure before replacement. The service definition is unchanged; back it up separately if changing it. Declared capability metadata, policy, identity, and task history are unchanged. No database migration runs until the new daemon starts.

The helper flushes backups before atomically replacing the executable and records its phase in `manifest.json`. It leaves the service stopped. Restart the same service (`launchctl bootstrap gui/UID <existing-plist>` or `systemctl --user start citta.service`), verify health, automatic enrollment, permission state, and a targeted task. An error leaves evidence in place: inspect the manifest and installed digest before recovery. There is no automatic rollback or task replay, especially after a new daemon might have migrated storage or accepted work. Do not blindly rerun the installer after an uncertain failure.

## Automatic enrollment

Use `citta enrollment` to inspect freshness and pairing states. A stale identity check suspends automatic credentials; a successful check restores eligible devices without restarting. Removal from the pinned owner's netmap invalidates and removes automatic credentials. Re-enrollment is automatic if the device returns. To opt a device out, disable its enrollment configuration and restart it; remove the device from Tailscale when access must be revoked network-wide. Preserve `data/enrollment.json` in private backups, and never copy a device's whole identity to a second device.

Automatic peers and existing explicit peers can coexist during an upgrade. Explicit peer entries and shared node tokens retain their old trust semantics; automatic revocation does not revoke those independent grants. After successful automatic pairing, remove legacy entries on both ends and rotate any shared node tokens. Do not remove old grants before the replacement path is verified.

## Credential rotation

Generate a fresh cryptographically random token (at least 32 URL-safe characters), atomically replace the protected node token, distribute it only to authorized peers, and restart those daemons and MCP subprocesses. Rotation is not hot-reloaded; coordinate it as maintenance. Tokens grant full node access. Removing a peer entry stops its claims but is not sufficient to revoke a copied node token; rotate the node token to revoke access.

## Troubleshooting

- `check` failure: correct absolute paths, existing capability programs, duplicate IDs, private token permissions, or invalid limits.
- Peer unreachable: inspect service status, exact Serve HTTPS origin, Tailscale policy, token pairing, TLS and node-ID mismatch. Do not weaken transport validation.
- Task stays queued: verify the capability exists, the target is reachable, reciprocal pairing exists, and a worker has a free slot. Capability names must match exactly.
- Task interrupted: inspect the origin's task record and external effects before creating another task. Citta will not automatically replay it.
- Task failed: inspect bounded stderr/error; fix the wrapper, missing environment/credentials, timeout, or output limit.
- Duplicate daemon error: find the existing service rather than deleting the lock or copying the database elsewhere.

### Recover interrupted setup grants

Setup keeps pending profiles and a permission-update intent in the private
`setup-pending-profile.json` beside the config. On a retry it checks the saved
policy revision and audit history before completing a grant. An already-applied
grant is retired without undoing a later owner revocation. Several profiles can
wait for one externally managed daemon restart.

If setup reports that audit evidence is missing or permissions changed during an
uncertain update, inspect the selected device's current permissions and history
in the panel. Resolve the named profiles to the owner's intended settings, then
archive the receipt in the private backup directory before rerunning setup.
Keep the receipt as recovery evidence; repeatedly running setup must not override
the owner's choices. If a pending profile is absent from the config, use the
specific recovery action in the error only when that profile is still intended.

## Agent inventory build: storage upgrade

The agent-spawning build upgrades SQLite `user_version` from 1 to 2 to guard the new agent-only task records. Existing tasks remain readable and are never replayed. Stop an idle daemon and take a consistent database backup alongside its config/binary before upgrading. Version-1 binaries deliberately refuse a version-2 database; do not edit the version pragma to force a downgrade. Rollback to such a binary requires its matching database/config backup and reconciliation of work accepted after that backup. Preserve newer history separately; never restore over uninspected running/interrupted work.

After upgrading, verify `agents`, the inventory collector, profile kinds, and the local `panel` session. On launchd, wait until `launchctl print gui/UID/io.citta.daemon` reports the old job removed before bootstrapping its replacement; bootout can return while unloading is still in progress.

## Permissions build: storage version 3

This build stores current policy and an append-only revision history in SQLite and upgrades storage to version 3. Older binaries refuse this database. Preserve a consistent database backup with the old binary/config before upgrading; restore matching backups only after reconciling later tasks and policy changes. Keep `control.token` and `control-enrollment.json` private and include them in protected backups. Policy updates apply live and need no service restart. Read [permissions](permissions.md) before changing unattended access or using the network panel link.

## Resource admission build: storage version 4

Resource-qualified requests are durable task input. This build upgrades SQLite `user_version` to 4 so older daemons cannot bypass the new claim predicates. The upgrade helper accepts versions 1–4 and refuses newer versions. Stop an idle service and preserve a consistent binary/config/database backup before upgrade. Never force the version backward; rollback requires the matching pre-upgrade backup and reconciliation of later work. Existing unqualified tasks and idempotency keys keep their semantics. Updated workers can still claim legacy tasks from older peers, but both ends need the new build for resource-qualified delegation.


## Desktop access from delegated workers

The installed `citta-desktop` skill supplies a reusable native adapter at `<harness-dir>/skills/citta-desktop/scripts/desktop-control.cjs`; its [usage reference](../skills/citta-desktop/references/usage.md) covers the pinned addon, first-use helper compilation and JSON commands. Ordinary setup writes only source and instructions. Compile current helper sources on first desktop use and again after source updates; setup preserves locally compiled binaries and unrelated files. No additional model API, desktop server or app-hosted tool is required for this shell-and-image path.

Use its [guarded session workflow](../skills/citta-desktop/references/usage.md#guarded-sessions) before input or capture. A per-user kernel lock admits one cooperating controller; the bounded session waits for desktop readiness and pauses on observed activity or unexpected foreground changes. Reassess before explicit resume. This is a helper workflow, not a durable scheduler or an OS sandbox: a waiting delegated task occupies a worker slot, and full-access tools remain available. Keep shell-only tasks independent of desktop readiness.

### Keeping a Mac available

For an owner-authorized always-on Mac, a separate user LaunchAgent can run `/usr/bin/caffeinate -s` with `RunAtLoad` and `KeepAlive` enabled. The `-s` system-sleep assertion applies on AC power; it does not require keeping the display on or disabling locking. Inspect existing assertions first, preserve unrelated power settings, and verify the new process with `launchctl print gui/$(id -u)/io.citta.keepawake` and `pmset -g assertions`. Keep a backup if replacing an existing plist. This job is independent of Citta and does not need a Citta restart.

A user LaunchAgent needs the user's login session. After reboot/FileVault preboot authentication or logout, normal login may still be required. Manual sleep, shutdown, loss of power and network failures can still interrupt tasks. An awake locked Mac can run ordinary shell jobs; computer use still needs a usable unlocked desktop. Keep the coordinating device awake during cross-device work as well, because losing the origin's lease interrupts execution.

To undo a separately installed `io.citta.keepawake` job, boot it out with `launchctl bootout gui/$(id -u) "$HOME/Library/LaunchAgents/io.citta.keepawake.plist"`, then restore its prior plist or remove only that newly created file. Do not alter the Citta daemon, Tailscale, display-sleep settings, lock policy or FileVault as part of this rollback.

### Verify desktop input and capture

Capture has a bounded CLI deadline and reports timeout separately from permission denial. Bound the caller’s subprocess too, since synchronous native code can block JavaScript timers. The Rust backend captures the main display; an explicit `system-window` backend can capture the owned fixture. Preserve backend errors and verify the returned image. Input results report issued actions until delivery is independently observed.

Citta Allow all and a harness's full-access sandbox mode do not grant macOS desktop permissions. Check Accessibility and Screen Recording inside the actual service-launched worker; an interactive Terminal can be authorized while an otherwise identical Node process launched through Citta is denied. Identify the actual permission-responsible executable from the system prompt or scoped OS diagnostics, then use System Settings → Privacy & Security for the owner's requested grant. Do not infer that identity solely from the immediate child process, its framework or a missing app-hosted control tool. On the qualified Mini, Accessibility appears under Device Control Data Access, and screen access is a separate Screen & System Audio Recording entry. Follow the actual OS labels; do not edit the privacy database.

Stop an idle service and restart it after permission changes, then verify the new worker. Confirm input delivery in an owned focused test window and inspect a real screenshot. Rusty Robot 0.1.2 input succeeded on the Mini, but its ScreenCaptureKit capture timed out even with permission granted. An explicitly selected `/usr/sbin/screencapture` fallback produced a valid screenshot from the same delegated worker. This is not evidence that the Rust capture backend succeeded. Do not terminate another capture session to force a check to pass.

Desktop probes are separate from external side effects. A harness run can exit successfully while its returned report says an email or UI task was blocked; inspect the result before claiming completion or retrying. A grant or service restart is not evidence that an earlier email was sent.

Report permission checks with their context: device, service-launched worker, tool/backend, observed input or capture, and task ID. Treat an unrun check as unverified. An API reporting a grant is useful diagnosis, but the functional test is the acceptance evidence. A permission granted on one device or in an interactive terminal does not establish access for another device's background worker. If changing a grant requires restarting the service, let active work finish first; restarting from inside its own worker can interrupt that task.

## App actions and account verification

For a requested external action, discover the tools and signed-in account on the receiving device before acting. Prefer an already connected app tool when it supports the task; use desktop interaction when appropriate and verify the required input/capture access there. App authentication, macOS Automation, Accessibility and Screen Recording are separate checks. Granting all desktop permissions does not connect an email account. Do not request desktop grants for a working API-only flow.

An email test should name its sender, recipient, unique subject marker and expected result. Verify the sender in the tool's account metadata or compose UI before sending; do not infer it from the default browser profile or an unrelated signed-in account. Keep unrelated accounts and mail out of the test. After sending, inspect the provider's sent record or returned message ID and retain the task ID, device and marker as evidence without copying inbox content. Provider acceptance or a Sent entry proves submission, not recipient delivery unless delivery is independently checked.

Separate outcomes in the report: `blocked_before_send`, `draft_created`, `send_confirmed` or `send_uncertain`, with the observed reason/evidence. These are reporting conventions, not new Citta task states. If a send may have happened, search the same account's sent records for the marker before retrying. A successful agent process is not itself send confirmation; an interrupted process is not proof that nothing happened. Task idempotency deduplicates submission while retained, not provider actions within a task.

## Free disk space before accepting work

Citta checks the filesystems holding its configured `data_dir` and `data_dir/work` before advertising capacity, claiming any task, creating its workspace, and starting a process. This applies to ordinary command tasks as well as agent tasks, even when CPU/memory `guard_all` is false. Unknown disk availability blocks new claims. A work directory that does not exist yet is measured using its nearest existing parent directory.

The default reserve is 64 MiB on each filesystem. Existing configurations receive this default automatically. Change it under the existing `[admission]` table, then restart an idle daemon to apply it:

```toml
[admission]
disk_headroom_bytes = 67108864
```

Use a larger reserve for workloads that create substantial temporary files. The setting must be positive; it is a minimum free-space check, not a quota or a promise that an arbitrary task will fit. Another process can consume space after a check. It does not reserve storage for model downloads or measure every filesystem a tool might write to.

`citta doctor`, device discovery and the panel report `disk_low` or `disk_unknown` with available bytes and the configured reserve. On older workers without disk telemetry, current placement reports `disk_unknown`; upgrade those workers before relying on disk-aware admission. An upgraded worker enforces the reserve itself; an older worker does not gain that behavior through pairing.

Upgraded origins also require fresh disk readiness in **every claim**, including legacy command jobs. A worker without it receives HTTP 428 (`disk_readiness_required`) and does not consume queued work. Upgrade workers before relying on a newer origin; mixed installations can otherwise leave tasks queued until their deadline. An upgraded worker serving an older origin omits the new claim field (older origins reject unknown fields), while still enforcing the disk checks locally. This compatibility behavior does not qualify an older worker for a new origin.

If disk capacity changes after a claim, the worker rechecks before creating a workspace and before starting a process. It records an interruption when the reserve is unavailable; it does not replay that task automatically. Free space through normal owner-directed cleanup, verify `citta doctor`, and inspect the original task before submitting new work. Citta never deletes personal files to make room. A failed workspace creation or full database can still prevent an older running daemon from helping remotely; recover that device locally rather than repeatedly delegating cleanup to a worker that cannot start.
