# Quality Gate

Quality gate runs before merge decision.

## Inputs

- task contract
- verification result
- review status
- task status
- risk level

## Risk Policy

- `low`: verification
- `medium`: review + verification
- `high`: review + verification + codex approval

## Outcomes

- `PASS`
- `REJECT`
- `NEEDS_REVIEW`

## Blocking Rules

- missing verification fails gate
- rejected verification fails gate
- non-passing acceptance criteria blocks integration
- high-risk work waits for Codex approval
