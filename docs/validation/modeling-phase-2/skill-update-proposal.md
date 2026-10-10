# Đề xuất cập nhật skill Phase2 — chờ đồng ý

Skill liên quan: [bản nguồn](../../../skills/openmatrix9-feature-port/SKILL.md), bản cài đặt `C:/Users/nguye/.codex/skills/openmatrix9-feature-port/SKILL.md`, reference `references/rhino5-3dm-contract.md`. Đây là quyết định OM9 được nghiệm thu trong runtime riêng, không suy ra hành vi Matrix gốc hoặc full openNURBS.

Trước: skill đã có quy tắc clipboard hai chiều và worker60% của Phase1; chưa có hợp đồng Phase2 về bounded snap và typed curve editing.

Sau: classifier tách native CAD/Mesh/cloud khỏi display mesh; CAD có tessellation vẫn là CAD. Snap CAD có budget/cache; InterpCrv/Rebuild/PointsOn/Join xử lý geometry hiện tại với kiểm tra trước transaction và Undo/Redo. Actual Rhino5 Export Selected/SaveAs và FreeCAD reread đã có gate riêng.

Nội dung đề xuất ghi:

- Mặc định End/Mid/Point chỉ lấy topology CAD đủ điều kiện. Không đọc native Mesh/cloud/SubD preview point arrays để bật lệnh hoặc quyết định snap. Display mesh không biến CAD thành Mesh.
- Snap dùng radius8 logical pixels, cap64 objects/2048 candidates mỗi object/8192 tổng trước allocation. Vượt budget trả incomplete, không chứng nhận snapped point; nhập tay/typed vẫn dùng được. Cache đổi theo Shape/placement/Link/visibility/camera/viewport/Undo/Redo/document lifecycle. p95 warm-query là phép đo theo hardware; không hứa cold rebuild có cùng thời gian. Counter zero-read chưa được instrument độc lập; ngoại lệ khi dựng lại camera index là Minor đang deferred.
- PointsOn Phase2 mở editor số cho một owning native single-edge curve được chọn rõ; CV dùng local frame. Kiểm degree/poles/positive weights/ordered distinct knots/multiplicities/domain/periodicity và stale signature trước một transaction. Link, multi-edge wire, surface/solid CV chưa nằm trong editor này.
- Join nhận2..16 native open-curve inputs, tối đa64 edges, tạo một wire độc lập và giữ inputs. Output phải là một chain/cycle không nhánh và duyệt đủ từng edge đúng một lần. Closed input hoặc nhánh/disconnected bị từ chối trước mutation. Rebuild cũng từ chối wire có nhánh/incomplete traversal trước dialog/transaction; mặc định DeleteInput vẫn có one Undo.
- Independent rational/periodic curves giữ native basis/domain trong scope. Periodic BRep/UV và surface/solid modeling nằm ở phase sau. PolyCurve working reread có thể trở thành NURBS tương đương hình học, không hứa giữ segment identity. Giữ clipboard hai chiều và default max(1,floor(0.60×logical threads)), giảm theo task/RAM.
- Đóng Phase2 cần source/runtime/fixture hashes, FreeCAD thực tế và Rhino5 Export Selected→edit/new→currentV5→Open/SaveAs→FreeCAD reread. Nghiệm thu Phase2 không suy ra full openNURBS hoặc Phase3–5.

Nguồn yêu cầu xác nhận: AGENTS.md và `skills/openmatrix9-workflow/references/business-rule-updates.md`: “Sau khi hoàn tất code và kiểm tra, nếu có đổi nghiệp vụ, hỏi người dùng có cập nhật nghiệp vụ đó vào skill không.” Không ghi vào source/installed skill trước consent cho chính thay đổi Phase2 này.
