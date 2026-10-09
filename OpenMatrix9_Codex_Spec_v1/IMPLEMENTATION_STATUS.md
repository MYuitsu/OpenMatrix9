# Implementation status — 2026-10-09

<!-- rhino-core-host-parameter-dependency:start -->
Đã nghiệm thu fixture T10 cho điểm phụ thuộc Circle/Line: dùng map openNURBS/OCCT
có sẵn, Rust kiểm điểm với source thật và chặn raw parameter chưa được chứng minh.
Query/basis/source identity được lưu FCStd và tính lại sau sửa/Undo/restore.
238 kiểm tra fixture +114 hồi quy FreeCAD;556 Rust và159 native query đạt.
Phạm vi Circle/Line và oracle lấy mẫu; **toàn bộ P0 vẫn đang triển khai.**
[Bằng chứng và giới hạn](../docs/validation/2026-10-10-rhino-core-host-parameter-dependency.md).
<!-- rhino-core-host-parameter-dependency:end -->

<!-- rhino-core-delayed-publication:start -->
Đã nghiệm thu fixture T08 cho native Rebuild: kết quả được giữ qua lượt giao,
source/session/options hết hiệu lực bị chặn trước Coin/transaction;36 envelope
và36 tham chiếu Shape giải phóng đủ.173 kiểm tra fixture và72 hồi quy đạt.
Solver vẫn đồng bộ, delivery ở GUI queue; lifetime mọi worker vẫn còn mở.
**Toàn bộ P0 vẫn đang triển khai.** [Bằng chứng và giới hạn](../docs/validation/2026-10-10-rhino-core-delayed-publication.md).
<!-- rhino-core-delayed-publication:end -->

<!-- rhino-core-subobject-ownership:start -->
Đã nghiệm thu fixture T11: sửa wrapper Curve/Surface sở hữu riêng, publish qua
native deep copy, giữ source/sibling, đúng Undo/Redo và phân biệt lifetime của
snapshot mạnh với request subobject.35 kiểm tra FreeCAD đạt; không viết lại editor.
**Toàn bộ P0 vẫn đang triển khai.** [Bằng chứng và giới hạn](../docs/validation/2026-10-10-rhino-core-subobject-ownership.md).
<!-- rhino-core-subobject-ownership:end -->

<!-- rhino-core-periodic-persistence:start -->
Đã nghiệm thu T09 bằng curve periodic thật và các face có seam/lỗ; dùng lại
serializer FCStd hiện có.204 kiểm tra warm +102 kiểm tra cold đạt, cùng221 kiểm
tra đối chứng kernel. Type/topology/basis/tọa độ và inventory giữ nguyên.
**Toàn bộ P0 vẫn đang triển khai.** [Bằng chứng và giới hạn](../docs/validation/2026-10-10-rhino-core-periodic-persistence.md).
<!-- rhino-core-periodic-persistence:end -->

<!-- rhino-core-hatch-input-gate:start -->
Đã bổ sung gate Rust cho dữ liệu NURBS của vòng Hatch trước đọc nguồn/copy/dựng
native; dùng lại editor và exporter hiện có.541 kiểm tra Rust,27 kiểm tra native,
5 bộ hồi quy native và269 kiểm tra FreeCAD đạt. Đây là một phần T07;
**toàn bộ P0 vẫn đang triển khai**. [Bằng chứng và giới hạn](../docs/validation/2026-10-10-rhino-core-hatch-input-gate.md).
<!-- rhino-core-hatch-input-gate:end -->

<!-- rhino-core-instance-isolation:start -->
Dùng lại App::Link, mirror/scale không đều và các exporter hiện có; thêm fixture
kiểm đúng tọa độ, giữ nguyên member/definition/instance A khi chỉ sửa B.
T06 đã nghiệm thu:146 kiểm tra native,98 kiểm tra FreeCAD và4 bộ hồi quy native.
Không cần viết lại code instance. **Toàn bộ P0 vẫn đang triển khai.**
[Bằng chứng và giới hạn](../docs/validation/2026-10-10-rhino-core-instance-isolation.md).
<!-- rhino-core-instance-isolation:end -->

<!-- rhino-core-polycurve-partition:start -->
Dùng lại wire, đảo hướng và FCStd của FreeCAD; sửa bước working 3DM để polycurve
giữ từng đoạn thay vì gộp thành một cạnh. Rust kiểm tra thứ tự, miền và điểm nối;
báo cáo có trước commit. T05 đã nghiệm thu với 403 kiểm tra native, 382 kiểm tra
FreeCAD và 523 Rust tests. **Toàn bộ P0 vẫn đang triển khai.**
[Bằng chứng và giới hạn](../docs/validation/2026-10-10-rhino-core-polycurve-partition.md).
<!-- rhino-core-polycurve-partition:end -->

<!-- rhino-core-curve-parameters:start -->
Dùng lại openNURBS/OCCT cho báo cáo query tham số toàn miền: giữ native form1/2,
chặn output lỗi trước khi tính điểm và giữ basis/report độc lập. 502 Rust tests;
159 kiểm tra query native; 257 kiểm tra FreeCAD. Fixture RCORE-01.T01 đã nghiệm thu
với oracle lấy mẫu; T10/History vẫn thiếu tích hợp host. **Toàn bộ P0 đang triển khai.**
[Bằng chứng và giới hạn](../docs/validation/2026-10-10-rhino-core-curve-parameters.md).
<!-- rhino-core-curve-parameters:end -->

<!-- rhino-core-kernel-contract:start -->
Dùng lại bộ chuyển đổi FreeCAD/OCCT/openNURBS; bổ sung validator basis Rust,
descriptor chỉ đọc và policy cho inspection/working 3DM. 480 Rust tests;
504 kiểm tra ứng dụng / 8 lần chạy, gồm mở FCStd trong tiến trình mới.
**P0 toàn bộ vẫn đang triển khai.** [Bằng chứng và phạm vi](../docs/validation/2026-10-10-rhino-core-kernel-contract.md).
<!-- rhino-core-kernel-contract:end -->

<!-- rhino-core-extended-input:start -->
Bổ sung nhập polar/spherical/surveyor, fraction/feet-inch và góc DMS bằng Rust,
dùng lại bộ dựng hình FreeCAD/OM9 hiện có. Sửa mốc tương đối và Undo của Circle/Sphere.
**421 Rust tests; 1108 native checks / 21 runs; 49 Python pass,1 skip.**
P0 toàn bộ còn mở; [phạm vi và bằng chứng](../docs/validation/2026-10-09-rhino-core-extended-input.md).

<!-- rhino-core-extended-input:end -->

<!-- rhino-core-point-consumers:start -->
Distance, Angle và PictureFrame đã dùng chung parser Rust, giữ tọa độ/đơn vị đầu vào và provenance.
CPlane sai scale/shear bị từ chối; PictureFrame kiểm giao dịch trước mutation.
**324 native checks / 9 runs; 364 Rust tests; 49 Python pass, 1 skip.**
P0 toàn bộ vẫn đang triển khai; [phạm vi và bằng chứng](../docs/validation/2026-10-09-rhino-core-point-consumers.md).

<!-- rhino-core-point-consumers:end -->

<!-- rhino-core-tolerance-workers:start -->
Rust đã nối dung sai document/override vào Join curve và surface, lưu cutoff thực tế
và provenance cùng kết quả; backend không được tự nâng dung sai. Worker nhập 3DM
dùng 60% logical CPUs, bỏ trần cố định 4, kiểm override trước mutation.
**444 native checks / 8 runs; 347 Rust tests; 49 Python pass, 1 skip.**
P0 toàn bộ vẫn đang triển khai; [báo cáo và giới hạn](../docs/validation/2026-10-09-rhino-core-tolerance-workers.md) ghi phạm vi chính xác.

<!-- rhino-core-tolerance-workers:end -->

<!-- rhino-core-object-foundations:start -->
## Bổ sung nền tảng object dùng chung — 2026-10-09

RCORE-01/04/06: registry Rust quản lý document session, object generation/revision
và Undo/Redo epoch; native guard được nối vào Edit/Boolean, Rebuild và Surface
tương tác. Input stale bị từ chối trước transaction, kể cả thay bằng đối tượng
cùng tên/cùng shape hoặc phục hồi bằng Undo.
[Báo cáo kiểm chứng](../docs/validation/2026-10-09-rhino-core-object-foundations.md) ghi kiểm thử và scope chính xác.
11 feature liên quan có `implementation_followup_evidence`; không đổi trạng thái
607 feature và không đóng toàn bộ P0. Policy quyền/chọn/layer, capability chung,
tolerance resolver và durable History vẫn còn việc.
<!-- rhino-core-object-foundations:end -->

<!-- rhino-core-p0-checkpoint:start -->
## Nền tảng P0 xuyên các nhóm — 2026-10-09

`RCORE-01–07/09–10`: `partially_implemented`; các phần có bằng chứng được ghi riêng tại
[checkpoint P0](../docs/validation/2026-10-09-rhino-core-p0.md).
Đã nối context đơn vị vào Curve, parser/frame chung, CPlane API/menu, snap Once/Only,
repeat thành công, validation spline và schema/dependencies 3DM, History Record/Update độc lập.
Không đóng toàn bộ chương hoặc nâng 607 feature từ các thay đổi hạ tầng này.
Những phần cần triển khai tiếp và fixture chưa nghiệm thu vẫn để mở.
<!-- rhino-core-p0-checkpoint:end -->

Các lệnh dưới đây đã có phần triển khai native; bằng chứng kiểm thử hoặc trạng thái chờ kiểm chứng được ghi riêng.
`partially_implemented` nghĩa là phần hỗ trợ đã hoạt động; các tùy chọn
còn thiếu được ghi rõ trong spec. `validated_supported_slice` chỉ xác nhận
phạm vi đã kiểm thử, không xác nhận đầy đủ tương thích Matrix/Rhino.

| ID | Lệnh | Trạng thái | Phạm vi / kiểm chứng |
|---|---|---|---|
| [OM9-CURVE-001](specs/02-curve/om9-curve-001-polyline.md) | PolyLine | Đã triển khai một phần; phần hỗ trợ đã kiểm chứng | Straight-segment Polyline, CMD/mouse, Close, PersistentClose, Length, Undo; arc modes remain unsupported. |
| [OM9-CURVE-002](specs/02-curve/om9-curve-002-line.md) | Line | Đã triển khai một phần; phần hỗ trợ đã kiểm chứng | Two-point Line, CMD/mouse, BothSides, shared CPlane/snaps/Ortho; specialist construction modes remain unsupported. |
| [OM9-CURVE-003](specs/02-curve/om9-curve-003-interp-curve.md) | Interp Curve | Đã triển khai một phần; phần hỗ trợ đã kiểm chứng | Native B-spline interpolation, Degree/Knots, smooth/sharp close, AutoClose/Alt, preview, Undo; tangent constraints remain unsupported. |
| [OM9-CURVE-009](specs/02-curve/om9-curve-009-rebuild.md) | Rebuild | Đã triển khai một phần; phần hỗ trợ đã kiểm chứng | Curve/wire rebuild, exact PointCount/Degree, sampled preview deviation, batch DeleteInput and persistence; active layers/subedges remain unsupported. |
| [OM9-CURVE-004](specs/02-curve/om9-curve-004-rectangle.md) | Rectangle | Đã triển khai một phần; phần hỗ trợ đã kiểm chứng | Native sharp Rectangle: opposite corners, Center, 3Point, Vertical, Length/Width, Shift square, preview, Undo/Redo and FCStd persistence; Rounded/Conic remain unsupported. |
| [OM9-CURVE-005](specs/02-curve/om9-curve-005-circle.md) | Circle | Đã triển khai một phần; phần hỗ trợ đã kiểm chứng | Native Circle basic/advanced modes; bounded spatial Tangent for noncoplanar NURBS with exact native contact verification, Radius/Point/FromFirstPoint/Vertical/Solution; versioned History for all Circle branches including FitPoints and Deformable, explicit point/direction/radius sources, Record/Update/Lock and cold FCStd; persistent active-layer membership/color/visibility/lock with Undo/Redo. Exhaustive roots, automatic topology correspondence and full Rhino layer inheritance remain unaccepted. |
| [OM9-CURVE-006](specs/02-curve/om9-curve-006-ellipse.md) | Ellipse | Đã triển khai một phần; phần hỗ trợ đã kiểm chứng | Native exact closed Ellipse: Center, Diameter/right-click, Corner, Vertical, spatial FromFoci/MarkFoci, AroundCurve exact bounded edge/tangent, Deformable Degree/PointCount; preview, current layer membership/color/visibility/lock, atomic ellipse plus two foci, Undo/Redo and FCStd. Snapshot outputs; History, automatic topology correspondence and full editing/Rhino layer compatibility remain unaccepted. |
| [OM9-SURFACE-001](specs/03-surface/om9-surface-001-sweep-1.md) | Sweep 1 | Đã triển khai một phần; phần hỗ trợ đã kiểm chứng | Native Sweep1 with section Refit using numerical hull bounds, optional associative History, Chain Edges, seams, Rebuild, Closed Sweep and Preview; Road-like/blending/miters/Refit Rail remain unsupported. |
| [OM9-SURFACE-003](specs/03-surface/om9-surface-003-sweep-2.md) | Sweep 2 | Đã triển khai một phần; phần hỗ trợ đã kiểm chứng | Native Sweep2 with bounded support-edge G1/G2, section Refit, one-profile/open-rail Add Slash, optional History and earlier options; constraint combinations and multiple-profile Maintain Height remain unsupported. |
| [OM9-SURFACE-009](specs/03-surface/om9-surface-009-loft.md) | Loft | Đã triển khai một phần; phần hỗ trợ đã kiểm chứng | Native Loft with bounded Normal/Tight support-edge tangent matching, section Refit, optional History and earlier styles/seams; unsupported fitting/closed/rational tangent combinations and SplitAtTangents remain. |
| [OM9-SOLID-012](specs/04-solid/om9-solid-012-box.md) | Box | Đã triển khai một phần; phần hỗ trợ đã kiểm chứng | Native Corners/Diagonal/3Point/Vertical/Center and oriented Cube Box; full dimensions, Enter defaults, signed extents, frozen CPlane, CMD/mouse, right-click shortcut, scene preview and persistence. Associative History/layer/Styles integration remains unverified. |
| [OM9-SOLID-014](specs/04-solid/om9-solid-014-sphere.md) | Sphere | Đã triển khai một phần; phần hỗ trợ đã kiểm chứng | Native Center/Diameter, 2Point, 3Point/Radius, 4Point/Radius, Vertical, FitPoints, AroundCurve and planar Tangent Sphere; exact curve references, explicit ambiguous branch selection, native mouse/CMD, preview and persistence. General nonplanar tangent solving and associative History remain unsupported. |
| [OM9-TOP11-005](specs/01-core/om9-top11-005-explode.md) | Explode | Đã triển khai một phần; phần hỗ trợ đã kiểm chứng | Polycurve edges, polysurface faces, compound children and Mesh connected components; source groups/world placement/display fields preserved. Final SDK retained3DM40/40 and native special25/25 validate supported adapters; earlier geometry scope remains validated. |
| [OM9-HISTORY-001](specs/01-core/om9-history-001-history-workflow.md) | History Workflow | partially_implemented; validated_supported_slice (2026-10-09) | Native Join/Surface History and reusable Builder record/output graphs with Record/Update/Lock/Clear, detach, Undo and FCStd persistence; native 3D Cage bindings update independently of global Record/Update. Arbitrary original Gem builders, topology correspondence and full Matrix/Rhino History remain unsupported. See [validation](../docs/validation/2026-10-09-reusable-builder-cage.md). |
| [OM9-TRANSFORM-041](specs/05-transform/om9-transform-041-create-cage.md) | Create Cage | partially_implemented; validated_supported_slice (2026-10-09) | Native menu/CMD Cage and createCage create one independent editable 3D CageControl from a World BoundingBox with positive finite X/Y/Z extents, explicit OM9 counts 2/2/2 and degrees 1/1/1, and no automatic captive capture. CPlane/3Point, base-point construction and 1D/2D controls remain unsupported. See [validation](../docs/validation/2026-10-09-reusable-builder-cage.md). |
| [OM9-TRANSFORM-022](specs/05-transform/om9-transform-022-releasefromcage.md) | ReleaseFromCage | partially_implemented; validated_supported_slice (2026-10-09) | Native menu/CMD ReleaseFromCage and releaseFromCage detach selected whole captives in one Undo transaction while preserving current geometry, controls and unselected relationships; native links persist through FCStd. No arbitrary original cage workflow compatibility is claimed. See [validation](../docs/validation/2026-10-09-reusable-builder-cage.md). |
| [OM9-TRANSFORM-020](specs/05-transform/om9-transform-020-cage-edit.md) | Cage Edit | partially_implemented; validated_supported_slice (2026-10-09) | Native menu/CMD CageEdit and captureCage bind whole captives to one existing native 3D cage with Global/Local and finite falloff; triangular Mesh and exact single-edge/rectangular-face nonlinear deformation, global affine-only general BRep, frozen current bind data and persistent links independent of global Record/Update. Verified retained 3D cage restoration is explicit. 1D/2D controls, Accurate/Fast and general BRep refit remain unsupported. See [validation](../docs/validation/2026-10-09-reusable-builder-cage.md). |
| [OM9-GEM-027](specs/10-gems/om9-gem-027-match-attributes.md) | Match Attributes | partially_implemented; validated_supported_slice (2026-10-09) | Native matchBuilderAttributes API copies all supported durable Builder records atomically to targets with the same explicit nonempty OM9GemShape tag. Original gvMatchAttributes command remains disabled; selective styles popup, arbitrary Gem settings/cutter solvers and .mss interchange remain unsupported. See [validation](../docs/validation/2026-10-09-reusable-builder-cage.md). |
| [OM9-BUILDERCORE-001](specs/01-core/om9-buildercore-001-builder-framework.md) | Builder Framework | partially_implemented; validated_supported_slice (2026-10-09) | Reusable native createBuilderRecord/restoreBuilderOutputs APIs with versioned affine-template recipes, durable parameters, independent output slots, grouped frames and History integration. API foundation only; original per-builder geometry solvers, sliders/handles/profile/Styles UI and .mss interchange remain unsupported and original commands stay disabled. See [validation](../docs/validation/2026-10-09-reusable-builder-cage.md). |
| [OM9-TOOLS-017](specs/09-tools/om9-tools-017-join-history.md) | Join History | Đã triển khai một phần; phần hỗ trợ đã kiểm chứng | Connected separate native curves create a retained-parent HistoryJoin child; surfaces and Mesh reject. See [native contract](../docs/features/history-native-contract.md). |
| [OM9-INFO-015](specs/01-core/om9-info-015-rhino-history.md) | Rhino History | Đã triển khai một phần; phần hỗ trợ đã kiểm chứng | History panel exposes Record, Update, Lock and BrokenHistoryWarning; each change applies immediately in its own Undo transaction. See [native contract](../docs/features/history-native-contract.md). |
| [OM9-INFO-016](specs/01-core/om9-info-016-matrix-clear-object-history.md) | Matrix Clear Object History | Đã triển khai một phần; phần hỗ trợ đã kiểm chứng | Selected HistoryJoin/SurfaceHistory objects lose incoming records while geometry and downstream links remain. See [native contract](../docs/features/history-native-contract.md). |
| [OM9-INFO-017](specs/01-core/om9-info-017-matrix-history-record.md) | Matrix History Record | Đã triển khai một phần; phần hỗ trợ đã kiểm chứng | RCORE-09 policy revision: Record controls new supported links; existing recorded descendants update whenever Update is On, independently of Record. See [native contract](../docs/features/history-native-contract.md). |
| [OM9-INFO-018](specs/01-core/om9-info-018-matrix-history-update.md) | Matrix History Update | Đã triển khai một phần; phần hỗ trợ đã kiểm chứng | Update toggle suspends/resumes recorded descendants without clearing links. See [native contract](../docs/features/history-native-contract.md). |
| [OM9-TOP11-008](specs/01-core/om9-top11-008-join.md) | Join | Đã triển khai một phần; phần hỗ trợ đã kiểm chứng | Curve endpoint midpoint snapping with explicit bounded tolerance and sewn surface shells; disconnected/mixed/ambiguous inputs rejected. Document-tolerance compatibility and rich History remain unverified. |
| [OM9-TOP11-010](specs/01-core/om9-top11-010-trim.md) | Trim | Đã triển khai một phần; phần hỗ trợ đã kiểm chứng | True 3D curve/surface splitting, ExtendLines and frozen orthographic/perspective ApparentIntersections for curves, GUI/CMD region removal, Undo/reset and persistence. Degenerate/overlapping projections and apparent surfaces reject. |
| [OM9-SOLID-001](specs/04-solid/om9-solid-001-boolean-difference.md) | BooleanDifference | Đã triển khai một phần; phần hỗ trợ đã kiểm chứng | Ordered target/cutter sets for solid BRep, surface-area BRep or closed Mesh via polyhedral OCCT; target replacement and cutter DeleteInput, cavities and persistence. Mixed categories and invalid/empty results reject. |
| [OM9-SOLID-002](specs/04-solid/om9-solid-002-boolean-intersection.md) | BooleanIntersection | Đã triển khai một phần; phần hỗ trợ đã kiểm chứng | Ordered solid BRep, surface-area BRep or closed Mesh sets; validated common results and persistence. Mixed categories, edge-only surfaces and invalid/empty results reject. |
| [OM9-SOLID-003](specs/04-solid/om9-solid-003-boolean-union.md) | BooleanUnion | Đã triển khai một phần; phần hỗ trợ đã kiểm chứng | Solid BRep, surface-area BRep or closed Mesh union, disconnected components/cavities, DeleteInput and persistence. Mixed categories and invalid results reject. |
| [OM9-SOLID-004](specs/04-solid/om9-solid-004-boolean-two-objects.md) | Boolean2Objects | Đã triển khai một phần; phần hỗ trợ đã kiểm chứng | Exactly two same-category BRep solid/surface or closed Mesh inputs; five Union/A-B/B-A/Common/XOR modes, DeleteInput and persistence. Exhaustive topology and live History remain unverified. |

| [OM9-SURFACE-002](specs/03-surface/om9-surface-002-sweep-1-history.md) | Sweep 1 History | Đã triển khai một phần; phần hỗ trợ đã kiểm chứng | Native gvSweepHistory CMD alias with exact ID, conditional Closed Yes, source/parent recompute, detach/Undo and FCStd; dedicated History menu/F6 and full Matrix workflow remain unverified. |

| [OM9-SURFACE-004](specs/03-surface/om9-surface-004-sweep-2-history.md) | Sweep 2 History | Đã triển khai một phần; phần hỗ trợ đã kiểm chứng | Native gvSweep2History CMD alias with exact ID, one-profile Maintain Height Yes, conditional Closed Yes, source/parent recompute and native persistence; full History/menu parity remains unverified. |

## Bằng chứng

- Ellipse: **75/75 native checks**, **199/199 hồi quy Curve/layer**, **316 Rust tests**; [phạm vi và bằng chứng](../docs/validation/2026-10-09-ellipse.md).

- Circle còn thiếu: **176/176 kiểm tra native mới** (30 Tangent 3D,89 History,52 layer,5 cold restore); **665/665 kiểm tra hồi quy native**; [phạm vi và bằng chứng](../docs/validation/2026-10-09-circle-complete-branches.md).

- Circle spatial/History: **54/54 native checks** gồm 51 spatial/History và 3 cold restore; **211/211 hồi quy Curve**, **148/148 hồi quy History/Surface**; [phạm vi và bằng chứng](../docs/validation/2026-10-09-circle-spatial-history.md).

- Circle nâng cao: **64/64 native checks**, **147/147 hồi quy Curve** và **252/252 hồi quy Box/Sphere** cho adapter dùng chung; [phạm vi và bằng chứng](../docs/validation/2026-10-09-circle-advanced.md).

- Surface advanced/History: **530/530 native checks**, gồm 108 advanced, 77 constraints, 65 graph, 3 cold restore và 277 hồi quy; [phạm vi và bằng chứng](../docs/validation/2026-10-09-surface-constraints-history.md).

- Circle: 42/42 kiểm tra native; 105/105 hồi quy Rectangle/Line/Polyline/Interp/Rebuild với module mới được xác nhận; [phạm vi và bằng chứng](../docs/validation/2026-10-09-circle.md).

- Rectangle: 36/36 kiểm tra native; [phạm vi và bằng chứng](../docs/validation/2026-10-09-rectangle.md).

- Line/Polyline: 26/26 kiểm tra native.
- Interp Curve/Rebuild: 40/40 kiểm tra native.
- Sweep1/Sweep2/Loft: 180/180 kiểm tra tùy chọn và 97/97 hồi quy native; [phạm vi, bằng chứng và giới hạn](../docs/validation/2026-10-09-surface-options.md).
- Box/Sphere: 251/251 kiểm tra native mở rộng (SDK), 70/70 hồi quy cơ bản và 115/115 hồi quy Edit; [báo cáo native và giới hạn](../docs/validation/2026-10-09-solid-box-sphere-options.md).
- Edit/Boolean: 115/115 core và 57/57 tùy chọn native; PDF84–87,223–224 đã đọc lại ngày 2026-10-09.
- Các báo cáo đã được đọc và xác nhận `ok=true`, mọi check đều qua.
- Chi tiết phiên chạy và build: [implementation ledger](../docs/openmatrix9-progress.json).

## Quy tắc cập nhật

Theo yêu cầu người dùng, phần đã hoàn thành được đồng bộ vào từng spec,
FEATURES.json và FEATURES.yaml; hợp đồng mục tiêu và bằng chứng native được bảo toàn.
Ledger ngoài package vẫn giữ bằng chứng kiểm thử chi tiết.
Các feature khác chưa được xác nhận trong lần đồng bộ này giữ nguyên trạng thái;
`not_started` của chúng không phải kết luận mới rằng code không tồn tại.
Sau mỗi lần triển khai/kiểm chứng tiếp theo, cập nhật phạm vi, ngày, checklist
và bằng chứng tại chính thư mục này. Không đánh dấu toàn bộ feature hoàn tất
chỉ vì một phần hoặc một icon đã hoạt động.


## Hướng dẫn mẫu — 2026-10-07

607 contract có mẫu Rust độc lập và kiểm thử validator. Kết quả này không thay đổi trạng thái native hoặc supported slice của các feature.
