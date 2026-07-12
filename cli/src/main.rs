mod tui;
mod client;
use clap::{Parser, Subcommand};
use core::ipc::{Command, Response};

#[derive(Parser)]
#[command(name = "together", about = "AI Department Orchestrator")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Daemon,
    Run {
        file: String,
    },
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::Daemon) => {
            println!("Starting daemon...");
        }
        Some(Commands::Run { file }) => {
            match std::fs::read_to_string(file) {
                Ok(yaml) => {
                    let cmd = Command::CreateTask { yaml };
                    let cmd_str = serde_json::to_string(&cmd).unwrap();
                    match client::send_command(client::DEFAULT_SOCKET_NAME, &cmd_str) {
                        Ok(resp_str) => {
                            if let Ok(resp) = serde_json::from_str::<Response>(&resp_str) {
                                match resp {
                                    Response::Ack { task_id } => println!("Task created: {}", task_id),
                                    Response::Error { message } => eprintln!("Error: {}", message),
                                }
                            } else {
                                println!("Raw response: {}", resp_str);
                            }
                        }
                        Err(e) => eprintln!("Failed to connect to daemon: {}", e),
                    }
                }
                Err(e) => eprintln!("Failed to read file: {}", e),
            }
        }
        None => {
            if let Err(err) = tui::run_tui() {
                eprintln!("Application error: {}", err);
                std::process::exit(1);
            }
        }
    }
}
