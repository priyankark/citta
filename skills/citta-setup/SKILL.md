---
name: citta-setup
description: Install, pair, repair, and verify Citta on macOS or Linux devices so agents can discover remote capabilities and delegate tasks across a trusted Tailscale mesh. Use for Citta device enrollment and MCP wiring, not unrelated Tailscale administration.
---

# Citta device setup

Set up the requested device and prove that a task executes on a different device. Citta's private repository is `priyankark/citta`. Find an existing checkout first; otherwise clone it with the user's established GitHub credentials. Use the same reviewed commit as the other devices. If no release is tagged, identify the commit being qualified; do not call it a production release.

Read the checkout's `docs/setup.md` for commands, `docs/capabilities.md` when adding a worker, and `docs/acceptance.md` for the rollout evidence. These files describe the actual installed version; prefer them to assumptions about CLI flags. Read `docs/operations.md` for repair, rotation, upgrades, or rollback.

## Establish existing state

Inspect OS/architecture, Citta binary/version, current config/service, Tailscale connection and Serve routes, and intended harness/capabilities. Discover device IDs and URLs from existing configuration where available. Ask only for missing device access or capability choices needed to finish. Continue local installation and validation while waiting for remote access.

Keep the existing identity, secrets, peers, and unrelated harness/network settings. `citta init` is for new installations and refuses to overwrite. A startup lock failure usually means a daemon is already running. Do not remove locks or reset a database to bypass it.

## Install and pair

1. Build and install from the reviewed checkout with `cargo install --path . --locked`. Initialize through the installed binary using an absolute private config directory and a unique stable node ID. Run `citta --config <absolute-config> check`.
2. Configure a user service with `scripts/render_service.py`; inspect the generated definition and enable the service within the requested setup scope. Reuse an existing service when repairing. Do not run the daemon as root.
3. Enable automatic enrollment with `citta --config <path> enroll --tailscale-bin <absolute executable>`, using the same HTTPS port on all devices. The command checks Tailscale identity, preserves unrelated Serve routes, refuses a conflicting port or Funnel, and pins this owner/tailnet/device. Start or restart the existing daemon. Do not generate pairing bundles, copy peer secrets, ask for pairing codes, or ask the user to relay instructions between agents. Enrollment is the daemon's job.
4. Use `citta enrollment` and discovery to verify automatic reciprocal pairing. The daemon accepts only untagged, unexpired devices owned by the pinned account, exchanges per-peer credentials through verified HTTPS challenges, and refreshes them without restarts. It fails closed when identity is stale. Existing manual peers are compatibility grants, not proof that automatic enrollment succeeded. Both endpoints must run a supporting build. If a device needs installation/upgrade and has no agent or management channel, report that deployment blocker accurately; do not substitute a human token exchange or claim the software installed itself.
5. Start with `echo`, then add only the requested trusted capabilities with conservative slots/timeouts. A capability reads JSON stdin, uses fixed operator-configured argv, and returns bounded UTF-8 stdout/stderr. No mesh credentials go to job processes. A work directory is not a security sandbox. Preserve the harness's normal approval and sandbox behavior.
6. Merge a local stdio MCP entry into the intended harness configuration, preserving other entries. Use the absolute Citta binary and `--config <absolute-config> mcp`; do not embed tokens. Verify actual tool discovery in the harness when access permits.

## Prove the result

Require the automatic enrollment report to list the other device as paired with fresh Tailscale identity, then run discovery in both directions. Submit a unique-key `echo` task with `target_node` set to the other device; inspect it on its originating device. Require the expected remote `worker_node`, successful result, and echoed payload. Repeat in the reverse direction. Local echo success alone is insufficient.

Next, discover and use one real capability installed only on the remote device through MCP. Record the tested commit, device IDs/OS, task IDs, and observed result without secrets. Follow `docs/acceptance.md` for interruption, sleep/wake, restart, and capacity checks as part of the requested rollout.

Reusing the same submission key is safe for a transport retry while history is retained. An interrupted task may have performed side effects: inspect before intentional resubmission with a new key. Do not change keys just to make a failed check appear successful.

For resource/session discovery, agent profiles, and the human panel, read `docs/discovery.md`. Declare capability `harness` and `model` metadata only when known; keep model metadata consistent with a pinned runtime configuration. Verify fresh CPU/RAM telemetry, the harness collector status, harness/model fields, and profile kinds through `citta agents`; absent/old telemetry is unknown. Test a requested profile with `spawn-agent` and inspect the durable result. Preserve the harness sandbox/approval policy and disable recursive Citta access in spawned profiles until budgets exist. For owner controls, read `docs/permissions.md`. Verify automatic management pairing as well as task pairing; older peers need the same permissions-enabled build. Use `panel --network` for a private HTTPS browser link from another device. Keep management credentials separate from MCP/task credentials, preserve existing policy, and do not enable allow-all merely to make a test pass. Issue panel links without logging them; they are short-lived owner credentials. An idle observed session is not permission to take it over.

Finish with installed paths, service state, paired device IDs, tested task IDs, and specific outstanding device checks. If remote access is unavailable, state that local setup is verified and cross-device acceptance remains pending; never report enrollment complete from a localhost test.
