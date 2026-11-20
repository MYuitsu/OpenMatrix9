# Kế hoạch hoàn thiện Project Manager và Job Bags

Ngày kiểm tra: 2026-11-17. Target: standalone Rust OM9 trên Windows. Phạm vi lần này là audit và lập kế hoạch cho panel **PROJECTS trong ảnh người dùng**, chưa triển khai các thay đổi sản phẩm bên dưới. Kế hoạch Tools 31 lệnh trước đó là công việc riêng.

Project Manager đã có model Rust, capture/insert, Master restore và lưu state trong `.om9`. Phần còn thiếu gồm cả giao diện Matrix và tính đúng của dữ liệu snapshot. Cần ưu tiên khôi phục độc lập với đối tượng gốc, giữ placement, bảo toàn resource khi lưu/mở và xác nhận trước khi thay bản vẽ; sau đó làm các ô động, năm nhóm snapshot và Mngr.

## Nguồn đúng cho panel trong ảnh

- Owner chính: **M01 `OM9-L2-03-15`**, đặc biệt L3 `003` đến `009`. L3 `001`/`002` là file lifecycle và import/export dùng chung. M01 `01-03` sở hữu document/IDs/instances/resources, `01-12` sở hữu codec; `01-02` sở hữu transaction/Undo, `01-01`/`03-07` sở hữu dock và layout.
- Trong các reference đã chuẩn hóa của project, hướng dẫn Project Manager là **Matrix 8 Book 1, PDF 58–62**, database/Mngr là **PDF 63–66**; milestone ghi trang in 48–56. Chưa tìm thấy section Project Manager/Job Bags riêng trong các extract Book 2 đã tra. Không gán nội dung Book 1 thành Book 2. Nếu có trang Book 2 khác được xác định sau này, bổ sung nguồn chính xác vào crosswalk.
- Ảnh đính kèm xác nhận cấu trúc nhìn thấy: danh sách project ở trái; hàng thumbnail có số ở trên; nút tạo, load, Mngr; vùng snapshot có tên Master, Creation, Parts, Render, Output ở dưới; thanh cuộn/expand và các nút In/Save/X. Ảnh không chứng minh semantics archive hoặc thuật toán lưu.
- VB6/Ghidra được đọc theo procedure liên quan; managed C# xác nhận đường mở Mngr, CRUD/catalog, job bag và preview. Chi tiết định danh/hash giữ trong ghi chú phân tích cục bộ `docs/analysis/matrix90/2026-11-17-project-manager.md`, không đưa raw source/ảnh thương mại vào public assets.
- [Receipt audit](../validation/2026-11-17-project-manager-audit.json) lưu fingerprint code, nguồn tham khảo, log kiểm tra, các giới hạn và các ca chẩn đoán. Không lấy trạng thái kế hoạch/L4 cũ làm bằng chứng nghiệm thu hiện tại.

## Cách dùng theo manual để làm tiêu chuẩn triển khai

Đây là workflow Matrix cần đối chiếu; các thao tác chưa có trong OM9 được ghi ở bảng gap phía sau.

| Thao tác | Hành vi cần có |
|---|---|
| Tạo project | Bấm `+`, nhập tên và Create/Cancel. Chọn một project row trước khi lưu các bước thiết kế. |
| Lưu một phần thiết kế | Chọn đối tượng trong viewport, bấm **In** của ô số. Job Bag thường chứa phần được chọn; sau đó có thể tiếp tục sửa/xóa đối tượng trong bản vẽ. |
| Lấy một phần đã lưu | Bấm thumbnail hoặc mũi tên **Load** của ô số: import thêm vào bản vẽ hiện tại; không xóa các đối tượng đang có. Menu chuột phải cho phép lọc gems/curves/surfaces/polysurfaces/meshes khi In hoặc Load. |
| Save hoặc X một ô | Biểu tượng đĩa lưu nội dung ô ra file; X làm rỗng nội dung ô, không tự xóa các đối tượng đang ở viewport. Compact mới dồn và đánh số lại các ô trống. |
| Lưu toàn bộ bản vẽ | Mở vùng nâng cao; In vào **Master/Creation/Parts/Render/Output** lưu toàn document, gồm đối tượng ẩn/khóa và thuộc tính liên quan. Không bắt buộc chọn đối tượng. Nút `+` của từng loại thêm phiên bản cùng loại. |
| Mở snapshot có tên | Load snapshot thuộc năm nhóm là luồng **Open/thay document**, khác với import ô thường. Kiểm tra dirty và cho người dùng lưu/hủy trước khi thay. |
| Quản lý row | Chuột phải tên row: Save/Load archive, Delete, Rename, Compact, Timer Start/Stop/Reset, Update; thanh cuộn duyệt các ô, expand thu/phóng vùng chứa. |
| Mngr | Mở catalog/database: tìm theo tên/tags/customer/designer/status/ngày/weights/labor; Info lưu metadata, Files xem job bags và preview; hide/show row, refresh, import project/folder, export và chọn thư mục project. |

Book 1 mô tả 10 ô khởi tạo rồi mở rộng. Listing Matrix90 ở nhánh JobBagIn có kiểm ngưỡng và cảnh báo tối đa 200; cần chốt quy tắc số ô/budget trong P00, không diễn giải ảnh chỉ thấy 3 ô thành giới hạn 3, cũng không hứa vô hạn. `.mpj` là archive được manual mô tả; tương thích đọc/ghi định dạng này chưa được chứng minh từ extension và không phải điều kiện để lưu native OM9 đúng dữ liệu.

## Những gì đã có và cần giữ lại

- `project_manager.rs`: ProjectId/SnapshotId, nhiều project row, create/rename/select, customer/notes, capture clone entities và Master clone document; validation text và một số giới hạn.
- `session/jobbag.rs`: insert entities qua Controller transaction/candidate scene, tránh trùng ObjectId và gom một Undo cho lần insert.
- `session/project_state.rs`: opaque resource `om9.project-manager.v1`, staging metadata và Master replacement có admission/validation trước publication.
- `ui.rs`: staging trước Save, đọc state khi mở `.om9`, `restore_master_live`; `ui/chrome.rs`: Projects dock trên Builders và UI capture/restore cơ bản.
- Các test point/notes/persistence đang có. Trong audit, hai binary integration vừa dựng chạy trực tiếp đạt **3 test UI + 5 test persistence**. Đây là bằng chứng cho các fixture hiện tại, chưa phải nghiệm thu hình học trang sức/BRep hoặc giao diện theo ảnh.

Không viết lại codec, history hoặc scene pipeline để làm panel này. Tái dùng service và quyền sở hữu Rust hiện có; native chỉ cho capability OCCT/openNURBS thật sự cần.

## Bảng thiếu sót theo milestone

Mã G01–G20 dùng để theo dõi gap trong plan, không phải command ID hoặc milestone mới. `Xác nhận bằng probe` là chẩn đoán trên model production được compile nguyên trạng; không phải test sửa lỗi đã xanh.

| Gap | Owner hiện có | Hiện trạng và bằng chứng | Việc cần làm |
|---|---|---|---|
| G01 Snapshot phụ thuộc live source | `03-15-005/006` | `Snapshot::available` đòi captured IDs còn trong document hiện tại. Probe: sau xóa source, snapshot vẫn chứa 1 entity nhưng UI availability=false. | Snapshot có nội dung tự khôi phục; source stamp dùng provenance, không dùng sự tồn tại live để khóa bag đã lưu. Thử source bị xóa, New/Open và insert vào document khác. |
| G02 Restore làm mất placement | `03-15-006`, `01-03` | `restore_entities` thay placement bằng translation cố định. Probe: point world `[12,23,34]` restore thành `[4,3,4]`. | Giữ transform gốc; nếu có tùy chọn đặt lại vị trí thì compose transform rõ ràng, không tự dời mặc định. Remap IDs đồng bộ. |
| G03 Thiếu dependency closure | `03-15-005/006`, `01-03` | Snapshot ô thường chỉ có `Vec<Entity>`; restore chỉ Insert entity. Chưa capture layer definitions/blocks/resources/history graph cần thiết. | Đóng gói phần nguồn được chọn cùng dependency bắt buộc; remap layer/definition/resource/feature IDs và membership. Không chứng nhận từ fixture Point. |
| G04 Master mất resource payload qua JSON | `03-15-007`, `01-12` | `Resource.payload` là `serde(skip)`, Master nằm trong JSON của manager. Probe: resource 16 byte có trước encode; sau manager decode báo thiếu embedded resource payload. | Snapshot codec/container có manifest và chunks được hydrate đúng hash/length. Dùng codec owner, tránh serialize Document như JSON độc lập rồi coi đã đủ. |
| G05 Rename đơn lẻ có thể không được stage | `03-15-003/008` | `has_persistent_state` không xét đổi tên row mặc định. Probe rename-only trả false; `stage_project_state` dùng predicate này. | Theo dõi metadata dirty/version; save/reopen khi chỉ rename phải giữ tên. Không làm bản vẽ mới dirty chỉ do default UI. |
| G06 Decode chưa kiểm đủ và notes không xóa trắng được | `03-15-003/008`, `01-12` | Probe `from_json` nhận duplicate ProjectId; `set_notes("")` bị từ chối. Code decode chủ yếu kiểm row/active/slot count. | Validate ID uniqueness/counters/ranges/text/dependencies sau decode; field được phép trống phải xóa được. Phân biệt giới hạn tài nguyên với schema hỏng. |
| G07 Ba ô cố định | `03-15-004` | `SLOT_COUNT=3`, UI `(0..3)`, decode buộc đúng 3. Test hiện cũng yêu cầu 3. | List ô động, index/ID ổn định, initial capacity theo contract đã chốt; mở rộng/compact và migration state cũ, budget có thông báo. |
| G08 Thumbnail chỉ là icon và chữ | `03-15-004`, `01-06` | `Thumbnail` lưu label/count/kind/revision; UI vẽ icon Project, không có ảnh capture. | Render/capture thumbnail thật có camera/source binding, lưu asset riêng; placeholder cho pending/error, cache có giới hạn. Không dùng bitmap thương mại. |
| G09 Thiếu hành vi card In/Load/Save/X | `03-15-004/005/006` | UI có text Capture/Restore. Response thumbnail không xử lý click load; chưa có Save/X/context menu theo loại. | Authored icon bindings và tooltip, click thumbnail/load cùng dispatcher; per-bag export/clear, filtered capture/insert và cancel/error. |
| G10 Chỉ một Master | `03-15-007` | Model có `master: Option<MasterSnapshot>`, chưa có Creation/Parts/Render/Output hoặc các phiên bản cùng loại. | Typed named bag kind với list version, toàn document độc lập; `+` thêm cùng loại. Kiểm hidden/locked/layers/materials/history/display khi nguồn hỗ trợ. |
| G11 Master thay bản vẽ ngay, atomicity toàn luồng cần kiểm | `03-15-007`, `03-15-001` | Click Restore Master gọi replacement rồi `stage_project_state`; chưa thấy dirty/save/cancel dialog ở đường này. Point test chỉ kiểm số entity. | Dùng cùng lifecycle guard của Open. Chuẩn bị cả state/scene/resource trước publish; kiểm lỗi staging sau replacement, giữ document/file path/Undo/Redo đúng contract. |
| G12 Thiếu row Save/Load/Delete/Compact | `03-15-003`, `03-15-001` | Có create/rename/select bằng model. Chưa có archive theo row, delete/compact/update row và xử lý file kèm. | Service row catalog, atomic archive/import/export; phân biệt xóa khỏi danh sách với xóa dữ liệu trên đĩa. Không dùng label làm stable identity. |
| G13 Chưa có workspace catalog độc lập document | `03-15-003/008`, `01-12` | Manager gắn vào UI của document và embedded resource; New/Open reload tạo/đọc manager của document đó. | Catalog project của người dùng tồn tại xuyên New/Open/restart; xác định phần app workspace, phần document và snapshot. Không để mở một file thay cả catalog mà không có policy. |
| G14 Layout chưa giống panel ảnh | `03-15-004/009`, `03-07` | Row dùng horizontal wrapped labels; các ô dựng dọc cùng field Name/Customer/Notes, không có hai hàng filmstrip. | List trái, numbered strip trên, named strip dưới, Create/Load/Mngr và scroll/expand; giữ thứ tự dock. Kiểm kích thước hẹp/DPI/floating và focus CMD. |
| G15 Timer chưa có | `03-15-003/008`, cần task con | Chưa có elapsed/start/stop/reset trong ProjectRow. Listing và Book 1 có timer riêng. | Timer theo row, chỉ chạy khi bật, persist elapsed/state theo contract; không dựa vào vòng render để cộng thời gian. |
| G16 Chưa có Mngr và search/visibility | `03-15-008` owner ứng viên, mở rộng task explicit | Không có cửa sổ/database catalog, search/filter, visible/hidden row hoặc Refresh/Import Folder/Change root. | Rust catalog/index + egui Mngr, typed filters và import/export; dùng ID/relative paths, xử lý file thiếu/duplicate và hủy. Không mang SQL/COM runtime gốc vào OM9. |
| G17 Metadata chỉ notes/customer text | `03-15-008` | Thiếu customer records/designer/status/tags/dates/job number/labor/ring/weights và Files/Info. | DTO/search contract có units/nullability; CRUD các danh mục bằng service chung, tạo metadata nhập tay trước; tích hợp gem/metal report chỉ khi source và unit đã xác minh. |
| G18 Thiếu command entry thống nhất | `03-15-003/009`, `01-02` | Decompile có `gvProjectManager`/`gvOpenProjectManager`; trong parser/registry đã tra chưa tìm thấy các alias này. Hành vi hiện gọi trực tiếp từ UI. | Panel/Mngr/menu/CMD đi qua typed action owner, phân biệt mở panel với mở Mngr; shortcut, icon và focus dùng cùng route; không tự tạo F6 geometry command không có căn cứ. |
| G19 Giới hạn nguồn và dữ liệu nặng | `03-15-005/007`, `01-12`, `01-03` | Capture clone đồng bộ, MAX_CAPTURE=64, JSON 16 MiB, Master clone toàn Document; chưa có receipt budget cho toàn phần snapshot/thumbnail/serialization. `.om9` v5 đang từ chối BRep ở container encode. | Admission và worker/lazy asset theo nhu cầu, reuse immutable source; xử lý quota/cancel/stale và BRep codec blocker. Không bỏ guard hoặc dùng display mesh thay nguồn để save thành công. |
| G20 Tiến độ cũ mô tả mâu thuẫn | `03-15` và L3 liên quan | Một số `remaining` nói chỉ in-session, trong khi source/test đã có point `.om9` embedding; Master scope cũ nói chỉ inspection nhưng đã có live restore. | Ghi audit mới phân biệt point/notes pass với resource/BRep/full document còn hở; giữ receipt lịch sử và manual gate pending. Không nâng toàn module thành hoàn tất. |

## Kế hoạch thực hiện

Theo chỉ đạo người dùng ngày 2026-11-17: **hoàn tất phần triển khai P00–P06 rồi mới chạy một đợt kiểm thử gộp ở P07**. Không chạy test/probe/benchmark hoặc lập đợt nghiệm thu riêng sau mỗi helper hay gói nhỏ. Các tiêu chí dữ liệu và giao diện dưới đây được giữ làm đầu vào cho đợt cuối, không phải các điểm dừng để test giữa chừng. Giữ nguyên bằng chứng audit đã có, không chạy lại để xác nhận cùng một điều.

### P00 Chốt hợp đồng Project Manager

Ưu tiên P0. Owner `03-15`, cùng owner document/codec/lifecycle liên quan.

- Dùng Book 1 PDF 58–66 và ảnh để lập bảng action: row CRUD/archive, ô thường, năm nhóm named snapshot, Mngr; ghi input/output/prompt/confirm/cancel/Undo/file effects.
- Chốt initial/dynamic slot count và giới hạn có budget; ghi xung đột Book 1 với listing Matrix90 thay vì coi nguồn cũ là mặc định sản phẩm mới. Chốt name uniqueness/display sorting, archive native và mức hỗ trợ `.mpj` riêng.
- Phân biệt **CaptureSelection/InsertSelection** và **CaptureDocument/OpenDocument**. Open đi qua dirty/save/cancel; Insert là một transaction/Undo. Capture/export không tự sửa geometry.
- Tạo hoặc bổ sung task/L4 cho timer, catalog, Mngr/search và archive nếu L3 hiện chỉ ghi basic metadata. Giữ IDs/owner hiện có; không lấy AFTER hoặc Tools menu làm owner thay thế.
- Chốt owner/task và hành vi cần đạt cho G01–G20, ghi rõ decision còn thiếu rồi tiếp tục triển khai. Chưa viết/chạy một bộ fixture riêng cho từng task; không coi snapshot offset hay source-deleted unavailable là nghiệp vụ đúng chỉ vì test cũ xanh.

### P01 Sửa tính độc lập và restore đúng hình học

Ưu tiên P0, phụ thuộc P00. Giải G01–G03; trước khi làm layout.

- Tái dùng immutable geometry và transaction/candidate scene hiện có. Capture nội dung được chọn cùng dependency closure cần thiết, không chỉ clone root entity rồi giữ IDs của document cũ.
- Load ô thường vẫn dùng được sau khi xóa/sửa source hoặc chuyển document. Giữ placement/world geometry; remap IDs và nội bộ references nhất quán, gồm layers, definitions/instances, resources, gems và History theo policy được chốt.
- Giữ geometry/placement, dependencies và scene bindings; một Undo xóa trọn lần insert, Redo giữ bindings. Không đổi dung sai hoặc hình học vì thumbnail. Các tình huống kiểm chứng được gom vào P07 sau khi triển khai xong.

### P02 Codec snapshot và lưu dữ liệu an toàn

Ưu tiên P0, phụ thuộc contract P00 và phối hợp P01. Giải G04–G06, G13, phần codec G19.

- Tách catalog app workspace, per-project metadata và content-addressed snapshot assets theo ownership đã có; reuse codec/resource owner. Cần schema version và migration từ manager JSON ba ô hiện tại.
- Full snapshot phải mang resource chunks và hydrate theo hash/length trước validate/replace; không coi `serde_json<Document>` là archive đầy đủ. Loại bản embedded manager cũ khỏi snapshot mới theo policy để tránh state lồng không cần thiết, giữ phần dữ liệu thiết kế cần thiết.
- Sửa dirty/persistent-state để rename-only giữ qua Save/Open và restart; notes/customer có thể clear khi contract cho phép. Validate IDs/counters/schema/bounds/dependency và lỗi file hỏng trước thay state hiện tại.
- `.om9` v5 BRep là blocker của owner codec, không giải bằng bypass guard. Point/mesh snapshot có thể nghiệm thu phạm vi riêng; BRep/native/opaque roundtrip chỉ đóng khi có fixture thực.
- Giữ resource payload/hash và IDs qua vòng lưu/mở, lỗi không thay state cũ; xử lý budget thay vì chỉ tăng MAX_CAPTURE. Kiểm chứng chung ở P07.

### P03 Ô động và năm nhóm named snapshots

Ưu tiên P1, sau P01/P02.

- Migrate fixed vector 3 ô sang numbered bags động; fill cuối thêm capacity theo P00, clear giữ ô trống, Compact dồn số có quy tắc và không đổi bag identity.
- Named bag kind gồm Master/Creation/Parts/Render/Output, mỗi kind có nhiều phiên bản; `+` tạo cùng loại. Tái dùng Document replacement owner.
- Thêm dirty/save/cancel cho Open named bag và overwrite/cancel cho capture theo contract. Chuẩn bị geometry, scene và manager metadata trước một lần publish; kiểm file path và lịch sử sau Open.
- Hoàn thiện cả năm loại và các phiên bản trước đợt kiểm thử P07; không dùng kết quả riêng Master để kết luận bốn nhóm còn lại đã đạt.

### P04 Giao diện Projects theo ảnh và thumbnail thật

Ưu tiên P1, có thể dựng layout sau P00 và nối đầy đủ hành vi với P01–P03 trước P07.

- Sidebar list project bên trái, numbered filmstrip phía trên, named filmstrip phía dưới, scroll/expand và Create/Load/Mngr. Reuse dock container và authored icon system.
- Nút In/Load/Save/X, click thumbnail, right-click filtered capture/load và nút thêm named version dùng cùng typed actions; hoàn thiện hit targets, tooltip và keyboard focus.
- Thumbnail từ nguồn snapshot qua GPU/offscreen hoặc capture pipeline hiện có, camera xác định và source revision riêng. Cache giới hạn, worker/cancel/stale guards khi cần; thumbnail đến chậm không khóa dữ liệu đã lưu hoặc thay CAD source.
- Hoàn thiện layout responsive, floating/collapse/reorder và nền/selection/readability theo ảnh; kiểm UI một lượt ở P07. Ảnh tham khảo chỉ để đối chiếu, không ship bitmap nguyên bản.

### P05 Row archive và quản lý project

Ưu tiên P1, sau P02 và action contract P00. Giải G12/G13/G15.

- Service create/select/rename/delete/update, visible/hidden, Compact, Timer Start/Stop/Reset. Persist workspace catalog qua New/Open/restart.
- Per-bag Save As và row archive/load/export gọi cùng codec/3DM adapters; native archive đầy đủ được ưu tiên. `.mpj` compatibility là capability riêng sau khảo sát format, không đổi extension ZIP để giả tương thích.
- File operations atomic, đường dẫn thuộc root hoặc đích người dùng chọn; import lỗi/duplicate và Cancel không làm mất row hiện tại. Phân biệt delete row view và xóa file thật, có xác nhận đúng hành vi.
- Hoàn thiện tên Unicode, archive/import, timer và xử lý lỗi file; không tự gửi dữ liệu hoặc dùng cloud sync để thay persistence. Gom kiểm luồng hoàn chỉnh vào P07.

### P06 Mngr và metadata tìm kiếm

Ưu tiên P2, sau catalog P02/P05. Giải G16/G17/G18.

- Mngr qua nút và aliases gốc đã xác minh; Files/Info, preview, search/reset, hide/show all và row visibility, Refresh/Import Project/Import Folder/Export/Open Folder/Change Root.
- Typed metadata theo Book 1: customer/designer/status/tags, created/due dates, job number, CAD/CAM labor, ring size, metal/gem weight, description. Ghi rõ unit và dữ liệu nhập tay so với dữ liệu tính toán; không tự lấy weight từ mesh thumbnail.
- Tái dùng service Rust, chọn storage tối giản sau xác định dữ liệu/query; không buộc dùng database engine/COM/SQL Server của bản gốc chỉ vì decompile có DAL.
- Hoàn thiện filters, metadata save/cancel, refresh/import/export và workspace restart; visibility không xóa dữ liệu. Kiểm cùng luồng Projects ở P07, không mở một đợt test riêng cho mỗi filter.

### P07 Một đợt kiểm thử sau khi triển khai xong

- Chỉ bắt đầu khi P00–P06 đã triển khai và tích hợp xong. Tái dùng tests hiện có, sửa những assertion đang khóa hành vi cũ; bổ sung hồi quy cần thiết cho sáu gap đã tái hiện. Không tạo test cho từng helper hoặc lặp lại toàn bộ audit.
- Chạy một bộ gộp tập trung vào ba luồng: **capture/insert đúng geometry và dependencies**, **snapshot/archive save→reopen→restore cùng lỗi/cancel/Undo**, **Projects và Mngr từ giao diện đến dữ liệu**. Dùng trường hợp đại diện có placement, source đã xóa, resource/instance, nhiều ô và đủ năm nhóm; kiểm hai mức display khi ảnh hưởng geometry, cùng các controls/filter chính và layout/DPI liên quan.
- Chạy native/SDK checks chỉ cho capability thực sự dùng hoặc công bố, rồi một lượt desktop cho luồng hoàn chỉnh. Ghi chung kết quả, source/binary hash và phần chưa đạt; không lập batch nghiệm thu theo từng gói nhỏ.
- Nếu đợt cuối phát hiện lỗi, sửa rồi chỉ chạy lại nhóm bị ảnh hưởng; chỉ lặp cả bộ khi thay đổi nền chung làm kết quả trước mất hiệu lực.
- Build dùng `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`, tối đa hai target. Nếu executable đang mở, giữ phiên người dùng; chạy test target hoặc runner phù hợp, không tự tắt ứng dụng để thay binary.
- Cập nhật L2/L3/L4 `runtime_implementation`, traceability, README, live ledger và checklist theo đúng scope; giữ manual M01-UI unsigned. Native 3DM checks không tự là measurement batch openNURBS; nếu thực hiện measurement thì cập nhật roadmap theo AGENTS.
- Public-source audit và prune runs sau batch. Sau coding có đổi nghiệp vụ, trình bày before/after và skill text để xin lưu theo consent policy. Lượt plan này không đổi skill hoặc source sản phẩm.

## Thứ tự bắt đầu

**P00 → P01 + P02 → P03/P04 → P05 → P06 → P07 kiểm thử gộp cuối**. Task coding đầu tiên là **G01/G02** trong `ProjectManager::restore_entities`, `Snapshot::available` và Controller insert: sửa snapshot độc lập với source và giữ placement, rồi tiếp tục các phần còn lại. Resource codec G04 được giải quyết trong P02. Chưa báo hoàn tất sản phẩm cho đến khi đợt kiểm thử cuối xác nhận phạm vi bàn giao.

## Kết quả kiểm tra của lượt audit

| Kiểm tra | Kết quả |
|---|---|
| Hai binary integration vừa dựng | `project_manager_ui`: 3 passed; `project_manager_persistence`: 5 passed. Có hash trong receipt; UI là egui headless, không phải desktop. |
| Lệnh Cargo integration tổng thể | Không kết thúc thành công: không thể thay `L:/om9-target/debug/om9.exe` vì access denied. Hai test executable vẫn được tạo và đã chạy trực tiếp. |
| `cargo test -p om9-app --lib project_manager::tests -j 2` | Không chạy test: lib-test build gặp 318 lỗi compile trong tập fixture/API BRep và các test khác; log được giữ, chưa sửa ngoài scope. |
| Harness Rust model riêng | Compile module production nguyên trạng và chạy được. Sáu ca chẩn đoán xác nhận placement sai, snapshot bị UI khóa sau xóa source, rename-only không được predicate stage, notes không clear, Master resource mất payload sau JSON, duplicate ProjectId decode được chấp nhận. Đây là gap reproductions, không phải sáu test sản phẩm đã pass. |
| Desktop/GPU, jewelry BRep, instance/resource bundle đầy đủ | Chưa nghiệm thu trong lượt audit; point/notes pass không phủ các trường hợp này. |

Log, kết quả probe, hash và public-source audit ở [receipt](../validation/2026-11-17-project-manager-audit.json). Công việc đã hoàn thành trong lượt này là đối chiếu và lập plan, không phải hoàn thiện Project Manager.
