# Enroll a device

## One-time local setup

Install the same tested Citta build once on each macOS/Linux device, with Tailscale connected to the same owner account. Run from the installed binary:

```sh
citta setup
citta doctor
```

`setup` configures private Tailscale Serve and automatic enrollment, starts a user service, merges Codex MCP settings, installs the setup/delegation skills, and adds a full-tool Codex profile when a signed-in local CLI is available. It needs no SSH, pairing code or copied token. New harness sessions load the tools and skills. Other harnesses can use the same MCP server but require their own config wiring.

### Choose what this device should do

Give the setup agent the intended use, such as “coding and tests” or “use my browser to send email from my personal account.” It can check existing access and explain any remaining step before the first real task. Permissions belong to the device doing the work; granting access on your laptop does not grant it on another computer.

| Intended use | Access needed on the receiving device | What proves it works |
| --- | --- | --- |
| Discover your other devices | Tailscale signed in to the same owner account, private Serve available, Citta set up on each device | Fresh pairing and an echo task executed on the named other device |
| Run shell commands, scripts, code edits and tests | Signed-in agent harness, installed tools, a permitted full-tool profile, access to the working files | A delegated worker writes a small test file and runs it |
| Use connected apps, MCP tools or skills | Those tools/skills installed for the worker's account; the requested app account connected there | The worker discovers and uses the required tool with the intended account |
| Click and type in desktop apps | An unlocked desktop session and macOS Accessibility access for the process identified by the system | A service-launched worker types a unique marker in an owned test window |
| See the desktop | macOS Screen Recording access for the process identified by the system | A service-launched worker captures an image and checks its contents |
| Control an app through Apple events | The specific macOS Automation grant requested for that app, if this method is used | The worker performs the intended app operation |

For macOS desktop work, use **System Settings → Privacy & Security** and follow the labels on that OS version. Accessibility may appear as **Device Control Data Access**; screen access may appear as **Screen & System Audio Recording**. The permission prompt or test should identify the executable that needs access. In the qualified Citta service setup this was the installed `citta` executable, commonly `~/.cargo/bin/citta`. Authorizing Terminal alone did not authorize its background worker. After a grant, restart the idle service and repeat the worker test; see [desktop access](operations.md#desktop-access-from-delegated-workers).

A macOS upgrade can change the executable's signing identity even at the same path, so existing desktop grants may need renewal. Check [signing identity and desktop access](operations.md#macos-signing-identity-and-desktop-access) before replacement, and verify the receiving service worker afterward. Keep the candidate staged if required OS approval cannot be completed on that device.

Coding does not require desktop permissions. A connected email API may work without any desktop permission; browser clicking needs its own input/capture checks. Add access for the chosen method as needed. Full Disk Access, SSH and public Funnel are not general onboarding prerequisites. Linux desktop access depends on the session and automation backend; the macOS steps do not apply there.

### Know when setup is complete

Readiness has three useful milestones:

1. **Paired:** the other device is reachable with fresh identity.
2. **Agent configured:** a profile is installed. Check `doctor` and placement for whether its execution policy and current capacity permit the requested task.
3. **Task verified:** a worker on that device performed the requested kind of work, including any app account or desktop access it needs.

An untested permission is **not verified**, rather than automatically denied or granted. `setup` and `doctor` describe configuration and connectivity; they do not certify every app login or desktop permission. Keep the remaining action concrete, for example “sign in to the personal email account on the Air” or “grant screen access to the Citta service on the Air, then rerun the capture check.”

Setup's `next_steps` lists unresolved configuration actions. Its `permissions` guide explains execution access, task-specific accounts and desktop checks; `permissions.desktop.service_executable` identifies the executable for setup’s managed service. It is null for `--no-service` and `doctor`; inspect the installed service definition in those cases. `invoked_executable` is the CLI that ran the check, which may differ from the service. `execution_verified: false` and `desktop_verified: false` mean setup has not run those functional tests, even if a previous task worked. Doctor includes the local permission guide and marks each device's execution/desktop verification as untested by that command. Use the actual task evidence to report what has been verified.

Commands default to `~/.config/citta/config.toml`; `--config /absolute/path/config.toml` selects another installation. `setup --dry-run` checks prerequisites without mutations. `--tailscale-bin`, `--codex-bin`, `--harness-dir`, and `--model` override discovery for a new profile. Existing identities, profiles, permissions and explicit MCP disablement are preserved; changed files are backed up under `setup-backups`. `--no-agent` intentionally omits a new agent profile. `--no-service` leaves service lifecycle to an external manager: restart it after config changes, then rerun setup to finish pending profile grants.

The default `codex` profile uses `danger-full-access` and approval policy `never`, with the user’s configured tools, MCP servers and skills. It can run shell commands, write code, run tests, access the network and delegate further. Profiles use the node’s slot count and allow tasks up to 24 hours; agent requests default to one hour. `setup --agent-access full` adds this profile to an existing device without rewriting its other profiles; `--agent-access read-only` creates an explicit `codex-readonly` alternative. An existing profile with the selected name is preserved. Child tasks have their own durable lifecycle and need explicit collection/cancellation; parent cancellation does not cascade.

Read the setup result: missing login/harness means pairing can work but agent delegation is incomplete. `doctor` distinguishes missing profiles, old builds, paused devices, unreachable peers and devices waiting for pairing; an agent profile is not proof that a model run succeeded. Verify a real remote task below. Setup can be rerun after fixing prerequisites. It refuses identity changes and occupied/public Serve routes before replacing an existing config or stopping its service. It does not install or update binaries on other devices. For an older installation, use the offline upgrade procedure in [operations](operations.md), then run setup locally.

For an authorized email test, specify the sending account and recipient in the delegated task. The worker must verify the account before composing, send one message with a unique test marker, and check the provider's sent record or message identifier. Keep a draft, a successful agent exit and a confirmed send distinct. If the result is interrupted or unclear, inspect that same account's sent record before another attempt; Citta's submission key prevents duplicate tasks while history is retained, but does not make email sending exactly once. See [app actions and account verification](operations.md#app-actions-and-account-verification).

The remaining sections describe lower-level/custom setup.

## Inventory and install

Record device name, OS/architecture, intended capabilities, idle compute capacity, current Tailscale name, and the harness that will use MCP. Use distinct stable node IDs. Install a verified artifact or build the same tested Citta commit on each device. Before compiling, check the compiler and available disk space using the [build and artifact preflight](operations.md#build-and-artifact-preflight). The initial platforms are macOS/Linux. Do not run Citta as root.

```sh
cargo install --path . --locked
citta init --dir "$HOME/.config/citta" --node DEVICE_ID
citta --config "$HOME/.config/citta/config.toml" check
```

If a config already exists, inspect it and preserve its identity, secret, peers, and capabilities. Never run `init` to repair an existing installation. Stop/restart the daemon to apply changes; hot reload is not implemented.

## Automatic enrollment — default setup

Once Citta and Tailscale are installed and Tailscale is signed in, the setup agent enables enrollment once on each device:

```sh
citta --config "$HOME/.config/citta/config.toml" enroll \
  --tailscale-bin /absolute/path/to/tailscale
```

On macOS the executable may be `/Applications/Tailscale.app/Contents/MacOS/Tailscale`. This command pins the current Tailscale account, tailnet and device identity, reuses a matching private Serve HTTPS route or adds one on unused port 7332, and preserves peers/capabilities. It refuses a conflicting route or public Funnel on that port. Use the same `--https-port` on participating devices. Start or restart the existing daemon after enabling it.

The daemon discovers same-owner, untagged devices from local `tailscale status --json`, performs reciprocal HTTPS challenge handshakes, stores private per-peer credentials, and updates workers without restarts. No pairing codes, token transfers, Taildrop bundles, human confirmation, or relayed agent instructions are part of pairing. Devices must already run an enrollment-capable build; this is not a remote software installer. Tailscale sign-in and tailnet access policy remain external prerequisites.

Inspect progress through the agent tools (`citta_discover`) or CLI:

```sh
citta --config "$HOME/.config/citta/config.toml" enrollment
citta --config "$HOME/.config/citta/config.toml" discover
```

A waiting peer may be offline, running an older build, missing enrollment configuration, blocked by tailnet policy, or using a different HTTPS port. Fix it through the available device agent/management channel. Do not fall back to asking the user to copy credentials. No remote execution channel is implicitly added by enrollment.

Trust is limited to the pinned owner and tailnet. Other users' devices, tagged devices and expired identities are excluded. Stale/unavailable Tailscale identity disables automatic credentials until a fresh check succeeds. Automatic enrollment grants trusted-mesh API access and execution of locally installed capabilities; it does not add capabilities, enable a shell, change task/harness approvals, or implement OS sandboxing. See [the protocol](protocol.md#automatic-enrollment) for the trust boundary.

## Explicit peers — compatibility mode

Existing `[[peers]]` entries still work. Their tokens remain explicit trust grants independent of automatic enrollment and are not revoked by netmap removal. Keep existing entries during a staged upgrade; migrate them only after both devices successfully enroll. Do not exchange new manual tokens during ordinary device setup.

A legacy peer has a stable `node_id`, HTTPS origin `url`, and private absolute `token_file`. Its copied node token grants full node access. Revoke a legacy token by rotating the owning node's token and removing obsolete copies; removing an entry alone does not revoke a copied token.

## Install a persistent service

The renderer writes a new service file; it does not enable it or overwrite an existing file. Resolve actual binary/config paths first. On macOS:

```sh
mkdir -p "$HOME/Library/LaunchAgents"
python3 scripts/render_service.py --platform macos \
  --binary "$HOME/.cargo/bin/citta" \
  --config "$HOME/.config/citta/config.toml" \
  --output "$HOME/Library/LaunchAgents/io.citta.daemon.plist"
plutil -lint "$HOME/Library/LaunchAgents/io.citta.daemon.plist"
launchctl bootstrap "gui/$(id -u)" "$HOME/Library/LaunchAgents/io.citta.daemon.plist"
```

On Linux:

```sh
mkdir -p "$HOME/.config/systemd/user"
python3 scripts/render_service.py --platform linux \
  --binary "$HOME/.cargo/bin/citta" \
  --config "$HOME/.config/citta/config.toml" \
  --output "$HOME/.config/systemd/user/citta.service"
systemctl --user daemon-reload
systemctl --user enable --now citta.service
```

A user service is scoped to that user's session. For operation after logout, choose the OS-supported account/service lifecycle deliberately; Linux user lingering is a separate account setting. The daemon's job guardian provides parent-death cleanup independently of the service manager. Linux `KillMode=control-group` adds service-level cleanup. These are not security sandboxes for hostile code.

## Verify the cross-device path

From laptop, submit to desktop explicitly:

```sh
printf '%s\n' '{"from":"laptop","test":"pairing"}' | \
  citta --config "$HOME/.config/citta/config.toml" submit \
  --capability echo --target desktop --input - --key pairing-UNIQUE_KEY --timeout 20
```

Inspect the returned UUID on laptop. Require `succeeded`, `origin_node = laptop`, `worker_node = desktop`, and the expected echoed JSON. Repeat the reverse direction with a new key. A local echo without `--target` does not prove device pairing.

Then install one useful capability on only one device, attach MCP on the other, discover it, delegate to it, and retrieve its result through MCP. Record commit, device IDs, task IDs, timestamps, and observed results in a private acceptance record. Use [acceptance.md](acceptance.md) for sleep/wake and failure checks.

## Roll back enrollment

Stop the service (`launchctl bootout` or `systemctl --user disable --now citta`). Disable only Citta's Serve port using `tailscale serve --https=7332 off`; never reset all Serve configuration. Disable enrollment on both devices or remove the device from the trusted Tailscale account; remove any legacy peer entries and rotate their copied node credentials. Automatic enrollment will otherwise re-establish trust on the next handshake. Preserve the database until task/result retention is resolved. Revoke a leaked token by replacing the node token and its authorized peer copies, then restart affected daemons.

## Verify resource discovery and monitoring

After enrollment, run `citta --config <path> agents`. Verify fresh hardware telemetry and distinguish observed harness sessions from managed jobs. Configure only requested agent profiles, test `spawn-agent`, and inspect its durable result. Run `panel` for a local browser login link. See [discovery and monitoring](discovery.md); older devices need an upgrade before they report telemetry or support dedicated spawning.
