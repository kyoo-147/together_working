---
name: together
description: Use when a task is large enough to benefit from orchestration across local AI agent CLIs, capability-based routing, approval-gated execution batches, agent health checks, or structured fallback when workers hit errors, limits, or token exhaustion.
---

# together

## Overview

`together` is an always-on orchestration skill for large tasks.

Codex is the control plane:
- decompose the task
- classify required capabilities
- discover available local agent CLIs
- rank worker options
- propose execution batches
- ask for approval before real external execution
- integrate results
- handle fallbacks
- produce pause/resume reports when necessary

Codex does **not** treat `agy`, `cmdc`, `claude`, `amp`, or other local agent CLIs as magical black boxes. It discovers them, checks their health, and routes them by fitness for the task.

## When to Use

Use `together` automatically when a task is large enough that one or more of the following are true:
- multiple capability types are needed
- the task can be decomposed into independent or semi-independent slices
- a worker CLI may be better than Codex for part of the work
- there is value in model diversity for review, synthesis, or fallback
- long-running work may need structured pause/resume handling

Do not use `together` for tiny one-step tasks where orchestration overhead would dominate the work.

## Core Rules

1. Codex remains PM, architect, reviewer, and final integrator.
2. Route by capability first, not by favorite agent.
3. Before any real external execution batch, ask the user for approval.
4. Always run local agent discovery before first delegation in a session unless a fresh registry snapshot already exists.
5. Treat rate limits, token exhaustion, wait states, auth problems, and empty outputs as first-class routing failures.
6. If all workers fail or are limited, Codex takes over where feasible.
7. If Codex must stop too, write a structured resume report.

## Required Flow

1. Read [reference/routing.md](reference/routing.md)
2. Read [reference/agent-discovery.md](reference/agent-discovery.md)
3. If orchestration seems useful, run local discovery
4. Build a capability map for the task
5. Read the matching profiles from [reference/agent-profiles.md](reference/agent-profiles.md)
6. Propose an approval-gated execution batch
7. If approved, dispatch carefully and monitor failures
8. If dispatch fails, apply [reference/fallbacks.md](reference/fallbacks.md)
9. If work must pause, write a report using [reference/pause-resume.md](reference/pause-resume.md)

## Quick Reference

Capability buckets:
- `planning`
- `research-web`
- `vision`
- `code-implementation`
- `code-review`
- `verification`
- `tool-execution`
- `long-context-synthesis`

Status buckets:
- `ready`
- `installed-but-not-configured`
- `installed-but-failing`
- `unknown`
- `not-found`

## Scripts

Discover local agents:

```bash
python skills/together/scripts/discover-agents.py --format json
```

PowerShell wrapper:

```powershell
powershell -ExecutionPolicy Bypass -File skills/together/scripts/discover-agents.ps1 -Format table
```

## Important Constraint

External execution is approval-gated by default in this skill.

Even if an agent is healthy and highly ranked, Codex must not run the external batch until the user approves the proposed dispatch.
