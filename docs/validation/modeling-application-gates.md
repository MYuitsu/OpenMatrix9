# Nghiệm thu trên ứng dụng ở từng phase

User yêu cầu ngày2026-10-08: **xác thực ứng dụng ở từng phase**. Quy tắc này bổ sung gate thực tế cho cả5 phase và thay tiêu chí đóng Phase1 trước đó. Code/test native/SDK đã đạt vẫn giữ nguyên bằng chứng; chưa đủ để gọi một phase đã nghiệm thu ứng dụng.

## Quy tắc chung

Mỗi phase có hai trạng thái riêng: **implementation_verified** và **application_accepted**. Chỉ đóng phase khi cả hai đạt trên cùng source/runtime đã ghi hash. Native tests, SDK đọc3DM, compile hoặc API File3dm chạy riêng không thay thế ứng dụng Rhino đang chạy.

Ứng dụng bắt buộc: FreeCAD/OpenMatrix9 thực tế và Rhino5 cho V5. Với target mới ở Phase5, phải chạy đúng ứng dụng/version được công bố thêm; không buộc geometry mới thành V5. Script trong ứng dụng được dùng cho đo tự động, nhưng phải thao tác document thật bằng lệnh Open/Import/Select/Export/Save, ghi app version/PID/result và kiểm tra file do app ghi. Workflow UI/CMD, chọn cạnh/mặt và viewport cũng phải có kiểm chứng tương ứng; không suy ra toàn bộ trải nghiệm chuột từ geometry API.

Mỗi lượt dùng document/profile/output riêng, xác minh sở hữu process; không chạy trên document Rhino của người dùng. Input originals bất biến. Có lỗi, crash, timeout, dialog chưa xử lý hoặc thiếu report thì trạng thái là failed/pending, không PASS. Timeout không tự chứng minh process đã dừng.

Luồng roundtrip chung: Rhino Export Selected3DM → FreeCAD import working → thao tác phase → Undo/Redo/Cancel → save/reopen FCStd không có bản nguồn → Export Selected current+new → Rhino Open/kiểm tra/select/SaveAs → FreeCAD reimport file Rhino đã lưu. Tách rõ geometry mới, đã sửa và đã xóa; không hồi sinh snapshot. Sai hình học hoặc promised adapter chưa có thì gate mở.

## Ma trận5 phase

| Phase | FreeCAD/OpenMatrix9 thực tế | Rhino target thực tế | Điều kiện đóng |
|---|---|---|---|
|1 — Nhận geometry|Working import CAD point/curve/BRep/extrusion/Mesh/cloud; mm/cm; chọn Edge/Face; Wireframe/Shaded; move object nhập, thêm circle/line; current Selected export; source-deleted FCStd|Open output gồm edited+new, cloud/point, nested placement và nhẫn thực tế; object count/type/units/validity/current geometry đúng; SaveAs và file đã lưu reimport vào FreeCAD đúng|Native/host/version proof và cả hai ứng dụng đạt, failure/rollback/Undo/Redo đạt; clipboard hai chiều phải đạt theo bổ sung Phase1|
|2 — Classify/snap/curves|Mesh/cloud1 triệu points đi cùng CAD; source inspection không đọc heavy arrays, bounded extraction và500 query/p95 mục tiêu16ms; counters chưa được instrument độc lập trên máy ghi rõ; End/Mid/Point CAD, sửa CV/weight/knot/nối curve, Undo/Redo/current export|Mở curves đã sửa và mới; kiểm tra endpoints,degree/knots/weights/rational shape phù hợp capability, placements/units; SaveAs→FreeCAD reread|Classification không đổi do display mesh; snap không đọc vertices/points ngoài scope; imported/new curve workflow đạt trên binaryPhase2|
|3 — BRep dựng tiếp|Nhẫn nhập→cutter mới→BooleanDifference→Sweep/Loft/Trim/Join theo capability→Undo/Redo→save/reopen→Selected export|Mở CAD sau cut và chi tiết mới; không đổi thành Mesh; kiểm tra trims/holes/seam/pole, solid validity, signed volume/area/bounds; chọn mặt/cạnh; SaveAs→FreeCAD reread|Workflow nhẫn đầy đủ, tolerance cứng đạt; fillet/offset/surface-CV nào chưa promised/implemented giữ capability riêng|
|4 — Blocks/references|Copy độc lập, member edit/delete, nested affine/reflection, CAD+Mesh mixed classification, CurveOnSurface/PolyEdge owning representation; references thiếu rollback|Kiểm tra gốc/copy khác nhau đúng, không resurrect member đã xóa; placement/dependencies đúng theo flatten/materialize policy; chọn geometry và SaveAs→FreeCAD reread|Chỉ đóng các mapping promised có exact geometry/owner proof và cả hai app đạt; reference unresolved vẫn không supported|
|5 — Geometry/bản cài|Installed candidate mới/profile sạch; kiểm từng promised type/operation; chạy lại workflowPhase1–4 trên binary cuối; resources/helper được đóng gói|V5 trên Rhino5; mỗi newer target promoted chạy đúng Rhino/version tương ứng; kiểm edited/new output của từng geometry áp dụng và file do target save lại|Ma trận promised operations/version đóng bằng native+host+actual-app evidence; adapter thiếu vẫn open; full-history/render/clipboard không tự thêm vào scope|

Ngưỡng geometry: bounds0.001mm; area/volume max(0.001,0.0001*abs(reference)); analytical fixtures giữ oracle riêng. Đo CAD tránh display tessellation cache. Nếu cross-kernel bbox API không chứa điểm geometry, cần diagnostic độc lập/trim-aware có bounded measurement và ghi raw failure; không bỏ qua hoặc nới ngưỡng để đóng gate.

## Bằng chứng và trạng thái hiện tại

Mỗi phase lưu `docs/validation/modeling-phase-N/application-evidence.json`: source/module/helper/script hashes; fixture input/output hashes; FreeCAD/Rhino versions và process owner; exact commands; các bước UI/CMD/document; numerical oracles; exit codes; reports/screenshots phù hợp và limits. Screenshot chỉ có UI chrome không chứng minh CAD pixels. Application acceptance không được cộng chung thành coverage percentage.

Phase1: **application_accepted=true**, scoped working V5 exchange đạt2026-10-09. Native/FreeCAD135 gate+8 performance checks,10 native suites,11 legacy SDK cases; actual Rhino5 có10 completed cases. Nhẫn thật đủ29 roots+1 definition member,369 Rhino checks, SaveAs và Export Selected đúng một NewRingCircle. Bốn FreeCAD saved-output lifecycle reports/230 checks chứng minh10 ca giữ geometry/metadata, FCStd không phụ thuộc bản nguồn và current export/reimport;5 ca có thêm Rhino Export Selected reimport. Raw failures và125-check diagnostic được giữ riêng, không cộng lại. Phase2 được nghiệm thu riêng bên dưới; Phase3 accepted tại checkpoint riêng bên dưới; Phase4–5 chưa accepted. Full openNURBS/history/render/primary integration không được suy ra. Clipboard hai chiều đã được bổ sung ở Phase1 và giữ hồi quy trong Phase2. Bằng chứng tại modeling-phase-1/application-evidence.json.

Phase2: **implementation_verified=true, application_accepted=true (accepted_scoped)** ngày2026-10-09. 24 bound host reports/437 checks; actual Rhino5 rational/periodic/placed3 fixtures/125 checks, SaveAs và FreeCAD reread57 checks. Classifier/snap budgets/cache, InterpCrv/Rebuild/typed PointsOn/Join, native rational/periodic edits, FCStd và current V5 đã đạt. Branch-loss fix RED→GREEN7; one final review; two Minor/proof limitations remain in modeling-phase-2/final-review.md. Manifest và mọi report trong modeling-phase-2/summary.json.

Bước tiếp theo: Phase4 — blocks/references theo kế hoạch riêng; Phase3 đã nghiệm thu phạm vi tại checkpoint bên dưới.

Phase3 **implementation_verified=true / application_accepted=true (accepted_scoped)**: 18/18 requirement groups;15 fixture Rhino5 / 599 checks, 14 OM9 reports / 504 checks. Actual two-way clipboard11 fixtures /496 Rhino /218 host +120 saved reread checks on the same module. Rust152 tests; native10 suites; tools72 tests; format/strict Clippy PASS; one independent whole-change review resolved. Source/runtime isolated at om9-phase3-dev/sdk; no primary integration. Full openNURBS remains incomplete: seven broader packages open, zero prepared Phase3 batches pending, future total unknown. [Summary](modeling-phase-3/summary.json).

Owned Rhino verifier uses controlled exit0 after closed/durable outputs, recorded in its report. This does not test Rhino/plugin graceful shutdown hooks. Caller user document is never used; child profiles/documents/PIDs are isolated. Native Part_Primitives Box task panel supplies the cutter command; custom Matrix Box-command parity is not claimed.
