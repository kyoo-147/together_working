use std::collections::BTreeMap;
use core::events::{RoutingTarget, AgentStatus};

#[derive(Debug, Clone, PartialEq)]
pub struct AgentInfo {
    pub status: AgentStatus,
    pub department: String,
}

pub struct AgentRegistry {
    agents: BTreeMap<String, AgentInfo>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self { agents: BTreeMap::new() }
    }

    pub fn register_agent(&mut self, name: String, department: String) {
        self.agents.insert(name, AgentInfo {
            status: AgentStatus::Ready,
            department,
        });
    }

    pub fn update_status(&mut self, name: &str, status: AgentStatus) {
        if let Some(info) = self.agents.get_mut(name) {
            info.status = status;
        }
    }

    pub fn get_available_agent(&self, target: &RoutingTarget) -> Option<String> {
        match target {
            RoutingTarget::Agent(name) => {
                if let Some(info) = self.agents.get(name) {
                    if info.status == AgentStatus::Ready {
                        return Some(name.clone());
                    }
                }
                None
            }
            RoutingTarget::Department(dept) => {
                for (name, info) in &self.agents {
                    if info.department == *dept && info.status == AgentStatus::Ready {
                        return Some(name.clone());
                    }
                }
                None
            }
            RoutingTarget::Any => {
                for (name, info) in &self.agents {
                    if info.status == AgentStatus::Ready {
                        return Some(name.clone());
                    }
                }
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_get_agent() {
        let mut registry = AgentRegistry::new();
        registry.register_agent("agent1".to_string(), "dept1".to_string());
        
        let target = RoutingTarget::Agent("agent1".to_string());
        assert_eq!(registry.get_available_agent(&target), Some("agent1".to_string()));
    }

    #[test]
    fn test_get_agent_busy() {
        let mut registry = AgentRegistry::new();
        registry.register_agent("agent1".to_string(), "dept1".to_string());
        registry.update_status("agent1", AgentStatus::Busy);
        
        let target = RoutingTarget::Agent("agent1".to_string());
        assert_eq!(registry.get_available_agent(&target), None);
    }

    #[test]
    fn test_department_routing() {
        let mut registry = AgentRegistry::new();
        registry.register_agent("agent1".to_string(), "dept1".to_string());
        registry.register_agent("agent2".to_string(), "dept2".to_string());
        
        let target = RoutingTarget::Department("dept2".to_string());
        assert_eq!(registry.get_available_agent(&target), Some("agent2".to_string()));
    }

    #[test]
    fn test_any_routing() {
        let mut registry = AgentRegistry::new();
        registry.register_agent("agent1".to_string(), "dept1".to_string());
        registry.update_status("agent1", AgentStatus::Offline);
        registry.register_agent("agent2".to_string(), "dept2".to_string());
        
        let target = RoutingTarget::Any;
        assert_eq!(registry.get_available_agent(&target), Some("agent2".to_string()));
    }
}
