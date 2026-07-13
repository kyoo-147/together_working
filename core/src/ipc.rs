use crate::chat::ChatSource;
use crate::settings::UiSettings;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    Sub,
    CreateTask { yaml: String },
    SendInput { task_id: String, input: String },
    SubmitChat { source: ChatSource, text: String },
    ConfirmProposal { proposal_id: String },
    RejectProposal { proposal_id: String },
    UpdateSettings { settings: UiSettings },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    Ack { task_id: String },
    Proposal { proposal_id: String },
    Error { message: String },
}
