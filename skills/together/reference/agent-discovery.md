# Agent Discovery

The purpose of discovery is not benchmarking. It is lightweight readiness detection.

## What discovery checks

- Is a known command on PATH?
- Does `--help` run successfully?
- Does `--version` or equivalent run successfully?
- If available, does a low-cost `models` or `status` command run?
- Does output suggest auth/config is missing?

## Output status

- `ready`: command exists and basic checks pass
- `installed-but-not-configured`: command exists but output suggests login, auth, or setup is missing
- `installed-but-failing`: command exists but basic checks error
- `unknown`: partial signal, not enough to trust
- `not-found`: command is not present

## Discovery cadence

- Run once before first delegation in a session
- Re-run after a failed worker batch if the failure suggests environment drift
- Re-run on user request

## Local registry

Discovery produces a registry snapshot. That snapshot is the source of truth for:
- which agents are available
- which ones are healthy
- which capabilities are plausible
- which fallbacks are possible right now
