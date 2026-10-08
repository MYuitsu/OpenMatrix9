# Tăng tốc import 3DM — 2026-10-08

Báo cáo lịch sử từ workspace kiểm chứng riêng. Cách build bản public nằm trong
[build-windows.md](../build-windows.md); đường dẫn H: dưới đây chỉ dành cho máy
đã thực hiện phép đo. Không cần có runtime/launcher riêng đó để build source này.

Đã tối ưu đường **Preserve source data** dùng chung menu/File/API trong source
phát triển; chưa tích hợp source chính hoặc push. SDK giữ commit
`eb92af3ba1806b0a34a99aba0d3bda83e3d46083`.

## Kết quả trên nhẫn người dùng

File `C:/Users/nguye/Downloads/nhan oval 11.91x8.37x4.97.3dm`:
31 record nguồn,29 object top-level,31 Part shapes kể cả affine preview.
Hai lượt đo không bật cProfile trên binary cuối, mỗi lượt so sánh1 và4 worker:

|Phép đo|Tuần tự, đã bỏ chữ ký lặp|4 tiến trình chuyển đổi|
|---|---:|---:|
|Import, trung vị|25.85 s|13.71 s|
|Import + hiển thị đầu tiên, trung vị|29.23 s|16.82 s|

Thời gian thay đổi theo tải máy. Đây là đo trên file cụ thể, không cam kết mọi
3DM tăng tốc bằng nhau. Baseline trước tối ưu25–32 s import;140 lần tạo
fingerprint giảm còn84, và chữ ký cuối vẫn giống baseline đã decode.
[Số liệu từng lượt, hashes và báo cáo](import-fast-20261008/summary.json).

## Thay đổi và kiểm chứng

- Tạo chữ ký nguồn một lần sau khi binding đầy đủ, bỏ lần tính trung gian.
- Chuyển đổi native bằng tối đa4 **tiến trình riêng**, không đọc SDK đồng thời
  trong các thread cùng host. openNURBS có bộ đếm SubD/error toàn cục nên bản
  thử dùng thread dù chạy nhanh đã bị review bác bỏ và không được dùng.
- UUID merge theo đúng thứ tự ModelGeometry rồi RenderLight; member selection
  vẫn xét cả thuộc tính definition và graph. Geometry/definition/affine preview
  có namespace staging riêng. FreeCAD binding/recompute/Undo vẫn ở host.
- Hết, lỗi hoặc thiếu worker được kiểm tra trước transaction. Khi một worker
  lỗi, sibling bị kill/join trước khi snapshot cleanup. Không fallback để che
  lỗi chuyển đổi, không đổi thuật toán BRep hoặc tolerance.

Binary cuối qua **12 báo cáo FreeCAD/151 kiểm tra**;
native đã build/link lại và đạt7/7 suite mục tiêu (geometry/archive/preservation/
inventory/block/hai file nhẫn). Không tính51 suite lịch sử là chạy lại toàn bộ.
Các kiểm tra bao gồm đối chiếu toàn record/chữ ký/CAD với golden tuần tự,
menu Qt Import/Export, hai file nhẫn, Wireframe/viewport/Link/density, malformed
input/rollback, FCStd nguồn độc lập, Undo/Redo theo preservation suite.
Ma trận mới có8 điểm top-level,8 điểm definition, SubD, light và block:
tuần tự/song song cùng manifest, thứ tự, thuộc tính/tolerance;19 record giữ đủ.
Test-only helper khiến một worker thoát lỗi khi3 sibling đang chờ; host trả lỗi
nhanh, document không đổi và không còn tiến trình phụ.

Review cuối không còn finding cần xử lý. Debug helper filename đã sửa bằng
CMake, chỉ kiểm chứng bằng source; chưa chạy native Debug runtime. Không chạy
lại Rhino5 vì lần này không đổi converter/export/V5 policy; bằng chứng Rhino
trước vẫn có phạm vi lịch sử, không được gọi là nghiệm thu binary này.

## Dùng bản nhanh

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File H:\FreeCAD-src\build\launch-om9-import-fast.ps1
```

Runtime `H:/FreeCAD-src/build/om9-import-fast-sdk`; launcher mở profile riêng
và nhẫn người dùng. Cửa sổ cũ vẫn tải binary cũ, cần dùng cửa sổ bản nhanh.
Helper phải nằm cạnh FreeCAD executable và được build/install cùng module.

Đường Geometry only vẫn chạy tuần tự. Preserve có ít hơn8 record trong mỗi
nhóm, file lớn hơn16 MiB hoặc máy chỉ có1 worker chạy tuần tự để hạn chế chi
phí đọc lại/memory. `OM9_3DM_WORKERS=1` dùng làm oracle tuần tự; tối đa4.
Thời gian dựng Wireframe/binding vẫn cần vài giây và nằm trong số liệu trên.
Giới hạn Wireframe đã công bố vẫn giữ nguyên.

**0 bộ Rhino đã chuẩn bị đang chờ;7 đợt(2–8) chưa đóng; tổng số bộ test tương
lai chưa xác định.** Tối ưu này thuộc đợt8, không tuyên bố full openNURBS.
Tiếp theo người dùng thử import bản nhanh; tiếp tục ma trận plan còn thiếu.

Phiên thử trực tiếp đã mở đúng runtime nhanh: PID77736 responding,
startup `ok:true`,29 top-level nhẫn người dùng.
[Bằng chứng](import-fast-20261008/interactive-startup.json).
