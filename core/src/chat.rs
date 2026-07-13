use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChatSource {
    CodexApp,
    TogetherChat,
    CliYaml,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatMessage {
    pub source: ChatSource,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProposalStatus {
    Pending,
    Confirmed,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProposalAction {
    CreateTask { yaml: String },
    VerifyTask { task_id: String },
    RerouteTask { task_id: String },
    ApproveTask { task_id: String },
    Status,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandProposal {
    pub proposal_id: String,
    pub source: ChatSource,
    pub title: String,
    pub summary: String,
    pub action: ProposalAction,
    pub preview: Option<String>,
    pub status: ProposalStatus,
}
