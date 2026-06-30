# Ranking

Ranking is heuristic, not benchmark-based.

Score drivers:
- readiness status
- capability fit
- likely task fit from static profiles
- diversity value relative to the currently selected model or agent

Preferred order:
1. `ready` and capability-aligned
2. `ready` but generic
3. `unknown` with promising profile
4. `installed-but-not-configured`
5. `installed-but-failing`
6. `not-found`

When two workers tie:
- prefer the one with narrower scope and lower coordination cost
- for review/verification, prefer a different model family than implementation
