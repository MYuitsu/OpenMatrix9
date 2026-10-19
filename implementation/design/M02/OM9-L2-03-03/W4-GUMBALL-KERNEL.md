# W4 — port kernel gumball & occurrence array (thuộc M02 `OM9-L2-03-03`)

**Vị trí milestone:** M02 – Viewport, Gumball, L2 `OM9-L2-03-03` (Febe viewport object selection and
gumball), dùng chung với `OM9-L2-01-07 L3-007` Generic Translation Rotation Scale Handles. Đây là **port
kernel có test**, không phải nghiệm thu L3.

| Hàm kernel | L3 đích trong M02 | Ghi chú |
| --- | --- | --- |
| `Handle`, `Axis`, `Plane`, `code`/`from_code` | `OM9-L3-01-07-007` Generic Translation Rotation Scale Handles | mã 0..12 giữ nguyên như wire legacy |
| `Frame::{world,new,validate,axis,plane_basis}` | `OM9-L3-03-03-005` Matrix Gumball Alignment And Relocate | frame trực chuẩn tại pivot; từ chối frame lệch |
| `transform(frame, handle, Drag::WorldDelta)` | `OM9-L3-03-03-004` Matrix Gumball Axis Drag | handle trục chỉ giữ thành phần theo trục; handle mặt phẳng bỏ thành phần pháp tuyến; free giữ cả delta |
| `transform(..., Drag::Angle/Factor)`, `rotation_about`, `scaling_along` | `OM9-L3-03-03-006` Matrix Gumball Numeric Edit | quay/scale quanh tâm frame; nhập số đi qua cùng đường |
| `rectangular_array`, `polar_array` | `OM9-L3-03-03-*` (array/occurrence) | mảng chữ nhật theo frame, mảng cực quanh trục `z`; trần `MAX_ARRAY_OCCURRENCES` |
| `Session` (`begin`/`drag`/`reset`/`move_mutations`/`copy_mutations`) | `OM9-L3-03-03-004` Axis Drag + `OM9-L3-03-03-006` Numeric Edit (một gesture = một Undo) | gộp nhiều drag thành **một** bước; Move ghi `Entity.placement`, Copy chèn bản sao với id mới |

Trạng thái L3: **không** L3 nào được đánh dấu nghiệm thu — đóng L3 M02 cần runbook/spec của M02.

## Đã port (đợt 1)

| Legacy | Hành vi giữ lại | Target |
| --- | --- | --- |
| `rust/src/gumball.rs` (`Axis`/`Plane`/`Handle`/`Frame`) | mô hình handle + frame trực chuẩn + mã wire | `gumball::{Axis, Plane, Handle, Frame}` |
| `rust/src/gumball.rs` (drag) | ràng buộc drag theo trục/mặt phẳng/free; quay & scale quanh tâm frame | `gumball::{transform, rotation_about, scaling_along}` |
| `rust/src/array_occurrences.rs` | mảng chữ nhật trên frame, mảng cực quanh trục, trừ bản gốc | `gumball::{rectangular_array, polar_array}` |
| `rust/src/gumball_owner.rs`, `gumball_associative.rs` (phần trạng thái) | phiên kéo: gộp nhiều drag thành một bước, reset khi Esc, sinh mutation `Placement` (Move) hoặc `Insert` bản sao (Copy) | `gumball::Session` |

Ghi chú thiết kế: góc quay và hệ số scale **không** suy ra từ delta thế giới — chúng đến từ chiếu màn hình
ở tầng scene (camera), nên kernel nhận `Drag::Angle`/`Drag::Factor` tường minh. Nhờ vậy kernel không phụ
thuộc camera và vẫn test được bằng oracle hình học.

`rotation_about` và `scaling_along` dựng ma trận affine 4×4 (row-major, cùng quy ước `Transform` của
`om9-core`), với translation = `origin - M·origin` để điểm tâm đứng yên. Scale chỉ nhận factor `> 0`
(scale âm là lật gương — không thuộc gumball).

## Test (15)

`crates/om9-core/tests/gumball.rs`:

- mã handle 0..12 round-trip đúng wire legacy, `from_code(13) = None`, normal của 3 mặt phẳng đúng;
- handle trục chỉ giữ thành phần theo trục (X/Z), free giữ cả delta;
- handle mặt phẳng bỏ thành phần pháp tuyến (XY/YZ/ZX);
- frame đã quay 90°: drag theo trục `x` của frame đi theo world +Y;
- drag sai loại input bị từ chối (`Invalid("gumball drag input")`), NaN bị từ chối;
- `rotation_about`: điểm tâm đứng yên, (3,0,0) quay 90° quanh (2,0,0) → (2,1,0), quay 360° = identity;
- `scaling_along`: tâm đứng yên, (3,0,0) scale 2 quanh (1,0,0) → (5,0,0), y/z không đổi;
- rotate/scale qua `transform` cho cùng kết quả; tham số xấu bị từ chối (trục 0, góc NaN, factor ≤ 0,
  frame không trực chuẩn);
- `rectangular_array` 3×2 spacing (10,4): 5 occurrence đúng vị trí, `include_source` thêm identity;
- `polar_array` 4 vòng tròn: 3 occurrence ở 90° bước; mảng hở kết thúc đúng góc yêu cầu;
'- từ chối: count 0, 1×1, vượt trần, spacing NaN, polar count < 2, góc 0;
- **Session**: hai drag liên tiếp compose đúng thứ tự thế giới (X 5 rồi Y 3 → dịch (5,3,0)); `reset` xoá
  tích luỹ nhưng giữ target; từ chối target rỗng/trùng/nil;
- **Move qua transaction**: 2 entity, một `Mutation::Placement` mỗi cái, commit → **một** Undo entry,
  `geometry` **không đổi**, điểm thế giới dịch đúng; undo trả placement về cũ;
- **Copy qua transaction**: 2 `Mutation::Insert`, entity mới có id mới, giữ geometry nguồn, placement
  nâng 4 mm; đếm entity +2 và **một** Undo entry; session Move không sinh được plan Copy và ngược lại.'

## Còn lại của W4

- **Hit-test handle + kéo chuột** (`gumball_view.rs`): cần camera/scene để chiếu con trỏ thành
  `Drag::Angle`/`Drag::Factor`/`Drag::WorldDelta`. Kernel đã nhận đúng ba loại input đó nên phần này chỉ
  là lớp mỏng trên `om9-scene`.
- **Associativity** (`gumball_associative.rs`): gắn gesture với History/recipe khi hình học là con của
  builder — thuộc M04, mở cùng HistoryGraph.
- **Instance graph** (`instance_graph.rs`): target đã có `Definition` + `Document::instance` cho
  occurrence; phần còn lại là bookkeeping chủ sở hữu, nối khi có lệnh Array/Import.
- **Command layer M02** (Move/Copy/Rotate/Scale + Array): descriptor + option schema + session, gọi
  `gumball::Session` rồi commit một transaction. Không cần thêm mutation nào — `Placement` và `Insert`
  đã có sẵn và đã được test ở đây.
- `*_ffi`: `LEGACY_HOST`, không port.

## Native sau (không chặn code)

Kernel này thuần Rust và chạy được ngay. Khi nối OCCT/openNURBS sau, lớp native chỉ cần nhận `Transform`
đã dựng và áp lên hình học nếu hình học đó là native (B-Rep); mọi record đi qua đều đã `validate`, lỗi
đi qua `Error` có nhãn.
