# Skills/agents/guidance → specs v2 — 2026-10-13

Theo yêu cầu người dùng, specs v2 là nguồn mặc định cho triển khai OM9 độc lập. QymCAD chỉ cung cấp code mẫu/routine; UI và thao tác giữ Matrix9 như mô tả hiện tại. Thay đổi lần này là hướng dẫn/routing, không triển khai executable, solver hoặc UI mới.

## Đã cập nhật

- 8 skill nguồn và 8 bản cài Codex tại C:/Users/nguye/.codex/skills, gồm agents/openai.yaml, references và scripts. Bản cài mới hơn được hợp nhất để giữ nghiệp vụ launcher, Box/Sphere và progress synchronization.
- AGENTS ở FreeCAD-src, module OM9, docs và docs/superpowers: v2/standalone là target; legacy SDK/FreeCAD plans chỉ cho maintenance/comparison cụ thể.
- Router workflow dùng package trực tiếp OpenMatrix9_Codex_Spec_v2, không ref/matrix9 và không fallback v1. Feature read_next chỉ trả tài liệu thực có, thay SOURCES.md không tồn tại; thêm standalone-foundation, tổng 7 stages.
- README/HANDOFF/skill guide/Rhino reference index, 130 core requirement records và bản đồ port. Existing dated feature/plan/report content giữ scope historical, không nâng các passes sang target.
- OM9-FILE-012 là internal extension legacy ngoài catalog607, không có trang riêng trong v1/v2. Định tuyến rõ tới RCORE-10 cùng Import/Export OM9-FILE-004/001; không bịa catalog ID hay page.
- Private image/manual paths resolve qua registry; không copy commercial assets hoặc sửa đường dẫn archive thật chỉ để đổi version.

## Kiểm chứng

- 16 lượt quick_validate cho 8 source + 8 installed skills: đạt.
- 35 skill/resource files đồng nhất giữa source và installed; 92 relative skill links hợp lệ, agent prompts chứa skill invocation và v2.
- Cả source/installed router audit:607 features,16 domains,7 stages; không thiếu required/conditional documents. Exact Gem Loader và standalone-foundation resolve đúng v2.
- 9 router unit tests đạt, gồm unknown-ID/missing-catalog/path-escape và v1-only checkout không fallback.
- 131 requirement records (130 core + 1 legacy internal extension) có đường đích thực và trạng thái standalone chưa triển khai.
- Toàn bộ665 file v1 giữ SHA-256;677 manifest records của v2 không đổi trong lần chỉnh guidance.
- Public source audit:0 errors. Đây là kiểm tra kỹ thuật payload, không chứng nhận legal clearance/runtime completion.

Kết quả máy đọc: [JSON](2026-10-13-spec-v2-guidance-routing.json). Hướng dẫn hiện hành: [skill guide](../openmatrix9-skills.md), [target chung](../../skills/openmatrix9-workflow/references/spec-v2-target.md), [v2](../../OpenMatrix9_Codex_Spec_v2/README.md).
