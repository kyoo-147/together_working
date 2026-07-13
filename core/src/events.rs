use crate::chat::{ChatSource, CommandProposal};
use crate::contracts::TaskContract;
use crate::review::ReviewStatus;
use crate::settings::UiSettings;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RoutingTarget {
    Agent(String),
    Department(String),
    Any,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentStatus {
    Unknown,
    Ready,
    Busy,
    Blocked,
    Degraded { reason: String },
    Offline,
    Cooldown { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Event {
    ChatMessageReceived {
        source: ChatSource,
        text: String,
    },
    CommandProposalCreated {
        proposal: CommandProposal,
    },
    CommandProposalConfirmed {
        proposal_id: String,
    },
    CommandProposalRejected {
        proposal_id: String,
    },
    SettingsUpdated {
        settings: UiSettings,
    },
    NeedsAttentionChanged {
        items: Vec<String>,
    },
    TaskCreated {
        task_id: String,
        contract: Box<TaskContract>,
    },
    TaskQueued {
        task_id: String,
        target: RoutingTarget,
    },
    TaskRouted {
        task_id: String,
        agent_name: String,
    },
    AgentStatusChanged {
        agent_name: String,
        status: AgentStatus,
    },
    RouteDecision {
        task_id: String,
        agent_name: String,
        score: i32,
        reason: String,
    },
    PtyOutput {
        task_id: String,
        chunk: String,
    },
    PtyInput {
        task_id: String,
        input: String,
    },
    TaskCompleted {
        task_id: String,
        success: bool,
    },
    VerificationCompleted {
        task_id: String,
        success: bool,
        summary: String,
    },
    ReviewRequested {
        task_id: String,
    },
    ReviewCompleted {
        task_id: String,
        status: ReviewStatus,
        summary: String,
    },
    ApprovalBlocked {
        task_id: String,
        reason: String,
    },
    TaskApproved {
        task_id: String,
    },
    TaskRejected {
        task_id: String,
        reason: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_event_serialization() {
        let event = Event::TaskCreated {
            task_id: "t-1".to_string(),
            contract: Box::new(TaskContract {
                task_id: "t-1".to_string(),
                title: Some("Test task".to_string()),
                intent: None,
                department: None,
                agent: None,
                scope: vec!["src/**".to_string()],
                allowed_files: vec!["src/lib.rs".to_string()],
                denied_files: vec![".env".to_string()],
                deliverables: vec![],
                success_criteria: vec!["tests pass".to_string()],
                reviewer_required: false,
                verification_required: true,
                merge_authority: Some("codex".to_string()),
                enforcement_mode: crate::contracts::EnforcementMode::Strict,
                unknown_files_policy: crate::contracts::UnknownFilesPolicy::NeedsReview,
            }),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("TaskCreated"));
    }

    #[test]
    fn serializes_review_and_approval_events() {
        let events = [
            Event::ReviewRequested {
                task_id: "t1".to_string(),
            },
            Event::ReviewCompleted {
                task_id: "t1".to_string(),
                status: crate::review::ReviewStatus::ChangesRequested,
                summary: "needs tests".to_string(),
            },
            Event::ApprovalBlocked {
                task_id: "t1".to_string(),
                reason: "verification failed".to_string(),
            },
            Event::TaskApproved {
                task_id: "t1".to_string(),
            },
            Event::TaskRejected {
                task_id: "t1".to_string(),
                reason: "out of scope".to_string(),
            },
        ];

        for event in events {
            let json = serde_json::to_string(&event).unwrap();
            let parsed: Event = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, event);
        }
    }
}
