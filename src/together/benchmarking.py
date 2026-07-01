from __future__ import annotations

import json
import subprocess
import time
from datetime import datetime, timezone
from pathlib import Path

from .contracts import _parse_simple_yaml
from .file_policy import evaluate_file_policy
from .git_tools import list_changed_files, normalize_repo_path, resolve_git_root, run_git
from .task_validation import validate_task

BENCHMARK_REQUIRED_FIELDS = {
    "task_id",
    "prompt",
    "repo_setup",
    "expected_files",
    "denied_files",
    "success_criteria",
    "test_command",
}

BENCHMARK_DIRS = ("tasks", "results", "reports")


def now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()


def build_excluded_paths(repo_root: Path, task_path: Path, task_id: str) -> set[str]:
    task_relative = normalize_repo_path(str(task_path.resolve().relative_to(repo_root.resolve())))
    return {
        task_relative,
        f".together/tasks/{task_id}.contract.yaml",
        f".together/tasks/{task_id}.status.json",
        f".together/tasks/{task_id}.verification.json",
        f".together/tasks/{task_id}.quality.json",
        f".together/tasks/{task_id}.merge.json",
    }


def filter_changed_files(changed_files: list[str], excluded_paths: set[str]) -> list[str]:
    return [path for path in changed_files if normalize_repo_path(path) not in excluded_paths and not normalize_repo_path(path).startswith("benchmarks/results/") and not normalize_repo_path(path).startswith("benchmarks/reports/")]


def ensure_benchmark_dirs(repo_root: Path) -> dict[str, Path]:
    root = repo_root / "benchmarks"
    paths = {name: root / name for name in BENCHMARK_DIRS}
    for path in paths.values():
        path.mkdir(parents=True, exist_ok=True)
    return paths


def load_benchmark_task(path: Path) -> dict:
    task = _parse_simple_yaml(path.read_text(encoding="utf-8"))
    normalized = dict(task)
    for field in ("expected_files", "denied_files", "success_criteria"):
        value = normalized.get(field, [])
        if isinstance(value, str):
            normalized[field] = [value]
        elif value is None:
            normalized[field] = []
        else:
            normalized[field] = list(value)
    return normalized


def validate_benchmark_task(task: dict) -> list[str]:
    errors: list[str] = []
    missing = BENCHMARK_REQUIRED_FIELDS - set(task)
    if missing:
        errors.append(f"missing fields: {sorted(missing)}")
    for field in ("expected_files", "denied_files", "success_criteria"):
        if not isinstance(task.get(field, []), list):
            errors.append(f"{field} must be a list")
    if not str(task.get("test_command", "")).strip():
        errors.append("test_command must not be empty")
    return errors


def measure_loc_changed(repo_root: Path, *, staged: bool = False, base: str | None = None) -> int:
    if base:
        proc = run_git(["diff", "--numstat", f"{base}...HEAD"], repo_root)
    elif staged:
        proc = run_git(["diff", "--numstat", "--cached"], repo_root)
    else:
        proc = run_git(["diff", "--numstat"], repo_root)
    if proc.returncode != 0:
        return 0

    total = 0
    tracked_files = set()
    for line in proc.stdout.splitlines():
        parts = line.split("\t")
        if len(parts) < 3:
            continue
        added, deleted, filename = parts[0], parts[1], parts[2]
        tracked_files.add(normalize_repo_path(filename))
        if added.isdigit():
            total += int(added)
        if deleted.isdigit():
            total += int(deleted)

    if base is None and not staged:
        untracked = run_git(["ls-files", "--others", "--exclude-standard"], repo_root)
        if untracked.returncode == 0:
            for line in untracked.stdout.splitlines():
                path = normalize_repo_path(line)
                if not path or path in tracked_files:
                    continue
                file_path = repo_root / path
                if file_path.exists() and file_path.is_file():
                    try:
                        total += len(file_path.read_text(encoding="utf-8", errors="replace").splitlines())
                    except OSError:
                        continue
    return total


def run_test_command(command: str, cwd: Path) -> dict:
    started = time.perf_counter()
    proc = subprocess.run(
        command,
        cwd=cwd,
        shell=True,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        check=False,
    )
    duration = time.perf_counter() - started
    status = "PASS" if proc.returncode == 0 else "FAIL"
    output = ((proc.stdout or "") + ("\n" + proc.stderr if proc.stderr else "")).strip()
    return {
        "status": status,
        "command": command,
        "exit_code": proc.returncode,
        "duration_seconds": round(duration, 4),
        "output": output[:4000],
    }


def build_contract_from_benchmark(task: dict, repo_root: Path) -> Path:
    tasks_dir = repo_root / ".together" / "tasks"
    tasks_dir.mkdir(parents=True, exist_ok=True)
    contract_path = tasks_dir / f"{task['task_id']}.contract.yaml"
    lines = [
        f"task_id: {task['task_id']}",
        f"title: {task['prompt']}",
        "department: engineering",
        "role: implementer",
        "owner: codex",
        "scope:",
    ]
    expected_files = task.get("expected_files", [])
    if expected_files:
        for item in expected_files:
            lines.append(f"  - {item}")
    else:
        lines.append("  - src/*")
    lines.extend(["allowed_files:"])
    if expected_files:
        for item in expected_files:
            lines.append(f"  - {item}")
    lines.extend(["denied_files:"])
    for item in task.get("denied_files", []):
        lines.append(f"  - {item}")
    lines.extend(["deliverables:"])
    for item in task.get("success_criteria", []):
        lines.append(f"  - {item}")
    lines.extend(["success_criteria:"])
    for item in task.get("success_criteria", []):
        lines.append(f"  - {item}")
    lines.extend(
        [
            "reviewer_required: false",
            "verification_required: true",
            "merge_authority: codex",
            "risk_level: low",
            "enforcement_mode: strict",
            "unknown_files_policy: needs_review",
        ]
    )
    contract_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return contract_path


def build_token_usage(manual_total_tokens: int | None = None) -> dict:
    if manual_total_tokens is None:
        return {
            "status": "unavailable",
            "prompt_tokens": None,
            "completion_tokens": None,
            "total_tokens": None,
            "manual_total_tokens": None,
        }
    return {
        "status": "manual",
        "prompt_tokens": None,
        "completion_tokens": None,
        "total_tokens": manual_total_tokens,
        "manual_total_tokens": manual_total_tokens,
    }


def write_markdown_report(report_path: Path, result: dict) -> None:
    metrics = result["metrics"]
    lines = [
        "# Benchmark Result",
        "",
        f"- Task: `{result['task_id']}`",
        f"- Mode: `{result['mode']}`",
        f"- Time: `{metrics['duration_seconds']}` s",
        f"- Files Changed: `{metrics['files_changed']}`",
        f"- LOC Changed: `{metrics['loc_changed']}`",
        f"- Tests: `{metrics['tests']['status']}`",
        f"- Scope Violations: `{metrics['scope_violations']}`",
        f"- Quality Gate: `{metrics['quality_gate_result'] or 'unavailable'}`",
        f"- Merge Decision: `{metrics['merge_decision'] or 'unavailable'}`",
        "",
    ]
    report_path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def write_result_artifacts(repo_root: Path, result: dict) -> dict[str, str]:
    dirs = ensure_benchmark_dirs(repo_root)
    stem = f"{result['task_id']}.{result['mode']}"
    result_path = dirs["results"] / f"{stem}.json"
    report_path = dirs["reports"] / f"{stem}.md"
    result_path.write_text(json.dumps(result, indent=2), encoding="utf-8")
    write_markdown_report(report_path, result)
    return {"result_json": str(result_path), "report_markdown": str(report_path)}


def benchmark_task(
    task_path: Path,
    *,
    mode: str,
    cwd: Path,
    staged: bool = False,
    base: str | None = None,
    write_result: bool = False,
    manual_total_tokens: int | None = None,
) -> tuple[dict, int]:
    task = load_benchmark_task(task_path)
    errors = validate_benchmark_task(task)
    if errors:
        return {"error": "invalid-benchmark-task", "task_path": str(task_path), "task_errors": errors}, 2

    repo_root, repo_error = resolve_git_root(cwd)
    if repo_root is None:
        return {"error": repo_error, "message": "Benchmark must run inside a git repository."}, 2

    task_path = task_path.resolve()
    excluded_paths = build_excluded_paths(repo_root, task_path, task["task_id"])

    started = time.perf_counter()
    changed_payload, diff_error = list_changed_files(repo_root, staged=staged, base=base)
    if diff_error:
        return diff_error, 2
    changed_files = filter_changed_files(changed_payload["changed_files"], excluded_paths)
    changed_payload = dict(changed_payload)
    changed_payload["changed_files"] = changed_files
    tests = run_test_command(str(task["test_command"]), repo_root)
    loc_changed = measure_loc_changed(repo_root, staged=staged, base=base)
    scope_policy = evaluate_file_policy(
        {
            "allowed_files": task.get("expected_files", []),
            "denied_files": task.get("denied_files", []),
            "unknown_files_policy": "needs_review",
            "enforcement_mode": "strict" if mode == "together" else "warn",
        },
        changed_files,
        enforcement_mode="strict" if mode == "together" else "warn",
        unknown_files_policy="needs_review",
    )
    scope_violations = len(scope_policy["denied_matched"]) + len(scope_policy["unknown_matched"])

    quality_gate_result = None
    merge_decision = None
    task_validation = None
    together_artifacts: dict[str, str] = {}
    if mode == "together":
        contract_path = build_contract_from_benchmark(task, repo_root)
        task_validation, validation_code = validate_task(
            contract_path,
            cwd=repo_root,
            changed_files=changed_files,
            mode="strict",
            write_artifacts=True,
        )
        quality_gate_result = task_validation["quality_gate"]["status"]
        merge_decision = task_validation["merge_decision"]["decision"]
        together_artifacts = {
            "contract": str(contract_path),
            "status": str(repo_root / ".together" / "tasks" / f"{task['task_id']}.status.json"),
            "verification": str(repo_root / ".together" / "tasks" / f"{task['task_id']}.verification.json"),
            "quality": str(repo_root / ".together" / "tasks" / f"{task['task_id']}.quality.json"),
            "merge": str(repo_root / ".together" / "tasks" / f"{task['task_id']}.merge.json"),
        }
        if validation_code not in {0, 1}:
            return task_validation, validation_code

    duration_seconds = round(time.perf_counter() - started, 4)
    result = {
        "task_id": task["task_id"],
        "mode": mode,
        "generated_at": now_iso(),
        "task": task,
        "metrics": {
            "duration_seconds": duration_seconds,
            "files_changed": len(changed_files),
            "loc_changed": loc_changed,
            "tests": tests,
            "scope_violations": scope_violations,
            "quality_gate_result": quality_gate_result,
            "merge_decision": merge_decision,
            "human_rating": None,
            "token_usage": build_token_usage(manual_total_tokens),
        },
        "changed_files": changed_payload,
        "scope_policy": scope_policy,
        "task_validation": task_validation,
        "artifacts": together_artifacts,
    }

    result_paths: dict[str, str] = {}
    if write_result:
        result_paths = write_result_artifacts(repo_root, result)
    result["benchmark_artifacts"] = result_paths
    return result, 0


def compare_results(codex_only: dict, together: dict) -> str:
    task_id = together.get("task_id") or codex_only.get("task_id")
    rows = [
        ("duration_seconds", codex_only["metrics"]["duration_seconds"], together["metrics"]["duration_seconds"]),
        ("files_changed", codex_only["metrics"]["files_changed"], together["metrics"]["files_changed"]),
        ("loc_changed", codex_only["metrics"]["loc_changed"], together["metrics"]["loc_changed"]),
        ("tests", codex_only["metrics"]["tests"]["status"], together["metrics"]["tests"]["status"]),
        ("scope_violations", codex_only["metrics"]["scope_violations"], together["metrics"]["scope_violations"]),
        ("quality_gate_result", codex_only["metrics"]["quality_gate_result"], together["metrics"]["quality_gate_result"]),
        ("merge_decision", codex_only["metrics"]["merge_decision"], together["metrics"]["merge_decision"]),
        ("human_rating", codex_only["metrics"]["human_rating"], together["metrics"]["human_rating"]),
        ("token_usage", codex_only["metrics"]["token_usage"]["status"], together["metrics"]["token_usage"]["status"]),
    ]
    lines = [
        "# Benchmark Comparison",
        "",
        f"Task: `{task_id}`",
        "",
        "| Metric | codex_only | together |",
        "|---|---|---|",
    ]
    for metric, left, right in rows:
        lines.append(f"| {metric} | {left} | {right} |")
    lines.append("")
    return "\n".join(lines)
