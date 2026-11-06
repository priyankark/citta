# Citta development

Citta is a cross-device Rust runtime targeting production use. Keep guarantees tied to verified behavior; `plan.md` distinguishes current functionality from planned sandbox/policy work.

When changing Citta or handing off a build, refresh CI with `python3 scripts/ci_health.py` if GitHub access is available. Read `docs/build-health.md` and the resulting `local/ci-history.json`. Preserve failed attempts and inspect failed steps/annotations before classifying a failure. Distinguish source/test failures, runner infrastructure failures, deliberate cancellations, and pending checks. If access fails, report stale/unavailable status rather than claiming green.

Run checks appropriate to the change. A CI infrastructure failure may receive one targeted retry after diagnosis; do not loop on unavailable runners. Inspect and fix source failures before retrying. Never merge unrelated dependency updates merely to clear a failing check.

Task interruption may mean external side effects already happened. Never automatically resubmit such tasks or weaken sandbox settings to make a task pass. Device acceptance remains required before a production release tag.
