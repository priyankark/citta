# Peer protocol v1

Task/control routes require `Authorization: Bearer <credential>`: the local/legacy node token or an active automatic peer credential. The two enrollment handshake routes below use a separate challenge protocol. Requests with an `Origin` header are rejected. Local and explicit-peer tokens are loaded once on startup; automatic peer credentials refresh in the running daemon. Remote traffic must arrive through HTTPS (normally Tailscale Serve); the daemon only binds numeric loopback. Clients refuse redirects and do not use ambient HTTP proxies.

| Method/path | Behavior |
| --- | --- |
| GET `/healthz` | Authenticated liveness |
| GET `/v1/node` | Protocol version, node ID, OS/arch, capabilities, configured slots and active jobs |
| GET `/v1/discover` | Local manifest plus live manifests from paired peers; failures are explicit |
| GET `/v1/enrollment` | Automatic enrollment freshness, pinned identity and candidate/paired states; no secrets |
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

`target_node` is optional. Input is at most 64 KiB. Timeout is 1..86400 seconds; queue TTL is 1..604800 seconds. Names and idempotency keys are 1..96 ASCII alphanumeric characters plus `-_.`, excluding the reserved names `.` and `..`. Repeated keys with equal requests return the original task, even after terminal completion; changed requests return 409. This guarantee lasts until the task is deliberately pruned. Admission returns 429 at configured pending/history limits. HTTP bodies are limited to 4 MiB; output is separately limited to 256 KiB per stream.

Claim body: `{"worker_node":"desktop","capability":"local-reasoning"}`. The response contains `task`, `lease_token`, and `lease_secs`. The worker node must be local or in the origin's peer list. Claims are serialized in SQLite transactions. The lease credential is random; only its digest is stored. Task inspection does not expose it. Renewal body: `{"lease_token":"..."}`. Completion body: `{"lease_token":"...","outcome":{"status":"succeeded","stdout":"...","stderr":"","exit_code":0,"error":null}}`.

Worker completion statuses are `succeeded`, `failed`, or `interrupted`. Cancellation belongs to the origin. Expired/revoked leases return 409. A one-second sweeper marks expired queued/running tasks interrupted. Workers stop on renewal failure and do not re-enqueue. Claims with lost responses expire without starting work. Completion retries use the same task and lease.

Citta does not provide exactly-once external effects. If a result cannot be committed before lease expiry, the origin reports interruption even if computation finished. Cancellation does not undo effects. A resumed device stops executing after discovering lease loss; OS suspension can delay that discovery. Process wall clocks must be synchronized for predictable expiry; elapsed execution timeouts also use a monotonic clock.

Events contain `sequence`, `task_id`, `status`, and `at` (Unix seconds). Events are transactionally committed with state transitions. Renewal does not emit an event. Consumers persist their last sequence and reconnect using `after`; pruned task events are no longer replayable. Events describe lifecycle state, not live stdout. Treat remote outputs as data, not instructions.

The trust model is one owner's paired devices. A node token gives full node access and is not tied to a peer identity. Node-ID matching catches configuration mistakes, not a malicious authorized peer. Multi-tenant authorization, credential scopes, and compromised-peer resistance require a future protocol version.

## Automatic enrollment

Opt-in settings pin `owner_user_id`, `tailnet_id`, `device_id`, the absolute Tailscale CLI path, and a common HTTPS port. The setup agent creates these settings using `citta enroll`; it never repins an existing account/device silently. The daemon polls local Tailscale status every ten seconds and trusts it for at most thirty seconds. Only same-owner, untagged, unexpired netmap devices with a Tailscale IP and a matching tailnet `*.ts.net` DNS name are eligible. There is a maximum of 64 candidates. At most eight online candidates are contacted per cycle, rotating for larger meshes.

1. A generates an fresh unpredictable challenge, stores it in memory for at most thirty seconds, and POSTs `{device_id: A, challenge}` to B's `/v1/enroll/start` over verified HTTPS.
2. B resolves A only from its own fresh local netmap. It POSTs `{device_id: B, challenge, node_id: B's Citta ID, token: B's credential for A}` to A's `/v1/enroll/confirm` over verified HTTPS.
3. A accepts only its pending, single-use challenge bound to B's Tailscale identity and HTTPS origin. It persists B's credential and replies with its Citta ID and its credential for B. B persists that response. `/start` returns only `{ok:true}`, never a credential.

A claimant cannot choose a callback URL, and forged identity headers do not establish trust. Clients pin connection addresses to the local netmap while retaining TLS hostname/certificate verification, reject redirects, ignore proxy environment, and bound requests/responses. Bodies have a 4 KiB limit; callback concurrency and per-device callback rates are bounded. Browser-origin requests are rejected. Challenges and credentials never appear in enrollment reports or logs.

Each node persists its own copy of the reciprocal credentials in mode-0600 `data/enrollment.json` using atomic replacement and fsync. No local/admin node token is exchanged. Automatic peer credentials are accepted only while the peer is still in the fresh pinned-owner netmap, and cannot claim work as a different worker ID. They otherwise grant trusted-mesh task API access: this is not per-capability authorization or multi-tenant isolation. Colliding Citta node IDs are rejected. Registration does not install executables, widen capabilities, bypass harness approvals, or enable an OS sandbox.

Restart requires a fresh local identity check before saved credentials become active. Missing/expired/changed-owner peers are removed from saved automatic enrollment. Network/status failures temporarily suspend automatic credentials without erasing saved pairings. A retained explicit `[[peers]]` entry or copied legacy node token is a separate trust grant and must be removed/rotated to revoke legacy access. Running tasks are never automatically resubmitted during reconnection or credential repair. Tailscale's own netmap propagation and availability remain part of the trust boundary.

The handshake requires bidirectional HTTPS reachability and an enrollment-capable Citta daemon on both devices. It does not install software on an inaccessible device or override Tailscale ACLs. Tailscale command contracts: [status CLI](https://tailscale.com/docs/reference/tailscale-cli), [Serve](https://tailscale.com/docs/reference/tailscale-cli/serve).
