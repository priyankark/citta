# Use a local model on another computer

Keep using your laptop while a model on your desktop handles a task. Citta sends the prompt to that computer, calls its local model, and returns the answer. Your laptop does not need the model installed.

The demonstrated workflow used Gemma 3 4B through Ollama to categorize six synthetic trip expenses. Choose a model that fits your own computer; that example is not a minimum hardware guarantee.

## Before you start

Set up Citta on both devices and confirm the desktop appears in `citta discover`. On the desktop, install Ollama using its [official instructions](https://docs.ollama.com/quickstart), download the model you want to use, and check that it works locally. Citta does not download models or start Ollama for you.

The adapter needs Python 3.11+ and a running Ollama service at `http://127.0.0.1:11434`. With the binary bundle, the adapter is installed at `~/.local/share/citta/0.1.0/examples/ollama_worker.py`. A source checkout includes the same file under `examples/`. If you installed with Cargo, download the matching source archive for this adapter; `cargo install` installs the executable, not the example files.

## Ask your setup agent

On the desktop, you can ask:

> “Configure Citta to share my installed Ollama model as local-reasoning. Keep the endpoint local to this computer, use one task at a time and a 512-token output budget, and set resource estimates that fit this model. Verify a real response from my other computer.”

Give it the installed model name. It should preserve your existing Citta identity and permissions, explain any missing prerequisite, and return the actual remote task result.

## Configure it manually

On the desktop, find the exact model name with `ollama list` and Python path with `command -v python3`. Back up your private Citta config, normally `~/.config/citta/config.toml`. Add the following capability, replacing all three placeholder paths/model names with actual values. Paths must be absolute; `~` is not expanded inside TOML. If `local-reasoning` already exists, edit that entry instead of adding a duplicate.

```toml
[[capabilities]]
name = "local-reasoning"
harness = "ollama"
model = "YOUR_INSTALLED_MODEL"
description = 'Use the desktop local model. Input: {"prompt":"text"}.'
program = "/absolute/path/to/python3"
args = ["/absolute/path/to/citta/examples/ollama_worker.py"]
slots = 1
max_timeout_secs = 300

[capabilities.env]
CITTA_OLLAMA_MODEL = "YOUR_INSTALLED_MODEL"
CITTA_OLLAMA_MAX_TOKENS = "512"

[capabilities.resources]
# Example estimate for the demonstrated small model, not a measured requirement.
# Increase these for larger models or context windows.
memory_bytes = 6442450944
cpu_cores = 2.0
```

For resource checks on ordinary capability submissions, set `guard_all = true` in the existing top-level `[admission]` table before restarting. Preserve its other settings. If that table is absent, add it before the `[[capabilities]]` sections. Resource checks estimate available CPU and memory; they do not enforce GPU memory limits or stop an already-running model from consuming resources.

Validate the file with `citta check`. Wait for active work to finish, then [restart the existing Citta service](operations.md#restart-after-a-configuration-change). Do not initialize a new identity. Keep Ollama running separately.

Open `citta panel`, select the desktop under **Permissions**, and allow `local-reasoning` for the intended originating device. A newly added capability needs an explicit grant when the device is in Restricted mode. It does not require enabling Allow all.

## Run it from the laptop

Use `citta discover` to find the desktop's node ID. Replace `DESKTOP_NODE_ID` below with that value:

```sh
printf '%s\n' '{"prompt":"Categorize these personal trip expenses: train ticket, hotel, dinner. Return a short list."}' \
  | citta submit --capability local-reasoning --target DESKTOP_NODE_ID \
      --input - --key local-model-first-check --timeout 300 --queue-ttl 300
```

The command returns a task ID. Inspect that ID on the laptop:

```sh
citta inspect TASK_UUID
```

Confirm the task succeeded, `worker_node` is the desktop, and the returned output contains an answer. A queued task has not run. If the first submission response was lost, use `citta lookup --key local-model-first-check` to recover that same task before submitting again. For a different prompt or a deliberate new test, choose a new `--key`.

After this works, ask your agent:

> “Use the desktop's local-reasoning capability to categorize these expenses, and show me the result.”

The adapter accepts a prompt and returns text; it does not give the model access to files or tools. Send the information it needs in the prompt. For arithmetic, compute and check totals with code; review model-generated categories and explanations.

## If it does not work

| Symptom | Check |
| --- | --- |
| Desktop is missing | `citta doctor` on both devices; Tailscale owner, reachability, and service state |
| `local-reasoning` is missing | Config validates, capability added on the correct computer, service restarted |
| Task stays queued | Device policy, available slots, CPU/memory estimates, and whether the desktop is awake |
| Connection refused | Ollama is running on the desktop at the fixed local endpoint |
| Model not found | The exact configured model is installed; Citta will not download it |
| Timeout or poor answer | Inspect the original task; check model speed, context size, and output budget before a deliberate new test |

Inference in this step happens on the receiving computer. Prompts and results move between your devices. If your agent uses a cloud model to prepare or interpret the request, that part remains cloud-backed.
