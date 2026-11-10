use citta::{model::*, store::Store};
use serde_json::json;
use std::sync::Arc;

fn request(key: &str) -> Submission {
    Submission {
        requirements: None,
        agent_only: false,
        capability: "test".into(),
        input: json!({"n":1}),
        idempotency_key: key.into(),
        target_node: Some("worker".into()),
        timeout_secs: 60,
        queue_ttl_secs: 100,
    }
}
fn claim() -> ClaimRequest {
    ClaimRequest {
        disk: Some(citta::resources::DiskReadiness::from_available(
            Some(1024),
            Some(1024),
            64,
            100,
        )),
        offer: None,
        agent_profile: false,
        worker_node: "worker".into(),
        capability: "test".into(),
    }
}
fn outcome() -> Outcome {
    Outcome {
        status: Status::Succeeded,
        stdout: "done".into(),
        stderr: String::new(),
        exit_code: Some(0),
        error: None,
    }
}
#[test]
fn idempotency_conflicts_and_capacity() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path(), "origin", 10, 1, 10).unwrap();
    let a = store.submit(request("key"), 100).unwrap();
    assert_eq!(a.id, store.submit(request("key"), 101).unwrap().id);
    let mut changed = request("key");
    changed.input = json!(2);
    assert!(store.submit(changed, 101).is_err());
    assert!(store.submit(request("other"), 101).is_err());
    assert_eq!(store.events(0).unwrap().len(), 1);
}
#[test]
fn one_claim_under_contention_and_target_matching() {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path(), "origin", 10, 100, 100).unwrap());
    store.submit(request("key"), 100).unwrap();
    assert!(
        store
            .claim(
                ClaimRequest {
                    disk: Some(citta::resources::DiskReadiness::from_available(
                        Some(1024),
                        Some(1024),
                        64,
                        100
                    )),
                    offer: None,
                    agent_profile: false,
                    worker_node: "wrong".into(),
                    capability: "test".into()
                },
                101
            )
            .unwrap()
            .is_none()
    );
    let handles: Vec<_> = (0..16)
        .map(|_| {
            let store = store.clone();
            std::thread::spawn(move || store.claim(claim(), 101).unwrap().is_some())
        })
        .collect();
    assert_eq!(
        handles
            .into_iter()
            .filter(|h| h.thread().id() != std::thread::current().id())
            .map(|h| usize::from(h.join().unwrap()))
            .sum::<usize>(),
        1
    );
}
#[test]
fn expired_lease_never_replays_or_accepts_late_results() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path(), "origin", 10, 100, 100).unwrap();
    let task = store.submit(request("key"), 100).unwrap();
    let lease = store.claim(claim(), 101).unwrap().unwrap();
    assert!(store.renew(&task.id, "wrong", 102).is_err());
    assert!(store.renew(&task.id, &lease.lease_token, 111).is_err());
    assert!(
        store
            .complete(
                &task.id,
                Completion {
                    lease_token: lease.lease_token.clone(),
                    outcome: outcome()
                },
                111
            )
            .is_err()
    );
    assert_eq!(store.sweep(111).unwrap(), 1);
    assert_eq!(store.get(&task.id).unwrap().status, Status::Interrupted);
    assert!(store.claim(claim(), 112).unwrap().is_none());
}
#[test]
fn cancellation_fences_worker_and_completion_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path(), "origin", 10, 100, 100).unwrap();
    let task = store.submit(request("cancel"), 100).unwrap();
    let lease = store.claim(claim(), 101).unwrap().unwrap();
    store.cancel(&task.id, 102).unwrap();
    assert!(store.renew(&task.id, &lease.lease_token, 103).is_err());
    assert!(
        store
            .complete(
                &task.id,
                Completion {
                    lease_token: lease.lease_token,
                    outcome: outcome()
                },
                103
            )
            .is_err()
    );
    let task = store.submit(request("complete"), 100).unwrap();
    let lease = store.claim(claim(), 101).unwrap().unwrap();
    let completion = Completion {
        lease_token: lease.lease_token,
        outcome: outcome(),
    };
    assert_eq!(
        store
            .complete(&task.id, completion.clone(), 102)
            .unwrap()
            .status,
        Status::Succeeded
    );
    assert_eq!(
        store.complete(&task.id, completion, 200).unwrap().status,
        Status::Succeeded
    );
    assert_eq!(store.events(0).unwrap().len(), 6);
    assert_eq!(
        store.cancel(&task.id, 201).unwrap().status,
        Status::Succeeded
    );
}
#[test]
fn restart_preserves_state_events_and_exclusive_ownership() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path(), "origin", 10, 100, 100).unwrap();
    let task = store.submit(request("key"), 100).unwrap();
    assert!(Store::open(dir.path(), "origin", 10, 100, 100).is_err());
    drop(store);
    assert!(Store::open(dir.path(), "different", 10, 100, 100).is_err());
    let store = Store::open(dir.path(), "origin", 10, 100, 100).unwrap();
    assert_eq!(store.get(&task.id).unwrap().status, Status::Queued);
    assert_eq!(store.events(0).unwrap().len(), 1);
    assert_eq!(store.sweep(200).unwrap(), 1);
    assert_eq!(store.prune(201).unwrap(), 1);
    assert!(store.events(0).unwrap().is_empty());
    let new = store.submit(request("new"), 201).unwrap();
    assert_ne!(new.id, task.id);
    assert!(store.events(0).unwrap()[0].sequence > 2);
}

#[test]
fn agent_runs_cannot_be_claimed_by_ordinary_or_legacy_workers() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path(), "origin", 10, 100, 100).unwrap();
    let mut agent = request("agent");
    agent.agent_only = true;
    let agent = store.submit(agent, 100).unwrap();
    let ordinary = store.submit(request("ordinary"), 100).unwrap();
    assert_eq!(
        store.claim(claim(), 101).unwrap().unwrap().task.id,
        ordinary.id
    );
    assert!(store.claim(claim(), 101).unwrap().is_none());
    let mut agent_claim = claim();
    agent_claim.agent_profile = true;
    assert_eq!(
        store.claim(agent_claim, 101).unwrap().unwrap().task.id,
        agent.id
    );
    assert!(
        !serde_json::to_string(&claim())
            .unwrap()
            .contains("agent_profile")
    );
}

#[test]
fn legacy_tasks_migrate_and_newer_storage_fails_closed() {
    let dir = tempfile::tempdir().unwrap();
    let task_id = {
        let store = Store::open(dir.path(), "origin", 10, 100, 100).unwrap();
        store.submit(request("legacy"), 100).unwrap().id
    };
    let db = rusqlite::Connection::open(dir.path().join("citta.db")).unwrap();
    db.execute_batch("PRAGMA user_version=1").unwrap();
    {
        let store = Store::open(dir.path(), "origin", 10, 100, 100).unwrap();
        assert!(!store.get(&task_id).unwrap().request.agent_only);
        assert_eq!(store.claim(claim(), 101).unwrap().unwrap().task.id, task_id);
    }
    assert_eq!(
        db.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        4
    );
    db.execute_batch("PRAGMA user_version=5").unwrap();
    assert!(Store::open(dir.path(), "origin", 10, 100, 100).is_err());
}

#[test]
fn permission_updates_persist_with_audit_and_conflicts_do_not_overwrite() {
    use citta::permissions::{Mode, Policy, Snapshot};
    let dir = tempfile::tempdir().unwrap();
    let original = Policy {
        mode: Mode::Restricted,
        allowed_capabilities: vec!["echo".into()],
        allowed_origins: None,
    };
    {
        let store = Store::open(dir.path(), "origin", 10, 100, 100).unwrap();
        let state = store.initialize_permissions(original.clone()).unwrap();
        let next = Snapshot {
            revision: 2,
            policy: Policy {
                mode: Mode::AllowAll,
                ..original.clone()
            },
            updated_at: 100,
            actor: "tailscale:owner-device".into(),
        };
        store.save_permissions(&next, state.revision).unwrap();
        assert!(store.save_permissions(&next, state.revision).is_err());
        assert_eq!(store.permission_history().unwrap().len(), 2);
    }
    let store = Store::open(dir.path(), "origin", 10, 100, 100).unwrap();
    let state = store.initialize_permissions(original).unwrap();
    assert_eq!(state.revision, 2);
    assert_eq!(state.policy.mode, Mode::AllowAll);
    assert_eq!(
        store.permission_history().unwrap()[0].actor,
        "tailscale:owner-device"
    );
    let db = rusqlite::Connection::open(dir.path().join("citta.db")).unwrap();
    db.execute("DELETE FROM device_permissions", []).unwrap();
    assert!(
        store.initialize_permissions(state.policy).is_err(),
        "missing saved policy must not reset grants"
    );
}
