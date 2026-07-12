use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum ReadinessState {
    Ready,
    Idle,
    Offline,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Agent {
    pub name: String,
    pub state: ReadinessState,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Department {
    pub name: String,
}
