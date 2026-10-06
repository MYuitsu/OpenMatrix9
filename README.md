# OpenMatrix9

![OpenMatrix9 — Open-source Jewelry CAD Workbench](Resources/branding/introduction.png)

An experimental jewelry CAD workbench for FreeCAD. Rust owns command catalog,
state and behavior; C++/Qt provides native integration. Unsupported commands
remain visible and disabled.

This project was developed with Matrix9 as a functional reference. Compatibility
identifiers and familiar color roles remain. This public snapshot excludes
commercial artwork, extracted resource archives, manuals, decompiled source and
private design samples. It does not claim clean-room development, affiliation
with the original vendors, or legal clearance of the implementation.

All **510 icon bindings** use independently authored SVGs. The exporter accepts
authored catalogs only and cannot restore legacy bitmaps.

## Ủng hộ phát triển / Donate

Nếu OpenMatrix9 hữu ích với bạn, bạn có thể donate để hỗ trợ phát triển và kiểm
thử các chức năng CAD, import/export 3DM và tương thích openNURBS.
Mọi đóng góp đều tự nguyện. Cảm ơn bạn đã hỗ trợ dự án!

- **PayPal:** gửi tới **`nguyenthaiduy277@gmail.com`** trong PayPal.
- **MoMo:** quét mã QR dưới đây. Tên người nhận: **NGUYEN THAI DUY**.

<img src="docs/images/donate/momo-qr.png" alt="Mã QR MoMo donate cho NGUYEN THAI DUY" width="300">

## Build và cài đặt

Xem [hướng dẫn build FreeCAD và OpenMatrix9 trên Windows](docs/build-windows.md):

- **Build cùng FreeCAD từ source**: clone workspace, tích hợp CMake, Pixi, build và chạy.
- **Build riêng OpenMatrix9**: dùng source/build SDK và dependencies của FreeCAD đã build.
- **Dùng với FreeCAD installer/portable**: yêu cầu ABI/SDK khớp, vị trí `.pyd` và resources, kiểm tra sau cài.

OpenMatrix9 có module native C++/Rust; chỉ clone vào `Mod` chưa đủ.
Bản cài FreeCAD thông thường không tự cung cấp SDK để build module này.

## Tiến độ lệnh / Command progress

Cập nhật: **2026-10-06**. Bảng này mô tả phạm vi đang có trong source public.
**510 icon bindings không tương đương 510 lệnh đã hoàn thành.** Lệnh chưa có
implementation vẫn hiển thị nhưng bị vô hiệu hóa. Có tên trong command completion
hoặc có phím tắt không chứng minh lệnh đã chạy được.

| Trạng thái | Ý nghĩa |
|---|---|
| 🟢 Đã làm (phạm vi hiện tại) | Có đường thực thi native/Rust và kiểm tra tương ứng; không cam kết toàn bộ tùy chọn của Matrix9/Rhino. |
| 🟡 Một phần | Có chức năng nền tảng nhưng còn giới hạn hoặc chưa tích hợp phần phát triển mới. |
| 🔴 Chưa hoàn thành | Chưa có đường thực thi được xác nhận; icon/catalog chưa được tính là implementation. |

### Danh sách từng lệnh theo nhóm

Mỗi dòng là một mục catalog hoặc lệnh bổ sung. **Tên hiển thị** mô tả mục menu;
**mã catalog** dùng để đối chiếu source, không phải mọi mã đều là command gõ được.
Màu trạng thái được giải thích ở đầu bảng: 🟢 đã làm trong phạm vi hiện tại,
🟡 làm một phần, 🔴 chưa hoàn thành. Ý nghĩa của mục chưa làm mô tả chức năng dự kiến theo tên catalog;
những tên nghiệp vụ chưa đủ căn cứ được ghi rõ cần xác minh.
Các nút sidebar cũng được liệt kê để không nhầm icon đã vẽ với chức năng đã làm.
Mục xuất hiện trong nhiều nhóm có thể lặp lại, cùng mã luôn có cùng trạng thái.

#### Quick tools — thao tác nhanh

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Trạng thái |
|---|---|---|---|
| Duplicate | `TopIconDuplicate` | Tạo bản sao của đối tượng. | 🔴 |
| Edit Points On | `TopIconEditPointsOn` | Bật các điểm chỉnh sửa đối tượng. | 🔴 |
| PointsOn | `TopIconControlPointsOn` | Bật control points để chỉnh hình dạng. | 🔴 |
| Mirror | `TopIconMirror` | Tạo bản đối xứng qua mặt phẳng. | 🔴 |
| Move | `TopIconMove` | Di chuyển đối tượng. | 🔴 |
| Rotate | `TopIconRotate` | Xoay đối tượng quanh tâm hoặc trục. | 🔴 |
| Explode | `TopIconExplode` | Tách đối tượng ghép thành các thành phần. | 🔴 |
| Join | `TopIconJoin` | Ghép các đường hoặc mặt tương thích. | 🔴 |
| Split | `TopIconSplit` | Chia đối tượng bằng đối tượng cắt. | 🔴 |
| Trim | `TopIconTrim` | Cắt bỏ phần thừa của đối tượng. | 🔴 |
| Ring Rail | `TopIconRingRail` | Tạo đường dẫn cơ sở cho thân nhẫn. | 🔴 |

#### File

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Trạng thái |
|---|---|---|---|
| New | `FileNew` | Tạo project FreeCAD mới. | 🟢 |
| Open | `FileOpen` | Mở project hoặc file được FreeCAD hỗ trợ. | 🟢 |
| Export Selected | `FileExportSelected` | Xuất các đối tượng đang chọn theo workflow export chung. | 🔴 |
| Import | `FileImport` | Nhập file theo workflow import chung. | 🔴 |
| Save | `FileSave` | Lưu project hiện hành. | 🟢 |
| Save As | `FileSaveAs` | Lưu project với tên hoặc vị trí khác. | 🟢 |
| Save Small | `FileSaveSmall` | Lưu bản dung lượng nhỏ theo workflow catalog. | 🔴 |
| Save Small As | `FileSaveSmallAs` | Lưu bản dung lượng nhỏ với tên khác. | 🔴 |
| User Library | `FileUserLibrary` | Mở thư viện đối tượng của người dùng. | 🔴 |
| User Library Add | `FileUserLibraryAdd` | Thêm đối tượng vào thư viện người dùng. | 🔴 |
| Stuller Submit | `FileStullerSubmit` | Gửi thiết kế theo workflow Stuller trong catalog. | 🔴 |
| Notes | `FileNotes` | Đọc và sửa ghi chú lưu cùng project. | 🟢 |
| Print | `FilePrint` | In bản vẽ hoặc nội dung project. | 🔴 |

#### View

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Trạng thái |
|---|---|---|---|
| PictureFrame | `ViewBackgroundBitmapPictureFrame` | Đặt ảnh tham chiếu vào không gian làm việc. | 🟢 |
| RestoreViewports | `ViewRestoreViewports` | Khôi phục bố cục bốn viewport của OM9. | 🟢 |
| SynchronizeViews | `OthersViewSynchronizeViews` | Đồng bộ các viewport theo quy tắc camera của OM9. | 🟢 |
| Zoom_Dynamic | `ViewZoomZoomDynamic` | Zoom tương tác bằng thao tác kéo chuột. | 🟢 |
| Zoom_Extents | `ViewZoomZoomExtents` | Zoom để thấy toàn bộ hình học trong viewport. | 🟢 |
| Zoom_Selected | `ViewZoomZoomSelected` | Zoom tới các đối tượng đang chọn. | 🟢 |
| Zoom_Window | `ViewZoomZoomWindow` | Zoom tới vùng hình chữ nhật được chọn. | 🟢 |
| View Manager | `ViewManager` | Quản lý các view đã lưu. | 🔴 |
| ViewCaptureToFile | `ViewCaptureToFile` | Lưu ảnh chụp viewport ra file. | 🟢 |
| Crosshairs | `OthersSetCrosshairs` | Bật hoặc tắt dấu chữ thập theo con trỏ. | 🟢 |
| 1 To1 | `ViewZoomZoom1To1` | Đặt tỷ lệ hiển thị 1:1. | 🔴 |
| 1 To1 Calibrate | `ViewZoomZoom1To1Calibrate` | Hiệu chuẩn tỷ lệ hiển thị 1:1 theo màn hình. | 🔴 |
| CenterViewport | `ViewSetCameraCenterViewport` | Đặt tâm camera của viewport. | 🟢 |
| Blueprint | `ClayooCreationBlueprint` | Tạo bố cục ảnh blueprint tham chiếu. | 🔴 |
| System6 HD3 View | `OthersSystem6HD3View` | Workflow hiển thị HD3View trong catalog; chi tiết chưa được xác minh. | 🔴 |

#### Utilities

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Trạng thái |
|---|---|---|---|
| Direction | `AnalyzeDirection` | Kiểm tra chiều đường cong hoặc hướng pháp tuyến. | 🔴 |
| Diagnostics Select Bad Objects | `AnalyzeDiagnosticsSelectBadObjects` | Chọn các đối tượng hình học bị lỗi. | 🔴 |
| Diagnostics Check | `AnalyzeDiagnosticsCheck` | Kiểm tra tính hợp lệ của đối tượng. | 🔴 |
| Edge Tools Show Edges | `AnalyzeEdgeToolsShowEdges` | Hiển thị các cạnh và cạnh hở. | 🔴 |
| Edge Tools Split Edge | `AnalyzeEdgeToolsSplitEdge` | Tách một cạnh hình học. | 🔴 |
| Edge Tools Merge Edge | `AnalyzeEdgeToolsMergeEdge` | Gộp các cạnh tương thích. | 🔴 |
| Edge Tools Join2 Naked Edges | `AnalyzeEdgeToolsJoin2NakedEdges` | Ghép hai cạnh hở. | 🔴 |
| Edge Tools Unjoin Edge | `AnalyzeEdgeToolsUnjoinEdge` | Tách liên kết giữa các cạnh đã ghép. | 🔴 |
| Make2 DDrawing | `DimensionMake2DDrawing` | Tạo hình chiếu hoặc bản vẽ 2D từ mẫu 3D. | 🔴 |
| Bounding Box | `AnalyzeBoundingBox` | Tạo hộp giới hạn của đối tượng. | 🔴 |
| gvCenterObjects | `AnalyzeCenterPoint` | Xác định tâm theo workflow gvCenterObjects. | 🔴 |
| Group | `EditGroupsGroup` | Gom các đối tượng thành nhóm. | 🔴 |
| UnGroup | `EditGroupsUnGroup` | Bỏ nhóm đối tượng. | 🔴 |
| Extract Bad Surface | `OthersExtractBadSurface` | Trích xuất các mặt bị lỗi để kiểm tra. | 🔴 |
| View Show ZBuffer | `ViewShowZBuffer` | Hiển thị dữ liệu độ sâu Z-buffer. | 🔴 |
| Fill Hole | `MeshMeshRepairToolsFillHole` | Lấp một lỗ trên mesh. | 🔴 |
| Fill Holes | `MeshMeshRepairToolsFillHoles` | Lấp các lỗ trên mesh. | 🔴 |
| Align Mesh Vertices | `MeshMeshRepairToolsAlignMeshVertices` | Căn chỉnh các đỉnh mesh. | 🔴 |
| From NURBSObject | `MeshFromNURBSObject` | Tạo mesh từ geometry NURBS. | 🔴 |
| Reduce Vertex Count | `MeshMeshEditToolsCollapseReduceVertexCount` | Giảm số đỉnh mesh bằng collapse. | 🔴 |
| Apply Mesh UVN | `MeshApplyMeshUVN` | Áp dụng thao tác UVN cho mesh. | 🔴 |
| Extract Render Mesh | `MeshMeshEditToolsExtractRenderMesh` | Trích xuất mesh dùng để render. | 🔴 |

#### Measure

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Trạng thái |
|---|---|---|---|
| Angle | `AnalyzeAngle` | Đo góc giữa hai hướng xác định bằng bốn điểm. | 🟢 |
| Distance | `AnalyzeDistance` | Đo khoảng cách giữa hai điểm trong không gian. | 🟢 |
| Length | `AnalyzeLength` | Đo chiều dài đường cong hoặc cạnh. | 🔴 |
| Radius | `AnalyzeRadius` | Đo bán kính đường tròn hoặc cung. | 🔴 |
| Measure Dim Horizontal | `OthersMeasureDimHorizontal` | Tạo kích thước theo phương ngang. | 🔴 |
| Measure Dim Vertical | `OthersMeasureDimVertical` | Tạo kích thước theo phương đứng. | 🔴 |
| Angle Dimension | `DimensionAngleDimension` | Tạo chú thích kích thước góc. | 🔴 |
| Rotated Dimension | `DimensionRotatedDimension` | Tạo kích thước theo phương xoay. | 🔴 |
| Aligned Dimension | `DimensionAlignedDimension` | Tạo kích thước thẳng hàng với hai điểm. | 🔴 |
| Diameter Dimension | `DimensionDiameterDimension` | Tạo kích thước đường kính. | 🔴 |
| Radial Dimension | `DimensionRadialDimension` | Tạo kích thước bán kính. | 🔴 |
| Leader | `DimensionLeader` | Tạo đường dẫn chú thích leader. | 🔴 |
| Text Block | `DimensionTextBlock` | Tạo khối chữ chú thích. | 🔴 |
| Measure Edit Text | `OthersMeasureEditText` | Sửa nội dung chữ chú thích. | 🔴 |
| Measure Edit Dim | `OthersMeasureEditDim` | Sửa đối tượng kích thước. | 🔴 |
| Recenter Dimension Text | `DimensionRecenterDimensionText` | Đưa chữ kích thước về giữa. | 🔴 |
| Measure Dim Options | `OthersMeasureDimOptions` | Thiết lập tùy chọn kích thước. | 🔴 |

#### Curve

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Trạng thái |
|---|---|---|---|
| Polyline | `CurvePolylinePolyline` | Vẽ đường gấp khúc qua nhiều điểm bằng chuột hoặc console. | 🟢 |
| Line | `CurveLineSingleLine` | Vẽ đoạn thẳng giữa các điểm bằng chuột hoặc console. | 🟢 |
| Interpolate Points | `CurveFreeFormInterpolatePoints` | Vẽ đường cong nội suy qua các điểm. | 🔴 |
| Rectangle Cornerto Corner | `CurveRectangleCornertoCorner` | Vẽ hình chữ nhật bằng hai góc. | 🔴 |
| Circle Center Radius | `CurveCircleCenterRadius` | Vẽ đường tròn từ tâm và bán kính. | 🔴 |
| Ellipse From Center | `CurveEllipseFromCenter` | Vẽ ellipse từ tâm. | 🔴 |
| Arc Center Start Angle | `CurveArcCenterStartAngle` | Vẽ cung tròn từ tâm, điểm đầu và góc. | 🔴 |
| Arc Start End Direction | `CurveArcStartEndDirection` | Vẽ cung tròn theo điểm đầu, điểm cuối và hướng. | 🔴 |
| Curve Rebuild | `OthersCurveRebuild` | Dựng lại đường cong với cấu trúc control points mới. | 🔴 |
| Refit To Tolerance | `CurveCurveEditToolsRefitToTolerance` | Fit lại đường cong theo dung sai. | 🔴 |
| Blend Curves | `CurveBlendCurvesBlendCurves` | Tạo đường nối chuyển tiếp giữa các đường cong. | 🔴 |
| Blend Crv | `CurveBlendCurvesBlendCrv` | Tạo đường blend với điều kiện liên tục. | 🔴 |
| Curve From2 Views | `CurveCurveFrom2Views` | Tạo đường cong 3D từ hai hình chiếu. | 🔴 |
| Crv2 View History | `BuilderCrv2ViewHistory` | Tạo đường cong từ hai view có quan hệ history. | 🔴 |
| Pullback | `CurveCurveFromObjectsPullback` | Kéo đường cong về mặt tham chiếu. | 🔴 |
| Project | `CurveCurveFromObjectsProject` | Chiếu đường cong lên mặt. | 🔴 |
| Intersection | `CurveCurveFromObjectsIntersection` | Tạo đường giao giữa các đối tượng. | 🔴 |
| Curve | `CurveOffsetCurve` | Tạo đường offset ở khoảng cách cho trước. | 🔴 |
| Offset Crv On Srf | `CurveOffsetOffsetCrvOnSrf` | Offset đường cong trên mặt. | 🔴 |
| Extract Isocurve | `CurveCurveFromObjectsExtractIsocurve` | Trích đường đẳng tham số trên mặt. | 🔴 |
| Fillet Curves | `CurveFilletCurves` | Bo tròn chỗ nối giữa các đường cong. | 🔴 |
| Fillet Corners | `CurveFilletCorners` | Bo tròn các góc của đường cong. | 🔴 |
| Chamfer Curves | `CurveChamferCurves` | Vát góc giữa các đường cong. | 🔴 |
| Create UVCurves | `CurveCurveFromObjectsCreateUVCurves` | Tạo đường biểu diễn trong miền UV của mặt. | 🔴 |
| Apply UVCurves | `CurveCurveFromObjectsApplyUVCurves` | Ánh xạ đường UV lên mặt. | 🔴 |
| Through Points | `CurveFreeFormThroughPoints` | Tạo đường cong đi qua các điểm. | 🔴 |
| Extend Curve | `CurveExtendCurveExtendCurve` | Kéo dài đường cong. | 🔴 |
| Polygon Center Radius | `CurvePolygonCenterRadius` | Vẽ đa giác từ tâm và bán kính. | 🔴 |
| Spiral | `CurveSpiral` | Tạo đường xoắn ốc phẳng. | 🔴 |
| Helix | `CurveHelix` | Tạo đường xoắn lò xo trong không gian. | 🔴 |
| Single Point | `CurvePointObjectSinglePoint` | Tạo một đối tượng điểm. | 🔴 |
| Mark Curve Start | `CurvePointObjectMarkCurveStart` | Đánh dấu điểm đầu đường cong. | 🔴 |
| Mark Curve End | `CurvePointObjectMarkCurveEnd` | Đánh dấu điểm cuối đường cong. | 🔴 |
| Adjust Closed Curve Seam | `CurveCurveEditToolsAdjustClosedCurveSeam` | Điều chỉnh seam của đường cong kín. | 🔴 |
| Continue Interp Curve | `CurveFreeFormContinueInterpCurve` | Tiếp tục một đường cong nội suy. | 🔴 |
| Curve Divide Curve | `OthersCurveDivideCurve` | Chia đường cong thành các đoạn hoặc điểm. | 🔴 |
| Curve Boolean | `CurveCurveEditToolsCurveBoolean` | Thực hiện Boolean giữa các vùng đường cong. | 🔴 |
| Tween Curves | `CurveTweenCurves` | Tạo các đường trung gian giữa hai đường cong. | 🔴 |
| Duplicate Edge | `CurveCurveFromObjectsDuplicateEdge` | Sao chép cạnh thành đường cong riêng. | 🔴 |
| Duplicate Border | `CurveCurveFromObjectsDuplicateBorder` | Sao chép biên mặt thành đường cong. | 🔴 |
| Control Points | `CurveFreeFormControlPoints` | Tạo đường cong bằng control points. | 🔴 |
| Continue Curve | `CurveFreeFormContinueCurve` | Tiếp tục đường cong đang có. | 🔴 |
| GVExtract Isocurve | `CurveGVExtractIsocurve` | Trích isocurve theo workflow GV. | 🔴 |
| Cross Section Profiles | `CurveCrossSectionProfiles` | Tạo các profile mặt cắt. | 🔴 |
| Sketch | `CurveFreeFormSketch` | Vẽ đường cong tự do bằng chuột. | 🔴 |
| Sketchon Surface | `CurveFreeFormSketchonSurface` | Vẽ đường tự do trên mặt. | 🔴 |
| Sketchon Polygon Mesh | `CurveFreeFormSketchonPolygonMesh` | Vẽ đường tự do trên mesh. | 🔴 |
| Polyline On Surface | `BuilderPolylineOnSurface` | Vẽ polyline trên mặt. | 🔴 |
| Interpolateon Surface | `CurveFreeFormInterpolateonSurface` | Nội suy đường cong trên mặt. | 🔴 |
| Convert Curve To Lines | `CurveConvertCurveToLines` | Chuyển đường cong thành các đoạn thẳng. | 🔴 |
| Section | `CurveCurveFromObjectsSection` | Tạo đường mặt cắt từ đối tượng. | 🔴 |
| Match | `CurveCurveEditToolsMatch` | Khớp đầu đường cong theo điều kiện tiếp xúc/liên tục. | 🔴 |
| Silhouette | `CurveCurveFromObjectsSilhouette` | Trích đường silhouette theo hướng nhìn. | 🔴 |
| Extract Wireframe | `CurveCurveFromObjectsExtractWireframe` | Trích mạng đường wireframe từ đối tượng. | 🔴 |
| Soft Edit | `CurveCurveEditToolsSoftEdit` | Sửa đường cong với ảnh hưởng mềm. | 🔴 |
| Offset Crv Normal To Surface | `CurveOffsetOffsetCrvNormalToSurface` | Offset đường cong theo pháp tuyến mặt. | 🔴 |
| Arc Blend | `CurveBlendCurvesArcBlend` | Nối đường cong bằng arc blend. | 🔴 |
| Intersect Two Sets | `CurveCurveFromObjectsIntersectTwoSets` | Tạo giao tuyến giữa hai tập đối tượng. | 🔴 |

#### Surface

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Trạng thái |
|---|---|---|---|
| Sweep1 Rail | `SurfaceSweepSweep1Rail` | Quét profile theo một rail để tạo mặt. | 🔴 |
| Sweep2 Rails | `SurfaceSweepSweep2Rails` | Quét profile theo hai rail để tạo mặt. | 🔴 |
| Profile Sweep | `SurfaceSweepProfileSweep` | Quét mặt từ profile theo workflow Profile Sweep. | 🔴 |
| Planar Curves | `SurfacePlanarCurves` | Tạo mặt phẳng từ các đường biên đồng phẳng. | 🔴 |
| Plane Cornerto Corner | `SurfacePlaneCornertoCorner` | Tạo mặt phẳng bằng hai góc. | 🔴 |
| Surface Rebuild | `OthersSurfaceRebuild` | Dựng lại mặt với cấu trúc control points mới. | 🔴 |
| Blend Surface | `SurfaceBlendSurface` | Tạo mặt chuyển tiếp giữa các mặt. | 🔴 |
| Variable Blend Surfaces | `SurfaceVariableFilletBlendChamferVariableBlendSurfaces` | Tạo blend giữa các mặt với tham số biến thiên. | 🔴 |
| Loft | `SurfaceLoft` | Tạo mặt loft qua các tiết diện. | 🔴 |
| Curve Network | `SurfaceCurveNetwork` | Tạo mặt từ mạng đường cong. | 🔴 |
| Shrink Trimmed Surface | `SurfaceSurfaceEditToolsShrinkTrimmedSurface` | Thu gọn miền nền của mặt đã trim. | 🔴 |
| Patch | `SurfacePatch` | Tạo mặt patch phủ vùng biên. | 🔴 |
| Edge Curves | `SurfaceEdgeCurves` | Tạo mặt từ các đường biên. | 🔴 |
| Untrim | `SurfaceSurfaceEditToolsUntrim` | Khôi phục phần nền chưa trim của mặt. | 🔴 |
| GVDExtrude All | `SurfaceGVDExtrudeAll` | Extrude các thành phần theo workflow GVD. | 🔴 |
| Revolve | `SurfaceRevolve` | Tạo mặt tròn xoay quanh trục. | 🔴 |
| Rail Revolve | `SurfaceRailRevolve` | Tạo mặt xoay theo rail. | 🔴 |
| Four Profile Sweep | `BuilderFourProfileSweep` | Tạo mặt quét với bốn profile. | 🔴 |
| Sweep Multi Rail | `SurfaceSweepSweepMultiRail` | Tạo mặt quét theo nhiều rail. | 🔴 |
| Tween Surfaces | `SurfaceTweenSurfaces` | Tạo các mặt trung gian. | 🔴 |
| Fillet Surfaces | `SurfaceFilletSurfaces` | Bo tròn giữa các mặt. | 🔴 |
| Variable Fillet Surfaces | `SurfaceVariableFilletBlendChamferVariableFilletSurfaces` | Bo mặt với bán kính biến thiên. | 🔴 |
| Chamfer Surfaces | `SurfaceChamferSurfaces` | Vát nối giữa các mặt. | 🔴 |
| Variable Chamfer Surfaces | `SurfaceVariableFilletBlendChamferVariableChamferSurfaces` | Vát mặt với kích thước biến thiên. | 🔴 |
| Offset Surface | `SurfaceOffsetSurface` | Offset mặt theo khoảng cách. | 🔴 |
| Variable Offset Surface | `SurfaceVariableOffsetSurface` | Offset mặt với khoảng cách biến thiên. | 🔴 |
| Merge | `SurfaceSurfaceEditToolsMerge` | Gộp các mặt tương thích. | 🔴 |
| Match | `SurfaceSurfaceEditToolsMatch` | Khớp mặt theo điều kiện liên tục. | 🔴 |
| Drape | `SurfaceDrape` | Tạo mặt phủ drape trên hình học. | 🔴 |
| Heightfieldfrom Image | `SurfaceHeightfieldfromImage` | Tạo mặt độ cao từ ảnh. | 🔴 |
| Splitat Isocurve | `SurfaceSurfaceEditToolsSplitatIsocurve` | Chia mặt tại đường đẳng tham số. | 🔴 |
| Extend Surface | `SurfaceExtendSurface` | Kéo dài mặt. | 🔴 |
| Extrude Curve Normal To Surface | `SurfaceExtrudeCurveNormalToSurface` | Extrude đường cong theo pháp tuyến mặt. | 🔴 |
| Unroll Developable Srf | `SurfaceUnrollDevelopableSrf` | Trải phẳng mặt có thể khai triển. | 🔴 |
| Smash | `SurfaceSmash` | Trải phẳng mặt theo workflow Smash. | 🔴 |
| Soft Edit | `SurfaceSurfaceEditToolsSoftEdit` | Sửa mặt với vùng ảnh hưởng mềm. | 🔴 |
| Plane Cutting Plane | `SurfacePlaneCuttingPlane` | Tạo mặt phẳng dùng để cắt. | 🔴 |
| Adjust Closed Surface Seam | `SurfaceSurfaceEditToolsAdjustClosedSurfaceSeam` | Điều chỉnh seam của mặt kín. | 🔴 |
| Set Surface Tangent | `SurfaceSurfaceEditToolsSetSurfaceTangent` | Thiết lập điều kiện tiếp tuyến của mặt. | 🔴 |
| Refitto Tolerance | `SurfaceSurfaceEditToolsRefittoTolerance` | Fit lại mặt theo dung sai. | 🔴 |

#### Solid

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Trạng thái |
|---|---|---|---|
| Union | `SolidUnion` | Hợp các khối bằng Boolean. | 🔴 |
| Difference | `SolidDifference` | Trừ khối cắt khỏi khối đích. | 🔴 |
| Intersection | `SolidIntersection` | Lấy phần giao của các khối. | 🔴 |
| Boolean Two Objects | `SolidBooleanTwoObjects` | Thực hiện Boolean theo workflow hai đối tượng. | 🔴 |
| Cap Planar Holes | `SolidCapPlanarHoles` | Đóng các lỗ đồng phẳng để tạo khối kín. | 🔴 |
| Extract Surface | `SolidExtractSurface` | Trích một mặt khỏi khối hoặc polysurface. | 🔴 |
| Fillet Edge | `SolidFilletEdgeFilletEdge` | Bo tròn các cạnh khối. | 🔴 |
| Text | `SolidText` | Tạo hình học chữ dạng khối. | 🔴 |
| Extrude Planar Curve Straight | `SolidExtrudePlanarCurveStraight` | Extrude đường kín đồng phẳng theo phương thẳng. | 🔴 |
| GVDAll Extrude | `SolidGVDAllExtrude` | Extrude theo workflow GVD. | 🔴 |
| Pipe | `SolidPipe` | Tạo ống theo đường dẫn. | 🔴 |
| Box Cornerto Corner Height | `SolidBoxCornertoCornerHeight` | Tạo hộp từ hai góc đáy và chiều cao. | 🔴 |
| Sphere Center Radius | `SolidSphereCenterRadius` | Tạo cầu từ tâm và bán kính. | 🔴 |
| Ellipsoid From Center | `SolidEllipsoidFromCenter` | Tạo ellipsoid từ tâm. | 🔴 |
| Torus | `SolidTorus` | Tạo khối xuyến. | 🔴 |
| Cylinder | `SolidCylinder` | Tạo hình trụ. | 🔴 |
| Tube | `SolidTube` | Tạo ống trụ rỗng. | 🔴 |
| Pyramid | `SolidPyramid` | Tạo hình chóp. | 🔴 |
| Cone | `SolidCone` | Tạo hình nón. | 🔴 |
| Truncated Cone | `SolidTruncatedCone` | Tạo hình nón cụt. | 🔴 |
| Boss | `SolidBoss` | Tạo phần nhô boss trên khối. | 🔴 |
| Rib | `SolidRib` | Tạo gân tăng cứng rib. | 🔴 |
| Slab | `SolidSlab` | Tạo khối slab từ profile. | 🔴 |
| Make Hole | `SolidSolidEditToolsHolesMakeHole` | Tạo lỗ trên khối. | 🔴 |
| Array Hole | `SolidSolidEditToolsHolesArrayHole` | Tạo mảng lỗ theo phương thẳng. | 🔴 |
| Array Hole Polar | `SolidSolidEditToolsHolesArrayHolePolar` | Tạo mảng lỗ quanh trục. | 🔴 |
| Move Hole | `SolidSolidEditToolsHolesMoveHole` | Di chuyển lỗ trên khối. | 🔴 |
| Move Face | `SolidSolidEditToolsFacesMoveFace` | Di chuyển một mặt của khối. | 🔴 |
| Shell | `SolidShell` | Tạo vỏ rỗng theo bề dày. | 🔴 |
| Pt On | `SolidPtOn` | Bật các điểm điều khiển của khối. | 🔴 |

#### Transform

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Trạng thái |
|---|---|---|---|
| Scale3 D | `TransformScaleScale3D` | Scale đối tượng theo ba chiều. | 🔴 |
| Scale2 D | `TransformScaleScale2D` | Scale đối tượng trong hai chiều. | 🔴 |
| Scale1 D | `TransformScaleScale1D` | Scale đối tượng theo một chiều. | 🔴 |
| Non Uniform Scale | `TransformScaleNonUniformScale` | Scale với hệ số khác nhau trên các trục. | 🔴 |
| Orient2 Points | `TransformOrient2Points` | Định hướng đối tượng bằng hai điểm. | 🔴 |
| Orient3 Points | `TransformOrient3Points` | Định hướng đối tượng bằng ba điểm. | 🔴 |
| Rotate3 D | `TransformRotate3D` | Xoay đối tượng quanh trục 3D. | 🔴 |
| Bend | `TransformBend` | Uốn cong đối tượng. | 🔴 |
| Polar | `TransformArrayPolar` | Tạo mảng đối tượng quanh tâm hoặc trục. | 🔴 |
| Projectto CPlane | `TransformProjecttoCPlane` | Chiếu đối tượng về construction plane. | 🔴 |
| Smart Flow | `gvSmartFlow` | Biến dạng đối tượng theo workflow Smart Flow. | 🔴 |
| Smart Flow Rigid | `gvSmartFlowRigid` | Flow đối tượng theo chế độ giữ cứng từng phần. | 🔴 |
| Smart Flow Break | `gvSmartFlowBreak` | Tách quan hệ của Smart Flow. | 🔴 |
| Smart Flow Pull Down Curves | `gvSmartFlowPullDownCurves` | Kéo đường cong xuống theo workflow Smart Flow. | 🔴 |
| Smart Pattern | `TransformSmartPattern` | Tạo pattern theo workflow Smart Pattern. | 🔴 |
| Orient On Surface | `TransformOrientOnSurface` | Định hướng đối tượng trên mặt. | 🔴 |
| Orient Perpendicularto Curve | `TransformOrientPerpendiculartoCurve` | Định hướng đối tượng vuông góc đường cong. | 🔴 |
| Symmetry | `TransformSymmetry` | Thiết lập đối xứng của đối tượng. | 🔴 |
| Solid Pt On | `TransformSolidPtOn` | Bật các điểm điều khiển của khối. | 🔴 |
| Cage Edit | `TransformCageEditingCageEdit` | Biến dạng đối tượng bằng lồng cage. | 🔴 |
| Along Curve | `TransformArrayAlongCurve` | Tạo mảng dọc đường cong. | 🔴 |
| Array Linear | `TransformArrayArrayLinear` | Tạo mảng theo phương thẳng. | 🔴 |
| Along Surface | `TransformArrayAlongSurface` | Tạo mảng trên mặt. | 🔴 |
| Along Curveon Surface | `TransformArrayAlongCurveonSurface` | Tạo mảng dọc đường cong nằm trên mặt. | 🔴 |
| Rectangular | `TransformArrayRectangular` | Tạo mảng hình chữ nhật. | 🔴 |
| Flow Along Curve | `TransformFlowAlongCurve` | Biến dạng đối tượng dọc đường cong. | 🔴 |
| Flow Along Surface | `TransformFlowAlongSurface` | Biến dạng đối tượng theo mặt. | 🔴 |
| Splop | `TransformSplop` | Đặt và biến dạng đối tượng lên mặt theo workflow Splop. | 🔴 |
| Set Points | `TransformSetPoints` | Đặt tọa độ các điểm theo phương được chọn. | 🔴 |
| Scale By Plane | `TransformScaleScaleByPlane` | Scale đối tượng theo mặt phẳng. | 🔴 |
| Stretch | `TransformStretch` | Kéo giãn một vùng đối tượng. | 🔴 |
| Taper | `TransformTaper` | Tạo biến dạng thuôn. | 🔴 |
| Shear | `TransformShear` | Tạo biến dạng xiên shear. | 🔴 |
| Twist | `TransformTwist` | Xoắn đối tượng quanh trục. | 🔴 |
| Maelstrom | `TransformMaelstrom` | Tạo biến dạng xoáy Maelstrom. | 🔴 |
| Make Down Facing | `BuilderMakeDownFacing` | Định hướng đối tượng quay xuống. | 🔴 |
| Orient Remapto CPlane | `TransformOrientRemaptoCPlane` | Chuyển đối tượng giữa các construction plane. | 🔴 |
| Move UVN | `TransformMoveUVN` | Di chuyển theo hệ tọa độ UVN. | 🔴 |
| Soft Move | `TransformSoftMove` | Di chuyển với ảnh hưởng mềm. | 🔴 |
| Create Cage | `TransformCageEditingCreateCage` | Tạo lồng điều khiển cage. | 🔴 |

#### Clayoo / SubD

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Trạng thái |
|---|---|---|---|
| Item | `ClayooEditItem` | Chỉnh sửa thành phần SubD/Clayoo. | 🔴 |
| Divide | `ClayooEditDivide` | Chia nhỏ các thành phần SubD. | 🔴 |
| Split Sides | `ClayooEditSplitSides` | Tách các cạnh bên của thành phần SubD. | 🔴 |
| Inset | `ClayooEditInset` | Tạo vùng inset trên mặt SubD. | 🔴 |
| Knife | `ClayooEditKnife` | Cắt topology bằng knife. | 🔴 |
| Offset | `ClayooEditOffset` | Offset thành phần SubD. | 🔴 |
| Extrude | `ClayooEditExtrude` | Extrude thành phần SubD. | 🔴 |
| Bridge | `ClayooEditBridge` | Nối các biên SubD bằng bridge. | 🔴 |
| Append Face | `ClayooCreationAppendFace` | Thêm một mặt vào topology SubD. | 🔴 |
| Pipe | `ClayooCreationPipe` | Tạo ống theo workflow Clayoo. | 🔴 |
| Clayoo Bezel | `ClayooCreationClayooBezel` | Tạo bezel theo workflow Clayoo. | 🔴 |
| Ring | `ClayooCreationRing` | Tạo nhẫn bằng workflow SubD/Clayoo. | 🔴 |
| Signet Ring | `ClayooCreationSignetRing` | Tạo nhẫn signet bằng workflow SubD/Clayoo. | 🔴 |
| Crease | `ClayooEditCrease` | Đặt cạnh sắc crease của SubD. | 🔴 |
| Flip Normals | `ClayooEditFlipNormals` | Đảo pháp tuyến. | 🔴 |
| Project To Plane | `ClayooEditProjectToPlane` | Chiếu thành phần về mặt phẳng. | 🔴 |
| Collapse | `ClayooEditCollapse` | Thu gộp thành phần topology. | 🔴 |
| Match | `ClayooEditMatch` | Khớp các thành phần SubD. | 🔴 |
| Weld | `ClayooEditWeld` | Hàn các đỉnh hoặc biên. | 🔴 |
| Symmetry | `ClayooEditSymmetry` | Thiết lập đối xứng SubD. | 🔴 |
| Box | `ClayooPrimitivesBox` | Tạo hộp SubD. | 🔴 |
| Sphere | `ClayooPrimitivesSphere` | Tạo cầu SubD. | 🔴 |
| Cylinder | `ClayooPrimitivesCylinder` | Tạo trụ SubD. | 🔴 |
| Cone | `ClayooPrimitivesCone` | Tạo nón SubD. | 🔴 |
| Torus | `ClayooPrimitivesTorus` | Tạo xuyến SubD. | 🔴 |
| Plane | `ClayooPrimitivesPlane` | Tạo mặt phẳng SubD. | 🔴 |
| Sweep1 Rail | `ClayooCreationSweep1Rail` | Quét SubD theo một rail. | 🔴 |
| Sweep2 Rails | `ClayooCreationSweep2Rails` | Quét SubD theo hai rail. | 🔴 |
| Loft | `ClayooCreationLoft` | Tạo loft SubD. | 🔴 |
| Wrap | `ClayooEditWrap` | Bọc đối tượng theo workflow Wrap. | 🔴 |
| Revolve | `ClayooCreationRevolve` | Tạo SubD tròn xoay. | 🔴 |
| From Curve | `ClayooCreationFromCurve` | Tạo SubD từ đường cong. | 🔴 |
| From2 Curves | `ClayooCreationFrom2Curves` | Tạo SubD từ hai đường cong. | 🔴 |
| Fill | `ClayooEditFill` | Lấp vùng biên SubD. | 🔴 |
| Shell | `ClayooEditShell` | Tạo vỏ có bề dày từ SubD. | 🔴 |
| Extract | `ClayooEditExtract` | Trích xuất thành phần SubD. | 🔴 |
| From Surface | `ClayooCreationFromSurface` | Tạo SubD từ surface. | 🔴 |
| From TSplines | `ClayooCreationFromTSplines` | Chuyển từ dữ liệu T-Splines sang workflow SubD. | 🔴 |
| From Mesh | `ClayooCreationFromMesh` | Tạo SubD từ mesh. | 🔴 |
| To Nurbs | `ClayooCreationToNurbs` | Chuyển SubD sang NURBS. | 🔴 |
| Selection Sets | `ClayooSelectionSelectionSets` | Quản lý các tập selection SubD. | 🔴 |
| Select Ring | `ClayooSelectionSelectRing` | Chọn dải cạnh ring. | 🔴 |
| Select Loop | `ClayooSelectionSelectLoop` | Chọn chuỗi cạnh loop. | 🔴 |
| Select UV | `ClayooSelectUV` | Chọn theo tọa độ UV. | 🔴 |
| Select All | `ClayooSelectionSelectAll` | Chọn tất cả thành phần SubD. | 🔴 |
| Select None | `ClayooSelectionSelectNone` | Bỏ chọn các thành phần SubD. | 🔴 |
| Paint Selection | `ClayooSelectionPaintSelection` | Chọn thành phần bằng thao tác tô. | 🔴 |
| Grow Selection | `ClayooSelectionGrowSelection` | Mở rộng vùng selection. | 🔴 |
| Shrink | `ClayooSelectionShrink` | Thu nhỏ vùng selection. | 🔴 |
| Invert Selection | `ClayooSelectionInvertSelection` | Đảo selection. | 🔴 |
| Select Creased | `ClayooSelectionSelectCreased` | Chọn các cạnh crease. | 🔴 |
| Select Naked | `ClayooSelectionSelectNaked` | Chọn các biên hở. | 🔴 |
| Select Coplanar | `ClayooSelectionSelectCoplanar` | Chọn các thành phần đồng phẳng. | 🔴 |
| Subdivision Level | `ClayooEditSubdivisionLevel` | Thay đổi mức subdivision. | 🔴 |
| Clayoo Library | `ClayooCreationClayooLibrary` | Mở thư viện đối tượng Clayoo/SubD. | 🔴 |
| Set Plane | `ClayooSelectionSetPlane` | Đặt mặt phẳng thao tác selection. | 🔴 |
| Reset Plane | `ClayooSelectionResetPlane` | Đặt lại mặt phẳng thao tác. | 🔴 |
| Create Isocurves | `ClayooCreateIsocurves` | Tạo isocurves theo workflow Clayoo. | 🔴 |

#### Emboss

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Trạng thái |
|---|---|---|---|
| Matrix Art | `EmbossMatrixArt` | Tạo relief theo workflow Matrix Art. | 🔴 |
| Heightfield Image | `EmbossHeightfieldImage` | Tạo relief độ cao từ ảnh. | 🔴 |
| Texture Builder | `TextureBuilder` | Tạo texture hình học. | 🔴 |
| Clay Emboss | `EmbossClayEmboss` | Tạo emboss theo workflow Clay Emboss. | 🔴 |
| Sculpt | `EmbossSculpt` | Điêu khắc hình học. | 🔴 |
| Decimator | `EmbossDecimator` | Giảm độ phức tạp mesh của relief. | 🔴 |
| Clay Emboss Resources | `EmbossClayEmbossResources` | Quản lý tài nguyên cho workflow Clay Emboss. | 🔴 |

#### Builder

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Trạng thái |
|---|---|---|---|
| Eternity | `BuilderEternity` | Tạo nhẫn eternity theo tham số. | 🔴 |
| Signet | `BuilderSignet` | Tạo nhẫn signet theo builder. | 🔴 |
| Signet Ring | `ClayooCreationSignetRing` | Tạo nhẫn signet bằng workflow SubD/Clayoo. | 🔴 |
| Ring | `ClayooCreationRing` | Tạo nhẫn bằng workflow SubD/Clayoo. | 🔴 |
| Text On Curve | `TextOnCurve` | Đặt chữ dọc đường cong. | 🔴 |
| Raised | `BuilderRaised` | Tạo chi tiết raised theo builder; tham số nghiệp vụ chưa xác minh. | 🔴 |
| Award Ring Builder | `BuilderAwardRingBuilder` | Tạo nhẫn award theo builder. | 🔴 |
| Rope | `Builder_Rope` | Tạo cấu trúc dây xoắn rope. | 🔴 |
| Jump | `BuilderJump` | Tạo vòng nối jump ring. | 🔴 |
| Free Form Knot | `BuilderFreeFormKnot` | Tạo nút thắt tự do. | 🔴 |
| Knot | `BuilderKnot` | Tạo nút thắt theo builder. | 🔴 |
| Pattern Builder | `BuilderPatternBuilder` | Tạo pattern theo builder. | 🔴 |
| Nautilus Builder | `BuilderNautilusBuilder` | Tạo hình xoắn Nautilus theo builder. | 🔴 |
| Mill Work | `BuilderMillWork` | Workflow Mill Work; quy tắc gia công cụ thể chưa xác minh. | 🔴 |
| Mill Area | `BuilderMillArea` | Workflow Mill Area; quy tắc vùng gia công chưa xác minh. | 🔴 |
| Mill Work C | `BuilderMillWorkC` | Biến thể Mill Work C; ý nghĩa chi tiết chưa xác minh. | 🔴 |

#### Tools

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Trạng thái |
|---|---|---|---|
| Ring Rail | `BuilderRingRail` | Tạo đường dẫn cơ sở cho thân nhẫn. | 🔴 |
| Profile Placer | `BuilderProfilePlacer` | Đặt profile trên rail. | 🔴 |
| Outside Ring Rail | `BuilderOutsideRingRail` | Tạo rail ngoài của nhẫn. | 🔴 |
| Custom Rail | `BuilderCustomRail` | Tạo rail tùy chỉnh. | 🔴 |
| Four Profile | `BuilderFourProfile` | Tạo bộ bốn profile cho thiết kế. | 🔴 |
| Metal Weights | `BuilderMetalWeights` | Tính khối lượng kim loại theo mẫu và vật liệu. | 🔴 |
| Resize | `BuilderResize` | Thay đổi size nhẫn. | 🔴 |
| Surface Pullback | `BuilderSurfacePullback` | Kéo đường cong về mặt theo builder. | 🔴 |
| Bermark Library | `BuilderBermarkLibrary` | Mở thư viện Bermark trong catalog. | 🔴 |
| Stuller Findings Library | `BuilderStullerFindingsLibrary` | Mở thư viện findings Stuller trong catalog. | 🔴 |
| Smart Target | `gvSmartTarget` | Đặt hoặc sử dụng Smart Target. | 🔴 |
| Smart Target Add | `gvSmartTargetAdd` | Thêm Smart Target. | 🔴 |
| Smart Target On Crv End | `gvSmartTargetOnCrvEnd` | Đặt Smart Target ở đầu đường cong. | 🔴 |
| Smart Blend | `gvSmartBlend` | Tạo chuyển tiếp theo workflow Smart Blend. | 🔴 |
| Smart MSR | `gvSmartMSR` | Workflow Smart MSR; thuật toán cụ thể chưa xác minh. | 🔴 |
| Join History | `gvJoinHistory` | Ghép đối tượng có quan hệ history. | 🔴 |
| Image Trace | `BuilderImageTrace` | Trace đường cong từ ảnh. | 🔴 |
| Object Checker | `BuilderObjectChecker` | Kiểm tra đối tượng trước xuất hoặc sản xuất. | 🔴 |
| Mesh Repair | `BuilderMeshRepair` | Sửa mesh theo builder. | 🔴 |
| 3 d Printing | `Builder3dPrinting` | Chuẩn bị mẫu cho in 3D. | 🔴 |
| Profile | `BuilderProfile` | Tạo profile theo builder. | 🔴 |
| Profile Merge | `BuilderProfileMerge` | Gộp các profile. | 🔴 |
| Profile End Cap | `BuilderProfileEndCap` | Đóng đầu profile bằng end cap. | 🔴 |
| Custom Rail Advanced | `BuilderCustomRailAdvanced` | Tạo rail tùy chỉnh nâng cao. | 🔴 |
| Object On Crv | `BuilderObjectOnCrv` | Đặt đối tượng dọc đường cong. | 🔴 |
| Surface Inset | `BuilderSurfaceInset` | Tạo inset trên mặt. | 🔴 |
| Mesh Reducer | `BuilderMeshReducer` | Giảm số phần tử mesh. | 🔴 |
| Mesh | `BuilderMesh` | Tạo mesh theo builder. | 🔴 |
| Curve Transform Tool | `BuilderCurveTransformTool` | Biến đổi đường cong bằng công cụ builder. | 🔴 |
| Center Line | `BuilderCenterLine` | Tạo đường tâm. | 🔴 |
| Boolean Builder | `BuilderBooleanBuilder` | Thực hiện Boolean theo builder. | 🔴 |

#### Gems

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Trạng thái |
|---|---|---|---|
| Gem Loader | `BuilderGemLoader` | Nạp mẫu đá từ thư viện gem. | 🔴 |
| Gem On Crv | `BuilderGemOnCrv` | Đặt đá dọc đường cong. | 🔴 |
| Gem On Crv Advanced | `BuilderGemOnCrvAdvanced` | Đặt đá dọc đường cong với tùy chọn nâng cao. | 🔴 |
| Gem Count On Crv | `BuilderGemCountOnCrv` | Đặt hoặc tính số đá trên đường cong. | 🔴 |
| Gem List On Crv | `BuilderGemListOnCrv` | Đặt danh sách đá trên đường cong. | 🔴 |
| Gem On Crv Multi | `BuilderGemOnCrvMulti` | Đặt nhiều loại đá dọc đường cong. | 🔴 |
| Gem Profile | `BuilderGemProfile` | Tạo profile của đá. | 🔴 |
| Gem Guides | `BuilderGemGuides` | Tạo đường hướng dẫn đặt đá. | 🔴 |
| Orient To Gem | `BuilderOrientToGem` | Định hướng đối tượng theo đá. | 🔴 |
| Halo | `BuilderHalo` | Tạo bố trí đá halo. | 🔴 |
| Gem On Surface | `BuilderGemOnSurface` | Đặt đá trên mặt. | 🔴 |
| Custom Gem | `BuilderCustomGem` | Tạo đá tùy chỉnh. | 🔴 |
| Emerald | `BuilderEmerald` | Tạo đá dạng emerald. | 🔴 |
| Baguette | `BuilderBaguette` | Tạo đá dạng baguette. | 🔴 |
| Taper Bag Channel | `BuilderTaperBagChannel` | Tạo channel với đá baguette thuôn. | 🔴 |
| Taper Bag Between Two Curves | `BuilderTaperBagBetweenTwoCurves` | Đặt baguette thuôn giữa hai đường cong. | 🔴 |
| Gem Between Two Curves | `BuilderGemBetweenTwoCurves` | Đặt đá giữa hai đường cong. | 🔴 |
| Cluster Builder Old | `BuilderClusterBuilderOld` | Tạo cụm đá theo workflow Cluster cũ. | 🔴 |
| Gem Reporter | `BuilderGemReporter` | Tạo báo cáo đá. | 🔴 |
| Gem Map | `BuilderGemMap` | Tạo bản đồ bố trí đá. | 🔴 |
| Pave Dialog | `BuilderPaveDialog` | Mở tùy chọn bố trí pavé. | 🔴 |
| Pave Builder | `BuilderPaveBuilder` | Tạo bố trí pavé. | 🔴 |
| Pave Azure Builder | `BuilderPaveAzureBuilder` | Tạo azure cho pavé. | 🔴 |
| Pave Prong Builder | `BuilderPaveProngBuilder` | Tạo chấu cho pavé. | 🔴 |
| Gem Springs | `BuilderGemSprings` | Workflow Gem Springs; quy tắc chi tiết chưa xác minh. | 🔴 |
| Gem Follow | `BuilderGemFollow` | Cho đá theo đối tượng hoặc đường tham chiếu. | 🔴 |
| Gem Control | `BuilderGemControl` | Điều khiển các tham số bố trí đá. | 🔴 |
| Match Attributes | `OthersMatchAttributes` | Sao chép thuộc tính giữa các đối tượng. | 🔴 |
| Save Style | `OthersSaveStyle` | Lưu style đối tượng. | 🔴 |
| Load Style | `OthersLoadStyle` | Nạp style đối tượng. | 🔴 |
| GVDGem Flow | `BuilderGVDGemFlow` | Flow đá theo workflow GVD. | 🔴 |
| GVDGem Splop | `BuilderGVDGemSplop` | Đặt đá lên mặt theo workflow Gem Splop. | 🔴 |
| Pave Sphere | `BuilderPaveSphere` | Tạo bố trí pavé trên cầu. | 🔴 |
| Gem Update | `BuilderGemUpdate` | Cập nhật bố trí hoặc tham số đá. | 🔴 |
| Gem Positioner | `BuilderGemPositioner` | Điều chỉnh vị trí đá. | 🔴 |

#### Settings — ổ đá và chấu

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Trạng thái |
|---|---|---|---|
| Head | `BuilderHead` | Tạo ổ hoặc đầu chấu giữ đá. | 🔴 |
| Bezel | `BuilderBezel` | Tạo ổ bezel. | 🔴 |
| Clayoo Bezel | `ClayooCreationClayooBezel` | Tạo bezel theo workflow Clayoo. | 🔴 |
| Pull To Rail | `BuilderPullToRail` | Kéo cấu trúc ổ đá về rail. | 🔴 |
| Bezel Cutter | `BuilderBezelCutter` | Tạo khối cắt cho bezel. | 🔴 |
| Prong Adder | `BuilderProngAdder` | Thêm chấu giữ đá. | 🔴 |
| Prong Editor | `BuilderProngEditor` | Chỉnh sửa chấu. | 🔴 |
| Prong On Surface | `BuilderProngOnSurface` | Đặt chấu trên mặt. | 🔴 |
| Bead On Surface | `BuilderBeadOnSurface` | Đặt bead trên mặt. | 🔴 |
| Metal Piece | `BuilderMetalPiece` | Tạo chi tiết kim loại theo builder. | 🔴 |
| Bead On Crv | `BuilderBeadOnCrv` | Đặt bead dọc đường cong. | 🔴 |
| Channel Border Curve | `BuilderChannelBorderCurve` | Tạo đường biên cho channel. | 🔴 |

#### Cutters

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Trạng thái |
|---|---|---|---|
| Gem Cutter | `BuilderGemCutter` | Tạo khối cắt chỗ đặt đá. | 🔴 |
| Gem Cutter Library | `BuilderGemCutterLibrary` | Mở thư viện gem cutters. | 🔴 |
| Channel Builder | `BuilderChannelBuilder` | Tạo channel theo builder. | 🔴 |
| Micro Prong Cutter | `BuilderMicroProngCutter` | Tạo khối cắt cho micro-prong. | 🔴 |
| Bright Cut Channel | `BuilderBrightCutChannel` | Tạo channel theo workflow bright cut. | 🔴 |
| Bright Cutter | `BuilderBrightCutter` | Tạo khối bright cutter. | 🔴 |
| Bezel Cutter | `BuilderBezelCutter` | Tạo khối cắt cho bezel. | 🔴 |
| Cutto Finger Rail | `OthersCuttoFingerRail` | Cắt đối tượng theo finger rail. | 🔴 |
| Planeand Cube Cutters | `BuilderPlaneandCubeCutters` | Tạo khối/mặt phẳng dùng để cắt. | 🔴 |
| Quad Flip | `BuilderQuadFlip` | Workflow Quad Flip; ý nghĩa hình học chi tiết chưa xác minh. | 🔴 |
| Boolean Builder | `BuilderBooleanBuilder` | Thực hiện Boolean theo builder. | 🔴 |

#### Render

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Trạng thái |
|---|---|---|---|
| Render Vray Render Styler | `BuilderRenderVrayRenderStyler` | Thiết lập render style theo workflow V-Ray trong catalog. | 🔴 |
| Create Spotlight | `RenderCreateSpotlight` | Tạo đèn spotlight. | 🔴 |
| Create Point Light | `RenderCreatePointLight` | Tạo đèn điểm. | 🔴 |
| Create Directional Light | `RenderCreateDirectionalLight` | Tạo đèn định hướng. | 🔴 |
| Set Spot Lightto View | `RenderSetSpotLighttoView` | Căn spotlight theo hướng nhìn. | 🔴 |
| Create Rectangular Light | `RenderCreateRectangularLight` | Tạo đèn hình chữ nhật. | 🔴 |
| Create Linear Light | `RenderCreateLinearLight` | Tạo đèn dạng đường. | 🔴 |
| Props | `BuilderProps` | Thêm props cho scene. | 🔴 |
| Set Renderer | `RenderSetRenderer` | Chọn render engine. | 🔴 |
| Surface Environment Map | `AnalyzeSurfaceEnvironmentMap` | Hiển thị environment map để kiểm tra mặt. | 🔴 |
| Apply Edge Softening | `RenderEffectsApplyEdgeSoftening` | Áp dụng hiệu ứng làm mềm cạnh khi render. | 🔴 |
| Animation Builder | `BuilderAnimationBuilder` | Tạo animation theo builder. | 🔴 |
| Animation Builder Advanced | `BuilderAnimationBuilderAdvanced` | Tạo animation với tùy chọn nâng cao. | 🔴 |
| Matrix Movie Maker | `MatrixMovieMaker` | Tạo phim theo workflow Matrix Movie Maker. | 🔴 |
| Render Editor | `BuilderRenderEditor` | Mở trình chỉnh sửa render. | 🔴 |
| Batch Render | `BuilderBatchRender` | Render theo lô. | 🔴 |
| Layout Tools | `BuilderLayoutTools` | Tạo bố cục trình bày. | 🔴 |
| Four View Capture | `BuilderFourViewCapture` | Chụp bộ ảnh từ bốn view. | 🔴 |
| Render Scheduler | `BuilderRenderScheduler` | Lập lịch render theo catalog. | 🔴 |
| Alpha Erase | `gvAlphaErase` | Workflow Alpha Erase; thao tác alpha cụ thể chưa xác minh. | 🔴 |
| Save Render Window As | `RenderSaveRenderWindowAs` | Lưu ảnh trong cửa sổ render. | 🔴 |
| Web Viewer | `WebViewer` | Mở viewer phục vụ chia sẻ trên web. | 🔴 |
| Apply Displacement | `RenderEffectsApplyDisplacement` | Áp dụng displacement khi render. | 🔴 |

#### Core / project / selection

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Trạng thái |
|---|---|---|---|
| Undo | `Undo` | Hoàn tác thao tác của project. | 🟢 |
| Redo | `Redo` | Làm lại thao tác đã hoàn tác. | 🟢 |
| Select All | `SelectAll` | Chọn các đối tượng của project hiện hành. | 🟢 |
| Select None | `SelectNone` | Bỏ selection hiện hành. | 🟢 |
| Delete | `Delete` | Xóa các đối tượng đang chọn, hỗ trợ Undo qua host. | 🟢 |
| Project Add | `ProjectAdd` | Tạo/thêm project từ sidebar. | 🔴 |
| Project Out | `ProjectOut` | Đưa dữ liệu ra project theo catalog. | 🔴 |
| Project In | `ProjectIn` | Nạp dữ liệu vào project theo catalog. | 🔴 |
| Project Save | `ProjectSave` | Lưu project từ sidebar. | 🔴 |
| Project Delete | `ProjectDelete` | Xóa project từ sidebar. | 🔴 |
| Project Manager | `ProjectManager` | Quản lý danh sách project. | 🔴 |

#### View và display bổ sung

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Trạng thái |
|---|---|---|---|
| View Front | `ViewFront` | Chuyển camera sang hướng nhìn trước. | 🟢 |
| View Top | `ViewTop` | Chuyển camera sang hướng nhìn trên. | 🟢 |
| View Right | `ViewRight` | Chuyển camera sang hướng nhìn phải. | 🟢 |
| View Left | `ViewLeft` | Chuyển camera sang hướng nhìn trái. | 🟢 |
| View Rear | `ViewRear` | Chuyển camera sang hướng nhìn sau. | 🟢 |
| View Bottom | `ViewBottom` | Chuyển camera sang hướng nhìn dưới. | 🟢 |
| View Isometric | `ViewIsometric` | Chuyển camera sang hướng nhìn isometric. | 🟢 |
| Fit All | `FitAll` | Zoom để thấy toàn bộ đối tượng. | 🟢 |
| ViewportTabs | `InfoSettingsViewportTabsToggle` | Bật/tắt tabs viewport. | 🟢 |
| ShowGrid | `GridON` | Bật/tắt lưới viewport. | 🟢 |
| View Grid Options | `ViewGridOptions` | Thiết lập lưới viewport. | 🔴 |
| Preview Cutter ON | `PreviewCutterON` | Bật preview cutter. | 🔴 |
| Preview Shade ON | `PreviewShadeON` | Bật chế độ shaded preview. | 🔴 |
| Shade Selected Only ON | `ShadeSelectedOnlyON` | Chỉ shade các đối tượng đang chọn. | 🔴 |
| GVGem View 1 | `GVGemView_1` | Đổi hiển thị đá theo catalog. | 🔴 |
| GVSurface View 1 | `GVSurfaceView_1` | Đổi hiển thị surface theo catalog. | 🔴 |
| Dial Wireframe | `Dial_Wireframe` | Đổi viewport sang hiển thị wireframe. | 🔴 |
| Dial Shaded | `Dial_Shaded` | Đổi viewport sang hiển thị shaded. | 🔴 |
| Dial Working Shade | `Dial_Working Shade` | Đổi viewport sang working shade. | 🔴 |
| Dial Working Render | `Dial_Working Render` | Đổi viewport sang working render. | 🔴 |
| Dial Ghosted | `Dial_Ghosted` | Đổi viewport sang hiển thị ghosted. | 🔴 |
| Dial Tech Shade | `Dial_Tech Shade` | Đổi viewport sang technical shade. | 🔴 |

#### Snap / ràng buộc

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Trạng thái |
|---|---|---|---|
| Osnap E | `ToolsObjectSnapEnd` | Bắt điểm đầu hoặc cuối cạnh/đường cong. | 🟢 |
| Near | `ToolsObjectSnapNear` | Bắt điểm gần nhất trên đối tượng. | 🔴 |
| Osnap P | `ToolsObjectSnapPoint` | Bắt đối tượng điểm hoặc vertex được hỗ trợ. | 🟢 |
| Osnap M | `ToolsObjectSnapMidpoint` | Bắt điểm giữa cạnh/đường cong được hỗ trợ. | 🟢 |
| Center | `ToolsObjectSnapCenter` | Bắt tâm đường tròn hoặc cung. | 🔴 |
| Intersection | `ToolsObjectSnapIntersection` | Bắt điểm giao của đối tượng. | 🔴 |
| Perpendicular To | `ToolsObjectSnapPerpendicularTo` | Bắt điểm tạo hướng vuông góc. | 🔴 |
| Tangent To | `ToolsObjectSnapTangentTo` | Bắt điểm tạo hướng tiếp tuyến. | 🔴 |
| Quadrant | `ToolsObjectSnapQuadrant` | Bắt các điểm một phần tư đường tròn. | 🔴 |
| Knot | `ToolsObjectSnapKnot` | Bắt knot của đường cong. | 🔴 |
| Snap Between | `SnapBetween` | Bắt điểm giữa hai tham chiếu. | 🔴 |
| Snap On Surface | `SnapOnSurface` | Bắt điểm trên mặt. | 🔴 |
| Snap On Polysurface | `SnapOnPolysurface` | Bắt điểm trên polysurface. | 🔴 |
| Grid Snap ON | `GridSnapON` | Bắt điểm theo lưới. | 🔴 |
| Ortho | `OrthoSnapON` | Ràng buộc hướng Ortho trong công cụ đang hỗ trợ. | 🟢 |
| Planar Snap ON | `PlanarSnapON` | Ràng buộc thao tác theo mặt phẳng. | 🔴 |
| Project Snap ON | `ProjectSnapON` | Chiếu điểm bắt về mặt phẳng thao tác. | 🔴 |

#### Properties / history / settings / layers

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Trạng thái |
|---|---|---|---|
| Display Properties | `InfoSettingsDisplayProperties` | Thiết lập thuộc tính hiển thị. | 🔴 |
| Rhino Options | `RhinoOptions` | Mở bộ tùy chọn tương đương workflow Rhino. | 🔴 |
| GVObject Info | `GVObjectInfo` | Xem thông tin đối tượng theo workflow GV. | 🔴 |
| Properties | `ObjectProperties` | Mở inspector thuộc tính native FreeCAD. | 🟢 |
| All Object Info | `AllObjectInfo` | Xem thông tin của toàn bộ đối tượng. | 🔴 |
| Command History | `CommandHistory` | Xem lịch sử command. | 🟢 |
| Project Notes | `ProjectNotes` | Đọc và sửa ghi chú project. | 🟢 |
| Super Select | `SuperSelect` | Selection nâng cao theo catalog. | 🔴 |
| Box Edit | `InfoSettingsBoxEdit` | Chỉnh đối tượng qua hộp giới hạn. | 🔴 |
| Libraries | `InfoSettingsLibraries` | Quản lý thư viện. | 🔴 |
| Selection Filter | `InfoSettingsSelectionFilter` | Lọc loại đối tượng được chọn. | 🔴 |
| Design Report | `InfoSettingsDesignReport` | Tạo báo cáo thiết kế. | 🔴 |
| Gumball Alignment | `InfoSettingsGumballAlignment` | Căn hệ trục gizmo thao tác. | 🔴 |
| Relocate Gumball | `InfoSettingsRelocateGumball` | Di chuyển tâm gizmo thao tác. | 🔴 |
| Gumball ON | `InfoSettingsGumballON` | Bật gizmo thao tác. | 🔴 |
| Smart Targets Gumball ON | `InfoSettingsSmartTargetsGumballON` | Bật gizmo cho Smart Targets. | 🔴 |
| Rhino Smart Track ON | `RhinoSmartTrackON` | Bật tracking thông minh theo catalog. | 🔴 |
| Rhino History ON | `RhinoHistoryON` | Bật history tương đương workflow Rhino. | 🔴 |
| GVHistory Update ON | `GVHistoryUpdateON` | Bật cập nhật history GV. | 🔴 |
| GVHistory Record ON | `GVHistoryRecordON` | Bật ghi history GV. | 🔴 |
| GVClear History | `InfoSettingsGVClearHistory` | Xóa quan hệ history GV. | 🔴 |
| Layer Arrow | `LayerArrow` | Chọn layer thao tác trên sidebar. | 🔴 |
| Layer Lock | `LayerLock` | Khóa layer trên sidebar. | 🔴 |
| Layer Visibility | `LayerVisibility` | Bật/tắt visibility layer trên sidebar. | 🔴 |
| Layer Hide | `LayerHide` | Ẩn layer theo catalog. | 🔴 |
| Layer Show | `LayerShow` | Hiện layer theo catalog. | 🔴 |

#### 3DM chuyên biệt / keyboard

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Trạng thái |
|---|---|---|---|
| Import3dm | `FileImport3dm` | Nhập geometry 3DM hoặc giữ snapshot nguồn trong FCStd. | 🟡 |
| Export3dm | `FileExport3dm` | Xuất geometry được chọn thành archive Rhino5; chưa full preservation. | 🟡 |
| F6 | `F6` | Mở context menu độc lập của OM9 theo selection. | 🟢 |

`Custom` và `Reset` là nhóm điều khiển sidebar, không phải nhóm lệnh tạo CAD;
không được tính thành một lệnh hình học hoàn thành. Danh sách không liệt kê lại
Editor vì Editor dùng cùng58 mã của nhóm Curve. Các tên chỉ có trong completion
mà chưa có menu/host implementation không được mặc định xem là đã làm.

Không công bố phần trăm hoàn thành tổng thể: số icon, tên lệnh và số test không
phải cùng một đơn vị đo. Danh mục giao diện: [MainMenu.ini](Resources/menu/MainMenu.ini);
đường dispatch: [Command.cpp](Gui/Command.cpp) và [Rust catalog](rust/src/ffi.rs).

Kiểm tra theo nhóm: [workspace](tests/workspace_smoke.FCMacro),
[curve](tests/curve_smoke.FCMacro), [views](tests/core_views_smoke.FCMacro),
[view controls](tests/core_view_controls_smoke.FCMacro),
[picture frame](tests/core_picture_frame_smoke.FCMacro),
[distance](tests/core_distance_smoke.FCMacro), [angle](tests/core_angle_smoke.FCMacro),
[snap tools](tests/core_snap_point_tools_smoke.FCMacro),
[keyboard](tests/core_keyboard_smoke.FCMacro), [notes](tests/core_notes_smoke.FCMacro).

## Tiến độ tương thích openNURBS / Rhino 3DM

**Chưa full openNURBS và chưa tương đương Rhino SDK.** Thư viện được tải từ
upstream public tại commit cố định trong [OpenNURBS.cmake](cmake/OpenNURBS.cmake).
Đọc được, giữ nguyên dữ liệu nguồn và chỉnh sửa được là các mức hỗ trợ khác nhau.

| Dữ liệu / chức năng | Source public hiện tại | Phần còn thiếu |
|---|---|---|
| ✅ Points và curves | Point, line/polyline, arc/circle và rational NURBS qua native CAD conversion. | Không suy ra mọi lớp curve hoặc mọi trường hợp suy biến đều đã tương thích. |
| ✅ Surfaces / BRep | Conversion surface/BRep được hỗ trợ, trim loops, holes, shells, solids và extrusion. | Kiểm chứng toàn bộ biến thể topology/dung sai và lớp native còn lại. |
| ✅ Mesh | Triangle/quad mesh; geometry export có cơ chế giữ quad nguồn khi mesh không đổi. | Full mesh attributes và mọi dữ liệu đi kèm chưa được cam kết. |
| 🟡 Block / `ON_InstanceRef` | Geometry-only import mở rộng embedded blocks thành CAD/mesh, áp dụng transform lồng nhau, scale không đều và reflection. | Source public chưa có structural block preservation writer; geometry-only export không giữ graph definition/reference. |
| ✅ Metadata hình học cơ bản | Name, nested layer path, color, visibility, lock và chuyển đơn vị trong phạm vi geometry exchange. | Full layer/material/document dependency remapping. |
| 🟡 Preserve source data | Snapshot nguồn bất biến, hash, namespace/UUID và file đính kèm trong FCStd; giữ record chưa chỉnh sửa được. | Giữ trong snapshot không đồng nghĩa render/edit/export lại đầy đủ. [Preservation checks](tests/three_dm_preservation_smoke.FCMacro). |
| 🟡 Annotation / lights / native classes khác | Có thể giữ dữ liệu trong snapshot và record retained; geometry-only không hỗ trợ mọi lớp. | Editable/display conversion cho dimensions, leaders, hatches, point clouds, clipping objects và lớp chưa có converter. |
| ⬜ Materials / textures / render settings | Snapshot có thể giữ nguồn; geometry-only export không bảo toàn đầy đủ. | Dependency closure, tài nguyên và appearance roundtrip. |
| ⬜ History / plugin userdata | Chưa có reconstruction của Rhino history hoặc Matrix builders. | Unknown reference-bearing/plugin payload cần chính sách bảo toàn và test; không chạy plugin Rhino. |
| 🟡 Rhino 5 export | Writer geometry nhắm archive version5; có native/host test. | Chưa kiểm thử trực tiếp bằng ứng dụng Rhino5; không tuyên bố hỗ trợ đầy đủ các phiên bản mới hơn. |

### Bản phát triển đang chờ tích hợp vào public

Các phần sau đã có kết quả test ở workspace phát triển riêng nhưng **chưa có
trong source public này**, nên clone repo hiện tại chưa nhận các chức năng đó:

| Phần phát triển | Kết quả đã kiểm tra | Còn trước khi hoàn thành |
|---|---|---|
| Selected preservation writer | Giữ native UUID và dependency closure; selected block giữ definition/member/reference thay vì flatten. | Review, tích hợp và nối standard/menu/CMD export. |
| Geometry/metadata overlays | Thay BRep/mesh được hỗ trợ; name Unicode, color, visibility, lock. | Layer/dependency/member edits và CAD mới chưa có source. |
| Copy/delete và FCStd | UUID mới cho copy, geometry/placement độc lập, không hồi sinh đối tượng bị bỏ khỏi selection. | Tài liệu cũ thiếu verified baseline và merge từ archive khác nhau. |
| Structural / affine blocks | Shared/nested definitions, rigid instances, preview scale/reflection; export rigid delta ghép với ma trận native. | Mixed mesh/shear host acceptance và chỉnh member definition đầy đủ. |

Kết quả phát triển đã ghi nhận: **8 bộ native**, **23 kiểm tra structural block**,
**14 signature**, **12 preserved export**, **19 preservation regression** pass.
Đây là bằng chứng từ workspace phát triển, không phải kết quả tái chạy trên
public snapshot. Hai mẫu nhẫn riêng đã dùng để test geometry; preservation export
của chúng còn bị chặn bởi plugin payload chưa hiểu. Mẫu riêng không được đưa vào repo.

Public snapshot có bằng chứng audit giao diện **3093/3093 checks** và **68 Rust
tests** trong [progress ledger](docs/openmatrix9-progress.json) và
[publication checks](docs/public-source-checks.json); audit icon không chứng minh
hoàn thành toàn bộ chức năng CAD. Hướng kiểm tra và yêu cầu fixture:
[build and validation](docs/build.md).

See [build and checks](docs/build.md), [publication audit](docs/public-source-review.md)
and [third-party notices](THIRD_PARTY_NOTICES.md).

The project follows its existing **LGPL-2.1-or-later** declaration; the license
text is in [LICENSE](LICENSE). Separate dependency notices remain applicable.
