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

## Agent inventory build: storage upgrade

The agent-spawning build upgrades SQLite `user_version` from 1 to 2 to guard the new agent-only task records. Existing tasks remain readable and are never replayed. Stop an idle daemon and take a consistent database backup alongside its config/binary before upgrading. Version-1 binaries deliberately refuse a version-2 database; do not edit the version pragma to force a downgrade. Rollback to such a binary requires its matching database/config backup and reconciliation of work accepted after that backup. Preserve newer history separately; never restore over uninspected running/interrupted work.

After upgrading, verify `agents`, the inventory collector, profile kinds, and the local `panel` session. On launchd, wait until `launchctl print gui/UID/io.citta.daemon` reports the old job removed before bootstrapping its replacement; bootout can return while unloading is still in progress.

## Permissions build: storage version 3

This build stores current policy and an append-only revision history in SQLite and upgrades storage to version 3. Older binaries refuse this database. Preserve a consistent database backup with the old binary/config before upgrading; restore matching backups only after reconciling later tasks and policy changes. Keep `control.token` and `control-enrollment.json` private and include them in protected backups. Policy updates apply live and need no service restart. Read [permissions](permissions.md) before changing unattended access or using the network panel link.

## Resource admission build: storage version 4

Resource-qualified requests are durable task input. This build upgrades SQLite `user_version` to 4 so older daemons cannot bypass the new claim predicates. The upgrade helper accepts versions 1–4 and refuses newer versions. Stop an idle service and preserve a consistent binary/config/database backup before upgrade. Never force the version backward; rollback requires the matching pre-upgrade backup and reconciliation of later work. Existing unqualified tasks and idempotency keys keep their semantics. Updated workers can still claim legacy tasks from older peers, but both ends need the new build for resource-qualified delegation.
