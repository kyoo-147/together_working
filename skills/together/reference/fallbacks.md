# Fallbacks

Treat these as first-class failures:
- rate limit reached
- token exhaustion
- forced wait state
- timeout
- auth/config failure
- empty output
- malformed output
- capability mismatch

## Fallback order

1. Same agent, different model
2. Different agent, same capability
3. Reduced scope batch
4. Codex takeover
5. Controlled pause with resume report

## Limit handling

Use a reactive-first strategy:
- if usage data is exposed, consider it
- otherwise dispatch normally and react when the failure appears

When a worker reports a wait or limit:
- mark current route degraded
- do not keep hammering the same route
- promote the next viable route

If all workers are blocked:
- Codex handles the task if practical
- otherwise pause and write a resume report
