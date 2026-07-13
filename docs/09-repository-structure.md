# 09 — Proposed Repository Structure

```text
together_working/
├── Cargo.toml
├── crates/
│   ├── together-core/
│   │   ├── domain/
│   │   ├── routing/
│   │   ├── verification/
│   │   └── events/
│   ├── together-daemon/
│   │   ├── server/
│   │   ├── scheduler/
│   │   ├── persistence/
│   │   └── supervisor/
│   ├── together-protocol/
│   ├── together-pty/
│   ├── together-adapters/
│   │   ├── codex/
│   │   ├── claude/
│   │   ├── gemini/
│   │   └── custom/
│   ├── together-cli/
│   └── together-tui/
│       ├── screens/
│       ├── widgets/
│       ├── keymap/
│       └── theme/
├── skills/
│   └── together/
├── schemas/
├── policies/
├── plugins/
├── docs/
│   ├── adr/
│   ├── architecture/
│   └── guides/
├── website/
├── tests/
│   ├── fixtures/fake-agents/
│   ├── integration/
│   └── snapshots/
└── scripts/
```

## Crate boundaries
- `core`: pure domain, không phụ thuộc terminal.
- `daemon`: orchestration application service.
- `protocol`: wire types/versioning.
- `pty`: process/terminal abstraction.
- `adapters`: provider integration.
- `cli`: thin client.
- `tui`: thin presentation client.

## Files nên thêm ngay vào repo
- `AGENTS.md`
- `CONTRIBUTING.md`
- `docs/SRS.md`
- `docs/ARCHITECTURE.md`
- `docs/TUI_SPEC.md`
- `docs/ROADMAP.md`
- `docs/adr/0001-rust-daemon-tui.md`
- `docs/adr/0002-task-centric-domain.md`
- `schemas/task-contract.schema.json`
- `schemas/protocol.schema.json`
