use crate::contracts::TaskContract;
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
}
