# Enroll a device

## Inventory and install

Record device name, OS/architecture, intended capabilities, idle compute capacity, current Tailscale name, and the harness that will use MCP. Use distinct stable node IDs. Build/install the same tested Citta commit on each device. The initial platforms are macOS/Linux. Do not run Citta as root.

```sh
cargo install --path . --locked
citta init --dir "$HOME/.config/citta" --node DEVICE_ID
citta --config "$HOME/.config/citta/config.toml" check
```

If a config already exists, inspect it and preserve its identity, secret, peers, and capabilities. Never run `init` to repair an existing installation. Stop/restart the daemon to apply changes; hot reload is not implemented.

## Tailscale transport

Citta does not provision Tailscale accounts or change tailnet policy. Inspect existing `tailscale status` and `tailscale serve status` first. Obtain the CLI path if a macOS app installation does not put it on PATH. On a connected device, expose Citta's loopback port through a dedicated, unused Serve HTTPS port:

```sh
tailscale serve --bg --https=7332 http://127.0.0.1:7331
tailscale serve status
```

Use the exact HTTPS URL reported by Serve, including its port. Keep other Serve routes intact. Enabling HTTPS may require the account owner's tailnet setting change. Never enable Funnel. Limit access to participating devices/users with the tailnet's grants or ACLs. Tailscale Serve supplies HTTPS within the tailnet; Citta additionally requires its bearer secret. [Tailscale Serve reference](https://tailscale.com/docs/reference/tailscale-cli/serve).

## Reciprocal pairing

Each node has its own `node.token`. Transfer a copy to the authorized peer through an existing authenticated channel such as SSH/SCP. Place it in the peer's private Citta directory with mode 0600. Do not print tokens, paste them into chat, commit them, or put them in process arguments. A token grants full access to its node within this trusted mesh.

On **laptop**, add a desktop peer:

```toml
[[peers]]
node_id = "desktop"
url = "https://desktop.YOUR-TAILNET.ts.net:7332"
token_file = "/absolute/private/citta/desktop.token"
```

On **desktop**, add the reciprocal laptop entry using the laptop's HTTPS URL and token file. The worker's origin must list that worker as a peer before it can claim work. Configuration permits at most 64 direct peers; no automatic transitive discovery or tailnet scanning occurs.

Run `check` on both nodes and restart each daemon. Run `discover` on both. Both peer entries must be reachable, and the reported node IDs must match configuration. If a peer fails, check service status, Serve URL, TLS, tailnet access policy, token file, and reciprocal IDs. Do not disable TLS verification or bind Citta publicly to work around connectivity.

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

Stop the service (`launchctl bootout` or `systemctl --user disable --now citta`). Disable only Citta's Serve port using `tailscale serve --https=7332 off`; never reset all Serve configuration. Remove reciprocal peer entries and their copied credentials. Preserve the database until task/result retention is resolved. Revoke a leaked token by replacing the node token and its authorized peer copies, then restart affected daemons.
