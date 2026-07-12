use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    Sub,
    CreateTask { yaml: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    Ack { task_id: String },
    Error { message: String },
}
