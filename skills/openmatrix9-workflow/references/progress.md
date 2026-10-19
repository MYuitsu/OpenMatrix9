# Persistent implementation progress

Use `docs/openmatrix9-progress.json` in the project checkout as the evidence ledger. milestone specs and the project README are synchronized progress views; their status fields do not replace the ledger.

Each work item records:

| Field | Meaning |
|---|---|
| `stage` | Route key from stages.json |
| `feature_ids` | Exact OM9 IDs, empty for infrastructure |
| `procedure_va` | Exact VA when native analysis is involved, otherwise null |
| `status` | planned, in_progress, implemented, validated, blocked, superseded, or documentation_validated (spec/skill-only scope) |
| `outputs` | Actual created source/analysis/artifact paths |
| `evidence` | File/line, source fingerprint or runtime artifact supporting the claim |
| `tests` | Command, result and relevant environment; absence is explicit |
| `uncertainties` | Missing manual, inferred type/default, untested behavior |
| `next_document` | Exact file and optional section/function/page |
| `next_action` | Small concrete continuation |

Update the ledger after a meaningful verified step; preserve unrelated work items. Use actual timestamps. If evidence contradicts a prior entry, correct it and explain why in that entry. File existence can show that a plan was written; it cannot show that the planned UI works.

Completion is stage-specific. A mapped procedure can still be behaviorally uncertain. An implemented feature can still lack runtime validation. A skill suite being available does not advance product implementation status.

## Kiểm tra milestone specs và README sau khi implement

Sau mỗi phần triển khai và kiểm tra, thực hiện bước đồng bộ này trước khi báo hoàn tất; không cần xin lại phép để ghi tiến độ kỹ thuật.

1. Nguồn nền tảng là `OM9_Milestones/M01 - CAD Shell, Viewport`; tra exact milestone/ID và owner bằng router. Không yêu cầu package policy cũ.
2. Cập nhật `runtime_implementation` riêng trong JSON milestone liên quan, giữ nguyên planning/source content và IDs: phạm vi đã làm, giới hạn, ngày kiểm tra, bằng chứng. Chỉ ghi validated khi có kiểm chứng đúng target; manual review giữ pending đến khi reviewer thực xác nhận.
3. Giữ các status/traceability views liên quan nhất quán với phạm vi đã kiểm chứng. Không tự tạo catalog/manifest cũ hoặc đổi hàng không liên quan.
4. Cập nhật mục tiến độ lệnh trong `README.md` ở gốc OpenMatrix9: trạng thái, phạm vi hỗ trợ, giới hạn và ngày cập nhật theo quy ước bảng hiện có. Đối chiếu các dòng lặp lại hoặc alias với đường thực thi thực tế; một alias chưa được kiểm tra không tự động hưởng trạng thái của lệnh khác. Nếu README nhóm đang có bảng tiến độ riêng, đồng bộ các dòng liên quan; README chỉ làm mục lục thì giữ vai trò điều hướng của nó.
5. Ghi bằng chứng chi tiết, tests, phần chưa kiểm chứng và bước tiếp theo vào ledger. Đọc lại diff của spec, các bảng trạng thái và README để bảo đảm cùng mô tả một phạm vi; dẫn tới các tài liệu đã cập nhật trong báo cáo cuối.

Nếu package hoặc tài liệu tiến độ cần cập nhật không có trong checkout, ghi rõ đường dẫn thiếu và phần chưa đồng bộ; tiếp tục cập nhật các tài liệu có sẵn. Với công việc chỉ sửa icon, mapping hoặc skill, ghi đúng loại công việc, không nâng trạng thái triển khai lệnh chỉ từ artwork, phân tích hay hướng dẫn mới.

Việc ghi tiến độ độc lập với xác nhận lưu nghiệp vụ mới vào skill. Áp dụng [business-rule-updates.md](business-rule-updates.md) cho việc lưu quy tắc nghiệp vụ.

## Tách bằng chứng chuyển nền

Ledger phải ghi target/scope và fingerprint thực. Native passes cũ vẫn là legacy_freecad, không nâng standalone status. Đồng bộ milestone runtime_implementation/status và README theo evidence; giữ hash/báo cáo cũ. Kiểm skill/routing chỉ ghi documentation_validated, không feature completion.
