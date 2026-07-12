# 10 — Decisions and Open Questions

## Đã chốt
- Terminal-first.
- CLI + TUI là interface chính.
- Daemon giữ state và process.
- Rust core.
- Ratatui/Crossterm.
- PTY thật.
- Local socket API.
- Task-centric domain.
- Human-controlled merge mặc định.
- Skill hiện tại tiếp tục tồn tại như integration/instruction layer.

## Cần quyết định
1. SQLite hay event log thuần cho MVP?
2. Git CLI hay libgit2?
3. Socket protocol JSON-RPC hay custom framed JSON?
4. Agent state parser dựa structural terminal hay adapter callback?
5. Windows trong MVP hay phase sau?
6. Có hỗ trợ nhiều workspace daemon cùng lúc không?
7. TUI quản lý split pane ở MVP hay một PTY focus view?
8. Plugin chạy in-process, subprocess hay WASM?
9. Có cần policy language riêng hay TOML/YAML đủ?
10. Codex bắt buộc là integrator hay configurable?

## ADR cần viết
- ADR-0001 Rust daemon + thin clients.
- ADR-0002 Task-centric domain over pane-centric domain.
- ADR-0003 Local socket protocol.
- ADR-0004 Persistence strategy.
- ADR-0005 Plugin isolation.
- ADR-0006 Merge authority policy.

## Rủi ro
- Scope creep thành tmux replacement.
- Parsing trạng thái agent không ổn định.
- PTY cross-platform phức tạp.
- Adapter drift khi CLI provider thay đổi.
- TUI quá dày thông tin.
- Security nếu plugin có quyền rộng.

## Giảm rủi ro
- Fake-agent test harness.
- Adapter version probes.
- Protocol versioning.
- TUI snapshot tests.
- Default-deny permissions.
- MVP không làm full pane manager.
