use core::models::{Agent, ReadinessState};

#[test]
fn test_agent_creation() {
    let agent = Agent {
        name: "codex".to_string(),
        state: ReadinessState::Ready,
    };
    assert_eq!(agent.name, "codex");
}
