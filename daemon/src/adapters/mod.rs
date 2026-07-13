use core::contracts::TaskContract;
use core::events::AgentStatus;
use std::process::Command;

#[cfg(test)]
pub mod fake;
pub mod real;

pub trait AgentAdapter: Send + Sync {
    fn build_command(&self, contract: &TaskContract) -> Command;
}

pub trait AgentProbe {
    fn probe_command(&self, agent_name: &str) -> AgentStatus;
}
