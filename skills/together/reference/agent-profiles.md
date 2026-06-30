# Agent Profiles

Static profiles describe likely strengths. Runtime discovery confirms whether an agent is actually usable.

## Codex

Best for:
- planning
- decomposition
- integration
- orchestration
- final review

Weakness:
- should not be the default worker for every subtask when better specialized local agents are available

## Command Code (`cmdc`)

Best for:
- model-rich routing
- long-context synthesis
- vision-capable coding models
- implementation, review, and synthesis with model selection

Weakness:
- may vary by configured model access and account state

## Antigravity (`agy`)

Best for:
- bounded worker tasks
- read-only audits
- scoped implementation slices
- sidecar execution with clear scope

Weakness:
- less suitable as the sole control plane

## Claude CLI / Claude Code

Best for:
- reasoning-heavy specs
- review and critique
- long-form synthesis when available

Weakness:
- availability/config may vary across installs

## Amp

Treat as:
- generic worker until discovery plus local profile data says otherwise

## Unknown agents

If an agent is discovered but has no strong profile:
- tag it `generic-worker`
- keep its ranking below known specialists unless runtime evidence is strong
