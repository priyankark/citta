# Launch follow-through

The first audience is people already using coding agents on two personal Macs or Linux computers. Give them one concrete reason to connect the two: use a tool, project or local model that is on the other machine.

For the first week after the November 14 launch, aim for **10 people reporting a completed remote agent/model task and 3 reporting another use**. These are targets. We do not yet have evidence of external task completions.

## Path from a post to a useful task

1. Link to the repository and its installation guide.
2. Follow [the first-task guide](../first-task.md): run an echo on the named other computer, then a small agent task.
3. Offer [setup help](https://github.com/priyankark/citta/issues/new?template=setup.yml) and [optional first-task feedback](https://github.com/priyankark/citta/issues/new?template=first-task.yml).
4. Fix the reported blocker and check whether the person can finish. Reply in the same issue with the concrete change.

## Distribution

Product Hunt is scheduled for November 14 at 12:01 AM Pacific. The prepared Reddit sequence starts with SideProject, followed by the current AI_Agents project thread after initial feedback. Check the current community rules and destination before each submission. The selfhosted and coding-thread drafts are conditional alternatives, not an instruction to publish everywhere.

Reddit posts remain unscheduled: the signed-in account's composer has no scheduling control. A written date is not a posting job. The latest authorized scope includes scheduling the prepared posts, but this repository does not install a browser autoposter or claim that future comments will run unattended.

## Daily review, November 13–21

- Read new setup reports and first-task feedback. Fix installation, pairing and agent-start failures before expanding promotion.
- Compare reach with task reports. Traffic without completions is a reason to investigate setup, not claim adoption.
- Record the published URL and time for each post. Attribute a user's source only when they tell us; GitHub referrers are aggregate context.
- Publish a follow-up when there is a useful fix or demonstrated workflow. Use an actual result, not an invented speedup or usage count.

## Measurement

Run `python3 scripts/growth_health.py`. It reads GitHub's traffic, repository counts, release archive downloads and the latest issue records, then writes a new private snapshot under `local/growth-history/`. It does not contact Citta installations, submit posts or reply to issues. `gh` must be signed in with repository traffic access.

Snapshots retain unavailable-source errors and never replace them with zeros. Traffic is a rolling window; do not add consecutive snapshots. Archive downloads include our testing and repeated downloads. First-task and repeat-use counts come from voluntary issue-form reports, excluding the repository owner. The issue window is capped at 100 records and marked when full; review beyond that window before claiming lifetime totals.

A scheduled metrics collector only gathers evidence. Reviewing feedback, fixing problems and publishing posts are separate work; report those actions only when completed.
