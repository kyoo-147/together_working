use crate::registry::{AgentRegistry, RouteDecision};
use core::contracts::TaskContract;
use core::events::{Event, RoutingTarget};

pub struct Router;

#[derive(Debug, Clone, PartialEq)]
pub struct RoutingOutcome {
    pub event: Event,
    pub decision: Option<RouteDecision>,
}

impl Router {
    pub fn route_task(contract: &TaskContract, registry: &AgentRegistry) -> Event {
        Self::route_task_with_decision(contract, registry).event
    }

    pub fn route_task_with_decision(
        contract: &TaskContract,
        registry: &AgentRegistry,
    ) -> RoutingOutcome {
        let target = if let Some(agent) = &contract.agent {
            RoutingTarget::Agent(agent.clone())
        } else if let Some(dept) = &contract.department {
            RoutingTarget::Department(dept.clone())
        } else {
            RoutingTarget::Any
        };

        match registry.rank_agent(&target) {
            Some(decision) => RoutingOutcome {
                event: Event::TaskRouted {
                    task_id: contract.task_id.clone(),
                    agent_name: decision.agent_name.clone(),
                },
                decision: Some(decision),
            },
            None => RoutingOutcome {
                event: Event::TaskQueued {
                    task_id: contract.task_id.clone(),
                    target,
                },
                decision: None,
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

        let mut contract = TaskContract::minimal("t-1");
        contract.agent = Some("codex".to_string());

        let event = Router::route_task(&contract, &registry);
        assert_eq!(
            event,
            Event::TaskRouted {
                task_id: "t-1".to_string(),
                agent_name: "codex".to_string()
            }
        );
    }

    #[test]
    fn test_queue_when_busy() {
        let mut registry = AgentRegistry::new();
        registry.register_agent("codex".to_string(), "eng".to_string());
        registry.update_status("codex", core::events::AgentStatus::Busy);

        let mut contract = TaskContract::minimal("t-1");
        contract.agent = Some("codex".to_string());

        let event = Router::route_task(&contract, &registry);
        assert_eq!(
            event,
            Event::TaskQueued {
                task_id: "t-1".to_string(),
                target: RoutingTarget::Agent("codex".to_string())
            }
        );
    }

    #[test]
    fn test_route_to_department() {
        let mut registry = AgentRegistry::new();
        registry.register_agent("alice".to_string(), "support".to_string());

        let mut contract = TaskContract::minimal("t-2");
        contract.department = Some("support".to_string());

        let event = Router::route_task(&contract, &registry);
        assert_eq!(
            event,
            Event::TaskRouted {
                task_id: "t-2".to_string(),
                agent_name: "alice".to_string()
            }
        );
    }
}
