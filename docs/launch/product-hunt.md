# Product Hunt launch copy

Prepared November 10 and updated November 12, 2026 (UTC). Product Hunt has an older saved draft with the name, listing copy, logo, gallery, pricing, categories and maker comment; it is not scheduled. The agent-first copy below is a prepared revision, not yet verified as saved externally. The video and its synthetic-data/license description are staged in the editor; global **Save changes** remains unverified. [Open the draft](https://www.producthunt.com/products/citta?launch=citta) while signed in as its maker. Schedule only after the [release gates](launch-plan.md) pass. Machine-readable fields are in [listing.json](listing.json).

## Listing

**Name:** Citta

**Tagline:** An agent mesh for tools, data and compute

**Description:** Let your agents discover capabilities, delegate work and bring results back. Citta connects them to workers with access to private data, installed tools, hardware or compute—from coding tasks to local-model inference. Runs on your own Macs and Linux machines over Tailscale. Source-available for permitted noncommercial use. Demo uses synthetic data.

**Product link:** `https://github.com/priyankark/citta` — the repository is public; verify the published installation path without the maker's account before scheduling.

**Pricing:** Free. Explain that this applies to uses permitted by the noncommercial license; external model/API subscriptions and other services may cost money.

**Suggested tags:** Artificial Intelligence, Developer Tools, Productivity. These labels were verified in the saved editor. Do not select the open-source designation.

## Video caption

[Watch the demonstration](https://www.youtube.com/watch?v=EPdPu5djxSQ). The unlisted video plays while signed out, and all 45 manual-caption cues match the source SRT. Its Product Hunt attachment and adjacent description still require a verified global save.

Demonstration with synthetic data. Citta is available under PolyForm Noncommercial 1.0.0; commercial use is not licensed. See the license for permitted uses.

Place this beside the existing video if retaining its “work trip” opening. The recording demonstrates capabilities; it does not grant commercial-use permission.

## First maker comment

Hi, I'm Priyankar, the maker of Citta.

Agents can write code, work with files and use tools. But the capability a task needs may belong to another agent: access to data, an installed tool, a local model or connected hardware.

Citta is an agent mesh for discovering those capabilities, delegating tasks and collecting results. A worker can use the files and tools available to its account. With full-tool setup, that includes Bash, scripts, code execution and installed skills; those tasks run without per-command approval, so this mode is for trusted agents and devices.

The mesh runs on your Macs and Linux machines connected through Tailscale. Set up the tools and accounts each worker needs, then ask your agent to find a suitable worker and send it a task.

The demo shows both directions. An agent on a Mac Mini asks a MacBook Air to organize synthetic trip receipts. Then the Air asks the Mini to categorize the expenses with a local Gemma model. The files and the model are on different computers, but they can contribute to the same task.

The name comes from the Sanskrit word *citta* (चित्त), meaning mind or thought. It reflects the idea of agents bringing different capabilities together around a shared task.

Citta is for people experimenting with agents across their own Macs and Linux machines. The source is available under PolyForm Noncommercial 1.0.0; commercial use isn't licensed. The license also expressly permits certain organizations, including educational and charitable institutions. This is an early release, and receiving devices still need the tools, accounts and permissions for the work you ask them to do.

What capability would you want your agent to find first? I'd especially like feedback on setup: where did you get stuck, and what did you expect to happen?

## Ready answers

**Why an agent mesh?** A worker may have access to data, an installed tool, a local model, connected hardware or available compute that the requesting agent lacks. Citta exposes worker capabilities and task handoff so an agent can choose where to send new work. It does not index every file or automatically grant access to everything on a device.

**What do I need?** Two supported computers under your control, Tailscale connected on both, and Citta installed on each. For delegated agent work, the receiving computer needs a configured agent harness and access to the tools it will use. See the README for the current supported setup.

**Does Tailscale give agents access to everything?** No. It connects the machines. Citta's device settings, the receiving harness, account sign-ins, and operating-system permissions still determine what a task can do. Desktop control and screen capture require separate permission checks.

**Does it automatically move work when resources run low?** Agents can discover available workers and request placement for a new task using declared requirements. Resource estimates are admission checks, not operating-system limits. Citta does not promise automatic rerouting of every task.

**Does Citta pool RAM or move a running agent?** It sends tasks to another computer to run there. It does not combine the machines' RAM or migrate an agent's live process.

**Is everything local?** Citta has no required cloud task coordinator, and a local model can run on a receiving device. The agent harness and tools may still use cloud services. The demonstrated orchestration used cloud-backed Codex; task results can contain file contents. Choose tools and data access accordingly.

**Is the demo real?** The two directions were executed on real Macs using synthetic text receipts. Python checked the totals and duplicates; Gemma supplied categories and short notes. Waiting was shortened in the video. The Air's report save failed because its disk was full, so the returned report was saved on the Mini. Later hardware examples are illustrations, not recordings of completed hardware tasks.

**Is it open source? Can my company use it?** It is source-available under PolyForm Noncommercial 1.0.0. Commercial use is not licensed. Read the actual license for its permitted purposes and express institutional allowances; attribution alone does not grant commercial permission.

**Will it work with every agent?** Citta exposes MCP tools and command adapters. Setup currently automates the documented Codex path; other harnesses need compatible MCP configuration and a working receiving-side adapter. Installed apps and permissions are not automatically copied between computers.

**What if a device sleeps or a task stops?** Inspect the task's status and output before trying it again. Interrupted tasks may already have changed files or performed an external action. Earlier candidate checks exercised bounded daemon restart and interruption. Whole-device sleep/wake and network-loss recovery remain unverified; no result is a promise that arbitrary work is safe to replay.

## Share copy

Give your agent more ways to get a task done. Citta is an agent mesh for discovering capabilities, delegating work and bringing results back—from files and installed tools to a local model on another machine. Watch the real two-way demo and try it for a permitted noncommercial project: [insert live Product Hunt link].

This is prepared copy only. Do not send messages or publish posts as part of preparation. Ask for product feedback, not votes, following [Product Hunt's promotion guidance](https://help.producthunt.com/en/articles/17734384-5-how-to-promote-your-launch-and-what-to-avoid).
