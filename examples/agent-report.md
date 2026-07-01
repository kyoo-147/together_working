# Together Agent Report

Generated: `2026-07-01T00:00:00+00:00`

Codex role: PM, control-plane, planner, coordinator, verifier, final-integrator.

## Summary

| Metric | Count |
|---|---:|
| Known Providers | 48 |
| Installed CLIs | 3 |
| Ready Agents | 2 |
| Broken Agents | 1 |
| Degraded Agents | 1 |

## Known Providers

| Provider | Commands | Hint Profile |
|---|---|---|
| Claude Code | `claude, claude-code` | research, review, verification, docs |
| Command Code | `cmdc, command-code` | vision, backend, frontend, review |
| OpenAI Codex CLI | `codex` | backend, frontend, review, verification |

## Installed CLIs

| Agent | Status | Command | Path |
|---|---|---|---|
| Command Code | ready | `cmdc` | `<sanitized>` |
| Claude Code | ready | `claude` | `<sanitized>` |
| OpenAI Codex CLI | permission-denied | `codex` | `<sanitized>` |

## Ready Agents

| Agent | Best Hints | Departments |
|---|---|---|
| Command Code | vision, backend, frontend, review | vision, engineering, review |
| Claude Code | research, review, verification, docs | planning, research, review |

## Broken Agents

| Agent | Status | Reason |
|---|---|---|
| OpenAI Codex CLI | permission-denied | CLI exists but OS permissions blocked execution. |

## Best Available Workers

| Rank | Agent | Best Hints | Departments |
|---:|---|---|---|
| 1 | Command Code | vision, backend | vision, engineering |
| 2 | Claude Code | research, review | planning, research |

## Best By Task

| Task | Best Worker | Alternatives |
|---|---|---|
| planning | Claude Code | - |
| review | Claude Code | Command Code |

## Degraded Agents

| Agent | Routing Status | Cooldown Until | Reason |
|---|---|---|---|
| OpenAI Codex CLI | permission-denied | 2026-07-01T00:15:00+00:00 | CLI exists but OS permissions blocked execution. |

## Task Routing

| Task | Preferred Ready | Fallback Policy |
|---|---|---|
| planning | claude | any ready agent with docs or long_task hints |
| review | claude, cmdc | any ready agent with review hints |
| verification | claude, cmdc | any ready agent with verification, review, or docs hints |

## Department View

| Department | Preferred Order | Ready Workers |
|---|---|---|
| planning | codex, claude | claude |
| review | claude, codex, cmdc | claude, cmdc |
| verification | codex, claude, cmdc | claude, cmdc |
| fallback | - | cmdc, claude |

## Task Contracts

| Task | Department | Role | Risk | Mode | Contract Status |
|---|---|---|---|---|---|
| TASK-123 | engineering | implementer | medium | warn | valid |

## Task Status

| Task | Current Status | Last Transition | Owner | Worker |
|---|---|---|---|---|
| TASK-123 | passed | passed | codex | cmdc |

## Scope Guard

| Task | Status | Evidence |
|---|---|---|
| TASK-123 | PASS | inside scope: src/together/routing.py, tests/test_routing.py |

## File Policy Validation

| Task | Allowed Files | Denied Files | Evidence |
|---|---|---|---|
| TASK-123 | PASS | PASS | allowed_matched: src/together/routing.py, tests/test_routing.py |

## Verification Results

| Task | Status | Verifier | Violations |
|---|---|---|---|
| TASK-123 | PASS | codex | - |

## Quality Gates

| Task | Status | Risk | Blockers |
|---|---|---|---|
| TASK-123 | PASS | medium | - |

## Merge Decisions

| Task | Decision | Authority | Reason |
|---|---|---|---|
| TASK-123 | MERGE | codex | verification passed, quality gate passed, task passed |

## Department Dashboard

| Department | Task Count | Status Distribution |
|---|---:|---|
| engineering | 1 | passed:1 |

## Last Known Good

- Timestamp: `2026-07-01T00:00:00+00:00`
- Ready Agents: 2
- Broken Agents: 1
- Degraded Agents: 1
- Workers: Command Code, Claude Code
