# Benchmarking

Together benchmark harness compares two modes on same task:

- `codex_only`
- `together`

## Goal

Measure practical workflow difference, not model quality claims.

Metrics:

- time
- token usage if available
- files changed
- LOC changed
- tests pass or fail
- scope violations
- quality gate result
- merge decision
- human rating placeholder

If token data is unavailable, benchmark marks it as unavailable.

## Layout

```text
benchmarks/
  tasks/
  results/
  reports/
```

## Task Spec

Each benchmark task must define:

- `task_id`
- `prompt`
- `repo_setup`
- `expected_files`
- `denied_files`
- `success_criteria`
- `test_command`

Example:

```bash
examples/benchmarks/TASK-001.yaml
```

## Run One Benchmark

Codex only:

```bash
python scripts/benchmark-task.py examples/benchmarks/TASK-001.yaml --mode codex_only --write-result
```

Together:

```bash
python scripts/benchmark-task.py examples/benchmarks/TASK-001.yaml --mode together --write-result
```

## Compare Results

```bash
python scripts/benchmark-compare.py benchmarks/results/TASK-001.codex_only.json benchmarks/results/TASK-001.together.json
```

## Together Mode Behavior

Together mode will:

1. derive task contract
2. collect changed files
3. run task validation
4. write enforcement artifacts
5. write benchmark result JSON
6. write benchmark markdown report

## Notes

- benchmark harness does not fake token usage
- benchmark harness does not run an autonomous agent
- human rating is left as `null` placeholder for later review
