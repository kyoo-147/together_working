use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReviewStatus {
    PendingReview,
    ChangesRequested,
    Approved,
    Rejected,
    BlockedByVerification,
}

impl ReviewStatus {
    pub fn label(&self) -> &'static str {
        match self {
            ReviewStatus::PendingReview => "pending_review",
            ReviewStatus::ChangesRequested => "changes_requested",
            ReviewStatus::Approved => "approved",
            ReviewStatus::Rejected => "rejected",
            ReviewStatus::BlockedByVerification => "blocked_by_verification",
        }
    }
}
