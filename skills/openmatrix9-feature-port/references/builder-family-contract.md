# Ring builder family và builder styles (OM9 chọn, người dùng duyệt 2026-10-27)

Nguồn: `OM9-L2-04-03` (06-febe-jewelry-ring-builder-family), `OM9-L2-03-12` (builder styles/presets),
`OM9-L2-03-11` (builder panel). Evidence: `implementation/evidence/M04/OM9-L2-04-03/` và
`implementation/evidence/M04/OM9-L2-03-12/`. Đây là hành vi OM9 đã chọn và kiểm bằng test, chưa phải
Matrix gốc đã chứng minh; gate M04-UI vẫn mở.

## Bốn mode ring builder

| Mode (`BuilderKind`) | Tag style | Command mở | Tham số riêng | Mặc định |
|---|---|---|---|---|
| `EternityBand` | `eternity_band` | `EternityBand`, `Eternity Band`, `Eternity` | `stone_count`, `stone_diameter_mm` | Ø18, rộng 6, dày 2, 12 viên Ø2 |
| `SignetRing` | `signet_ring` | `SignetRing`, `Signet Ring`, `Signet` | `top_width_mm`, `top_length_mm`, `shoulder_mm` | Ø18, rộng 6, dày 2.4, top 8x10, vai 2 |
| `RaisedBand` | `raised_band` | `RaisedBand`, `Raised Band`, `Raised` | `raised_height_mm`, `raised_width_mm` | Ø18, rộng 6, dày 3.5, gờ 3.0x4.0 |
| `AwardRing` | `award_ring` | `AwardRing`, `Award Ring`, `Award` | `top_width_mm`, `top_length_mm`, `shoulder_mm` | Ø18, rộng 6, dày 2.6, top 10x12, vai 2.5 |

Mọi mode đều có `diameter_mm`, `width_mm`, `thickness_mm`. Bốn mode là tab trong panel Builder, mở từ
command line bằng đúng một `BuilderSession`; không có input role nên selection còn lại không phải lỗi.

## Hình học và giới hạn

- Thân nhẫn là **một section kín `(r,z)` quay quanh trục Z** (96 segment, 4 góc/section) → mesh solid
  kín; cả hai backend lấy đúng các section đó: producer portable trả mesh, adapter OCCT
  (`om9_kernel::associative::RingFamilyBrepProducer`) **sew các quad thành một B-Rep solid** nên body
  trong document là B-Rep, preview vẫn là hình học dùng chung. Quad ở section biến thiên không phẳng
  nên mặt B-Rep được chia thành tam giác phẳng; sai lệch cung ở 96 segment ≈ 5 µm, dưới deflection
  0.01 mm.
- Cổng duy nhất cho mọi biểu diễn: `ring_builder::inspect_measures` (kín, thể tích > 0), gọi qua
  `inspect_solid` cho mesh và `inspect_geometry` cho mesh **hoặc** B-Rep. Adapter OCCT tự từ chối
  trước khi tới cổng nếu không sew được đúng **một shell kín** hoặc thể tích ≤ 0, nên không có body
  hở nào vào document.
- `RaisedBand` đẩy gờ ra ngoài, `SignetRing`/`AwardRing` đẩy top pad; `EternityBand` giữ band trơn.
- `MIN_WALL_MM = 0.3`: mọi mode phải còn ít nhất 0.3 mm thành trong sau khi trừ modulation.
- `EternityBand` chỉ xuất **frame đặt đá** `stone_01..stone_NN` (tên, vị trí, pháp tuyến); không tạo gem
  solid — gem là phạm vi M05.
- Cổng cuối `inspect_solid`: kín (mỗi cạnh vô hướng dùng đúng 2 lần) và thể tích dương. Swap OCCT B-Rep
  sau này phải đi qua đúng cổng này, không mở cổng riêng.

## Từ chối có kiểu (không clamp, không đoán)

`Invalid("ring diameter")`, `("ring width")`, `("ring wall")`, `("ring stone count")`,
`("ring stone diameter")`, `("ring stone spacing")`, `("ring raised height")`, `("ring raised width")`,
`("ring signet top")`, `("ring shoulder")`; cổng cuối: `("ring solid empty")`, `("ring solid face")`,
`("ring solid open")`, `("ring solid volume")`.

Quy tắc hình học đằng sau: đá không xếp vừa chu vi → refuse; top dài hơn nửa chu vi (vai gấp) → refuse;
gờ sâu hơn `thickness_mm - MIN_WALL_MM` → refuse; thiếu vai hoặc top âm → refuse. Người dùng sửa lại
tham số thì preview trở lại; lần từ chối không ghi gì vào document và session vẫn mở.

## Viewport handle của ring family

Mỗi mode vẽ grip tại đúng silhouette; kéo grip đi qua `BuilderSession::adjust_field` nên grip và
panel sửa **cùng một field** (không có đường thứ hai). `scale` là số mm field thay đổi trên mỗi mm kéo.

| Grip | Field | Vị trí / trục | scale | Mode |
|---|---|---|---|---|
| `diameter` | `diameter_mm` | mặt trong +X, trục X | 2.0 | cả bốn |
| `width` | `width_mm` | mép +Z, trục Z | 2.0 | cả bốn |
| `wall` | `thickness_mm` | mặt ngoài −X, trục −X | 1.0 | cả bốn |
| `stone` | `stone_diameter_mm` | mặt ngoài +X | 1.0 | Eternity |
| `raised` | `raised_height_mm` | đỉnh gờ +X | 1.0 | Raised |
| `shoulder` | `shoulder_mm` | đỉnh pad +Y | 1.0 | Signet, Award |
| `top_width` | `top_width_mm` | mép pad +Z | 2.0 | Signet, Award |
| `top_length` | `top_length_mm` | mút cung pad (tangent) | 2.0 | Signet, Award |

Grip bị domain chặn (wall không xuống dưới `MIN_WALL_MM`; gờ không sâu quá wall) — kéo quá sẽ clamp
hoặc bị từ chối có kiểu, không tạo giá trị ngoài miền.

## Preset style của builder

- File `om9.builder_style.v1`, typed parameters: `length_mm`, `number`, `choice`, `boolean`,
  `angle_deg` (mới, cho `roll_deg`). Sai version → `Invalid("style schema")`; tên tham số lạ →
  `Invalid("style parameter")`; quá ngân sách → `Budget("style parameters")`.
- Family `band`/`signet`/`gem` là nhãn phân loại; **mode ring builder nhận mọi family** vì cùng là
  builder họ nhẫn — panel bật/tắt theo `BuilderKind`, không theo family.
- Tham số ring addressable qua alias: `diameter_mm`, `width_mm`, `thickness_mm`, `stone_count`,
  `stone_diameter_mm`, `top_width_mm`, `top_length_mm`, `shoulder_mm`, `raised_height_mm`,
  `raised_width_mm` (cộng `position`, `roll_deg` của Rail Profile).
- Builtin hiện có (`Classic 2.2`, `Signet 3.0`, `Brilliant Gem 1.9`) vẫn dùng vocabulary
  width/height của rail-profile: áp `Signet 3.0` lên `SignetRing` chỉ đặt `width_mm = 3`, phần top
  giữ mặc định. Vocabulary preset riêng cho ring (stone count, top pad) **chưa được duyệt** — cần hỏi
  trước khi đổi, vì nó đổi nghĩa file preset người dùng đã lưu.

## History của ring

Resize = mở lại recipe đã ghi (`adopt_history`), sửa `diameter_mm`, commit
`Mutation::Geometry` + `Mutation::HistoryUpdate` trong một transaction. Owner id và recipe id **không
đổi**, không sinh nhẫn thứ hai; Undo trả thể tích cũ, Redo trả thể tích mới. Recipe vẫn là nguồn sự thật
sau commit (không bake).

## Chưa claim

- Mặt B-Rep NURBS trơn: hiện là solid ghép tam giác phẳng từ đúng các section đã sample (sai lệch cung
  ≈ 5 µm); nâng lên surface NURBS/boolean là việc sau, phải đi qua cùng `inspect_geometry`.
- Gem solid: Eternity chỉ có frame, gem thuộc M05.
- Nghiệm thu tay M04-UI: chưa có người duyệt 4 builder trên màn hình, chưa có reviewer signature.
- macOS runtime; parity với Matrix gốc cho từng mode.
