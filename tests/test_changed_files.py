from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "changed-files.py"


def run(cmd: list[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, check=False)


def init_repo(tmp_path: Path) -> Path:
    run(["git", "init"], tmp_path)
    run(["git", "config", "user.name", "Together Test"], tmp_path)
    run(["git", "config", "user.email", "together@example.com"], tmp_path)
    (tmp_path / "app.txt").write_text("one\n", encoding="utf-8")
    run(["git", "add", "app.txt"], tmp_path)
    run(["git", "commit", "-m", "init"], tmp_path)
    return tmp_path


def test_changed_files_json_reports_working_tree_changes(tmp_path: Path) -> None:
    repo = init_repo(tmp_path)
    (repo / "app.txt").write_text("two\n", encoding="utf-8")

    proc = run([sys.executable, str(SCRIPT), "--json"], repo)

    assert proc.returncode == 0
    payload = json.loads(proc.stdout)
    assert payload["mode"] == "working_tree"
    assert payload["changed_files"] == ["app.txt"]


def test_changed_files_supports_staged_mode(tmp_path: Path) -> None:
    repo = init_repo(tmp_path)
    (repo / "app.txt").write_text("two\n", encoding="utf-8")
    run(["git", "add", "app.txt"], repo)

    proc = run([sys.executable, str(SCRIPT), "--json", "--staged"], repo)

    assert proc.returncode == 0
    payload = json.loads(proc.stdout)
    assert payload["mode"] == "staged"
    assert payload["changed_files"] == ["app.txt"]


def test_changed_files_supports_base_ref(tmp_path: Path) -> None:
    repo = init_repo(tmp_path)
    (repo / "nested").mkdir()
    (repo / "nested" / "file.txt").write_text("x\n", encoding="utf-8")
    run(["git", "add", "nested/file.txt"], repo)
    run(["git", "commit", "-m", "nested"], repo)

    proc = run([sys.executable, str(SCRIPT), "--json", "--base", "HEAD~1"], repo)

    assert proc.returncode == 0
    payload = json.loads(proc.stdout)
    assert payload["base"] == "HEAD~1"
    assert payload["mode"] == "base_ref"
    assert payload["changed_files"] == ["nested/file.txt"]


def test_changed_files_safe_error_outside_git_repo(tmp_path: Path) -> None:
    proc = run([sys.executable, str(SCRIPT), "--json"], tmp_path)

    assert proc.returncode == 2
    payload = json.loads(proc.stdout)
    assert payload["error"] == "not-a-git-repo"


def test_changed_files_preserves_leading_dot_paths(tmp_path: Path) -> None:
    repo = init_repo(tmp_path)
    (repo / ".env.example").write_text("A=1\n", encoding="utf-8")

    proc = run([sys.executable, str(SCRIPT), "--json"], repo)

    assert proc.returncode == 0
    payload = json.loads(proc.stdout)
    assert ".env.example" in payload["changed_files"]
