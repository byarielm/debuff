use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Moderator types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub(super) struct AddModeratorBody {
    pub(super) did: String,
}

#[derive(Serialize)]
pub(super) struct ModeratorSummary {
    pub(super) did: String,
    pub(super) role: String,
    pub(super) created_at: DateTime<Utc>,
    pub(super) last_used_at: Option<DateTime<Utc>>,
}
