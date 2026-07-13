use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    Sub,
    CreateTask { yaml: String },
    SendInput { task_id: String, input: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    Ack { task_id: String },
    Error { message: String },
}
