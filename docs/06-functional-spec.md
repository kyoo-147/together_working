# 06 — Functional Specification

## CLI commands

```bash
together init
together discover
together status
together agents list
together agents inspect <id>
together departments list
together departments create
together task create "..."
together task inspect <id>
together task route <id>
together task attach <id>
together task cancel <id>
together task reroute <id>
together review <id>
together verify <id>
together approve <id>
together logs <id>
together daemon start|stop|status
together config edit
```

## TUI actions
- create task;
- assign department;
- edit contract;
- start execution;
- send input to PTY;
- detach/reattach;
- review diff;
- run verification;
- reroute;
- approve integration.

## Routing policy example

```toml
[departments.engineering]
capabilities = ["code", "refactor", "debug"]
primary = ["opencode", "amp"]
fallback = ["codex"]

[routing]
require_ready = true
prefer_low_load = true
max_retries = 2
cooldown_seconds = 300
```

## Task contract example

```yaml
id: TASK-142
intent: Fix flaky authentication test
department: engineering
allowed_paths:
  - src/auth/**
  - tests/auth/**
denied_paths:
  - .env
  - secrets/**
constraints:
  max_changed_lines: 200
  new_dependencies: false
success_criteria:
  - test passes 3 consecutive times
  - lint passes
  - no unrelated changes
reviewer: codex
verification_required: true
merge_policy: human_approval
```

## Adapter contract

```rust
trait AgentAdapter {
    fn detect(&self) -> DetectionResult;
    async fn probe(&self) -> Readiness;
    fn capabilities(&self) -> Vec<Capability>;
    async fn spawn(&self, task: &TaskContract, pty: PtyHandle) -> Execution;
    fn parse_state(&self, frame: &TerminalFrame) -> AgentState;
    async fn resume(&self, execution: &Execution) -> Result<()>;
}
```

## Local API examples

```json
{"method":"task.create","params":{"intent":"Fix auth test"}}
{"method":"task.reroute","params":{"task_id":"TASK-142"}}
{"method":"events.subscribe","params":{"workspace":"current"}}
```
