# Layer Management - kiểm tra spec và code, 2026-10-11

> Đây là audit trước triển khai. Hiện trạng mới: [Layer panel implementation](2026-10-11-layer-panel-implementation.md). Các sai khác và source fingerprints dưới đây giữ làm lịch sử.

Đã có spec **OM9-LAYER-001**. Source có một phần triển khai thực tế nên sửa
trạng thái `not_started` thành `partially_implemented`. Lần này cập nhật tài
liệu và kiểm tra code; không triển khai thêm hành vi sản phẩm.

## Nguồn đã đọc

- Người dùng cung cấp bản trích `matrix8_book_1 Copy.pdf`, 4 trang PDF tương ứng
  trang in 44-47, Chapter 9 / The Layers Menu. Đã extract text và render/kiểm
  toàn bộ bốn trang; fingerprint ghi trong [report JSON](2026-10-11-layer-management-audit.json).
- Ảnh panel LAYERS trong yêu cầu: 16 layer cơ bản, Advanced 16 layer, Lights,
  Hide/Show và footer combo cùng `C A L V C`. PDF mô tả nhóm controls chính nhưng
  không giải thích footer này. Không chép ảnh/manual riêng vào repository.
- [Spec đã cập nhật](../../OpenMatrix9_Codex_Spec_v1/specs/01-core/om9-layer-001-layer-management.md)
  giữ material-analysis trước đây riêng khỏi các hành vi panel vừa đối chiếu.

## Ma trận hiện trạng

| Phần trong ảnh/manual | Hiện trạng source | Bằng chứng |
|---|---|---|
| 32 tên, màu và thứ tự hàng | Có; bảng tên cố định và hai block/two-column grid. Màu tuyệt đối là OM9 defaults, chưa chứng nhận palette nguồn. | `rust/src/layers.rs:7`, `Gui/MatrixSidebar.cpp:198` |
| Current layer và highlight | Có per-document `OM9ActiveLayer`, typed index/tên/None; chỉ Circle/Ellipse nhận layer output. Index 0 là legacy root. | `Gui/CoreLayers.cpp:59`, `:143`, `:154`; `Gui/MatrixSidebar.cpp:340`; `Gui/CurveController.cpp:119`; `Gui/CurveEllipse.cpp:145` |
| In arrow Assign selection | **Sai khác:** code chọn current, chưa chuyển object đang chọn. Manual trang in 45 phân biệt rõ Assign với current. | `Gui/MatrixSidebar.cpp:205`; `Gui/CoreLayers.cpp:143` |
| Right-click swatch chọn theo layer | Chưa có; swatch chỉ nối clicked với dialog màu. | `Gui/MatrixSidebar.cpp:207` |
| Lock bảo vệ object cũ | Một phần: cờ lock và output rejection có, selection/Region/edit chung chưa đọc `OM9LayerLocked`. Không đổi Selectable của member. | `Gui/CoreLayers.cpp:165`; `rust/src/layers.rs:106`; global source search cho flag; `Gui/CoreMouse.cpp:115` |
| Visibility từng layer | Có, nhưng toggle ghi Visibility cho mọi member; không giữ object-hidden độc lập layer-off. | `Gui/CoreLayers.cpp:169-175` |
| Group visibility | Chưa có control dài/handler riêng cho nhóm. | `Gui/MatrixSidebar.cpp:198-221` |
| Hide/Show trái và phải | Có nút text nhưng disabled; không handler trái/phải. | `Gui/MatrixSidebar.cpp:160`, `:197` |
| Advanced mở/thu | Chưa có; separator là label, cả hai block luôn được dựng. | `Gui/MatrixSidebar.cpp:161`, `:198` |
| Advanced rename/color gesture | Rename chưa có; UI name luôn lấy Rust. Đổi màu có nhưng left-click cho cả 32 layer, khác Shift+right-click chỉ Advanced trong manual. | `Gui/CoreLayers.cpp:89`; `Gui/MatrixSidebar.cpp:207-214`, `:340` |
| Lights và Matrix Art restriction | Lights chỉ layout/disabled icon; không layer slot Lights và parser từ chối tên này. Chưa thấy policy restriction tương ứng được tích hợp. | `Gui/MatrixSidebar.cpp:195-197`; `rust/tests/layers.rs:51`; `rust/src/layers.rs:9` |
| Combo/footer `C A L V C` | Chưa có trong layer section. Ý nghĩa/defaults cần thêm evidence; bốn trang PDF không giải thích. | Ảnh người dùng; `Gui/MatrixSidebar.cpp:221` chuyển sang Projects |
| Undo/FCStd của slice | Adapter có transaction và dynamic metadata; report Circle layer lịch sử 52/52 còn tồn tại. Chưa có nghiệm thu cả panel theo exact Layer ID. | `Gui/CoreLayers.cpp:128`; `tests/circle_layers_smoke.FCMacro`; [record trước đây](2026-10-09-circle-complete-branches.md) |

## Tests và giới hạn chứng minh

- Chạy mới `cargo test --manifest-path rust/Cargo.toml --locked --offline --test layers`:
  **6/6 đạt**, exit 0. Kiểm slot/name/color, parser, invalid/locked/missing output,
  hidden/root output, snapshot isolation và ABI inputs. Đây là Rust policy tests,
  không phải nghiệm thu thao tác Qt/native.
- Đọc lại report `build/circle_layers_smoke-1/04d2073d994748429eafa5eae05e0917/results.json`:
  `ok=true`, **52/52 checks**. Report được tạo trước lần audit này và macro ghi
  feature `OM9-CURVE-005`; không coi là suite toàn bộ `OM9-LAYER-001` trên binary
  hiện tại. Có cả lớp/layer, current, Undo/Redo, FCStd, imported metadata isolation.
- `Gui/tests/MatrixSidebarTest.cpp` có layout-count test 32 labels/locks, không
  chứng minh Assign, lock selection hoặc Hide/Show. Không chạy lại Qt/native suites
  và không build host/plugin vì không đổi code sản phẩm.
- Kiểm tra tài liệu sau sửa: **17/17 đạt**, gồm trạng thái spec/JSON/YAML,
  manifest hash/size của 5 file package, liên kết local và bảo toàn các feature/
  ledger items không liên quan. `tools/public_source_audit.py` đạt, exit 0:
  2.434 source files, 511 icon bindings, 510 SVG files, không lỗi.
- Layer capture hiện ở commit, không ở đầu phiên dựng. General hierarchy/
  imported-layer inheritance, session-time layer generation và mọi output family
  còn mở. Cờ lock không thay thế permission service.

## Tài liệu đã đồng bộ

Spec, `FEATURES.json`/`FEATURES.yaml`, README nhóm 01-core,
`IMPLEMENTATION_STATUS.md`, README dự án, feature record và live ledger ghi cùng
phạm vi. Manifest cập nhật hash/size các file package thay đổi. Rust example và
các acceptance runtime chưa có evidence vẫn giữ chưa hoàn tất.

Các fixture cần tiếp theo nằm trong mục **Ca nghiệm thu riêng cho panel còn mở**
của spec. Ưu tiên Assign selection và khóa selection/edit; rồi visibility hai tầng,
Hide/Show và Advanced. Footer phải xác minh riêng trước khi chọn hành vi.
