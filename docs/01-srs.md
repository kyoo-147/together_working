# 01 — Software Requirements Specification

## 1. Scope
Together là local-first agent orchestration runtime có CLI, TUI và local API.

## 2. Actors
- Operator: người điều khiển.
- Coordinator: agent lập kế hoạch và điều phối.
- Worker: agent thực thi.
- Reviewer: agent hoặc người review.
- Verifier: chạy checks/policy gates.
- Integrator: thực hiện tích hợp cuối.
- Plugin/Tool: hệ thống bên ngoài gọi API.

## 3. Functional requirements

### FR-001 Agent discovery
- Scan PATH và config để tìm CLI.
- Xác định version, auth state, capabilities.
- Cho phép custom adapter.

### FR-002 Readiness
- Trạng thái: unknown, ready, busy, blocked, degraded, offline, cooldown.
- Health check nhẹ, không tiêu tốn nhiều token.
- Lưu last heartbeat, latency, lỗi gần nhất.

### FR-003 Department management
- Tạo/sửa/xóa department.
- Gán primary, secondary, fallback worker.
- Khai báo capability và routing policy.

### FR-004 Task contract
Mỗi task phải có:
- id, intent, scope;
- inputs;
- allowed/denied paths;
- constraints;
- deliverables;
- success criteria;
- reviewer/verifier;
- timeout/retry policy;
- merge policy.

### FR-005 Routing
- Route theo capability, readiness, load, policy, cost và affinity.
- Hỗ trợ manual override.
- Ghi rõ lý do route.

### FR-006 PTY execution
- Mỗi execution có PTY thật.
- Hỗ trợ stdin/stdout, resize, attach/detach.
- Agent tiếp tục chạy khi TUI đóng.

### FR-007 Task lifecycle
queued → planning → researching → executing → reviewing → verifying → ready_to_merge → merged.
Nhánh lỗi: blocked, failed, fallback, cancelled.

### FR-008 Review
- Hiển thị diff, files changed, summary, risks.
- Reviewer approve, request changes hoặc reject.

### FR-009 Verification
- Chạy test/lint/typecheck/security/custom policies.
- Kết quả có evidence và exit code.
- Không được merge nếu gate bắt buộc chưa pass.

### FR-010 Fallback
- Timeout, crash, blocked hoặc health degradation kích hoạt fallback.
- Preserve task contract và context tối thiểu.
- Ghi audit event.

### FR-011 Merge authority
- Không auto-merge mặc định.
- Operator hoặc configured integrator quyết định cuối.

### FR-012 Persistence
- Persist workspace, task, execution, event, PTY/session metadata.
- Khôi phục sau restart.

### FR-013 Local API
- Unix domain socket / named pipe.
- JSON request-response + event subscription.
- CLI, TUI và plugin dùng chung API.

### FR-014 Plugins
- Agent adapters.
- Verification packs.
- Notification hooks.
- Review integrations.

## 4. Non-functional requirements
- Local-first và offline-capable.
- One binary cho runtime/TUI nếu có thể.
- Startup < 500 ms mục tiêu.
- UI render mượt trong terminal phổ biến.
- Không gửi code/context ra ngoài ngoài provider người dùng đã cấu hình.
- Auditability: mọi route, approval, fallback có event.
- Cross-platform: macOS, Linux trước; Windows theo phase.
- Backward compatibility với skill hiện tại.

## 5. Out of scope MVP
- Multi-user cloud control plane.
- Full IDE.
- Thay thế Git hosting.
- Tự huấn luyện model.
- Tự động merge không có policy.
