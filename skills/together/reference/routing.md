# Routing

Two splits matter:

1. classify work
2. choose ready worker

Task hints:
- `vision`
- `backend`
- `frontend`
- `research`
- `review`
- `docs`
- `shell`
- `short_task`
- `long_task`
- `multi_file`

Department preference:
- Planning -> `codex`, `claude`
- Research -> `gemini`, `agy`, `claude`
- Vision -> `gemini`, `cmdc`, `kimi-code`
- Engineering -> `codex`, `cmdc`, `amp`, `opencode`
- Review -> `claude`, `codex`, `cmdc`

Routing law:
- ready first
- capability fit second
- fallback always

Codex still owns batch plan and final integration.
