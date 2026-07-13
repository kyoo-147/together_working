mod client;
mod tui;
pub mod ui;

use clap::{Parser, Subcommand};
use core::events::Event;
use core::ipc::{Command, Response};
use std::io::Write;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Parser)]
#[command(name = "together", about = "AI Department Orchestrator")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Daemon,
    Run { file: String },
    Attach { task_id: String },
}

fn run_task(file: &str) {
    let yaml = match std::fs::read_to_string(file) {
        Ok(y) => y,
        Err(e) => {
            eprintln!("Failed to read file: {}", e);
            return;
        }
    };
    let cmd = Command::CreateTask { yaml };
    let cmd_str = format!("{}\n", serde_json::to_string(&cmd).unwrap());
    let resp_str = match client::send_command(client::DEFAULT_SOCKET_NAME, &cmd_str) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to connect to daemon: {}", e);
            return;
        }
    };
    if let Ok(resp) = serde_json::from_str::<Response>(&resp_str) {
        match resp {
            Response::Ack { task_id } => println!("Task created: {}", task_id),
            Response::Error { message } => eprintln!("Error: {}", message),
        }
    } else {
        println!("Raw response: {}", resp_str);
    }
}

fn start_daemon_foreground() -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(".together")?;
    let store = Arc::new(Mutex::new(daemon::store::EventStore::new(
        ".together/events.db",
    )?));
    let registry = Arc::new(Mutex::new(daemon::registry::AgentRegistry::new()));
    let bootstrap_events = {
        let mut reg = registry.lock().unwrap();
        daemon::server::bootstrap_registry(&mut reg)
    };
    {
        let store_lock = store.lock().unwrap();
        for event in &bootstrap_events {
            let _ = store_lock.append(event);
        }
    }
    daemon::server::start_server(client::DEFAULT_SOCKET_NAME, store, registry)?;
    println!(
        "Together daemon listening on {}",
        client::DEFAULT_SOCKET_NAME
    );
    loop {
        std::thread::sleep(Duration::from_secs(3600));
    }
}

fn ensure_daemon() -> Result<(), std::io::Error> {
    if client::subscribe(client::DEFAULT_SOCKET_NAME).is_ok() {
        return Ok(());
    }

    let exe = std::env::current_exe()?;
    let mut child = std::process::Command::new(exe)
        .arg("daemon")
        .current_dir(std::env::current_dir()?)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    let deadline = Instant::now() + Duration::from_secs(15);
    while Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(100));
        if client::subscribe(client::DEFAULT_SOCKET_NAME).is_ok() {
            return Ok(());
        }
        if let Some(status) = child.try_wait()? {
            return Err(std::io::Error::other(format!(
                "daemon exited before becoming ready: {status}"
            )));
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::TimedOut,
        "daemon did not become ready",
    ))
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::Daemon) => {
            if let Err(err) = start_daemon_foreground() {
                eprintln!("Failed to start daemon: {}", err);
                std::process::exit(1);
            }
        }
        Some(Commands::Run { file }) => {
            run_task(file);
        }
        Some(Commands::Attach { task_id }) => {
            match client::subscribe(client::DEFAULT_SOCKET_NAME) {
                Ok(rx) => {
                    for event_res in rx {
                        match event_res {
                            Ok(Event::PtyOutput {
                                task_id: tid,
                                chunk,
                            }) if tid == *task_id => {
                                print!("{chunk}");
                                let _ = std::io::stdout().flush();
                            }
                            Ok(Event::TaskCompleted {
                                task_id: tid,
                                success,
                            }) if tid == *task_id => {
                                println!("\n[Task {} completed with success={}]", tid, success);
                                break;
                            }
                            Ok(_) => {}
                            Err(_) => {
                                println!("\n[Connection to daemon lost]");
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to connect to daemon: {}", e);
                }
            }
        }
        None => {
            if let Err(err) = ensure_daemon() {
                eprintln!("Failed to auto-start daemon: {}", err);
                std::process::exit(1);
            }
            if let Err(err) = tui::run_tui() {
                eprintln!("Application error: {}", err);
                std::process::exit(1);
            }
        }
    }
}
