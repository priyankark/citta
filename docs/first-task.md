# Run your first task on another computer

You need Citta installed on two computers, both awake and connected to Tailscale under the same owner account. Follow the [installation guide](install.md) first. Run `citta setup` and `citta doctor` on both.

## Check the connection

On the computer you want to send work from, run:

```sh
citta agents
```

Find the other computer's `node_id` and confirm it is reachable and offers `echo`. Use that node ID below. This small task returns your message without starting a model or changing files.

```sh
other_node='REPLACE_WITH_OTHER_NODE_ID'
task_key="first-task-$(python3 -c 'import uuid; print(uuid.uuid4())')"
printf '%s\n' '{"message":"Hello from my other computer"}' | \
  citta submit --target "$other_node" --capability echo \
  --input - --key "$task_key" --timeout 20 --queue-ttl 120
```

Keep the returned task ID. Inspect it from the same computer:

```sh
citta inspect TASK_ID
```

It may initially be queued or running. Inspect the same ID again. Success means `status` is `succeeded`, `worker_node` is the other computer, and the output contains your message. A device appearing in the list is only discovery; this result proves a task ran there.

## Give an agent a small job

If the other computer has a signed-in Codex CLI and an allowed agent profile, open a **new Codex session** on the sending computer and ask:

> Use Citta to discover my other computer. On that device, run a small Python check that `sum(range(1, 101))` equals 5050. Return the worker's node ID, task ID, and output. Do not install anything or change existing files. If that device is unavailable, report why.

Check that the result names the intended remote worker. Then try a task that benefits from its tools or data, such as [using a local model on the other computer](local-model.md).

## If you get stuck

| Result | Check |
| --- | --- |
| Other computer missing or unreachable | Run `citta doctor` on both. Check that both are awake, on Tailscale under the same owner, and running Citta. |
| Task stays queued | Run `citta panel`. Check the receiving device's permissions, available resources and occupied slots. |
| Echo works but an agent task fails | Check the receiving device's harness sign-in, profile and required tools. |
| Task failed or was interrupted | Inspect its existing result before submitting another task. |

[Report a setup problem](https://github.com/priyankark/citta/issues/new?template=setup.yml) or [tell us how your first task went](https://github.com/priyankark/citta/issues/new?template=first-task.yml). Reports are public; omit credentials, private paths and task data.
