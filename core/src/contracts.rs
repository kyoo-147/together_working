use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskContract {
    pub task_id: String,
    pub department: Option<String>,
    pub agent: Option<String>,
}

impl TaskContract {
    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_contract() {
        let yaml = "
task_id: test-1
department: engineering
agent: codex
";
        let contract = TaskContract::from_yaml(yaml).unwrap();
        assert_eq!(contract.task_id, "test-1");
        assert_eq!(contract.department, Some("engineering".to_string()));
        assert_eq!(contract.agent, Some("codex".to_string()));
    }
}
