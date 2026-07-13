use core::chat::{ChatMessage, CommandProposal, ProposalStatus};
use core::contracts::TaskContract;
use core::events::{AgentStatus, Event, RoutingTarget};
use core::settings::UiSettings;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Draft,
    Queued(RoutingTarget),
    Routed(String),
    Completed(bool),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TaskDetail {
    pub task_id: String,
    pub title: Option<String>,
    pub department: Option<String>,
    pub scope_summary: String,
    pub assigned_agent: Option<String>,
    pub route: Option<String>,
    pub verification: Option<String>,
    pub approval_blocked: bool,
}

#[derive(Default)]
pub struct TuiState {
    pub agents: HashMap<String, AgentStatus>,
    pub tasks: HashMap<String, TaskStatus>,
    pub task_details: HashMap<String, TaskDetail>,
    pub logs: HashMap<String, Vec<String>>,
    pub route_decisions: HashMap<String, String>,
    pub timelines: HashMap<String, Vec<String>>,
    pub chat_messages: Vec<ChatMessage>,
    pub proposals: HashMap<String, CommandProposal>,
    pub latest_proposal_id: Option<String>,
    pub needs_attention: Vec<String>,
    pub settings: UiSettings,
    pub selected_task_id: Option<String>,
}

impl TuiState {
    pub fn process_event(&mut self, event: Event) {
        match event {
            Event::ChatMessageReceived { source, text } => {
                self.chat_messages.push(ChatMessage { source, text });
            }
            Event::CommandProposalCreated { proposal } => {
                self.latest_proposal_id = Some(proposal.proposal_id.clone());
                self.proposals
                    .insert(proposal.proposal_id.clone(), proposal);
            }
            Event::CommandProposalConfirmed { proposal_id } => {
                if let Some(proposal) = self.proposals.get_mut(&proposal_id) {
                    proposal.status = ProposalStatus::Confirmed;
                }
            }
            Event::CommandProposalRejected { proposal_id } => {
                if let Some(proposal) = self.proposals.get_mut(&proposal_id) {
                    proposal.status = ProposalStatus::Rejected;
                }
            }
            Event::SettingsUpdated { settings } => {
                self.settings = settings;
            }
            Event::NeedsAttentionChanged { items } => {
                self.needs_attention = items;
            }
            Event::AgentStatusChanged { agent_name, status } => {
                if let AgentStatus::Degraded { reason } = &status {
                    self.push_attention(format!("{agent_name} degraded: {reason}"));
                }
                self.agents.insert(agent_name, status);
            }
            Event::TaskCreated { task_id, contract } => {
                self.upsert_contract_detail(&task_id, &contract);
                self.tasks
                    .entry(task_id.clone())
                    .or_insert(TaskStatus::Draft);
                self.timeline(&task_id, "contract created");
                self.selected_task_id.get_or_insert(task_id);
            }
            Event::TaskQueued { task_id, target } => {
                self.tasks.insert(task_id, TaskStatus::Queued(target));
            }
            Event::TaskRouted {
                task_id,
                agent_name,
            } => {
                self.tasks
                    .insert(task_id.clone(), TaskStatus::Routed(agent_name.clone()));
                self.task_details
                    .entry(task_id.clone())
                    .or_insert_with(|| TaskDetail {
                        task_id: task_id.clone(),
                        ..TaskDetail::default()
                    })
                    .assigned_agent = Some(agent_name.clone());
                self.timeline(&task_id, &format!("routed to {agent_name}"));
            }
            Event::PtyOutput { task_id, chunk } => {
                for line in split_log_chunk(&chunk) {
                    self.logs.entry(task_id.clone()).or_default().push(line);
                }
            }
            Event::PtyInput { task_id, input } => {
                self.logs
                    .entry(task_id)
                    .or_default()
                    .push(format!("> {input}"));
            }
            Event::TaskCompleted { task_id, success } => {
                self.tasks
                    .insert(task_id.clone(), TaskStatus::Completed(success));
                self.timeline(
                    &task_id,
                    if success {
                        "completed successfully"
                    } else {
                        "completed with failure"
                    },
                );
            }
            Event::RouteDecision {
                task_id,
                agent_name,
                score,
                reason,
            } => {
                let route = format!("{agent_name} score={score}: {reason}");
                self.route_decisions.insert(task_id.clone(), route.clone());
                self.task_details
                    .entry(task_id.clone())
                    .or_insert_with(|| TaskDetail {
                        task_id: task_id.clone(),
                        ..TaskDetail::default()
                    })
                    .route = Some(route);
                self.timeline(&task_id, &format!("route {agent_name} score={score}"));
            }
            Event::VerificationCompleted {
                task_id,
                success,
                summary,
            } => {
                self.task_details
                    .entry(task_id.clone())
                    .or_insert_with(|| TaskDetail {
                        task_id: task_id.clone(),
                        ..TaskDetail::default()
                    })
                    .verification = Some(summary.clone());
                if let Some(detail) = self.task_details.get_mut(&task_id) {
                    detail.approval_blocked = !success;
                }
                if !success {
                    self.push_attention(format!("verification failed: {summary}"));
                }
                self.logs
                    .entry(task_id.clone())
                    .or_default()
                    .push(format!("[verification success={success}] {summary}"));
                self.timeline(
                    &task_id,
                    if success {
                        "verification passed"
                    } else {
                        "verification blocked approval"
                    },
                );
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

    pub fn task_detail(&self, task_id: &str) -> Option<&TaskDetail> {
        self.task_details.get(task_id)
    }

    pub fn selected_task_detail(&self) -> Option<&TaskDetail> {
        self.selected_task_id
            .as_deref()
            .and_then(|task_id| self.task_detail(task_id))
    }

    pub fn timeline_for(&self, task_id: &str) -> Vec<String> {
        self.timelines.get(task_id).cloned().unwrap_or_default()
    }

    pub fn latest_pending_proposal(&self) -> Option<&CommandProposal> {
        self.latest_proposal_id
            .as_deref()
            .and_then(|id| self.proposals.get(id))
            .filter(|proposal| proposal.status == ProposalStatus::Pending)
    }

    fn upsert_contract_detail(&mut self, task_id: &str, contract: &TaskContract) {
        self.task_details.insert(
            task_id.to_string(),
            TaskDetail {
                task_id: task_id.to_string(),
                title: contract.title.clone().or_else(|| contract.intent.clone()),
                department: contract.department.clone(),
                scope_summary: if contract.scope.is_empty() {
                    "unscoped".to_string()
                } else {
                    contract.scope.join(", ")
                },
                assigned_agent: contract.agent.clone(),
                route: None,
                verification: None,
                approval_blocked: false,
            },
        );
    }

    fn timeline(&mut self, task_id: &str, entry: &str) {
        self.timelines
            .entry(task_id.to_string())
            .or_default()
            .push(entry.to_string());
    }

    fn push_attention(&mut self, item: String) {
        if !self
            .needs_attention
            .iter()
            .any(|existing| existing == &item)
        {
            self.needs_attention.push(item);
        }
    }
}

fn split_log_chunk(chunk: &str) -> Vec<String> {
    let lines = chunk
        .lines()
        .map(ToString::to_string)
        .collect::<Vec<String>>();
    if lines.is_empty() {
        vec![chunk.to_string()]
    } else {
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::chat::{ChatSource, CommandProposal, ProposalAction};

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

    #[test]
    fn test_task_metadata_timeline_and_blocked_review_state() {
        let mut state = TuiState::default();
        let mut contract = core::contracts::TaskContract::minimal("t1");
        contract.title = Some("Improve cockpit".into());
        contract.department = Some("engineering".into());
        contract.scope = vec!["cli/src/**".into()];

        state.process_event(Event::TaskCreated {
            task_id: "t1".into(),
            contract: Box::new(contract),
        });
        state.process_event(Event::RouteDecision {
            task_id: "t1".into(),
            agent_name: "cmdc".into(),
            score: 182,
            reason: "codex degraded; cmdc ready".into(),
        });
        state.process_event(Event::VerificationCompleted {
            task_id: "t1".into(),
            success: false,
            summary: "blocked: changed .env".into(),
        });

        let task = state.task_detail("t1").expect("task detail should exist");

        assert_eq!(task.title.as_deref(), Some("Improve cockpit"));
        assert_eq!(task.department.as_deref(), Some("engineering"));
        assert_eq!(task.scope_summary, "cli/src/**");
        assert_eq!(task.verification.as_deref(), Some("blocked: changed .env"));
        assert!(task.approval_blocked);
        assert!(state
            .timeline_for("t1")
            .iter()
            .any(|entry| entry.contains("route cmdc score=182")));
    }

    #[test]
    fn test_chat_proposal_does_not_create_task_before_confirmation() {
        let mut state = TuiState::default();
        let proposal = CommandProposal {
            proposal_id: "p1".into(),
            source: ChatSource::TogetherChat,
            title: "Create task".into(),
            summary: "preview only".into(),
            action: ProposalAction::CreateTask {
                yaml: "task_id: draft".into(),
            },
            preview: None,
            status: ProposalStatus::Pending,
        };

        state.process_event(Event::ChatMessageReceived {
            source: ChatSource::TogetherChat,
            text: "build a landing page".into(),
        });
        state.process_event(Event::CommandProposalCreated { proposal });

        assert_eq!(state.chat_messages.len(), 1);
        assert_eq!(state.latest_pending_proposal().unwrap().proposal_id, "p1");
        assert!(state.tasks.is_empty());

        state.process_event(Event::CommandProposalRejected {
            proposal_id: "p1".into(),
        });
        assert!(state.latest_pending_proposal().is_none());
        assert!(state.tasks.is_empty());
    }

    #[test]
    fn test_settings_and_needs_attention_events_update_state() {
        let mut state = TuiState::default();

        state.process_event(Event::SettingsUpdated {
            settings: core::settings::UiSettings {
                theme_preset: "Ocean Blue".into(),
                custom_bg: None,
                custom_main: None,
            },
        });
        state.process_event(Event::NeedsAttentionChanged {
            items: vec!["codex degraded: Access is denied".into()],
        });

        assert_eq!(state.settings.theme_preset, "Ocean Blue");
        assert_eq!(state.needs_attention.len(), 1);
    }
}
