# Đề xuất lưu quy tắc modeling 3DM vào skill — chờ đồng ý

Code Phase1 và nghiệm thu đã hoàn tất; đề xuất này không chặn việc sử dụng runtime. Chưa tạo/sửa skill.

Đề xuất tạo skill riêng `openmatrix9-3dm-modeling` cho workflow Rhino → OpenMatrix9 dựng tiếp. Không ghi các quy tắc riêng của OM9 thành hành vi Matrix/Rhino gốc đã chứng minh.

- Trước: preservation là luồng mặc định, giữ dependency/source archive. Sau: giữ mặc định đó và bổ sung lựa chọn **Working geometry (continue modeling)**, nhận geometry độc lập với working UUID mới; history/render/layout nằm ngoài scope.
- Import/Export Selected qua file3DM trước; clipboard chưa là gate. CAD có display mesh vẫn là CAD; Mesh/PointCloud không bắt vertex/point mặc định. Chưa coi đây là bounded snap/index của Phase2.
- Modeling export dùng geometry hiện tại và mới, không hồi sinh object đã xóa hoặc geometry nguồn cũ. Giữ placement/units, preflight geometry không hỗ trợ, transaction Undo/Redo/rollback và staged V5 reread trước atomic replace.
- FCStd phải hoạt động khi không có file nguồn. Bằng chứng ràng buộc source/runtime hashes, fixtures và native/host/version; không dùng SDK proof để tuyên bố đã kiểm chứng trên ứng dụng Rhino.
- Mức hiện tại chỉ Phase1; Phase2–5 và full openNURBS vẫn pending. Các nguyên tắc dung sai bounds/area/volume và phép đo native không phụ thuộc display tessellation được dẫn tới báo cáo kỹ thuật, không ghi số timing máy này thành cam kết chung.

Bằng chứng: [báo cáo Phase1](2026-10-08-modeling-phase-1.md), [18 requirements](modeling-phase-1/requirements.json), [summary/hash audit](modeling-phase-1/summary.json).

Nguồn quy tắc xin đồng ý: [openmatrix9-build-validation/SKILL.md](../../skills/openmatrix9-build-validation/SKILL.md) yêu cầu: “Sau khi hoàn tất code và kiểm tra, nếu có đổi nghiệp vụ, hỏi người dùng có cập nhật nghiệp vụ đó vào skill không.” Chi tiết trong business-rule-updates.md của openmatrix9-workflow. Nếu được đồng ý, tạo bản nguồn/bản cài đặt và kiểm tra retrieval/sync theo skill-creator/writing-skills; nếu không, giữ quy tắc trong tài liệu dự án.
