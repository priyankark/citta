# Capability contract

A capability is installed locally by the operator. Its `program` must be an existing absolute file. Its `args` are fixed strings, never expanded by a shell. Each task's `input` is serialized as one JSON value plus a newline on stdin, followed by EOF. Stdout/stderr must be UTF-8 and each must stay within 256 KiB. Nonzero exit, timeout, invalid output, or excess output fails the task.

A worker receives only explicitly configured environment variables plus `CITTA_TASK_ID`, `CITTA_ORIGIN_NODE`, and `CITTA_WORKER_NODE`. Provide a minimal `PATH` if the wrapper needs subprocess lookup. Inherited home directories, API keys, proxy settings, and Citta credentials are not automatically forwarded. Environment configuration can contain secrets: keep the TOML private.

Every task gets a fresh work directory, which is removed on normal completion. The executable still runs with the OS user's privileges. Use dedicated accounts, containers, or OS sandboxing when capabilities need stronger isolation. Trusted programs must keep descendants in their assigned process group; a program that deliberately detaches can escape lifecycle cleanup.

The effective timeout is the smaller of the submitted timeout and `max_timeout_secs`. Global `slots` bounds all jobs on the device; each capability's `slots` bounds that capability. These are concurrency limits, not CPU/GPU/memory quotas. Multiple capability names using the same GPU must be configured conservatively. Model API work can continue server-side after its requesting wrapper is cancelled; enforce model-server budgets independently.

## Local model on another device

`examples/ollama_worker.py` calls only a fixed local Ollama endpoint. Choose an already-installed model; the adapter never downloads one. Add a capability using actual paths:

```toml
[[capabilities]]
name = "local-reasoning"
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

## Agent harness

`examples/prompt_worker.py` adapts a **noninteractive harness accepting its prompt on stdin**. It expects the same `{"prompt":"..."}` task input. Set `CITTA_AGENT_PROGRAM` to the harness's absolute executable and `CITTA_AGENT_ARGS` to an operator-chosen JSON argument array. Add only the environment variables needed for its established credentials and sandbox configuration.

Declare a spawnable harness with `kind = "agent"` (omitting it keeps the ordinary `task` kind):

```toml
[[capabilities]]
name = "codex-readonly"
kind = "agent"
description = "Read-only analysis with a fresh Codex agent; fixed device permissions."
program = "/absolute/path/to/python3"
args = ["/absolute/path/to/citta/examples/prompt_worker.py"]
slots = 1
max_timeout_secs = 180
[capabilities.env]
CITTA_AGENT_PROGRAM = "/absolute/path/to/codex"
CITTA_AGENT_ARGS = '["--no-daemon", "-a", "never", "-c", "mcp_servers.citta.enabled=false", "exec", "--sandbox", "read-only", "--ephemeral", "--skip-git-repo-check", "--color", "never", "-"]'
HOME = "/absolute/user/home"
PATH = "/usr/bin:/bin:/absolute/trusted/bin"
```

This example uses an established Codex login, a separate noninteractive process, read-only harness sandboxing, and no interactive approval fallback. It disables the Citta MCP entry for this worker to avoid unsupported recursive delegation. Other existing harness settings remain in effect. Read-only does not mean secret-isolated: access to HOME and harness credentials is a local trust decision. Citta does not add an OS sandbox around arbitrary profile executables. Inspect supported flags on the installed harness before using this example.

Use `citta_spawn_agent` / `spawn-agent` to submit the prompt directly. The receiver's profile determines permissions, not the caller. Ordinary capabilities retain their existing JSON contract. See [discovery and spawning](discovery.md).

Verify the installed harness's supported noninteractive mode and approval behavior before configuring it. Keep its normal restrictions; do not introduce flags that bypass approvals or grant unrestricted filesystem access. The adapter forwards stdout/stderr and exit status. It cannot interact with a session that waits for UI input, access an existing conversation, or create repository checkouts for you. Set time/output limits appropriate to the task.

Do not pass Citta's mesh credentials into delegated workers. Nested autonomous delegation with shared budgets and parent cancellation is a later protocol feature. The originating agent can already submit independent tasks to multiple devices and combine results itself.
