# Native desktop controls

These helpers supply native desktop controls to a Citta-launched agent through its existing shell and image-viewing tools. The receiving worker needs an unlocked desktop and the applicable OS grants. Verify actual delivery and capture before using the helpers for an app task.

Use Node 18+ with the pinned `@priyankark/rusty-robot@0.1.2` package. This pinned package includes prebuilt native libraries, so no Rust build is needed. The default addon path is `~/.local/share/citta/desktop-control/node_modules/@priyankark/rusty-robot`, resolved from the receiving user’s home directory. This works when the skill is installed in `~/.codex/skills/citta-desktop` without per-call environment configuration. Verify the installed package version and native loader. To reuse a different verified install, set `CITTA_DESKTOP_ADDON=/absolute/path/to/package` explicitly. No RobotJS or system-capture fallback happens automatically.

For a new installation, check free disk space, then install the pinned prebuilt package in that private location:

```sh
npm install --prefix "$HOME/.local/share/citta/desktop-control" --ignore-scripts --save-exact @priyankark/rusty-robot@0.1.2
```

If that exact registry version is unpublished, use the same-version release archive `https://github.com/priyankark/rusty-robot/releases/download/v0.1.2/priyankark-rusty-robot-0.1.2.tgz` with the same `--prefix`, `--ignore-scripts` and `--save-exact` flags. Verify version 0.1.2 and the native loader’s integrity check. Do not silently select another version or build from source on a constrained receiver.

The commands below run from this skill’s `scripts` directory. Compile the two tiny AppKit helpers from the current bundled sources on the receiving Mac on first desktop use and again after those sources change. Ordinary Citta setup preserves locally compiled binaries but does not rebuild them. For an Apple Silicon Mac, select an explicit deployment target:

```sh
swiftc -target arm64-apple-macos14.0 desktop-context.swift -o desktop-context
swiftc -target arm64-apple-macos14.0 owned-fixture.swift -o owned-fixture
```

On an Intel Mac, use x86_64-apple-macos14.0 instead. Compile these small helpers on the receiving device, or separately verify the architecture, minimum OS, provenance and execution of transferred binaries. Replacing a permission-responsible executable can change its signing identity. Inspect the actual receiving process attribution before prescribing an OS grant; helper presence does not resolve that grant.

Requests are JSON files or stdin, avoiding shell interpolation of text. Capture requests have a 15-second CLI watchdog by default; `timeout_ms` accepts 500–60000ms. On expiry the helper emits `capture_deadline_exceeded` and exits124 immediately, ending pending native capture work. Successful or failed capture completion clears its timer. Also bound the caller’s subprocess (for example, 20seconds for the default watchdog): synchronous native code can block JavaScript timers, and the outer deadline must still end that helper. A timeout is not capture success; inspect or discard any output file before reuse. Results are JSON; failure exits nonzero. `status` reports permission flags, active display bounds, cursor, process path and current foreground PID. It does not claim actual desktop verification.

```sh
node desktop-control.cjs - <<'JSON'
{"action":"status"}
JSON
```

For onboarding, start an owned fixture with a fresh private state path and unique marker, retain its PID, and keep it running while the delegated worker checks it. Never reuse the old fixture state. `verify-fixture` refuses a stale or unfocused/nonempty fixture, types only its marker and then checks the text reported by that exact fixture process. It does not infer delivery from a successful native call.

```sh
./owned-fixture /private/evidence/fixture.json UNIQUE-MARKER
# From a separate Citta worker while this fixture remains alive:
node desktop-control.cjs - <<'JSON'
{"action":"verify-fixture","state":"/private/evidence/fixture.json","marker":"UNIQUE-MARKER"}
JSON
```

Capture only the fixture window during initial diagnosis. Use the `window_id` from its fresh state. The explicit `system-window` backend uses macOS `screencapture` in a fresh private temporary directory, validates PNG dimensions, sets mode0600 and atomically replaces the requested output. Missing or invalid new captures preserve any prior output and return failure. View the resulting file with the image-viewing tool to establish visual success. Failure is returned directly; a screenshot's existence or permission flag alone is not visual proof.

```json
{"action":"capture","backend":"system-window","window_id":123,"output":"/private/evidence/fixture.png"}
```

For subsequently authorized desktop work, `capture` defaults to Rusty Robot's main-display capture. The response supplies output dimensions and logical display bounds for coordinate mapping. Resize proportions matter: convert observed image positions back to logical display coordinates before clicking. The Rust backend captures the main display only; do not silently apply those coordinates to another display.

```json
{"action":"capture","width":1440,"timeout_ms":15000,"output":"/private/evidence/screen.png"}
{"action":"click","expect_pid":1234,"x":400,"y":250,"button":"left"}
{"action":"type","expect_pid":1234,"text":"example"}
{"action":"key","expect_pid":1234,"key":"enter","modifiers":[]}
{"action":"scroll","expect_pid":1234,"x":0,"y":-4}
```

Input rechecks the unlocked session and expected foreground application immediately before issuing the operation. This reduces stale-focus mistakes; it is not an atomic OS focus lock. Coordinates remain signed logical desktop coordinates. The caller must observe the relevant UI and confirm the intended application/field before input, and inspect the result afterward. Generic input returns `delivery_verified:false` until the caller verifies it. The helper does not establish account identity or authorize email sending. Preserve the user's specified account and existing unique action marker, and never replay an uncertain Send.

Validation performed locally: Swift compilation, Node syntax, and isolated adapter contract tests (`node test-control.cjs`), including a never-resolving native-capture stand-in whose process is terminated at the deadline and success/error cases that clear the timer. These offline checks do not establish desktop readiness on an installed device; record its functional verification separately.
