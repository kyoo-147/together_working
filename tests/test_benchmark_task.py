from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "benchmark-task.py"


def run(cmd: list[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, check=False)


def init_repo(tmp_path: Path) -> Path:
    run(["git", "init"], tmp_path)
    run(["git", "config", "user.name", "Together Test"], tmp_path)
    run(["git", "config", "user.email", "together@example.com"], tmp_path)
    (tmp_path / "src").mkdir()
    (tmp_path / "src" / "app.py").write_text("print('v1')\n", encoding="utf-8")
    run(["git", "add", "src/app.py"], tmp_path)
    run(["git", "commit", "-m", "init"], tmp_path)
    return tmp_path


def write_task(path: Path) -> Path:
    task = path / "TASK-001.yaml"
    task.write_text(
        "\n".join(
            [
                "task_id: TASK-001",
                "prompt: Update app output",
                "repo_setup: clean git repo",
                "expected_files:",
                "  - src/app.py",
                "denied_files:",
                "  - secrets.txt",
                "success_criteria:",
                "  - test command passes",
                "test_command: python -m py_compile src/app.py",
            ]
        ),
        encoding="utf-8",
    )
    return task


def test_benchmark_task_codex_only_outputs_result_json(tmp_path: Path) -> None:
    repo = init_repo(tmp_path)
    task = write_task(repo)
    (repo / "src" / "app.py").write_text("print('v2')\n", encoding="utf-8")

    proc = run([sys.executable, str(SCRIPT), str(task), "--mode", "codex_only"], repo)

    assert proc.returncode == 0
    payload = json.loads(proc.stdout)
    assert payload["mode"] == "codex_only"
    assert payload["metrics"]["files_changed"] == 1
    assert payload["metrics"]["tests"]["status"] == "PASS"
    assert payload["metrics"]["token_usage"]["status"] == "unavailable"


def test_benchmark_task_together_writes_enforcement_artifacts(tmp_path: Path) -> None:
    repo = init_repo(tmp_path)
    task = write_task(repo)
    (repo / "src" / "app.py").write_text("print('v2')\n", encoding="utf-8")

    proc = run([sys.executable, str(SCRIPT), str(task), "--mode", "together", "--write-result"], repo)

    assert proc.returncode == 0
    payload = json.loads(proc.stdout)
    assert payload["mode"] == "together"
    assert payload["metrics"]["quality_gate_result"] == "PASS"
    assert payload["metrics"]["merge_decision"] == "MERGE"
    assert (repo / ".together" / "tasks" / "TASK-001.verification.json").exists()
    assert (repo / "benchmarks" / "results").exists()

