# Build health and failure tracking

Citta's development tracker records GitHub CI runs and individual attempts without erasing earlier failures after a successful rerun. It runs on demand and does not monitor the repository continuously.

```sh
python3 scripts/ci_health.py
python3 scripts/ci_health.py --run 42271955325
python3 scripts/test_ci_health.py
```

Use the existing `gh` login. The script reads GitHub's API, saves a private local snapshot to `local/ci-history.json` (ignored by Git), and prints the latest observed run plus failure history. Set `--state` for a different protected location. It imports earlier attempts of observed runs, so starting after a retry still preserves the original outcome. It fetches the most recent 20 runs on the selected branch; use `--run` to backfill older runs. Previously recorded attempts remain in the file. Keep this file for longitudinal history; a fresh checkout does not contain snapshots from other devices.

Categories include infrastructure failure, check failure, timeout, cancellation, and pending. An unfamiliar failed check remains `check_failure`; the tracker does not guess that a compiler or test caused it. Job/step names, bounded check annotations, commits, attempt numbers, timestamps, and links provide investigation context. It does not download complete logs or store credentials. An API failure leaves the prior snapshot intact. Run only one refresh against a given state file at a time.

A successful later attempt of the same run is linked as recovery. A different green commit does not silently resolve an earlier failure. Human investigation is still needed to determine whether a source defect was actually fixed. No automatic task/build retry or sandbox relaxation is performed.

## Recorded incidents

| Run | Observed outcome | Classification and action |
| --- | --- | --- |
| [42271582304](https://github.com/priyankark/citta/actions/runs/42271582304), attempt 1 | macOS, Linux, and dependency audit passed | Initial implementation baseline |
| [42271799418](https://github.com/priyankark/citta/actions/runs/42271799418), attempt 1 | Cancelled | Superseded development verification; not evidence of a source failure |
| [42271955325](https://github.com/priyankark/citta/actions/runs/42271955325), attempt 1 | Linux and audit passed; macOS never acquired a runner | GitHub infrastructure failure. Annotation: "The job was not acquired by Runner of type hosted even after multiple attempts" |
| Same run, attempt 2 | Targeted macOS rerun passed; complete run is green | Infrastructure incident recovered; attempt 1 remains in history |
| [42679383677](https://github.com/priyankark/citta/actions/runs/42679383677), attempt 1 | Linux and audit passed; macOS ran no steps because a hosted runner was not acquired | GitHub runner infrastructure failure, with Apple Silicon capacity reported. One targeted macOS retry was requested. |
| Same run, attempt 2 | Native ARM runner acquired; mesh test 17 timed out waiting for a permission-update response after 10 seconds; subsequent tests passed | Check failure, separate from runner availability. No permanent deadlock or root cause established. Ten fresh local ARM repetitions (70 permission POSTs) passed; this does not reproduce the preceding shared-test history. Preserve this unresolved timeout rather than treating a runner change as its fix. |
| [42684378198](https://github.com/priyankark/citta/actions/runs/42684378198), attempt 1 | Linux, native Intel macOS, ARM compile check, and audit passed | Validates the revised runner matrix. Does not establish resolution of the earlier native ARM timeout. |
| [42688766138](https://github.com/priyankark/citta/actions/runs/42688766138), attempt 1 | Native ARM, Linux, and audit passed | Supplemental native ARM qualification of the same runtime; the earlier timeout was not reproduced. |
| [42689302074](https://github.com/priyankark/citta/actions/runs/42689302074), attempt 1 | Linux, audit, every native Intel test, and release build passed; Rust ARM target download failed with DNS lookup error for `static.rust-lang.org` | Toolchain download infrastructure failure; ARM compilation did not start. Move target installation before builds so missing prerequisites surface early. Preserve this failed attempt. |

The full attempt history is preserved in the local snapshot and GitHub. A pending/rerunning check is not counted as a passing release gate.

Routine macOS CI now uses the standard Intel `macos-26-intel` runner, retaining every native test and artifact step. It also runs `cargo check --locked --target aarch64-apple-darwin` to check ARM compilation; this does not execute ARM tests. A manual CI dispatch can select `macos-26` for the same suite on native ARM. These are [GitHub's standard macOS runner labels](https://docs.github.com/en/actions/reference/runners/github-hosted-runners). Artifacts retain the actual runner OS and architecture in their names; an Intel artifact is not an Air/Apple Silicon build. Physical ARM validation remains separate from hosted CI, and changing runners does not erase the earlier infrastructure failure or guarantee runner availability.

The cold Intel job above took approximately 23 minutes including native release and ARM compilation. Its job budget is 40 minutes to accommodate those two builds; individual request/test deadlines remain unchanged.

Product task failure tracking is a separate milestone in [the plan](../plan.md#failure-visibility-and-build-history): policy denials, sandbox errors, command failures, timeouts, and interrupted tasks should have a durable CLI/MCP failure inbox. Existing task history already retains terminal outcomes; the richer inbox is planned, not implemented.

## Device integration incidents

- 2026-11-07, automatic enrollment build `308c92a`: the macOS launchd check failed closed because the bundled Tailscale executable selected GUI mode without a terminal environment. The follow-up fix sets `TAILSCALE_BE_CLI=1` only for Tailscale subprocesses and adds a regression fixture that requires it. This is a runtime integration defect, not a CI runner failure. The original observation remains in the private device acceptance record. See [Tailscale CLI guidance](https://tailscale.com/docs/reference/tailscale-cli).
