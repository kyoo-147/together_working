use interprocess::local_socket::{prelude::*, GenericNamespaced, ListenerOptions};
use interprocess::TryClone;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::adapters::real::CommandProbe;
use crate::adapters::AgentProbe;
use crate::proposals::build_proposal;
use crate::registry::AgentRegistry;
use crate::router::Router;
use crate::store::EventStore;
use crate::supervisor::{send_pty_input, PtyInputMap};
use core::chat::{CommandProposal, ProposalAction, ProposalStatus};
use core::contracts::TaskContract;
use core::events::{AgentStatus, Event};
use core::ipc::{Command, Response};
use core::review::ReviewStatus;

pub fn start_server(
    socket_name: &str,
    store: Arc<Mutex<EventStore>>,
    registry: Arc<Mutex<AgentRegistry>>,
) -> std::io::Result<()> {
    let name = socket_name.to_ns_name::<GenericNamespaced>()?;
    let opts = ListenerOptions::new().name(name);
    let listener = opts.create_sync()?;

    let subscribers: Arc<Mutex<Vec<Sender<Event>>>> = Arc::new(Mutex::new(Vec::new()));
    let pty_inputs: PtyInputMap = Arc::new(Mutex::new(HashMap::new()));
    let proposals: Arc<Mutex<HashMap<String, CommandProposal>>> =
        Arc::new(Mutex::new(HashMap::new()));

    std::thread::spawn(move || {
        for mut conn in listener.incoming().filter_map(Result::ok) {
            let mut conn_clone = match conn.try_clone() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to clone connection: {}", e);
                    continue;
                }
            };

            let store = store.clone();
            let registry = registry.clone();
            let subscribers = subscribers.clone();
            let pty_inputs = pty_inputs.clone();
            let proposals = proposals.clone();

            std::thread::spawn(move || {
                let mut reader = BufReader::new(&mut conn_clone);
                let mut buffer = String::new();

                if let Ok(bytes) = reader.read_line(&mut buffer) {
                    if bytes == 0 {
                        return;
                    }

                    if let Ok(cmd) = serde_json::from_str::<Command>(&buffer) {
                        match cmd {
                            Command::CreateTask { yaml } => {
                                let resp = dispatch_task(
                                    &yaml,
                                    store.clone(),
                                    registry.clone(),
                                    subscribers.clone(),
                                    pty_inputs.clone(),
                                );
                                write_response(&mut conn, resp);
                            }
                            Command::SendInput { task_id, input } => {
                                let ok = send_pty_input(&pty_inputs, &task_id, input.clone());
                                if ok {
                                    let event = Event::PtyInput {
                                        task_id: task_id.clone(),
                                        input,
                                    };
                                    {
                                        let store_lock = store.lock().unwrap();
                                        let _ = store_lock.append(&event);
                                    }
                                    {
                                        let mut subs = subscribers.lock().unwrap();
                                        subs.retain(|sender| sender.send(event.clone()).is_ok());
                                    }
                                    let resp = Response::Ack { task_id };
                                    write_response(&mut conn, resp);
                                } else {
                                    let resp = Response::Error {
                                        message: format!("No active PTY for task {task_id}"),
                                    };
                                    write_response(&mut conn, resp);
                                }
                            }
                            Command::SubmitChat { source, text } => {
                                let chat_event = Event::ChatMessageReceived {
                                    source: source.clone(),
                                    text: text.clone(),
                                };
                                emit_event(&store, &subscribers, &chat_event);

                                let proposal = build_proposal(source, &text);
                                let proposal_id = proposal.proposal_id.clone();
                                proposals
                                    .lock()
                                    .unwrap()
                                    .insert(proposal_id.clone(), proposal.clone());
                                let event = Event::CommandProposalCreated { proposal };
                                emit_event(&store, &subscribers, &event);
                                write_response(&mut conn, Response::Proposal { proposal_id });
                            }
                            Command::ConfirmProposal { proposal_id } => {
                                let mut proposal = {
                                    let mut lock = proposals.lock().unwrap();
                                    lock.remove(&proposal_id)
                                };

                                if let Some(mut proposal) = proposal.take() {
                                    proposal.status = ProposalStatus::Confirmed;
                                    let event = Event::CommandProposalConfirmed {
                                        proposal_id: proposal_id.clone(),
                                    };
                                    emit_event(&store, &subscribers, &event);
                                    let response = match proposal.action {
                                        ProposalAction::CreateTask { yaml } => dispatch_task(
                                            &yaml,
                                            store.clone(),
                                            registry.clone(),
                                            subscribers.clone(),
                                            pty_inputs.clone(),
                                        ),
                                        ProposalAction::ApproveTask { task_id } => approve_task(
                                            &task_id,
                                            store.clone(),
                                            subscribers.clone(),
                                        ),
                                        ProposalAction::Status
                                        | ProposalAction::VerifyTask { .. }
                                        | ProposalAction::RerouteTask { .. } => Response::Ack {
                                            task_id: proposal_id,
                                        },
                                    };
                                    write_response(&mut conn, response);
                                } else {
                                    write_response(
                                        &mut conn,
                                        Response::Error {
                                            message: format!("Unknown proposal {proposal_id}"),
                                        },
                                    );
                                }
                            }
                            Command::RejectProposal { proposal_id } => {
                                let removed = proposals.lock().unwrap().remove(&proposal_id);
                                if removed.is_some() {
                                    let event = Event::CommandProposalRejected {
                                        proposal_id: proposal_id.clone(),
                                    };
                                    emit_event(&store, &subscribers, &event);
                                    write_response(
                                        &mut conn,
                                        Response::Ack {
                                            task_id: proposal_id,
                                        },
                                    );
                                } else {
                                    write_response(
                                        &mut conn,
                                        Response::Error {
                                            message: format!("Unknown proposal {proposal_id}"),
                                        },
                                    );
                                }
                            }
                            Command::UpdateSettings { settings } => {
                                let root = std::env::current_dir().unwrap_or_else(|_| ".".into());
                                match crate::settings::save_settings(&root, &settings) {
                                    Ok(settings) => {
                                        let event = Event::SettingsUpdated { settings };
                                        emit_event(&store, &subscribers, &event);
                                        write_response(
                                            &mut conn,
                                            Response::Ack {
                                                task_id: "settings".to_string(),
                                            },
                                        );
                                    }
                                    Err(e) => write_response(
                                        &mut conn,
                                        Response::Error {
                                            message: e.to_string(),
                                        },
                                    ),
                                }
                            }
                            Command::GetSettings => {
                                let root = std::env::current_dir().unwrap_or_else(|_| ".".into());
                                let settings = crate::settings::load_settings(&root);
                                write_response(&mut conn, Response::Settings { settings });
                            }
                            Command::GetStatus => {
                                write_response(
                                    &mut conn,
                                    Response::Status {
                                        json: status_json(&store),
                                    },
                                );
                            }
                            Command::RequestReview { task_id } => {
                                let event = Event::ReviewRequested {
                                    task_id: task_id.clone(),
                                };
                                emit_event(&store, &subscribers, &event);
                                write_response(&mut conn, Response::Ack { task_id });
                            }
                            Command::ApproveTask { task_id } => {
                                let response =
                                    approve_task(&task_id, store.clone(), subscribers.clone());
                                write_response(&mut conn, response);
                            }
                            Command::RejectTask { task_id, reason } => {
                                let event = Event::TaskRejected {
                                    task_id: task_id.clone(),
                                    reason,
                                };
                                emit_event(&store, &subscribers, &event);
                                write_response(&mut conn, Response::Ack { task_id });
                            }
                            Command::RequestChanges {
                                task_id,
                                instructions,
                            } => {
                                let event = Event::ReviewCompleted {
                                    task_id: task_id.clone(),
                                    status: ReviewStatus::ChangesRequested,
                                    summary: instructions,
                                };
                                emit_event(&store, &subscribers, &event);
                                write_response(&mut conn, Response::Ack { task_id });
                            }
                            Command::Sub => {
                                let (tx, rx) = channel::<Event>();
                                let events_to_send = {
                                    let store_lock = store.lock().unwrap();
                                    let mut subs = subscribers.lock().unwrap();
                                    subs.push(tx);
                                    store_lock.get_all().unwrap_or_default()
                                };

                                for event in events_to_send {
                                    let event_str = serde_json::to_string(&event).unwrap() + "\n";
                                    if conn.write_all(event_str.as_bytes()).is_err() {
                                        return;
                                    }
                                }

                                while let Ok(event) = rx.recv() {
                                    let event_str = serde_json::to_string(&event).unwrap() + "\n";
                                    if conn.write_all(event_str.as_bytes()).is_err() {
                                        break;
                                    }
                                }
                            }
                        }
                    } else {
                        let resp = Response::Error {
                            message: "Invalid command format".to_string(),
                        };
                        write_response(&mut conn, resp);
                    }
                }
            });
        }
    });
    Ok(())
}

fn dispatch_task(
    yaml: &str,
    store: Arc<Mutex<EventStore>>,
    registry: Arc<Mutex<AgentRegistry>>,
    subscribers: Arc<Mutex<Vec<Sender<Event>>>>,
    pty_inputs: PtyInputMap,
) -> Response {
    let mut contract = match TaskContract::from_yaml(yaml) {
        Ok(contract) => contract,
        Err(e) => {
            return Response::Error {
                message: e.to_string(),
            }
        }
    };

    if let Err(errors) = contract.validate_for_dispatch() {
        return Response::Error {
            message: errors.join("; "),
        };
    }

    let task_id = Uuid::new_v4().to_string();
    contract.task_id = task_id.clone();

    let event1 = Event::TaskCreated {
        task_id: task_id.clone(),
        contract: Box::new(contract.clone()),
    };

    let outcome = {
        let reg_lock = registry.lock().unwrap();
        Router::route_task_with_decision(&contract, &reg_lock)
    };
    let event2 = outcome.event;
    let route_decision_event = outcome
        .decision
        .as_ref()
        .map(|decision| Event::RouteDecision {
            task_id: task_id.clone(),
            agent_name: decision.agent_name.clone(),
            score: decision.score,
            reason: decision.reason.clone(),
        });

    emit_event(&store, &subscribers, &event1);
    emit_event(&store, &subscribers, &event2);
    if let Some(event) = &route_decision_event {
        emit_event(&store, &subscribers, event);
    }

    if let Event::TaskRouted {
        task_id,
        agent_name,
    } = &event2
    {
        crate::supervisor::ExecutionSupervisor::run_task(
            task_id.clone(),
            agent_name.clone(),
            contract,
            store.clone(),
            subscribers.clone(),
            pty_inputs.clone(),
        );
    }

    Response::Ack { task_id }
}

fn emit_event(
    store: &Arc<Mutex<EventStore>>,
    subscribers: &Arc<Mutex<Vec<Sender<Event>>>>,
    event: &Event,
) {
    {
        let store_lock = store.lock().unwrap();
        let _ = store_lock.append(event);
    }
    {
        let mut subs = subscribers.lock().unwrap();
        subs.retain(|sender| sender.send(event.clone()).is_ok());
    }
}

fn write_response<W: Write>(writer: &mut W, response: Response) {
    let resp_str = serde_json::to_string(&response).unwrap() + "\n";
    if let Err(e) = writer.write_all(resp_str.as_bytes()) {
        eprintln!("Failed to write response: {}", e);
    }
}

fn approve_task(
    task_id: &str,
    store: Arc<Mutex<EventStore>>,
    subscribers: Arc<Mutex<Vec<Sender<Event>>>>,
) -> Response {
    let events = store.lock().unwrap().get_all().unwrap_or_default();
    let completed = events.iter().rev().find_map(|event| match event {
        Event::TaskCompleted {
            task_id: id,
            success,
        } if id == task_id => Some(*success),
        _ => None,
    });
    let verification = events.iter().rev().find_map(|event| match event {
        Event::VerificationCompleted {
            task_id: id,
            success,
            summary,
        } if id == task_id => Some((*success, summary.clone())),
        _ => None,
    });

    let block_reason = match (completed, verification) {
        (Some(true), Some((true, _))) => None,
        (None, _) => Some("task not completed".to_string()),
        (Some(false), _) => Some("task completed with failure".to_string()),
        (Some(true), None) => Some("verification missing".to_string()),
        (Some(true), Some((false, summary))) => Some(format!("verification failed: {summary}")),
    };

    if let Some(reason) = block_reason {
        let event = Event::ApprovalBlocked {
            task_id: task_id.to_string(),
            reason: reason.clone(),
        };
        emit_event(&store, &subscribers, &event);
        Response::Error { message: reason }
    } else {
        let review = Event::ReviewCompleted {
            task_id: task_id.to_string(),
            status: ReviewStatus::Approved,
            summary: "approved".to_string(),
        };
        emit_event(&store, &subscribers, &review);
        let event = Event::TaskApproved {
            task_id: task_id.to_string(),
        };
        emit_event(&store, &subscribers, &event);
        Response::Ack {
            task_id: task_id.to_string(),
        }
    }
}

fn status_json(store: &Arc<Mutex<EventStore>>) -> String {
    let events = store.lock().unwrap().get_all().unwrap_or_default();
    let task_count = events
        .iter()
        .filter(|event| matches!(event, Event::TaskCreated { .. }))
        .count();
    let proposal_count = events
        .iter()
        .filter(|event| matches!(event, Event::CommandProposalCreated { .. }))
        .count();
    let attention_count = events
        .iter()
        .rev()
        .find_map(|event| match event {
            Event::NeedsAttentionChanged { items } => Some(items.len()),
            _ => None,
        })
        .unwrap_or(0);
    serde_json::json!({
        "daemon": "local",
        "tasks": task_count,
        "proposals": proposal_count,
        "needs_attention": attention_count
    })
    .to_string()
}

pub fn bootstrap_registry(registry: &mut AgentRegistry) -> Vec<Event> {
    let probe = CommandProbe;
    let agents = [
        ("codex", "integration"),
        ("cmdc", "engineering"),
        ("agy", "engineering"),
        ("claude", "review"),
    ];

    let mut events = Vec::new();
    for (agent, department) in agents {
        registry.register_agent(agent.to_string(), department.to_string());
        let status = probe.probe_command(agent);
        registry.update_status(agent, status.clone());
        events.push(Event::AgentStatusChanged {
            agent_name: agent.to_string(),
            status,
        });
    }
    let attention = needs_attention_from_events(&events);
    if !attention.is_empty() {
        events.push(Event::NeedsAttentionChanged { items: attention });
    }
    events
}

fn needs_attention_from_events(events: &[Event]) -> Vec<String> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::AgentStatusChanged {
                agent_name,
                status: AgentStatus::Degraded { reason },
            } => Some(format!("{agent_name} degraded: {reason}")),
            Event::VerificationCompleted {
                success: false,
                summary,
                ..
            } => Some(format!("verification failed: {summary}")),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::chat::ChatSource;
    use core::ipc::{Command, Response};
    use interprocess::local_socket::{GenericNamespaced, Stream};
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn test_ipc_communication() {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let socket_name = format!("together-test-{}-{}.sock", std::process::id(), id);

        let store = Arc::new(Mutex::new(EventStore::in_memory().unwrap()));
        let registry = Arc::new(Mutex::new(AgentRegistry::new()));

        start_server(&socket_name, store, registry).expect("Failed to start server");

        std::thread::sleep(std::time::Duration::from_millis(100));

        let name = socket_name.to_ns_name::<GenericNamespaced>().unwrap();
        let mut conn = Stream::connect(name).expect("Failed to connect to server");

        let cmd = Command::CreateTask {
            yaml: "
task_id: dummy
title: Server test task
department: HR
agent: Bob
scope:
  - src/**
allowed_files:
  - src/lib.rs
denied_files:
  - .env
success_criteria:
  - completes
enforcement_mode: strict
"
            .to_string(),
        };
        let cmd_str = serde_json::to_string(&cmd).unwrap() + "\n";
        conn.write_all(cmd_str.as_bytes()).unwrap();

        let mut reader = BufReader::new(&mut conn);
        let mut response_str = String::new();
        reader.read_line(&mut response_str).unwrap();

        let resp: Response = serde_json::from_str(&response_str).unwrap();
        match resp {
            Response::Ack { task_id } => {
                assert!(!task_id.is_empty());
            }
            _ => panic!("Unexpected response type: {:?}", resp),
        }
    }

    #[test]
    fn submit_chat_creates_proposal_without_creating_task() {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let socket_name = format!("together-chat-test-{}-{}.sock", std::process::id(), id);

        let store = Arc::new(Mutex::new(EventStore::in_memory().unwrap()));
        let registry = Arc::new(Mutex::new(AgentRegistry::new()));

        start_server(&socket_name, store.clone(), registry).expect("Failed to start server");

        std::thread::sleep(std::time::Duration::from_millis(100));

        let name = socket_name.to_ns_name::<GenericNamespaced>().unwrap();
        let mut conn = Stream::connect(name).expect("Failed to connect to server");

        let cmd = Command::SubmitChat {
            source: ChatSource::TogetherChat,
            text: "build a landing page".to_string(),
        };
        let cmd_str = serde_json::to_string(&cmd).unwrap() + "\n";
        conn.write_all(cmd_str.as_bytes()).unwrap();

        let mut reader = BufReader::new(&mut conn);
        let mut response_str = String::new();
        reader.read_line(&mut response_str).unwrap();

        let resp: Response = serde_json::from_str(&response_str).unwrap();
        assert!(matches!(resp, Response::Proposal { .. }));

        let events = store.lock().unwrap().get_all().unwrap();
        assert!(events
            .iter()
            .any(|event| matches!(event, Event::CommandProposalCreated { .. })));
        assert!(!events
            .iter()
            .any(|event| matches!(event, Event::TaskCreated { .. })));
    }

    #[test]
    fn approval_is_blocked_until_task_completed_and_verification_passed() {
        let store = Arc::new(Mutex::new(EventStore::in_memory().unwrap()));
        let subscribers: Arc<Mutex<Vec<Sender<Event>>>> = Arc::new(Mutex::new(Vec::new()));

        let response = approve_task("t1", store.clone(), subscribers.clone());

        assert!(matches!(response, Response::Error { .. }));
        let events = store.lock().unwrap().get_all().unwrap();
        assert!(events.iter().any(|event| {
            matches!(
                event,
                Event::ApprovalBlocked {
                    task_id,
                    reason
                } if task_id == "t1" && reason.contains("not completed")
            )
        }));

        emit_event(
            &store,
            &subscribers,
            &Event::TaskCompleted {
                task_id: "t1".to_string(),
                success: true,
            },
        );
        emit_event(
            &store,
            &subscribers,
            &Event::VerificationCompleted {
                task_id: "t1".to_string(),
                success: true,
                summary: "scope clean".to_string(),
            },
        );

        let response = approve_task("t1", store.clone(), subscribers.clone());

        assert!(matches!(response, Response::Ack { .. }));
        let events = store.lock().unwrap().get_all().unwrap();
        assert!(events
            .iter()
            .any(|event| matches!(event, Event::TaskApproved { task_id } if task_id == "t1")));
    }
}
