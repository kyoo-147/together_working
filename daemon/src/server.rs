use interprocess::local_socket::{prelude::*, GenericNamespaced, ListenerOptions};
use interprocess::TryClone;
use std::io::{BufRead, BufReader, Write};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender};
use uuid::Uuid;

use core::ipc::{Command, Response};
use core::contracts::TaskContract;
use core::events::Event;
use crate::store::EventStore;
use crate::registry::AgentRegistry;
use crate::router::Router;

#[cfg(test)]
mod tests {
    use super::*;
    use interprocess::local_socket::{GenericNamespaced, Stream};
    use core::ipc::{Command, Response};
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

        let cmd = Command::CreateTask { yaml: "task_id: dummy\ndepartment: HR\nagent: Bob\n".to_string() };
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
}

pub fn start_server(
    socket_name: &str, 
    store: Arc<Mutex<EventStore>>, 
    registry: Arc<Mutex<AgentRegistry>>
) -> std::io::Result<()> {
    let name = socket_name.to_ns_name::<GenericNamespaced>()?;
    let opts = ListenerOptions::new().name(name);
    let listener = opts.create_sync()?;
    
    let subscribers: Arc<Mutex<Vec<Sender<Event>>>> = Arc::new(Mutex::new(Vec::new()));
    
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
            
            std::thread::spawn(move || {
                let mut reader = BufReader::new(&mut conn_clone);
                let mut buffer = String::new();
                
                if let Ok(bytes) = reader.read_line(&mut buffer) {
                    if bytes == 0 { return; }
                    
                    if let Ok(cmd) = serde_json::from_str::<Command>(&buffer) {
                        match cmd {
                            Command::CreateTask { yaml } => {
                                match TaskContract::from_yaml(&yaml) {
                                    Ok(mut contract) => {
                                        let task_id = Uuid::new_v4().to_string();
                                        contract.task_id = task_id.clone();
                                        
                                        let event1 = Event::TaskCreated {
                                            task_id: task_id.clone(),
                                            contract: contract.clone(),
                                        };
                                        
                                        let event2 = {
                                            let reg_lock = registry.lock().unwrap();
                                            Router::route_task(&contract, &reg_lock)
                                        };
                                        
                                        {
                                            let store_lock = store.lock().unwrap();
                                            let _ = store_lock.append(&event1);
                                            let _ = store_lock.append(&event2);
                                        }
                                        
                                        {
                                            let mut subs = subscribers.lock().unwrap();
                                            subs.retain(|sender| {
                                                sender.send(event1.clone()).is_ok() && 
                                                sender.send(event2.clone()).is_ok()
                                            });
                                        }
                                        
                                        let resp = Response::Ack { task_id };
                                        let resp_str = serde_json::to_string(&resp).unwrap() + "\n";
                                        if let Err(e) = conn.write_all(resp_str.as_bytes()) {
                                            eprintln!("Failed to write response: {}", e);
                                        }
                                    }
                                    Err(e) => {
                                        let resp = Response::Error { message: e.to_string() };
                                        let resp_str = serde_json::to_string(&resp).unwrap() + "\n";
                                        if let Err(e) = conn.write_all(resp_str.as_bytes()) {
                                            eprintln!("Failed to write response: {}", e);
                                        }
                                    }
                                }
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
                        let resp = Response::Error { message: "Invalid command format".to_string() };
                        let resp_str = serde_json::to_string(&resp).unwrap() + "\n";
                        if let Err(e) = conn.write_all(resp_str.as_bytes()) {
                            eprintln!("Failed to write response: {}", e);
                        }
                    }
                }
            });
        }
    });
    Ok(())
}
