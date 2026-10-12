# Gumball — đối chiếu ảnh/PDF và revision ba spec lõi

Ngày 2026-10-11. Phạm vi: cập nhật yêu cầu `OM9-INFO-020/021/022`, không
triển khai lại plugin, không nâng trạng thái từ `partially_implemented`.

## Nguồn đã đọc

- Ảnh bốn viewport do người dùng cung cấp, được xác nhận là Rhino 5/Matrix,
  không phải ảnh OM9. SHA256:
  `4722a3aee52bbce1b0e99b88afaacd46e3547a04ed2878a714a4363ffb03fd19`.
- Bản trích `matrix8_book_1 Copy.pdf`: chỉ 3 trang PDF, tương ứng trang in
  35–37. Đã extract text và render/view đủ ba trang. SHA256:
  `62e1debade716296ff6ad65bc252d91e1f456326512e5a8c40d44213a352308d`.
  PDF45–47 trong evidence trước là số trang của quyển đầy đủ; History PDF48
  là mốc continuation đã ghi trước đó, không nằm trong bản trích hiện tại.
- [Rhino 5 Gumball Help](https://docs.mcneel.com/rhino/5/help/en-us/commands/gumball.htm)
  và [Rhino 5 Gumball Options](https://docs.mcneel.com/rhino/5/help/en-us/options/modeling_aids_gumball.htm)
  đọc ngày 2026-10-11 để kiểm tra modifiers, alignment, menu và preferences.

Private images/PDF/rendered manual pages được giữ ngoài source public. Các
spec chỉ chứa behavioral summaries và lựa chọn OM9, không chứa artwork.

## Phân biệt evidence và lựa chọn

Ảnh thể hiện cùng Gumball trong bốn view, arrows/arcs thanh, Scale squares
trên biên âm selection và dotted leaders, plane indicator hai màu có cross,
origin nhỏ và menu vòng trắng dưới-trái có đuôi chấm. Ảnh dùng RGB axes;
PDF dùng X đỏ/Y cyan hoặc xanh lam/Z vàng. OM9 lấy ảnh làm chuẩn màu và
hình thức, lấy PDF làm chuẩn thao tác lõi.

Ảnh không chứng minh thuật toán bounds cho mọi object, default alignment,
hover, Osnap hay pixel settings của Rhino. Shared Rust frame/session, phép
tính extents theo frame, fallback cho pivot ngoài bounds và profile logical
px ở spec020 là lựa chọn OM9 để kết quả đo được. Default Off/CPlane đã duyệt
được giữ; World dùng cho fixture so ảnh. Không đổi workspace background,
selection color, camera ordering hoặc đưa Smart MSR/Clayoo vào ba spec này.

## Native observation trước khi sửa spec

Probe riêng chạy trên shared FreeCAD host và plugin visual đã được kiểm
trước đó, với profile/document riêng. Không rebuild FreeCAD hoặc plugin.

- Plugin SHA256:
  `A058D753A01AFC5369F3CD1B7552C5DE8E78373E7DAB271149F63C224BFAB8DF`.
- Shared host SHA256:
  `98EEF899354A8AA84CFA2934E6E1A3C8558C7E6FFF3B89C347C06731F06018A8`.
- Artifact cohort trong workspace FreeCAD:
  `build/shared-host-tests/gumball-reference-compare-20261011`.
  `results.json`, `shared-host-launch.json` và `om9-four-views.png` là native
  observations; ảnh ghép render cả bốn scenes ở cùng trạng thái, không lần
  lượt activate view để tạo cảm giác có bốn overlays.
- 5/5 checks của probe đạt, actual host exit0: tồn tại bốn views, Command
  tồn tại, On bật selection, Off dọn overlay, geometry/Undo không đổi.
  Đây là checks của probe, **không phải năm tiêu chí parity đã đạt**.
- Khi Perspective active, overlays theo slot0..3 là `[false,true,false,false]`.
  Activate từng ô chuyển overlay sang ô đó; chưa bao giờ có đồng thời 4/4.
  CPlane basis đổi theo active view; relocated owner cache giữ pivot qua
  idle focus. Điều này không chứng minh multi-view refresh/pick/preview.
- Source `GumballController` giữ một view và một overlay. `GumballOverlay`
  dùng Scale offset cố định, plane một màu gần tâm và menu disk đặc. Native
  render xác nhận widget gần tâm thay vì Scale grips ở biên object trong ảnh.

## Phần cần triển khai/nghiệm thu

Spec020 quy định Command lifecycle, collection overlay, camera-specific
visibility/picking, bounds-derived Scale, plane hai màu/menu ring, pixel/DPI
profile, gesture/modifiers và core preferences. Spec021 quy định World/Object/
active CPlane trên shared frame, relocated override và Reset. Spec022 quy
định origin/X/optional Y, parser CPlane/world, Ctrl timing và shared Relocate.

Checklist multi-view/visual/settings mới đều còn mở. Native geometry slice
trước vẫn có giá trị trong phạm vi đã ghi; Scale parametric/Link, Object frame
adapters thiếu, full snap parity, cold pivot persistence và host old-Redo
limitation chưa được revision tài liệu này giải quyết.

Rà tài liệu: 46 checks UTF-8, liên kết, metadata/status ba ID, manifest hashes
và ledger đạt; `git diff --check` exit0; public source audit exit0/zero errors.
Các catalog/ledger/manifest đang có cập nhật Curve song song được giữ; checks
metadata giới hạn đúng ba ID Gumball và hash tài liệu vừa đổi. Không chạy lại
Rust/native geometry suites vì không đổi product code.
