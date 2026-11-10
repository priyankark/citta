//! Admission estimates, not OS quotas. Reservations prevent simultaneous claims
//! from spending the same observed capacity; running tasks are never migrated.
use crate::{config::Capability, inventory::Snapshot, model::valid_name};
use anyhow::{Result, ensure};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::Path,
    sync::{Arc, Mutex},
};

pub const MAX_AGE: i64 = 20;

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Estimate {
    pub memory_bytes: u64,
    pub cpu_cores: f64,
}
impl Estimate {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.memory_bytes <= (1u64 << 50),
            "memory estimate exceeds 1 PiB"
        );
        ensure!(
            self.cpu_cores.is_finite() && (0.0..=65536.0).contains(&self.cpu_cores),
            "invalid CPU estimate"
        );
        Ok(())
    }
    pub fn max(self, other: Self) -> Self {
        Self {
            memory_bytes: self.memory_bytes.max(other.memory_bytes),
            cpu_cores: self.cpu_cores.max(other.cpu_cores),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Requirements {
    pub memory_bytes: u64,
    pub cpu_cores: f64,
    pub os: Option<String>,
    pub arch: Option<String>,
    pub harness: Option<String>,
    pub model: Option<String>,
    /// Require a worker on a different device from the task origin.
    pub remote_only: bool,
}
impl Default for Requirements {
    fn default() -> Self {
        Self {
            memory_bytes: 256 * 1024 * 1024,
            cpu_cores: 0.25,
            os: None,
            arch: None,
            harness: None,
            model: None,
            remote_only: false,
        }
    }
}
impl Requirements {
    pub fn estimate(&self) -> Estimate {
        Estimate {
            memory_bytes: self.memory_bytes,
            cpu_cores: self.cpu_cores,
        }
    }
    pub fn validate(&self) -> Result<()> {
        self.estimate().validate()?;
        for name in [&self.os, &self.arch, &self.harness].into_iter().flatten() {
            ensure!(valid_name(name), "invalid resource requirement identifier");
        }
        ensure!(
            self.model.as_ref().is_none_or(|v| !v.trim().is_empty()
                && v.len() <= 256
                && !v.chars().any(char::is_control)),
            "invalid required model"
        );
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Policy {
    /// Apply admission to legacy submissions too. Resource-qualified tasks always enforce it.
    pub guard_all: bool,
    pub memory_headroom_bytes: u64,
    /// Minimum free space on both the data and work filesystems, for every task.
    pub disk_headroom_bytes: u64,
    pub max_cpu_percent: f64,
}
impl Default for Policy {
    fn default() -> Self {
        Self {
            guard_all: false,
            memory_headroom_bytes: 512 * 1024 * 1024,
            disk_headroom_bytes: 64 * 1024 * 1024,
            max_cpu_percent: 90.0,
        }
    }
}
impl Policy {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.max_cpu_percent.is_finite() && (1.0..=100.0).contains(&self.max_cpu_percent),
            "max_cpu_percent must be 1..100"
        );
        ensure!(
            self.memory_headroom_bytes <= (1u64 << 50),
            "invalid memory headroom"
        );
        ensure!(
            (1..=(1u64 << 50)).contains(&self.disk_headroom_bytes),
            "disk_headroom_bytes must be 1..1 PiB"
        );
        Ok(())
    }
}

/// Fresh filesystem observation; this is a reserve check, not a disk quota.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskReadiness {
    pub observed_at: i64,
    pub data_available_bytes: Option<u64>,
    pub work_available_bytes: Option<u64>,
    pub reserve_bytes: u64,
    pub accepting: bool,
    pub reason: Option<String>,
}
impl DiskReadiness {
    pub fn from_available(data: Option<u64>, work: Option<u64>, reserve: u64, at: i64) -> Self {
        let reason = match (data, work) {
            (Some(data), Some(work)) if data >= reserve && work >= reserve => None,
            (Some(_), Some(_)) => Some("disk_low".into()),
            _ => Some("disk_unknown".into()),
        };
        Self {
            observed_at: at,
            data_available_bytes: data,
            work_available_bytes: work,
            reserve_bytes: reserve,
            accepting: reason.is_none(),
            reason,
        }
    }
    pub fn probe(data_dir: &Path, policy: &Policy) -> Self {
        Self::from_available(
            available_space(data_dir),
            available_space(&data_dir.join("work")),
            policy.disk_headroom_bytes,
            crate::model::now(),
        )
    }
    pub fn fresh_reason(&self, at: i64) -> Option<&str> {
        if self.reserve_bytes == 0
            || !at
                .checked_sub(self.observed_at)
                .is_some_and(|age| (0..=MAX_AGE).contains(&age))
        {
            Some("disk_unknown")
        } else if !self.accepting
            || self.data_available_bytes.is_none()
            || self.work_available_bytes.is_none()
        {
            Some(self.reason.as_deref().unwrap_or("disk_unknown"))
        } else if self
            .data_available_bytes
            .is_some_and(|n| n < self.reserve_bytes)
            || self
                .work_available_bytes
                .is_some_and(|n| n < self.reserve_bytes)
        {
            Some("disk_low")
        } else {
            None
        }
    }
}

fn available_space(path: &Path) -> Option<u64> {
    // A new work directory inherits the filesystem of its nearest existing
    // directory. Permission errors and non-directories remain unknown.
    let mut current = path;
    loop {
        match std::fs::metadata(current) {
            Ok(meta) if meta.is_dir() => return fs2::available_space(current).ok(),
            Ok(_) => return None,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                // metadata follows symlinks: a dangling link is an existing,
                // unusable work path, not an uncreated directory on its parent.
                if std::fs::symlink_metadata(current).is_ok() {
                    return None;
                }
                current = current.parent()?;
            }
            Err(_) => return None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Offer {
    pub observed_at: i64,
    pub memory_bytes: u64,
    pub cpu_cores: f64,
    pub os: String,
    pub arch: String,
    pub harness: Option<String>,
    pub model: Option<String>,
}
impl Offer {
    pub fn fits(&self, req: &Requirements, worker: &str, origin: &str, at: i64) -> bool {
        (0..=MAX_AGE).contains(&(at - self.observed_at))
            && self.cpu_cores.is_finite()
            && self.cpu_cores >= req.cpu_cores
            && self.memory_bytes >= req.memory_bytes
            && (!req.remote_only || worker != origin)
            && req.os.as_ref().is_none_or(|v| *v == self.os)
            && req.arch.as_ref().is_none_or(|v| *v == self.arch)
            && req
                .harness
                .as_ref()
                .is_none_or(|v| Some(v) == self.harness.as_ref())
            && req
                .model
                .as_ref()
                .is_none_or(|v| Some(v) == self.model.as_ref())
    }
    pub fn can_reserve(&self, estimate: Estimate) -> bool {
        self.memory_bytes >= estimate.memory_bytes && self.cpu_cores >= estimate.cpu_cores
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Readiness {
    pub accepting: bool,
    pub reason: Option<String>,
    pub offer: Option<Offer>,
}
impl Readiness {
    pub fn with_disk(mut self, disk: &DiskReadiness, at: i64) -> Self {
        if let Some(reason) = disk.fresh_reason(at) {
            self.accepting = false;
            self.reason = Some(reason.into());
            self.offer = None;
        }
        self
    }
}
#[derive(Default)]
pub struct Admission {
    // Held across offer/claim/reservation so two workers cannot reserve the same snapshot.
    pub claims: tokio::sync::Mutex<()>,
    reserved: Mutex<BTreeMap<String, Estimate>>,
}
impl Admission {
    pub fn reserve(self: &Arc<Self>, id: &str, estimate: Estimate) -> Reservation {
        self.reserved
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(id.into(), estimate);
        Reservation {
            admission: self.clone(),
            id: id.into(),
        }
    }
    pub fn readiness(
        &self,
        policy: &Policy,
        snapshot: Option<&Snapshot>,
        cap: &Capability,
        excluding: Option<&str>,
        at: i64,
    ) -> Readiness {
        let failure = |reason: &str| Readiness {
            accepting: false,
            reason: Some(reason.into()),
            offer: None,
        };
        let Some(snapshot) = snapshot else {
            return failure("telemetry_unavailable");
        };
        if !(0..=MAX_AGE).contains(&(at - snapshot.observed_at)) {
            return failure("telemetry_stale");
        }
        let r = &snapshot.resources;
        let Some(cpu) = r
            .cpu_usage_percent
            .filter(|v| v.is_finite() && (0.0..=100.0).contains(v))
        else {
            return failure("cpu_unknown");
        };
        if f64::from(cpu) >= policy.max_cpu_percent {
            return failure("cpu_pressure");
        }
        let Some(memory) = r.admission_memory_bytes else {
            return failure("memory_unknown");
        };
        let reserved = self.reserved.lock().unwrap_or_else(|e| e.into_inner());
        let used = reserved
            .iter()
            .filter(|(id, _)| Some(id.as_str()) != excluding)
            .fold(Estimate::default(), |a, (_, b)| Estimate {
                memory_bytes: a.memory_bytes.saturating_add(b.memory_bytes),
                cpu_cores: a.cpu_cores + b.cpu_cores,
            });
        let memory = memory
            .saturating_sub(policy.memory_headroom_bytes)
            .saturating_sub(used.memory_bytes);
        let cpu =
            ((1.0 - f64::from(cpu) / 100.0) * r.logical_cpus as f64 - used.cpu_cores).max(0.0);
        let offer = Offer {
            observed_at: snapshot.observed_at,
            memory_bytes: memory,
            cpu_cores: cpu,
            os: std::env::consts::OS.into(),
            arch: std::env::consts::ARCH.into(),
            harness: cap.harness.clone(),
            model: cap.model.clone(),
        };
        let accepting = offer.can_reserve(cap.resources);
        Readiness {
            accepting,
            reason: (!accepting).then(|| "profile_resource_estimate_exceeds_capacity".into()),
            offer: Some(offer),
        }
    }
}
pub struct Reservation {
    admission: Arc<Admission>,
    id: String,
}
impl Drop for Reservation {
    fn drop(&mut self) {
        self.admission
            .reserved
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&self.id);
    }
}
