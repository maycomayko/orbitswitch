//! Legacy account-registry tags, kept solely for backward-compatible serialization.
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub enum Provider {
    #[default]
    Antigravity,
    Codex,
    Claude,
}
