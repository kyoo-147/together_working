# 08 — AI Development Guide

## Mục đích
Giúp coding agent tiếp tục làm việc mà không làm lệch kiến trúc.

## Trước khi code
AI phải đọc:
1. README.md
2. 00-product-brief.md
3. 01-srs.md
4. 02-architecture.md
5. file liên quan đến module đang sửa
6. ADR liên quan

## Quy tắc
- Không đặt orchestration logic trong TUI.
- Không gọi agent trực tiếp từ UI component.
- Mọi state mutation đi qua daemon command handler.
- Mọi thay đổi quan trọng phát event.
- Task không được execute nếu thiếu contract hợp lệ.
- Không tự mở rộng permissions khi fallback.
- Không auto-merge mặc định.
- Adapter-specific logic không rò vào core domain.

## Format task giao cho AI

```markdown
Goal:
Scope:
Allowed files:
Denied files:
Constraints:
Acceptance criteria:
Tests required:
Docs required:
```

## Definition of done
- cargo fmt;
- cargo clippy;
- tests pass;
- không phá protocol;
- docs cập nhật;
- migration có rollback hoặc compatibility note;
- UI snapshot nếu đổi TUI.

## Prompt mẫu

```text
Read docs/01-srs.md, docs/02-architecture.md and docs/08-ai-development-guide.md.
Implement only <module>. Do not change public protocol unless necessary.
First provide a short plan and list files to modify. Then implement tests.
```
