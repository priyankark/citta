# Reddit launch plan

Prepared November 12, 2026. **Drafts only: no Reddit posts, comments, messages or scheduled jobs have been created.** The target launch is November 14, but posting depends on readiness and time to answer people. This plan does not change the Product Hunt date.

## What readers should understand

An agent can get stuck because the file, tool or model it needs is somewhere else. Citta lets it find another worker, send that worker a task and get the result back. Lead with that problem and a concrete example. Explain what the user can get done before naming the architecture. Use plain language, concrete examples and short paragraphs. Avoid em dashes, unexplained terms such as worker or harness, and generic feedback questions.

Link directly to the [repository and setup guide](https://github.com/priyankark/citta). Use the [demo](https://youtu.be/EPdPu5djxSQ) when it helps explain the handoff. The purpose is to help people try Citta and learn where setup fails; Product Hunt votes are not the call to action.

## Communities and posting order

These are editorial choices, not predictions about reach. Rules were researched November 12; recheck the live sidebar, pinned threads and submission form immediately before posting. Account eligibility and posting history have not been inspected.

| Priority | Community | Format and angle | Current rule evidence / remaining check |
| --- | --- | --- | --- |
| 1 | r/SideProject | One maker post: an agent can get help from another agent with the files or tools it needs. | The [community](https://www.reddit.com/r/SideProject/) is oriented around side projects, but the accessible rules page did not expose detailed rules. Check the signed-in composer, pins and flair before posting; absence of visible rules is not clearance. |
| 2 | r/AI_Agents | Comment in the current weekly Project Display thread: discovery, delegation and returned results. | [Rule 3](https://www.reddit.com/r/AI_Agents/) directs project links to that thread; rules also limit self-promotion and require context. Find the current thread and check account history. |
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

**Title:** I built Citta so your AI agent can ask other agents for help

Say you ask your agent to sort out some receipts, but it can't access the files. Another agent can. Citta lets the first agent ask the second one to read them and send back the results.

It works the same way with tools and local AI models. An agent that has what you need can do that part of the job, and you can carry on with the agent you were already talking to.

I built Citta to make this possible. In the demo, one agent gets help reading receipts, then another uses a local Gemma model to categorize the expenses.

It runs on Macs and Linux machines connected through Tailscale. There's a setup guide and a first task to try in the README.

[Try it](https://github.com/priyankark/citta) · [Watch the demo](https://youtu.be/EPdPu5djxSQ)

It's an early release. The code is available under PolyForm Noncommercial 1.0.0; commercial use isn't licensed.

## Draft 2: r/AI_Agents Project Display comment

I built **Citta** to let agents ask each other for help when they have access to different things.

One might have the files you need. Another might have a tool installed, or a local model ready to run. With Citta, your agent can find out what's available, send a task over and get the result back.

We tried this with receipt analysis. One agent asked another to read the files. Then we sent the categorization step to a local Gemma model. The agents could use what was already available where each task ran.

You set it up on your own Macs or Linux machines with Tailscale. Each agent still needs its tools and permissions. In the default setup, it can run commands and edit files with that account's access, so this is for agents and devices you trust.

[Code, setup and demo](https://github.com/priyankark/citta)

It's an early release, under PolyForm Noncommercial 1.0.0. Commercial use isn't licensed.

## Draft 3: r/selfhosted New Project Megathread comment

**Project:** Citta. I'm the maker.

**What it does:** Lets your AI agent get help from another agent that has the files, tools or local model it needs. For example, you can ask an agent on your laptop to send a task to a local model running on your server and bring back the answer.

**Install:** Mac and Linux downloads are available, along with a Cargo package. Install Citta on each machine and connect them with Tailscale. The README covers setup and a first task. You'll also need the tools and agent software you want to run on each machine.

Citta runs on your machines. Whether data goes to a cloud service depends on the agents and tools you use. The default agent setup can run commands and edit files with your account's access, so use devices you trust.

**Code and guide:** https://github.com/priyankark/citta

**License:** PolyForm Noncommercial 1.0.0. The source is available; commercial use isn't licensed. This is an early release, with known limitations listed in the repo.

**AI involvement:** Built with AI coding agents, including Codex. The demo uses Codex and a local Gemma model. AI also helped draft this post.

## Reserve draft: r/ChatGPTCoding weekly thread

Your coding agent has the code, but the tool it needs is set up somewhere else. You could switch over and do that step yourself, or ask an agent there to handle it.

I built **Citta** for the second option. Your agent can find another agent, send it a task and get the results back. That agent can run commands, write code, run tests and use its installed tools and skills.

For example, you could keep working with Codex on your laptop while asking another agent to run tests in an environment you've already set up. Each agent uses the tools and access available where it runs.

The setup guide covers Codex on Macs and Linux machines connected through Tailscale. This is for your own trusted devices: delegated tasks can run commands and change files without asking you to approve each command.

[Setup and first task](https://github.com/priyankark/citta)

It's an early release. Source is available under PolyForm Noncommercial 1.0.0; commercial use isn't licensed.

## Replies and follow-through

Answer the question asked, with a direct docs link where useful. Explain that Citta adds capability discovery and tracked task handoff when someone asks why they would use it instead of SSH. Do not claim that a dedicated model API needs Citta; its value is the wider workflow across agents, tools and data.

If asked about privacy, explain that task payloads and returned results can contain data and the chosen harness may use a cloud model. If asked about the demo, describe the real two-way handoff and the synthetic receipt fixture accurately. Keep those details in the relevant answer or demo context, rather than making them the opening pitch.

Disclose affiliation, use the actual license name, and explain which receiving tools a hardware example would need. Only call it a completed demonstration if it was actually run. Do not invent usage numbers, speedups or user stories.

Record each published URL, community, time, draft revision, useful feedback and resulting issue in a private launch log. If a submission is removed, inspect the reason before doing anything else. Prioritize first-task completions and actionable setup reports over votes. No bulk DMs, vote requests or repeated promotional comments are part of this plan.
