# Agent Discovery

Discovery now separates:
- Known Providers
- Installed CLIs
- Ready Agents

Minimal rules:
1. PATH lookup only
2. lightweight check only
3. no model calls
4. write local cache

Primary outputs:
- `.together/cache/agent-registry.json`
- `.together/reports/agent-report.md`

Status buckets:
- `not-installed`
- `ready`
- `auth-required`
- `permission-denied`
- `installed-but-broken`
- `installed-unknown`
