# Changelog

## Unreleased

- productized repository structure for public open-source use
- added `src/`, `tests/`, `examples/`, `commands/`, `bin/`, and CI workflow
- added installation, contributing, agent-instruction, and product docs
- added validation scripts and committed scrubbed examples

## 0.3.1

- hardened override loading for UTF-8 BOM and malformed JSON
- aligned internal version strings to `0.3.1`
- upgraded README and committed generated product visuals

## 0.2.0

- expanded registry from a few CLI adapters to a broader known-provider catalog
- split Known Providers, Installed CLIs, Ready Agents, and Broken Agents
- added lightweight health classification for auth and permission failures
- added `.together/cache/agent-registry.json`
- added `.together/reports/agent-report.md`
- added task routing and department view rendering
- upgraded README and skill docs toward a real orchestration system

## 0.3.0

- added Verification department to routing, docs, and reporting
- added `.together/cache/last-known-good.json`
- added `.together/cache/runtime-state.json`
- added `.together/providers.override.json`
- added lightweight cooldown-based failover memory
- extended report with best-available, best-by-task, degraded, and last-known-good views
