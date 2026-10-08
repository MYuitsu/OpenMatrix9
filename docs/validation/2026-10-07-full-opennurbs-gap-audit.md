# Kiểm toán phần còn thiếu của full openNURBS — 2026-10-07

Ghi chú cập nhật: phần dưới giữ nguyên snapshot kiểm toán ban đầu. Danh mục
nguồn/runtime đã chuẩn hóa và các giới hạn multi-shell/mapping đã thay đổi;
xem [báo cáo hiện tại](2026-10-07-opennurbs-packages-1-3.md) và
[audit hiện tại](../3dm-current-support-audit.json). Không dùng các số123/128
hoặc line anchor lịch sử dưới đây làm inventory/source hiện tại.

**Kết luận: chưa đủ bằng chứng để gọi là full support.** Đã có nhiều chức năng
trao đổi 3DM được kiểm chứng, nhưng còn thiếu adapter, ngữ nghĩa tham chiếu,
tương thích phiên bản và kiểm chứng theo từng loại dữ liệu. Đây là kiểm toán
code và bằng chứng hiện có; không phải một lượt chạy mới toàn bộ CAD tests.

Phạm vi kiểm tra: source `H:/FreeCAD-src/build/om9-dev`; openNURBS pin
`eb92af3ba1806b0a34a99aba0d3bda83e3d46083`; xuất công khai nhắm Rhino5/V5.
Hợp đồng đã duyệt: `docs/superpowers/specs/2026-10-06-full-opennurbs-design.md`.
Dữ liệu kiểm toán: `docs/3dm-full-support-audit.json`; chạy lại bằng
`python tools/audit_opennurbs_support.py --sdk-root H:/FreeCAD-src/build/dependencies/opennurbs`.

## Điều kiện gọi là đầy đủ

Mỗi loại dữ liệu áp dụng trong revision đã chốt phải có đường đọc, bảo toàn,
hiển thị/chỉnh sửa khi được quảng bá và xuất đúng ngữ nghĩa, kèm test. Nội dung
FreeCAD không chỉnh được có thể giữ nguồn rõ ràng. Nội dung thực sự không biểu
diễn được trong Rhino5 cần test chứng minh bất tương thích và từ chối trước khi
ghi đè. Từ chối vì adapter chưa được implement vẫn là việc chưa hoàn thành.

Không được âm thầm bỏ dữ liệu, dùng mesh thay CAD hoặc báo nguồn giữ nguyên là
CAD chỉnh sửa được. Cần xác minh UUID, references, components, nội dung và bytes
tài nguyên; hash toàn file nguồn chỉ chứng minh snapshot, không chứng minh
ngữ nghĩa của file đã xuất. Lớp abstract/helper hoặc không có serialization
phải được phân loại đúng, không ép mọi lớp thành đối tượng chỉnh sửa độc lập.

openNURBS tập trung vào đọc/ghi 3DM và một số công cụ hình học. Full exchange
không yêu cầu tái tạo mọi lệnh của Rhino SDK, chạy plugin độc quyền hay tính lại
history Matrix. Giữ payload/history và xử lý references an toàn vẫn thuộc
phạm vi trao đổi. [Giới hạn do McNeel công bố](https://developer.rhino3d.com/guides/opennurbs/what-is-opennurbs/).

## Danh mục bao phủ cần sửa trước khi thống kê tiến độ

- Bảng cũ có128 tên lớp:98 concrete,22 obsolete compatibility,8 abstract.
- Quét lại source, loại comment và giữ chuỗi C++: **123 khai báo macro đăng ký
  không nằm trong comment**, gồm93 concrete,22 obsolete compatibility,8 abstract
  theo phân loại hiện có. Đây là discovery tĩnh, chưa chứng minh mọi nhánh build.
- Năm dòng bị tính nhầm: `ON_Font`, `ON_Font_Windows_LFTM_UserData`,
  `ON_MeshVertexRef`, `ON_MeshEdgeRef`, `ON_MeshFaceRef`. Chúng có macro bị comment.
  Cần phân loại applicability/legacy/helper; không tự động coi là năm adapter thiếu.
- Đủ16 tên nhóm component và6 nhóm document trong danh mục hiện tại. Bằng chứng
  field-by-field và tương tác chưa phủ đầy đủ các nhóm này.
-22 lớp có `tested_slices`; các entry này đều không tuyên bố hoàn thành toàn lớp.
  Không được tính22/123 thành phần trăm hoàn thành. Các test CAD cơ bản còn có
  bằng chứng ngoài `tested_slices`, nên cũng không được coi các lớp khác là chưa
  có code chỉ vì không có entry này.
- Trạng thái tổng `display/editable/rhino5_write` còn ghi unverified ở toàn bộ
  bảng cũ; một số `validation_scope` giữ giới hạn cũ dù đã có slice mới, ví dụ
  TextDot payload editing, Hatch UI, CurveOnSurface transforms. Cần đồng bộ theo
  phạm vi, không nâng thành validated toàn lớp chỉ từ một slice.

Snapshot `docs/3dm-coverage.json` được giữ nguyên để không phá hash của các báo
cáo lịch sử. JSON audit mới tách riêng row comment, trạng thái cũ và slice evidence.

## Các nhóm còn thiếu

| Nhóm | Đã có trong phạm vi đã test | Còn phải làm hoặc kiểm chứng | Điều kiện nghiệm thu |
|---|---|---|---|
| CurveOnSurface / PolyEdge / references | Đọc cây native, một số biến đổi coupled, graph có owner/lifetime, edge/trim domains và đầu cuối | Mapping nội miền phi tuyến, UV, reversal, seam/singular; remap khi copy/edit; current host edit/display/selected export | Native fixtures và Rhino5 cho từng reference/type; kiểm tra toàn đường cong/trim, không chỉ hai endpoint; edit/copy/FCStd không dùng payload cũ |
| BRep / solids | Các mặt trim, NURBS, khối một shell, block phản chiếu và xử lý+2 chuyển được sang một solid | Multi-shell/cavities, topology/seam/singular tổng quát; phân loại+2 chưa chuyển được; bảo toàn native khối hướng vào trong qua Rhino5 | Mẫu nhiều shell/khoang, orientation/parity, volume dấu, native graph/UUID và Rhino5 roundtrip; refusal phải phân biệt lỗi nguồn với adapter thiếu |
| Hình học và annotation còn lại | CAD cơ bản, Mesh, một số TextDot/PointCloud/Hatch | Adapter và test đầy đủ cho SubD/component refs, NurbsCage/MorphControl, PointGrid, OffsetSurface/SurfaceProxy; text/leader/dimensions/centermark, clipping/page/detail | Mỗi type có native fixture, placement/units, nội dung và dependency; cái không thuộc V5 phải có compatibility rejection được kiểm chứng |
| Hatch / TextDot / PointCloud | Native fields, boundary/point/text previews, loop editing có phạm vi, copy/block/FCStd; chặn gradient/intensity mất dữ liệu | Full Hatch pattern/fill rendering và pattern-line editing; tạo/đổi đầy đủ class/rationality/child; độ trung thực TextDot; dữ liệu cloud lớn/streaming/locking và legacy upgrades còn lại | Rhino5 rendering+SaveAs theo từng loại; so sánh nội dung/field/bytes, mm/cm, copy và source-deleted FCStd; không bỏ gradient/intensity âm thầm |
| Appearance và resources | Giữ snapshot, inventory một số dependencies, RDK3 UTF-8 không có tài nguyên nhúng | Object rendering/material/mapping, images/textures, embedded files, render content/lights, linked resources và các reference chains | Selected dependency closure; texture bytes/mapping đúng; FCStd không phụ thuộc đường dẫn nguồn; missing resource được báo rõ; kiểm chứng native và Rhino5 |
| Block liên kết và gộp nhiều nguồn | Embedded/nested/current/copied/shared graph, namespaces, affine và stable identities trong phạm vi đã test | Linked/LinkedAndEmbedded resource semantics, nguồn ngoài không embedded; metadata policy cho các archive khác nhau; remap references tổng quát | Copy/merge archives khác nhau, UUID collision, nguồn bị xóa/di chuyển, unit changes và target closure; không xuất geometry thừa |
| Components / document | Inventory và giữ nguồn, một số lớp/layer/group/style references | Đủ mọi field16 nhóm; properties/settings/views/named views/layout/cplanes; đổi đơn vị view và policy gộp tài liệu | Semantic equality theo category, units và selected/full-project semantics; source unavailable FCStd; views/styles/resources vẫn tham chiếu đúng |
| Userdata / history / plugin payload | Giữ archive bất biến, user strings và các graph an toàn đã được test; guard unknown references | Closure/remap/version cho opaque userdata, dictionaries, history và plugin records an toàn; invalidate history khi sửa | Payload/decoded fields và references đúng sau edit/copy/merge; incompatibility/unknown semantics từ chối atomic; không yêu cầu chạy plugin độc quyền |
| Phiên bản và nghiệm thu bao phủ | Native/independent SDK2013 cho các slice; Rhino5 thật10 file và reimport8 exports; đo bổ sung nhẫn50/50 | Ma trận type×field×source version×target version, aliases/obsolete classes, interaction fixtures, actual Rhino5 cho các nhóm còn lại | Không còn row áp dụng chỉ unverified; mỗi row có native+host+target evidence hoặc compatibility refusal hợp lệ; không lấy số assertions làm độ phủ |
| Tích hợp và phát hành | Runtime riêng đã test ở `build/3dm-preservation-sdk` | Primary checkout chưa đồng nhất; kiểm thử bản cài/prefix thông thường và đóng gói các FreeCAD patches cần thiết | Đồng bộ source đã review, build/install từ primary, chạy lại acceptance trên binary cuối, archive evidence và phiên bản build rõ ràng |

## Bằng chứng code của các giới hạn thật

- `Gui/ThreeDmBrep.cpp:65`: solid nhập phải tạo được một shell liên thông.
- `Gui/ThreeDmArchive.cpp:27`: bảo toàn native hướng vào trong bị từ chối;
  Geometry only là đường khác, không chứng minh nguyên bản graph được giữ.
- `Gui/ThreeDmArchive.cpp:96`: external-only block cần geometry embedded.
- `Gui/ThreeDmArchive.cpp:131`: geometry-only chưa chuyển annotation/plugin objects.
- `Gui/ThreeDmMerge.cpp:92,97,503`: thiếu RDK content/material, rendering/mapping dependencies.
- `Gui/ThreeDmMerge.cpp:122`: archive khác SHA cần document merge policy.
- `Gui/ThreeDmMerge.cpp:217,222,228,283`: embedded RDK4/V5, view unit normalization,
  retained type normalization và copied linked resource guards.
- `Gui/ThreeDmMerge.cpp:81`: unknown opaque userdata dependency bị từ chối.
- `Gui/ThreeDmNativeReferences.cpp:51`: `interior_mapping=unverified`.

Audit JSON giữ line anchors và SHA source. Guard là cơ chế bảo vệ dữ liệu;
việc có guard không phải bằng chứng rằng input tương ứng đã được hỗ trợ.

## Bằng chứng nhẫn và integration

Nhẫn vẫn có bằng chứng native35/35, GUI1910/1910, fixtures894/894, actual Rhino5
10 lifecycle và8 saved-export reimports33/33. Lượt đo bổ sung50/50 giữ nguyên
ngưỡng0.001 mm; raw bbox49/50 vẫn riêng. Đây là dữ liệu test lịch sử đã kiểm tra
ở slice trước, không phải native/GUI run mới trong kiểm toán này. Nó không bao
phủ annotation, render resources, SubD, toàn bộ16 component/6 document categories.

So sánh trực tiếp5 file đại diện với `H:/FreeCAD-src/Mod/OpenMatrix9`: ba file
`ThreeDm.py`, `Gui/ThreeDmArchive.cpp`, `Gui/ThreeDmBrep.cpp` khác SHA; primary
chưa có `Gui/ThreeDmMerge.cpp` và `docs/3dm-coverage.json`. Vì vậy không được gắn
bằng chứng runtime riêng cho source/binary public hoặc bản FreeCAD cài sẵn.

## Thứ tự ưu tiên tiếp tục

1. Chuẩn hóa inventory, applicability và ma trận bằng chứng type/field/version.
2. Hoàn thiện CurveOnSurface/PolyEdge interior mapping, host edit/display/export;
   bổ sung BRep multi-shell/cavity và preservation orientation fixtures.
3. Annotation/style và các geometry còn thiếu, bắt đầu từ loại người dùng có file thật.
4. Resources/material/mapping/linked blocks, components/document merge và userdata/history.
5. Ma trận actual Rhino5, review/integration, final installed-runtime acceptance.

Kết thúc audit không đồng nghĩa kết thúc toàn bộ các hạng mục này. Kết luận
hiện tại là **hỗ trợ nhiều slice 3DM đã kiểm chứng; full exchange vẫn in_progress**.
