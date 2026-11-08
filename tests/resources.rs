use citta::{config, inventory, model::*, resources::*, store::Store};
use serde_json::json;
use std::sync::Arc;

fn snapshot() -> inventory::Snapshot {
    serde_json::from_value(json!({
        "observed_at":100,"agents":[],"runtimes":[],"codex_status":"connected","agents_truncated":false,
        "resources":{"cpu_model":"fixture","logical_cpus":4,"physical_cores":4,"cpu_usage_percent":25.0,
            "load_average":[1.0,1.0,1.0],"memory_total_bytes":8589934592u64,"memory_available_bytes":2147483648u64,
            "admission_memory_bytes":2147483648u64,"admission_memory_source":"fixture",
            "memory_used_bytes":6442450944u64,"swap_used_bytes":0,"os_version":"fixture","uptime_secs":100}
    })).unwrap()
}
fn offer() -> Offer {
    Offer {
        observed_at: 100,
        memory_bytes: 1024 * 1024 * 1024,
        cpu_cores: 1.0,
        os: "macos".into(),
        arch: "aarch64".into(),
        harness: Some("codex".into()),
        model: Some("fixture-model".into()),
    }
}
fn task(key: &str, requirements: Requirements) -> Submission {
    Submission {
        requirements: Some(requirements),
        agent_only: true,
        capability: "analysis".into(),
        input: json!({"prompt":"test"}),
        idempotency_key: key.into(),
        target_node: None,
        timeout_secs: 30,
        queue_ttl_secs: 100,
    }
}
fn claim() -> ClaimRequest {
    ClaimRequest {
        offer: Some(offer()),
        agent_profile: true,
        worker_node: "worker".into(),
        capability: "analysis".into(),
    }
}

#[test]
fn resource_requirements_reject_stale_unknown_wrong_platform_and_overcommit() {
    let req = Requirements {
        os: Some("macos".into()),
        arch: Some("aarch64".into()),
        harness: Some("codex".into()),
        model: Some("fixture-model".into()),
        remote_only: true,
        ..Default::default()
    };
    let o = offer();
    assert!(o.fits(&req, "worker", "origin", 101));
    assert!(!o.fits(&req, "origin", "origin", 101));
    assert!(!o.fits(&req, "worker", "origin", 121));
    assert!(!o.fits(&req, "worker", "origin", 99));
    for bad in [
        Offer {
            cpu_cores: 0.0,
            ..o.clone()
        },
        Offer {
            memory_bytes: 0,
            ..o.clone()
        },
        Offer {
            os: "linux".into(),
            ..o.clone()
        },
        Offer {
            arch: "x86_64".into(),
            ..o.clone()
        },
        Offer {
            harness: None,
            ..o.clone()
        },
        Offer {
            model: Some("wrong".into()),
            ..o.clone()
        },
        Offer {
            cpu_cores: f64::NAN,
            ..o.clone()
        },
    ] {
        assert!(!bad.fits(&req, "worker", "origin", 101));
    }
    assert!(
        Requirements {
            cpu_cores: f64::INFINITY,
            ..req.clone()
        }
        .validate()
        .is_err()
    );
    assert!(
        Requirements {
            cpu_cores: -1.0,
            ..req
        }
        .validate()
        .is_err()
    );
}

#[test]
fn admission_fails_closed_and_reservations_release_without_reusing_capacity() {
    let dir = tempfile::tempdir().unwrap();
    let path = config::init(dir.path(), "test".into(), 7331).unwrap();
    let mut cap = config::Config::load(&path).unwrap().capabilities.remove(0);
    cap.resources = Estimate {
        memory_bytes: 1024 * 1024 * 1024,
        cpu_cores: 1.0,
    };
    let a = Arc::new(Admission::default());
    let policy = Policy::default();
    let mut s = snapshot();
    assert!(!a.readiness(&policy, None, &cap, None, 100).accepting);
    assert!(!a.readiness(&policy, Some(&s), &cap, None, 121).accepting);
    assert!(a.readiness(&policy, Some(&s), &cap, None, 100).accepting);
    let reservation = a.reserve("one", cap.resources);
    assert!(!a.readiness(&policy, Some(&s), &cap, None, 100).accepting);
    assert!(
        a.readiness(&policy, Some(&s), &cap, Some("one"), 100)
            .accepting
    );
    drop(reservation);
    assert!(a.readiness(&policy, Some(&s), &cap, None, 100).accepting);
    s.resources.cpu_usage_percent = Some(99.0);
    assert_eq!(
        a.readiness(&policy, Some(&s), &cap, None, 100)
            .reason
            .as_deref(),
        Some("cpu_pressure")
    );
    s.resources.cpu_usage_percent = None;
    assert!(!a.readiness(&policy, Some(&s), &cap, None, 100).accepting);
    s.resources.cpu_usage_percent = Some(25.0);
    s.resources.admission_memory_bytes = None;
    assert!(!a.readiness(&policy, Some(&s), &cap, None, 100).accepting);
}

#[test]
fn queue_skips_unfit_tasks_preserves_idempotency_and_refuses_legacy_claims() {
    let dir = tempfile::tempdir().unwrap();
    let first_id;
    {
        let store = Store::open(dir.path(), "origin", 10, 100, 100).unwrap();
        let large = task(
            "large",
            Requirements {
                memory_bytes: 8 * 1024 * 1024 * 1024,
                ..Default::default()
            },
        );
        first_id = store.submit(large.clone(), 100).unwrap().id;
        let small = store
            .submit(task("small", Requirements::default()), 100)
            .unwrap();
        let mut legacy = claim();
        legacy.offer = None;
        assert!(store.claim(legacy, 101).unwrap().is_none());
        let mut stale = claim();
        stale.offer.as_mut().unwrap().observed_at = 0;
        assert!(store.claim(stale, 101).unwrap().is_none());
        let claimed = store.claim(claim(), 101).unwrap().unwrap();
        assert_eq!(claimed.task.id, small.id);
        assert_eq!(store.get(&first_id).unwrap().status, Status::Queued);
        assert_eq!(store.submit(large.clone(), 102).unwrap().id, first_id);
        let mut conflict = large;
        conflict.requirements.as_mut().unwrap().memory_bytes = 1;
        assert!(store.submit(conflict, 102).is_err());
    }
    let store = Store::open(dir.path(), "origin", 10, 100, 100).unwrap();
    assert_eq!(store.get(&first_id).unwrap().status, Status::Queued);
    let mut enough = claim();
    enough.offer.as_mut().unwrap().memory_bytes = 16 * 1024 * 1024 * 1024;
    assert_eq!(store.claim(enough, 103).unwrap().unwrap().task.id, first_id);
}

#[tokio::test]
async fn concurrent_admission_cannot_reserve_the_same_snapshot_twice() {
    let dir = tempfile::tempdir().unwrap();
    let path = config::init(dir.path(), "test".into(), 7331).unwrap();
    let mut cap = config::Config::load(&path).unwrap().capabilities.remove(0);
    cap.resources = Estimate {
        memory_bytes: 1024 * 1024 * 1024,
        cpu_cores: 1.0,
    };
    let a = Arc::new(Admission::default());
    let mut jobs = tokio::task::JoinSet::new();
    for id in ["one", "two"] {
        let a = a.clone();
        let cap = cap.clone();
        jobs.spawn(async move {
            let _gate = a.claims.lock().await;
            a.readiness(&Policy::default(), Some(&snapshot()), &cap, None, 100)
                .accepting
                .then(|| a.reserve(id, cap.resources))
        });
    }
    let mut retained = Vec::new();
    while let Some(result) = jobs.join_next().await {
        retained.push(result.unwrap());
    }
    assert_eq!(retained.iter().filter(|v| v.is_some()).count(), 1);
}
