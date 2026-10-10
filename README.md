> **OpenMatrix9 0.0.2 (experimental, Windows x64)**: [Download release](https://github.com/MYuitsu/OpenMatrix9/releases/tag/v0.0.2) · [Cài đặt tiếng Việt](INSTALL.vi.md) · [English installation](INSTALL.en.md). This native package targets **official FreeCAD 1.1.4** and installs in the versioned user Mod folder. See [fresh compatibility evidence and limits](docs/validation/2026-10-10-stock-freecad-1.1.4.md). Historical development-host reports below do not certify every feature on this stock host. The [0.0.1 release](https://github.com/MYuitsu/OpenMatrix9/releases/tag/v0.0.1) remains available for its separate development 27.1 portable host.

# OpenMatrix9

Workbench FreeCAD lấy giao diện Matrix9 làm tham chiếu. Rust quản lý danh mục và trạng thái; C++/Qt dựng giao diện và tích hợp lệnh FreeCAD.

Rust ưu tiên số1 theo yêu cầu2026-10-09: Safe Rust sở hữu portable logic/state/validation/dữ liệu/index/cache/worker; C++ chỉ giữ cầu nối native.7 skill OM9/17 bản đã kiểm chứng. Chuyển logic Phase2 hiện tại đã nghiệm thu trong phạm vi [kế hoạch đã duyệt](docs/superpowers/plans/2026-10-09-phase2-rust-owned-core.md), dùng runtime riêng om9-phase2-rust-sdk. [Báo cáo Rust/native và ứng dụng](docs/validation/2026-10-09-phase2-rust-owned-core.md). [Policy và bằng chứng](docs/validation/2026-10-09-rust-first-skills.md).

Kiểm chứng Phase2 trực tiếp từ Rhino5: [lệnh chạy script và report](docs/validation/2026-10-09-rhino5-phase2-manual-verification.md). Entry point mở phiên test riêng, giữ nguyên bản vẽ đang mở; lần chạy của người dùng PASS với ba fixture125 kiểm tra Rhino/35 OM9/55 nhập lại SaveAs; sau sửa cảnh báo STA, harness9/9 đạt và không còn cảnh báo vòng chờ.

Sidebar hiện có 7 khối, 18 nhóm menu và 11 nút nhanh. Những chức năng chưa port được giữ disabled. Bốn viewport và thuật toán trang sức đang ở giai đoạn tiếp theo.

Xem [hướng dẫn mở, build và kết quả kiểm tra](docs/menu-validation.md), [tiến độ](docs/openmatrix9-progress.json) và [cách đọc tài liệu theo skill](docs/openmatrix9-skills.md).


Tiến độ openNURBS trong bản phát triển, cập nhật 2026-10-08. ✅ là phạm vi đã kiểm chứng; ⬜ là chức năng chưa hoàn thành. Bản phát triển chưa tích hợp vào checkout public.

Import 3DM trong bản phát triển đã được tăng tốc bằng tiến trình native riêng, giữ chữ ký và archive nguồn. [Số liệu và cách mở bản nhanh](docs/validation/2026-10-08-fast-3dm-import.md).

|Chức năng|Ý nghĩa và phạm vi|Tiến độ|
|---|---|---|
|Import 3dm nhanh (Preserve)|Tối đa4 tiến trình native; giữ dữ liệu/UUID, kiểm tra lỗi trước transaction; đã đo nhẫn cụ thể, còn Geometry only tuần tự.|✅|
|Migrate 3dm copy origins|Khôi phục provenance của block copy bằng UUID → object gốc; giữ geometry, target, Undo/Redo và FCStd.|✅|
|Native rational proxy copy|Giữ payload NURBS rational 2D nguyên bản khi migration và export.|✅|
|Migrate affine targets|Chỉ rõ definition đích cho reference affine; giữ ma trận native, geometry và Undo/Redo/FCStd, rollback graph lỗi.|✅|
|Fork archive namespace|Tách scope cho archive copy đệ quy; giữ geometry, block native, Undo/Redo và FCStd.|✅|
|Recursive shared CAD/mesh và NURBS|Copy graph dùng chung geometry; giữ biến đổi affine, mesh native và CRC NURBS rational.|✅|
|Stable source copy IDs|Giữ UUID đối tượng copy, member và proxy khi export lặp lại và lưu/mở FCStd trong cùng scope/host; từ chối UUID trùng geometry/layer.|✅|
|File Export / Export3dm|File Export, menu và CMD bảo toàn graph native; Geometry only là lựa chọn bỏ dữ liệu có ghi rõ.|✅|
|Migrate legacy hosts|Nâng cấp CAD/mesh/instance legacy có provenance; giữ UUID, geometry, liên kết và Undo/Redo/FCStd.|✅|
|Transform native TextDot / PointCloud|Di chuyển, xoay đối tượng và member block; giữ tọa độ, normal, màu, Undo/Redo và FCStd trong phạm vi đã test.|✅|
|Export riêng member block|Xuất các member/proxy đã test; giữ UUID và tọa độ. Khối hướng âm trong nhánh giữ nguồn bị chặn; Geometry only giữ dấu qua block phản chiếu, gồm toàn bộ block có proxy đã test.|✅|
|Giữ nguồn với khối hướng âm, gồm member block|Còn bảo toàn đầy đủ UUID/history/plugin graph qua Rhino5; hiện từ chối trước khi ghi để tránh đảo hướng.|⬜|
|Hướng solid chưa xác định (+2)|BRep chuyển được thành một solid OCCT: giữ winding và dấu qua biến đổi; block lồng đã kiểm chứng trong Rhino5 và nhập ngược FreeCAD.|✅|
|BRep +2 nhiều shell, khoang rỗng và đảo chiều|Giữ shell/face, dấu và khoang rỗng trong phạm vi đã test; giữ native khi OCCT chưa có solid chỉnh sửa hợp lệ. Còn nghiệm thu toàn ma trận.|✅|
|BRep shell cong và khoang rỗng|Bốn mẫu trụ/cầu/torus qua native, FreeCAD và Rhino5 Open/SaveAs; graph/UUID giữ nguyên. Phép đo Rhino chính xác đạt8/8 với ngưỡng giữ nguyên; báo cáo mặc định1/4 giữ riêng. Còn shell tiếp xúc/giao nhau tổng quát.|✅|
|BRep giữ nguồn sau FCStd|Mốc CAD riêng đi qua cùng bộ lưu FreeCAD; không nhận nhầm làm tròn khi lưu là sửa hình học. Giữ kiểm tra sửa thật và Undo/Redo; project cũ cần migration riêng.|✅|
|NURBS 2D ra ngoài mặt phẳng|Giữ tọa độ Z khi transform bằng biểu diễn 3D; giữ weight và knot nguyên bản.|✅|
|TextDot native|Hiển thị chữ, sửa điểm neo, Unicode, font/cỡ chữ và cờ; giữ dữ liệu native khi export, copy block và mở lại FCStd. Preview còn giới hạn.|✅|
|PointCloud native|Sửa điểm/normal double, màu RGBA và plane; giữ dữ liệu khi copy block, Undo/Redo và mở lại FCStd; hiển thị điểm màu.|✅|
|PointCloud geometry-only|Import/export điểm double cùng CAD/mesh; làm phẳng block lồng nhau, giữ normal/RGBA/plane và FCStd.|✅|
|Preview PointCloud affine|Hiển thị native cloud trong block hỗn hợp; tự cập nhật từ member và kiểm tra dữ liệu trước export.|✅|
|Bảo vệ intensity khi xuất Rhino 5|SDK Rhino5 đời2013 làm mất intensity; giữ trong FCStd và chặn export trừ khi người dùng chủ động xóa.|✅|
|Kiểm chứng bằng ứng dụng Rhino 5|10/10 mở/lưu/mở lại; nhập ngược33/33. Kiểm chứng hình học bổ sung50/50 ở ngưỡng0,001 mm; bbox Rhino cũ49/50 được giữ riêng.|✅|
|RDK3 cho Rhino5|Ghi XML UTF-8 đúng layout v3 khi không có tài nguyên nhúng; reader SDK2013 đọc đủ Unicode. Resource chưa tương thích được chặn trước export.|✅|
|Mặt tròn xoay và chiều solid|Giữ UV/trim gốc và vùng trim khi đảo U/V; giữ dấu thể tích qua block phản chiếu và export/reimport trong phạm vi đã test.|✅|
|Hatch/HatchPattern native|Giữ pattern, loop NURBS rational và vị trí khi export riêng; đúng spacing mm/cm, Undo/Redo, copy và FCStd trong phạm vi đã test.|✅|
|Hatch affine native|Biến đổi loop/pattern theo ma trận shear, scale không đều và phản chiếu; copy/block/proxy mm/cm trong phạm vi test; còn kiểm chứng hiển thị Rhino5.|✅|
|Hatch current fields|Sửa origin/trục/base point/góc/tỷ lệ/pattern; giữ native loop khi copy/block, Undo/Redo và FCStd.|✅|
|Hatch boundary preview|Hiển thị đường biên từ native NURBS; dựng lại từ FCStd, độc lập dữ liệu export.|✅|
|Hatch loop native và dữ liệu con|Giữ NURBS/Arc/Polyline/PolyCurve lồng nhau; kiểm tra userdata/reference và gradient None còn dữ liệu.|✅|
|Hatch typed loop edits (API)|Sửa CV/weight/knot/radius/điểm/đường ghép, thêm/xóa biên; Undo/Redo, copy/block và FCStd trong phạm vi test; cần bản sửa core FileIncluded.|✅|
|Hatch loop editor: numeric controls|Double-click/menu sửa trường số của5 kiểu curve, role/thêm circle/xóa loop; Undo/Redo, copy và FCStd trong phạm vi đã test.|✅|
|Hatch loop editor: structural rows|Nhân đôi/xóa CV, knot, point, parameter và segment; kiểm tra native, Undo/Redo và FCStd trong phạm vi test.|✅|
|Hatch loop/pattern/render đầy đủ|Còn tạo/đổi kiểu native, curve tham chiếu/surface, nội dung pattern và fill/dash; kiểm chứng Rhino5 thực tế.|⬜|
|Hatch qua SDK Rhino5 độc lập|Reader2013 kiểm tra5 kiểu curve, loop đã sửa/base0 và block mm/cm;32 ca archive native; chưa kiểm chứng renderer Rhino5.|✅|
|Gradient khi xuất Rhino5|Đã xác nhận mất dữ liệu ở định dạng v5; giữ snapshot và chặn export.|✅|
|CurveOnSurface archive recovery|Đọc đủ curve tham số, approximation tùy chọn và surface; giữ archive gốc trong FCStd; reader có bản sửa được công bố.|✅|
|CurveOnSurface native transform (kernel)|159 ca NURBS/Plane/Rev/Sum/Extrusion: đổi surface và approximation cùng nhau, giữ UV và dữ liệu native; còn các mapping tham chiếu/UV khác.|✅|
|CurveOnSurface schema native|Dữ liệu source của curve con/surface, metadata và tham chiếu PolyEdge; lưu lại đúng trong FCStd, phát hiện đích tham chiếu bị thiếu.|✅|
|PolyEdge: liên kết model nguồn|Graph native riêng giữ đúng curve/Brep edge/trim, domain và chiều đảo trong các ca đã kiểm tra; báo lỗi tham chiếu sai trước import.|✅|
|PolyEdge: chia sẻ và kiểm tra tham số|Dùng chung target native; giữ closure và lifetime; giới hạn công việc giữa các root; kiểm tra domain edge/proxy trên Brep box.|✅|
|PolyEdge: kiểm tra trim cong|Kiểm tra đầu cuối và15 điểm nội miền bằng phép chiếu vật lý rồi đo lại native; thêm seam khép kín và tham số Arc/type2. Giữ riêng giới hạn phép đo.|✅|
|CurveOnSurface: xuất profile không có C3|30 profile UV Line/Arc/Nurbs/Polyline/PolyCurve trên Nurbs/Plane/Rev/Sum/Extrusion qua API và Open/SaveAs Rhino5; nhập ngược đúng trường native; placement, Undo/Redo, copy/xóa và FCStd đã test.|✅|
|CurveOnSurface: UV lồng nhau đã kiểm chứng|6 mẫu Line trên UV NURBS 2D bilinear, không rational, hình chữ nhật trong miền: Rhino5 API giữ đủ dữ liệu, nhập ngược đúng trường native; OM9 xuất V5, di chuyển, copy/xóa, Undo/Redo và FCStd đã test; 12 file OM9 xuất qua Open/SaveAs Rhino5, nhập ngược đúng dữ liệu native.|✅|
|CurveOnSurface: UV shear/rational/bậc cao đã kiểm chứng|90 mẫu UV single-span2x2 không rational/rational dương và4x4 không rational, CV trong miền; 5 kiểu curve con qua API Rhino5 và nhập ngược đủ trường native. OM9 V5/lifecycle đã test; Open/SaveAs180 file OM9 xuất và nhập ngược1.081 kiểm tra đạt.|✅|
|CurveOnSurface: UV lồng nhau tổng quát|Còn rational bicubic, bậc/span khác, UV đảo chiều, lồng sâu và các kiểu con khác; các trường chưa được kiểm chứng vẫn bị chặn khi xuất V5.|⬜|
|Dữ liệu plugin do Rhino5 SaveAs thêm|Giữ đủ archive trong FCStd; chặn xuất chọn lọc khi chưa biết đủ dependency của bảng plugin. Chưa có adapter đầy đủ.|⬜|
|CurveOnSurface: bảo vệ dữ liệu C3|Rhino5 thực tế bỏ đối tượng có m_c3 trong ba mẫu khớp hình học; OM9 giữ nguồn và từ chối xuất trước khi thay file đích.|✅|
|CurveOnSurface đầy đủ|Còn edit/display, giải quyết/remap tham chiếu, các mapping UV/type, export và kiểm chứng trên Rhino5.|⬜|
|Full openNURBS|Còn references/UV, multi-shell, annotation, geometry, resources/document/version và integration. [Danh sách còn thiếu](docs/validation/2026-10-07-full-opennurbs-gap-audit.md).|⬜|

Lần Rhino5 trước khi sửa RDK:8 file, FreeCAD reimport25/25; đối chiếu nguồn/export40/41, chưa đạt toàn bộ. [Kết quả và vấn đề RDK](docs/validation/2026-10-07-rhino5-application-test.md).

Kiểm tra: native35/35, GUI1910/1910, nhẫn894/894; Rhino5 10/10 vòng mở/lưu/mở lại, nhập ngược33/33. Kiểm chứng hình học bổ sung50/50 ở ngưỡng0,001 mm; bbox Rhino5 cũ49/50 được giữ riêng vì bỏ sót điểm thật. Hướng+2 trong phạm vi một solid đã kiểm chứng; full openNURBS vẫn đang hoàn thiện. Xem [báo cáo](docs/validation/2026-10-07-rhino5-ring-bounds.md).

Full regression trước đó: FreeCAD1901/1901; native30/30. Rust không đổi, giữ bằng chứng75/75. [Bằng chứng và giới hạn](docs/validation/2026-10-07-trim-domain-correspondence.md).

Chỉnh Hatch loop qua API cần FreeCAD có bản sửa `PropertyFileIncluded` để file của bản sao độc lập. Build chỉ OpenMatrix9 trên bản cài FreeCAD cũ chưa được xác nhận cho chức năng này; xem báo cáo kiểm chứng phía trên.

Giao diện trường số Hatch loop qua104 kiểm tra, thao tác hàng CV/knot/point/segment qua103; full regression1479/1479. Còn tạo/đổi kiểu native và rationality, surface/reference/plugin, pattern/render và Rhino5 thực tế. [Phạm vi và bằng chứng](docs/validation/2026-10-07-3dm-hatch-loop-rows.md).

CurveOnSurface giữ nguyên nguồn trong FCStd. 30 profile không có C3 đã qua Rhino5 thực tế và giữ đúng trường native khi nhập ngược; những loại con chưa kiểm chứng vẫn bị chặn. Bản sửa SDK chỉ thay hàm Read trong file build sinh ra, nguồn SDK gốc vẫn nguyên vẹn. Mất đối tượng có C3 là tương thích reader Rhino5 đã quan sát, không phải kết luận rằng định dạng V5 không chứa được C3.

### Rhino5 bounds follow-up

10/10 mở/lưu/mở lại; nhập ngược33/33. Kiểm chứng hình học bổ sung50/50 ở ngưỡng0,001 mm; bbox Rhino cũ49/50 được giữ riêng. [Báo cáo và giới hạn phép đo](docs/validation/2026-10-07-rhino5-ring-bounds.md). Full openNURBS vẫn đang hoàn thiện.

### Kiểm toán full openNURBS — 2026-10-07

Danh mục chuẩn hóa có131 khai báo nguồn,128 lớp runtime đã đối chiếu trong FreeCAD; phân loại abstract/helper/obsolete/concrete riêng. Năm dòng comment và ba lớp obsolete không được SDK build đã được ghi rõ. Báo cáo cũ128/123 giữ nguyên làm lịch sử, không còn là inventory đầy đủ. Không dùng số lớp hay số assertions để tính phần trăm hoàn thành. Regression hiện tại42/42 native; Rhino5 GUI35/35 profile, nhập ngược API/GUI qua484 kiểm tra FreeCAD. Bốn mẫu shell cong qua46 kiểm tra FreeCAD và29 kiểm tra nhập ngược file Rhino đã lưu; phép đo Rhino chính xác8/8 giữ ngưỡng1e-6 mm³, báo cáo mặc định1/4 giữ riêng; `Shape.Volume` mặc định còn sai số tích phân được ghi riêng, kiểm chứng dùng tích phân thích nghi với ngưỡng thể tích giữ nguyên. [Capability từng trục](docs/3dm-capabilities.json) · [Capability từng thuộc tính](docs/3dm-capability-slices.json) · [Audit hiện tại](docs/3dm-current-support-audit.json) · [Phạm vi đã kiểm chứng và phần còn thiếu](docs/validation/2026-10-07-opennurbs-packages-1-3.md).

## Kiểm chứng cập nhật 2026-10-08

6 mẫu UV lồng nhau đã qua API Rhino5 và nhập ngược đúng toàn bộ trường native,
UUID, dependencies và CRC. OM9 mở xuất V5 cho phạm vi Line trên UV NURBS 2D
bilinear không rational, hình chữ nhật trong miền; di chuyển, copy/xóa,
Undo/Redo và FCStd reopen đã kiểm chứng. UUID của đối tượng copy giữ ổn định
khi xuất lại; UUID trùng geometry/layer bị từ chối.

Regression mới: 42 suite native, 27 báo cáo FreeCAD với 2.131 kiểm tra và 11 test
Python đều đạt. Hai file nhẫn giữ 894 kiểm tra, ngưỡng bounds 0,001 mm và ngưỡng
area/volume cũ. Không dùng số kiểm tra để tính phần trăm hoàn thành.

12 file OM9 xuất giữ vị trí/di chuyển đã qua Open/SaveAs Rhino5 và nhập ngược
73 kiểm tra FreeCAD. 90 mẫu UV shear/rational/bậc cao đã qua API Rhino5 và nhập ngược361 kiểm tra;
OM9 V5/lifecycle qua900 kiểm tra. Open/SaveAs180 file OM9 xuất và nhập ngược1.081 kiểm tra đã đạt, không có bad objects. Còn UV lồng nhau tổng quát,
mapping/remap tham chiếu và các đợt còn lại. Full openNURBS chưa
hoàn thành. [Bằng chứng và giới hạn](docs/validation/2026-10-08-opennurbs-nested-uv-profile.md).

Checkpoint mới: 42 suite native, 29 báo cáo FreeCAD với 2.925 kiểm tra. [Bằng chứng UV mở rộng và phần còn thiếu](docs/validation/2026-10-08-opennurbs-expanded-nested-uv.md).

Checkpoint mở V5: 42 suite native, 30 báo cáo FreeCAD với 3.465 kiểm tra trên binary mới; [phạm vi UV và bằng chứng](docs/validation/2026-10-08-opennurbs-expanded-v5-admission.md).

Checkpoint GUI mở rộng: 42 suite native, 31 báo cáo FreeCAD với 4.546 kiểm tra. [Bằng chứng GUI180 và PolyEdge còn thiếu](docs/validation/2026-10-08-opennurbs-expanded-gui-polyedge.md).

[Hàng đợi test và tiến trình tới full openNURBS](docs/validation/opennurbs-test-roadmap.md): PolyEdge12 đã đo xong nhưng không đạt trong Rhino; OM9 giữ nguồn và chặn V5 không an toàn. Regression binary mới đạt44 suite native và33 báo cáo/4.731 kiểm tra;0 bộ đã chuẩn bị chờ chạy,7 đợt chưa đóng; tổng số bộ tương lai chưa xác định. Tổng số bộ còn lại chưa xác định. Cập nhật sau mỗi lần đo.

PolyEdge độc lập: Rhino đo xong12/12 nhưng0 đạt, curve trả UnsetPoint; SaveAs thu gọn6 lớp và mất6 cờ đảo chiều. OM9 giữ graph nguồn/owner trong FCStd, chặn V5 không an toàn:825 kiểm tra native và121 FreeCAD đạt. [Bằng chứng và phạm vi](docs/validation/2026-10-08-opennurbs-standalone-reference-support.md).

PolyCurve chứa PolyEdge con: Rhino6/6 không đạt, SaveAs mất class/metadata của6 curve con và3 cờ đảo chiều. OM9 giữ cây nguồn/owner trong FCStd và chặn xuất V5 lỗi;3 bản SaveAs đứt nối bị từ chối nhập trước khi sửa document.363 kiểm tra native +64 FreeCAD đạt. [Bằng chứng và giới hạn](docs/validation/2026-10-08-opennurbs-mixed-reference-support.md). Chưa full support.

UI menu reference follow-up2026-10-08: actual configuration `Resources/menu/MainMenu.ini` was already present; obsolete `ref/MainMenu.ini` routing corrected. Primary/dev decoded menu configuration matches18 groups/11 quick icons. Whole tools42/42 now passes; earlier41/42 logs preserved. No runtime menu/3DM/SDK change or Rhino rerun required. [Evidence](docs/validation/2026-10-08-ui-menu-reference-route.md).

Copy/remap checkpoint2026-10-08: namespace owner UUID bug and silent snapshot restoration fixed. Native36/36 cases (18 originals),2611 checks; FreeCAD18/18 lifecycle cases,612 checks. Final45/45 native;34 reports/5343 checks; tools42/42. Public V5 incompatibility remains; native owner geometry edits refuse safely. Packages2–8 open,total future batches unknown. Evidence: [docs/validation/2026-10-08-native-reference-remap.md](docs/validation/2026-10-08-native-reference-remap.md).

Seam preparation checkpoint2026-10-08:8/8 native(229 checks),8/8 FreeCAD(32 checks), actual Rhino5 pending. Current regression46/46 suites,35 reports/5375 checks; same verified production binary.1 prepared pending batch,7 packages(2–8) open,total future count unknown. Source/fixture bindings verified; no primary integration or push. [Evidence](docs/validation/2026-10-08-seam-profiles-preparation.md).

Seam actual checkpoint2026-10-08:Rhino5 GUI8/8 passed,no bad objects. Exact saved native decode200 and actual FreeCAD reimport49 checks passed. Final47/47 native;36 reports/5424 checks.0 prepared batches,7 packages2–8 open,total future count unknown. Scoped no-C3 seam/pole/periodic exchange verified; singular reference and general CAD editing remain open. SDK unchanged,no primary integration/push. [Evidence](docs/validation/2026-10-08-seam-profiles-complete.md).

UV-reference checkpoint2026-10-08:native8/8(895),actual FreeCAD8/8(272) passed. Current graph serialization/lifetime and child userdata copy-count retention corrected; source retained. Final48/48 native,37 reports/5696 checks,tools42/42.1 prepared8-file target batch awaiting actualRhino5;7 packages open,total future batch count unknown. PublicV5 still guarded; no full/primary integration/push claim. [Evidence](docs/validation/2026-10-08-uv-reference-remap.md).

Actual UV-reference8:Rhino read8,0passed/8invalid roots,8valid owners,8SaveAs command failures,0saved outputs;no saved reimport. Native507 source-retention/atomicV5-refusal checks passed. Scope declared incompatible;895 native remap and272 host checks retained. Final49/49 native,37 runtime reports/5696 checks,tools42/42.0 prepared batches,7 packages open,total future count unknown. Next seam/singular native applicability. No full/integration/push claim. [Evidence](docs/validation/2026-10-08-uv-reference-rhino5.md).

BRep seam/pole checkpoint2026-10-08:2/2 native(211),2/2 FreeCAD(65) passed including copy/edit/UndoRedo/source-deletedFCStd. Cylinder negative-proxy cap synchronization fixed with bounded complete-basis checks. Final50/50 native,38 reports/5761 checks.1 prepared4-file Rhino5 batch(2sources+2edited exports),7 packages open,total future batch count unknown;actual Rhino pending,full/integration/push incomplete. [Evidence](docs/validation/2026-10-08-seam-trim-preparation.md).

BRep seam/pole+edit actual checkpoint2026-10-08:RhinoGUI4/4 passed,0 bad objects;native decoded topology/commonUVbasis1800 +actual FreeCAD reimport45 pass. Rhino knot insertion verified by complete common basis,maxCVdifference4.44e-16;rawC2 bytes not identical. Final51native/39reports/5806checks,tools42/42.0prepared batches,7packages open,total future count unknown. Full/integration/push incomplete. [Evidence](docs/validation/2026-10-08-seam-trim-complete.md).


Menu 3DM cập nhật 2026-10-08: **OpenMatrix9 → Import 3DM... / Export Selected 3DM...**, và toolbar **3DM**. Mở/tạo project để nhập; chọn cả đối tượng để xuất V5. 18 kiểm tra UI FreeCAD thật và 75 Rust tests đạt; handler bảo toàn dữ liệu/guard giữ nguyên. [Hướng dẫn và bằng chứng](docs/validation/2026-10-08-3dm-exchange-menu.md).


Sửa crash menu Import/Export 3DM 2026-10-08: khóa GIL khi Qt gọi Python; file nhẫn oval mới nhập được 52 hình học hợp lệ qua QAction thực tế. Launcher mới dùng `om9-ui-3dm-gil-sdk`; cửa sổ cũ cần mở lại. Export file nhẫn mới/Rhino5 roundtrip chưa đo. [Kết quả và giới hạn](docs/validation/2026-10-08-native-menu-gil-fix.md).


Wireframe CAD 2026-10-08: OM9 vẽ cạnh CAD và isocurve theo mặt đã trim, giữ màu/density nguồn; bỏ đường tam giác tessellation trên BRep. 8 bộ FreeCAD thực tế/105 kiểm tra đạt, gồm nhẫn oval, viewport, block liên kết và menu Import/Export. Runtime thử riêng `build/om9-cad-wire-sdk`; giới hạn hiển thị và bằng chứng trong [báo cáo](docs/validation/2026-10-08-cad-wireframe.md). Chưa full openNURBS.

Working modeling Phase1 application acceptance2026-10-09: **validated within scoped independent working V5 exchange**. Import CAD point/curve/BRep/extrusion/native Mesh/cloud, edit/add geometry, current Selected export and source-independent FCStd verified on matching runtime.10 actual Rhino5 completed cases; real ring369 checks/29 roots+1 definition member;4 actual FreeCAD target-saved lifecycle reports230 checks, including Selected circle. Phase2–5 and full openNURBS remain open; no primary install/integration. [Evidence](docs/validation/2026-10-08-modeling-phase-1.md).

Phase1 clipboard/performance extension2026-10-09: native Rhino5 ↔ OM9 Copy/Paste through File menu, Command `Copy3dm`/`Paste3dm` and Ctrl+C/V in the viewport/model tree. Current selection remains editable; text controls retain text shortcuts. Native worker budget uses60% of available logical threads, reduced by tasks/RAM, with BelowNormal priority; CAD remains native and Mesh uses bulk binding. Run `H:/FreeCAD-src/build/om9-clipboard-sdk/bin/FreeCAD.exe`. Final acceptance, measured stage medians and the recorded Rhino5 scalar-probe limitation: [clipboard evidence](docs/validation/2026-10-09-modeling-phase-1-clipboard.md). This candidate has separate source/runtime bindings; the prior file-only acceptance stays scoped.

Phase1/2 performance2026-10-09: isolatedRustcache optimization +compactJSON, actualRhino gates passed; use H:/FreeCAD-src/build/launch-om9-perf.cmd. [Measured gains and limits](docs/validation/2026-10-09-phase12-performance-complete.md).0preparedpending;7fullopenNURBSpackagesopen,totalfutureunknown.

Rhino5 user verifier: run `_-RunPythonScript "H:\FreeCAD-src\build\om9-perf-dev\tests\rhino5_verify_phase12.py"`. [Instructions and scoped proof](docs/validation/2026-10-09-rhino5-phase12-user-script.md).

Rhino5 Phase1/2 verifier: clipboard fixture stabilization verified; rerun the same `rhino5_verify_phase12.py`. Native Paste is never retried by this test correction. See [updated evidence](docs/validation/2026-10-09-rhino5-phase12-user-script.md).

Rhino Phase1/2 verifier uses the shared system clipboard: wait for PASS/FAIL before Copy/Cut in another app. Latest user run0820a4cc was interrupted by Explorer file clipboard replacement; previous full proof retained. [Evidence](docs/validation/2026-10-09-rhino5-phase12-user-script.md).

Latest user Rhino5 Phase1/2 replay ca077635: PASS14fixtures,621Rhino/770OM9/175saved checks. Slow Copy performance remains under investigation. [Evidence](docs/validation/2026-10-09-rhino5-phase12-user-script.md).

Phase1/2 verifier now logs actual Copy/Paste stages and writes `timing-summary.json`; same Rhino entry command. Scoped ring/curve/mesh probe passed; full updated entry prepared. [Evidence](docs/validation/2026-10-09-phase12-copy-timing.md).

Rhino5 live timing reader corrected for IronPython str/bytes and stale module cache; actual caller-drain12/12PASS. Use new `tests/rhino5_verify_phase12_timing.py`; legacy filename repeatedly rewritten externally and left intact. User9974failure retained; full replay pending. [Evidence](docs/validation/2026-10-09-phase12-copy-timing.md).

Rhino5 live timing reader corrected for IronPython str/bytes and stale module cache; actual caller-drain12/12PASS. Use new `tests/rhino5_verify_phase12_timing.py`; legacy filename repeatedly rewritten externally and left intact. User9974failure retained; full replay pending. [Evidence](docs/validation/2026-10-09-phase12-copy-timing.md).


Full timing-entry replay 4f741966: PASS 14 fixtures / 621 Rhino / 38 OM9 reports / 770 OM9 / 175 saved reread checks. Logging: 198 events, 44 command totals, no errors. Ring OM9 Copy 16.146s (native encode/validate 14.925s); finer native and publication profiling remains next. Zero prepared replays for the reader correction; seven full-openNURBS packages open, future total batches unknown. See [timing evidence](docs/validation/2026-10-09-phase12-copy-timing.md).


Phase3 **implementation_verified=true / application_accepted=true (accepted_scoped)**: 18/18 requirement groups;15 fixture Rhino5 / 599 checks, 14 OM9 reports / 504 checks. Actual two-way clipboard11 fixtures /496 Rhino /218 host +120 saved reread checks on the same module. Rust152 tests; native10 suites; tools72 tests; format/strict Clippy PASS; one independent whole-change review resolved. Source/runtime isolated at om9-phase3-dev/sdk; no primary integration. Full openNURBS remains incomplete: seven broader packages open, zero prepared Phase3 batches pending, future total unknown. [Summary](docs/validation/modeling-phase-3/summary.json).

Phase3 user replay: PASS15 fixtures/599 Rhino/14 host reports504 checks on the accepted manifest/module; exact scripts, report and Rhino outputs verified. [User evidence](docs/validation/modeling-phase-3/user-verification.json).

Layer session handoff candidate is in progress. `Copy3dm` transfers Selected geometry with the complete palette, including an empty selection; `CopySession3dm` reads current working shape/mesh/cloud objects including locked and hidden objects. `ExportLayerSelection3dm` and `ExportSession3dm` provide the same file scopes. Paste validates the native geometry/metadata pair and rejects malformed present metadata. Scoped evidence: 253 Rust tests, 8 native suites and 250 shared-host checks across 11 reports. Retained block/archive Session provenance, remaining mutation guards and actual Matrix two-way application acceptance are pending. Actual direct Matrix read-only API checks now pass9/9 with frontend/core loaded and the document stamp unchanged. Run `_-RunPythonScript "H:\FreeCAD-src\build\om9-layer-edits\tests\rhino5_verify_layer_api.py"` in Matrix9. This entry does not launch another app or load .rhp. Command Undo/rollback and full two-way handoff remain pending. [Direct API evidence](docs/validation/layer-session/matrix-direct-api-fix.json). [Candidate evidence](docs/validation/layer-session/transfer-slice.json).

Native source-owner validation passes its recorded14 host/26 receive/8 native checks. Default `rhino5_verify_layer_command_api.py` now runs the actual two-app test in CURRENT blank Matrix and starts one owned OM9 companion on the shared FreeCAD host. It loads no Rhino plugin and touches no registry. Matrix native Copy/Paste/Undo/Redo and OM9 public clipboard/local Undo are compared in both directions, including complete palette, empty/nested layers, RGB, locks/visibility/active and current geometry. Whole local Undo/Redo and tracked blank-document cleanup are required for fixture PASS. Historical Matrix-only `.rhp` diagnostic is retired from this entry. Actual OM9 companion smoke8 passes and offline harness7/installed IronPython3/PS syntax pass; actual Matrix roundtrip remains pending. [Two-app preparation](docs/validation/layer-session/matrix-handoff-test-preparation.json). Full retained collection, product Matrix bridge and whole-plan acceptance remain incomplete.


Actual user two-app run303c failed: native Matrix clipboard omits unused layers/source palette, reverse Paste/count and owned cleanup unresolved. Corrected Rhino5 IronPython RGB JSON/report and guarded Idle; durable per-stage observations prepared. [Failure analysis](docs/validation/layer-session/matrix-handoff-failure-analysis.json). Read-only current Matrix evidence: `tests/rhino5_inspect_matrix_handoff.py`. The Matrix product Rust/C# transfer bridge and full two-way acceptance remain incomplete.

Rhino5 test correction: `IdefObjects=true` meant definition-only, so earlier zero-model/Paste/cleanup observations were invalid. Model iterator and CLR Byte JSON corrected; offline10 and installed IronPython5 prepare checks pass. [Iterator fix](docs/validation/layer-session/matrix-model-iterator-fix.json). Dedicated failed303c fixture recovery is `tests/rhino5_recover_matrix_handoff.py`, guarded against foreign/changed documents. Native replay and product full-palette handoff remain unaccepted.

Matrix Rust full-palette adapter preparation: [evidence](docs/validation/layer-session/matrix-rust-palette-preparation.json). Current Matrix test `tests/rhino5_verify_matrix_om9_palette.py` uses shared Rust policy and native RhinoCommon API through the existing command owner, with actual OM9 clipboard, two failure injections and repeated local Undo/Redo. No Rhino launch/RHP loading. Prepared adapter is not packaged commands or native acceptance; original layer-session Tasks1-5 remain incomplete. Recovery30c4 removed five test objects/30 layers and preserved the observed34-layer palette.

Layer-session dc77 native gate: Native dc77 run: Matrix->OM9 full palette/geometry/locks/active and actual local Undo/Redo PASS. Reverse receive failed at injected after-layer rollback and cleanup: Rhino5 SetUserString uses ConstPointer while CommitChanges refuses a document-controlled wrapper. Native helper now uses EnsurePrivateCopy + LayerTable.Modify; behavior-model RED reproduced exact failure, GREEN5 + Rust9/ABI/harness10/recovery-ownership5/installed-IronPython4 and SDK build PASS. Actual reverse rollback, repeated Undo/Redo and dc77 recovery remain pending; full handoff/UI packaging not accepted. [Evidence](docs/validation/layer-session/matrix-witness-fix.json).

Actual 01fe run:20 checks,16 PASS; geometry/full palette/locks/active pass BOTH directions, both injected rollback stages pass, cleanup/owned companion finish pass. Only four Matrix Undo/Redo equality checks fail. Original run lacks snapshots at those steps, so exact cause remains unproven. Prepared explicit native cursor readiness/error/10s gate (old one-yield RED3 -> GREEN3), native queue/projection trace, four Undo/Redo snapshots and field differences; full raw comparisons unchanged. Actual replay pending, application acceptance false. [Evidence](docs/validation/layer-session/matrix-undo-gate-preparation.json).

User 2026-10-10 narrowed Undo/Redo to OM9 only; Matrix transfer remains required. Removed custom Matrix Undo callbacks/test commands and active-cursor code; bootstrap detaches only old owned diagnostic hooks. Historical117f report is preserved FAIL under its original broader scope; explicit reassessment passes all16 now-required native checks (both transfers, OM9 Undo/Redo, two rollback faults, cleanup). New transfer-only adapter SDK build/Rust9/ABI/harness12/witness5/recovery5/IronPython4 pass; fresh native candidate replay and OM9 UI deployment remain pending. [Scope evidence](docs/validation/layer-session/om9-only-undo-scope.json).

Native7c9 run: all16 required application checks PASS on exact current managed/Rust/OM9 candidate runtime. Both transfer directions preserve full palette, locks/visibility/current and geometry; actual OM9 Undo/Redo, two rollback injections and cleanup PASS. Original false terminal FAIL is preserved; separate handoff-verification.json confirms scoped acceptance using current evaluator. Cached old criteria reproduced under installed Rhino5 IronPython; fresh module loader resolves summary-only bug and rejects real OM9 Undo failure. No new native run, no full Tasks1-5 closure, no primary UI deployment claim. [Scoped native acceptance](docs/validation/layer-session/palette-native-acceptance.json).
