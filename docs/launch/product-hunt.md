# Product Hunt launch copy

Prepared November 10, 2026. This is submission copy, not a published or scheduled launch. Use it after the [release gates](launch-plan.md) pass. Machine-readable fields are in [listing.json](listing.json).

## Listing

**Name:** Citta

**Tagline:** Let your AI agents use your other computers

**Description:** Your agent is on one computer. The files, tools, hardware or spare compute it needs may be on another. Citta connects your Macs and Linux machines over Tailscale so agents can discover capabilities, delegate tasks and bring results back. Built for personal projects and other uses permitted by its noncommercial license.

**Product link:** `https://github.com/priyankark/citta` — use only after it is public and the installation path works without the maker's account.

**Pricing:** Free. Explain that this applies to uses permitted by the noncommercial license; external model/API subscriptions and other services may cost money.

**Suggested tags:** Artificial Intelligence, Developer Tools, Productivity. Confirm the exact available labels in the editor. Do not select the open-source designation.

## Video caption

Demonstration with synthetic data. Citta is available under PolyForm Noncommercial 1.0.0; commercial use is not licensed. See the license for permitted uses.

Place this beside the existing video if retaining its “work trip” opening. The recording demonstrates capabilities; it does not grant commercial-use permission.

## First maker comment

Hi, I'm Priyankar, the maker of Citta.

Your agent may be on your laptop, while the thing it needs is somewhere else: files on another Mac, a model on your desktop, or hardware plugged into a different machine.

Citta lets your agents use those computers together. Connect your devices with Tailscale, set up Citta on each one, and your agent can discover what is available, send a task to the right device, and get the result back.

The demo shows both directions. An agent on a Mac Mini asks a MacBook Air to organize synthetic trip receipts. Then the Air asks the Mini to categorize the expenses with a local Gemma model. The files and the model are on different computers, but they can contribute to the same task.

The name comes from the Sanskrit word *citta* (चित्त), meaning mind or thought. It reflects the idea of bringing your devices' capabilities together around what you want to do.

Citta is for people experimenting with agents across their own Macs and Linux machines. The source is available under PolyForm Noncommercial 1.0.0; commercial use isn't licensed. The license also expressly permits certain organizations, including educational and charitable institutions. This is an early release, and receiving devices still need the tools, accounts and permissions for the work you ask them to do.

What is the first task you would send to your other computer? I'd especially like feedback on setup: where did you get stuck, and what did you expect to happen?

## Ready answers

**Why another computer?** It may have a file, installed tool, local model, connected device, or spare CPU and memory that the computer you're using does not. Citta provides discovery and task handoff across those devices.

**What do I need?** Two supported computers under your control, Tailscale connected on both, and Citta installed on each. For delegated agent work, the receiving computer needs a configured agent harness and access to the tools it will use. See the README for the current supported setup.

**Does Tailscale give agents access to everything?** No. It connects the machines. Citta's device settings, the receiving harness, account sign-ins, and operating-system permissions still determine what a task can do. Desktop control and screen capture require separate permission checks.

**Does Citta pool RAM or move a running agent?** It sends tasks to another computer to run there. It does not combine the machines' RAM or migrate an agent's live process.

**Is everything local?** Citta has no required cloud task coordinator, and a local model can run on a receiving device. The agent harness and tools may still use cloud services. The demonstrated orchestration used cloud-backed Codex; task results can contain file contents. Choose tools and data access accordingly.

**Is the demo real?** The two directions were executed on real Macs using synthetic text receipts. Python checked the totals and duplicates; Gemma supplied categories and short notes. Waiting was shortened in the video. The Air's report save failed because its disk was full, so the returned report was saved on the Mini. Later hardware examples are illustrations, not recordings of completed hardware tasks.

**Is it open source? Can my company use it?** It is source-available under PolyForm Noncommercial 1.0.0. Commercial use is not licensed. Read the actual license for its permitted purposes and express institutional allowances; attribution alone does not grant commercial permission.

**Will it work with every agent?** Citta exposes MCP tools and command adapters. Setup currently automates the documented Codex path; other harnesses need compatible MCP configuration and a working receiving-side adapter. Installed apps and permissions are not automatically copied between computers.

**What if a device sleeps or a task stops?** Inspect the task's status and output before trying it again. Interrupted tasks may already have changed files or performed an external action. Release qualification covers restart and interruption behavior; it is not a promise that arbitrary work is safe to replay.

## Share copy

Your other computer has files, tools and computing power your agent could use. Citta lets agents send tasks across your Macs and Linux machines over Tailscale. Watch the two-way demo, try it for a personal project, and tell me where setup could be easier: [insert live Product Hunt link].

This is prepared copy only. Do not send messages or publish posts as part of preparation. Ask for product feedback, not votes, following [Product Hunt's promotion guidance](https://help.producthunt.com/en/articles/17734384-5-how-to-promote-your-launch-and-what-to-avoid).
