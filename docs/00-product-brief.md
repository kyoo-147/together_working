# 00 — Product Brief

## Tên sản phẩm
Together

## Một câu mô tả
Together là lớp điều phối local-first biến nhiều AI coding agent CLI thành một phòng ban có cấu trúc, có routing, ranh giới nhiệm vụ, review, verification, fallback và quyền merge cuối cùng.

## Vấn đề
Developer đang sử dụng Codex, Claude Code, Gemini CLI, OpenCode, Amp và các agent khác theo kiểu từng phiên chat riêng lẻ. Kết quả là:
- context phình to;
- agent nhận quá nhiều dữ liệu;
- khó biết agent nào sẵn sàng;
- không có contract rõ ràng;
- review và verification rời rạc;
- khi agent lỗi, workflow dừng;
- quyền tích hợp cuối cùng không rõ.

## Giải pháp
Together:
1. phát hiện agent CLI đã cài;
2. kiểm tra readiness;
3. nhóm agent thành department;
4. tạo task contract;
5. route task theo capability, load và policy;
6. chạy agent trong PTY thật;
7. theo dõi trạng thái và log;
8. review và verify output;
9. fallback nếu agent lỗi;
10. chỉ tích hợp khi được phê duyệt.

## Đối tượng
- developer dùng nhiều coding agent;
- indie hacker;
- team kỹ thuật local-first;
- nhóm cần audit, policy và human approval;
- nhóm muốn terminal-first workflow.

## Định vị
- Không phải chat app.
- Không phải tmux replacement.
- Không phải workflow canvas tổng quát.
- Là **department orchestrator for agent work**.

## Khác biệt với Herdr
Herdr là agent-aware terminal multiplexer: workspaces, tabs, panes, PTY, detach/reattach, remote attach và socket API. Together nên học runtime terminal-native của Herdr nhưng lấy domain trung tâm là workspace → department → task → execution → review → verification → integration. citeturn626344view0turn582266search5

## North-star experience
Người dùng chạy:

```bash
together
```

và thấy ngay:
- department nào hoạt động;
- agent nào ready/busy/degraded;
- task nào đang chạy;
- PTY output trực tiếp;
- verification gate;
- fallback event;
- approve/reject/reroute/merge.
