use crate::models::{Agent, ReadinessState};

pub fn scan_agents() -> Vec<Agent> {
    vec![
        Agent { name: "codex".to_string(), state: ReadinessState::Ready },
        Agent { name: "claude".to_string(), state: ReadinessState::Idle },
    ]
}
