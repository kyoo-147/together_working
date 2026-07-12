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

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::Daemon) => {
            println!("Starting daemon...");
        }
        Some(Commands::Run { file }) => {
            run_task(file);
        }
        None => {
            if let Err(err) = tui::run_tui() {
                eprintln!("Application error: {}", err);
                std::process::exit(1);
            }
        }
    }
}
