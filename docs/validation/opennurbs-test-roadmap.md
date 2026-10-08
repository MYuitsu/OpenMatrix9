Checkpoint hiện tại: **51/51 suite native,39 báo cáo FreeCAD/5.806 kiểm tra**.
Hai file nhẫn giữ bounds0,001mm và area/volume cũ;tools42/42 đạt.

Bộ BRep seam/cực và edit mới:**Rhino5 4/4 đạt,0 đối tượng lỗi**.
Native decode1800 và nhập ngược FreeCAD45 kiểm tra đạt. Rhino chèn knot
vào4 đường UV của trụ sau edit; cơ sở chung chứng minh hình học tương đương,
sai lệch CV tối đa4,44×10⁻¹⁶; không tuyên bố raw UV byte-identical.
UV-reference8 vẫn đóng với incompatibility có phạm vi và giữ chặn xuất V5.

**0 bộ đã chuẩn bị đang chờ;7 đợt(2–8) chưa đóng. Tổng số bộ test còn lại
chưa xác định.** Tiếp theo các tương tác component/owner-edit/transform và
phạm vi correspondence còn thiếu. Chưa full. [Bằng chứng](2026-10-08-seam-trim-complete.md).

| Đợt | Phạm vi | Trạng thái |
|---|---|---|
| 1 | Chuẩn hóa coverage | Hoàn tất |
| 2 | CurveOnSurface / PolyEdge | Đang làm |
| 3 | BRep / solid | Đang làm |
| 4 | Các loại hình học còn lại | Còn mở |
| 5 | Annotation / styles / layout | Còn mở |
| 6 | Resources / block liên kết | Còn mở |
| 7 | Document / merge / payload | Còn mở |
| 8 | Nghiệm thu / tích hợp / build cuối | Còn mở |

**Còn 7 đợt chưa đóng. Tổng số bộ test còn lại chưa xác định**, vì chưa liệt kê
hết fixture theo loại dữ liệu, thuộc tính và tương tác của các đợt còn lại.
Khi một ma trận được xác định, thêm vào hàng đợi và báo số mới; nếu phát hiện
lỗi mới cần regression, giải thích việc tăng số bộ. Không hứa một số hữu hạn
mà chưa có ma trận đầy đủ.

Mẫu báo cáo sau mỗi lần đo:

- **Vừa đo:** tên bộ, đợt, số file đã hoàn tất / đạt / không đạt; lỗi script nếu có.
- **Đã chứng minh:** hình học/dữ liệu/tham chiếu nào; dẫn báo cáo.
- **Còn lại:** số bộ đã chuẩn bị, danh sách tiếp theo, số đợt chưa đóng;
  tổng số bộ chưa biết thì ghi rõ.
- **Tiếp theo:** bộ sẽ chạy hoặc phần phải sửa trước khi chạy; lý do thêm test nếu có.

Full chỉ được xác nhận khi không còn loại/thuộc tính trong phạm vi thiếu adapter
hoặc `unverified`, mọi incompatibility V5 có bằng chứng, và binary cuối qua
nghiệm thu. Kết quả một bộ đo không tự đóng toàn bộ đợt.

## Copy/remap regression completed — 2026-10-08

18 fixture độc lập:36/36 trường hợp native và18/18 vòng FreeCAD đạt;2.611 kiểm
tra native và612 FreeCAD. Regression binary45/45 suite,34 báo cáo/5.343 kiểm tra;
tools42/42. Bộ này kiểm chứng graph nội bộ SDK80 và từ chối V5 an toàn, không
đổi12+6 trường hợp Rhino5 không tương thích thành đạt.0 bộ còn trong hàng chờ
đã chuẩn bị tại checkpoint này;7 đợt (2–8) chưa đóng; tổng bộ test tương lai chưa
xác định. Tiếp theo ma trận seam/cực/đường khép kín, rồi singular trim mapping.

## Seam/cực/periodic mới — 2026-10-08

1 bộ đã chuẩn bị đang chờ Rhino5:8 fixture,8/8 native(229 kiểm tra) và8/8 FreeCAD(32 kiểm tra) đạt. Chưa có kết quả Rhino hoặc kiểm tra decode sau SaveAs. Bộ này bổ sung phạm vi seam và hai đầu cực còn thiếu trong ma trận trước; không tính là chạy lại bộ cũ.7 đợt(2–8) chưa đóng;tổng bộ còn lại chưa xác định.

Seam preparation checkpoint2026-10-08:8/8 native(229 checks),8/8 FreeCAD(32 checks), actual Rhino5 pending. Current regression46/46 suites,35 reports/5375 checks; same verified production binary.1 prepared pending batch,7 packages(2–8) open,total future count unknown. Source/fixture bindings verified; no primary integration or push. [Evidence](2026-10-08-seam-profiles-preparation.md).

## UV-reference owner/copy/remap — 2026-10-08

8/8 fixture native895 và FreeCAD272 kiểm tra đạt. Đã sửa dựng graph lúc ghi và giữ nguyên copy-count userdata; dữ liệu nguồn bất biến. Bộ này bổ sung UV2D tham chiếu owner riêng, chưa có bằng chứng Rhino.1 bộ đã chuẩn bị đang chờ,7 đợt chưa đóng,tổng số bộ còn lại chưa xác định. Regression48/48 native;37 báo cáo/5696 kiểm tra;tools42/42 đạt. [Bằng chứng](2026-10-08-uv-reference-remap.md).

## Actual Rhino5 UV-reference8 — 2026-10-08

Đã đọc8/8,0 đạt/8 không đạt:8 owner hợp lệ nhưng8 root UnsetPoint,8 SaveAs thất bại;không có bản lưu để nhập ngược. Native507 kiểm tra nguồn/graph/từ chối V5 atomic đạt. Bộ đóng với incompatibility có phạm vi.0 bộ chuẩn bị đang chờ,7 đợt chưa đóng,tổng còn lại chưa xác định. [Bằng chứng](2026-10-08-uv-reference-rhino5.md).


Native Qt menu regression (`native-qt-exchange-user-ring-crash`): 2 fixtures (new oval Import and independent box Export),8 QAction checks +18 UI checks passed after GIL fix. Actual ring source preserved;52 host geometries valid.0 prepared Rhino batches,7 packages open,total future batches unknown. [Evidence](2026-10-08-native-menu-gil-fix.md).


`oval-wireframe-display-diagnosis`:1 fixture, read-only native basis and GUI comparison completed; display polygon-line limitation confirmed, no renderer fix claimed.0 prepared Rhino batches,7 packages open,total future batches unknown. [Evidence](2026-10-08-oval-display-diagnostic.md).

## Wireframe CAD — 2026-10-08

Chẩn đoán polygon-line tại checkpoint trên đã được xử lý có phạm vi: BRep dùng
native CAD edges +trimmed isocurves; giữ geometry/source và mode theo view.
Ma trận CAD13 fixture và hồi quy mục tiêu:8 báo cáo FreeCAD/105 kiểm tra đạt
trên runtime mới, gồm menu Qt và hai file nhẫn.0 bộ Rhino mới đã đo.
Checkpoint geometry51 suites cũ không được tính là chạy lại trên binary này.
0 bộ đã chuẩn bị đang chờ,7 đợt chưa đóng; tổng bộ còn lại chưa xác định.
[Kết quả và giới hạn](2026-10-08-cad-wireframe.md).

## Import nhanh — 2026-10-08

Bộ `fast-preservation-import-and-regressions` thuộc đợt8: một nhẫn đo2 lượt
paired, một nguồn hỗn hợp19 record và targeted regressions;12 báo cáo/
151 kiểm tra đạt. Import trung vị 25.85→13.71s;
bao gồm display 29.23→16.82s.
0 bộ Rhino chờ,7 đợt chưa đóng,tổng bộ tương lai chưa xác định.
[Kết quả, giới hạn](2026-10-08-fast-3dm-import.md).
