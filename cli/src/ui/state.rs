use std::collections::HashMap;
use core::events::{AgentStatus, Event, RoutingTarget};

#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Queued(RoutingTarget),
    Routed(String),
    Completed(bool),
}

#[derive(Default)]
pub struct TuiState {
    pub agents: HashMap<String, AgentStatus>,
    pub tasks: HashMap<String, TaskStatus>,
    pub logs: HashMap<String, Vec<String>>,
    pub selected_task_id: Option<String>,
}

impl TuiState {
    pub fn process_event(&mut self, event: Event) {
        match event {
            Event::AgentStatusChanged { agent_name, status } => {
                self.agents.insert(agent_name, status);
            }
            Event::TaskQueued { task_id, target } => {
                self.tasks.insert(task_id, TaskStatus::Queued(target));
            }
            Event::TaskRouted { task_id, agent_name } => {
                self.tasks.insert(task_id, TaskStatus::Routed(agent_name));
            }
            Event::PtyOutput { task_id, chunk } => {
                self.logs.entry(task_id).or_default().push(chunk);
            }
            Event::TaskCompleted { task_id, success } => {
                self.tasks.insert(task_id, TaskStatus::Completed(success));
            }
            Event::TaskCreated { .. } => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_event_task_lifecycle() {
        let mut state = TuiState::default();
        
        state.process_event(Event::TaskQueued { task_id: "t1".into(), target: RoutingTarget::Any });
        assert_eq!(state.tasks.get("t1"), Some(&TaskStatus::Queued(RoutingTarget::Any)));
        
        state.process_event(Event::TaskRouted { task_id: "t1".into(), agent_name: "a1".into() });
        assert_eq!(state.tasks.get("t1"), Some(&TaskStatus::Routed("a1".into())));

        state.process_event(Event::PtyOutput { task_id: "t1".into(), chunk: "log".into() });
        assert_eq!(state.logs.get("t1").unwrap().len(), 1);

        state.process_event(Event::TaskCompleted { task_id: "t1".into(), success: true });
        assert_eq!(state.tasks.get("t1"), Some(&TaskStatus::Completed(true)));
    }
    
    #[test]
    fn test_process_event_agent_status() {
        let mut state = TuiState::default();
        
        state.process_event(Event::AgentStatusChanged { agent_name: "a1".into(), status: AgentStatus::Ready });
        assert_eq!(state.agents.get("a1"), Some(&AgentStatus::Ready));
    }
}
