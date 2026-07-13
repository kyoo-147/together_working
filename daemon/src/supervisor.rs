use crate::adapters::real::RealAgentAdapter;
use crate::adapters::AgentAdapter;
use crate::pty::PtyManager;
use crate::scope::{verify_changed_files, ScopeDecision};
use crate::store::EventStore;
use core::contracts::TaskContract;
use core::events::Event;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;

pub type PtyInputMap = Arc<Mutex<HashMap<String, Sender<String>>>>;

pub struct ExecutionSupervisor;

impl ExecutionSupervisor {
    pub fn run_task(
        task_id: String,
        agent_name: String,
        contract: TaskContract,
        store: Arc<Mutex<EventStore>>,
        subscribers: Arc<Mutex<Vec<Sender<Event>>>>,
        inputs: PtyInputMap,
    ) {
        thread::spawn(move || {
            let adapter = RealAgentAdapter { agent_name };
            let mut command = adapter.build_command(&contract);
            let worktree = match prepare_task_worktree(&task_id) {
                Ok(worktree) => {
                    command.current_dir(&worktree);
                    worktree
                }
                Err(error) => {
                    eprintln!("Supervisor failed to prepare worktree: {}", error);
                    let event = Event::TaskCompleted {
                        task_id: task_id.clone(),
                        success: false,
                    };
                    Self::broadcast_event(&event, &store, &subscribers);
                    return;
                }
            };
            Self::run_command(
                task_id,
                command,
                Some((contract, worktree)),
                store,
                subscribers,
                inputs,
            );
        });
    }

    fn run_command(
        task_id: String,
        command: Command,
        verification: Option<(TaskContract, PathBuf)>,
        store: Arc<Mutex<EventStore>>,
        subscribers: Arc<Mutex<Vec<Sender<Event>>>>,
        inputs: PtyInputMap,
    ) {
        let pty_process = match PtyManager::spawn(command) {
            Ok(proc) => proc,
            Err(e) => {
                eprintln!("Supervisor failed to spawn PTY: {}", e);
                let event = Event::TaskCompleted {
                    task_id: task_id.clone(),
                    success: false,
                };
                Self::broadcast_event(&event, &store, &subscribers);
                return;
            }
        };

        let master = pty_process.master;
        let mut child = pty_process.child;
        let mut reader = pty_process.reader;
        let mut writer = pty_process.writer;
        let (tx_input, rx_input) = std::sync::mpsc::channel::<String>();

        if let Ok(mut inputs_lock) = inputs.lock() {
            inputs_lock.insert(task_id.clone(), tx_input);
        }

        let (tx_exit, rx_exit) = std::sync::mpsc::channel();
        let task_id_for_input = task_id.clone();
        let inputs_for_cleanup = inputs.clone();
        thread::spawn(move || {
            while let Ok(input) = rx_input.recv() {
                if writer.write_all(input.as_bytes()).is_err() {
                    break;
                }
                let _ = writer.flush();
            }
            if let Ok(mut inputs_lock) = inputs_for_cleanup.lock() {
                inputs_lock.remove(&task_id_for_input);
            }
        });

        thread::spawn(move || {
            let success = match child.wait() {
                Ok(status) => status.success(),
                Err(_) => false,
            };
            let _ = tx_exit.send(success);
            drop(master);
        });

        let mut buf = [0u8; 1024];
        loop {
            match reader.read(&mut buf) {
                Ok(n) if n > 0 => {
                    let chunk = String::from_utf8_lossy(&buf[..n]).to_string();
                    let event = Event::PtyOutput {
                        task_id: task_id.clone(),
                        chunk,
                    };
                    Self::broadcast_event(&event, &store, &subscribers);
                }
                _ => break,
            }
        }

        let success = rx_exit.recv().unwrap_or(false);

        let event = Event::TaskCompleted {
            task_id: task_id.clone(),
            success,
        };
        Self::broadcast_event(&event, &store, &subscribers);

        if let Some((contract, worktree)) = verification {
            let (verification_success, summary) = verify_worktree_diff(&contract, &worktree);
            let event = Event::VerificationCompleted {
                task_id,
                success: success && verification_success,
                summary,
            };
            Self::broadcast_event(&event, &store, &subscribers);
        }
    }

    fn broadcast_event(
        event: &Event,
        store: &Arc<Mutex<EventStore>>,
        subscribers: &Arc<Mutex<Vec<Sender<Event>>>>,
    ) {
        if let Ok(store_lock) = store.lock() {
            let _ = store_lock.append(event);
        }
        if let Ok(mut subs_lock) = subscribers.lock() {
            subs_lock.retain(|sender| sender.send(event.clone()).is_ok());
        }
    }
}

fn verify_worktree_diff(contract: &TaskContract, worktree: &Path) -> (bool, String) {
    let output = std::process::Command::new("git")
        .args(["diff", "--name-only"])
        .current_dir(worktree)
        .output();

    let changed_files = match output {
        Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>(),
        Ok(output) => {
            return (false, format!("git diff failed with {}", output.status));
        }
        Err(error) => return (false, format!("git diff failed: {error}")),
    };

    match verify_changed_files(contract, &changed_files) {
        ScopeDecision::Pass => (true, "scope verification passed".to_string()),
        ScopeDecision::NeedsReview { files } => (
            false,
            format!("scope verification needs review: {}", files.join(", ")),
        ),
        ScopeDecision::Reject { files } => (
            false,
            format!("scope verification rejected: {}", files.join(", ")),
        ),
    }
}

pub fn send_pty_input(inputs: &PtyInputMap, task_id: &str, input: String) -> bool {
    inputs
        .lock()
        .ok()
        .and_then(|inputs_lock| inputs_lock.get(task_id).cloned())
        .map(|sender| sender.send(input).is_ok())
        .unwrap_or(false)
}

fn prepare_task_worktree(
    task_id: &str,
) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let repo_root = std::env::current_dir()?;
    prepare_task_worktree_at(&repo_root, task_id)
}

fn prepare_task_worktree_at(
    repo_root: &Path,
    task_id: &str,
) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let safe_task_id = task_id
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>();
    let worktree_dir = repo_root
        .join(".together")
        .join("worktrees")
        .join(&safe_task_id);
    if worktree_dir.exists() {
        return Ok(worktree_dir);
    }
    if let Some(parent) = worktree_dir.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let status = std::process::Command::new("git")
        .args(["worktree", "add", "--detach"])
        .arg(&worktree_dir)
        .current_dir(repo_root)
        .status()?;

    if status.success() {
        Ok(worktree_dir)
    } else {
        Err(format!("git worktree add failed with {status}").into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::fake::FakeAdapter;
    use std::sync::mpsc::channel;

    #[test]
    fn test_execution_supervisor() {
        let store = Arc::new(Mutex::new(EventStore::in_memory().unwrap()));
        let subscribers = Arc::new(Mutex::new(Vec::new()));

        let (tx, rx) = channel();
        subscribers.lock().unwrap().push(tx);

        let contract = TaskContract::minimal("test-task");

        let inputs = Arc::new(Mutex::new(HashMap::new()));
        let command = FakeAdapter.build_command(&contract);
        ExecutionSupervisor::run_command(
            "test-task".to_string(),
            command,
            None,
            store.clone(),
            subscribers.clone(),
            inputs,
        );

        let mut got_output = false;
        let mut got_completed = false;

        let mut full_output = String::new();

        for _i in 0..20 {
            if let Ok(event) = rx.recv_timeout(std::time::Duration::from_millis(500)) {
                match event {
                    Event::PtyOutput { task_id, chunk } => {
                        assert_eq!(task_id, "test-task");
                        full_output.push_str(&chunk);
                        got_output = true;
                    }
                    Event::TaskCompleted { task_id, success } => {
                        assert_eq!(task_id, "test-task");
                        assert!(success);
                        got_completed = true;
                        break; // exit loop after completion
                    }
                    _ => {}
                }
            }
        }

        assert!(got_output, "Did not receive PtyOutput event");
        assert!(
            full_output.contains("Doing fake work..."),
            "Output did not contain expected text. Got: {}",
            full_output
        );
        assert!(got_completed, "Did not receive TaskCompleted event");
    }
}
