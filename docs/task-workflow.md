# Task Workflow

Together v0.5 can validate task boundaries end-to-end.

## Flow

1. create contract
2. assign worker
3. make changes
4. collect changed files
5. validate task
6. inspect artifacts
7. render report
8. Codex decides merge

## Example Commands

```bash
python scripts/changed-files.py --json
python scripts/validate-task.py examples/task-contract.example.yaml --mode warn
python scripts/validate-task.py examples/task-contract.example.yaml --mode strict --write-artifacts
python skills/together/scripts/render-report.py
```

## Artifacts

When `--write-artifacts` is used, Together writes:

- `.together/tasks/<task_id>.status.json`
- `.together/tasks/<task_id>.verification.json`
- `.together/tasks/<task_id>.quality.json`
- `.together/tasks/<task_id>.merge.json`

These files are runtime state. Do not commit them.

## Warn vs Strict

- `warn`: report violations, write artifacts, return exit `0` unless input/schema invalid
- `strict`: reject boundary violations, missing required review, missing codex approval, return exit `1`
