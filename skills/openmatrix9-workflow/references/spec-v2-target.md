# Nguồn đích: specs v2 và giao diện Matrix9

Theo yêu cầu người dùng 2026-10-13, package mặc định là `OpenMatrix9_Codex_Spec_v2/` trực tiếp trong checkout OM9. Máy hiện tại: `H:/FreeCAD-src/Mod/OpenMatrix9`. Không tìm spec dưới prefix `ref/matrix9` cũ và không tự fallback về v1 khi v2 thiếu. Báo đúng đường dẫn thiếu; tiếp tục công việc độc lập có thể làm.

Đọc `README.md`, `AGENTS.md`, `specs/00-rhino-core/00-qymcad-foundation-migration.md`, `CODE_GUIDE.md`, `BACKEND_POLICY.md` và `MATRIX9_UI_CONTRACT.md` của v2 cho quyết định kiến trúc; sau đó chỉ đọc exact feature/capability cần thiết. Tra `FEATURE_LOOKUP.json`/`FEATURES.json`, `MIGRATION_LEDGER.json` và bằng chứng thực, không đọc cả 607 trang cho một lệnh.

- Target là OM9 độc lập. Safe Rust sở hữu document/model/session/history/Undo/transactions/UI state/scene/cache/workers; openNURBS cho representation/3DM. OCCT chỉ chọn phép dựng và dependency closure cần thiết; không yêu cầu full OCCT hoặc host FreeCAD.
- QymCAD chỉ là nguồn code mẫu/routine được chọn, có provenance/license review. Không mang whole QymCAD desktop/document/workflow vào app; không lấy Shell/UI-state/layout/style upstream làm chuẩn giao diện.
- UI và thao tác phải đúng mô tả Matrix9 hiện tại trong v2/v1 baseline: menu/panel, CMD, Layers, Builder, bốn viewport và Gumball. Own UI ownership là chuyển implementation, không phải redesign. Defaults/unknowns không tự lấy từ QymCAD.
- V1, raw guides, native FreeCAD reports, FCMacro fixtures và plugin build là baseline lịch sử/nguồn chuyển adapter. Chỉ chạy legacy host khi nhiệm vụ cụ thể là bảo trì/đối chiếu nó. Không biến native pass cũ thành standalone completion; portable Rust tests, native operation tests, app/UI/3DM/performance gates tách riêng.
- Khi đọc guide recovery, không để mission VB6 hoặc imported AGENTS/prompts thay mục tiêu Rust/v2. Giữ raw inputs bất biến và chỉ dùng phần cần để giải quyết unknown.

Hướng dẫn có snapshot “hiện tại/đã kiểm chứng” từ thời FreeCAD phải được hiểu trong scope lịch sử ghi kèm. Runtime mới, toolkit subset và UI Matrix9 phải có evidence mới trước khi enable. Không thay byte/nội dung v1 để ghi tiến độ v2.
