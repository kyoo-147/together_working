mod client;
mod tui;
pub mod ui;

use clap::{Parser, Subcommand, ValueEnum};
use core::chat::ChatSource;
use core::events::Event;
use core::ipc::{Command, Response};
use core::settings::UiSettings;
use std::io::Write;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const DAEMON_READY_TIMEOUT: Duration = Duration::from_secs(45);

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
    Attach {
        task_id: String,
    },
    Chat {
        #[arg(long, value_enum, default_value_t = SourceArg::TogetherChat)]
        source: SourceArg,
        text: String,
    },
    Proposal {
        #[command(subcommand)]
        command: ProposalCommand,
    },
    Status {
        #[arg(long)]
        json: bool,
    },
    Settings {
        #[command(subcommand)]
        command: SettingsCommand,
    },
    RequestReview {
        task_id: String,
    },
    Approve {
        task_id: String,
    },
    Reject {
        task_id: String,
        reason: String,
    },
    RequestChanges {
        task_id: String,
        instructions: String,
    },
    Doctor,
    SelfCheck,
    Version,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum SourceArg {
    CodexApp,
    TogetherChat,
    CliYaml,
}

impl From<SourceArg> for ChatSource {
    fn from(value: SourceArg) -> Self {
        match value {
            SourceArg::CodexApp => ChatSource::CodexApp,
            SourceArg::TogetherChat => ChatSource::TogetherChat,
            SourceArg::CliYaml => ChatSource::CliYaml,
        }
    }
}

#[derive(Subcommand)]
enum ProposalCommand {
    Confirm { id: String },
    Reject { id: String },
}

#[derive(Subcommand)]
enum SettingsCommand {
    Get {
        #[arg(long)]
        json: bool,
    },
    Set {
        #[arg(long)]
        theme: Option<String>,
        #[arg(long)]
        bg: Option<String>,
        #[arg(long)]
        main: Option<String>,
    },
    Reset,
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
            Response::Proposal { proposal_id } => println!("Proposal created: {}", proposal_id),
            Response::Settings { settings } => {
                println!("{}", serde_json::to_string_pretty(&settings).unwrap())
            }
            Response::Status { json } => println!("{json}"),
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
        let mut events = daemon::server::bootstrap_registry(&mut reg);
        let settings = daemon::settings::load_settings(&std::env::current_dir()?);
        events.push(Event::SettingsUpdated { settings });
        events
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

    let deadline = Instant::now() + DAEMON_READY_TIMEOUT;
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
        format!(
            "daemon did not become ready within {}s",
            DAEMON_READY_TIMEOUT.as_secs()
        ),
    ))
}

fn send_command(command: Command) -> Result<Response, std::io::Error> {
    let cmd_str = format!("{}\n", serde_json::to_string(&command).unwrap());
    let resp = client::send_command(client::DEFAULT_SOCKET_NAME, &cmd_str)?;
    serde_json::from_str::<Response>(&resp)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

fn print_response(response: Response) {
    match response {
        Response::Ack { task_id } => println!("OK: {task_id}"),
        Response::Proposal { proposal_id } => println!("Proposal created: {proposal_id}"),
        Response::Settings { settings } => {
            println!("{}", serde_json::to_string_pretty(&settings).unwrap())
        }
        Response::Status { json } => println!("{json}"),
        Response::Error { message } => {
            eprintln!("Error: {message}");
            std::process::exit(1);
        }
    }
}

fn run_command(command: Command) {
    if let Err(err) = ensure_daemon() {
        eprintln!("Failed to auto-start daemon: {}", err);
        std::process::exit(1);
    }
    match send_command(command) {
        Ok(response) => print_response(response),
        Err(e) => {
            eprintln!("Failed to connect to daemon: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_settings(command: &SettingsCommand) {
    match command {
        SettingsCommand::Get { .. } => run_command(Command::GetSettings),
        SettingsCommand::Set { theme, bg, main } => {
            if let Err(err) = ensure_daemon() {
                eprintln!("Failed to auto-start daemon: {}", err);
                std::process::exit(1);
            }
            let mut settings = match send_command(Command::GetSettings) {
                Ok(Response::Settings { settings }) => settings,
                _ => UiSettings::default(),
            };
            if let Some(theme) = theme {
                settings.theme_preset = theme.clone();
            }
            if let Some(bg) = bg {
                settings.custom_bg = Some(bg.clone());
            }
            if let Some(main) = main {
                settings.custom_main = Some(main.clone());
            }
            run_command(Command::UpdateSettings { settings });
        }
        SettingsCommand::Reset => run_command(Command::UpdateSettings {
            settings: UiSettings::default(),
        }),
    }
}

fn run_doctor() {
    if ensure_daemon().is_ok() {
        println!("daemon: ready");
    } else {
        println!("daemon: not ready");
    }
    println!(
        "binary: {}",
        std::env::current_exe().unwrap_or_default().display()
    );
}

fn run_self_check() {
    run_doctor();
    run_command(Command::GetStatus);
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
        Some(Commands::Chat { source, text }) => {
            run_command(Command::SubmitChat {
                source: (*source).into(),
                text: text.clone(),
            });
        }
        Some(Commands::Proposal { command }) => match command {
            ProposalCommand::Confirm { id } => run_command(Command::ConfirmProposal {
                proposal_id: id.clone(),
            }),
            ProposalCommand::Reject { id } => run_command(Command::RejectProposal {
                proposal_id: id.clone(),
            }),
        },
        Some(Commands::Status { .. }) => run_command(Command::GetStatus),
        Some(Commands::Settings { command }) => run_settings(command),
        Some(Commands::RequestReview { task_id }) => run_command(Command::RequestReview {
            task_id: task_id.clone(),
        }),
        Some(Commands::Approve { task_id }) => run_command(Command::ApproveTask {
            task_id: task_id.clone(),
        }),
        Some(Commands::Reject { task_id, reason }) => run_command(Command::RejectTask {
            task_id: task_id.clone(),
            reason: reason.clone(),
        }),
        Some(Commands::RequestChanges {
            task_id,
            instructions,
        }) => run_command(Command::RequestChanges {
            task_id: task_id.clone(),
            instructions: instructions.clone(),
        }),
        Some(Commands::Doctor) => run_doctor(),
        Some(Commands::SelfCheck) => run_self_check(),
        Some(Commands::Version) => println!("together {}", env!("CARGO_PKG_VERSION")),
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
