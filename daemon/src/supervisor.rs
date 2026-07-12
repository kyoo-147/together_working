use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use std::thread;
use std::io::Read;
use core::events::Event;
use core::contracts::TaskContract;
use crate::store::EventStore;
use crate::pty::PtyManager;
use crate::adapters::AgentAdapter;
use crate::adapters::fake::FakeAdapter;

pub struct ExecutionSupervisor;

impl ExecutionSupervisor {
    pub fn run_task(
        task_id: String,
        contract: TaskContract,
        store: Arc<Mutex<EventStore>>,
        subscribers: Arc<Mutex<Vec<Sender<Event>>>>,
    ) {
        thread::spawn(move || {
            let adapter = FakeAdapter;
            let command = adapter.build_command(&contract);
            
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
            
            let (tx_exit, rx_exit) = std::sync::mpsc::channel();
            
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
                task_id,
                success,
            };
            Self::broadcast_event(&event, &store, &subscribers);
        });
    }
    
    fn broadcast_event(
        event: &Event, 
        store: &Arc<Mutex<EventStore>>, 
        subscribers: &Arc<Mutex<Vec<Sender<Event>>>>
    ) {
        if let Ok(store_lock) = store.lock() {
            let _ = store_lock.append(event);
        }
        if let Ok(mut subs_lock) = subscribers.lock() {
            subs_lock.retain(|sender| sender.send(event.clone()).is_ok());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::channel;

    #[test]
    fn test_execution_supervisor() {
        let store = Arc::new(Mutex::new(EventStore::in_memory().unwrap()));
        let subscribers = Arc::new(Mutex::new(Vec::new()));
        
        let (tx, rx) = channel();
        subscribers.lock().unwrap().push(tx);
        
        let contract = TaskContract {
            task_id: "test-task".to_string(),
            department: None,
            agent: None,
        };
        
        ExecutionSupervisor::run_task("test-task".to_string(), contract, store.clone(), subscribers.clone());
        
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
        assert!(full_output.contains("Doing fake work..."), "Output did not contain expected text. Got: {}", full_output);
        assert!(got_completed, "Did not receive TaskCompleted event");
    }
}
