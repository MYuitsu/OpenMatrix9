---
id: RCORE-12
title: Display, render và vòng đời tài nguyên
priority: P1/P2
review_date: '2026-10-09'
evidence_level: source_inspected
runtime_validation: not_run
freecad_commit: 21d36cfa1eb110a1d0667050ff31706298805bbd
openmatrix9_commit: 52e887ab706dcd5d80e23979fb1f1b62d8d869b0
---

# RCORE-12 — Display, render và vòng đời tài nguyên

## Phạm vi và mức bằng chứng

Chi tiết hóa [nhóm nền Rhino](README.md).
Liên quan `OM9-DISPLAY`, `OM9-RENDER`, `OM9-PICTURE`, các feature nhóm Render và workspace views.
Phạm vi gồm view state, tessellation/cache, appearance, scene resources, output và worker lifecycle.
FreeCAD source đã có Wireframe/Shaded, view providers, materials, camera và ảnh chụp viewport.
Những cơ chế đó chưa chứng minh pipeline renderer Rhino, shader/plugin hoặc scene editor đầy đủ.
Source rà ngày 2026-10-09 ở working tree có thay đổi; không chạy GPU/native/render tests.
Mọi fixture dưới đây là backlog, không phải kết quả nghiệm thu của binary hiện tại.

Rhino 5 User's Guide PDF 101–104 / trang in 93–96 nêu workflow lights, materials,
environment, ground plane rồi render/save; xem [audit nguồn](../../../docs/reviews/2026-10-09-rhino-core-gap-audit.md).
Mục tiêu compatibility cần scene resources tương ứng, không chỉ ảnh shaded có màu.
EMap đã có [spec OM9-RENDER-018](../13-render/om9-render-018-environment-map.md).
Không dùng EMap, viewport capture hoặc tên Rendered trong menu làm bằng chứng renderer production.
V-Ray, Clayoo và T-Splines có capability riêng; sự xuất hiện tên không cam kết có plugin runtime.

## Hợp đồng state và representation

Input display là document/object revision, viewer identity, mode và object/layer visibility policy.
Output display là scene representation tạm; topology/geometry CAD gốc không được thay đổi.
Tách per-object display mode, per-viewport override, global preferences và preview override.
Wireframe/Shaded/Rendered/Analysis có state riêng và supported options riêng.
Một mode chưa có backend phải disabled với lý do; không tự rơi về shaded rồi báo rendered.
Selection/pick vẫn resolve identity/subelement thật, kể cả khi các view dùng mode khác nhau.
Preview có ownership session; không tham gia save/export/selection như geometry đã commit.
Đổi display density chỉ thay tessellation, không sửa Shape, units hoặc tolerance modeling.

| Dữ liệu | Owner và lifetime mục tiêu |
|---|---|
| Geometry/topology chuẩn | App document/native kernel; lifetime theo object generation. |
| Display mesh/edge buffers | Cache service theo geometry revision và tessellation settings. |
| Viewer state | Viewer identity; camera/projection/override riêng cho từng viewport. |
| Material assignment | Persistent object/layer/material IDs; scene adapter resolve inheritance. |
| Lights/environment/ground | Persistent scene records, backend capability và units rõ. |
| Texture/reference image | Resource ID/hash/path policy; decoded buffer có budget và owner. |
| Preview/cache worker | Immutable request snapshot + cancel token; kết quả kiểm stale trước publish. |
| Render output | Job ID, scene revision, dimensions/alpha/color settings và path result. |

Cache key cần geometry identity/generation/revision, display settings và backend version.
Shape đổi hoặc object được thay thế phải invalidate mọi dependent mesh/edge/analysis buffers.
Object ẩn có thể trì hoãn tạo mesh nhưng khi hiện lại phải kiểm revision, không hiện mesh cũ.
Undo/Redo cần invalidate theo shape thực; không chỉ dựa vào tên object hoặc pointer trùng địa chỉ.
Chuyển workbench/đóng viewport cần trả override và tháo callback/scene nodes đúng owner.
Tessellation của CAD không chuyển classification thành mesh-only; export/copy dùng representation thật.

## Material, ánh sáng, môi trường và resources

Material schema tách display color/transparency khỏi shader renderer và texture/bump parameters.
Assignment object/layer/parent có thứ tự kế thừa explicit, không biến per-face colors thành layer style.
Color space, opacity/alpha và texture channel phải được ghi; không hứa ảnh giống nhau giữa backend.
Light record cần loại, transform, color, intensity và đơn vị; backend không hỗ trợ phải báo rõ.
Headlight viewport không thay toàn bộ light objects cần lưu và sửa của scene Rhino.
Environment background, reflection/lighting environment và analysis EMap là roles khác nhau.
Ground plane render là scene setting với elevation/material; không tự tạo mặt CAD vô hạn.
Camera lưu projection, placement, lens/focus hoặc view height theo backend, không suy từ CPlane.
Reference image có placement và kích thước model; render texture có mapping khác.
Resource path cần relative/absolute/embedded policy, hash, relink và missing-resource diagnostic.
Không âm thầm lấy texture trùng tên từ thư mục khác hoặc giữ bản decoded cũ sau relink.
FCStd lưu material/resource metadata nào phải được công bố theo field; 3DM retention xét riêng.

## Render/export và lifecycle

Input render đóng băng scene/camera revision, backend, pixel dimensions, alpha và color policy.
Resolution phải positive finite/integer trong budget; kiểm multiplication overflow trước allocation.
Output gồm image/file, scene hash và diagnostic backend; file path không phải chứng cứ scene đúng.
Không đưa print scale vào pixel resolution; layout image dùng hợp đồng [RCORE-11](11-annotation-layout-print.md).
Failure gồm backend unavailable, shader/light unsupported, texture missing, GPU/RAM và write error.
Capability UI phải phản ánh backend chưa sẵn sàng trước job; lỗi giữa job không đổi thành success.
Cancel có bounded progress/cancel points theo backend; phần không ngắt được cần công bố rõ.
Đóng document/viewport hủy hoặc detach job theo policy; kết quả muộn không truy cập Qt/App đã chết.
Worker chỉ nhận snapshots hoặc kernel copy có cơ sở thread safety; không mutate document/Qt.
Publish scene buffers/commit metadata chạy trên host thread sau kiểm identity/generation/revision.
Budget dùng chung cho preview, mesh, import và render để tránh mỗi subsystem chiếm toàn tài nguyên.
Undo áp cho material/resource/scene edits được lưu; render job và navigation không tự sinh geometry Undo.
Lỗi xuất ghi ra file tạm và cleanup; không thay file đích tốt bằng output dang dở.

## Chi tiết bổ sung từ code decompile

Các chi tiết dưới đây được đọc từ code decompile; chưa chạy binary, UI hoặc fixture runtime.
Phân biệt logic managed, wrapper gọi native và phần triển khai native khi xác định phạm vi đã hiểu.
Các phát hiện này không nâng trạng thái triển khai hoặc đóng nghiệm thu FreeCAD/OM9.

| Hành vi và hệ quả hợp đồng | Giới hạn |
|---|---|
| ObjectAttributes có display override theo viewport ID hoặc toàn bộ viewports qua overload dùng Guid.Empty; remove theo scope tương ứng. RhinoViewport.DisplayMode là state của viewport khác với object override. UI OM9 phải biểu diễn rõ scope và thao tác trở về mode viewport. | Semantics scope được mô tả tại wrapper; lựa chọn precedence khi đồng thời có nhiều override và draw path native chưa được runtime xác minh. |
| RenderContent.Create nhận type ID, document và optional parent/child slot, trả null khi native không tạo được; API ghi rõ đây là content persistent do RDK sở hữu. LoadFromFile tạo content chưa persistent với AutoDelete=true; AddPersistentRenderContent đổi ownership flag và gọi native. Import thư viện, gắn scene và render phải là lifecycle riêng. | Ownership persistent dựa trên API contract; AutoDelete bị đổi trước bool kết quả AddPersistent nên adapter OM9 phải xử lý failure riêng. Không có chứng cứ renderer/shader tương đương. |
| Bridge RDK có thân hàm riêng cho ground-plane enable, altitude và material instance ID, chuyển tiếp vào CRhRdkGroundPlane. Đây là scene resource state độc lập; không thấy các hàm này thêm mặt CAD vào object table. | Bridge gọi RDK bên ngoài; không chứng minh shader, ánh sáng, kết quả ảnh, default UI hoặc cách mọi backend biểu diễn ground plane. |
| Matrix Layout RhinoUtils chọn đường render/save render-window khi bVrayOverride, còn capture thông thường dùng ViewCaptureToFile hoặc CaptureToBitmap. VRayRenderForm khởi phát render từ timer rồi hỏi HasRenderFinished ở tick sau. Gửi lệnh, capture viewport và hoàn tất job renderer là ba mốc bằng chứng khác nhau. | Callers phụ thuộc Rhino/VRayInterface runtime bên ngoài; call sequence không chứng minh file đã ghi, alpha đúng, output thành công hay hủy job an toàn. |

Các yêu cầu về schema state/identity, worker, transaction và xử lý lỗi của OM9 vẫn là quyết định
thiết kế, trừ hành vi gốc được nêu rõ ở trên. Không suy defaults toàn ứng dụng từ một caller.

## FreeCAD đã có đến đâu; OM9 cần bổ sung gì

Rà giới hạn ở Gui/Part view providers và OM9 source liên quan; không kết luận mọi renderer add-on.

| Capability | Yêu cầu | FreeCAD | Bằng chứng/phạm vi | OM9 riêng |
|---|---|---|---|---|
| RCORE-12.C01 | Wireframe/Shaded/Flat Lines/Points | UI+API | [ViewProviderExt.cpp](../../../../../src/Mod/Part/Gui/ViewProviderExt.cpp): `setDisplayMode/getDisplayModes`; [CommandView.cpp](../../../../../src/Gui/CommandView.cpp): `StdCmdDrawStyle`. | [CoreWorkspace.cpp](../../../Gui/CoreWorkspace.cpp): `displayTitle/reconcileDisplay` có wire/shaded integration. |
| RCORE-12.C02 | Tessellation density | API | `Deviation`, `AngularDeflection`, `updateVisual` trong [ViewProviderExt.cpp](../../../../../src/Mod/Part/Gui/ViewProviderExt.cpp); chưa rà UI editor của hai thuộc tính này. | Cần capability/tolerance tách modeling; chưa chạy invariant geometry. |
| RCORE-12.C03 | Invalidation khi shape đổi | API | `updateData` kiểm Shape/Touched; object ẩn dùng `VisualTouched`. | Nền có sẵn; cần generation/cache keys cho async OM9 và Undo/Redo. |
| RCORE-12.C04 | Color/material/transparency display | UI+API | [ViewProviderGeometryObject.cpp](../../../../../src/Gui/ViewProviderGeometryObject.cpp): appearance/SoMaterial; [PropertyItem.cpp](../../../../../src/Gui/PropertyEditor/PropertyItem.cpp): `PropertyMaterialItem`; [TaskAppearance.cpp](../../../../../src/Gui/TaskView/TaskAppearance.cpp): transparency UI. | Chưa chứng minh mapping mọi Rhino material/shader/texture. |
| RCORE-12.C05 | Camera và viewport lighting | API | [View3DInventorViewer.cpp](../../../../../src/Gui/View3DInventorViewer.cpp): `setCamera`, `setCameraType`, headlight. | Cần scene light persistence riêng; headlight không là đủ light workflow. |
| RCORE-12.C06 | Camera/GUI persistence | API | [Gui Document.cpp](../../../../../src/Gui/Document.cpp): `Save/Restore`, camera settings. | Per-view OM9 mode/frame persistence cần fixture; không suy từ camera host duy nhất. |
| RCORE-12.C07 | Viewport image export | UI+API | [CommandView.cpp](../../../../../src/Gui/CommandView.cpp): saveImage command; [View3DInventorViewer.cpp](../../../../../src/Gui/View3DInventorViewer.cpp): `savePicture`. | Là capture path; chưa phải renderer production tương thích Rhino. |
| RCORE-12.C08 | Reference image/PictureFrame | Một phần | Native ImagePlane được OM9 adapter gọi. | [CorePictureFrame.cpp](../../../Gui/CorePictureFrame.cpp) + [core_picture_frame.rs](../../../rust/src/core_picture_frame.rs) có placement/preview/transaction; missing/relink policy cần mở rộng. |
| RCORE-12.C09 | Environment/ground plane/light editor parity | Chưa kiểm chứng | Các file Gui/Part vừa rà chưa chứng minh pipeline Rhino scene tương đương. | Cần backend mapping riêng và field-level support table, không suy từ RenderLight retained. |
| RCORE-12.C10 | EMap/analysis display | Một phần | [TextureMapping.cpp](../../../../../src/Gui/TextureMapping.cpp): `SoTextureCoordinateEnvironment`, `onCheckEnvToggled`; [CommandView.cpp](../../../../../src/Gui/CommandView.cpp): `Std_TextureMapping`. | OM9-RENDER-018 có spec; host có environment mapping, parity EMap và continuity fixtures chưa kiểm. |
| RCORE-12.C11 | V-Ray/Clayoo/T-Splines runtime | Chưa kiểm chứng | Không audit add-on/plugin ngoài phạm vi source đã chọn. | Menu/catalog không chứng minh installation, license, ABI hoặc supported rendering. |
| RCORE-12.C12 | Async render budget/cancel/cleanup | Chưa kiểm chứng | Lifecycle hooks/view refs là nền; chưa kiểm toàn pipeline job. | Cần Rust scheduler và stale-result tests xuyên preview/mesh/render/import. |

## Đọc source và giới hạn kết luận

`ViewProviderPartExt::updateData` tính visual ngay khi visible, đánh `VisualTouched` nếu đang ẩn.
Tessellation dùng OCCT triangulation/edge polygon để dựng buffers; đây không phải shape gốc mới.
`ViewProviderGeometryObject` đồng bộ appearance sang SoMaterial; shader equivalence chưa kiểm.
`savePicture` có offscreen image rendering; nó không tự cung cấp scene/light editor kiểu Rhino.
`TextureMapping::onCheckEnvToggled` có environment-coordinate mapping để quan sát phản chiếu.
Nền này hữu ích cho EMap nhưng không tự đo G0/G1/G2 hay tạo lighting environment của renderer.
`CoreWorkspace::displayTitle` resolve native mode, giữ pickable edge topology ở provider.
`CoreWorkspace::reconcileDisplay` xử lý wire/shaded override; chưa chứng minh mọi mode trong title list.
`CorePictureFrame::clearPreview/cancel` tháo scene nodes và xóa QImage session.
`CorePictureFrame::finish` tạo ImagePlane và transaction; path resource cold reload cần kiểm tiếp.
[3DM archive code](../../../Gui/ThreeDmArchive.cpp) có RenderLight retention khi preserve,
nhưng giữ source record không chứng minh hiển thị/sửa/xuất current light state.

## Kế hoạch adapter Rust trước

Rust sở hữu mode/capability registry, render scene schema, resource resolver và cache metadata.
Rust quản immutable snapshots, generation/revision, allocation budgets và job/cancel state.
Cache portable buffers dùng Vec/typed structs; không giữ native scene node pointers trong worker.
C++ bridge giữ refs Coin/Qt/App đúng thread, trích OCCT shape copy và publish mesh buffers.
Native rendering APIs là ngoại lệ cần thiết; policy fallback/unsupported vẫn thuộc Rust.
FFI có checked lengths, owner cấp phát/giải phóng và error mapping; không coi native dependencies memory-safe.
Migration tách phần state khỏi CoreWorkspace/CorePictureFrame theo baseline đã nghiệm thu.
Không thay display/tessellation defaults chỉ để thống nhất tài liệu; regression cần geometry và pick.
Python dành cho bootstrap và fixture ứng dụng, không đặt scheduler/scene business logic mới.
Phụ thuộc RCORE-01 representation, RCORE-02 tolerance, RCORE-04 lifecycle, RCORE-06 visibility/identity.
Lưu metadata/resources dùng [RCORE-10](10-persistence-3dm-clipboard.md); layout dùng RCORE-11.

## Fixtures nghiệm thu bắt buộc — chưa chạy

| Fixture | Thao tác | Expected invariant |
|---|---|---|
| RCORE-12.T01 — display invariant | Rational CAD/trimmed face qua Wireframe/Shaded, đổi Deviation. | Shape/topology/units/tolerance modeling không đổi; selection vẫn đúng subelement. |
| RCORE-12.T02 — cache revision | Đổi radius khi visible, hide sửa rồi show; Undo/Redo. | Mesh/edge buffers ứng với shape hiện tại, không lóe kết quả cũ hoặc nhầm generation. |
| RCORE-12.T03 — nhiều viewport | Hai view mode/camera khác nhau, preview đang chạy. | Mode/camera riêng; pick/visibility nhất quán; cancel dọn preview mọi owner liên quan. |
| RCORE-12.T04 — material inheritance | Object dùng layer material, child override và transparency. | Assignment/override đúng schema, đổi layer chỉ tác động đối tượng kế thừa; reload giữ IDs. |
| RCORE-12.T05 — resource missing/relink | Di chuyển texture/reference image rồi relink bản nội dung khác. | Báo missing rõ; relink invalidate cache theo hash, không dùng decoded image cũ. |
| RCORE-12.T06 — render capability | Chọn backend thiếu, light unsupported hoặc shader không có. | UI/job báo khả năng thiếu; không fallback shaded rồi báo render thành công. |
| RCORE-12.T07 — image output | Render 800×600 với alpha, thử write failure/disk-full. | Đúng kích thước/channel theo contract; file tốt không bị thay bởi output một phần. |
| RCORE-12.T08 — stale/cancel/close | Đổi scene, cancel và đóng document khi worker chạy. | Kết quả cũ bị loại; không mutate Qt/App từ worker, callback không dùng object đã hủy. |
| RCORE-12.T09 — budget stress | Đồng thời preview mesh, import và render scene lớn. | Tổng budget được tôn trọng, cancellation có phản hồi; không allocation overflow hoặc job tự chiếm toàn RAM. |
| RCORE-12.T10 — cold scene persistence | Save FCStd, mở lại với camera/material/resources và renderer khác khả dụng. | Fields hỗ trợ phục hồi; phần backend thiếu báo rõ; CAD vẫn độc lập tài nguyên render. |
| RCORE-12.T11 — override scopes | Gán override toàn viewport rồi override một viewport; xóa từng scope. | Không sửa geometry; trạng thái scope rõ và trả đúng về viewport mode theo precedence được adapter công bố. |
| RCORE-12.T12 — content ownership failure | Load thư viện, preview, thêm persistent thất bại hoặc cancel. | Không rò content hoặc lưu preview vào scene; ownership và diagnostic chính xác theo kết quả thật. |
| RCORE-12.T13 — capture vs renderer | Cùng camera qua viewport capture và renderer, gây thất bại ghi file/backend. | UI ghi đúng loại output và chỉ hoàn tất khi kết quả kiểm chứng; gửi script hoặc timer start chưa là success. |

## Unknowns và điều kiện đóng

Chưa có runtime GPU/driver evidence hoặc benchmark bộ nhớ của các view modes ở checkout này.
Chưa audit renderer add-ons; nhãn chưa kiểm chứng không có nghĩa FreeCAD ecosystem không có renderer.
Chưa chốt color management, texture embedding/relink, environment/ground schema và light units.
Chưa chứng minh cancellation/preemption của từng native backend hoặc thread safety của kernel copy.
Chỉ đóng capability sau đo shape/pick/state/resource invariants và output đã chạy trên runtime có hash.
Hình ảnh nhìn hợp lý, tên menu và request-planner tests không đủ để đánh dấu rendering tương thích.
