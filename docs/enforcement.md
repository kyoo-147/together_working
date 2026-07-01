# Enforcement Engine

Together v0.5 moves governance from guidance to control.

## Goal

- define work boundaries before execution
- validate changed files against scope
- require machine-readable verification
- block integration when evidence is missing

## Core Artifacts

- `.together/tasks/*.contract.yaml`
- `.together/tasks/*.status.json`
- `.together/tasks/*.verification.json`
- `.together/tasks/*.quality.json`
- `.together/tasks/*.merge.json`

## Operator Commands

```bash
python scripts/changed-files.py --json
python scripts/validate-task.py examples/task-contract.example.yaml --mode warn
python scripts/validate-task.py examples/task-contract.example.yaml --mode strict --write-artifacts
```

## Runtime Order

1. Codex creates task contract.
2. Worker executes inside assigned scope.
3. Scope guard checks changed files.
4. Verification writes structured result.
5. Quality gate decides if integration may continue.
6. Codex makes final merge decision.

## Modes

- `warn`: report violations, hold integration for review
- `strict`: reject out-of-scope or policy-breaking work

## Enforced Now

- contract schema validation
- changed-file capture from git
- scope guard
- allowed and denied file policy
- machine-readable verification result
- quality gate
- merge decision artifact
- report sections for task enforcement

## Not Included Yet

- autonomous task runner
- automatic worker sandbox execution
- automatic commit or PR creation
- distributed scheduler
