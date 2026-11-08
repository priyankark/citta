# Enroll a device

## One-time local setup

Install the same tested Citta build once on each macOS/Linux device, with Tailscale connected to the same owner account. Run from the installed binary:

```sh
citta setup
citta doctor
```

`setup` configures private Tailscale Serve and automatic enrollment, starts a user service, merges Codex MCP settings, installs the setup/delegation skills, and adds a bounded read-only Codex profile when a signed-in local CLI is available. It needs no SSH, pairing code or copied token. New harness sessions load the tools and skills. Other harnesses can use the same MCP server but require their own config wiring.

Commands default to `~/.config/citta/config.toml`; `--config /absolute/path/config.toml` selects another installation. `setup --dry-run` checks prerequisites without mutations. `--tailscale-bin`, `--codex-bin`, `--harness-dir`, and `--model` override discovery for a new profile. Existing identities, profiles, permissions and explicit MCP disablement are preserved; changed files are backed up under `setup-backups`. `--no-agent` intentionally omits a new agent profile. `--no-service` leaves service lifecycle to an external manager: restart it after config changes, then rerun setup to finish pending profile grants.

Read the setup result: missing login/harness means pairing can work but agent delegation is incomplete. `doctor` distinguishes missing profiles, old builds, paused devices, unreachable peers and devices waiting for pairing; an agent profile is not proof that a model run succeeded. Verify a real remote task below. Setup can be rerun after fixing prerequisites. It refuses identity changes and occupied/public Serve routes before replacing an existing config or stopping its service. It does not install or update binaries on other devices. For an older installation, use the offline upgrade procedure in [operations](operations.md), then run setup locally.

The remaining sections describe lower-level/custom setup.

## Inventory and install

Record device name, OS/architecture, intended capabilities, idle compute capacity, current Tailscale name, and the harness that will use MCP. Use distinct stable node IDs. Build/install the same tested Citta commit on each device. The initial platforms are macOS/Linux. Do not run Citta as root.

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
