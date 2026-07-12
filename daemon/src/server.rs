use interprocess::local_socket::{prelude::*, GenericNamespaced, ListenerOptions, Stream};
use interprocess::TryClone;
use std::io::{BufRead, BufReader, Write};

#[cfg(test)]
mod tests {
    use super::*;
    use interprocess::local_socket::{prelude::*, GenericNamespaced, Stream};
    use std::io::{BufRead, BufReader, Write};
    use core::events::Event;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn test_ipc_communication() {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let socket_name = format!("together-test-{}-{}.sock", std::process::id(), id);

        // Start the server in the background
        // Limitation: full cleanup is difficult with blocking I/O (listener.incoming() is blocking).
        // For now, we use a unique socket name per test to avoid cross-test pollution.
        start_server(&socket_name).expect("Failed to start server");

        // Give it a moment to start
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Connect as a client
        let name = socket_name.to_ns_name::<GenericNamespaced>().unwrap();
        let mut conn = Stream::connect(name).expect("Failed to connect to server");

        // Send a command
        let cmd = "test_contract.rs\n";
        conn.write_all(cmd.as_bytes()).unwrap();

        // Read the response
        let mut reader = BufReader::new(&mut conn);
        let mut response = String::new();
        reader.read_line(&mut response).unwrap();

        // Verify the response
        let event: Event = serde_json::from_str(&response).unwrap();
        match event {
            Event::TaskCreated { task_id, contract_path } => {
                assert_eq!(task_id, "1");
                assert_eq!(contract_path, "test_contract.rs");
            }
            _ => panic!("Unexpected event type"),
        }
    }
}

pub fn start_server(socket_name: &str) -> std::io::Result<()> {
    let name = socket_name.to_ns_name::<GenericNamespaced>()?;
    let opts = ListenerOptions::new().name(name);
    let listener = opts.create_sync()?;
    
    std::thread::spawn(move || {
        for mut conn in listener.incoming().filter_map(Result::ok) {
            let mut conn_clone = match conn.try_clone() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to clone connection: {}", e);
                    continue;
                }
            };
            std::thread::spawn(move || {
                let mut reader = BufReader::new(&mut conn_clone);
                let mut buffer = String::new();
                while let Ok(bytes) = reader.read_line(&mut buffer) {
                    if bytes == 0 { break; }
                    println!("Daemon received: {}", buffer);
                    // TODO: Echo back as TaskCreated event for now - this is a temporary stub
                    let event = core::events::Event::TaskCreated { task_id: "1".to_string(), contract_path: buffer.trim().to_string() };
                    let response = serde_json::to_string(&event).unwrap() + "\n";
                    if let Err(e) = conn.write_all(response.as_bytes()) {
                        eprintln!("Failed to write to client: {}", e);
                        break;
                    }
                    buffer.clear();
                }
            });
        }
    });
    Ok(())
}
