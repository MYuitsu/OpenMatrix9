# OM9-L2-04-15 — 02-febe-prong-channel-metal-and-bead-settings

Trạng thái: **implemented, automated evidence recorded, human UI review open**.
Dashboard giữ cả 10 L3 ở `WIP` vì chưa có người ký review M06-UI; tài liệu này không phải một
chữ ký.

## Phạm vi đã làm

| L3 | Lệnh | Adapter | Ghi chú |
| --- | --- | --- | --- |
| `OM9-L3-04-15-001` | `BuilderProngAdder` | `session::settings_commands` | 22 preset prong theo tên, staging preview → Enter/Cancel |
| `OM9-L3-04-15-002` | `BuilderProngEditor` | 〃 | sửa tại chỗ từ payload `om9.settings.prong` |
| `OM9-L3-04-15-003` | `BuilderProngOnSurface` | 〃 | prong dựng theo pháp tuyến surface + UV |
| `OM9-L3-04-15-004` | `BuilderMetalPiece` | 〃 | 6 mode mở + 3 mode kín; `Border=On` bám channel border đã commit |
| `OM9-L3-04-15-005` | `BuilderChannelBorderCurve` | 〃 | hai guide curve trái/phải, extension Line/Arc/Smooth |
| `OM9-L3-04-15-006` | `BuilderBeadOnCrv` | 〃 | `Spacing=` hoặc `Count=`, taper, direction F4/Down/Object |
| `OM9-L3-04-15-007` | `BuilderBeadOnSurface` | 〃 | một bead tại UV được pick |
| `OM9-L3-04-15-008` | `BuilderPullToRail` | 〃 | `Mode=Rigid|Deform` trên rail là đường tròn được chọn |
| `OM9-L3-04-15-009` | (option của lệnh prong) | 〃 | `Ends=`/`Spacing=`, không có icon riêng |
| `OM9-L3-04-15-010` | `SettingsMass` | 〃 | khối lượng từ metal owner; cutter bị loại khỏi mass |

Dùng chung đúng luật L2: input `settings::gems::GemRow` dựng từ record gem của owner gems,
output là body + payload settings; không có kernel hay store thứ hai.

## Bằng chứng (chạy thật, có log)

| Lệnh | Kết quả | Log |
| --- | --- | --- |
| `cargo check -p om9-core -j 2` | exit 0 | `OM9-L3-04-15-001/logs/check-core.log` |
| `cargo check -p om9-app -j 2` | exit 0 | `OM9-L3-04-15-001/logs/check-app.log` |
| `cargo test -p om9-core --lib -j 2 settings::` | 90 passed | `OM9-L3-04-15-001/logs/core-settings.log` |
| `cargo test -p om9-core --test settings_family -j 2` | 14 passed | `OM9-L3-04-15-001/logs/core-family.log` |
| `cargo test -p om9-app --test settings_commands -j 2` | 24 passed | `OM9-L3-04-15-001/logs/app-settings.log` |
| `cargo test -p om9-core -j 2` | 781 passed / 70 suite | `OM9-L3-04-15-001/logs/core-all.log` |

Tổng hợp máy đọc: `implementation/evidence/M06/OM9-L3-04-15-001/summary.json`.
Mỗi L3 có `implementation/{design,fixtures,evidence}/M06/<L3-id>/` với
`source_crosswalk.json`, `api.md`, `positive.json`, `negative.json`, receipt `-03`/`-04`.
Sinh lại bằng `python tools/record_m06_l2_04_15_evidence.py`.

## Giới hạn đã ghi trong record (không claim)

- `4 Prongs Close` dựng đúng luật sheet: hai prong mỗi bên, sát nhau hơn `4 Prongs 0 Degrees`
  và vẫn là hai prong riêng.
- Hàng quá sát có thể đặt prong đối diện của hai viên lên cùng một điểm: builder gộp thành một
  prong chia sẻ và báo `merged_prongs`, không từ chối style.
- `Metal Piece` mode `Curve` bám theo curve được pick trong cùng selection (`Missing("metal piece curve")` khi không có), hoặc theo channel border đã commit với `Border=On`.
- `SettingsMass` và row-end modes không có icon toolbar; chúng là text command và option.
- `Pull To Rail` chỉ nhận rail là đường tròn được chọn.

## Việc kế tiếp

1. Mở cửa sổ OpenMatrix9, chạy từng icon trên: preview → đổi một option → `Enter` → `Cancel`.
2. Ký review M06-UI cho từng L3; khi đó dashboard mới chuyển `WIP` → `DONE`.
3. Không còn style prong nào bị từ chối vì lý do hình học; chỉ còn gate review M06-UI.
