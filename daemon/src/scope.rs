use core::contracts::TaskContract;

#[derive(Debug, Clone, PartialEq)]
pub enum ScopeDecision {
    Pass,
    NeedsReview { files: Vec<String> },
    Reject { files: Vec<String> },
}

pub fn verify_changed_files(contract: &TaskContract, changed_files: &[String]) -> ScopeDecision {
    let mut rejected = Vec::new();
    let mut needs_review = Vec::new();

    for file in changed_files {
        let normalized = normalize_path(file);
        if matches_any(&normalized, &contract.denied_files) {
            rejected.push(file.clone());
            continue;
        }

        let in_allowed = matches_any(&normalized, &contract.allowed_files)
            || matches_any(&normalized, &contract.scope);

        if !in_allowed {
            match (contract.enforcement_mode, contract.unknown_files_policy) {
                (core::contracts::EnforcementMode::Strict, _)
                | (_, core::contracts::UnknownFilesPolicy::Reject) => rejected.push(file.clone()),
                (_, core::contracts::UnknownFilesPolicy::NeedsReview) => {
                    needs_review.push(file.clone())
                }
                (_, core::contracts::UnknownFilesPolicy::Allow) => {}
            }
        }
    }

    if !rejected.is_empty() {
        ScopeDecision::Reject { files: rejected }
    } else if !needs_review.is_empty() {
        ScopeDecision::NeedsReview {
            files: needs_review,
        }
    } else {
        ScopeDecision::Pass
    }
}

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/").trim_start_matches("./").to_string()
}

fn matches_any(path: &str, patterns: &[String]) -> bool {
    patterns
        .iter()
        .map(|pattern| normalize_path(pattern))
        .any(|pattern| matches_pattern(path, &pattern))
}

fn matches_pattern(path: &str, pattern: &str) -> bool {
    if pattern == path || pattern == "**" {
        return true;
    }

    wildcard_match(path.as_bytes(), pattern.as_bytes())
}

fn wildcard_match(text: &[u8], pattern: &[u8]) -> bool {
    let (mut text_i, mut pattern_i) = (0usize, 0usize);
    let mut star_i = None;
    let mut match_i = 0usize;

    while text_i < text.len() {
        if pattern_i < pattern.len()
            && (pattern[pattern_i] == b'?' || pattern[pattern_i] == text[text_i])
        {
            text_i += 1;
            pattern_i += 1;
        } else if pattern_i < pattern.len() && pattern[pattern_i] == b'*' {
            star_i = Some(pattern_i);
            match_i = text_i;
            pattern_i += 1;
        } else if let Some(star) = star_i {
            pattern_i = star + 1;
            match_i += 1;
            text_i = match_i;
        } else {
            return false;
        }
    }

    while pattern_i < pattern.len() && pattern[pattern_i] == b'*' {
        pattern_i += 1;
    }

    pattern_i == pattern.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::contracts::{EnforcementMode, TaskContract, UnknownFilesPolicy};

    fn contract() -> TaskContract {
        TaskContract {
            task_id: "TASK-1".to_string(),
            title: Some("Scoped change".to_string()),
            intent: None,
            department: Some("engineering".to_string()),
            agent: None,
            scope: vec!["src/together/*.py".to_string()],
            allowed_files: vec!["src/together/routing.py".to_string()],
            denied_files: vec!["src/together/registry.py".to_string(), ".env".to_string()],
            deliverables: vec!["routing fix".to_string()],
            success_criteria: vec!["tests pass".to_string()],
            reviewer_required: true,
            verification_required: true,
            merge_authority: Some("codex".to_string()),
            enforcement_mode: EnforcementMode::Strict,
            unknown_files_policy: UnknownFilesPolicy::NeedsReview,
        }
    }

    #[test]
    fn strict_scope_passes_allowed_files() {
        let changed = vec!["src/together/routing.py".to_string()];

        assert_eq!(
            verify_changed_files(&contract(), &changed),
            ScopeDecision::Pass
        );
    }

    #[test]
    fn strict_scope_rejects_denied_files() {
        let changed = vec!["src/together/registry.py".to_string()];

        assert_eq!(
            verify_changed_files(&contract(), &changed),
            ScopeDecision::Reject {
                files: vec!["src/together/registry.py".to_string()]
            }
        );
    }

    #[test]
    fn strict_scope_rejects_out_of_scope_files() {
        let changed = vec!["README.md".to_string()];

        assert_eq!(
            verify_changed_files(&contract(), &changed),
            ScopeDecision::Reject {
                files: vec!["README.md".to_string()]
            }
        );
    }
}
