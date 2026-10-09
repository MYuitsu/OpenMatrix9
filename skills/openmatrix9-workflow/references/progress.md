# Persistent implementation progress

Use `docs/openmatrix9-progress.json` in the project checkout as the evidence ledger. Spec v1 and the project README are synchronized progress views; their status fields do not replace the ledger.

Each work item records:

| Field | Meaning |
|---|---|
| `stage` | Route key from stages.json |
| `feature_ids` | Exact OM9 IDs, empty for infrastructure |
| `procedure_va` | Exact VA when native analysis is involved, otherwise null |
| `status` | planned, in_progress, implemented, validated, blocked, or superseded |
| `outputs` | Actual created source/analysis/artifact paths |
| `evidence` | File/line, source fingerprint or runtime artifact supporting the claim |
| `tests` | Command, result and relevant environment; absence is explicit |
| `uncertainties` | Missing manual, inferred type/default, untested behavior |
| `next_document` | Exact file and optional section/function/page |
| `next_action` | Small concrete continuation |

Update the ledger after a meaningful verified step; preserve unrelated work items. Use actual timestamps. If evidence contradicts a prior entry, correct it and explain why in that entry. File existence can show that a plan was written; it cannot show that the planned UI works.

Completion is stage-specific. A mapped procedure can still be behaviorally uncertain. An implemented feature can still lack runtime validation. A skill suite being available does not advance product implementation status.

## Kiểm tra Spec v1 và README sau khi implement

Sau mỗi phần triển khai và kiểm tra, thực hiện bước đồng bộ này trước khi báo hoàn tất; không cần xin lại phép để ghi tiến độ kỹ thuật.

1. Xác định package Spec v1 trong checkout hiện tại: `OpenMatrix9_Codex_Spec_v1/` hoặc `ref/matrix9/OpenMatrix9_Codex_Spec_v1/`. Nếu cả hai tồn tại, dùng package đang được dự án tham chiếu; không sửa đồng loạt các bản sao. Đối chiếu đúng OM9 ID, command key và phạm vi đã làm với spec tương ứng.
2. Cập nhật phần quyết định/tiến độ triển khai và checklist nghiệm thu của từng spec liên quan: phần đã làm, giới hạn, ngày kiểm tra và đường dẫn bằng chứng. Chỉ đánh dấu `[x]` cho tiêu chí thực sự đã kiểm chứng; giữ `[ ]` cho phần chưa làm hoặc chưa kiểm tra. Giữ nguyên nội dung hành vi nguồn. Rust tests qua nhưng chưa chạy native FreeCAD thì ghi `implemented` và native chưa kiểm chứng, không ghi `validated`.
3. Đồng bộ `IMPLEMENTATION_STATUS.md` trong package nếu có. Nếu `FEATURES.json` và `FEATURES.yaml` đang chứa trạng thái triển khai theo quy ước của package, cập nhật cùng các ID đã kiểm tra và giữ hai bản nhất quán; không thay đổi metadata nguồn hoặc các ID không liên quan. Một slice đã kiểm chứng vẫn là feature triển khai một phần khi còn yêu cầu chưa hỗ trợ. Sau khi sửa nội dung package, làm mới `MANIFEST.json` nếu có, theo `IMPLEMENTATION_RULES.md` của package.
4. Cập nhật mục tiến độ lệnh trong `README.md` ở gốc OpenMatrix9: trạng thái, phạm vi hỗ trợ, giới hạn và ngày cập nhật theo quy ước bảng hiện có. Đối chiếu các dòng lặp lại hoặc alias với đường thực thi thực tế; một alias chưa được kiểm tra không tự động hưởng trạng thái của lệnh khác. Nếu README nhóm đang có bảng tiến độ riêng, đồng bộ các dòng liên quan; README chỉ làm mục lục thì giữ vai trò điều hướng của nó.
5. Ghi bằng chứng chi tiết, tests, phần chưa kiểm chứng và bước tiếp theo vào ledger. Đọc lại diff của spec, các bảng trạng thái và README để bảo đảm cùng mô tả một phạm vi; dẫn tới các tài liệu đã cập nhật trong báo cáo cuối.

Nếu package hoặc tài liệu tiến độ cần cập nhật không có trong checkout, ghi rõ đường dẫn thiếu và phần chưa đồng bộ; tiếp tục cập nhật các tài liệu có sẵn. Với công việc chỉ sửa icon, mapping hoặc skill, ghi đúng loại công việc, không nâng trạng thái triển khai lệnh chỉ từ artwork, phân tích hay hướng dẫn mới.

Việc ghi tiến độ độc lập với xác nhận lưu nghiệp vụ mới vào skill. Áp dụng [business-rule-updates.md](business-rule-updates.md) cho việc lưu quy tắc nghiệp vụ.
