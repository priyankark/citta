# Launch date and release gates

**User-selected target: Saturday, November 14, 2026, at 12:01 AM Pacific (08:01 UTC).** The user chose a launch four days after November 10. This replaces the earlier December proposals. Product Hunt now confirms this date on the independently reloaded launch page.

Product Hunt advises choosing a day when the product is ready and the makers can participate. Scheduled launches begin at 12:01 AM Pacific. No weekday guarantees a ranking or homepage feature. [Official timing guidance](https://help.producthunt.com/en/articles/17734355-3-set-your-goals-and-pick-your-launch-date).

Work toward November 14 while preserving the release checks below. Report any unresolved blocker before launch; do not silently move the date or claim an unqualified build passed.

## What a useful launch looks like

Lead with **“A network for agents to share tools, data and compute.”** Citta connects your AI agents into a private network, so they can use each other's tools, access data and put spare compute to work. Show how agents use the data, tools, local models or hardware available to members of the network. Explain that spare CPU and RAM are used by running a task on the machine with capacity. Tailscale connects the network across supported Macs and Linux machines, locally or across locations. Do not suggest pooled memory, migration of a running agent, automatic access to every file or unrestricted automatic routing.

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
- [x] The video has the explicit adjacent license context in [the media brief](assets.md), or its “work trip” opening has been revised to personal travel. A revoice is optional. README, video-host and saved Product Hunt maker-comment context are verified.
- [ ] Maker account/profile and launch-day availability are confirmed; actual available topic labels are selected.

The November 12 early release is public: native builds/signing/notarization, final signed two-device coding/model/recovery checks, anonymous downloads, registry installation and public Chrome ARM installation passed. Product Hunt has the saved network tagline and description. The shortened maker comment and revised gallery order have been independently verified, and Product Hunt confirms the November 14 schedule. The full clean-OS-account onboarding rehearsal remains unverified; existing-account isolation and explicit prerequisite instructions are the tested boundary.

At the November 10 preparation snapshot, CI execution was blocked by GitHub account billing/spending limits and same-build release qualification was incomplete. Track current technical status in [release readiness](../release-readiness.md). A polished launch page does not clear these gates.

## Workback schedule

| Date (Pacific) | Deliverable |
| --- | --- |
| Tue Nov 10 | Finish README, package/installer, license notices, setup recipes, gallery and listing copy. Resolve CI billing and distribution signing access. |
| Wed Nov 11 | Build the recorded candidate; run platform CI and clean-install checks. Complete two-device task, permission and local-model checks. |
| Thu Nov 12 | Exercise reconnect/restart, cancellation, upgrade/rollback and soak. Fix failures and repeat affected checks; review exact public source and archives. |
| Fri Nov 13, noon | Technical go/no-go against the evidence checklist. Report any unresolved blocker immediately. |
| Fri Nov 13, afternoon | Final public download rehearsal, listing preview, video/license context and maker account checks. Reconfirm the November 14 schedule. |
| Sat Nov 14, 00:01 | Verify the live listing, maker comment, product link and downloads. |
| Nov 14, 00:01–02:00 | Opening coverage; respond to setup questions and record failures. |
| Nov 14, 07:00–10:00 and 16:00–19:00 | Proposed maker coverage windows; reply to feedback and publish verified updates. |
| Nov 15–17 | Follow up on reproducible issues, ship tested fixes, and report outcomes. |

No automatic fallback date is selected. If a launch gate remains blocked, present the concrete evidence and the smallest remaining action.

## Reddit follow-through

The [Reddit plan and tailored drafts](reddit.md) cover a SideProject post, the AI_Agents project thread, and conditional selfhosted/coding threads. Start only when the product is ready and the maker can respond, then stagger posts to incorporate feedback. Community rules, account eligibility and the current thread must be checked before submission. The SideProject post was published November 12. A local posting run for the AI_Agents project thread is scheduled for November 13 at 09:00 Pacific; see the Reddit plan for execution requirements.

## Submission mechanics

Use the maker's personal Product Hunt account. Fill [listing.json](listing.json) and [product-hunt.md](product-hunt.md), then inspect the preview. The official editor distinguishes saving a draft from scheduling publication. Record any remaining verification limits separately from the confirmed schedule. Product Hunt accepts a direct usable/downloadable product URL, including a GitHub repository. [Submission guidance](https://help.producthunt.com/en/articles/17834537-4-build-your-product-page).

The network tagline and description are saved in Product Hunt. The unlisted YouTube video plays signed-out and all 45 manual-caption cues match the source SRT; its attachment is visible in the saved Product Hunt editor. The shortened maker comment, final gallery order and YouTube title, description and thumbnail have been independently verified. Product Hunt confirms the November 14 schedule.

The user has authorized public source, release distribution and Product Hunt scheduling for the selected date. The repository is now public after an independent history and artifact review. Product Hunt is scheduled for November 14 at 12:01 AM Pacific. Downloads are published. Reddit publication began November 12; the next local run is scheduled for November 13.


## November 12 scheduling update

The user authorized scheduling all prepared posts. Product Hunt confirms November 14, 2026 at 12:01 AM PST. Revised gallery and video media are visible on its independently reloaded page. The maker comment and YouTube description were shortened at the user's request. The personal Reddit account is signed in; its SideProject composer offers Post and Save Draft, with no scheduling control. That initial native scheduling check did not publish or schedule a Reddit post. The later user instruction moved Reddit publication forward: the SideProject post was published November 12 and a local one-time agent run is armed for the AI_Agents project thread on November 13 at 09:00 Pacific. The Mini must remain powered on and unlocked; the timer does not survive reboot. The worker checks the current thread and account eligibility and records the actual outcome without automatically replaying uncertain submissions.
