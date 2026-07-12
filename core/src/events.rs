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
    Ready,
    Busy,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Event {
    TaskCreated { task_id: String, contract: TaskContract },
    TaskQueued { task_id: String, target: RoutingTarget },
    TaskRouted { task_id: String, agent_name: String },
    AgentStatusChanged { agent_name: String, status: AgentStatus },
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_event_serialization() {
        let event = Event::TaskCreated { 
            task_id: "t-1".to_string(), 
            contract: TaskContract {
                task_id: "t-1".to_string(),
                department: None,
                agent: None,
            }
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("TaskCreated"));
    }
}
