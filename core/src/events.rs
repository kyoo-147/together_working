use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Event {
    TaskCreated { task_id: String, contract_path: String },
    TaskRouted { task_id: String, agent_name: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_event_serialization() {
        let event = Event::TaskCreated { task_id: "t-1".to_string(), contract_path: "path".to_string() };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("TaskCreated"));
    }
}
