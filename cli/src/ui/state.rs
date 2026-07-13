use core::events::{AgentStatus, Event, RoutingTarget};
use std::collections::HashMap;

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
    pub route_decisions: HashMap<String, String>,
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
            Event::TaskRouted {
                task_id,
                agent_name,
            } => {
                self.tasks.insert(task_id, TaskStatus::Routed(agent_name));
            }
            Event::PtyOutput { task_id, chunk } => {
                self.logs.entry(task_id).or_default().push(chunk);
            }
            Event::PtyInput { task_id, input } => {
                self.logs
                    .entry(task_id)
                    .or_default()
                    .push(format!("> {input}"));
            }
            Event::TaskCompleted { task_id, success } => {
                self.tasks.insert(task_id, TaskStatus::Completed(success));
            }
            Event::RouteDecision {
                task_id,
                agent_name,
                score,
                reason,
            } => {
                self.route_decisions
                    .insert(task_id, format!("{agent_name} score={score}: {reason}"));
            }
            Event::VerificationCompleted {
                task_id,
                success,
                summary,
            } => {
                self.logs
                    .entry(task_id)
                    .or_default()
                    .push(format!("[verification success={success}] {summary}"));
            }
            Event::TaskCreated { task_id, .. } => {
                self.selected_task_id.get_or_insert(task_id);
            }
        }
    }

    pub fn select_next_task(&mut self) {
        let mut ids = self.tasks.keys().cloned().collect::<Vec<_>>();
        ids.sort();
        if ids.is_empty() {
            return;
        }
        let next = self
            .selected_task_id
            .as_ref()
            .and_then(|selected| ids.iter().position(|id| id == selected))
            .map(|index| (index + 1) % ids.len())
            .unwrap_or(0);
        self.selected_task_id = Some(ids[next].clone());
    }

    pub fn select_previous_task(&mut self) {
        let mut ids = self.tasks.keys().cloned().collect::<Vec<_>>();
        ids.sort();
        if ids.is_empty() {
            return;
        }
        let previous = self
            .selected_task_id
            .as_ref()
            .and_then(|selected| ids.iter().position(|id| id == selected))
            .map(|index| if index == 0 { ids.len() - 1 } else { index - 1 })
            .unwrap_or(0);
        self.selected_task_id = Some(ids[previous].clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_event_task_lifecycle() {
        let mut state = TuiState::default();

        state.process_event(Event::TaskQueued {
            task_id: "t1".into(),
            target: RoutingTarget::Any,
        });
        assert_eq!(
            state.tasks.get("t1"),
            Some(&TaskStatus::Queued(RoutingTarget::Any))
        );

        state.process_event(Event::TaskRouted {
            task_id: "t1".into(),
            agent_name: "a1".into(),
        });
        assert_eq!(
            state.tasks.get("t1"),
            Some(&TaskStatus::Routed("a1".into()))
        );

        state.process_event(Event::PtyOutput {
            task_id: "t1".into(),
            chunk: "log".into(),
        });
        assert_eq!(state.logs.get("t1").unwrap().len(), 1);

        state.process_event(Event::TaskCompleted {
            task_id: "t1".into(),
            success: true,
        });
        assert_eq!(state.tasks.get("t1"), Some(&TaskStatus::Completed(true)));
    }

    #[test]
    fn test_process_event_agent_status() {
        let mut state = TuiState::default();

        state.process_event(Event::AgentStatusChanged {
            agent_name: "a1".into(),
            status: AgentStatus::Ready,
        });
        assert_eq!(state.agents.get("a1"), Some(&AgentStatus::Ready));
    }

    #[test]
    fn test_process_event_route_decision_and_input() {
        let mut state = TuiState::default();

        state.process_event(Event::RouteDecision {
            task_id: "t1".into(),
            agent_name: "cmdc".into(),
            score: 180,
            reason: "codex degraded".into(),
        });
        state.process_event(Event::PtyInput {
            task_id: "t1".into(),
            input: "continue\r\n".into(),
        });

        assert!(state
            .route_decisions
            .get("t1")
            .unwrap()
            .contains("cmdc score=180"));
        assert_eq!(state.logs.get("t1").unwrap()[0], "> continue\r\n");
    }

    #[test]
    fn test_task_selection_moves_between_sorted_tasks() {
        let mut state = TuiState::default();
        state.process_event(Event::TaskQueued {
            task_id: "b".into(),
            target: RoutingTarget::Any,
        });
        state.process_event(Event::TaskQueued {
            task_id: "a".into(),
            target: RoutingTarget::Any,
        });

        state.select_next_task();
        assert_eq!(state.selected_task_id.as_deref(), Some("a"));
        state.select_next_task();
        assert_eq!(state.selected_task_id.as_deref(), Some("b"));
        state.select_previous_task();
        assert_eq!(state.selected_task_id.as_deref(), Some("a"));
    }
}
