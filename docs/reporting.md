# Reporting

Doctor flow writes:
- `.together/cache/agent-registry.json`
- `.together/cache/last-known-good.json`
- `.together/cache/runtime-state.json`
- `.together/reports/agent-report.md`

Report sections:
- Summary
- Known Providers
- Installed CLIs
- Ready Agents
- Broken Agents
- Best Available Workers
- Best By Task
- Degraded Agents
- Task Routing
- Department View
- Last Known Good

Purpose:
- fast status read
- repeatable routing context
- resume-friendly local memory
