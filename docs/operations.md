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
