//! View models sent to the frontend. The frontend renders `Snapshot` and
//! never sees tokens.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const SNAPSHOT_EVENT: &str = "orbitswitch://snapshot";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub platform: &'static str,
    pub version: &'static str,
    pub accounts: Vec<AccountView>,
    pub client: ClientStatus,
    pub operation: Operation,
    pub refreshing: bool,
    pub refreshed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountView {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub plan: Option<Plan>,
    pub quota: QuotaState,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum QuotaState {
    Loading,
    #[serde(rename_all = "camelCase")]
    Ready {
        groups: Vec<QuotaGroup>,
        fetched_at: DateTime<Utc>,
        stale: bool,
    },
    Failed {
        code: String,
        message: String,
    },
    /// Authorization is no longer accepted; the account must sign in again.
    Reauth,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientStatus {
    pub active_id: Option<String>,
    pub installed: bool,
    pub running: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Operation {
    Idle,
    #[serde(rename_all = "camelCase")]
    Adding {
        can_copy_link: bool,
    },
    Removing,
    #[serde(rename_all = "camelCase")]
    Switching {
        target_id: String,
        step: SwitchStep,
    },
    Stopping,
    Restarting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SwitchStep {
    Preparing,
    Closing,
    Writing,
    Launching,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GroupKey {
    Gemini,
    Claude,
    Other,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaWindow {
    /// 0.0–1.0 share of the window still available.
    pub remaining: f64,
    pub reset_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub duration_minutes: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaGroup {
    pub key: GroupKey,
    pub label: String,
    pub models: Vec<String>,
    pub five_hour: Option<QuotaWindow>,
    pub weekly: Option<QuotaWindow>,
    #[serde(default)]
    pub extra: Vec<QuotaWindow>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PlanKind {
    Free,
    Pro,
    Ultra,
    Plus,
    Max,
    Team,
    Business,
    Enterprise,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    pub kind: PlanKind,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ProcessKind {
    App,
    Helper,
    LanguageServer,
    Cli,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelatedProcess {
    pub pid: u32,
    pub kind: ProcessKind,
}
