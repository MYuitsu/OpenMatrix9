# P1 — Kiểm kê nguồn, mesh và lệnh

**Ngày:** 2026-11-06. Đây là kiểm kê trên worktree hiện tại, kể cả thay đổi chưa commit; không phải kết quả nghiệm thu đầy đủ hoặc tổng số lệnh OM9. Người dùng ưu tiên core/Curve/Solid trước; Settings, Gem, plugin và lệnh khác giữ trong danh sách để xử lý khi test tới. Các test được nhắc đến có phạm vi fixture riêng.

## A0 — Hai lệnh cần phân biệt (ảnh chụp lịch sử 2026-11-10)

Bảng A0 bên dưới giữ nguyên phạm vi tại thời điểm chốt ban đầu, không phải tình trạng hiện nay. Các mở rộng đã kiểm và giới hạn còn lại được cập nhật tại [plan hiện hành](PLAN_CORE_CURVE_SOLID.md) và [hợp đồng ExtractWireframe](../../../../docs/features/OM9-CURVE-054.md).

| ID / menu / alias | Chọn và input | Output, placement, layer | Lỗi và transaction |
|---|---|---|---|
| OM9-CURVE-040, Duplicate Edge, `DupEdge` | Cạnh chọn trước hoặc chọn sau khi gọi lệnh, của một OCCT B-rep có proof, placement rigid | Mỗi cạnh một NURBS nguồn có trim và hướng; áp placement, layer hiện hành; cạnh kín được kiểm, seam chưa nghiệm thu riêng | Thiếu proof hoặc input ngoài phạm vi bị từ chối; tất cả output trong một transaction. Chọn nhiều cạnh viewport được mô phỏng; desktop chưa nghiệm thu |
| OM9-CURVE-054, Extract Wireframe, `ExtractWireframe` / `ExtractWire` | Một BRep OCCT được chọn, có proof và placement rigid | Mỗi cạnh topology một NURBS nguồn; áp placement và layer hiện hành. Kiểm seam của một shell; isocurve và seam tổng quát chưa nghiệm thu | Commit một transaction; cạnh suy biến hoặc input ngoài phạm vi bị từ chối. [Evidence](../../../../docs/validation/2026-11-10-extractwire-source-first-slice.md) |

## Quy tắc đếm

- Một mục là một feature ID/lệnh chính trong phạm vi dưới đây; alias, lựa chọn và tên gọi khác của cùng feature không tính thêm. Không suy số lệnh từ số hàm, file hoặc số test.
- Phạm vi đã liệt kê: **22 feature Solid + 8 Settings + 7 mục kiểm trực tiếp = 37 mục**. Đây là danh sách bắt đầu, không phải danh sách đầy đủ mọi lệnh liên quan trong OM9. Registry Solid còn 5 identifier bổ sung nêu bên dưới, chưa nhập vào tổng 37 vì cần đối chiếu command độc lập với mode/route. Gem, Top11, addon/plugin và lệnh khác chưa được thống kê theo command.
- **Có đường nguồn** không đồng nghĩa lệnh đã hoàn tất P1 hoặc mọi input/option chính xác. **Cần quyết định** không mặc định là lỗi: mesh do người dùng chủ động tạo/sửa phải được giữ là mesh. **Chưa kiểm** không đồng nghĩa bị lỗi.
- Số file dưới đây là file **ứng viên để rà**, không phải số file chắc chắn phải sửa. Một file có nhiều lệnh; một lệnh đi qua nhiều file. Số mục chưa kiểm của toàn ứng dụng và phần trăm hoàn thành P1: **chưa xác định**.

## 22 Solid: registry đã xác định, chưa nghiệm thu từng lệnh cho P1

Nguồn danh sách: `FEATURES` trong [solid.rs](../../../../crates/om9-app/src/session/solid.rs). Box/Sphere có đường tạo OCCT B-rep; Boolean Union/Difference/Intersection đã có phép so sánh một fixture ở hai deflection, không chứng nhận mọi topology. Các mục khác là **ứng viên rà**, không đánh dấu xong chỉ vì nằm trong registry.

| ID | Lệnh | Phạm vi bằng chứng P1 |
|---|---|---|
| OM9-SOLID-001 | Boolean Difference | Fixture hộp/cầu, hai deflection; còn kiểm điều kiện khác |
| OM9-SOLID-002 | Boolean Intersection | Fixture hộp/cầu, hai deflection; còn kiểm điều kiện khác |
| OM9-SOLID-003 | Boolean Union | Fixture hộp/cầu, hai deflection; còn kiểm điều kiện khác |
| OM9-SOLID-004 | Boolean Two Objects | Đã sửa lỗi tịnh tiến trong fixture cầu/hộp; năm preview và Undo/Redo đạt. Scale không đều tái hiện lỗi và placement không cứng nay bị từ chối tạm. [Evidence và giới hạn](../../../../docs/validation/2026-11-12-boolean-two-placed-source.md); affine đã bake vào source, topology và desktop còn mở |
| OM9-SOLID-005 | Cap Planar | Một shell trụ lưu/mở với hai profile display qua menu dispatcher/CMD cho thể tích nguồn và Undo/Redo đúng; [evidence](../../../../docs/validation/2026-11-12-cap-planar-source-profiles.md). Input tổng quát, desktop và native peak còn mở |
| OM9-SOLID-006 | Extract Surface | Đã kiểm fixture mặt phẳng/trụ và v4 hoán vị, một/nhiều mặt, Copy và Undo/Redo; [evidence](../../../../docs/validation/2026-11-11-extractsurface-reopened-face.md). Trim/import và native peak còn mở |
| OM9-SOLID-007 | Fillet Edge | Đã kiểm v4 hoán vị một/hai cạnh và contour trong phạm vi fixture; [evidence](../../../../docs/validation/2026-11-11-fillet-reopened-edge-regression.md). Cạnh trim/import và native peak còn mở |
| OM9-SOLID-008 | Text | Chưa kiểm representation và các mode cho P1 |
| OM9-SOLID-009 | Extrude Curve Straight | Đã sửa placement curve sau lưu/mở; fixture chữ nhật và đường tròn phản chiếu/CPlane YZ đạt bounds, Undo/Redo; [evidence](../../../../docs/validation/2026-11-12-extrude-curve-placement.md). Curve/cplane tổng quát và desktop còn mở |
| OM9-SOLID-010 | Solid Extrude All | Hai curve tịnh tiến được đùn đúng vị trí trong một Undo/Redo; lỗi input cuối giữ Redo trong fixture hiện có. [Evidence](../../../../docs/validation/2026-11-12-extrude-curve-placement.md) |
| OM9-SOLID-011 | Pipe | PipeChain chọn cạnh B-rep v4 hoán vị đã kiểm cạnh thẳng, volume và Undo/Redo; [evidence](../../../../docs/validation/2026-11-11-pipechain-reopened-edge.md). Các mode Pipe khác không được chứng nhận bởi fixture này |
| OM9-SOLID-012 | Box | Có đường tạo B-rep; cache/Undo/v5 chưa đạt |
| OM9-SOLID-013 | Ellipsoid | Chưa kiểm riêng cho P1 |
| OM9-SOLID-014 | Sphere | FitPoints lấy đỉnh source; AroundCurve v4 hoán vị đã kiểm cạnh thẳng/tròn, từ chối sai và Undo/Redo: [evidence](../../../../docs/validation/2026-11-11-sphere-aroundcurve-reopened-regression.md). Các mode còn lại, cache/Undo/v5 không suy từ fixture này |
| OM9-SOLID-015 | Torus | Chưa kiểm riêng cho P1 |
| OM9-SOLID-016 | Cylinder | Chưa kiểm riêng cho P1 |
| OM9-SOLID-017 | Tube | Chưa kiểm riêng cho P1 |
| OM9-SOLID-018 | Pyramid | Chưa kiểm riêng cho P1 |
| OM9-SOLID-019 | Cone | Chưa kiểm riêng cho P1 |
| OM9-SOLID-020 | Truncated Cone | Chưa kiểm riêng cho P1 |
| OM9-SOLID-021 | Boss | Chưa kiểm riêng cho P1 |
| OM9-SOLID-029 | Shell | Đã kiểm v4 hoán vị một/hai mặt, đường số và chọn mặt, từ chối sai và Undo/Redo; [evidence](../../../../docs/validation/2026-11-11-shell-reopened-face.md). Trim/import, desktop và native peak còn mở |

Ngoài 22 mã trên, `handles()` còn ghi `SolidBooleanSplit`, `SolidCreateSolid`, `SolidExtractSurfaceCopy`, `SolidPipeChain`, `SolidGVDMultiPipe`. Chúng cần phân loại với aliases/mode trước khi đếm là lệnh độc lập.

## 8 Settings: dữ liệu commit hiện tại

Nguồn: [settings_commands.rs](../../../../crates/om9-app/src/session/settings_commands.rs), đặc biệt `SettingsTool::ALL` và `commit_settings_pending`. Đây là representation quan sát trong mã, chưa đánh giá đầy đủ mọi option hoặc độ chính xác.

| Mục | Commit hiện quan sát | Cần làm rõ |
|---|---|---|
| BuilderProngAdder | `Geometry::Mesh` | Có cần nguồn CAD tham số/B-rep cho output hay không |
| BuilderProngEditor | Thay geometry bằng `Mesh` | Giữ compatibility và lịch sử khi thay nguồn |
| BuilderProngOnSurface | `Mesh` | Phân biệt mặt nguồn và thân prong |
| BuilderMetalPiece | `Mesh` | Kiểm cutter/solid, không tự suy hình học gốc |
| BuilderBeadOnCrv | Các bead là `Mesh` | Hạt cầu mesh: xem [bead.rs](../../../../crates/om9-core/src/settings/bead.rs:241); cần quyết định CAD source |
| BuilderBeadOnSurface | Bead là `Mesh` | Cần quyết định CAD source và placement |
| BuilderChannelBorderCurve | `Curve` | Không ép đường cong gốc thành B-rep; rà các input |
| BuilderPullToRail | Rigid sửa placement; Deform sửa `Mesh` | Rà theo từng mode, không giả định là cùng representation |

`SettingsMass` là readout riêng và chưa tính vào 8 icon. Gem có đường chọn B-rep/mesh tại [gem.rs](../../../../crates/om9-app/src/session/gem.rs:757), chưa đếm số lệnh Gem hoặc khẳng định B-rep có mặt cong gốc: adapter ở đây tạo từ biên mesh của gem.

## 7 mục kiểm trực tiếp

| Mục | Đã thấy trong code/evidence | Còn thiếu |
|---|---|---|
| DupBorder | B-rep lấy biên OCCT, một transaction; v4 hoán vị, trụ, vỏ bốn mặt và từ chối Box kín đã kiểm: [fixture](../../../../docs/validation/2026-11-11-dupborder-source-boundary.md) | Topology/placement phức tạp, desktop và native peak |
| BoundingBox | B-rep/NativeBrep theo source; [fixture](../../../../docs/validation/2026-11-06-brep-source-world-bounds.md) | Các geometry/consumer khác, model lớn và desktop |
| CenterObject | Dùng world bounds và placement; cùng evidence | Các representation khác và input biên |
| Export STL | B-rep tessellate từ source; desktop capture/edit/cancel đã kiểm | Native memory, global error/coverage và các oracle P3a |
| MeshFromNurbs | Hàm `mesh_from_nurbs_selected` tạo `Geometry::Mesh` từ surface; tìm kiếm hiện tại chỉ thấy hàm và test gọi trực tiếp, chưa thấy kết nối CMD/menu | Trước khi coi là chức năng sản phẩm, xác định đường gọi và hợp đồng; hàm hiện lấy mặt nền, chưa dùng trim hay placement nguồn. Mesh do người dùng tạo là dữ liệu riêng |
| ReduceMesh | Sửa mesh được chọn; [session.rs](../../../../crates/om9-app/src/session.rs:4047) | Không ép sang B-rep; kiểm deviation và topology theo P3d |
| Mesh / Curve on Mesh | Nhận mesh + điểm và tạo curve; [mesh_curve.rs](../../../../crates/om9-app/src/session/mesh_curve.rs) | Kiểm placement, sai số và tiêu chí người dùng |

Surface snap/picking, Undo/Redo, New/Open/Save, GPU và 3DM là **đường dùng chung**, không cộng vào 37 mục lệnh trên. Thực trạng nền: [Brep](../../../../crates/om9-core/src/brep.rs:19) vẫn sở hữu display; [Undo](../../../../crates/om9-core/src/transaction.rs:700) giữ snapshot document; [codec](../../../../crates/om9-archive/src/container.rs:15) vẫn v4; remesh source có worker thử nghiệm nhưng chưa được command/UI thông thường yêu cầu. Các điểm này chặn đóng P1 dù một số lệnh đã được sửa.

## Số file và điểm rà theo tìm kiếm

Tìm literal `Geometry::Brep`, `Geometry::NativeBrep` hoặc `Geometry::Mesh` trong `crates/om9-app/src/session/**/*.rs`, loại file có `test`/`regressions` trong đường dẫn, ra **33 file** dưới đây. Kết quả bao gồm preview, import/export, helper và có thể chứa test inline; bỏ sót alias/type suy luận và không bao gồm `session.rs` ở thư mục cha, UI, core, kernel, scene, archive. Chỉ là ranh giới tìm kiếm để chia việc, không phải 33 file cần sửa:

`addon_command.rs`, `addon_session.rs`, `channel_cutters.rs`, `clipboard.rs`, `dup_border.rs`, `files.rs`, `gem.rs`, `gem_cutters.rs`, `gem_edits.rs`, `head_commands.rs`, `measure.rs`, `measure/extended.rs`, `mesh_curve.rs`, `metal.rs`, `point_input.rs`, `ring_tools.rs`, `settings_commands.rs`, `solid.rs`, `solid_boolean.rs`, `solid_cone.rs`, `solid_geometry.rs`, `solid_pipe.rs`, `solid_text_stages.rs`, `stl.rs`, `stress.rs`, `subobjects.rs`, `surface_commands.rs`, `surface_snap.rs`, `text_input_surface.rs`, `top11.rs`, `topology_remesh.rs`, `transform/cage.rs`, `wireframe.rs`.

Tìm literal `display.deflection_mm` riêng trong 3 owner ứng dụng phát hiện **42 dòng**: `solid.rs` **15**, `top11.rs` **26**, `ring_tools.rs` **1**. Đây là chỗ cần xác định tham số chỉ điều khiển mesh hay còn ảnh hưởng input/output và so sánh hình học; **42 không phải 42 lỗi hoặc 42 lệnh**. Các lệnh có thể truyền cùng một deflection nhiều lần. Tìm literal có giới hạn, không chứng minh các biến đổi gián tiếp đều đã được phát hiện.

**Phân loại bước đầu, 2026-11-11:** Boolean Union/Difference/Intersection trong `editing_bridge.cpp::om9_occt_boolean` đọc hai source, thực hiện phép OCCT rồi mới truyền deflection vào `emit` để tạo snapshot output; bằng chứng hai profile hiện có chỉ bao phủ fixture đã ghi, không chứng nhận mọi input hoặc mọi caller. Shell và Extract Surface nay tách profile witness với profile output. Với scene thô, nắp trụ `Copy=No` số/chọn, chọn hai mặt `Copy=Yes/No` và Shell nắp đã qua; Shell bỏ mặt bên bị OCCT từ chối cả trên source gốc và giữ document/Redo. Witness chọn theo diện tích là heuristic: [evidence](../../../../docs/validation/2026-11-11-extractsurface-reopened-face.md). Các đường còn lại chưa được phân loại bởi kết luận này.

## Theo dõi không loop

1. Theo phạm vi A/B đã chốt trong plan chính: A0 bảng hợp đồng hai lệnh → A1 DupEdge tại `dispatch_wireframe` → A4 nghiệm thu → A2 ExtractWireframe riêng. Không sửa `duplicate_brep_edge`/`edge_endpoints` chỉ vì test đang gọi; giữ mapping, placement và một transaction cho cả tập. B0 đo rồi quyết định; B1–B3 chỉ source-only lịch sử Sphere, current document còn display, worker dựng đầy đủ trước Undo/Redo publication. Giữ v4 fallback, proof/worker thử nghiệm không đóng gate B.
2. Với danh sách 37, điền từng mục theo các cột: input/source, output, phụ thuộc display, bằng chứng, trạng thái. Mỗi mục chỉ đóng khi kiểm input/mode thực sự trong phạm vi cam kết; mở rộng inventory theo registry còn thiếu, không suy số tổng từ 37.
3. Rà 42 dòng theo nhóm lệnh, ghi nguyên nhân và test cần thiết; không tự đổi giá trị deflection, không coi số dòng là khối lượng công việc.
4. A1/A2 có batch riêng: oracle cạnh cong/trim/placement, hai profile, menu/CMD và lỗi cạnh cuối không ghi một phần. B dùng đúng workload B0, đạt gate lợi ích/latency B3 và Save/Save As v4 lưu/mở lại sau Undo/Redo lạnh; không được hoãn khả năng lưu hiện tại cùng codec v5. A3, current-display eviction, v5 và quality controls còn mở trong P1 tổng, không tự kéo vào ba bàn giao này. Chỉ chạy lại phần bị tác động/thất bại, không đổi workload để tìm số đo có lợi.
