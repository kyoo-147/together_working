# Routing

Routing rule:
- ready first
- fit second
- fallback always

Codex never disappears from loop.

Department order:
- Planning: `codex`, `claude`
- Research: `gemini`, `agy`, `claude`
- Vision: `gemini`, `cmdc`, `kimi-code`
- Engineering: `codex`, `cmdc`, `amp`, `opencode`
- Review: `claude`, `codex`, `cmdc`
- Verification: `codex`, `claude`, `cmdc`

Task routing chooses:
1. preferred ready provider
2. backup ready provider
3. Codex verification path

If no preferred provider is ready, fallback becomes any ready provider with matching hints.

Runtime failover adds:
- cooldown for recently failed agents
- degraded state tracking
- automatic return after cooldown expires
