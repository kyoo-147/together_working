use core::contracts::TaskContract;
use core::events::AgentStatus;
use std::process::Command;

use super::{AgentAdapter, AgentProbe};

pub struct RealAgentAdapter {
    pub agent_name: String,
}

impl AgentAdapter for RealAgentAdapter {
    fn build_command(&self, contract: &TaskContract) -> Command {
        let prompt = contract.task_prompt();
        match self.agent_name.as_str() {
            "cmdc" => shell_command(&format!(
                "cmdc --print {} --permission-mode auto-accept --skip-onboarding",
                shell_quote(&prompt)
            )),
            "agy" => shell_command(&format!(
                "agy --print {} --dangerously-skip-permissions --print-timeout 15m",
                shell_quote(&prompt)
            )),
            "claude" => shell_command(&format!(
                "claude --print {} --permission-mode auto",
                shell_quote(&prompt)
            )),
            "codex" => shell_command(&format!("codex exec {}", shell_quote(&prompt))),
            other => shell_command(&format!("{} {}", other, shell_quote(&prompt))),
        }
    }
}

pub struct CommandProbe;

impl AgentProbe for CommandProbe {
    fn probe_command(&self, agent_name: &str) -> AgentStatus {
        let output = shell_command(&format!("{} --help", agent_name)).output();
        match output {
            Ok(output) if output.status.success() => AgentStatus::Ready,
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                let reason = if !stderr.trim().is_empty() {
                    stderr.trim().to_string()
                } else if !stdout.trim().is_empty() {
                    stdout.trim().to_string()
                } else {
                    format!("probe exited with {}", output.status)
                };
                AgentStatus::Degraded { reason }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => AgentStatus::Offline,
            Err(error) => AgentStatus::Degraded {
                reason: error.to_string(),
            },
        }
    }
}

fn shell_command(script: &str) -> Command {
    if cfg!(target_os = "windows") {
        let mut command = Command::new("powershell");
        command.args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ]);
        command
    } else {
        let mut command = Command::new("sh");
        command.args(["-lc", script]);
        command
    }
}

fn shell_quote(value: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("'{}'", value.replace('\'', "''"))
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct AccessDeniedProbe;

    impl AgentProbe for AccessDeniedProbe {
        fn probe_command(&self, _agent_name: &str) -> AgentStatus {
            AgentStatus::Degraded {
                reason: "Access is denied".to_string(),
            }
        }
    }

    #[test]
    fn codex_access_denied_probe_is_degraded() {
        let status = AccessDeniedProbe.probe_command("codex");

        assert_eq!(
            status,
            AgentStatus::Degraded {
                reason: "Access is denied".to_string()
            }
        );
    }

    #[test]
    fn real_adapter_builds_cmdc_print_command_with_contract_prompt() {
        let adapter = RealAgentAdapter {
            agent_name: "cmdc".to_string(),
        };
        let mut contract = TaskContract::minimal("TASK-1");
        contract.title = Some("Fix scoped bug".to_string());

        let command = adapter.build_command(&contract);
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().to_string())
            .collect::<Vec<_>>();

        assert!(
            command
                .get_program()
                .to_string_lossy()
                .contains("powershell")
                || command.get_program().to_string_lossy() == "sh"
        );
        assert!(args.iter().any(|arg| arg.contains("--print")));
        assert!(args.iter().any(|arg| arg.contains("Fix scoped bug")));
        assert!(args.iter().any(|arg| arg.contains("Allowed files")));
    }
}
