# Wireframe CAD kiểu Rhino/Matrix — 2026-10-08

OM9 Wireframe dùng cạnh CAD và isocurve trên miền mặt đã trim. BRep không còn
vẽ đường chéo tam giác của render mesh. Shaded dùng mặt native; mỗi viewport
đổi mode độc lập. Nhẫn oval được đo/chụp trong FreeCAD thực tế; source và
Shape hash giữ nguyên.

Source phát triển `H:/FreeCAD-src/build/om9-dev`, runtime riêng
`H:/FreeCAD-src/build/om9-cad-wire-sdk`. Không tích hợp source chính, commit,
push hoặc phát hành. openNURBS giữ commit
`eb92af3ba1806b0a34a99aba0d3bda83e3d46083`.

## Kết quả trên binary cuối

8 báo cáo FreeCAD thực tế, **105/105 kiểm tra đạt**, process exit code0 do
harness kiểm tra. [Báo cáo, source/binary hash và ảnh](cad-wireframe-20261008/summary.json).

| Bộ | Đạt | Phạm vi |
|---|---:|---|
| CAD Wireframe | 25 | Box không có đường chéo; edge/face picking; mixed views; mesh; hole trim; edit/cache; knot density; FCStd; nhẫn oval |
| Viewport | 45 | Camera, mode theo view, Undo, selection, lifecycle và chuyển workbench |
| Hai file nhẫn hồi quy | 10 | Hình học hợp lệ, native manifest/archive, UUID/graph, FCStd reopen |
| Native Qt Import/Export | 8 | QAction thực tế nhập nhẫn oval, xuất box; không crash GIL; source không đổi |
| Document lifecycle | 2 | Đóng document và ứng dụng |
| External reflected Link | 5 | Source không hoạt động vẫn có isocurve; geometry/mode được phục hồi |
| Geometry import density | 5 | Native V5 sphere độc lập, density −1/0/1/3/33 nhập đúng |
| Nested external Link/container | 5 | App::Part có child/internal Link, bridge ngoại, Link cuối phản chiếu; phục hồi/dọn scene nodes |

Ma trận CAD có13 fixture:6 hình học chính,1 external reflected Link,
5 density và1 nested container/link. Hồi quy dùng thêm hai file nhẫn gốc và
box menu. Không dùng assertions hoặc fixture làm phần trăm full.

![Nhẫn oval Wireframe CAD](cad-wireframe-20261008/cad_wireframe_smoke/user-ring-wire-iso.png)

## Hiển thị và bảo toàn

- Native CAD edges/vertices giữ picking/selection. Isocurves chỉ là đường
  hướng dẫn, không tạo document objects và không pick được.
- Isocurve cắt theo pcurve/trim, kiểm tra miền mặt; không chạy xuyên lỗ.
  Analytical plane chỉ vẽ biên.
- `m_wire_density` nguồn được đọc vào `ViewObject.OM9IsoCurveDensity` ở cả
  geometry/preservation import, giữ qua FCStd. −1: chỉ biên;0: đường knot;
  1: thêm đường trong miền không có knot nội;N≥2: N−1 đường mỗi knot span.
  Native source archive giữ thuộc tính nguồn.
- Màu quá tối được tăng tương phản trên nền đen trong Wireframe, giá trị
  LineColor không đổi. Các màu còn lại dùng RGB nguồn.
- Link dùng source provider, gồm source ở document khác. Cache đổi theo
  Shape/density/màu. Thoát OM9 thu hồi adapter và phục hồi mode.
- Mesh giữ polygon Wireframe; preview không thay dữ liệu native/BRep.

## Review và giới hạn

RED thực tế tái hiện tam giác render, cạnh tối, thiếu isocurve ở external
source và mất density khi geometry import. Báo cáo RED được giữ cùng GREEN.
Review bổ sung hai rủi ro thứ tự phục hồi shared container/link graph: thu
thập mọi mode trước unwrap và bỏ ghi mode qua forwarding-only Link. Đây là
hardening từ review; nested fixture đạt cả trước/sau, không gọi là RED→GREEN.

Deflection0,005 mm dành cho **đường hiển thị**, không đổi tolerance hình học,
bounds0,001 mm hoặc ngưỡng area/volume đã chấp thuận. Density hiển thị giới
hạn32/có warning; density33 nguồn vẫn giữ. Giới hạn4096 đường tham số/mặt,
65.536 điểm/đường,2 triệu điểm/object. Lỗi preview được báo và giữ cạnh CAD;
không tự chuyển dữ liệu gốc thành mesh.

Chưa có oracle Rhino5 mới cho số isocurve hoặc ảnh pixel. Camera/background/
antialiasing có thể khác Rhino. Affine compound preview chưa giữ riêng density
khác nhau của từng member; mixed affine preview children còn default1.
Đổi density ViewObject chỉ đổi hiển thị; geometry-only export chưa ghi thay
đổi này vào native attributes. Không công bố tương đương mọi thuộc tính
hiển thị Rhino hoặc full openNURBS.

## Tự mở để thử

```powershell
rtk proxy powershell -NoProfile -ExecutionPolicy Bypass -File H:\FreeCAD-src\build\launch-om9-cad-wire.ps1
```

Launcher mở runtime mới với nhẫn
`C:/Users/nguye/Downloads/nhan oval 11.91x8.37x4.97.3dm`, profile riêng, giữ
cửa sổ mở. Chọn Wireframe/Shaded trong menu viewport để so sánh.

**0 bộ Rhino5 đã chuẩn bị đang chờ;7 đợt(2–8) chưa đóng; tổng bộ test còn lại
chưa xác định.** Bộ này hoàn tất hiển thị CAD có phạm vi, chưa đóng đợt8.
Checkpoint geometry cũ51 native suites/39 runtime reports/5806 checks là
bằng chứng lịch sử binary trước, không tính là chạy lại trên binary mới.
[Roadmap](opennurbs-test-roadmap.json).

## Skill và phiên thử trực tiếp

Người dùng đồng ý lưu quy tắc Wireframe mới vào skill. Đã sửa repository
`skills/openmatrix9-workspace-contract` và cài cho tài khoản hiện tại tại
`C:/Users/nguye/.codex/skills/openmatrix9-workspace-contract` (thư mục này
trước đó chưa tồn tại).9 file nguồn/cài khớp SHA256; cả hai quick_validate đạt.
Baseline retrieval độc lập chỉ tìm được hợp đồng polygon cũ và thiếu quy tắc
RGB/density/BRep-vs-mesh; forward retrieval mới tìm đúng hợp đồng và giới hạn,
đồng thời kiểm tra linked assets. Đây là test tra cứu skill, không thay test
runtime. [Bằng chứng đồng bộ](cad-wireframe-20261008/skill-sync.json).

Phiên interactive mở đúng module runtime mới, nhập29 top-level objects từ
nhẫn oval, startup `ok:true`, process25392 responding.
[Startup](cad-wireframe-20261008/interactive-startup.json).
