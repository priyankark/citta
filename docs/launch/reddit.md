# Reddit launch plan

Prepared November 12, 2026. **Drafts only: no Reddit posts, comments, messages or scheduled jobs have been created.** The target launch is November 14, but posting depends on readiness and time to answer people. This plan does not change the Product Hunt date.

## What readers should understand

Citta is a private network for agents to share access to tools, data and compute. Lead with the network, then show what members can do together. Each agent brings its own tools and access. Spare CPU and RAM are used by running a task on the machine with capacity; memory is not pooled. Keep the language plain, use concrete examples and avoid em dashes or generic feedback questions.

Link directly to the [repository and setup guide](https://github.com/priyankark/citta). Use the [demo](https://youtu.be/EPdPu5djxSQ) when it helps explain the handoff. The purpose is to help people try Citta and learn where setup fails; Product Hunt votes are not the call to action.

## Communities and posting order

These are editorial choices, not predictions about reach. Rules were researched November 12; recheck the live sidebar, pinned threads and submission form immediately before posting. Account eligibility and posting history have not been inspected.

| Priority | Community | Format and angle | Current rule evidence / remaining check |
| --- | --- | --- | --- |
| 1 | r/SideProject | One maker post: connect agents into a network to use tools, data and spare compute. | The [community](https://www.reddit.com/r/SideProject/) is oriented around side projects, but the accessible rules page did not expose detailed rules. Check the signed-in composer, pins and flair before posting; absence of visible rules is not clearance. |
| 2 | r/AI_Agents | Comment in the current weekly Project Display thread: what agents bring to the network and how they share work. | [Rule 3](https://www.reddit.com/r/AI_Agents/) directs project links to that thread; rules also limit self-promotion and require context. Find the current thread and check account history. |
| 3 | r/selfhosted | Comment in the current New Project Megathread: running the mesh yourself, installation and dependencies. | [Rule 6](https://www.reddit.com/r/selfhosted/) routes projects younger than three months to the megathread. The [November 5 thread](https://www.reddit.com/r/selfhosted/comments/1xox23i/new_project_megathread_week_of_05_nov_2026/) asks for deployment details and AI involvement. Use the current thread, not that historical URL. Rule 2 also asks for production-ready apps with docs: Citta is an early release, so resolve whether its documented limits fit the current thread before posting. Do not relabel it production-ready. |
| Reserve | r/ChatGPTCoding | Current weekly self-promotion thread: delegating coding work with the receiving agent's tools and skills. | The [November 2 thread](https://www.reddit.com/r/ChatGPTCoding/comments/1xlra2u/weekly_self_promotion_thread/) requires context and affiliation, and discourages repeated weekly promotion without meaningful changes. Recheck the current thread. |
| Hold | r/LocalLLaMA | Potential future technical discussion about the local-model handoff. | [Rules 3 and 4](https://old.reddit.com/r/LocalLLaMA/about/rules/) restrict primarily LLM-generated copy/code and self-promotion. Citta's AI-assisted development and these AI-prepared drafts make this unsuitable for the initial campaign. No ready-to-post draft is supplied. A paraphrase is not a workaround. |

## Timing and readiness

Use Pacific time. These are proposed response windows, not automated appointments or claims about the best algorithmic posting time.

| Window | Action |
| --- | --- |
| Before launch | Complete the outstanding [launch checks](launch-plan.md), verify public install/demo links, review the drafts against actual release behavior, and identify the personal Reddit account. Check each community's rules and account eligibility. |
| Launch day, proposed Nov 14 at 08:30 | Publish the SideProject post if ready and the maker can answer questions for the next hour. If unavailable, use the next suitable response window. |
| Following day, proposed Nov 15 at 09:00 | Review the first post's feedback, fix confusing instructions, then share the AI_Agents comment in its current project thread if eligible. |
| Nov 16–17, after feedback | Consider the selfhosted megathread only after resolving its readiness requirement. Use ChatGPTCoding as a focused alternative if coding delegation is the clearest fit; do not fill every channel just because a draft exists. |
| After useful fixes or results | Reply to the original threads with the specific change. Make a new post only when there is substantial new material and the community allows it. |

If readiness slips, shift the Reddit sequence. No post should say the Product Hunt launch is live until it actually is. Confirm publication intent and the personal Reddit account when moving from this requested plan to posting; no account access is needed to finish preparation.

## Draft 1: r/SideProject

**Title:** I built a network for AI agents to share tools, data and compute

I built **Citta** to connect your AI agents into a private network. They can use each other's tools, access data and put spare compute to work.

One agent might have access to your files. Another might have a local model ready to run, or enough spare capacity for a task your current setup can't handle. Citta lets your agent find what's available on the network and send work there.

In the demo, agents use receipt files and a local Gemma model to put together an expense report. It's one example of what they can do with the resources available across the network.

It runs on Macs and Linux machines connected through Tailscale, at home or across locations. The README walks through setup and a first task.

[Try it](https://github.com/priyankark/citta) · [Watch the demo](https://youtu.be/EPdPu5djxSQ)

It's an early release. The code is available under PolyForm Noncommercial 1.0.0; commercial use isn't licensed.

## Draft 2: r/AI_Agents Project Display comment

I built **Citta**, a private network for agents to share access to tools, data and compute.

Each agent brings what it can already use: files, installed tools, local models or connected hardware. Other agents on the network can find out what's available and send it work.

That also gives your agents a way to use spare CPU and RAM elsewhere on the network. The task runs on the machine with capacity and sends its result back. Memory stays on that machine; Citta doesn't combine RAM into one pool.

We ran a demo where agents use receipt data and a local Gemma model to produce an expense report. The same network can run coding tasks with the receiving agent's shell tools and skills.

Setup covers Macs and Linux machines connected through Tailscale. Each agent needs its own tools and permissions. The default setup can run commands and edit files with that account's access, so use agents and devices you trust.

[Code, setup and demo](https://github.com/priyankark/citta)

Early release, under PolyForm Noncommercial 1.0.0. Commercial use isn't licensed.

## Draft 3: r/selfhosted New Project Megathread comment

**Project:** Citta. I'm the maker.

**What it does:** Connects your AI agents into a private network so they can use each other's tools, access data and put spare compute to work. An agent on your laptop could use a local model on your server, or ask another agent to work with files it has access to.

Tasks use the CPU and RAM of the machine running them. Citta doesn't pool memory. Tailscale connects the network, including machines in different locations.

**Install:** Mac and Linux downloads, plus a Cargo package. Install Citta on each machine and follow the README's setup and first task. Each machine needs the tools and agent software you want to run there.

Citta runs on your machines. Whether data goes to a cloud service depends on the agents and tools you choose. The default agent setup can run commands and edit files with your account's access, so use devices you trust.

**Code and guide:** https://github.com/priyankark/citta

**License:** PolyForm Noncommercial 1.0.0. Source is available; commercial use isn't licensed. This is an early release, with known limitations in the repo.

**AI involvement:** Built with AI coding agents, including Codex. The demo uses Codex and a local Gemma model. AI also helped draft this post.

## Reserve draft: r/ChatGPTCoding weekly thread

I built **Citta** to connect coding agents into a network where they can share access to tools, project files and spare compute.

Your agent can find what's available, send a task to another agent and get the result back. Each agent can run commands, write code, run tests and use the tools and skills installed where it runs.

For example, you could keep working with Codex on your laptop while another agent runs tests in an environment you've already set up. Or send a model task to a machine with enough RAM to run it.

The setup guide covers Codex on Macs and Linux machines connected through Tailscale. This is for your own trusted devices: delegated tasks can run commands and change files without asking you to approve each command.

[Setup and first task](https://github.com/priyankark/citta)

It's an early release. Source is available under PolyForm Noncommercial 1.0.0; commercial use isn't licensed.

## Replies and follow-through

Answer the question asked, with a direct docs link where useful. Explain that Citta adds capability discovery and tracked task handoff when someone asks why they would use it instead of SSH. Do not claim that a dedicated model API needs Citta; its value is the wider workflow across agents, tools and data.

If asked about privacy, explain that task payloads and returned results can contain data and the chosen harness may use a cloud model. If asked about the demo, describe the real two-way handoff and the synthetic receipt fixture accurately. Keep those details in the relevant answer or demo context, rather than making them the opening pitch.

Disclose affiliation, use the actual license name, and explain which receiving tools a hardware example would need. Only call it a completed demonstration if it was actually run. Do not invent usage numbers, speedups or user stories.

Record each published URL, community, time, draft revision, useful feedback and resulting issue in a private launch log. If a submission is removed, inspect the reason before doing anything else. Prioritize first-task completions and actionable setup reports over votes. No bulk DMs, vote requests or repeated promotional comments are part of this plan.
