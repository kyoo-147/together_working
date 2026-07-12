mod tui;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "together", about = "AI Department Orchestrator")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Daemon,
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::Daemon) => {
            println!("Starting daemon...");
        }
        None => {
            tui::run_tui().unwrap();
        }
    }
}
