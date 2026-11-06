use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const MAX_INPUT: usize = 64 * 1024;
pub const MAX_OUTPUT: usize = 256 * 1024;
pub const MAX_BODY: usize = 4 * 1024 * 1024;

pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
pub fn valid_name(s: &str) -> bool {
    !s.is_empty()
        && s != "."
        && s != ".."
        && s.len() <= 96
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Submission {
    pub capability: String,
    pub input: Value,
    /// Reuse this key when retrying a submission; use a new key for intentional re-execution.
    pub idempotency_key: String,
    #[serde(default)]
    pub target_node: Option<String>,
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    #[serde(default = "default_ttl")]
    pub queue_ttl_secs: u64,
}
fn default_timeout() -> u64 {
    300
}
fn default_ttl() -> u64 {
    3600
}
impl Submission {
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(valid_name(&self.capability), "invalid capability name");
        anyhow::ensure!(valid_name(&self.idempotency_key), "invalid idempotency key");
        anyhow::ensure!(
            self.target_node.as_ref().is_none_or(|n| valid_name(n)),
            "invalid target node"
        );
        anyhow::ensure!(
            (1..=86400).contains(&self.timeout_secs),
            "timeout_secs must be 1..86400"
        );
        anyhow::ensure!(
            (1..=604800).contains(&self.queue_ttl_secs),
            "queue_ttl_secs must be 1..604800"
        );
        anyhow::ensure!(
            serde_json::to_vec(&self.input)?.len() <= MAX_INPUT,
            "input exceeds 64 KiB"
        );
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
    Interrupted,
}
impl Status {
    pub fn terminal(self) -> bool {
        !matches!(self, Self::Queued | Self::Running)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub origin_node: String,
    pub request: Submission,
    pub status: Status,
    pub worker_node: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub started_at: Option<i64>,
    pub lease_expires_at: Option<i64>,
    pub outcome: Option<Outcome>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Outcome {
    pub status: Status,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub error: Option<String>,
}
impl Outcome {
    pub fn error(status: Status, reason: impl Into<String>) -> Self {
        Self {
            status,
            stdout: String::new(),
            stderr: String::new(),
            exit_code: None,
            error: Some(reason.into()),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimRequest {
    pub worker_node: String,
    pub capability: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    pub task: Task,
    pub lease_token: String,
    pub lease_secs: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LeaseRequest {
    pub lease_token: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Completion {
    pub lease_token: String,
    pub outcome: Outcome,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub sequence: i64,
    pub task_id: String,
    pub status: Status,
    pub at: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityInfo {
    pub name: String,
    pub description: String,
    pub slots: usize,
    pub active: usize,
    pub max_timeout_secs: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub protocol: u32,
    pub node_id: String,
    pub version: String,
    pub os: String,
    pub arch: String,
    pub slots: usize,
    pub active: usize,
    pub capabilities: Vec<CapabilityInfo>,
}
