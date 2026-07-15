# Product

Together is a local-first Codex operator console for observable AI worker teams.

The product lets a developer keep working in Codex while a terminal-native UI shows what is happening behind the scenes: task contracts, agent readiness, route decisions, live PTY output, verification gates, review state, settings, and attention items.

## Current Identity

- Codex remains the preferred planner, coordinator, verifier, and integrator.
- Together is the local control plane around Codex-led work.
- Worker agents are replaceable, scoped, and observable.
- The daemon owns state, policy, routing, review, and verification.
- The TUI is a thin client that renders daemon events and sends commands.

## Product Surface

- `together` opens the Monitor + Chat Dock terminal UI.
- `together daemon` runs the local event and worker daemon.
- `together chat --source codex-app "<text>"` bridges Codex-sourced requests into the daemon.
- `skills/together/scripts/submit-chat.py` is the skill bridge helper.
- `together status`, `doctor`, `settings`, `proposal`, and review commands expose the same state through CLI flows.

## Core Layers

- agent discovery and readiness
- deterministic routing and fallback
- task contracts and scoped execution
- PTY-backed live worker output
- proposal confirmation before mutation
- review and approval gates
- verification before approval
- local settings and theme presets
- release and install helpers for Windows-first distribution

## Current Stage

Together is in active dogfooding. The CLI/TUI product is running and being tested inside real developer sessions to improve:

- operator UX
- route visibility
- degraded-agent fallback
- scoped task creation
- review and approval flows
- verification evidence
- benchmark and performance measurement

It is not yet a distributed enterprise scheduler, centralized policy server, or complete multi-machine workforce platform.

## Long-Term Vision

Together can grow into an AI Department Operating System: small contexts, specialized workers, explicit contracts, verified outcomes, and auditable integration across local and team AI workflows.
