# Hoàn thành tách nguồn hình học và viewport — M01 đến M09

## Bổ sung được người dùng duyệt: lát cắt đo hai cầu, 2026-11-14

Người dùng yêu cầu triển khai đủ Sphere → Sphere → Boolean Union để đo hai mức viewport thủ công, chọn **Nhẹ làm mặc định**, rồi yêu cầu thực hiện kế hoạch. Phạm vi này được phép kiểm thử và đo runtime, thay hướng dẫn compile-only bên dưới cho riêng lát cắt đó. Hai mức giữ nguồn BRep cho Boolean; mesh hiển thị và tham số tessellation riêng. Bằng chứng 9 test và 8 lượt native-window release tại [hồ sơ viewport detail](../../../../docs/validation/2026-11-14-viewport-detail-sphere-boolean.md). Không coi lát cắt này là hoàn tất C1–C5, codec v5, NativeBrep hay các consumer khác.

## Chỉ đạo thực hiện hiện hành — 2026-11-14

**Làm xong toàn bộ phần tách lõi và tích hợp trước. Người dùng tự kiểm thử các lệnh sau đó; lỗi phát hiện được sẽ được truy về và sửa phần tách lõi tương ứng.** Chỉ đạo này thay mọi trình tự kiểm thử sau từng helper, từng lệnh hoặc từng gói trong checkpoint cũ.

- Phạm vi thực hiện: C1 → C2 → C3 cùng toàn bộ thay đổi consumer C4 và phần I1 liên quan; kết thúc bằng bàn giao C5. Không dừng sau một helper, một worker hoặc một nhóm lệnh để mở đợt kiểm thử.
- Trong lúc triển khai chỉ chạy kiểm tra biên dịch cần thiết để nối API và sửa lỗi tích hợp. Không thêm/chạy test hàm, test lệnh, bộ hồi quy hay benchmark trong đợt này.
- C1: Document/Undo chỉ sở hữu nguồn canonical; tách kết quả kernel tạm thành nguồn và representation; scene/cache giữ representation với pin và quota riêng. Bao gồm OCCT và NativeBrep; giữ mesh nguồn và disposition rõ cho payload opaque.
- Làm rõ ý định người dùng 2026-11-14: viewport phải có mesh hiển thị nhẹ, ít tam giác hơn khi mức chi tiết đó đủ để quan sát; tính toán và xuất vẫn dùng nguồn chính xác với dung sai riêng. Chỉ chuyển mesh hiện tại sang cache khác chưa đáp ứng ý định này.
- C2: Open/Undo/Redo dựng representation thiếu trên worker, chuẩn bị scene rồi publish nguyên tử; eviction chỉ nhả cache không pin.
- C3: codec v5 canonical-only; đường đọc/migration v4 giữ dữ liệu nguồn; v5 không ghi display dẫn xuất.
- C4: chuyển hết consumer đang bật chịu ảnh hưởng sang đúng nguồn hoặc representation; không mở dự án cải tiến từng lệnh ngoài nhu cầu tích hợp.
- C5: bàn giao code tích hợp, cách build/chạy, phạm vi đã chuyển và phần còn thiếu. Ghi rõ “đã triển khai, chờ người dùng kiểm thử” nếu chưa có bằng chứng runtime; biên dịch thành công không đồng nghĩa nghiệm thu hay đạt gate M09.
- Kế hoạch này không chứa danh sách test lệnh, ma trận nghiệm thu hoặc số lượt đo. Không tự mở lại các phần đó từ tài liệu lịch sử hay chỉ mục L4.

**Trạng thái: đang triển khai, chưa tách xong toàn bộ lõi.** Geometry OCCT và NativeBrep đã chuyển sang kiểu nguồn không chứa display; import và scene đang nối representation store. Kiểm tra biên dịch workspace và native adapter đã qua, chưa có bằng chứng runtime. Open/Undo/Redo khi thiếu cache, migration v4/v5, consumer native chính xác và I1 còn mở; encoder v5 tạm từ chối tài liệu có BRep cho đến khi chứng thực topology và disposition payload hoàn tất. Open chọn v4 hoặc v5, giữ display cũ trong candidate v4. Chi tiết: [ledger kỹ thuật](../../../../docs/validation/2026-11-14-canonical-representation-transfer.md#v5-container-and-versioned-open-in-progress).

Checkpoint cũ giữ tại [hồ sơ lịch sử](SOURCE_DISPLAY_CHECKPOINTS_BEFORE_CORE_MIGRATION_20261114.md), chỉ để tra bằng chứng và lỗi đã ghi nhận; không là hàng đợi thực hiện hiện hành.

## Kết quả rà và phạm vi thay thế

- [Chỉ mục L4 đầy đủ](../../../../docs/validation/2026-11-13-source-viewport-m01-m09-l4-index.md) kiểm kê 615 L3 và 2.686 L4, gồm đủ 30 L4 release gate M09. JSON giữ ID, đường dẫn, positive/negative hoặc tiêu chí theo schema khác, dependencies và ô kết quả NOT_RUN. Quét tiêu chí không chứng minh runtime hay mọi tính năng đã bật.
- M09 không có owner L2/L3: lấy L4 từ manifest G-MVP/G-FULL gốc, không tạo ID giả. M01 STL L3 hiện không có file L4 con; cần bổ sung thiết kế truy vết trước khi ký đúng mục đó.
- Plan cũ A1/A2/A3 giữ bằng chứng lịch sử. B0 đã hoãn một thử nghiệm **Sphere history-only** vì cận trên lợi ích dưới ngưỡng 32 MiB. Kết quả đó vẫn đúng cho workload đã đo, nhưng không là gate lợi ích cho mục tiêu core rộng hơn đang được lập ở đây. Không chạy lại workload 128 Sphere kéo dài chỉ để tìm số có lợi. B1–B3 cũ vẫn chưa thực hiện; trình tự dưới đây thay phương án ColdSphere riêng bằng tách nguồn/cache dùng chung khi bắt đầu đợt implementation mới.
- Tách nguồn khỏi viewport không đòi dựng một kernel mới hoặc port runtime FreeCAD/Blender. Rust tiếp tục own document/history/cache/workers/UI; native chỉ adapter OCCT/openNURBS bắt buộc.

## Điểm xuất phát và phần triển khai còn lại

| Vị trí | Điều hiện có | Phần phải hoàn thành |
|---|---|---|
| `crates/om9-core/src/brep.rs` | Đã bắt đầu đổi geometry OCCT sang `CanonicalBrep`; `Brep` có display là kết quả kernel tạm | Hoàn tất handoff, accounting và lifetime của representation; canonical equality/serialization chỉ tính nguồn |
| `crates/om9-core/src/geometry.rs` | `NativeBrep` chứa payload cùng `Arc<Mesh>` display | Áp cùng ranh giới cho native payload có thể dựng lại; không bỏ payload opaque hoặc mesh nguồn |
| `crates/om9-core/src/transaction.rs` | UndoEntry giữ before/after `Arc<Document>`; undo_with/redo_with chuẩn bị trước publication; source lease/proof và accounting đã có | Snapshot canonical không kéo display sống; giữ matching/equality dữ liệu tài liệu và publication nguyên tử |
| `crates/om9-scene/src/scene.rs`, `session/topology_remesh.rs` | Scene buffers/pins, nguồn có lease, worker và freshness checks; đang nối representation store | Hoàn tất mọi đường capture/publish; cache miss có đường dựng lại thật, owner cuối trả đúng charge |
| `crates/om9-archive/src/container.rs` | Header v5 và phân nhánh Open v4/v5 đã nối; v5 tạm từ chối ghi BRep | Hoàn tất codec v5 canonical-only cho BRep, migration v4 và load/rebuild trước publication |
| `crates/om9-kernel/src/brep_snapshot.hxx` và `brep_bridge.cpp` | Các đường OCCT đã đọc có Write với flags không ghi triangulation/normals | Tái dùng, kiểm lại từng serializer source; không khẳng định mọi payload native đều loại display |
| `session/wireframe.rs` | Isocurve mặc định của mặt NURBS riêng không trim; B-rep/NativeBrep vẫn trích topology edges | Wireframe/isocurve theo mặt nguồn, trim và density của representation cho B-rep |

## Mô hình đích và điều kiện hoàn tất triển khai

Document/Undo chứa **hình học canonical**: source, topology/semantic references, payload cần thiết, units/tolerance, placement, attributes, instance/definition và feature recipe/history theo hợp đồng. Chúng không giữ mesh/đường viewport dẫn xuất. Mesh do người dùng tạo/nhập và dữ liệu facet có ý nghĩa hình học vẫn là nguồn; không xóa chúng vì có cùng dạng mảng với display.

Display ở **representation store/cache do Rust quản lý**, không nằm trong `Brep` canonical. Khóa gồm source identity/revision, topology binding, display profile và version adapter; occurrence placement/camera chỉ tham gia khi representation thực sự phụ thuộc chúng. Scene/presentation/pick/worker có lease/pin thực với lifetime và account; record cache không được giữ một Document ẩn có display. Cache miss là trạng thái được xử lý, không là geometry rỗng giả.

Viewport dùng **profile tessellation riêng để tạo mesh ít tam giác**, thay vì bắt buộc vẽ mesh dày phục vụ một dung sai tính toán/xuất. Ví dụ hình cầu chính xác được giữ bằng biểu diễn hình học nguồn/BRep; số tam giác của mesh viewport chỉ quyết định độ mịn khi nhìn. Nguồn chính xác không bắt buộc là một mesh dày. Đo đạc, Boolean và xuất phải lấy nguồn hoặc dựng mesh theo dung sai của tác vụ, không lấy mesh viewport thưa làm kết quả chính xác.

Profile hiển thị phải có tham số sai số/độ mịn và giới hạn tài nguyên rõ; giữ binding về mặt/cạnh nguồn và hiệu chỉnh pick/snap về nguồn khi cần. Với mesh là dữ liệu nguồn, chỉ bản hiển thị dẫn xuất được đơn giản hóa; không thay các tam giác nguồn. Bàn giao ghi riêng việc đã nối profile mesh nhẹ, số đỉnh/tam giác của representation khi có bằng chứng, và hiệu năng runtime chưa đo. Không tự hứa tỷ lệ giảm tam giác hay tăng FPS cho mọi mô hình; giai đoạn triển khai vẫn chỉ kiểm tra biên dịch theo chỉ đạo hiện hành.

Operation adapter có thể trả nguồn mới kèm display đã chuẩn bị trong **kết quả tạm**; caller tách nguồn để commit và representation để publish. Không tạo hai nguồn sự thật. Tên struct/API dưới đây là trách nhiệm thiết kế, không khẳng định API đã có. Geometry canonical chỉ có một representation nguồn; tránh Full/ColdSphere trở thành hai model tài liệu lâu dài.

Undo/Redo và Open có thể cần worker dựng representation. Giữ world/scene đang công bố đến khi candidate đủ geometry, topology binding và scene hợp lệ. Failure/cancel/stale/budget không đổi document, stack, selection hoặc file đích; không giữ account lock qua native call. Các bước cập nhật cache không thêm Undo, đổi source hoặc đánh dấu dirty. B-rep/NativeBrep không có khả năng dựng lại chứng thực được phải có disposition tương thích rõ trước migration, không âm thầm xóa object hoặc thay bằng mesh.

**Triển khai chỉ hoàn tất khi**: geometry canonical không own display; Undo không pin cache gián tiếp; source-only restore qua caller thật; eviction giải phóng allocation không pin và dựng lại dùng được; v5 không serialize display dẫn xuất; nhập v4 không mất dữ liệu; wireframe/isocurve B-rep có nguồn/trim/binding đúng ở phạm vi cam kết; các downstream đã bật chịu ảnh hưởng đã được nối đúng API. Bàn giao phải ghi riêng trạng thái triển khai và trạng thái chưa được người dùng kiểm thử; không suy ra đã đạt gate M09 hay đã hoàn tất mọi tính năng M01–M09.

## Các bàn giao và thứ tự

| Gói | Đầu ra triển khai | Phụ thuộc | Điều kiện hoàn tất triển khai |
|---|---|---|---|
| C0 — Khóa migration và caller | Chốt canonical/cache key, ownership, legacy topology, codec v5 và danh sách caller đang bật phải chuyển | Rà hiện tại | Đủ quyết định để sửa code; không mở audit toàn ứng dụng vô hạn |
| C1 — Canonical và representation store | Tách display khỏi OCCT và NativeBrep geometry; nối kernel-result/scene consumers, source lease, equality, account và profile tessellation nhẹ riêng cho viewport | C0 | Canonical Document/Undo không own display, kể cả qua compatibility field; scene/pick/worker pin đúng representation; viewport có đường tạo mesh thưa theo profile riêng, không chỉ đổi owner của mesh cũ |
| C2 — Source-only restore và eviction | Nối dựng lại display trên worker vào Open/Undo/Redo; thu hồi cache không pin theo recency/admission hiện có | C1 | Luồng cache miss, success/cancel/stale/budget và atomic publication được tích hợp; không cần giữ display trong Undo |
| C3 — `.om9` v5 canonical-only | Codec v5 không ghi mesh/wires viewport; parser/migration v4 riêng; load dựng scene từ nguồn; atomic save | C1; dùng C2 | Header/schema đúng; giữ nguồn, mesh nguồn, IDs theo disposition, trim, placement, attributes, block, resource và recipe |
| I1 — Nối wireframe/isocurve | Wires từ mặt nguồn, outer/inner trim và placement; scene và ExtractWireframe dùng cùng parameter/density | C1–C4 theo phần chịu ảnh hưởng | Biên topology và iso hiển thị có owner/binding riêng; payload opaque chưa có khả năng dựng mặt được xử lý rõ và giữ nguồn |
| C4 — Chuyển consumer M01–M08 | Chuyển hết caller đang bật chịu ảnh hưởng sang query nguồn, scene cache hoặc preview đúng owner; giữ menu/CMD/SDK dùng chung implementation | C1–C3; I1 khi liên quan | Không còn caller phụ thuộc display trong canonical; không port tính năng chưa triển khai chỉ để mở rộng phạm vi |
| C5 — Bàn giao triển khai | Ghi code đã tích hợp, hướng dẫn build/chạy, giới hạn và phần còn thiếu | C1–C4, I1 | Hết việc triển khai trong phạm vi; phân biệt rõ bàn giao code với nghiệm thu runtime và gate M09 |

I1 giữ phạm vi mật độ mặc định 1 đã có căn cứ; không mở rộng mật độ khác trong đợt tách lõi này. Thiết kế giữ đường truyền density theo object/tài liệu và đồng bộ scene/extraction; phần chưa hỗ trợ được ghi rõ, không suy diễn parity 3DM density. NativeBrep opaque không được chứng thực mặt phải từ chối có lý do và giữ nguồn.

Mật độ isocurve của I1 là số đường vẽ trên mặt, khác với độ mịn/số tam giác của mesh viewport; giữ density 1 không ngăn dùng profile tessellation nhẹ cho mesh hiển thị.

C1 chuyển qua adapter nội bộ tạm theo từng consumer; dùng kiểm tra biên dịch để sửa tích hợp, không chuyển sang test từng consumer; **không đóng C1 khi canonical vẫn giữ display qua compatibility field**. Không dùng `Option<display>` trong model làm đích cuối để chỉ né compile errors. Không dùng hash/UUID đơn lẻ thay topology attestation, hoặc so source text làm phép bằng nhau của native topology chưa chứng thực.

C2 áp dụng cho cache có thể tái tạo, gồm history-only không pin; active scene/presentation được pin trong đợt này. Không thu hồi representation đang vẽ; không thêm interaction với geometry thiếu hình trên scene đã công bố. Rigid transform/camera không tự tessellate lại hình học local. Không có timer quét mọi frame, native mutable shape cache chung hoặc cache disk mới nếu không có nhu cầu đo được.

C3 giữ import resources/provenance/layer/definition/recipe, không chỉ BRep source. Không serialize danh sách Undo mới vào v5 nếu lifecycle dự án không có hợp đồng đó; Open tạo phiên Undo theo lifecycle hiện hành. Parser v4 riêng đọc snapshot đầy đủ tạm, admission overlap source/old-display/new-cache, chuyển binding có kiểm chứng rồi giải phóng old display. Không ghi v5 dưới header v4; app cũ từ chối rõ file version mới. Payload opaque cần adapter strip/canonicalize được chứng minh giữ nguyên dữ liệu cần thiết; nếu cache nằm trong payload chưa tách an toàn được, ghi blocker C3 thay vì tuyên bố source-only hoàn tất.

## Tham chiếu phạm vi consumer từ M01 tới M09

| Mốc | Phần tích hợp chịu ảnh hưởng | Điểm L4/owner tham chiếu |
|---|---|---|
| M01 | Model/geometry handle; tessellation và cache/pin; transactions/Undo/Cancel; codec và compatibility; STL theo source | `OM9-L4-01-03-003-05`, `OM9-L4-01-05-005-05`, `OM9-L4-01-05-008-05`, `OM9-L4-01-06-002-05`, `OM9-L4-01-06-003-05`, `OM9-L4-01-06-004-05`, `OM9-L4-01-02-006-05`, `OM9-L4-01-02-007-05`, `OM9-L4-01-12-007-05`; STL L4 thiếu phải bổ sung riêng |
| M02 | Pick/snap binding, camera/scene epoch, Gumball placement, ghost/previews và stale selection | `OM9-L4-01-07-001-05`, `OM9-L4-02-03-004-05`; các L4 Gumball trong index |
| M03 | Các lệnh nguồn đã làm; isocurve/wireframe; transformation/boolean; 3DM editable/opaque/blocks; đo đạc/xuất | `OM9-L4-03-16-039-03`, `OM9-L4-03-16-040-03`, `OM9-L4-03-16-020-03`, `OM9-L4-03-16-054-03`, `OM9-L4-03-18-001-03`, `OM9-L4-03-18-004-03`; đầy đủ lệnh khác theo index |
| M04 | Recompute/History rollback giữ nguồn cũ và viewport; preview khác commit; stable dependency refs; khối lượng từ nguồn | `OM9-L4-01-04-007-05/06/07` (ba ID cụ thể trong index), `OM9-L4-04-19-001-05/06/07`; không lấy mesh display để cân kim loại |
| M05 | Gem asset mesh/BRep theo semantics; đặt đá theo mặt nguồn/normal/trim, instance sharing và history/ID | `OM9-L4-10-007-03` là ID thực của Gem On Surface; không tự thêm tiền tố 04 theo suy đoán; nguồn facet có thể là mesh |
| M06 | Head/Bezel/Prong trên mặt/girdle; hình nguồn và preview cập nhật cùng lúc; không tự chuyển mọi mesh setting thành BRep | `OM9-L4-14-009-04`, `OM9-L4-15-003-04` và các L4 setting trong index |
| M07 | Cutter/interactive Boolean dùng nguồn; Keep/Remove, watertight và rollback; sau cache miss không cắt bằng GPU mesh | `OM9-L4-04-16-006-03`, `OM9-L4-04-16-006-04`; đọc geometry_invariants/error_cases theo schema M07 |
| M08 | Smartflow rigid giữ hình đá; nonrigid sửa nguồn có chủ ý; preview không ghi persistent BRep; UV/normal/trim, Pattern/Rope/library roundtrip | `OM9-L4-04-05-003-03/04`, `OM9-L4-04-08-008-03/04`, `OM9-L4-05-01-005-03/04`, `OM9-L4-05-02-005-03/04`; từng ID đầy đủ ở index |
| M09 | Giữ truy vết phạm vi bàn giao; không tự ký gate phát hành từ việc hoàn tất triển khai | G-MVP 006 kernel, 007 history, 008 save, 009 3DM, 010–012 downstream, 013 STL, 014 recovery/perf, 015 desktop; G-FULL 003–010; dùng đủ 30 leaf/manifest, không bỏ governance/signoff |

Các ký hiệu `-05/06/07` trong bảng là cách viết ngắn các ID chung prefix; tra exact ID/path trong index JSON. Bảng và chỉ mục L4 chỉ xác định owner/phạm vi tích hợp, không tạo hàng đợi test lệnh hay điều kiện chặn triển khai trong đợt này.

## Thứ tự tiếp tục và điểm bàn giao

1. Hoàn tất C1: canonical OCCT/NativeBrep, representation store, chuyển kernel-result và accounting; không giữ display trong model để né lỗi biên dịch.
2. Hoàn tất C2: worker dựng lại representation khi thiếu, nối Open/Undo/Redo và eviction/pin; giữ world/scene cũ đến khi candidate sẵn sàng.
3. Hoàn tất C3: codec v5, parser/migration v4 và vòng đời load/save. Payload opaque chưa tách an toàn phải được xử lý hoặc ghi phần việc chưa hoàn tất, không báo source-only thành công.
4. Hoàn tất C4 và I1 liên quan: chuyển hết caller đã bật chịu ảnh hưởng; query chính xác dùng nguồn, viewport/pick/preview dùng representation với binding phù hợp.
5. Bàn giao C5 khi hết phần triển khai và tích hợp; ghi rõ trạng thái build, giới hạn và trạng thái chờ người dùng kiểm thử. Sau đó xử lý lỗi người dùng phản hồi ở đúng lõi/consumer chịu ảnh hưởng.

Trong đợt triển khai chỉ kiểm tra biên dịch cần thiết; tái dùng tối đa hai target build hiện có, `-j 2`, target ngoài checkout theo AGENTS. Không cần build hoặc chạy test để xác nhận riêng bản sửa plan này.
