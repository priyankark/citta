# Peer protocol v1

Task routes require `Authorization: Bearer <credential>`: the local/legacy node token or an active automatic peer credential. The two enrollment handshake routes below use a separate challenge protocol. Requests with an `Origin` header are rejected. Local and explicit-peer tokens are loaded once on startup; automatic peer credentials refresh in the running daemon. Remote traffic must arrive through HTTPS (normally Tailscale Serve); the daemon only binds numeric loopback. Clients refuse redirects and do not use ambient HTTP proxies.

| Method/path | Behavior |
| --- | --- |
| GET `/healthz` | Authenticated liveness |
| GET `/v1/node` | Protocol version, node ID, OS/arch, capabilities, configured slots, active jobs, optional hardware/session inventory and managed agents |
| GET `/v1/discover` | Local manifest plus live manifests from paired peers; failures are explicit |
| GET `/v1/agents` | Same live mesh manifest as discovery, including sessions/resources/profile kinds |
| POST `/v1/agents/spawn` | Validate an advertised agent profile and enqueue a bounded prompt run |
| POST `/v1/panel/session` | Separate local management token only: issue single-use local/private HTTPS browser login links |
| GET/POST `/v1/control/permissions` | Separate owner management credentials: read policy/history or atomically update using expected revision |
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

The trust model is one owner's paired devices. A node task token gives task API access and is not tied to a peer identity. Node-ID matching catches configuration mistakes, not a malicious authorized peer. Multi-tenant authorization, credential scopes, and compromised-peer resistance require a future protocol version.

## Automatic enrollment

Opt-in settings pin `owner_user_id`, `tailnet_id`, `device_id`, the absolute Tailscale CLI path, and a common HTTPS port. The setup agent creates these settings using `citta enroll`; it never repins an existing account/device silently. The daemon polls local Tailscale status every ten seconds and trusts it for at most thirty seconds. Only same-owner, untagged, unexpired netmap devices with a Tailscale IP and a matching tailnet `*.ts.net` DNS name are eligible. There is a maximum of 64 candidates. At most eight online candidates are contacted per cycle, rotating for larger meshes.

1. A generates an fresh unpredictable challenge, stores it in memory for at most thirty seconds, and POSTs `{device_id: A, challenge}` to B's `/v1/enroll/start` over verified HTTPS.
2. B resolves A only from its own fresh local netmap. It POSTs `{device_id: B, challenge, node_id: B's Citta ID, token: B's credential for A}` to A's `/v1/enroll/confirm` over verified HTTPS.
3. A accepts only its pending, single-use challenge bound to B's Tailscale identity and HTTPS origin. It persists B's credential and replies with its Citta ID and its credential for B. B persists that response. `/start` returns only `{ok:true}`, never a credential.

A claimant cannot choose a callback URL, and forged identity headers do not establish trust. Clients pin connection addresses to the local netmap while retaining TLS hostname/certificate verification, reject redirects, ignore proxy environment, and bound requests/responses. Bodies have a 4 KiB limit; callback concurrency and per-device callback rates are bounded. Browser-origin requests are rejected. Challenges and credentials never appear in enrollment reports or logs.

Each node persists its own copy of the reciprocal credentials in mode-0600 `data/enrollment.json` using atomic replacement and fsync. No local/admin node token is exchanged. Automatic peer credentials are accepted only while the peer is still in the fresh pinned-owner netmap, and cannot claim work as a different worker ID. They otherwise grant trusted-mesh task API access: this is not per-capability authorization or multi-tenant isolation. Colliding Citta node IDs are rejected. Registration does not install executables, widen capabilities, bypass harness approvals, or enable an OS sandbox.

Restart requires a fresh local identity check before saved credentials become active. Missing/expired/changed-owner peers are removed from saved automatic enrollment. Network/status failures temporarily suspend automatic credentials without erasing saved pairings. A retained explicit `[[peers]]` entry or copied legacy node token is a separate trust grant and must be removed/rotated to revoke legacy access. Running tasks are never automatically resubmitted during reconnection or credential repair. Tailscale's own netmap propagation and availability remain part of the trust boundary.

The handshake requires bidirectional HTTPS reachability and an enrollment-capable Citta daemon on both devices. It does not install software on an inaccessible device or override Tailscale ACLs. Tailscale command contracts: [status CLI](https://tailscale.com/docs/reference/tailscale-cli), [Serve](https://tailscale.com/docs/reference/tailscale-cli/serve).

## Additive discovery and panel extension

Capabilities advertise `kind` (`task` by default, or `agent`) and optional declared `harness` / `model` strings. Missing metadata means unspecified. Managed agent records carry the selected profile’s metadata; observed Codex sessions use the harness registry’s model field. `inventory` is optional for older nodes; `managed_agents` defaults to an empty list. Consumers must not infer idle capacity from an absent/stale inventory. See [discovery](discovery.md) for sampling, provenance, and data minimization.

Agent spawn body: `{"profile":"codex-readonly","prompt":"Analyze this input","idempotency_key":"analysis-1","timeout_secs":180}`. Optional `target_node` pins placement; queue TTL defaults to 3600 seconds. Unknown fields are rejected. Empty prompts return 400; no reachable advertising profile returns 422. Existing equal submissions remain retrievable while peers are offline. The stored Submission adds `agent_only:true`; only a claim with `agent_profile:true` can take it. These fields default false and are omitted when false, preserving ordinary task traffic with older peers. Legacy workers cannot claim agent-only runs. Profile type is a scheduling contract within the trusted mesh, not an authorization boundary against a malicious peer.

The embedded static page and `/panel/*` use a separate owner browser session, verified local/private HTTPS Host/Origin checks, SameSite cookies, and a restrictive CSP. Panel session creation is denied to automatic peer credentials. `/panel/task` resolves only known local/enrolled node IDs, never a caller-supplied URL. Browser cookies cannot authorize task/control API calls. See [monitoring](discovery.md#human-monitoring) for session lifetime and access.

## Owner management and execution permissions

The management API accepts `data/control.token` locally or a current management peer credential. It rejects node/task peer credentials. POST `/v1/control/permissions` takes `{"expected_revision":1,"policy":{"mode":"restricted","allowed_capabilities":["echo"],"allowed_origins":null}}`. Modes are `restricted`, `allow_all`, and `paused`. A null origin list means all trusted devices; an empty list allows none. Named origins are `local:<node>`, `peer:<node>` for explicit legacy peers, or `tailscale:<stable-device-id>`. Unknown fields and invalid names are rejected. A stale revision returns 409. The target persists the policy plus audit event before acknowledging success.

Management enrollment mirrors the verified task handshake at `/v1/control/enroll/start` and `/v1/control/enroll/confirm`, with independent state and credentials. A task-channel challenge cannot authenticate a management callback. A management peer cannot mint a browser session on another device. Both credential channels require fresh same-owner identity.

Node manifests expose the current permission revision/policy as discovery metadata. Task `requester` is assigned by the server from authentication, never accepted from a submitted payload. It prevents remote submission into a local queue from inheriting local-device work permissions. Older tasks without requester attribution remain compatible; an explicitly scoped local policy does not treat them as attributed local work. See [permissions](permissions.md) for queue, revocation, trust, and browser behavior.

## Disk readiness on claims

Nodes with disk admission expose optional `disk` metadata: `observed_at`, `data_available_bytes`, `work_available_bytes`, `reserve_bytes`, `accepting`, and `reason`. Missing measurements, a zero reserve, observations older than 20 seconds or from the future, and capacity below the worker's configured positive reserve are not eligible. Reported values are available bytes on the configured data/work filesystems, not a storage quota.

`POST /v1/claim` adds an optional `disk` object with that shape. New origins require a fresh accepting value for **all** tasks and reply HTTP 428 without claiming a task when it is missing, stale, unknown or low. The store repeats the admission check before selecting work. Missing readiness returns `disk_readiness_required`; other failures carry the disk reason. This check enforces the trusted worker's reported readiness, not attestation against a malicious peer.

A new worker includes `disk` only when the origin's node manifest exposes that field. It omits the field for an older origin to preserve the older strict request schema, and still checks its own disk before claiming and launching. Old workers cannot consume work from new origins until upgraded. API protocol version and resource offer version remain 1; field presence negotiates this additional claim requirement. All devices must use the qualified candidate for release acceptance.
