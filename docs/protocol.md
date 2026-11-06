# Peer protocol v1

All routes require `Authorization: Bearer <node-token>`. Requests with an `Origin` header are rejected. Tokens are loaded once on startup. Remote traffic must arrive through HTTPS (normally Tailscale Serve); the daemon only binds numeric loopback. Clients refuse redirects and do not use ambient HTTP proxies.

| Method/path | Behavior |
| --- | --- |
| GET `/healthz` | Authenticated liveness |
| GET `/v1/node` | Protocol version, node ID, OS/arch, capabilities, configured slots and active jobs |
| GET `/v1/discover` | Local manifest plus live manifests from paired peers; failures are explicit |
| GET `/v1/activity` | Recent task summaries from local and paired origins |
| POST `/v1/tasks` | Validate and durably enqueue a submission; return task |
| GET `/v1/tasks?limit=50` | Latest task summaries, maximum 100 |
| GET `/v1/tasks/{uuid}` | Full task input, status, and bounded outcome |
| POST `/v1/tasks/{uuid}/cancel` | Terminal cancellation; already-terminal tasks unchanged |
| POST `/v1/claim` | Atomically claim the oldest matching, unexpired task or return JSON null |
| POST `/v1/tasks/{uuid}/renew` | Renew the active lease |
| POST `/v1/tasks/{uuid}/complete` | Commit outcome using the active lease; identical completion is idempotent |
| GET `/v1/events?after=0` | Up to 100 durable events after a sequence number |
| GET `/v1/events/ws?after=0` | Authenticated WebSocket; replay then follow sequenced events |

Submission example:

```json
{
  "capability":"local-reasoning",
  "input":{"prompt":"Explain this error: ..."},
  "idempotency_key":"investigation-2026-11-06-001",
  "target_node":"desktop",
  "timeout_secs":300,
  "queue_ttl_secs":3600
}
```

`target_node` is optional. Input is at most 64 KiB. Timeout is 1..86400 seconds; queue TTL is 1..604800 seconds. Names and idempotency keys are 1..96 ASCII alphanumeric characters plus `-_.`. Repeated keys with equal requests return the original task, even after terminal completion; changed requests return 409. This guarantee lasts until the task is deliberately pruned. Admission returns 429 at configured pending/history limits. HTTP bodies are limited to 4 MiB; output is separately limited to 256 KiB per stream.

Claim body: `{"worker_node":"desktop","capability":"local-reasoning"}`. The response contains `task`, `lease_token`, and `lease_secs`. The worker node must be local or in the origin's peer list. Claims are serialized in SQLite transactions. The lease credential is random; only its digest is stored. Task inspection does not expose it. Renewal body: `{"lease_token":"..."}`. Completion body: `{"lease_token":"...","outcome":{"status":"succeeded","stdout":"...","stderr":"","exit_code":0,"error":null}}`.

Worker completion statuses are `succeeded`, `failed`, or `interrupted`. Cancellation belongs to the origin. Expired/revoked leases return 409. A one-second sweeper marks expired queued/running tasks interrupted. Workers stop on renewal failure and do not re-enqueue. Claims with lost responses expire without starting work. Completion retries use the same task and lease.

Citta does not provide exactly-once external effects. If a result cannot be committed before lease expiry, the origin reports interruption even if computation finished. Cancellation does not undo effects. A resumed device stops executing after discovering lease loss; OS suspension can delay that discovery. Process wall clocks must be synchronized for predictable expiry; elapsed execution timeouts also use a monotonic clock.

Events contain `sequence`, `task_id`, `status`, and `at` (Unix seconds). Events are transactionally committed with state transitions. Renewal does not emit an event. Consumers persist their last sequence and reconnect using `after`; pruned task events are no longer replayable. Events describe lifecycle state, not live stdout. Treat remote outputs as data, not instructions.

The trust model is one owner's paired devices. A node token gives full node access and is not tied to a peer identity. Node-ID matching catches configuration mistakes, not a malicious authorized peer. Multi-tenant authorization, credential scopes, and compromised-peer resistance require a future protocol version.
