# Product Hunt launch copy

Updated November 12, 2026 (UTC). This revision presents Citta as a private network for agents to share tools, data and compute. External save of this copy and the revised gallery is pending. Product Hunt has the earlier saved copy and is not scheduled. [Open the draft](https://www.producthunt.com/products/citta?launch=citta). Machine-readable fields are in [listing.json](listing.json).

## Listing

**Name:** Citta

**Tagline:** A network for agents to share tools, data and compute

**Description:** Citta connects your AI agents into a private network, so they can use each other's tools, access data and put spare compute to work. Each agent brings what it has access to: private files, installed tools, local models or connected hardware. When a task needs more capacity, your agent can send it to a member of the network with the resources to run it.

**Product link:** `https://github.com/priyankark/citta`; the repository is public; verify the published installation path without the maker's account before scheduling.

**Pricing:** Free. Explain that this applies to uses permitted by the noncommercial license; external model/API subscriptions and other services may cost money.

**Suggested tags:** Artificial Intelligence, Developer Tools, Productivity. These labels were verified in the saved editor. Do not select the open-source designation.

## Video caption

[Watch the demonstration](https://www.youtube.com/watch?v=EPdPu5djxSQ). The unlisted video plays while signed out, and all 45 manual-caption cues match the source SRT. Its Product Hunt attachment still needs a fresh saved-preview check. Keep demo context with the video or maker comment, rather than in the short product description.

Demonstration with synthetic data. Citta is available under PolyForm Noncommercial 1.0.0; commercial use is not licensed. See the license for permitted uses.

Place this beside the existing video if retaining its “work trip” opening. The recording demonstrates capabilities; it does not grant commercial-use permission.

## First maker comment

Hi, I'm Priyankar. I built Citta to connect AI agents into a network where they can share access to tools, data and compute.

One agent might have the files you need. Another might have a local model ready to run, a useful tool installed or access to connected hardware. Citta lets your agent find what is available, send work there and get the result back.

You can also put spare compute to work. A task runs using the CPU and RAM of the machine that accepts it. Citta doesn't combine memory into one pool.

The demo shows agents using receipt data and a local Gemma model on the same network. The receipts are synthetic, and the tasks ran on real Macs. The video description includes the recording details.

To try it, install Citta on your own Macs or Linux machines connected through Tailscale. Each agent needs its tools and accounts set up. Full-tool agents can run commands, edit files and use installed skills without asking for approval for each command, so this is for devices and agents you trust.

Citta (चित्त) means mind or thought in Sanskrit. The name fits the idea of agents bringing what they know and can do into a shared network.

This is an early release. The code is available under PolyForm Noncommercial 1.0.0; commercial use isn't licensed. The license also permits certain organizations, including educational and charitable institutions. Setup instructions and known limitations are in the README.

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

Citta connects your AI agents into a private network, so they can use each other's tools, access data and put spare compute to work. Try it for a permitted noncommercial project: https://github.com/priyankark/citta

This is prepared copy only. Do not send messages or publish posts as part of preparation. Ask for product feedback, not votes, following [Product Hunt's promotion guidance](https://help.producthunt.com/en/articles/17734384-5-how-to-promote-your-launch-and-what-to-avoid).
