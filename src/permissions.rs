//! Device-owned execution policy. Management credentials are distinct from task credentials.
use crate::{
    config::Config,
    model::{now, valid_name},
    store::{Rejection, Store},
};
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Restricted,
    AllowAll,
    Paused,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub mode: Mode,
    pub allowed_capabilities: Vec<String>,
    /// None accepts all currently trusted origins; Some accepts only these stable identities.
    pub allowed_origins: Option<Vec<String>>,
}
impl Policy {
    pub fn initial(config: &Config) -> Self {
        Self {
            mode: Mode::Restricted,
            allowed_capabilities: config.capabilities.iter().map(|c| c.name.clone()).collect(),
            allowed_origins: None,
        }
    }
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.allowed_capabilities.len() <= 64
                && self.allowed_capabilities.iter().all(|n| valid_name(n)),
            "invalid allowed capabilities"
        );
        ensure!(
            self.allowed_capabilities
                .iter()
                .collect::<HashSet<_>>()
                .len()
                == self.allowed_capabilities.len(),
            "duplicate capabilities"
        );
        if let Some(origins) = &self.allowed_origins {
            ensure!(
                origins.len() <= 129
                    && origins.iter().all(|v| v
                        .split_once(':')
                        .is_some_and(|(kind, id)| matches!(kind, "local" | "tailscale" | "peer")
                            && valid_name(id))),
                "invalid allowed origins"
            );
            ensure!(
                origins.iter().collect::<HashSet<_>>().len() == origins.len(),
                "duplicate origins"
            );
        }
        Ok(())
    }
    pub fn allows_capability(&self, capability: &str) -> bool {
        self.mode != Mode::Paused
            && (self.mode == Mode::AllowAll
                || self.allowed_capabilities.iter().any(|c| c == capability))
    }
    pub fn allows(&self, origin: &str, capability: &str) -> bool {
        self.mode != Mode::Paused
            && self
                .allowed_origins
                .as_ref()
                .is_none_or(|origins| origins.iter().any(|id| id == origin))
            && (self.mode == Mode::AllowAll
                || self
                    .allowed_capabilities
                    .iter()
                    .any(|name| name == capability))
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub revision: i64,
    pub policy: Policy,
    pub updated_at: i64,
    pub actor: String,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Update {
    pub expected_revision: i64,
    pub policy: Policy,
}
pub struct Permissions {
    state: Mutex<Snapshot>,
    store: Arc<Store>,
    active: Mutex<BTreeMap<String, Active>>,
}
struct Active {
    origin: String,
    capability: String,
    cancel: CancellationToken,
}
pub struct Run {
    permissions: Arc<Permissions>,
    id: String,
    cancel: CancellationToken,
}
impl Run {
    pub async fn revoked(&self) {
        self.cancel.cancelled().await;
    }
}
impl Drop for Run {
    fn drop(&mut self) {
        self.permissions
            .active
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&self.id);
    }
}
impl Permissions {
    pub fn open(config: &Config, store: Arc<Store>) -> Result<Arc<Self>> {
        let state = store.initialize_permissions(Policy::initial(config))?;
        state.policy.validate()?;
        Ok(Arc::new(Self {
            state: Mutex::new(state),
            store,
            active: Mutex::new(BTreeMap::new()),
        }))
    }
    pub fn snapshot(&self) -> Snapshot {
        self.state.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }
    pub fn allows(&self, origin: &str, capability: &str) -> bool {
        self.state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .policy
            .allows(origin, capability)
    }
    /// Policy updates and process launch share this lock, preventing check/spawn races.
    pub fn launch<T>(
        self: &Arc<Self>,
        origin: &str,
        capability: &str,
        spawn: impl FnOnce() -> Result<T>,
    ) -> Result<(T, Run)> {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        ensure!(
            state.policy.allows(origin, capability),
            "device permissions deny this execution"
        );
        let value = spawn()?;
        let id = uuid::Uuid::new_v4().to_string();
        let cancel = CancellationToken::new();
        self.active
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(
                id.clone(),
                Active {
                    origin: origin.into(),
                    capability: capability.into(),
                    cancel: cancel.clone(),
                },
            );
        Ok((
            value,
            Run {
                permissions: self.clone(),
                id,
                cancel,
            },
        ))
    }
    pub fn update(&self, request: Update, actor: String) -> Result<Snapshot> {
        request.policy.validate()?;
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.revision != request.expected_revision {
            return Err(Rejection::Conflict.into());
        }
        let revision = state
            .revision
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("policy revision exhausted"))?;
        let next = Snapshot {
            revision,
            policy: request.policy,
            updated_at: now(),
            actor,
        };
        self.store.save_permissions(&next, state.revision)?;
        *state = next.clone();
        for run in self
            .active
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .values()
        {
            if !next.policy.allows(&run.origin, &run.capability) {
                run.cancel.cancel();
            }
        }
        Ok(next)
    }
}

pub fn control_token(config: &Config) -> Result<String> {
    let path = config.data_dir.join("control.token");
    if !path.exists() {
        crate::enrollment::atomic_private(
            &path,
            format!(
                "{}{}",
                uuid::Uuid::new_v4().simple(),
                uuid::Uuid::new_v4().simple()
            )
            .as_bytes(),
        )?;
    }
    crate::config::read_token(&path)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn allow_all_keeps_origin_scope_and_pause_denies_everything() {
        let mut p = Policy {
            mode: Mode::Restricted,
            allowed_capabilities: vec!["echo".into()],
            allowed_origins: Some(vec!["tailscale:one".into()]),
        };
        assert!(p.allows("tailscale:one", "echo"));
        assert!(!p.allows("tailscale:two", "echo"));
        assert!(!p.allows("tailscale:one", "agent"));
        p.mode = Mode::AllowAll;
        assert!(p.allows("tailscale:one", "agent"));
        assert!(!p.allows("tailscale:two", "agent"));
        p.mode = Mode::Paused;
        assert!(!p.allows("tailscale:one", "echo"));
    }
}
