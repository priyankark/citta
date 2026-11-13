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
{"action":"verify-fixture","expect_pid":1234,"state":"/private/evidence/fixture.json","marker":"UNIQUE-MARKER"}
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


## Capture isolation on macOS

The guarded helper automatically runs each Rusty Robot screenshot in a private copy of the current Node executable. On macOS 27, concurrent capture clients sharing a Node executable path can make ReplayKit repeatedly reconnect and leave ScreenCaptureKit waiting until timeout. A separate executable path avoids that collision in the reproduced case. The copy retains the runtime's bytes and signing identity; normal permission checks still apply. This does not change TCC grants or stop other capture apps.

The helper uses an APFS clone when available and a bounded ordinary copy otherwise. Each copy operation has a two-second deadline. The private runtime is removed after capture success, error or timeout; a forcibly killed broker can leave its own temporary directory behind. Isolation failures are reported rather than retrying capture through the shared runtime. The native subprocess retains its 20-second deadline and inherited desktop lease. Status, input and the explicit `system-window` backend keep their existing execution path. This fixes the Citta helper path; applications importing Rusty Robot directly still need their own capture isolation.

Verified on an unlocked Apple Silicon Mini running macOS 27.0 (26A425), Node 24.22.0 and the installed Rusty Robot 0.1.2 package: the shared-runtime capture timed out, while five consecutive guarded captures with automatic isolation succeeded in 2.22–2.78 seconds, producing 1280×720 PNGs. The final image was inspected visually, temporary runtimes were removed, and the existing Node apps remained running. This is bounded local verification, not a macOS-wide guarantee.

## Guarded sessions

Python 3.9+ and Unix `fcntl` are required. Setup installs all portable sources; compile `desktop-context.swift` again after this update to enable HID idle observations. Credentials stay in a task-private directory, never in prompts, logs, or command arguments. A credential is a bearer capability for this cooperative helper only. Use a different nonexistent credential filename for each session.

```sh
umask 077
mkdir -p "$HOME/.local/state/citta/my-desktop-task"
env -u CITTA_DESKTOP_SESSION python3 desktop-session.py status
export CITTA_DESKTOP_SESSION="$HOME/.local/state/citta/my-desktop-task/session.json"
python3 desktop-session.py start --session "$CITTA_DESKTOP_SESSION" --ttl 300
python3 desktop-session.py wait --session "$CITTA_DESKTOP_SESSION" --timeout 30
# Only state active permits the existing JSON capture/input commands.
python3 desktop-session.py status --session "$CITTA_DESKTOP_SESSION"
python3 desktop-session.py pause --session "$CITTA_DESKTOP_SESSION"
# Reassess the owner's availability and intended foreground app before explicit resume.
python3 desktop-session.py resume --session "$CITTA_DESKTOP_SESSION"
python3 desktop-session.py end --session "$CITTA_DESKTOP_SESSION"
```

Start reserves a per-user lease even while waiting, and exits nonzero if readiness is waiting or paused. Its local credential still exists in that case; use wait/end rather than starting again. Wait emits its reason to stderr and exits nonzero on its bounded deadline or pause. Status without a credential is a read-only observation, not ownership. A lease lasts 1–3600 seconds (default 300); it does not renew automatically. The broker exits at expiry and `end` releases early. After expiry/crash, delete only your own stale credential file before starting a new session. Never unlink `lease.lock`: `flock` releases on process death, and retaining the inode prevents split ownership. The broker removes a stale socket only after owning the lock. A subprocess inherits the lock so broker death during an operation cannot admit a second controller while that operation remains alive. A native operation has a 20-second outer deadline; an operation already in flight may finish after the lease deadline, with exclusivity retained until it exits. The owner should set the caller deadline above 28 seconds. Waiting inside a Citta task consumes that task's worker slot; there is no scheduler integration.

The helper requires fresh Accessibility and Screen Recording flags, established unlock state, active displays, a known foreground PID and at least two seconds without observed HID input. It checks readiness at each action; the native subprocess checks permissions/unlock/display again and focus immediately before input. Supply `expect_pid` for every input, including `verify-fixture`. Unexpected foreground change, lock, activity or uncertain native failure pauses an active session. Resume explicitly accepts a newly observed foreground PID and activity baseline; it never repeats the previous action. Captures also require ownership and readiness.

Human detection uses CoreGraphics `CGEventSource.secondsSinceLastEventType(.hidSystemState, kCGAnyInputEventType)`. The installed Apple SDK describes this as combined hardware-source state, separate from combined session state. This is a sampled advisory idle signal, not an exclusive OS input grab: remote/accessibility/synthetic events may be missed or conservatively trigger pauses, transient foreground changes between observations can be missed, and focus or a human can change between the last check and delivery. Native input may itself cause a conservative pause; inspect and resume deliberately. There is no continuous event tap, login unlock, desktop reservation against humans, or isolation from arbitrary full-access Bash/addon imports. Locking never restricts ordinary coding/shell tasks. Do not claim unattended ownership merely from an idle timer.

If the broker crashes during a synchronous native hang, its outer subprocess watchdog is also gone. The orphaned native process retains the lock to prevent overlapping input. Inspect and terminate that specific orphan before starting another session; never delete the lock to force access. Normal broker expiry/crash with no in-flight operation releases the kernel lock automatically. The detached broker can survive cancellation of its originating task until its bounded expiry, so end the session in task cleanup when possible.

Install by backing up the existing installed `citta-desktop` and `citta-delegate` directories, copying this revision's portable skill files into them, and replacing only the compiled context helper after compiling locally. Preserve local addon/fixture binaries and OS grants. Normal future `citta setup` embeds these portable files and makes its own backups; running setup is unnecessary for a live helper-only update. Do not restart or replace Citta for this installation. Verify with `python3 test-session.py`, `node test-control.cjs`, and fresh installed `status`. Run the owned-window delivery/capture test only when unlocked and the owner has made the desktop available.

Rollback: end your session (or wait for its bounded expiry), then restore the backed-up skill directories and context binary. Remove only newly added session source/test files if absent from the backup. Do not remove the lock file while any broker or operation is alive. Uninstall the guarded helper by the same procedure; this does not uninstall the independent keep-awake LaunchAgent or change power/lock/TCC settings. Full-tool agents discover the installed skill on their next skill read; an already-running agent must reread it. Parent-side skill routing remains a separate verification.
