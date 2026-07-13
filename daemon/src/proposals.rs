use core::chat::{ChatSource, CommandProposal, ProposalAction, ProposalStatus};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserBackend {
    Deterministic,
    Assisted,
    Auto,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProposalContext {
    pub selected_task_id: Option<String>,
}

pub struct DeterministicParser;
pub struct AssistedParser;

pub fn build_proposal(source: ChatSource, text: &str) -> CommandProposal {
    build_proposal_with_backend(
        source,
        text,
        ParserBackend::Auto,
        &ProposalContext::default(),
    )
}

pub fn build_proposal_with_backend(
    source: ChatSource,
    text: &str,
    backend: ParserBackend,
    context: &ProposalContext,
) -> CommandProposal {
    match backend {
        ParserBackend::Deterministic => DeterministicParser.parse(source, text, context),
        ParserBackend::Assisted => AssistedParser.parse(source, text, context),
        ParserBackend::Auto if text.trim_start().starts_with('/') => {
            DeterministicParser.parse(source, text, context)
        }
        ParserBackend::Auto => AssistedParser.parse(source, text, context),
    }
}

impl DeterministicParser {
    pub fn parse(
        &self,
        source: ChatSource,
        text: &str,
        _context: &ProposalContext,
    ) -> CommandProposal {
        let trimmed = text.trim();
        let (title, summary, action, preview) = if let Some(rest) = trimmed.strip_prefix("/task") {
            let title = non_empty(rest.trim(), "New scoped task");
            let yaml = task_yaml(title);
            (
                format!("Create task: {title}"),
                "Create a scoped task draft. Confirm to dispatch through daemon routing."
                    .to_string(),
                ProposalAction::CreateTask { yaml: yaml.clone() },
                Some(yaml),
            )
        } else if let Some(rest) = trimmed.strip_prefix("/verify") {
            let task_id = non_empty(rest.trim(), "selected");
            (
                format!("Verify task {task_id}"),
                "Request verification for the selected task.".to_string(),
                ProposalAction::VerifyTask {
                    task_id: task_id.to_string(),
                },
                None,
            )
        } else if let Some(rest) = trimmed.strip_prefix("/reroute") {
            let task_id = non_empty(rest.trim(), "selected");
            (
                format!("Reroute task {task_id}"),
                "Ask the daemon to route the task to the best healthy worker.".to_string(),
                ProposalAction::RerouteTask {
                    task_id: task_id.to_string(),
                },
                None,
            )
        } else if let Some(rest) = trimmed.strip_prefix("/approve") {
            let task_id = non_empty(rest.trim(), "selected");
            (
                format!("Approve task {task_id}"),
                "Approve the task if verification gates allow it.".to_string(),
                ProposalAction::ApproveTask {
                    task_id: task_id.to_string(),
                },
                None,
            )
        } else if trimmed == "/status" {
            (
                "Inspect system status".to_string(),
                "Show current daemon, task, agent, and gate status.".to_string(),
                ProposalAction::Status,
                None,
            )
        } else {
            let title = non_empty(trimmed, "New scoped work");
            let yaml = task_yaml(title);
            (
                format!("Draft task from chat: {title}"),
                "Natural language is converted into a safe scoped task draft for review."
                    .to_string(),
                ProposalAction::CreateTask { yaml: yaml.clone() },
                Some(yaml),
            )
        };

        CommandProposal {
            proposal_id: Uuid::new_v4().to_string(),
            source,
            title,
            summary,
            action,
            preview,
            status: ProposalStatus::Pending,
        }
    }
}

impl AssistedParser {
    pub fn parse(
        &self,
        source: ChatSource,
        text: &str,
        _context: &ProposalContext,
    ) -> CommandProposal {
        let title = non_empty(text.trim(), "New scoped work");
        let yaml = task_yaml(title);
        let preview = format!(
            "action: create_task\nrisk: medium\nparser: assisted\nsummary: Codex-assisted safe draft; review before confirm.\n{}",
            yaml
        );

        CommandProposal {
            proposal_id: Uuid::new_v4().to_string(),
            source,
            title: format!("Assisted draft: {title}"),
            summary: "assisted parser produced a safe scoped task draft; confirm before dispatch"
                .to_string(),
            action: ProposalAction::CreateTask { yaml },
            preview: Some(preview),
            status: ProposalStatus::Pending,
        }
    }
}

fn non_empty<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    if value.trim().is_empty() {
        fallback
    } else {
        value.trim()
    }
}

fn task_yaml(title: &str) -> String {
    format!(
        "task_id: draft\ntitle: {}\ndepartment: engineering\nscope:\n  - '**'\nallowed_files:\n  - '**'\ndenied_files:\n  - .env\n  - '**/secrets/*'\ndeliverables:\n  - implementation\nsuccess_criteria:\n  - task completes\nreviewer_required: false\nverification_required: true\nmerge_authority: codex\nenforcement_mode: strict\nunknown_files_policy: needs_review\n",
        quote_yaml(title)
    )
}

fn quote_yaml(value: &str) -> String {
    format!("{:?}", value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_task_command_as_pending_create_task_proposal() {
        let proposal = build_proposal(ChatSource::TogetherChat, "/task Build landing page");

        assert_eq!(proposal.status, ProposalStatus::Pending);
        assert!(proposal.title.contains("Build landing page"));
        assert!(matches!(proposal.action, ProposalAction::CreateTask { .. }));
        assert!(proposal.preview.unwrap().contains("allowed_files"));
    }

    #[test]
    fn parses_status_command() {
        let proposal = build_proposal(ChatSource::TogetherChat, "/status");

        assert_eq!(proposal.action, ProposalAction::Status);
    }

    #[test]
    fn natural_language_creates_safe_task_draft() {
        let proposal = build_proposal(ChatSource::CodexApp, "make the homepage cleaner");

        assert!(matches!(proposal.action, ProposalAction::CreateTask { .. }));
        assert!(proposal.summary.contains("safe scoped task draft"));
    }

    #[test]
    fn assisted_parser_adds_action_scope_and_risk_preview() {
        let parser = AssistedParser;
        let proposal = parser.parse(
            ChatSource::CodexApp,
            "create a landing page in README and docs",
            &ProposalContext::default(),
        );

        assert!(proposal.summary.contains("assisted"));
        let preview = proposal.preview.as_deref().unwrap();
        assert!(preview.contains("action: create_task"));
        assert!(preview.contains("scope:"));
        assert!(preview.contains("allowed_files:"));
        assert!(preview.contains("success_criteria:"));
        assert!(preview.contains("risk:"));
    }

    #[test]
    fn parser_backend_auto_uses_deterministic_for_slash_commands() {
        let proposal = build_proposal_with_backend(
            ChatSource::TogetherChat,
            "/status",
            ParserBackend::Auto,
            &ProposalContext::default(),
        );

        assert_eq!(proposal.action, ProposalAction::Status);
    }
}
