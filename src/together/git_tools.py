from __future__ import annotations

import subprocess
from pathlib import Path


def normalize_repo_path(value: str) -> str:
    normalized = value.replace("\\", "/").strip()
    if normalized.startswith("./"):
        normalized = normalized[2:]
    return normalized


def run_git(args: list[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["git", *args],
        cwd=cwd,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        check=False,
    )


def resolve_git_root(cwd: Path) -> tuple[Path | None, str | None]:
    proc = run_git(["rev-parse", "--show-toplevel"], cwd)
    if proc.returncode != 0:
        return None, "not-a-git-repo"
    return Path(proc.stdout.strip()), None


def list_changed_files(
    cwd: Path,
    *,
    staged: bool = False,
    base: str | None = None,
) -> tuple[dict | None, dict | None]:
    git_root, error = resolve_git_root(cwd)
    if git_root is None:
        return None, {"error": error, "message": "Current directory is not inside a git repository."}

    mode = "working_tree"
    if staged:
        mode = "staged"
        diff_proc = run_git(["diff", "--name-only", "--cached"], git_root)
    elif base:
        mode = "base_ref"
        diff_proc = run_git(["diff", "--name-only", f"{base}...HEAD"], git_root)
    else:
        tracked_proc = run_git(["diff", "--name-only"], git_root)
        untracked_proc = run_git(["ls-files", "--others", "--exclude-standard"], git_root)
        if tracked_proc.returncode != 0:
            return None, {"error": "git-diff-failed", "message": tracked_proc.stderr.strip() or "git diff failed"}
        if untracked_proc.returncode != 0:
            return None, {"error": "git-untracked-failed", "message": untracked_proc.stderr.strip() or "git ls-files failed"}
        changed_files = sorted(
            {
                normalize_repo_path(line)
                for line in (tracked_proc.stdout.splitlines() + untracked_proc.stdout.splitlines())
                if line.strip()
            }
        )
        return {
            "base": "HEAD",
            "mode": mode,
            "git_root": str(git_root),
            "changed_files": changed_files,
        }, None

    if diff_proc.returncode != 0:
        return None, {"error": "git-diff-failed", "message": diff_proc.stderr.strip() or "git diff failed"}

    changed_files = sorted({normalize_repo_path(line) for line in diff_proc.stdout.splitlines() if line.strip()})
    return {
        "base": base or "HEAD",
        "mode": mode,
        "git_root": str(git_root),
        "changed_files": changed_files,
    }, None
