# 02 — Architecture

## Kiến trúc tổng thể

```text
┌──────────────────────────────────────────────┐
│ Interfaces                                   │
│ CLI | TUI | Local Web | SDK | Plugins        │
└──────────────────────┬───────────────────────┘
                       │ local socket API
┌──────────────────────▼───────────────────────┐
│ togetherd daemon                             │
│ command handler | event bus | scheduler      │
├──────────────────────────────────────────────┤
│ Domain services                              │
│ discovery | readiness | departments          │
│ tasks | routing | execution | review         │
│ verification | fallback | integration        │
├──────────────────────────────────────────────┤
│ Runtime                                      │
│ PTY manager | process supervisor | sessions  │
│ persistence | logs | git adapter             │
├──────────────────────────────────────────────┤
│ Adapters                                     │
│ Codex | Claude | Gemini | OpenCode | custom  │
└──────────────────────────────────────────────┘
```

## Tách daemon và client
Herdr cho thấy lợi ích rõ của session sống độc lập với UI: agent tiếp tục chạy khi detach, reattach từ terminal khác, và agent có thể điều khiển runtime qua socket API. Together nên áp dụng mô hình daemon/client tương tự. citeturn626344view0

## Thành phần

### togetherd
- giữ state chuẩn;
- quản lý PTY/process;
- route task;
- phát event;
- persist dữ liệu;
- enforce policy.

### together CLI
- lệnh một lần;
- automation/scripting;
- admin runtime.

### Together TUI
- client đọc state và event;
- không chứa business logic quan trọng;
- gửi command qua socket.

### Adapter layer
Mỗi agent adapter cung cấp:
- detect();
- probe();
- spawn();
- parse_state();
- resume();
- capabilities();
- approval_patterns().

## Event-driven model
Các event chính:
- AgentDiscovered
- AgentReadinessChanged
- TaskCreated
- TaskRouted
- ExecutionStarted
- ExecutionOutput
- ExecutionBlocked
- ReviewRequested
- ReviewCompleted
- VerificationCompleted
- FallbackTriggered
- MergeApproved
- TaskCompleted

## Persistence
MVP: SQLite + append-only event log.
- SQLite cho query.
- Event log cho audit và replay.
- PTY scrollback lưu file chunked.

## Security boundaries
- allowlist command;
- allowed/denied paths;
- env filtering;
- secret redaction;
- permission prompt;
- no implicit network proxy;
- plugin capability declaration.

## Khác Herdr
Herdr lấy pane/session làm trung tâm. Together lấy task contract và workflow state làm trung tâm. PTY là execution substrate, không phải domain object chính.
