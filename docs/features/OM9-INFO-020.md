# OM9-INFO-020 — Gumball

Ngày 2026-10-11; trạng thái `partially_implemented`, native `validated_supported_slice`.
Command `Gumball`, `OM9_Gumball`; legacy `OM9_InfoSettingsGumballON` dùng cùng
Rust handler. [Spec](../../OpenMatrix9_Codex_Spec_v1/specs/01-core/om9-info-020-gumball.md).

Nguồn: Matrix 8 Book 1, PDF 45–47 / trang in 35–37; trang PDF 48 bắt đầu History.
Đã xem trang và continuation. Manual xác nhận move axes/planes/origin,
rotate arcs, scale squares, numeric, Alt toggle Copy và Ctrl giữ tới khi nhả
chuột để extrude. Không lưu artwork/manual riêng trong source public.

Quyết định OM9 hiện hành: default Off/CPlane, X đỏ/Y xanh lá/Z xanh dương.
Ngày 2026-10-11 user duyệt chỉnh giao diện theo Rhino 5, thay Z vàng của
thiết kế lõi trước đó: arrows và quarter arcs thanh, outline Scale squares
phía đối diện arrows, dashed guides không pickable, menu ball cạnh tâm,
numeric editor gọn và nằm trong viewport. Màu Y manual là xanh lam/cyan;
đây là lựa chọn OM9 theo Rhino, không phải bằng chứng visual parity Matrix.
Shift scale cả ba chiều theo Rhino 5. Safe Rust giữ session/frame/ray math,
input hữu hạn, length/angle/factor, copy và operation plan. C++ chỉ giữ
Qt/Coin/FreeCAD/OCCT/Mesh/Points và transaction native.

Selection editable cùng document; parent+child chuẩn hóa thành một root,
group có private geometry của members, App::Link cùng document giữ shared
definition. Move/Rotate dùng Placement nên giữ native parametric owner.
Scale affine chỉ plain Part::Feature/Mesh::Feature/Points::Feature, hệ số
khác 0, cho phép âm; circle nonuniform có thể thành rational BSpline exact.
Extrude arrows hỗ trợ planar Edge/Wire/Face, kể cả Shell/Compound một child
và face có hole. Không thay geometry nguồn khi Copy/Extrude.

13 tay nắm transform và menu ball riêng là native picked scene paths;
menu mở Relocate/Reset và CPlane/Object/World bằng các handler hiện hữu,
không mở geometry transaction. Reset recapture tâm selection theo alignment
hiện hành. Popup nhận mouse release không nuốt click tay nắm kế tiếp.
Preview bounds-excluded/unpickable,
không thêm document object/Undo. Click mở numeric editor; Enter khi đang
drag chỉ ràng buộc distance/bước góc, release mới commit. Distance khi giữ drag
theo phía con trỏ so với điểm press; số âm đảo chiều
đó. Đúng tại điểm press giữ chiều cuối; numeric click vẫn dùng dấu theo trục.
Move và Extrude đều kiểm kéo qua hai phía sau Enter bằng Qt thật.
Alt tap toggle, auto-repeat bỏ qua, Alt numeric tạo Copy có UUID riêng. Ctrl có từ mouse-down
chỉ chỉnh widget; Ctrl thêm sau motion và còn giữ ở release mới Extrude.
Esc đầu đóng numeric editor, Esc tiếp hủy gesture. Navigation/foreign editor,
source delete/replace/permission/unit/CPlane/document/view changes dọn phiên.

Một thao tác geometry có một transaction Undo/Redo. Ghi report JSON Rust,
input raw/unit/frame/source stamps và affine; giữ material/layer/parent/retained
3DM metadata. Copy group có members riêng; Link Copy là occurrence riêng cùng
definition. Không tạo associative History graph mới; existing native parametric
dependencies vẫn recompute. Provenance/successful geometry lưu trong FCStd.

Giới hạn rõ trước transaction: Scale parametric/Link, external-document Link,
unresolved geometry/capability. Không có fallback mesh. Callback synchronous
sửa nguồn trong publication abort cả batch. Mesh/Points được so sánh với
staged output sau setter callbacks, trước khi ghi nhớ expected state.
**FreeCAD xóa Redo khi mở
transaction; abort sau callback không phục hồi Redo cũ.** Model/Undo được rollback;
input/capability rejection trước transaction giữ Redo. Chưa xác nhận mọi owner,
mọi native application/IME và parity Matrix/Rhino cho mọi representation.

Tolerance: typed distance double kiểm 1e-10 mm; box dimensions/volume 1e-8;
ellipse optimal bounds có native edge tolerance 1e-7 mỗi phía, kiểm 1e-6 mm.
Không nới tolerance của solver để hợp thức kết quả.

[Evidence và lệnh tái kiểm](../validation/2026-10-11-gumball-core.md).
[Giao diện Rhino và kiểm tra native](../validation/2026-10-11-gumball-rhino-visual.md).
Tiếp theo: native owner adapters cho scale parametric/Link, multi-document
atomic publication và host Redo-preserving transaction API.
