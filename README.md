# together

`together` is a system-style Agent Skill pack for Codex-family workflows.

It turns Codex into a control plane:
- Codex acts as PM, planner, decomposer, reviewer, and final integrator
- local agent CLIs such as `agy`, `cmdc`, `claude`, `codex`, `amp`, and others become worker candidates
- routing happens in two layers:
  - capability -> what kind of work is needed
  - agent + model + fallback -> who should do it

The first release focuses on:
- lightweight local agent discovery
- health checks for installed CLIs
- static capability profiles and ranking
- approval-gated execution batches
- fallback handling for model errors, agent failures, timeouts, and rate/token limits
- pause/resume reports when work must stop cleanly

## Install

The `npx skills add` ecosystem scans the `skills/` folder in this repo.

```bash
npx skills add https://github.com/kyoo-147/together_working
```

Install the main skill explicitly:

```bash
npx skills add https://github.com/kyoo-147/together_working --skill "together"
```

## What the skill does

- detects local AI agent CLIs installed on the machine
- checks whether they are ready, broken, or not configured
- classifies them by capabilities such as `vision`, `code`, `review`, `research-web`, and `long-context-synthesis`
- proposes which agent/model is best for a task
- asks for approval before dispatching any real external execution batch
- writes structured resume context if work must pause because every worker is blocked or limited

## Repository layout

- `skills/together/SKILL.md`: main entrypoint
- `skills/together/reference/`: routing, profiles, fallback, and approval docs
- `skills/together/scripts/`: local discovery and reporting scripts
- `skills/together/data/`: static agent profiles and routing presets
- `.claude-plugin/`: packaging metadata for skill ecosystems

## Local discovery

The discovery script is intentionally lightweight. It does not benchmark models.
It:
- finds known CLI commands on PATH
- runs low-cost commands such as `--help`, `--version`, `models`, or `status`
- assigns a status like `ready`, `installed-but-not-configured`, `installed-but-failing`, `unknown`, or `not-found`
- merges that runtime signal with static profiles to produce a local registry snapshot

Example:

```bash
python skills/together/scripts/discover-agents.py --format json
```

## Design constraints

- Codex remains the control plane
- external workers never run automatically without user approval
- fallback for rate limits and token exhaustion is first-class
- if all workers are blocked, Codex takes over if practical
- if Codex must also stop, the system writes a resume report with exact next steps

## License

MIT
