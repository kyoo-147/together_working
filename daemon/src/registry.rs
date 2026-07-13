use core::events::{AgentStatus, RoutingTarget};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct AgentInfo {
    pub status: AgentStatus,
    pub department: String,
    pub capabilities: Vec<String>,
    pub preferred_rank: i32,
    pub recent_failures: u32,
}

pub struct AgentRegistry {
    agents: BTreeMap<String, AgentInfo>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RouteDecision {
    pub agent_name: String,
    pub score: i32,
    pub reason: String,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: BTreeMap::new(),
        }
    }

    pub fn register_agent(&mut self, name: String, department: String) {
        let preferred_rank = match name.as_str() {
            "codex" => 100,
            "cmdc" => 80,
            "agy" => 70,
            "claude" => 65,
            _ => 40,
        };
        self.agents.insert(
            name,
            AgentInfo {
                status: AgentStatus::Ready,
                department,
                capabilities: vec!["code".to_string(), "terminal".to_string()],
                preferred_rank,
                recent_failures: 0,
            },
        );
    }

    pub fn update_status(&mut self, name: &str, status: AgentStatus) {
        if let Some(info) = self.agents.get_mut(name) {
            info.status = status;
        }
    }

    pub fn get_available_agent(&self, target: &RoutingTarget) -> Option<String> {
        self.rank_agent(target).map(|decision| decision.agent_name)
    }

    pub fn rank_agent(&self, target: &RoutingTarget) -> Option<RouteDecision> {
        let degraded_agents = self
            .agents
            .iter()
            .filter_map(|(name, info)| match &info.status {
                AgentStatus::Degraded { .. } | AgentStatus::Cooldown { .. } => Some(name.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();

        self.agents
            .iter()
            .filter(|(name, info)| Self::matches_target(name, info, target))
            .filter_map(|(name, info)| Self::score_agent(info).map(|score| (name, info, score)))
            .max_by(|(name_a, _, score_a), (name_b, _, score_b)| {
                score_a.cmp(score_b).then_with(|| name_b.cmp(name_a))
            })
            .map(|(name, info, score)| RouteDecision {
                agent_name: name.clone(),
                score,
                reason: format!(
                    "preferred_rank={} readiness={:?} recent_failures={} degraded_skipped={}",
                    info.preferred_rank,
                    info.status,
                    info.recent_failures,
                    if degraded_agents.is_empty() {
                        "none".to_string()
                    } else {
                        degraded_agents.join(",")
                    }
                ),
            })
    }

    pub fn agents(&self) -> &BTreeMap<String, AgentInfo> {
        &self.agents
    }

    pub fn mark_failure(&mut self, name: &str, reason: String) {
        if let Some(info) = self.agents.get_mut(name) {
            info.recent_failures += 1;
            info.status = AgentStatus::Degraded { reason };
        }
    }

    fn matches_target(name: &str, info: &AgentInfo, target: &RoutingTarget) -> bool {
        match target {
            RoutingTarget::Agent(target_name) => name == target_name,
            RoutingTarget::Department(dept) => info.department == *dept,
            RoutingTarget::Any => true,
        }
    }

    fn score_agent(info: &AgentInfo) -> Option<i32> {
        let readiness_score = match &info.status {
            AgentStatus::Ready => 100,
            AgentStatus::Unknown => 0,
            AgentStatus::Busy
            | AgentStatus::Blocked
            | AgentStatus::Degraded { .. }
            | AgentStatus::Offline
            | AgentStatus::Cooldown { .. } => return None,
        };

        Some(info.preferred_rank + readiness_score - (info.recent_failures as i32 * 10))
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_get_agent() {
        let mut registry = AgentRegistry::new();
        registry.register_agent("agent1".to_string(), "dept1".to_string());

        let target = RoutingTarget::Agent("agent1".to_string());
        assert_eq!(
            registry.get_available_agent(&target),
            Some("agent1".to_string())
        );
    }

    #[test]
    fn test_get_agent_busy() {
        let mut registry = AgentRegistry::new();
        registry.register_agent("agent1".to_string(), "dept1".to_string());
        registry.update_status("agent1", AgentStatus::Busy);

        let target = RoutingTarget::Agent("agent1".to_string());
        assert_eq!(registry.get_available_agent(&target), None);
    }

    #[test]
    fn test_department_routing() {
        let mut registry = AgentRegistry::new();
        registry.register_agent("agent1".to_string(), "dept1".to_string());
        registry.register_agent("agent2".to_string(), "dept2".to_string());

        let target = RoutingTarget::Department("dept2".to_string());
        assert_eq!(
            registry.get_available_agent(&target),
            Some("agent2".to_string())
        );
    }

    #[test]
    fn test_any_routing() {
        let mut registry = AgentRegistry::new();
        registry.register_agent("agent1".to_string(), "dept1".to_string());
        registry.update_status("agent1", AgentStatus::Offline);
        registry.register_agent("agent2".to_string(), "dept2".to_string());

        let target = RoutingTarget::Any;
        assert_eq!(
            registry.get_available_agent(&target),
            Some("agent2".to_string())
        );
    }

    #[test]
    fn test_ranking_prefers_codex_when_ready() {
        let mut registry = AgentRegistry::new();
        registry.register_agent("cmdc".to_string(), "engineering".to_string());
        registry.register_agent("codex".to_string(), "engineering".to_string());

        let decision = registry
            .rank_agent(&RoutingTarget::Department("engineering".to_string()))
            .unwrap();

        assert_eq!(decision.agent_name, "codex");
        assert!(decision.reason.contains("preferred_rank"));
    }

    #[test]
    fn test_ranking_falls_back_when_codex_degraded() {
        let mut registry = AgentRegistry::new();
        registry.register_agent("codex".to_string(), "engineering".to_string());
        registry.register_agent("cmdc".to_string(), "engineering".to_string());
        registry.update_status(
            "codex",
            AgentStatus::Degraded {
                reason: "Access is denied".to_string(),
            },
        );

        let decision = registry
            .rank_agent(&RoutingTarget::Department("engineering".to_string()))
            .unwrap();

        assert_eq!(decision.agent_name, "cmdc");
        assert!(decision.reason.contains("codex"));
    }
}
