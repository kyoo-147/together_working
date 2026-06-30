# Routing

`together` uses two routing layers.

## Layer 1: Capability Routing

Classify the task by the work it needs, not by the tool you want to use.

Capability set:
- `planning`
- `research-web`
- `vision`
- `code-implementation`
- `code-review`
- `verification`
- `tool-execution`
- `long-context-synthesis`

A single user request may expand into multiple capabilities.

Examples:
- "analyze this screenshot and fix the page" -> `vision`, `planning`, `code-implementation`, `verification`
- "read docs and propose architecture" -> `research-web`, `long-context-synthesis`, `planning`
- "implement feature and have another model review it" -> `planning`, `code-implementation`, `code-review`, `verification`

## Layer 2: Execution Routing

After capability is known, choose:
- agent
- model
- fallback model
- fallback agent
- output contract
- verification path

Default preference:
- choose the narrowest capable worker
- prefer model diversity for review and verification
- avoid giving every step to the same agent/model pair

## Approval Gate

Before any real external batch, produce:
- objective
- capability
- recommended agent
- recommended model
- fallback chain
- expected output
- risk notes

Then ask the user to approve the batch.
