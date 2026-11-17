# C0 — Inventaire chuyển đổi nguồn và representation

Ngày 2026-11-13. Đối chiếu mã hiện tại; đây là hồ sơ làm việc, **chưa đóng C0** và không phải bằng chứng runtime. Giữ nguyên các thay đổi có sẵn trong workspace.

## Ranh giới hiện tại và hướng chuyển

| Owner / caller | Quan sát từ mã | Chuyển đổi cần thiết |
|---|---|---|
| `om9-core/src/brep.rs` | `Brep` giữ `BrepSource`, `Arc<BrepDisplay>`, hash và identity display; `PartialEq` và serde v4 bao gồm display. `CanonicalBrepRecord` kiểm integrity, không chứng thực topology native. | Tách kiểu kết quả operation thành source và representation trước khi publication; document chỉ giữ source. Đổi equality sau khi các caller phân biệt source và display đã được chuyển. Không dùng hash làm chứng thực native. |
| `om9-core/src/geometry.rs` | `NativeBrep` giữ payload opaque, CRC và mesh display; serde/equality gồm cả ba. `Geometry::Mesh` có thể là nguồn. | Giữ nguyên payload và CRC; chỉ bỏ display sau khi adapter có thể dựng lại và kiểm chứng binding. Mesh nguồn vẫn tồn tại. Payload có chứa cache chưa được chứng minh sạch: không tự xóa bytes. |
| `om9-core/src/transaction.rs` | Undo giữ snapshot document; `undo_with`/`redo_with` chuẩn bị trước publication. Source lease riêng đã có cho OCCT và native. | Snapshot source-only; chuẩn bị candidate scene trước khi đổi world/stacks. Lease tiếp tục tính owner thật; source lease hiện có không tự giải phóng display document. |
| `om9-core/src/transaction/cache_pressure.rs` | `OptionalMemoryCache` có recency và gọi reclaim ngoài account lock. | Tái dùng cơ chế khi store có ownership/pin thật; chỉ báo bytes đã nhả. Không đăng ký cache giả hoặc tính bytes vẫn còn reader. |
| `om9-scene/src/scene.rs`, `scene/admission.rs` | Scene tạo buffer từ display document; có display pin/receipt, fallback đọc BRep. API `CapturedRootBrepDisplay` giữ captured document đã được loại; face worker dùng source lease và pin riêng. Native scene đọc mesh trực tiếp. | Nhận representation có lease độc lập; loại fallback phụ thuộc display document. Giữ pin đến reader cuối, đảm bảo stale/source/scene checks và tính overlap buffer. Không giữ document cũ để giữ representation. |
| `om9-app/src/session/topology_remesh.rs` | Remesh worker hiện là opt-in, có reservation, proof, guard và publication; cache hit kiểm scene publication/profile. | Tái dùng orchestration nhưng bổ sung cache miss, nhiều object/instance và lifecycle Open/Undo/Redo. Worker hiện tại không chứng minh restore lạnh. |
| `om9-archive/src/container.rs` | Header `VERSION = 4`, decode schema hiện tại và hydrate document; v4 chứa display BRep. | Thêm parser v4 riêng và writer/reader v5 riêng; admission cả bản cũ, source, cache mới và scratch. Không ghi v5 dưới header v4; không bỏ v4 display trước khi rebind được chứng thực. |
| `om9-kernel/src/brep.rs`, `editing.rs`, `brep/topology_capture.rs` | Snapshot/result và topology mapping dùng display để bind hoặc kiểm chứng; một số helper dùng deflection và segment display. | Chuyển các hàm cần representation sang tham số/candidate có pin; thao tác chính xác tiếp tục dùng source và tolerance riêng. Không thay chứng thực topology bằng so chuỗi/hash đơn lẻ. |
| `om9-core/src/curve_surface_ops.rs` | `duplicate_brep_edge` công khai trả polyline từ display, có ghi rõ xấp xỉ; trong repo hiện chỉ tìm thấy test gọi hàm này. | Không dùng helper này làm exact DupEdge; xác định disposition API trước khi bỏ display. Lệnh app đã có đường source riêng. |
| `om9-app/src/session/wireframe.rs` | Source edge và NativeBrep đã có đường riêng; `capture_source_topology` vẫn lấy deflection từ display. ExtractWireframe BRep chưa có interior face isocurve. | Profile là tham số representation, không lấy ngầm từ document; I1 phải dùng chung tham số cho scene/extraction, phân biệt iso và topology edge, cắt trim và giữ placement. |
| `om9-app/src/session/measure.rs`, `stl.rs` | Có nhánh BRep/native; STL tạo snapshot riêng và tiêu thụ triangles của snapshot xuất. | Giữ measurement/source và dung sai xuất độc lập viewport; không xóa mesh nguồn. Kiểm cả hai profile qua caller sản phẩm. |
| `om9-app/src/session/solid.rs`, `solid_boolean.rs`; `om9-kernel/src/editing.rs` | Fillet lấy witness và profile của display scene hiện hành; handle cạnh kín cũng đọc scene. Profile tessellation kết quả vẫn lấy từ display document. Fixture trụ hai profile đã kiểm trong `docs/validation/2026-11-13-fillet-scene-witness-profile.md`. | Chuyển profile kết quả sang policy representation/candidate độc lập document; tiếp tục chuyển các phép Solid khác dùng `display().deflection_mm`. Không coi fixture trụ là chứng thực mọi topology hay desktop. |

## Quy tắc migration cần giữ

- Key cần binding source và topology được chứng thực, display profile và adapter version; identity process-local/hash đơn lẻ không đủ. Placement và camera chỉ vào key nếu phép dựng thực sự phụ thuộc chúng.
- Cache miss không tạo geometry rỗng. Giữ world/scene cũ trong khi worker chuẩn bị; cancel, stale, budget và lỗi native không đổi document, selection, Undo/Redo hay đích lưu.
- V4 chưa có topology attestation không được gán ID tùy ý. Xác định và kiểm chứng disposition từng input trước khi thu hồi display; nếu không thể thì từ chối migration có lý do, giữ file và document.
- Nhập NativeBrep phải bảo toàn payload opaque. Không coi CRC là topology proof; không nói payload không chứa display khi chưa có bằng chứng adapter.
- Pending input và budget của restore chưa được chốt. Phải đọc luồng sản phẩm và kiểm các nhánh cancel/stale trước khi đóng C0; không suy ra mức budget từ managed counters.

## Fixture và oracle tối thiểu cần chuẩn bị

1. Box, cylinder và mặt có lỗ, có placement: so điểm với source/trim, kiểm iso không thành edge; cùng default density cho scene và ExtractWireframe, menu/CMD, Undo/Redo, save/open. Không tuyên bố density khác hoặc desktop chỉ từ test controller.
2. Cùng source và topology dưới hai display profile: so measurement, extraction và STL theo tolerance source; kiểm pick/snap binding và không phát sinh dirty/Undo khi thay profile.
3. Tạo/sửa hoặc xóa → thu hồi display không pin → Undo lạnh → Redo → Undo; thêm pin/reader, branch, block sharing, worker cũ và cancel/stale/budget. Đo charge tại owner cuối và xác nhận old world/stacks nguyên vẹn khi lỗi.
4. V4 có/thiếu proof, native opaque, mesh nguồn, trim, block và attributes → Save As v5 → mở lại; oracle độc lập cho source/IDs/placement; lỗi version/checksum/ghi/cancel không thay đích hoặc file gốc.
5. M04–M08: chỉ chọn caller đã bật thực sự chịu ảnh hưởng sau inventory; chưa có runtime thì ghi NOT_RUN. M09 giữ gate riêng, không ký từ fixture helper.

## Còn phải làm để đóng C0

- Rà toàn bộ caller sản phẩm của `display()`/`NativeBrep.display`, đặc biệt selection, snap, preview, 3DM và SDK; phân loại candidate, scene read hay source operation. Danh sách trên là nhóm owner, không khẳng định đã bao phủ mọi caller.
- Xác định chiến lược topology attestation/rebind v4 và native payload, key cụ thể, input policy khi restore pending và budget từ workload thực.
- Chọn fixture hiện có và oracle độc lập theo danh sách trên; kiểm khả năng dựng I1 trim/isocurve qua OCCT/openNURBS trước khi cam kết phạm vi. Các test và migration đều chưa chạy trong đợt này.

## Đối chiếu tích hợp 2026-11-14

- `cargo check --workspace -j 2` với `CARGO_TARGET_DIR=L:\om9-target` hoàn tất; đây chỉ là bằng chứng biên dịch của trạng thái hiện tại, không phải runtime hay nghiệm thu.
- `Geometry::Brep` đã giữ `CanonicalBrep`, còn `NativeBrep` vẫn giữ mesh display. Store OCCT nhận candidate qua `register_brep_candidate`; `evict_unpinned` chưa được nối vào đường thu hồi theo áp lực bộ nhớ.
- History worker gọi `prepare_history_scene`, sau đó scene tìm representation trong store. Chưa có bước dựng representation thiếu trước scene capture, nên source-only cold restore chưa được hoàn thành. Open cũng phải được nối cùng đường chuẩn bị, với guard và reservation của nó.
- `container.rs` vẫn ghi header v4 mặc dù serde của OCCT B-rep đã chuyển sang canonical record; NativeBrep vẫn ghi display. Decoder dùng cùng kiểu `Document` cho v4, trong khi B-rep v4 cũ có trường `display` và `display_sha256`. Do đó không coi writer hiện tại là codec v4 tương thích hoặc v5 canonical-only; cần parser v4 riêng, disposition/topology rebind, writer v5 và dựng scene trước publication trong cùng migration.
- Không chạy test, benchmark hay kiểm thử lệnh trong đợt này theo chỉ đạo hiện hành. C1–C5 và I1 vẫn mở; không suy ra gate M09.
