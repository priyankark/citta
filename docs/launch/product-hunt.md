# Product Hunt launch copy

Updated November 12, 2026 (UTC). Product Hunt confirms the launch is scheduled for **November 14 at 12:01 AM Pacific**. The revised gallery order and shortened maker comment have been verified in freshly opened Product Hunt pages. [Open the scheduled launch](https://www.producthunt.com/products/citta?launch=citta). Machine-readable fields are in [listing.json](listing.json).

## Listing

**Name:** Citta

**Tagline:** A network for agents to share tools, data and compute

**Description:** Citta connects your AI agents into a private network, so they can use each other's tools, access data and put spare compute to work. Each agent brings what it has access to: private files, installed tools, local models or connected hardware. When a task needs more capacity, your agent can send it to a member of the network with the resources to run it.

**Product link:** `https://github.com/priyankark/citta`; the repository is public; anonymous downloads and public installation have been verified.

**Pricing:** Free. Explain that this applies to uses permitted by the noncommercial license; external model/API subscriptions and other services may cost money.

**Suggested tags:** Artificial Intelligence, Developer Tools, Productivity. These labels were verified in the saved editor. Do not select the open-source designation.

## Video caption

[Watch the demonstration](https://www.youtube.com/watch?v=EPdPu5djxSQ). The unlisted video plays while signed out, and all 45 manual-caption cues match the source SRT. Its attachment is visible in the saved Product Hunt editor. Keep demo context with the video or maker comment, rather than in the short product description.

Demonstration with synthetic data. Citta is available under PolyForm Noncommercial 1.0.0; commercial use is not licensed. See the license for permitted uses.

Place this beside the existing video if retaining its “work trip” opening. The recording demonstrates capabilities; it does not grant commercial-use permission.

## First maker comment

Hi, I'm Priyankar. I built Citta so agents on different computers can work together.

An agent can discover the tools, models and compute available on your other machines, delegate a task and collect the result. You can monitor the work and set each device's permissions from a control panel.

In the demo, an agent on my Mac Mini asks my MacBook Air to organize sample receipts. The Air then sends expense categorization back to the Mini, where Gemma runs locally.

Citta is written in Rust and connects Macs and Linux machines through Tailscale. It exposes MCP tools and command adapters; setup currently handles Codex, with other harnesses requiring configuration.

Install it on two machines and try a task: https://github.com/priyankark/citta

This is an early release, available under PolyForm Noncommercial 1.0.0. Commercial use isn't licensed.

## Ready answers

**Why a network for agents?** Each member can bring access to different data, tools, models, hardware or compute. Agents can discover what is available and send work to the member that can do it. Network membership does not automatically grant access to every file or app.

**What do I need?** Two supported computers under your control, Tailscale connected on both, and Citta installed on each. For delegated agent work, the receiving computer needs a configured agent harness and access to the tools it will use. See the README for the current supported setup.

**Does Tailscale give agents access to everything?** No. It connects the machines. Citta's device settings, the receiving harness, account sign-ins, and operating-system permissions still determine what a task can do. Desktop control and screen capture require separate permission checks.

**Does it automatically move work when resources run low?** Agents can discover available workers and request placement for a new task using declared requirements. Resource estimates are admission checks, not operating-system limits. Citta does not promise automatic rerouting of every task.

**How does sharing compute and memory work?** A task uses the CPU and RAM of the machine where it runs. Citta can place a new task on a member with enough resources. It does not pool RAM or move an agent's running process.

**Is it limited to my local network?** No. Tailscale connects your devices privately, including across locations. The machines still need Citta, their tools and the required access configured.

**Is everything local?** Citta has no required cloud task coordinator, and a local model can run on a receiving device. The agent harness and tools may still use cloud services. The demonstrated orchestration used cloud-backed Codex; task results can contain file contents. Choose tools and data access accordingly.

**Is the demo real?** The two directions were executed on real Macs using synthetic text receipts. Python checked the totals and duplicates; Gemma supplied categories and short notes. Waiting was shortened in the video. The Air's report save failed because its disk was full, so the returned report was saved on the Mini. Later hardware examples are illustrations, not recordings of completed hardware tasks.

**Is it open source? Can my company use it?** It is source-available under PolyForm Noncommercial 1.0.0. Commercial use is not licensed. Read the actual license for its permitted purposes and express institutional allowances; attribution alone does not grant commercial permission.

**Will it work with every agent?** Citta exposes MCP tools and command adapters. Setup currently automates the documented Codex path; other harnesses need compatible MCP configuration and a working receiving-side adapter. Installed apps and permissions are not automatically copied between computers.

**What if a device sleeps or a task stops?** Inspect the task's status and output before trying it again. Interrupted tasks may already have changed files or performed an external action. Earlier candidate checks exercised bounded daemon restart and interruption. Whole-device sleep/wake and network-loss recovery remain unverified; no result is a promise that arbitrary work is safe to replay.

## Share copy

Citta connects your AI agents into a private network, so they can use each other's tools, access data and put spare compute to work. Code and setup: https://github.com/priyankark/citta

This is prepared copy only. Do not send messages or publish posts as part of preparation. Ask for product feedback, not votes, following [Product Hunt's promotion guidance](https://help.producthunt.com/en/articles/17734384-5-how-to-promote-your-launch-and-what-to-avoid).
