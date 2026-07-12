# 03 — Tech Stack

## Đề xuất chính

### Core/runtime
- Rust 2021/2024
- Tokio: async runtime
- Clap: CLI
- Ratatui: TUI
- Crossterm: terminal backend/input
- portable-pty: PTY
- Serde + serde_json: protocol/state
- TOML: configuration
- SQLite via rusqlite/sqlx
- tracing + tracing-subscriber
- interprocess hoặc tokio Unix socket / Windows named pipe
- git2 hoặc gọi git CLI ở MVP
- notify: file watching
- regex/structured parser cho agent state

Herdr hiện dùng Rust, Ratatui, Crossterm, Tokio và portable-pty; repo phát hành một binary, hỗ trợ PTY thật, detach/reattach và socket API. Đây là bằng chứng stack phù hợp với bài toán terminal-native. citeturn626344view0turn626344view1

## Website/docs
- Astro hoặc Next.js static export
- MDX
- CSS variables, không cần component library nặng

## Test
- cargo test
- insta snapshots cho TUI
- expectrl hoặc custom PTY harness
- integration tests với fake agent CLI
- property tests cho routing/state machine

## Build/release
- cargo-dist hoặc GitHub Actions
- Homebrew tap
- install.sh
- checksums + signed releases

## Vì sao không chọn TypeScript + Ink làm core
Ưu điểm TS: nhanh prototyping.
Nhược điểm: PTY, daemon, binary distribution, low-level terminal và cross-platform phức tạp hơn. Có thể dùng TS cho website hoặc plugin SDK, nhưng core dài hạn nên Rust.

## Phương án hybrid
- Rust core/daemon/TUI.
- JSON socket protocol.
- TypeScript SDK cho plugins và integrations.
