from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "validate-task.py"


def run(cmd: list[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, check=False)


def init_repo(tmp_path: Path) -> Path:
    run(["git", "init"], tmp_path)
    run(["git", "config", "user.name", "Together Test"], tmp_path)
    run(["git", "config", "user.email", "together@example.com"], tmp_path)
    (tmp_path / "src").mkdir()
    (tmp_path / "src" / "module.py").write_text("print('v1')\n", encoding="utf-8")
    run(["git", "add", "src/module.py"], tmp_path)
    run(["git", "commit", "-m", "init"], tmp_path)
    return tmp_path


def write_contract(repo: Path, body: str) -> Path:
    path = repo / "TASK-123.contract.yaml"
    path.write_text(body, encoding="utf-8")
    return path


def test_validate_task_warn_mode_reports_violation_but_exits_zero(tmp_path: Path) -> None:
    repo = init_repo(tmp_path)
    (repo / "README.md").write_text("changed\n", encoding="utf-8")
    contract = write_contract(
        repo,
        "\n".join(
            [
                "task_id: TASK-123",
                "title: Test",
                "department: engineering",
                "role: implementer",
                "owner: codex",
                "scope:",
                "  - src/*.py",
                "allowed_files:",
                "  - src/*.py",
                "denied_files:",
                "  - secrets/*",
                "deliverables:",
                "  - update module",
                "success_criteria:",
                "  - checks pass",
                "reviewer_required: false",
                "verification_required: true",
                "merge_authority: codex",
                "risk_level: low",
                "enforcement_mode: warn",
            ]
        ),
    )

    proc = run([sys.executable, str(SCRIPT), str(contract), "--mode", "warn"], repo)

    assert proc.returncode == 0
    payload = json.loads(proc.stdout)
    assert payload["scope_guard"]["status"] == "NEEDS_REVIEW"
    assert payload["quality_gate"]["status"] == "NEEDS_REVIEW"


def test_validate_task_strict_mode_writes_artifacts_and_fails(tmp_path: Path) -> None:
    repo = init_repo(tmp_path)
    (repo / "README.md").write_text("changed\n", encoding="utf-8")
    contract = write_contract(
        repo,
        "\n".join(
            [
                "task_id: TASK-123",
                "title: Test",
                "department: engineering",
                "role: implementer",
                "owner: codex",
                "scope:",
                "  - src/*.py",
                "allowed_files:",
                "  - src/*.py",
                "denied_files:",
                "  - secrets/*",
                "deliverables:",
                "  - update module",
                "success_criteria:",
                "  - checks pass",
                "reviewer_required: true",
                "verification_required: true",
                "merge_authority: codex",
                "risk_level: medium",
                "enforcement_mode: strict",
            ]
        ),
    )

    proc = run([sys.executable, str(SCRIPT), str(contract), "--mode", "strict", "--write-artifacts"], repo)

    assert proc.returncode == 1
    payload = json.loads(proc.stdout)
    assert payload["merge_decision"]["decision"] == "REJECT"
    assert (repo / ".together" / "tasks" / "TASK-123.verification.json").exists()
    assert (repo / ".together" / "tasks" / "TASK-123.quality.json").exists()


def test_validate_task_invalid_contract_returns_two(tmp_path: Path) -> None:
    repo = init_repo(tmp_path)
    contract = write_contract(repo, "task_id: TASK-123\n")

    proc = run([sys.executable, str(SCRIPT), str(contract), "--mode", "warn"], repo)

    assert proc.returncode == 2
    payload = json.loads(proc.stdout)
    assert payload["error"] == "invalid-contract"
