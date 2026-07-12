use core::contracts::TaskContract;
use std::process::Command;

pub mod fake;

pub trait AgentAdapter: Send + Sync {
    fn build_command(&self, contract: &TaskContract) -> Command;
}
