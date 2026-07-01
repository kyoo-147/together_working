<p align="center">
  <img src="docs/assets/generated/together-logo.png" alt="Together logo" width="200">
</p>

<h1 align="center">🍠 together</h1>

<p align="center">
  AI Department for local agent teams.
</p>

<p align="center">
  Together manages agents. It does not replace them.
</p>

<p align="center">
  <a href="https://github.com/kyoo-147/together_working/stargazers">
    <img src="https://img.shields.io/github/stars/kyoo-147/together_working?style=flat-square" alt="GitHub stars">
  </a>
  <a href="https://github.com/kyoo-147/together_working/blob/main/LICENSE">
    <img src="https://img.shields.io/github/license/kyoo-147/together_working?style=flat-square" alt="License">
  </a>
  <a href="https://github.com/kyoo-147/together_working/commits/main">
    <img src="https://img.shields.io/github/last-commit/kyoo-147/together_working?style=flat-square" alt="Last commit">
  </a>
  <img src="https://img.shields.io/badge/skill-system--level-blue?style=flat-square" alt="System level skill">
</p>

<p align="center">
  <a href="#why-this-exists"><strong>Why</strong></a>
  |
  <a href="#how-system-works"><strong>Architecture</strong></a>
  |
  <a href="#install"><strong>Install</strong></a>
  |
  <a href="#commands"><strong>Commands</strong></a>
  |
  <a href="#repo-map"><strong>Docs</strong></a>
</p>

<p align="center">
  One potato. Many workers. Small tasks. Big outcomes.
</p>

<p align="center">
  AI departments, not AI chats.
</p>

<p align="center">
  Small contexts. Right workers. Clear boundaries.
</p>

<p align="center">
  <img src="docs/assets/generated/together-hero.png" alt="Together hero" width="100%">
</p>

## Sweet Potato Architecture

One potato: Codex

Many potato workers: agents

Small potato tasks

Big potato outcome

Why use one giant context when small contexts do trick.

- Give each worker only context they need.
- Route task to right worker instead of growing one chat forever.
- Keep verification separate from implementation.
- Spend fewer tokens. Get cleaner output. Keep merge control clear.

## Why This Exists

Most agent setups break same way:

- one agent sees too much
- one context grows too long
- routing hidden in prompts
- verification weak
- merge authority unclear

`together` exists to stop that.

It turns installed agents into department workers with clear work boundaries, clear routing, clear verification, clear final control.

## What Together Is

Together is:

- discovery
- registry
- routing
- governance
- verification
- merge control

Together is not:

- another chat wrapper
- benchmark suite
- giant autonomous scheduler
- claim that one model best at everything

## What You Get

- discovery of supported agent providers
- scan of installed CLIs on current machine
- lightweight ready-state health checks
- capability-based department routing
- verification before merge
- failover and degraded-mode handling
- local cache and operator-readable reports

## Before / After

Before:

- one giant chat
- one giant context
- one worker sees too much
- verification happens late or not at all

After:

- department workflow
- small scoped tasks
- right worker for right job
- Codex verifies and integrates final output

## How System Works

<p align="center">
  <img src="docs/assets/generated/together-architecture-overview.png" alt="Together architecture overview" width="100%">
</p>

```text
Known Providers
↓
Installed CLIs
↓
Ready Agents
↓
Departments
↓
Routing
↓
Verification
↓
Output
```

Flow meaning:

- Known Providers: curated ecosystem Together knows how to classify.
- Installed CLIs: commands actually found on machine.
- Ready Agents: installed workers that pass light health checks.
- Departments: planning, research, vision, engineering, review, verification, fallback.
- Routing: capability-first assignment with failover.
- Verification: output, scope, and routing sanity checks before integration.

## How It Works

- load known provider registry
- detect installed CLI workers on PATH
- run lightweight health checks
- classify ready, broken, and degraded workers
- route work by department and capability hints
- fail over when preferred worker is unavailable
- let Codex verify, integrate, and decide merge outcome

## Registry / Ready-State Model

<p align="center">
  <img src="docs/assets/generated/together-registry-ready-state.png" alt="Together registry and ready-state model" width="100%">
</p>

This is one of Together's core ideas:

- `Known Providers` means agent ecosystem Together understands.
- `Installed CLIs` means commands found on this machine.
- `Ready Agents` means installed workers that pass lightweight health checks.

Health checks stay cheap:

- command exists on PATH
- version or help command runs
- obvious auth or config failures surfaced
- immediate permission-denied state detected

## Department Model

- Planning: `codex`, `claude`
- Research: `gemini`, `agy`, `claude`
- Vision: `gemini`, `cmdc`, `kimi-code`
- Engineering: `codex`, `cmdc`, `amp`, `opencode`
- Review: `claude`, `codex`, `cmdc`
- Verification: `codex`, `claude`, `cmdc`
- Fallback: any ready worker with matching capability hints

Codex role:

- planner
- coordinator
- verifier
- integrator
- merge authority

## Work Governance

<p align="center">
  <img src="docs/assets/generated/together-governance-control.png" alt="Together work governance and control" width="100%">
</p>

Agents must not do everything.

Agents should only do assigned work inside assigned scope.

Verification checks contract and boundaries.

Codex decides merge.

```text
Task
↓
Assignment
↓
Execution
↓
Review
↓
Verification
↓
Merge Decision
```

### Task Contract

Each meaningful task should carry contract:

- task id
- scope
- allowed files
- denied files
- deliverables
- success criteria
- reviewer required
- verification required

Goal:

- worker changes only assigned scope
- reviewer checks quality
- verification checks compliance

### Permission Model

- Observer: read, search, analyze
- Researcher: gather, compare, summarize
- Implementer: modify assigned scope only
- Reviewer: review, approve, reject
- Integrator: merge, final decision

Codex defaults to Integrator.

## Workflow Control

<p align="center">
  <img src="docs/assets/generated/together-department-pipeline.png" alt="Together department pipeline" width="100%">
</p>

Together runs work through departments, not one endless conversation.

```text
Request
↓
Planning
↓
Research
↓
Vision
↓
Engineering
↓
Review
↓
Verification
↓
Integration
↓
Output
```

Verification outcomes:

- `PASS`
- `REJECT`
- `NEEDS_REVIEW`

Verification checks:

- scope compliance
- allowed files
- denied files
- acceptance criteria
- routing correctness
- architecture compliance

## Failover / Degraded Mode

<p align="center">
  <img src="docs/assets/generated/together-failover-degraded-mode.png" alt="Together failover and degraded mode" width="100%">
</p>

When a preferred worker fails:

- mark degraded
- store recent failure state
- start cooldown
- route to healthy fallback worker
- probe recovery after cooldown
- return to preferred worker when healthy again

Codex still verifies and integrates final result.

## Operations Model

<p align="center">
  <img src="docs/assets/generated/together-operations-artifacts.png" alt="Together operations and artifacts" width="100%">
</p>

Together keeps lightweight operator memory:

- registry cache
- last known good snapshot
- runtime failover state
- provider override
- human-readable report

Generated artifacts:

- `.together/cache/agent-registry.json`
- `.together/cache/last-known-good.json`
- `.together/cache/runtime-state.json`
- `.together/reports/agent-report.md`
- `.together/providers.override.json`

Health states:

- `not-installed`
- `ready`
- `auth-required`
- `permission-denied`
- `installed-but-broken`
- `installed-unknown`

Runtime controls:

- machine-local provider override
- cooldown for recently failed agents
- degraded-agent tracking
- last-known-good fallback context

## Install

Install full skill:

```bash
npx skills add https://github.com/kyoo-147/together_working
```

Install only main skill:

```bash
npx skills add https://github.com/kyoo-147/together_working --skill "together"
```

## Commands

Scan machine:

```bash
python skills/together/scripts/discover-agents.py --format table
```

Write registry cache:

```bash
python skills/together/scripts/write-registry.py
```

Write full operator snapshot:

```bash
python skills/together/scripts/doctor.py
```

Edit machine-local override:

```bash
notepad .together/providers.override.json
```

Override knobs:

- disable provider
- rank adjust
- disable or add capability hints
- reorder task preference
- reorder department preference
- change cooldown seconds

## Repo Map

- `skills/together/SKILL.md`
- `docs/architecture.md`
- `docs/governance.md`
- `docs/task-contract.md`
- `docs/permission-model.md`
- `docs/workflow-control.md`
- `docs/routing.md`
- `docs/registry.md`
- `docs/health-check.md`
- `docs/reporting.md`

## Links

- [GitHub Repo](https://github.com/kyoo-147/together_working)
- [Skill Entry](skills/together/SKILL.md)
- [Architecture Doc](docs/architecture.md)
- [Routing Doc](docs/routing.md)
- [Governance Doc](docs/governance.md)
- [Reporting Doc](docs/reporting.md)

## Limits

- registry is curated, not exhaustive
- capability hints are hints, not benchmark claims
- report is for operator clarity, not model ranking science
- failover memory is simple cooldown state, not distributed scheduler

## License

MIT
