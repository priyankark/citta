# Launch date and release gates

**User-selected target: Saturday, November 14, 2026, at 12:01 AM Pacific (08:01 UTC).** The user chose a launch four days after November 10. This replaces the earlier December proposals. No date has been scheduled externally.

Product Hunt advises choosing a day when the product is ready and the makers can participate. Scheduled launches begin at 12:01 AM Pacific. No weekday guarantees a ranking or homepage feature. [Official timing guidance](https://help.producthunt.com/en/articles/17734355-3-set-your-goals-and-pick-your-launch-date).

Work toward November 14 while preserving the release checks below. Report any unresolved blocker before launch; do not silently move the date or claim an unqualified build passed.

## What a useful launch looks like

Lead with **“An agent mesh for tools, data and compute.”** Show an agent discovering a capability, delegating a task and collecting a result. Explain why that worker is useful: it has access to the data, tool, local model or hardware the task needs. Then explain the supported computers, Tailscale connection and receiving-side accounts and permissions. Do not suggest pooled memory, migration of a running agent or unrestricted automatic routing.

Proposed targets: 10 people report completing their first task across two devices, and 15 actionable pieces of setup feedback. These are planning targets, not forecasts. Count voluntary feedback and downloads; do not introduce silent telemetry or imply downloads are successful installations.

## Before scheduling

- [x] A specific candidate commit and archive hashes are recorded.
- [x] Required CI checks run and pass. Billing or runner failures are resolved, not described as code passes.
- [x] All advertised platforms have usable packages and documented requirements.
- [ ] A person without the maker's repository credentials can download, install, enroll two devices and complete a task using only the published guide.
- [x] The same candidate passes physical coding/model acceptance, restart/reconnect of an owned test endpoint, incorrect-token rejection and expected cancellation/interruption reporting. Whole-device sleep/network loss is an explicit early-release limitation until a safe independent recovery path is available; do not claim it passed.
- [x] macOS signing, notarization and upgrade identity are qualified, or the actual early-release limitations are explicit and the installation path is tested. Do not advertise a signed/notarized build without evidence.
- [x] Desktop helpers in the release match the reviewed source. Onboarding explains receiving-worker grants, and unverified signed-worker desktop continuity is explicitly labeled experimental. A permission flag alone is not desktop acceptance.
- [x] The local-model example has a working setup recipe and lists model/runtime prerequisites.
- [x] Published source/history and archive contents are reviewed for private material; LICENSE, NOTICE and required dependency notices ship with downloads.
- [ ] The public product link, download links, logo, four gallery images, final video link and captions work from a signed-out browser.
- [ ] The video has the explicit adjacent license context in [the media brief](assets.md), or its “work trip” opening has been revised to personal travel. A revoice is optional. README/video-host context is verified; the Product Hunt editor still needs its global save.
- [ ] Maker account/profile and launch-day availability are confirmed; actual available topic labels are selected.

The November 12 early release is public: native builds/signing/notarization, final signed two-device coding/model/recovery checks, anonymous downloads, registry installation and public Chrome ARM installation passed. Product Hunt still requires saving the revised copy/gallery and verifying the scheduled date. The full clean-OS-account onboarding rehearsal remains unverified; existing-account isolation and explicit prerequisite instructions are the tested boundary.

At the November 10 preparation snapshot, CI execution was blocked by GitHub account billing/spending limits and same-build release qualification was incomplete. Track current technical status in [release readiness](../release-readiness.md). A polished launch page does not clear these gates.

## Workback schedule

| Date (Pacific) | Deliverable |
| --- | --- |
| Tue Nov 10 | Finish README, package/installer, license notices, setup recipes, gallery and listing copy. Resolve CI billing and distribution signing access. |
| Wed Nov 11 | Build the recorded candidate; run platform CI and clean-install checks. Complete two-device task, permission and local-model checks. |
| Thu Nov 12 | Exercise reconnect/restart, cancellation, upgrade/rollback and soak. Fix failures and repeat affected checks; review exact public source and archives. |
| Fri Nov 13, noon | Technical go/no-go against the evidence checklist. Report any unresolved blocker immediately. |
| Fri Nov 13, afternoon | Final public download rehearsal, listing preview, video/license context and maker account checks. Schedule the complete listing for November 14. |
| Sat Nov 14, 00:01 | Verify the live listing, maker comment, product link and downloads. |
| Nov 14, 00:01–02:00 | Opening coverage; respond to setup questions and record failures. |
| Nov 14, 07:00–10:00 and 16:00–19:00 | Proposed maker coverage windows; reply to feedback and publish verified updates. |
| Nov 15–17 | Follow up on reproducible issues, ship tested fixes, and report outcomes. |

No automatic fallback date is selected. If a launch gate remains blocked, present the concrete evidence and the smallest remaining action.

## Submission mechanics

Use the maker's personal Product Hunt account. Fill [listing.json](listing.json) and [product-hunt.md](product-hunt.md), then inspect the preview. The official editor distinguishes saving a draft from scheduling publication; keep the page as a draft while incomplete. Product Hunt accepts a direct usable/downloadable product URL, including a GitHub repository. [Submission guidance](https://help.producthunt.com/en/articles/17834537-4-build-your-product-page).

The agent-first listing revision is prepared in the local copy files. The unlisted YouTube video plays signed-out and all 45 manual-caption cues match the source SRT. The video and its adjacent synthetic-data/license description are staged in Product Hunt, with the global save still unverified. Verify the final copy, gallery and media again after **Save changes**, then inspect the preview.

The user has authorized public source, release distribution and Product Hunt scheduling for the selected date. The repository is now public after an independent history and artifact review. Product Hunt has a saved draft; it is not scheduled. Downloads are published; verify the saved video and listing before scheduling. No outreach is planned.
