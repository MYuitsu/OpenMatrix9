# Settings parts: prong, metal piece, bead, mass (OM9 chọn, người dùng duyệt 2026-10-27)

Nguồn: `OM9-L2-04-15` (02-febe-prong-channel-metal-and-bead). Evidence:
`implementation/evidence/M06/OM9-L2-04-15-COMPLETION/README.md` và
`implementation/evidence/M06/OM9-L3-04-15-*/`. Đây là hành vi OM9 đã chọn, có test đo được, chưa
phải Matrix gốc đã chứng minh; gate M06-UI vẫn mở nên dashboard giữ 10 L3 ở WIP.

## Input/output dùng chung (luật L2)

- Mọi lệnh settings đọc **một** `settings::gems::GemRow` dựng từ record `om9.gem` của owner gems và
  trả body đã commit + `om9.settings.prong` payload; không có kernel/store thứ hai.
- Chọn gems kèm **curve** là hợp lệ: curve là input của `Curve` mode, gems vẫn là row. Object không
  phải gem/curve trong selection bị từ chối `invalid settings need gem objects`; không có gem nào
  cũng bị từ chối.

## Prong

- Góc 0° = **ngang hàng** (phía North của viên, tức trục `side` của row), 90° = **dọc hàng**; 180°
  là phía đối diện. Prong "side" là 0°, prong "end" là 90°/270°.
- Hai family đúng sheet: **Gem Flow** (mặc định, `2 Shared Prongs`) đặt prong theo đầu/cuối hàng và
  junction chia sẻ; **Gem North** đặt quanh từng viên theo góc, 22 preset tên kiểu `4 Prongs 0
  Degrees`.
- `4 Prongs Close`: hai prong mỗi phía, **sát nhau hơn `4 Prongs 0 Degrees`** và vẫn là **hai prong
  riêng**. Test đo đúng câu chữ của sheet (khoảng cách cặp Close < cặp `4 Prongs 0`, và > 0.05 mm);
  không có hằng số độ tự đặt.
- `6 Prongs 0 Degrees` = bốn prong ở vị trí Close + một prong ở +90° và một ở −90° (tức `2 Prongs
  90°`). Trên hàng sát, cặp prong đối diện vẫn dựng được.
- **Hợp lưu prong chia sẻ**: khi hai *anchor khác nhau* đặt prong vào cùng một điểm (cách nhau dưới
  nửa đường kính prong), builder gộp thành **một** prong tại trung điểm, lấy thân lớn hơn,
  `angle_deg = None`, và đếm vào `ProngSet.merged_prongs` (đọc được trong `summary()`). Cặp prong
  **cùng một viên** không bao giờ bị gộp: đó là cụm Close do mode định nghĩa.
- End modes `Ends=Two/One/None/FollowMode` và `Spacing=` chỉ đổi cách đặt prong, không đổi mode.
- Editor sửa **một** prong từ payload và từ chối `invalid prong overlap` nếu người dùng kéo nó lên
  prong khác; layout tự động thì hợp lưu như trên.

## Metal Piece

- 6 mode open + 3 mode closed như sheet; `Outside`/`Top`/`Round Corners`/`Pinched Ends` xuất hai
  thân (mỗi bên một tường), `Inside`/`Outside Center` một thân chữ U.
- `Border=On` bám **channel border đã commit** (mặt trong trùng đường border); app không tự suy lại
  border từ hàng.
- `Mode=Curve` bám **curve được pick trong cùng selection** (sheet: "prompted to select a curve on
  the target surface"), sample theo arc length, `MetalPiece.on_curve`; không có curve thì
  `missing metal piece curve`. `Border=On` thắng khi cả hai cùng bật.
- Mode `Top` là **cutter**: commit thành body riêng nhưng không bao giờ vào mass.

## Bead và mass

- `Bead On Curve`: `Spacing=` **hoặc** `Count=` (đặt cái này xoá cái kia); bead cuối nằm trên end
  fraction; đường kính gốc 1.0 mm là default kỹ thuật của OM9 (sheet chỉ cho scale).
- `SettingsMass`: chỉ cân feature settings đã commit trong phiên, **loại cutter** và báo
  `excluded_cutters`; là text command, không có icon toolbar.

## Từ chối có kiểu (không clamp, không đoán)

`Invalid("prong gem row")`, `("prong layout")`, `("prong site")`, `("prong placement")`,
`("prong overlap")`, `("metal piece mode option")`, `("metal piece gem row")`, `("metal piece border
row shape")`, `("metal piece curve mode option")`, `("metal piece curve length")`,
`Missing("metal piece curve")`, `Missing("metal piece channel border")`, `Missing("gem row
selection")`, `Invalid("settings need gem objects")`.
