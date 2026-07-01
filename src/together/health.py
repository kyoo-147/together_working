from __future__ import annotations

HEALTH_STATES = (
    "ready",
    "auth-required",
    "permission-denied",
    "installed-but-broken",
    "installed-unknown",
    "not-installed",
    "disabled-by-override",
    "cooldown",
)

LIGHTWEIGHT_CHECKS = (
    "command exists",
    "version/help runs",
    "auth/config obvious failures",
    "permission denied detection",
)

