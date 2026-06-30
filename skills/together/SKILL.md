---
name: together
description: Use when a task is large enough to benefit from agent-agnostic orchestration, local agent discovery, readiness checks, department-style routing, or structured fallback across multiple AI worker CLIs.
---

# together

## Overview

`together` turns Codex into orchestration layer.

Codex does:
- plan
- split work
- choose workers
- ask approval before external execution
- verify outputs
- merge final answer

Workers do scoped tasks only.

## Use when

Use for tasks that need one or more:
- multiple capability types
- long or multi-file execution
- model diversity for review
- fallback when one worker fails
- local agent readiness scan before delegation

Skip for tiny one-shot tasks.

## Core rules

1. Codex stays PM and final integrator.
2. Route by readiness first, then task fit.
3. Known Providers != Installed CLIs != Ready Agents.
4. Health checks must stay lightweight.
5. External execution stays approval-gated.
6. If all workers fail, Codex takes over if practical.
7. If work must pause, write resume context.

## Quick flow

1. Run registry doctor if current readiness unknown.
2. Read cache/report.
3. Map task into hints: `vision`, `backend`, `frontend`, `research`, `review`, `docs`, `shell`, `short_task`, `long_task`, `multi_file`.
4. Pick ready worker from department order.
5. Keep fallback path ready.
6. Integrate back into Codex.

## Commands

```bash
python skills/together/scripts/discover-agents.py --format table
python skills/together/scripts/write-registry.py
python skills/together/scripts/doctor.py
```

## References

- `reference/agent-discovery.md`
- `reference/routing.md`
- `reference/agent-profiles.md`
- `reference/local-registry-schema.md`
