use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskContract {
    pub task_id: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub intent: Option<String>,
    #[serde(default)]
    pub department: Option<String>,
    #[serde(default)]
    pub agent: Option<String>,
    #[serde(default)]
    pub scope: Vec<String>,
    #[serde(default)]
    pub allowed_files: Vec<String>,
    #[serde(default)]
    pub denied_files: Vec<String>,
    #[serde(default)]
    pub deliverables: Vec<String>,
    #[serde(default)]
    pub success_criteria: Vec<String>,
    #[serde(default)]
    pub reviewer_required: bool,
    #[serde(default)]
    pub verification_required: bool,
    #[serde(default)]
    pub merge_authority: Option<String>,
    #[serde(default)]
    pub enforcement_mode: EnforcementMode,
    #[serde(default)]
    pub unknown_files_policy: UnknownFilesPolicy,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum EnforcementMode {
    #[default]
    Warn,
    Strict,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum UnknownFilesPolicy {
    #[default]
    NeedsReview,
    Reject,
    Allow,
}

impl TaskContract {
    pub fn minimal(task_id: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
            title: Some("Minimal task".to_string()),
            intent: None,
            department: None,
            agent: None,
            scope: vec!["**".to_string()],
            allowed_files: vec!["**".to_string()],
            denied_files: vec![".env".to_string(), "**/secrets/*".to_string()],
            deliverables: Vec::new(),
            success_criteria: vec!["task completes".to_string()],
            reviewer_required: false,
            verification_required: false,
            merge_authority: Some("codex".to_string()),
            enforcement_mode: EnforcementMode::Warn,
            unknown_files_policy: UnknownFilesPolicy::NeedsReview,
        }
    }

    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }

    pub fn validate_for_dispatch(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.task_id.trim().is_empty() {
            errors.push("task_id is required".to_string());
        }
        if self.title.as_deref().unwrap_or("").trim().is_empty()
            && self.intent.as_deref().unwrap_or("").trim().is_empty()
        {
            errors.push("title or intent is required".to_string());
        }
        if self.scope.iter().all(|item| item.trim().is_empty()) {
            errors.push("scope is required before dispatch".to_string());
        }
        if self.allowed_files.iter().all(|item| item.trim().is_empty()) {
            errors.push("allowed_files is required before dispatch".to_string());
        }
        if self
            .success_criteria
            .iter()
            .all(|item| item.trim().is_empty())
        {
            errors.push("success_criteria is required before dispatch".to_string());
        }
        if self.denied_files.iter().any(|item| item.trim().is_empty()) {
            errors.push("denied_files cannot contain empty patterns".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn task_prompt(&self) -> String {
        let title = self
            .title
            .as_deref()
            .or(self.intent.as_deref())
            .unwrap_or("Untitled task");
        format!(
            "Task: {title}\n\nScope:\n{}\n\nAllowed files:\n{}\n\nDenied files:\n{}\n\nSuccess criteria:\n{}\n\nStay inside scope. Use auto-accept only for files allowed by the task contract.",
            render_list(&self.scope),
            render_list(&self.allowed_files),
            render_list(&self.denied_files),
            render_list(&self.success_criteria)
        )
    }
}

fn render_list(items: &[String]) -> String {
    if items.is_empty() {
        "- none".to_string()
    } else {
        items
            .iter()
            .map(|item| format!("- {item}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_contract() {
        let yaml = "
task_id: test-1
department: engineering
agent: codex
";
        let contract = TaskContract::from_yaml(yaml).unwrap();
        assert_eq!(contract.task_id, "test-1");
        assert_eq!(contract.department, Some("engineering".to_string()));
        assert_eq!(contract.agent, Some("codex".to_string()));
        assert_eq!(contract.enforcement_mode, EnforcementMode::Warn);
    }

    #[test]
    fn test_parse_full_task_contract_vocabulary() {
        let yaml = "
task_id: TASK-123
title: Harden routing fallback validation
intent: Make fallback routing choose a healthy worker
department: engineering
scope:
  - src/together/*.py
allowed_files:
  - src/together/routing.py
denied_files:
  - .env
  - \"**/secrets/*\"
deliverables:
  - updated routing fallback logic
success_criteria:
  - routing tests pass
reviewer_required: true
verification_required: true
merge_authority: codex
enforcement_mode: strict
unknown_files_policy: needs_review
";

        let contract = TaskContract::from_yaml(yaml).unwrap();

        assert_eq!(contract.task_id, "TASK-123");
        assert_eq!(
            contract.title.as_deref(),
            Some("Harden routing fallback validation")
        );
        assert_eq!(
            contract.intent.as_deref(),
            Some("Make fallback routing choose a healthy worker")
        );
        assert_eq!(contract.scope, vec!["src/together/*.py"]);
        assert_eq!(contract.allowed_files, vec!["src/together/routing.py"]);
        assert_eq!(contract.denied_files, vec![".env", "**/secrets/*"]);
        assert!(contract.reviewer_required);
        assert!(contract.verification_required);
        assert_eq!(contract.merge_authority.as_deref(), Some("codex"));
        assert_eq!(contract.enforcement_mode, EnforcementMode::Strict);
        assert_eq!(
            contract.unknown_files_policy,
            UnknownFilesPolicy::NeedsReview
        );
    }

    #[test]
    fn test_strict_preflight_rejects_missing_scope() {
        let yaml = "
task_id: TASK-124
title: Missing scope
department: engineering
allowed_files:
  - src/lib.rs
success_criteria:
  - cargo test passes
enforcement_mode: strict
";
        let contract = TaskContract::from_yaml(yaml).unwrap();

        let err = contract.validate_for_dispatch().unwrap_err();

        assert!(err.iter().any(|msg| msg.contains("scope")));
    }

    #[test]
    fn test_strict_preflight_rejects_empty_success_criteria() {
        let yaml = "
task_id: TASK-125
title: Missing acceptance
department: engineering
scope:
  - src/lib.rs
allowed_files:
  - src/lib.rs
enforcement_mode: strict
";
        let contract = TaskContract::from_yaml(yaml).unwrap();

        let err = contract.validate_for_dispatch().unwrap_err();

        assert!(err.iter().any(|msg| msg.contains("success_criteria")));
    }
}
