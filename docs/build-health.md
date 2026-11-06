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

The full attempt history is preserved in the local snapshot and GitHub. A pending/rerunning check is not counted as a passing release gate.

Product task failure tracking is a separate milestone in [the plan](../plan.md#failure-visibility-and-build-history): policy denials, sandbox errors, command failures, timeouts, and interrupted tasks should have a durable CLI/MCP failure inbox. Existing task history already retains terminal outcomes; the richer inbox is planned, not implemented.
