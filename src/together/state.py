from __future__ import annotations

from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
TOGETHER_DIR = REPO_ROOT / ".together"
CACHE_DIR = TOGETHER_DIR / "cache"
REPORT_DIR = TOGETHER_DIR / "reports"

GENERATED_RUNTIME_FILES = (
    ".together/cache/agent-registry.json",
    ".together/cache/last-known-good.json",
    ".together/cache/runtime-state.json",
    ".together/reports/agent-report.md",
)

