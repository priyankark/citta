# Capability contract

A capability is installed locally by the operator. Its `program` must be an existing absolute file. Its `args` are fixed strings, never expanded by a shell. Each task's `input` is serialized as one JSON value plus a newline on stdin, followed by EOF. Stdout/stderr must be UTF-8 and each must stay within 256 KiB. Nonzero exit, timeout, invalid output, or excess output fails the task.

A worker receives only explicitly configured environment variables plus `CITTA_TASK_ID`, `CITTA_ORIGIN_NODE`, and `CITTA_WORKER_NODE`. Provide a minimal `PATH` if the wrapper needs subprocess lookup. Inherited home directories, API keys, proxy settings, and Citta credentials are not automatically forwarded. Environment configuration can contain secrets: keep the TOML private.

Every task gets a fresh work directory, which is removed on normal completion. The executable still runs with the OS user's privileges. Use dedicated accounts, containers, or OS sandboxing when capabilities need stronger isolation. Trusted programs must keep descendants in their assigned process group; a program that deliberately detaches can escape lifecycle cleanup.

The effective timeout is the smaller of the submitted timeout and `max_timeout_secs`. Global `slots` bounds all jobs on the device; each capability's `slots` bounds that capability. These are concurrency limits, not CPU/GPU/memory quotas. Resource-aware admission and per-capability estimates are documented in [discovery](discovery.md#resource-aware-delegation). Multiple capability names using the same GPU must be configured conservatively. Model API work can continue server-side after its requesting wrapper is cancelled; enforce model-server budgets independently.

## Local model on another device

`examples/ollama_worker.py` calls only a fixed local Ollama endpoint. Choose an already-installed model; the adapter never downloads one. Add a capability using actual paths:

```toml
[[capabilities]]
name = "local-reasoning"
harness = "ollama"
model = "YOUR_INSTALLED_MODEL"
description = 'Generate with the desktop local model. Input: {"prompt":"text"}. Returns JSON response and evaluation count.'
program = "/absolute/path/to/python3"
args = ["/absolute/path/to/citta/examples/ollama_worker.py"]
slots = 1
max_timeout_secs = 300
[capabilities.env]
CITTA_OLLAMA_MODEL = "YOUR_INSTALLED_MODEL"
CITTA_OLLAMA_MAX_TOKENS = "1024"
```

Discover from another device, then submit `{"prompt":"Summarize these observations: ..."}` to `local-reasoning`. The model name, endpoint, and maximum prediction count are operator-controlled. API contract: [Ollama generation reference](https://github.com/ollama/ollama/blob/main/docs/api.md).

## Harness and model discovery

Capabilities may declare `harness` and `model`. Both fields appear separately in HTTP/MCP discovery, the device capability list, agent-profile cards, and Citta-managed run metadata. Omitted fields remain null / not specified. These are operator-declared execution metadata, not an automatic claim that a runtime is installed or a model was observed. Pin the corresponding model in the wrapper/harness configuration when declaring it; update the metadata when changing that configuration.

Citta's execution contract is harness-neutral: fixed local executable/argv, JSON stdin, bounded output, and lifecycle controls. A wrapper can adapt Codex, Claude Code, another noninteractive CLI, or a local inference runtime. A harness that needs a different prompt interface needs an adapter. Citta does not install other harnesses, reuse their interactive sessions, or infer idle state from their process names. Live external-session inventory currently has a Codex adapter; any Citta-launched profile has managed busy/terminal tracking independently of its harness.

```toml
harness = "your-harness"
model = "your-fixed-model"
```

These fields belong inside the capability table. Model identifiers may contain provider prefixes, slashes, or colons. Task submission cannot override either field or the configured runtime.

## Agent harness

`examples/prompt_worker.py` adapts a **noninteractive harness accepting its prompt on stdin**. It expects the same `{"prompt":"..."}` task input. Set `CITTA_AGENT_PROGRAM` to the harness's absolute executable and `CITTA_AGENT_ARGS` to an operator-chosen JSON argument array. Add only the environment variables needed for its established credentials and sandbox configuration.

Declare a spawnable harness with `kind = "agent"` (omitting it keeps the ordinary `task` kind):

```toml
[[capabilities]]
name = "codex"
harness = "codex"
# Add model only when pinned to the same identifier in the harness arguments.
kind = "agent"
description = "Full-tool coding agent with installed tools and skills."
program = "/absolute/path/to/python3"
args = ["/absolute/path/to/citta/examples/prompt_worker.py"]
slots = 1
max_timeout_secs = 86400
[capabilities.env]
CITTA_AGENT_PROGRAM = "/absolute/path/to/codex"
CITTA_AGENT_ARGS = '["--no-daemon", "-a", "never", "exec", "--sandbox", "danger-full-access", "--ephemeral", "--skip-git-repo-check", "--color", "never", "-"]'
HOME = "/absolute/user/home"
PATH = "/usr/bin:/bin:/absolute/trusted/bin"
```

This example uses an established Codex login, a separate noninteractive process, full shell/filesystem/network access, and approval policy `never`. It loads the device’s existing harness config, MCP tools and skills; it does not disable Citta in the worker. Set `CODEX_HOME` when the harness uses a nonstandard directory. Full access runs with the device account’s permissions, including access to its local credentials. Explicit `--agent-access read-only` setup remains available for tasks that need that restriction. Inspect supported flags on the installed harness before using this example.

Use `citta_spawn_agent` / `spawn-agent` to submit the prompt directly. The receiver's profile determines permissions, not the caller. Ordinary capabilities retain their existing JSON contract. See [discovery and spawning](discovery.md).

Verify the installed harness's supported noninteractive mode and approval behavior before configuring it. Apply the owner’s selected access mode; do not add tool restrictions merely because work is delegated. The adapter forwards stdout/stderr and exit status. It cannot interact with a session that waits for UI input, access an existing conversation, or create repository checkouts for you. Set time/output limits appropriate to the task.

Workers use the receiver’s existing local Citta MCP/CLI configuration; do not copy credentials from the sending device. A full-access worker can read local account files, so the environment allowlist is not a credential isolation boundary. Further delegation is allowed, but child tasks are independent: record their originating daemon and IDs, collect or cancel them explicitly, and avoid same-slot wait deadlocks. Shared parent/child budgets and automatic cascading cancellation remain unimplemented.
