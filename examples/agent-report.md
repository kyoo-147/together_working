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

## Ready Agents

| Agent | Best Hints | Departments |
|---|---|---|
| Command Code | vision, backend, frontend, review | vision, engineering, review |
| Claude Code | research, review, verification, docs | planning, research, review |

## Broken Agents

| Agent | Status | Reason |
|---|---|---|
| OpenAI Codex CLI | permission-denied | CLI exists but OS permissions blocked execution. |

## Best By Task

| Task | Best Worker | Alternatives |
|---|---|---|
| planning | Claude Code | - |
| review | Claude Code | Command Code |
| verification | Claude Code | Command Code |

## Last Known Good

- Timestamp: `2026-07-01T00:00:00+00:00`
- Ready Agents: 2
- Broken Agents: 1
- Degraded Agents: 1
- Workers: Command Code, Claude Code

