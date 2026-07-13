use crate::chat::ChatSource;
use crate::settings::UiSettings;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    Sub,
    CreateTask {
        yaml: String,
    },
    SendInput {
        task_id: String,
        input: String,
    },
    SubmitChat {
        source: ChatSource,
        text: String,
    },
    ConfirmProposal {
        proposal_id: String,
    },
    RejectProposal {
        proposal_id: String,
    },
    UpdateSettings {
        settings: UiSettings,
    },
    GetSettings,
    GetStatus,
    RequestReview {
        task_id: String,
    },
    ApproveTask {
        task_id: String,
    },
    RejectTask {
        task_id: String,
        reason: String,
    },
    RequestChanges {
        task_id: String,
        instructions: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    Ack { task_id: String },
    Proposal { proposal_id: String },
    Settings { settings: UiSettings },
    Status { json: String },
    Error { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat::ChatSource;

    #[test]
    fn serializes_production_completion_commands() {
        let commands = [
            Command::SubmitChat {
                source: ChatSource::CodexApp,
                text: "create a landing page".to_string(),
            },
            Command::ConfirmProposal {
                proposal_id: "p1".to_string(),
            },
            Command::RejectProposal {
                proposal_id: "p1".to_string(),
            },
            Command::GetStatus,
            Command::GetSettings,
            Command::ApproveTask {
                task_id: "t1".to_string(),
            },
            Command::RejectTask {
                task_id: "t1".to_string(),
                reason: "scope failed".to_string(),
            },
            Command::RequestChanges {
                task_id: "t1".to_string(),
                instructions: "tighten tests".to_string(),
            },
            Command::RequestReview {
                task_id: "t1".to_string(),
            },
        ];

        for command in commands {
            let json = serde_json::to_string(&command).unwrap();
            let parsed: Command = serde_json::from_str(&json).unwrap();
            assert_eq!(format!("{parsed:?}"), format!("{command:?}"));
        }
    }
}
