# Reddit launch plan

Updated November 12, 2026. The user authorized publication starting today, ahead of the November 14 Product Hunt launch. The [SideProject post](https://www.reddit.com/r/SideProject/comments/1xulw8z/i_built_a_network_for_ai_agents_to_share_tools/) was published and its author, complete text and permalink were verified in the signed-in browser. A local run for the AI_Agents project thread is scheduled for November 13 at 09:00 Pacific. Scheduled runs remain subject to account eligibility, community rules and desktop availability. This does not change the Product Hunt date.

## What readers should understand

Citta is a private network for agents to share access to tools, data and compute. Lead with the network, then show what members can do together. Each agent brings its own tools and access. Spare CPU and RAM are used by running a task on the machine with capacity; memory is not pooled. Keep the language plain, use concrete examples and avoid em dashes or generic feedback questions.

Link directly to the [repository and setup guide](https://github.com/priyankark/citta). Use the [demo](https://youtu.be/EPdPu5djxSQ) when it helps explain the handoff. The purpose is to help people try Citta and learn where setup fails; Product Hunt votes are not the call to action.

## Communities and posting order

These are editorial choices, not predictions about reach. Rules were researched November 12; recheck the live sidebar, pinned threads and submission form immediately before posting. The personal account and recent history were checked for the SideProject post. Eligibility must still be checked for each later destination.

| Priority | Community | Format and angle | Current rule evidence / remaining check |
| --- | --- | --- | --- |
| 1 | r/SideProject | One maker post: connect agents into a network to use tools, data and spare compute. | The [community](https://www.reddit.com/r/SideProject/) is oriented around side projects, but the accessible rules page did not expose detailed rules. Check the signed-in composer, pins and flair before posting; absence of visible rules is not clearance. |
| 2 | r/AI_Agents | Comment in the current weekly Project Display thread: what agents bring to the network and how they share work. | [Rule 3](https://www.reddit.com/r/AI_Agents/) directs project links to that thread; rules also limit self-promotion and require context. Find the current thread and check account history. |
| 3 | r/selfhosted | Comment in the current New Project Megathread: running the mesh yourself, installation and dependencies. | [Rule 6](https://www.reddit.com/r/selfhosted/) routes projects younger than three months to the megathread. The [November 5 thread](https://www.reddit.com/r/selfhosted/comments/1xox23i/new_project_megathread_week_of_05_nov_2026/) asks for deployment details and AI involvement. Use the current thread, not that historical URL. Rule 2 also asks for production-ready apps with docs: Citta is an early release, so resolve whether its documented limits fit the current thread before posting. Do not relabel it production-ready. |
| Reserve | r/ChatGPTCoding | Current weekly self-promotion thread: delegating coding work with the receiving agent's tools and skills. | The [November 2 thread](https://www.reddit.com/r/ChatGPTCoding/comments/1xlra2u/weekly_self_promotion_thread/) requires context and affiliation, and discourages repeated weekly promotion without meaningful changes. Recheck the current thread. |
| Hold | r/LocalLLaMA | Potential future technical discussion about the local-model handoff. | [Rules 3 and 4](https://old.reddit.com/r/LocalLLaMA/about/rules/) restrict primarily LLM-generated copy/code and self-promotion. Citta's AI-assisted development and these AI-prepared drafts make this unsuitable for the initial campaign. No ready-to-post draft is supplied. A paraphrase is not a workaround. |

## Timing and readiness

Use Pacific time. The first two entries are authorized publication runs, not claims about the best algorithmic posting time. Later communities remain conditional.

| Window | Action |
| --- | --- |
| Before launch | Complete the outstanding [launch checks](launch-plan.md), verify public install/demo links, review the drafts against actual release behavior, and identify the personal Reddit account. Check each community's rules and account eligibility. |
| Nov 12, 11:18 | [SideProject post published](https://www.reddit.com/r/SideProject/comments/1xulw8z/i_built_a_network_for_ai_agents_to_share_tools/); text, author and permalink verified. |
| Nov 13 at 09:00, local timer | Review the first post's feedback, fix confusing instructions, then share the AI_Agents comment in its current project thread if eligible. |
| Nov 16–17, after feedback | Consider the selfhosted megathread only after resolving its readiness requirement. Use ChatGPTCoding as a focused alternative if coding delegation is the clearest fit; do not fill every channel just because a draft exists. |
| After useful fixes or results | Reply to the original threads with the specific change. Make a new post only when there is substantial new material and the community allows it. |

If readiness slips, shift the Reddit sequence. No post should say the Product Hunt launch is live until it actually is. The user has authorized posting from the personal account; verify its identity again at execution. The local timer needs the Mini powered on and unlocked with Reddit signed in. It does not survive reboot. An interrupted or uncertain submission is not automatically retried.

## Draft 1: r/SideProject

**Title:** I built a network for AI agents to share tools, data and compute

I built **Citta** to connect your AI agents into a private network. They can use each other's tools, access data and put spare compute to work.

One agent might have access to your files. Another might have a local model ready to run, or enough spare capacity for a task your current setup can't handle. Citta lets your agent find what's available on the network and send work there.

In the demo, an agent on my Mac Mini asks my MacBook Air to organize sample receipts. The Air then sends expense categorization to Gemma running locally on the Mini.

It runs on Macs and Linux machines connected through Tailscale, at home or across locations. The README walks through setup and a first task.

[Try it](https://github.com/priyankark/citta) · [Watch the demo](https://youtu.be/EPdPu5djxSQ)

It's an early release. The code is available under PolyForm Noncommercial 1.0.0; commercial use isn't licensed.

## Draft 2: r/AI_Agents Project Display comment

I built **Citta**, a private network for agents to share access to tools, data and compute.

Each agent brings what it can already use: files, installed tools, local models or connected hardware. Other agents on the network can find out what's available and send it work.

Tasks run on the machine with the required tools and capacity, then return their results.

We ran a demo where agents use receipt data and a local Gemma model to produce an expense report. The same network can run coding tasks with the receiving agent's shell tools and skills.

Setup covers Codex on your Macs and Linux machines connected through Tailscale. A control panel shows tasks and lets you set permissions for each device.

[Code, setup and demo](https://github.com/priyankark/citta)

Early release, under PolyForm Noncommercial 1.0.0. Commercial use isn't licensed.

## Draft 3: r/selfhosted New Project Megathread comment

**Project:** Citta. I'm the maker.

**What it does:** Connects your AI agents into a private network so they can use each other's tools, access data and put spare compute to work. An agent on your laptop could use a local model on your server, or ask another agent to work with files it has access to.

Tasks run on the receiving machine. Tailscale connects the devices across locations.

**Install:** Mac and Linux downloads, plus a Cargo package. Install Citta on each machine and follow the README's setup and first task. Each machine needs the tools and agent software you want to run there.

Citta runs on your machines without a central task server. The agents and tools you choose may still use cloud services.

**Code and guide:** https://github.com/priyankark/citta

**License:** PolyForm Noncommercial 1.0.0. Source is available; commercial use isn't licensed. This is an early release, with known limitations in the repo.

**AI involvement:** Built with AI coding agents, including Codex. The demo uses Codex and a local Gemma model. AI also helped draft this post.

## Reserve draft: r/ChatGPTCoding weekly thread

I built **Citta** to connect coding agents into a network where they can share access to tools, project files and spare compute.

Your agent can find what's available, send a task to another agent and get the result back. Each agent can run commands, write code, run tests and use the tools and skills installed where it runs.

For example, you could keep working with Codex on your laptop while another agent runs tests in an environment you've already set up. Or send a model task to a machine with enough RAM to run it.

The setup guide covers Codex on Macs and Linux machines connected through Tailscale. You can monitor tasks and configure device permissions from the control panel.

[Setup and first task](https://github.com/priyankark/citta)

It's an early release. Source is available under PolyForm Noncommercial 1.0.0; commercial use isn't licensed.

## Replies and follow-through

Answer the question asked, with a direct docs link where useful. Explain that Citta adds capability discovery and tracked task handoff when someone asks why they would use it instead of SSH. Do not claim that a dedicated model API needs Citta; its value is the wider workflow across agents, tools and data.

If asked about privacy, explain that task payloads and returned results can contain data and the chosen harness may use a cloud model. If asked about the demo, describe the real two-way handoff and the synthetic receipt fixture accurately. Keep those details in the relevant answer or demo context, rather than making them the opening pitch.

Disclose affiliation, use the actual license name, and explain which receiving tools a hardware example would need. Only call it a completed demonstration if it was actually run. Do not invent usage numbers, speedups or user stories.

Record each published URL, community, time, draft revision, useful feedback and resulting issue in a private launch log. If a submission is removed, inspect the reason before doing anything else. Prioritize first-task completions and actionable setup reports over votes. No bulk DMs, vote requests or repeated promotional comments are part of this plan.
