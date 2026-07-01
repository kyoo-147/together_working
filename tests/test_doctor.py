from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def test_doctor_runs() -> None:
    proc = subprocess.run(
        [sys.executable, str(ROOT / "skills" / "together" / "scripts" / "doctor.py")],
        cwd=ROOT,
        check=False,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    assert proc.returncode == 0

