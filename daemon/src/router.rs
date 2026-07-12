use core::contracts::TaskContract;
use core::events::{Event, RoutingTarget};
use crate::registry::AgentRegistry;

pub struct Router;

impl Router {
    pub fn route_task(contract: &TaskContract, registry: &AgentRegistry) -> Event {
        let target = if let Some(agent) = &contract.agent {
            RoutingTarget::Agent(agent.clone())
        } else if let Some(dept) = &contract.department {
            RoutingTarget::Department(dept.clone())
        } else {
            RoutingTarget::Any
        };

        match registry.get_available_agent(&target) {
            Some(agent_name) => Event::TaskRouted {
                task_id: contract.task_id.clone(),
                agent_name,
            },
            None => Event::TaskQueued {
                task_id: contract.task_id.clone(),
                target,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_to_agent() {
        let mut registry = AgentRegistry::new();
        registry.register_agent("codex".to_string(), "eng".to_string());

        let contract = TaskContract {
            task_id: "t-1".to_string(),
            department: None,
            agent: Some("codex".to_string()),
        };

        let event = Router::route_task(&contract, &registry);
        assert_eq!(event, Event::TaskRouted { task_id: "t-1".to_string(), agent_name: "codex".to_string() });
    }

    #[test]
    fn test_queue_when_busy() {
        let mut registry = AgentRegistry::new();
        registry.register_agent("codex".to_string(), "eng".to_string());
        registry.update_status("codex", core::events::AgentStatus::Busy);

        let contract = TaskContract {
            task_id: "t-1".to_string(),
            department: None,
            agent: Some("codex".to_string()),
        };

        let event = Router::route_task(&contract, &registry);
        assert_eq!(event, Event::TaskQueued { 
            task_id: "t-1".to_string(), 
            target: RoutingTarget::Agent("codex".to_string()) 
        });
    }

    #[test]
    fn test_route_to_department() {
        let mut registry = AgentRegistry::new();
        registry.register_agent("alice".to_string(), "support".to_string());

        let contract = TaskContract {
            task_id: "t-2".to_string(),
            department: Some("support".to_string()),
            agent: None,
        };

        let event = Router::route_task(&contract, &registry);
        assert_eq!(event, Event::TaskRouted { task_id: "t-2".to_string(), agent_name: "alice".to_string() });
    }
}
