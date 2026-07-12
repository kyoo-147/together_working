# 07 — Delivery Roadmap

## Phase 0 — Refactor documentation (1 week)
- Freeze domain terminology.
- Add SRS, architecture, ADRs.
- Keep existing skill working.

## Phase 1 — Core daemon + CLI (2–4 weeks)
- Rust workspace.
- config + SQLite.
- agent discovery/readiness.
- task contract.
- routing.
- local socket.
- basic CLI.

Exit criteria:
- create task;
- route to fake/real adapter;
- persist/recover state;
- event log works.

## Phase 2 — PTY execution (2–4 weeks)
- portable PTY;
- process supervisor;
- attach/detach;
- scrollback;
- Codex/Claude/Gemini adapters.

## Phase 3 — TUI MVP (3–5 weeks)
- overview;
- departments/tasks/agents;
- real PTY view;
- status bar;
- command palette;
- responsive layout.

## Phase 4 — Review/verification/fallback (3–5 weeks)
- git diff;
- review view;
- checks engine;
- policy gates;
- fallback routing;
- approvals.

## Phase 5 — Plugin SDK and polish
- adapter SDK;
- verification packs;
- notifications;
- docs;
- packaging.

## Phase 6 — Remote/team
- headless remote daemon;
- SSH attach;
- auth;
- multi-machine pool;
- optional local web dashboard.

## MVP definition
MVP hoàn thành khi người dùng có thể:
1. discover agent;
2. tạo task contract;
3. route task;
4. xem agent chạy trong PTY;
5. detach/reattach;
6. xem verification;
7. approve hoặc reroute;
8. khôi phục sau restart.
