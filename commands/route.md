# route

Purpose:
- pick best ready worker for a task or department

When to use:
- after scan
- when preferred worker is unavailable

Inputs:
- task hint
- current ready state

Outputs:
- preferred worker
- fallback path

Safety rules:
- readiness first
- capability fit second
- fallback always

Example:
- route `research` to `agy` or `claude` depending on readiness

