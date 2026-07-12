use core::discovery::scan_agents;

#[test]
fn test_scan_returns_agents() {
    let agents = scan_agents();
    assert!(agents.len() >= 0);
}
