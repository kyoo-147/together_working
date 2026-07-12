use std::process::Command;
use core::contracts::TaskContract;
use super::AgentAdapter;

pub struct FakeAdapter;

impl AgentAdapter for FakeAdapter {
    fn build_command(&self, _contract: &TaskContract) -> Command {
        #[cfg(target_os = "windows")]
        {
            let mut cmd = Command::new("cmd");
            cmd.args(&["/c", "echo", "Doing fake work..."]);
            cmd
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            let mut cmd = Command::new("sh");
            cmd.args(&["-c", "echo 'Doing fake work...'"]);
            cmd
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_command_returns_fake_work_command() {
        let contract = TaskContract {
            task_id: "test-task".to_string(),
            department: None,
            agent: None,
        };
        let adapter = FakeAdapter;
        let cmd = adapter.build_command(&contract);
        
        #[cfg(target_os = "windows")]
        {
            assert_eq!(cmd.get_program(), "cmd");
            let args: Vec<_> = cmd.get_args().collect();
            assert_eq!(args.len(), 3);
            assert_eq!(args[0], "/c");
            assert_eq!(args[1], "echo");
            assert_eq!(args[2], "Doing fake work...");
        }

        #[cfg(not(target_os = "windows"))]
        {
            assert_eq!(cmd.get_program(), "sh");
            let args: Vec<_> = cmd.get_args().collect();
            assert_eq!(args.len(), 2);
            assert_eq!(args[0], "-c");
            assert_eq!(args[1], "echo 'Doing fake work...'");
        }
    }
}
