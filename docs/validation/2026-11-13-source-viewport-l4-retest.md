# Kiểm lại nguồn hình học và viewport theo L4 — 2026-11-13

Đây là checklist thao tác 19 mục ban đầu M01–M03. [Chỉ mục mở rộng M01–M09](2026-11-13-source-viewport-m01-m09-l4-index.md) bao phủ toàn bộ cây L4 và các gate M09; giữ checklist này làm phần thao tác có phạm vi, không coi nó là toàn bộ milestone.

**Core chưa hoàn tất toàn bộ.** Nguồn/topology và display có cấu trúc riêng; một số lệnh đã tính theo nguồn. Nhưng Brep vẫn giữ Arc display, Undo giữ snapshot document và codec v4 ghi cả source/display. Source-only restore/eviction B1–B3 đang hoãn. Điều này không tự chứng minh các phép CAD dùng sai mesh.

Đây là checklist phần hiện có, không phải biên bản nghiệm thu hoặc lệnh triển khai ngay. Toàn bộ kết quả lượt này là NOT_RUN. M01 thường dùng L4-05 acceptance, M02 có L4-05 regression/stress; các lệnh M03 ở đây dùng L4-03 acceptance.

## Cách dùng

- Ghi build, fixture, camera, chế độ hiển thị và dung sai; dùng bản vẽ thử riêng. Không tạo thêm target build.
- Chạy menu và CMD trên cùng input gốc ở hai lượt độc lập; kiểm hình học nguồn và viewport/selection. Không chỉ đếm object.
- Positive/negative lấy từ L4 gốc, link bên dưới; thao tác là hướng dẫn áp dụng cho standalone OM9. QymCAD/API đề xuất trong tài liệu nhập không thay target hiện hành.
- Đọc prerequisites/receipt trong JSON đi kèm. Đóng toàn L4 cần dependency và reviewer đúng tiêu chí gốc; checklist này không tự đóng các bước thiết kế/tích hợp L4 khác.
- Ghi PASS/FAIL/NOT_RUN/UNAVAILABLE, actual, evidence và reviewer. Nếu chỉ kiểm input hỗ trợ, ghi SCOPED; không coi toàn L4 PASS.
- ID, cache hits, topology, volume, lỗi mô phỏng và memory cần kiểm tự động. Desktop không thay được các bằng chứng đó.

## Danh sách kiểm

### 1. OM9-L4-01-03-003-05

[L4 gốc](<../../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-03 - Be Document Scene Entities/L3-003 - Geometry Handle And Attributes/L4-05 - Acceptance Test/l4.json>) — Hồi quy, thủ công và bàn giao — Geometry Handle And Attributes

- [ ] Positive: Entity co GeometryHandle va attributes name/color/visible co the query rieng
- [ ] Negative: Missing geometry handle hoac dangling handle khong bi render/serialize nhu object hop le
- Thao tác/kiểm chứng: Cần test tự động của owner để kiểm source/topology/handle/volume và lỗi; quan sát desktop chỉ bổ trợ.
- Kết quả: **NOT_RUN**. Actual / bằng chứng / reviewer: …

### 2. OM9-L4-01-05-005-05

[L4 gốc](<../../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-05 - Be Occt Brep Kernel Bridge/L3-005 - Face Wire Edge Topology Access/L4-05 - Acceptance Test/l4.json>) — Hồi quy, thủ công và bàn giao — Face Wire Edge Topology Access

- [ ] Positive: Box 10x20x30 face/wire/edge enumeration distinguishes persistent refs
- [ ] Negative: Index-based subshape access after topology change must not silently pick wrong face
- Thao tác/kiểm chứng: Cần test tự động của owner để kiểm source/topology/handle/volume và lỗi; quan sát desktop chỉ bổ trợ.
- Kết quả: **NOT_RUN**. Actual / bằng chứng / reviewer: …

### 3. OM9-L4-01-05-006-05

[L4 gốc](<../../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-05 - Be Occt Brep Kernel Bridge/L3-006 - OCCT Transform Matrix Service/L4-05 - Acceptance Test/l4.json>) — Hồi quy, thủ công và bàn giao — OCCT Transform Matrix Service

- [ ] Positive: Transform box translation(1,2,3) moves bbox with volume still 6000
- [ ] Negative: Singular affine transform/NaN matrix refused without state damage
- Thao tác/kiểm chứng: Cần test tự động của owner để kiểm source/topology/handle/volume và lỗi; quan sát desktop chỉ bổ trợ.
- Kết quả: **NOT_RUN**. Actual / bằng chứng / reviewer: …

### 4. OM9-L4-01-05-008-05

[L4 gốc](<../../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-05 - Be Occt Brep Kernel Bridge/L3-008 - Shape Tessellation Interface/L4-05 - Acceptance Test/l4.json>) — Hồi quy, thủ công và bàn giao — Shape Tessellation Interface

- [ ] Positive: OCCT box tessellates with nonempty normals+triangle IDs; BRep volume remains 6000
- [ ] Negative: Bad tessellation tolerance or missing BRep must not replace source truth
- Thao tác/kiểm chứng: Cần test tự động của owner để kiểm source/topology/handle/volume và lỗi; quan sát desktop chỉ bổ trợ.
- Kết quả: **NOT_RUN**. Actual / bằng chứng / reviewer: …

### 5. OM9-L4-01-06-002-05

[L4 gốc](<../../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-06 - Febe Viewport Gpu Draw Backend/L3-002 - Shared Scene Graphics Resources/L4-05 - Acceptance Test/l4.json>) — Hồi quy, thủ công và bàn giao — Shared Scene Graphics Resources

- [ ] Positive: Two viewports share geometry mesh buffers but maintain camera state independently
- [ ] Negative: Deleting entity invalidates GPU resource safely; no use-after-free
- Thao tác/kiểm chứng: Giao diện: hai viewport cùng Box, đổi camera từng view, xóa rồi Undo/Redo; không hình ma/crash. Dùng chung buffer cần kiểm tự động.
- Kết quả: **NOT_RUN**. Actual / bằng chứng / reviewer: …

### 6. OM9-L4-01-06-003-05

[L4 gốc](<../../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-06 - Febe Viewport Gpu Draw Backend/L3-003 - NURBS B-Rep Display Tessellation Cache/L4-05 - Acceptance Test/l4.json>) — Hồi quy, thủ công và bàn giao — NURBS B-Rep Display Tessellation Cache

- [ ] Positive: Same BRep revision hits tessellation cache twice; geom edit invalidates once
- [ ] Negative: Changed tolerance or geometry revision must not use stale cache entry
- Thao tác/kiểm chứng: Kiểm cache hit/invalidation bằng test hoặc bộ đo. UI bổ trợ: sửa hình phải cập nhật scene. Chưa xác nhận control đổi deflection trong UI; ghi UNAVAILABLE nếu thiếu.
- Kết quả: **NOT_RUN**. Actual / bằng chứng / reviewer: …

### 7. OM9-L4-01-06-004-05

[L4 gốc](<../../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-06 - Febe Viewport Gpu Draw Backend/L3-004 - Wireframe And Isocurve Pass/L4-05 - Acceptance Test/l4.json>) — Hồi quy, thủ công và bàn giao — Wireframe And Isocurve Pass

- [ ] Positive: Wireframe edges and isocurves appear separately from shaded geometry
- [ ] Negative: Display isocurve must not be selectable as BRep edge
- Thao tác/kiểm chứng: Mặt NURBS phẳng/trụ không trim: đổi Wireframe/Shaded, kiểm đường nội bộ và biên. Thử chọn isocurve: không được nhận là cạnh topology B-rep. B-rep interior wires tổng quát còn thiếu, chưa đủ đóng toàn L4.
- Kết quả: **NOT_RUN**. Actual / bằng chứng / reviewer: …

### 8. OM9-L4-01-02-006-05

[L4 gốc](<../../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-02 - Be Command Dispatch Undo Redo/L3-006 - Atomic Command Transactions/L4-05 - Acceptance Test/l4.json>) — Hồi quy, thủ công và bàn giao — Atomic Command Transactions

- [ ] Positive: One UI action creates one transaction and one revision even with multiple objects
- [ ] Negative: Mid-operation failure rolls back every change and preserves undo journal
- Thao tác/kiểm chứng: ExtractWireframe Box 10x20x30: 12 cạnh, một Undo bỏ toàn bộ output và Redo khôi phục. Lỗi giữa batch kiểm tự động để xác nhận document/history giữ nguyên.
- Kết quả: **NOT_RUN**. Actual / bằng chứng / reviewer: …

### 9. OM9-L4-01-02-007-05

[L4 gốc](<../../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-02 - Be Command Dispatch Undo Redo/L3-007 - Undo Redo Journal/L4-05 - Acceptance Test/l4.json>) — Hồi quy, thủ công và bàn giao — Undo Redo Journal

- [ ] Positive: Create object -> undo removes -> redo restores stable ID and layer
- [ ] Negative: Undo after delete/reorder must not replay corrupted references
- Thao tác/kiểm chứng: Tạo Box, Undo/Redo, xóa rồi Undo; kiểm hình và layer. ID/tham chiếu nội bộ cần so tự động.
- Kết quả: **NOT_RUN**. Actual / bằng chứng / reviewer: …

### 10. OM9-L4-01-02-008-05

[L4 gốc](<../../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-02 - Be Command Dispatch Undo Redo/L3-008 - Command Cancel Rollback/L4-05 - Acceptance Test/l4.json>) — Hồi quy, thủ công và bàn giao — Command Cancel Rollback

- [ ] Positive: Preview geometry canceled with Escape restores document and no undo entry
- [ ] Negative: Cancel after partial state or C++ error must not leave orphan preview entity
- Thao tác/kiểm chứng: Bắt đầu Sphere/Box, nhập điểm có preview rồi Esc: không còn preview hoặc bước Undo mới. Nhánh lỗi native kiểm tự động.
- Kết quả: **NOT_RUN**. Actual / bằng chứng / reviewer: …

### 11. OM9-L4-01-12-007-05

[L4 gốc](<../../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-12 - Be Native Project Codec Stl Export/L3-007 - Document Reload And Restoration/L4-05 - Acceptance Test/l4.json>) — Hồi quy, thủ công và bàn giao — Document Reload And Restoration

- [ ] Positive: Open saved project restores IDs/layers/instances and active document state
- [ ] Negative: Failed load leaves current unsaved document untouched
- Thao tác/kiểm chứng: Lưu/mở Box, trụ và đường trích; kiểm geometry/scene/chọn cạnh. Kiểm Undo/Redo của thao tác mới sau mở; không giả định .om9 lưu lịch sử Undo.
- Kết quả: **NOT_RUN**. Actual / bằng chứng / reviewer: …

### 12. OM9-L4-01-07-001-05

[L4 gốc](<../../OM9_Milestones/M02 - Viewport, Gumball/L2-01-07 - Febe Picking Selection Generic Handles/L3-001 - Screen Ray And Hit Test Query/L4-05 - Regression and Stress Test/l4.json>) — REGRESSION_STRESS: Screen Ray And Hit Test Query

- [ ] Positive: Box tại tâm tia, điểm click ở viewport P trúng BodyId của box
- [ ] Negative: Ray không giao đối tượng trả danh sách rỗng, không chọn
- Thao tác/kiểm chứng: Bấm Box trong Perspective và vùng trống: chọn đúng object, không hit giả. Thêm hai object chồng nhau kiểm bảng chọn.
- Kết quả: **NOT_RUN**. Actual / bằng chứng / reviewer: …

### 13. OM9-L4-02-03-004-05

[L4 gốc](<../../OM9_Milestones/M02 - Viewport, Gumball/L2-02-03 - Febe Osnaps And Basic Smarttrack/L3-004 - Osnap Surface And Polysurface/L4-05 - Regression and Stress Test/l4.json>) — REGRESSION_STRESS: Osnap Surface And Polysurface

- [ ] Positive: Đối với mặt phẳng BRep, nearest point nằm trên trimmed face và normal đúng
- [ ] Negative: Điểm gần ngoài trim/face invalid không snap lên phần mặt bị cắt
- Thao tác/kiểm chứng: Fixture mặt có lỗ: Surface Snap phải nhận vùng vật liệu, không snap lên phần bị cắt. Có thể nhận mặt sau qua lỗ nếu hợp lệ; tọa độ và normal kiểm với oracle nguồn.
- Kết quả: **NOT_RUN**. Actual / bằng chứng / reviewer: …

### 14. OM9-L4-03-16-039-03

[L4 gốc](<../../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-039 - Curve · Duplicate Edge - menu binding/L4-03 - Acceptance and Evidence/l4.json>) — Acceptance — Curve · Duplicate Edge — menu binding

- [ ] Positive: selected owner service runs once and returns typed non-noop document delta
- [ ] Negative: no typed selection, invalid numerical option, locked layer, missing backend owner → visible refusal, no document change, no fake success
- Thao tác/kiểm chứng: Mở lại Box; DupEdge qua menu và CMD ở hai lượt độc lập; chọn hai cạnh rồi Enter, kiểm đúng cạnh/layer và một Undo/Redo.
- Kết quả: **NOT_RUN**. Actual / bằng chứng / reviewer: …

### 15. OM9-L4-03-16-040-03

[L4 gốc](<../../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-040 - Curve · Duplicate Border - menu binding/L4-03 - Acceptance and Evidence/l4.json>) — Acceptance — Curve · Duplicate Border — menu binding

- [ ] Positive: selected owner service runs once and returns typed non-noop document delta
- [ ] Negative: no typed selection, invalid numerical option, locked layer, missing backend owner → visible refusal, no document change, no fake success
- Thao tác/kiểm chứng: Vỏ trụ sau lưu/mở: DupBorder qua menu/CMD cho hai biên thật, không có seam nội bộ. Khối kín không biên hở báo rõ. Mesh/hatch chưa hỗ trợ ở đường đã rà.
- Kết quả: **NOT_RUN**. Actual / bằng chứng / reviewer: …

### 16. OM9-L4-03-16-020-03

[L4 gốc](<../../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-020 - Curve · Extract Isocurve - menu binding/L4-03 - Acceptance and Evidence/l4.json>) — Acceptance — Curve · Extract Isocurve — menu binding

- [ ] Positive: exact face/surface created or edited and checked by BRep topology/UV validator
- [ ] Negative: no typed selection, invalid numerical option, locked layer, missing backend owner → visible refusal, no document change, no fake success
- Thao tác/kiểm chứng: Mặt phẳng/trụ và B-rep có lỗ đã chuẩn bị: trích U/V qua menu/CMD, kiểm placement và cắt theo lỗ, Undo/Redo và Esc. Chọn không có tia vẫn có giới hạn từ chối thận trọng.
- Kết quả: **NOT_RUN**. Actual / bằng chứng / reviewer: …

### 17. OM9-L4-03-16-054-03

[L4 gốc](<../../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-054 - Curve · Extract Wire Frame - menu binding/L4-03 - Acceptance and Evidence/l4.json>) — Acceptance — Curve · Extract Wire Frame — menu binding

- [ ] Positive: selected owner service runs once and returns typed non-noop document delta
- [ ] Negative: no typed selection, invalid numerical option, locked layer, missing backend owner → visible refusal, no document change, no fake success
- Thao tác/kiểm chứng: Box cho 12 cạnh; mặt NURBS phẳng không trim cho 4 biên + 2 đường giữa ở mặc định hiện tại; trụ theo fixture nguồn/knot. Một Undo/Redo cho batch. B-rep thiếu interior wires không được coi toàn hợp đồng đã đạt.
- Kết quả: **NOT_RUN**. Actual / bằng chứng / reviewer: …

### 18. OM9-L4-03-18-001-03

[L4 gốc](<../../OM9_Milestones/M03 - Geometry 3DM/L2-03-18 - 10-febe/L3-001 - Solid · Boolean Union - menu binding/L4-03 - Acceptance and Evidence/l4.json>) — Acceptance — Solid · Boolean Union — menu binding

- [ ] Positive: BRep volume=1500 mm³
- [ ] Negative: no typed selection, invalid numerical option, locked layer, missing backend owner → visible refusal, no document change, no fake success
- Thao tác/kiểm chứng: Hai Box giao nhau có kích thước biết trước: chạy Union qua menu/CMD, kiểm hình và DeleteInput theo tùy chọn, Undo/Redo. Cầu-hộp chứng thực kiểm thêm bằng fixture đã chuẩn bị và thể tích giải tích.
- Kết quả: **NOT_RUN**. Actual / bằng chứng / reviewer: …

### 19. OM9-L4-03-18-004-03

[L4 gốc](<../../OM9_Milestones/M03 - Geometry 3DM/L2-03-18 - 10-febe/L3-004 - Solid · Boolean Two Objects - menu binding/L4-03 - Acceptance and Evidence/l4.json>) — Acceptance — Solid · Boolean Two Objects — menu binding

- [ ] Positive: valid exact BRep and explicit positive volume/shape status
- [ ] Negative: no typed selection, invalid numerical option, locked layer, missing backend owner → visible refusal, no document change, no fake success
- Thao tác/kiểm chứng: Hai khối: lặp năm mode preview, Enter nhận kết quả, Esc giữ nguồn; kiểm oracle riêng của từng mode và Undo/Redo. Không suy mọi input đều hỗ trợ.
- Kết quả: **NOT_RUN**. Actual / bằng chứng / reviewer: …

## Chưa đủ điều kiện nghiệm thu

- B-rep interior isocurves và mật độ khác mặc định chưa đầy đủ. Phải kiểm negative: đường hiển thị không được gán thành cạnh topology.
- Source-only history, eviction và Undo/Redo lạnh sau eviction chưa triển khai. Undo thường hoặc xóa Undo để thử lại không chứng minh các chức năng này.
- Hai profile display cùng nguồn/dung sai tính toán và cấu hình xuất là kiểm bổ sung theo hợp đồng nguồn/viewport. Có route chỉ kiểm được qua adapter/controller, chưa có nút UI; ghi UNAVAILABLE nếu chưa có cách thao tác.
- L3 STL Tessellation Export Codec hiện chỉ có l3.json và reference, không có tệp L4 bên dưới trong cây đã đọc. Xuất STL từ nguồn có thể kiểm bổ sung, nhưng không được gán một ID L4 giả.
- Danh sách tập trung core và các lệnh đã làm trong goal; không phải toàn bộ L4 mọi lệnh Curve/Solid. Kiểm lại không tự chứng nhận toàn bộ phần source/display đã hoàn tất.
