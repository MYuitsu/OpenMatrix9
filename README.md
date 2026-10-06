# OpenMatrix9

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
| ✅ Đã triển khai trong phạm vi ghi rõ | Có đường thực thi native/Rust và kiểm tra tương ứng; không cam kết toàn bộ tùy chọn của Matrix9/Rhino. |
| 🟡 Một phần | Có chức năng nền tảng nhưng còn giới hạn hoặc chưa tích hợp phần phát triển mới. |
| ⬜ Chưa hoàn thành | Chưa có đường thực thi được xác nhận; icon/catalog chưa được tính là implementation. |

### Danh sách từng lệnh theo nhóm

Mỗi dòng là một mục catalog hoặc lệnh bổ sung. **Tên hiển thị** mô tả mục menu;
**mã catalog** dùng để đối chiếu source, không phải mọi mã đều là command gõ được.
✅ là đã có implementation trong phạm vi hiện tại; ⬜ chưa có đường thực thi
được xác nhận. Ý nghĩa của mục chưa làm mô tả chức năng dự kiến theo tên catalog;
những tên nghiệp vụ chưa đủ căn cứ được ghi rõ cần xác minh.
Các nút sidebar cũng được liệt kê để không nhầm icon đã vẽ với chức năng đã làm.
Mục xuất hiện trong nhiều nhóm có thể lặp lại, cùng mã luôn có cùng trạng thái.

#### Quick tools — thao tác nhanh

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Đã làm chưa? |
|---|---|---|---|
| Duplicate | `TopIconDuplicate` | Tạo bản sao của đối tượng. | ⬜ Chưa hoàn thành |
| Edit Points On | `TopIconEditPointsOn` | Bật các điểm chỉnh sửa đối tượng. | ⬜ Chưa hoàn thành |
| PointsOn | `TopIconControlPointsOn` | Bật control points để chỉnh hình dạng. | ⬜ Chưa hoàn thành |
| Mirror | `TopIconMirror` | Tạo bản đối xứng qua mặt phẳng. | ⬜ Chưa hoàn thành |
| Move | `TopIconMove` | Di chuyển đối tượng. | ⬜ Chưa hoàn thành |
| Rotate | `TopIconRotate` | Xoay đối tượng quanh tâm hoặc trục. | ⬜ Chưa hoàn thành |
| Explode | `TopIconExplode` | Tách đối tượng ghép thành các thành phần. | ⬜ Chưa hoàn thành |
| Join | `TopIconJoin` | Ghép các đường hoặc mặt tương thích. | ⬜ Chưa hoàn thành |
| Split | `TopIconSplit` | Chia đối tượng bằng đối tượng cắt. | ⬜ Chưa hoàn thành |
| Trim | `TopIconTrim` | Cắt bỏ phần thừa của đối tượng. | ⬜ Chưa hoàn thành |
| Ring Rail | `TopIconRingRail` | Tạo đường dẫn cơ sở cho thân nhẫn. | ⬜ Chưa hoàn thành |

#### File

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Đã làm chưa? |
|---|---|---|---|
| New | `FileNew` | Tạo project FreeCAD mới. | ✅ Đã làm (phạm vi hiện tại) |
| Open | `FileOpen` | Mở project hoặc file được FreeCAD hỗ trợ. | ✅ Đã làm (phạm vi hiện tại) |
| Export Selected | `FileExportSelected` | Xuất các đối tượng đang chọn theo workflow export chung. | ⬜ Chưa hoàn thành |
| Import | `FileImport` | Nhập file theo workflow import chung. | ⬜ Chưa hoàn thành |
| Save | `FileSave` | Lưu project hiện hành. | ✅ Đã làm (phạm vi hiện tại) |
| Save As | `FileSaveAs` | Lưu project với tên hoặc vị trí khác. | ✅ Đã làm (phạm vi hiện tại) |
| Save Small | `FileSaveSmall` | Lưu bản dung lượng nhỏ theo workflow catalog. | ⬜ Chưa hoàn thành |
| Save Small As | `FileSaveSmallAs` | Lưu bản dung lượng nhỏ với tên khác. | ⬜ Chưa hoàn thành |
| User Library | `FileUserLibrary` | Mở thư viện đối tượng của người dùng. | ⬜ Chưa hoàn thành |
| User Library Add | `FileUserLibraryAdd` | Thêm đối tượng vào thư viện người dùng. | ⬜ Chưa hoàn thành |
| Stuller Submit | `FileStullerSubmit` | Gửi thiết kế theo workflow Stuller trong catalog. | ⬜ Chưa hoàn thành |
| Notes | `FileNotes` | Đọc và sửa ghi chú lưu cùng project. | ✅ Đã làm (phạm vi hiện tại) |
| Print | `FilePrint` | In bản vẽ hoặc nội dung project. | ⬜ Chưa hoàn thành |

#### View

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Đã làm chưa? |
|---|---|---|---|
| PictureFrame | `ViewBackgroundBitmapPictureFrame` | Đặt ảnh tham chiếu vào không gian làm việc. | ✅ Đã làm (phạm vi hiện tại) |
| RestoreViewports | `ViewRestoreViewports` | Khôi phục bố cục bốn viewport của OM9. | ✅ Đã làm (phạm vi hiện tại) |
| SynchronizeViews | `OthersViewSynchronizeViews` | Đồng bộ các viewport theo quy tắc camera của OM9. | ✅ Đã làm (phạm vi hiện tại) |
| Zoom_Dynamic | `ViewZoomZoomDynamic` | Zoom tương tác bằng thao tác kéo chuột. | ✅ Đã làm (phạm vi hiện tại) |
| Zoom_Extents | `ViewZoomZoomExtents` | Zoom để thấy toàn bộ hình học trong viewport. | ✅ Đã làm (phạm vi hiện tại) |
| Zoom_Selected | `ViewZoomZoomSelected` | Zoom tới các đối tượng đang chọn. | ✅ Đã làm (phạm vi hiện tại) |
| Zoom_Window | `ViewZoomZoomWindow` | Zoom tới vùng hình chữ nhật được chọn. | ✅ Đã làm (phạm vi hiện tại) |
| View Manager | `ViewManager` | Quản lý các view đã lưu. | ⬜ Chưa hoàn thành |
| ViewCaptureToFile | `ViewCaptureToFile` | Lưu ảnh chụp viewport ra file. | ✅ Đã làm (phạm vi hiện tại) |
| Crosshairs | `OthersSetCrosshairs` | Bật hoặc tắt dấu chữ thập theo con trỏ. | ✅ Đã làm (phạm vi hiện tại) |
| 1 To1 | `ViewZoomZoom1To1` | Đặt tỷ lệ hiển thị 1:1. | ⬜ Chưa hoàn thành |
| 1 To1 Calibrate | `ViewZoomZoom1To1Calibrate` | Hiệu chuẩn tỷ lệ hiển thị 1:1 theo màn hình. | ⬜ Chưa hoàn thành |
| CenterViewport | `ViewSetCameraCenterViewport` | Đặt tâm camera của viewport. | ✅ Đã làm (phạm vi hiện tại) |
| Blueprint | `ClayooCreationBlueprint` | Tạo bố cục ảnh blueprint tham chiếu. | ⬜ Chưa hoàn thành |
| System6 HD3 View | `OthersSystem6HD3View` | Workflow hiển thị HD3View trong catalog; chi tiết chưa được xác minh. | ⬜ Chưa hoàn thành |

#### Utilities

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Đã làm chưa? |
|---|---|---|---|
| Direction | `AnalyzeDirection` | Kiểm tra chiều đường cong hoặc hướng pháp tuyến. | ⬜ Chưa hoàn thành |
| Diagnostics Select Bad Objects | `AnalyzeDiagnosticsSelectBadObjects` | Chọn các đối tượng hình học bị lỗi. | ⬜ Chưa hoàn thành |
| Diagnostics Check | `AnalyzeDiagnosticsCheck` | Kiểm tra tính hợp lệ của đối tượng. | ⬜ Chưa hoàn thành |
| Edge Tools Show Edges | `AnalyzeEdgeToolsShowEdges` | Hiển thị các cạnh và cạnh hở. | ⬜ Chưa hoàn thành |
| Edge Tools Split Edge | `AnalyzeEdgeToolsSplitEdge` | Tách một cạnh hình học. | ⬜ Chưa hoàn thành |
| Edge Tools Merge Edge | `AnalyzeEdgeToolsMergeEdge` | Gộp các cạnh tương thích. | ⬜ Chưa hoàn thành |
| Edge Tools Join2 Naked Edges | `AnalyzeEdgeToolsJoin2NakedEdges` | Ghép hai cạnh hở. | ⬜ Chưa hoàn thành |
| Edge Tools Unjoin Edge | `AnalyzeEdgeToolsUnjoinEdge` | Tách liên kết giữa các cạnh đã ghép. | ⬜ Chưa hoàn thành |
| Make2 DDrawing | `DimensionMake2DDrawing` | Tạo hình chiếu hoặc bản vẽ 2D từ mẫu 3D. | ⬜ Chưa hoàn thành |
| Bounding Box | `AnalyzeBoundingBox` | Tạo hộp giới hạn của đối tượng. | ⬜ Chưa hoàn thành |
| gvCenterObjects | `AnalyzeCenterPoint` | Xác định tâm theo workflow gvCenterObjects. | ⬜ Chưa hoàn thành |
| Group | `EditGroupsGroup` | Gom các đối tượng thành nhóm. | ⬜ Chưa hoàn thành |
| UnGroup | `EditGroupsUnGroup` | Bỏ nhóm đối tượng. | ⬜ Chưa hoàn thành |
| Extract Bad Surface | `OthersExtractBadSurface` | Trích xuất các mặt bị lỗi để kiểm tra. | ⬜ Chưa hoàn thành |
| View Show ZBuffer | `ViewShowZBuffer` | Hiển thị dữ liệu độ sâu Z-buffer. | ⬜ Chưa hoàn thành |
| Fill Hole | `MeshMeshRepairToolsFillHole` | Lấp một lỗ trên mesh. | ⬜ Chưa hoàn thành |
| Fill Holes | `MeshMeshRepairToolsFillHoles` | Lấp các lỗ trên mesh. | ⬜ Chưa hoàn thành |
| Align Mesh Vertices | `MeshMeshRepairToolsAlignMeshVertices` | Căn chỉnh các đỉnh mesh. | ⬜ Chưa hoàn thành |
| From NURBSObject | `MeshFromNURBSObject` | Tạo mesh từ geometry NURBS. | ⬜ Chưa hoàn thành |
| Reduce Vertex Count | `MeshMeshEditToolsCollapseReduceVertexCount` | Giảm số đỉnh mesh bằng collapse. | ⬜ Chưa hoàn thành |
| Apply Mesh UVN | `MeshApplyMeshUVN` | Áp dụng thao tác UVN cho mesh. | ⬜ Chưa hoàn thành |
| Extract Render Mesh | `MeshMeshEditToolsExtractRenderMesh` | Trích xuất mesh dùng để render. | ⬜ Chưa hoàn thành |

#### Measure

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Đã làm chưa? |
|---|---|---|---|
| Angle | `AnalyzeAngle` | Đo góc giữa hai hướng xác định bằng bốn điểm. | ✅ Đã làm (phạm vi hiện tại) |
| Distance | `AnalyzeDistance` | Đo khoảng cách giữa hai điểm trong không gian. | ✅ Đã làm (phạm vi hiện tại) |
| Length | `AnalyzeLength` | Đo chiều dài đường cong hoặc cạnh. | ⬜ Chưa hoàn thành |
| Radius | `AnalyzeRadius` | Đo bán kính đường tròn hoặc cung. | ⬜ Chưa hoàn thành |
| Measure Dim Horizontal | `OthersMeasureDimHorizontal` | Tạo kích thước theo phương ngang. | ⬜ Chưa hoàn thành |
| Measure Dim Vertical | `OthersMeasureDimVertical` | Tạo kích thước theo phương đứng. | ⬜ Chưa hoàn thành |
| Angle Dimension | `DimensionAngleDimension` | Tạo chú thích kích thước góc. | ⬜ Chưa hoàn thành |
| Rotated Dimension | `DimensionRotatedDimension` | Tạo kích thước theo phương xoay. | ⬜ Chưa hoàn thành |
| Aligned Dimension | `DimensionAlignedDimension` | Tạo kích thước thẳng hàng với hai điểm. | ⬜ Chưa hoàn thành |
| Diameter Dimension | `DimensionDiameterDimension` | Tạo kích thước đường kính. | ⬜ Chưa hoàn thành |
| Radial Dimension | `DimensionRadialDimension` | Tạo kích thước bán kính. | ⬜ Chưa hoàn thành |
| Leader | `DimensionLeader` | Tạo đường dẫn chú thích leader. | ⬜ Chưa hoàn thành |
| Text Block | `DimensionTextBlock` | Tạo khối chữ chú thích. | ⬜ Chưa hoàn thành |
| Measure Edit Text | `OthersMeasureEditText` | Sửa nội dung chữ chú thích. | ⬜ Chưa hoàn thành |
| Measure Edit Dim | `OthersMeasureEditDim` | Sửa đối tượng kích thước. | ⬜ Chưa hoàn thành |
| Recenter Dimension Text | `DimensionRecenterDimensionText` | Đưa chữ kích thước về giữa. | ⬜ Chưa hoàn thành |
| Measure Dim Options | `OthersMeasureDimOptions` | Thiết lập tùy chọn kích thước. | ⬜ Chưa hoàn thành |

#### Curve

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Đã làm chưa? |
|---|---|---|---|
| Polyline | `CurvePolylinePolyline` | Vẽ đường gấp khúc qua nhiều điểm bằng chuột hoặc console. | ✅ Đã làm (phạm vi hiện tại) |
| Line | `CurveLineSingleLine` | Vẽ đoạn thẳng giữa các điểm bằng chuột hoặc console. | ✅ Đã làm (phạm vi hiện tại) |
| Interpolate Points | `CurveFreeFormInterpolatePoints` | Vẽ đường cong nội suy qua các điểm. | ⬜ Chưa hoàn thành |
| Rectangle Cornerto Corner | `CurveRectangleCornertoCorner` | Vẽ hình chữ nhật bằng hai góc. | ⬜ Chưa hoàn thành |
| Circle Center Radius | `CurveCircleCenterRadius` | Vẽ đường tròn từ tâm và bán kính. | ⬜ Chưa hoàn thành |
| Ellipse From Center | `CurveEllipseFromCenter` | Vẽ ellipse từ tâm. | ⬜ Chưa hoàn thành |
| Arc Center Start Angle | `CurveArcCenterStartAngle` | Vẽ cung tròn từ tâm, điểm đầu và góc. | ⬜ Chưa hoàn thành |
| Arc Start End Direction | `CurveArcStartEndDirection` | Vẽ cung tròn theo điểm đầu, điểm cuối và hướng. | ⬜ Chưa hoàn thành |
| Curve Rebuild | `OthersCurveRebuild` | Dựng lại đường cong với cấu trúc control points mới. | ⬜ Chưa hoàn thành |
| Refit To Tolerance | `CurveCurveEditToolsRefitToTolerance` | Fit lại đường cong theo dung sai. | ⬜ Chưa hoàn thành |
| Blend Curves | `CurveBlendCurvesBlendCurves` | Tạo đường nối chuyển tiếp giữa các đường cong. | ⬜ Chưa hoàn thành |
| Blend Crv | `CurveBlendCurvesBlendCrv` | Tạo đường blend với điều kiện liên tục. | ⬜ Chưa hoàn thành |
| Curve From2 Views | `CurveCurveFrom2Views` | Tạo đường cong 3D từ hai hình chiếu. | ⬜ Chưa hoàn thành |
| Crv2 View History | `BuilderCrv2ViewHistory` | Tạo đường cong từ hai view có quan hệ history. | ⬜ Chưa hoàn thành |
| Pullback | `CurveCurveFromObjectsPullback` | Kéo đường cong về mặt tham chiếu. | ⬜ Chưa hoàn thành |
| Project | `CurveCurveFromObjectsProject` | Chiếu đường cong lên mặt. | ⬜ Chưa hoàn thành |
| Intersection | `CurveCurveFromObjectsIntersection` | Tạo đường giao giữa các đối tượng. | ⬜ Chưa hoàn thành |
| Curve | `CurveOffsetCurve` | Tạo đường offset ở khoảng cách cho trước. | ⬜ Chưa hoàn thành |
| Offset Crv On Srf | `CurveOffsetOffsetCrvOnSrf` | Offset đường cong trên mặt. | ⬜ Chưa hoàn thành |
| Extract Isocurve | `CurveCurveFromObjectsExtractIsocurve` | Trích đường đẳng tham số trên mặt. | ⬜ Chưa hoàn thành |
| Fillet Curves | `CurveFilletCurves` | Bo tròn chỗ nối giữa các đường cong. | ⬜ Chưa hoàn thành |
| Fillet Corners | `CurveFilletCorners` | Bo tròn các góc của đường cong. | ⬜ Chưa hoàn thành |
| Chamfer Curves | `CurveChamferCurves` | Vát góc giữa các đường cong. | ⬜ Chưa hoàn thành |
| Create UVCurves | `CurveCurveFromObjectsCreateUVCurves` | Tạo đường biểu diễn trong miền UV của mặt. | ⬜ Chưa hoàn thành |
| Apply UVCurves | `CurveCurveFromObjectsApplyUVCurves` | Ánh xạ đường UV lên mặt. | ⬜ Chưa hoàn thành |
| Through Points | `CurveFreeFormThroughPoints` | Tạo đường cong đi qua các điểm. | ⬜ Chưa hoàn thành |
| Extend Curve | `CurveExtendCurveExtendCurve` | Kéo dài đường cong. | ⬜ Chưa hoàn thành |
| Polygon Center Radius | `CurvePolygonCenterRadius` | Vẽ đa giác từ tâm và bán kính. | ⬜ Chưa hoàn thành |
| Spiral | `CurveSpiral` | Tạo đường xoắn ốc phẳng. | ⬜ Chưa hoàn thành |
| Helix | `CurveHelix` | Tạo đường xoắn lò xo trong không gian. | ⬜ Chưa hoàn thành |
| Single Point | `CurvePointObjectSinglePoint` | Tạo một đối tượng điểm. | ⬜ Chưa hoàn thành |
| Mark Curve Start | `CurvePointObjectMarkCurveStart` | Đánh dấu điểm đầu đường cong. | ⬜ Chưa hoàn thành |
| Mark Curve End | `CurvePointObjectMarkCurveEnd` | Đánh dấu điểm cuối đường cong. | ⬜ Chưa hoàn thành |
| Adjust Closed Curve Seam | `CurveCurveEditToolsAdjustClosedCurveSeam` | Điều chỉnh seam của đường cong kín. | ⬜ Chưa hoàn thành |
| Continue Interp Curve | `CurveFreeFormContinueInterpCurve` | Tiếp tục một đường cong nội suy. | ⬜ Chưa hoàn thành |
| Curve Divide Curve | `OthersCurveDivideCurve` | Chia đường cong thành các đoạn hoặc điểm. | ⬜ Chưa hoàn thành |
| Curve Boolean | `CurveCurveEditToolsCurveBoolean` | Thực hiện Boolean giữa các vùng đường cong. | ⬜ Chưa hoàn thành |
| Tween Curves | `CurveTweenCurves` | Tạo các đường trung gian giữa hai đường cong. | ⬜ Chưa hoàn thành |
| Duplicate Edge | `CurveCurveFromObjectsDuplicateEdge` | Sao chép cạnh thành đường cong riêng. | ⬜ Chưa hoàn thành |
| Duplicate Border | `CurveCurveFromObjectsDuplicateBorder` | Sao chép biên mặt thành đường cong. | ⬜ Chưa hoàn thành |
| Control Points | `CurveFreeFormControlPoints` | Tạo đường cong bằng control points. | ⬜ Chưa hoàn thành |
| Continue Curve | `CurveFreeFormContinueCurve` | Tiếp tục đường cong đang có. | ⬜ Chưa hoàn thành |
| GVExtract Isocurve | `CurveGVExtractIsocurve` | Trích isocurve theo workflow GV. | ⬜ Chưa hoàn thành |
| Cross Section Profiles | `CurveCrossSectionProfiles` | Tạo các profile mặt cắt. | ⬜ Chưa hoàn thành |
| Sketch | `CurveFreeFormSketch` | Vẽ đường cong tự do bằng chuột. | ⬜ Chưa hoàn thành |
| Sketchon Surface | `CurveFreeFormSketchonSurface` | Vẽ đường tự do trên mặt. | ⬜ Chưa hoàn thành |
| Sketchon Polygon Mesh | `CurveFreeFormSketchonPolygonMesh` | Vẽ đường tự do trên mesh. | ⬜ Chưa hoàn thành |
| Polyline On Surface | `BuilderPolylineOnSurface` | Vẽ polyline trên mặt. | ⬜ Chưa hoàn thành |
| Interpolateon Surface | `CurveFreeFormInterpolateonSurface` | Nội suy đường cong trên mặt. | ⬜ Chưa hoàn thành |
| Convert Curve To Lines | `CurveConvertCurveToLines` | Chuyển đường cong thành các đoạn thẳng. | ⬜ Chưa hoàn thành |
| Section | `CurveCurveFromObjectsSection` | Tạo đường mặt cắt từ đối tượng. | ⬜ Chưa hoàn thành |
| Match | `CurveCurveEditToolsMatch` | Khớp đầu đường cong theo điều kiện tiếp xúc/liên tục. | ⬜ Chưa hoàn thành |
| Silhouette | `CurveCurveFromObjectsSilhouette` | Trích đường silhouette theo hướng nhìn. | ⬜ Chưa hoàn thành |
| Extract Wireframe | `CurveCurveFromObjectsExtractWireframe` | Trích mạng đường wireframe từ đối tượng. | ⬜ Chưa hoàn thành |
| Soft Edit | `CurveCurveEditToolsSoftEdit` | Sửa đường cong với ảnh hưởng mềm. | ⬜ Chưa hoàn thành |
| Offset Crv Normal To Surface | `CurveOffsetOffsetCrvNormalToSurface` | Offset đường cong theo pháp tuyến mặt. | ⬜ Chưa hoàn thành |
| Arc Blend | `CurveBlendCurvesArcBlend` | Nối đường cong bằng arc blend. | ⬜ Chưa hoàn thành |
| Intersect Two Sets | `CurveCurveFromObjectsIntersectTwoSets` | Tạo giao tuyến giữa hai tập đối tượng. | ⬜ Chưa hoàn thành |

#### Surface

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Đã làm chưa? |
|---|---|---|---|
| Sweep1 Rail | `SurfaceSweepSweep1Rail` | Quét profile theo một rail để tạo mặt. | ⬜ Chưa hoàn thành |
| Sweep2 Rails | `SurfaceSweepSweep2Rails` | Quét profile theo hai rail để tạo mặt. | ⬜ Chưa hoàn thành |
| Profile Sweep | `SurfaceSweepProfileSweep` | Quét mặt từ profile theo workflow Profile Sweep. | ⬜ Chưa hoàn thành |
| Planar Curves | `SurfacePlanarCurves` | Tạo mặt phẳng từ các đường biên đồng phẳng. | ⬜ Chưa hoàn thành |
| Plane Cornerto Corner | `SurfacePlaneCornertoCorner` | Tạo mặt phẳng bằng hai góc. | ⬜ Chưa hoàn thành |
| Surface Rebuild | `OthersSurfaceRebuild` | Dựng lại mặt với cấu trúc control points mới. | ⬜ Chưa hoàn thành |
| Blend Surface | `SurfaceBlendSurface` | Tạo mặt chuyển tiếp giữa các mặt. | ⬜ Chưa hoàn thành |
| Variable Blend Surfaces | `SurfaceVariableFilletBlendChamferVariableBlendSurfaces` | Tạo blend giữa các mặt với tham số biến thiên. | ⬜ Chưa hoàn thành |
| Loft | `SurfaceLoft` | Tạo mặt loft qua các tiết diện. | ⬜ Chưa hoàn thành |
| Curve Network | `SurfaceCurveNetwork` | Tạo mặt từ mạng đường cong. | ⬜ Chưa hoàn thành |
| Shrink Trimmed Surface | `SurfaceSurfaceEditToolsShrinkTrimmedSurface` | Thu gọn miền nền của mặt đã trim. | ⬜ Chưa hoàn thành |
| Patch | `SurfacePatch` | Tạo mặt patch phủ vùng biên. | ⬜ Chưa hoàn thành |
| Edge Curves | `SurfaceEdgeCurves` | Tạo mặt từ các đường biên. | ⬜ Chưa hoàn thành |
| Untrim | `SurfaceSurfaceEditToolsUntrim` | Khôi phục phần nền chưa trim của mặt. | ⬜ Chưa hoàn thành |
| GVDExtrude All | `SurfaceGVDExtrudeAll` | Extrude các thành phần theo workflow GVD. | ⬜ Chưa hoàn thành |
| Revolve | `SurfaceRevolve` | Tạo mặt tròn xoay quanh trục. | ⬜ Chưa hoàn thành |
| Rail Revolve | `SurfaceRailRevolve` | Tạo mặt xoay theo rail. | ⬜ Chưa hoàn thành |
| Four Profile Sweep | `BuilderFourProfileSweep` | Tạo mặt quét với bốn profile. | ⬜ Chưa hoàn thành |
| Sweep Multi Rail | `SurfaceSweepSweepMultiRail` | Tạo mặt quét theo nhiều rail. | ⬜ Chưa hoàn thành |
| Tween Surfaces | `SurfaceTweenSurfaces` | Tạo các mặt trung gian. | ⬜ Chưa hoàn thành |
| Fillet Surfaces | `SurfaceFilletSurfaces` | Bo tròn giữa các mặt. | ⬜ Chưa hoàn thành |
| Variable Fillet Surfaces | `SurfaceVariableFilletBlendChamferVariableFilletSurfaces` | Bo mặt với bán kính biến thiên. | ⬜ Chưa hoàn thành |
| Chamfer Surfaces | `SurfaceChamferSurfaces` | Vát nối giữa các mặt. | ⬜ Chưa hoàn thành |
| Variable Chamfer Surfaces | `SurfaceVariableFilletBlendChamferVariableChamferSurfaces` | Vát mặt với kích thước biến thiên. | ⬜ Chưa hoàn thành |
| Offset Surface | `SurfaceOffsetSurface` | Offset mặt theo khoảng cách. | ⬜ Chưa hoàn thành |
| Variable Offset Surface | `SurfaceVariableOffsetSurface` | Offset mặt với khoảng cách biến thiên. | ⬜ Chưa hoàn thành |
| Merge | `SurfaceSurfaceEditToolsMerge` | Gộp các mặt tương thích. | ⬜ Chưa hoàn thành |
| Match | `SurfaceSurfaceEditToolsMatch` | Khớp mặt theo điều kiện liên tục. | ⬜ Chưa hoàn thành |
| Drape | `SurfaceDrape` | Tạo mặt phủ drape trên hình học. | ⬜ Chưa hoàn thành |
| Heightfieldfrom Image | `SurfaceHeightfieldfromImage` | Tạo mặt độ cao từ ảnh. | ⬜ Chưa hoàn thành |
| Splitat Isocurve | `SurfaceSurfaceEditToolsSplitatIsocurve` | Chia mặt tại đường đẳng tham số. | ⬜ Chưa hoàn thành |
| Extend Surface | `SurfaceExtendSurface` | Kéo dài mặt. | ⬜ Chưa hoàn thành |
| Extrude Curve Normal To Surface | `SurfaceExtrudeCurveNormalToSurface` | Extrude đường cong theo pháp tuyến mặt. | ⬜ Chưa hoàn thành |
| Unroll Developable Srf | `SurfaceUnrollDevelopableSrf` | Trải phẳng mặt có thể khai triển. | ⬜ Chưa hoàn thành |
| Smash | `SurfaceSmash` | Trải phẳng mặt theo workflow Smash. | ⬜ Chưa hoàn thành |
| Soft Edit | `SurfaceSurfaceEditToolsSoftEdit` | Sửa mặt với vùng ảnh hưởng mềm. | ⬜ Chưa hoàn thành |
| Plane Cutting Plane | `SurfacePlaneCuttingPlane` | Tạo mặt phẳng dùng để cắt. | ⬜ Chưa hoàn thành |
| Adjust Closed Surface Seam | `SurfaceSurfaceEditToolsAdjustClosedSurfaceSeam` | Điều chỉnh seam của mặt kín. | ⬜ Chưa hoàn thành |
| Set Surface Tangent | `SurfaceSurfaceEditToolsSetSurfaceTangent` | Thiết lập điều kiện tiếp tuyến của mặt. | ⬜ Chưa hoàn thành |
| Refitto Tolerance | `SurfaceSurfaceEditToolsRefittoTolerance` | Fit lại mặt theo dung sai. | ⬜ Chưa hoàn thành |

#### Solid

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Đã làm chưa? |
|---|---|---|---|
| Union | `SolidUnion` | Hợp các khối bằng Boolean. | ⬜ Chưa hoàn thành |
| Difference | `SolidDifference` | Trừ khối cắt khỏi khối đích. | ⬜ Chưa hoàn thành |
| Intersection | `SolidIntersection` | Lấy phần giao của các khối. | ⬜ Chưa hoàn thành |
| Boolean Two Objects | `SolidBooleanTwoObjects` | Thực hiện Boolean theo workflow hai đối tượng. | ⬜ Chưa hoàn thành |
| Cap Planar Holes | `SolidCapPlanarHoles` | Đóng các lỗ đồng phẳng để tạo khối kín. | ⬜ Chưa hoàn thành |
| Extract Surface | `SolidExtractSurface` | Trích một mặt khỏi khối hoặc polysurface. | ⬜ Chưa hoàn thành |
| Fillet Edge | `SolidFilletEdgeFilletEdge` | Bo tròn các cạnh khối. | ⬜ Chưa hoàn thành |
| Text | `SolidText` | Tạo hình học chữ dạng khối. | ⬜ Chưa hoàn thành |
| Extrude Planar Curve Straight | `SolidExtrudePlanarCurveStraight` | Extrude đường kín đồng phẳng theo phương thẳng. | ⬜ Chưa hoàn thành |
| GVDAll Extrude | `SolidGVDAllExtrude` | Extrude theo workflow GVD. | ⬜ Chưa hoàn thành |
| Pipe | `SolidPipe` | Tạo ống theo đường dẫn. | ⬜ Chưa hoàn thành |
| Box Cornerto Corner Height | `SolidBoxCornertoCornerHeight` | Tạo hộp từ hai góc đáy và chiều cao. | ⬜ Chưa hoàn thành |
| Sphere Center Radius | `SolidSphereCenterRadius` | Tạo cầu từ tâm và bán kính. | ⬜ Chưa hoàn thành |
| Ellipsoid From Center | `SolidEllipsoidFromCenter` | Tạo ellipsoid từ tâm. | ⬜ Chưa hoàn thành |
| Torus | `SolidTorus` | Tạo khối xuyến. | ⬜ Chưa hoàn thành |
| Cylinder | `SolidCylinder` | Tạo hình trụ. | ⬜ Chưa hoàn thành |
| Tube | `SolidTube` | Tạo ống trụ rỗng. | ⬜ Chưa hoàn thành |
| Pyramid | `SolidPyramid` | Tạo hình chóp. | ⬜ Chưa hoàn thành |
| Cone | `SolidCone` | Tạo hình nón. | ⬜ Chưa hoàn thành |
| Truncated Cone | `SolidTruncatedCone` | Tạo hình nón cụt. | ⬜ Chưa hoàn thành |
| Boss | `SolidBoss` | Tạo phần nhô boss trên khối. | ⬜ Chưa hoàn thành |
| Rib | `SolidRib` | Tạo gân tăng cứng rib. | ⬜ Chưa hoàn thành |
| Slab | `SolidSlab` | Tạo khối slab từ profile. | ⬜ Chưa hoàn thành |
| Make Hole | `SolidSolidEditToolsHolesMakeHole` | Tạo lỗ trên khối. | ⬜ Chưa hoàn thành |
| Array Hole | `SolidSolidEditToolsHolesArrayHole` | Tạo mảng lỗ theo phương thẳng. | ⬜ Chưa hoàn thành |
| Array Hole Polar | `SolidSolidEditToolsHolesArrayHolePolar` | Tạo mảng lỗ quanh trục. | ⬜ Chưa hoàn thành |
| Move Hole | `SolidSolidEditToolsHolesMoveHole` | Di chuyển lỗ trên khối. | ⬜ Chưa hoàn thành |
| Move Face | `SolidSolidEditToolsFacesMoveFace` | Di chuyển một mặt của khối. | ⬜ Chưa hoàn thành |
| Shell | `SolidShell` | Tạo vỏ rỗng theo bề dày. | ⬜ Chưa hoàn thành |
| Pt On | `SolidPtOn` | Bật các điểm điều khiển của khối. | ⬜ Chưa hoàn thành |

#### Transform

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Đã làm chưa? |
|---|---|---|---|
| Scale3 D | `TransformScaleScale3D` | Scale đối tượng theo ba chiều. | ⬜ Chưa hoàn thành |
| Scale2 D | `TransformScaleScale2D` | Scale đối tượng trong hai chiều. | ⬜ Chưa hoàn thành |
| Scale1 D | `TransformScaleScale1D` | Scale đối tượng theo một chiều. | ⬜ Chưa hoàn thành |
| Non Uniform Scale | `TransformScaleNonUniformScale` | Scale với hệ số khác nhau trên các trục. | ⬜ Chưa hoàn thành |
| Orient2 Points | `TransformOrient2Points` | Định hướng đối tượng bằng hai điểm. | ⬜ Chưa hoàn thành |
| Orient3 Points | `TransformOrient3Points` | Định hướng đối tượng bằng ba điểm. | ⬜ Chưa hoàn thành |
| Rotate3 D | `TransformRotate3D` | Xoay đối tượng quanh trục 3D. | ⬜ Chưa hoàn thành |
| Bend | `TransformBend` | Uốn cong đối tượng. | ⬜ Chưa hoàn thành |
| Polar | `TransformArrayPolar` | Tạo mảng đối tượng quanh tâm hoặc trục. | ⬜ Chưa hoàn thành |
| Projectto CPlane | `TransformProjecttoCPlane` | Chiếu đối tượng về construction plane. | ⬜ Chưa hoàn thành |
| Smart Flow | `gvSmartFlow` | Biến dạng đối tượng theo workflow Smart Flow. | ⬜ Chưa hoàn thành |
| Smart Flow Rigid | `gvSmartFlowRigid` | Flow đối tượng theo chế độ giữ cứng từng phần. | ⬜ Chưa hoàn thành |
| Smart Flow Break | `gvSmartFlowBreak` | Tách quan hệ của Smart Flow. | ⬜ Chưa hoàn thành |
| Smart Flow Pull Down Curves | `gvSmartFlowPullDownCurves` | Kéo đường cong xuống theo workflow Smart Flow. | ⬜ Chưa hoàn thành |
| Smart Pattern | `TransformSmartPattern` | Tạo pattern theo workflow Smart Pattern. | ⬜ Chưa hoàn thành |
| Orient On Surface | `TransformOrientOnSurface` | Định hướng đối tượng trên mặt. | ⬜ Chưa hoàn thành |
| Orient Perpendicularto Curve | `TransformOrientPerpendiculartoCurve` | Định hướng đối tượng vuông góc đường cong. | ⬜ Chưa hoàn thành |
| Symmetry | `TransformSymmetry` | Thiết lập đối xứng của đối tượng. | ⬜ Chưa hoàn thành |
| Solid Pt On | `TransformSolidPtOn` | Bật các điểm điều khiển của khối. | ⬜ Chưa hoàn thành |
| Cage Edit | `TransformCageEditingCageEdit` | Biến dạng đối tượng bằng lồng cage. | ⬜ Chưa hoàn thành |
| Along Curve | `TransformArrayAlongCurve` | Tạo mảng dọc đường cong. | ⬜ Chưa hoàn thành |
| Array Linear | `TransformArrayArrayLinear` | Tạo mảng theo phương thẳng. | ⬜ Chưa hoàn thành |
| Along Surface | `TransformArrayAlongSurface` | Tạo mảng trên mặt. | ⬜ Chưa hoàn thành |
| Along Curveon Surface | `TransformArrayAlongCurveonSurface` | Tạo mảng dọc đường cong nằm trên mặt. | ⬜ Chưa hoàn thành |
| Rectangular | `TransformArrayRectangular` | Tạo mảng hình chữ nhật. | ⬜ Chưa hoàn thành |
| Flow Along Curve | `TransformFlowAlongCurve` | Biến dạng đối tượng dọc đường cong. | ⬜ Chưa hoàn thành |
| Flow Along Surface | `TransformFlowAlongSurface` | Biến dạng đối tượng theo mặt. | ⬜ Chưa hoàn thành |
| Splop | `TransformSplop` | Đặt và biến dạng đối tượng lên mặt theo workflow Splop. | ⬜ Chưa hoàn thành |
| Set Points | `TransformSetPoints` | Đặt tọa độ các điểm theo phương được chọn. | ⬜ Chưa hoàn thành |
| Scale By Plane | `TransformScaleScaleByPlane` | Scale đối tượng theo mặt phẳng. | ⬜ Chưa hoàn thành |
| Stretch | `TransformStretch` | Kéo giãn một vùng đối tượng. | ⬜ Chưa hoàn thành |
| Taper | `TransformTaper` | Tạo biến dạng thuôn. | ⬜ Chưa hoàn thành |
| Shear | `TransformShear` | Tạo biến dạng xiên shear. | ⬜ Chưa hoàn thành |
| Twist | `TransformTwist` | Xoắn đối tượng quanh trục. | ⬜ Chưa hoàn thành |
| Maelstrom | `TransformMaelstrom` | Tạo biến dạng xoáy Maelstrom. | ⬜ Chưa hoàn thành |
| Make Down Facing | `BuilderMakeDownFacing` | Định hướng đối tượng quay xuống. | ⬜ Chưa hoàn thành |
| Orient Remapto CPlane | `TransformOrientRemaptoCPlane` | Chuyển đối tượng giữa các construction plane. | ⬜ Chưa hoàn thành |
| Move UVN | `TransformMoveUVN` | Di chuyển theo hệ tọa độ UVN. | ⬜ Chưa hoàn thành |
| Soft Move | `TransformSoftMove` | Di chuyển với ảnh hưởng mềm. | ⬜ Chưa hoàn thành |
| Create Cage | `TransformCageEditingCreateCage` | Tạo lồng điều khiển cage. | ⬜ Chưa hoàn thành |

#### Clayoo / SubD

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Đã làm chưa? |
|---|---|---|---|
| Item | `ClayooEditItem` | Chỉnh sửa thành phần SubD/Clayoo. | ⬜ Chưa hoàn thành |
| Divide | `ClayooEditDivide` | Chia nhỏ các thành phần SubD. | ⬜ Chưa hoàn thành |
| Split Sides | `ClayooEditSplitSides` | Tách các cạnh bên của thành phần SubD. | ⬜ Chưa hoàn thành |
| Inset | `ClayooEditInset` | Tạo vùng inset trên mặt SubD. | ⬜ Chưa hoàn thành |
| Knife | `ClayooEditKnife` | Cắt topology bằng knife. | ⬜ Chưa hoàn thành |
| Offset | `ClayooEditOffset` | Offset thành phần SubD. | ⬜ Chưa hoàn thành |
| Extrude | `ClayooEditExtrude` | Extrude thành phần SubD. | ⬜ Chưa hoàn thành |
| Bridge | `ClayooEditBridge` | Nối các biên SubD bằng bridge. | ⬜ Chưa hoàn thành |
| Append Face | `ClayooCreationAppendFace` | Thêm một mặt vào topology SubD. | ⬜ Chưa hoàn thành |
| Pipe | `ClayooCreationPipe` | Tạo ống theo workflow Clayoo. | ⬜ Chưa hoàn thành |
| Clayoo Bezel | `ClayooCreationClayooBezel` | Tạo bezel theo workflow Clayoo. | ⬜ Chưa hoàn thành |
| Ring | `ClayooCreationRing` | Tạo nhẫn bằng workflow SubD/Clayoo. | ⬜ Chưa hoàn thành |
| Signet Ring | `ClayooCreationSignetRing` | Tạo nhẫn signet bằng workflow SubD/Clayoo. | ⬜ Chưa hoàn thành |
| Crease | `ClayooEditCrease` | Đặt cạnh sắc crease của SubD. | ⬜ Chưa hoàn thành |
| Flip Normals | `ClayooEditFlipNormals` | Đảo pháp tuyến. | ⬜ Chưa hoàn thành |
| Project To Plane | `ClayooEditProjectToPlane` | Chiếu thành phần về mặt phẳng. | ⬜ Chưa hoàn thành |
| Collapse | `ClayooEditCollapse` | Thu gộp thành phần topology. | ⬜ Chưa hoàn thành |
| Match | `ClayooEditMatch` | Khớp các thành phần SubD. | ⬜ Chưa hoàn thành |
| Weld | `ClayooEditWeld` | Hàn các đỉnh hoặc biên. | ⬜ Chưa hoàn thành |
| Symmetry | `ClayooEditSymmetry` | Thiết lập đối xứng SubD. | ⬜ Chưa hoàn thành |
| Box | `ClayooPrimitivesBox` | Tạo hộp SubD. | ⬜ Chưa hoàn thành |
| Sphere | `ClayooPrimitivesSphere` | Tạo cầu SubD. | ⬜ Chưa hoàn thành |
| Cylinder | `ClayooPrimitivesCylinder` | Tạo trụ SubD. | ⬜ Chưa hoàn thành |
| Cone | `ClayooPrimitivesCone` | Tạo nón SubD. | ⬜ Chưa hoàn thành |
| Torus | `ClayooPrimitivesTorus` | Tạo xuyến SubD. | ⬜ Chưa hoàn thành |
| Plane | `ClayooPrimitivesPlane` | Tạo mặt phẳng SubD. | ⬜ Chưa hoàn thành |
| Sweep1 Rail | `ClayooCreationSweep1Rail` | Quét SubD theo một rail. | ⬜ Chưa hoàn thành |
| Sweep2 Rails | `ClayooCreationSweep2Rails` | Quét SubD theo hai rail. | ⬜ Chưa hoàn thành |
| Loft | `ClayooCreationLoft` | Tạo loft SubD. | ⬜ Chưa hoàn thành |
| Wrap | `ClayooEditWrap` | Bọc đối tượng theo workflow Wrap. | ⬜ Chưa hoàn thành |
| Revolve | `ClayooCreationRevolve` | Tạo SubD tròn xoay. | ⬜ Chưa hoàn thành |
| From Curve | `ClayooCreationFromCurve` | Tạo SubD từ đường cong. | ⬜ Chưa hoàn thành |
| From2 Curves | `ClayooCreationFrom2Curves` | Tạo SubD từ hai đường cong. | ⬜ Chưa hoàn thành |
| Fill | `ClayooEditFill` | Lấp vùng biên SubD. | ⬜ Chưa hoàn thành |
| Shell | `ClayooEditShell` | Tạo vỏ có bề dày từ SubD. | ⬜ Chưa hoàn thành |
| Extract | `ClayooEditExtract` | Trích xuất thành phần SubD. | ⬜ Chưa hoàn thành |
| From Surface | `ClayooCreationFromSurface` | Tạo SubD từ surface. | ⬜ Chưa hoàn thành |
| From TSplines | `ClayooCreationFromTSplines` | Chuyển từ dữ liệu T-Splines sang workflow SubD. | ⬜ Chưa hoàn thành |
| From Mesh | `ClayooCreationFromMesh` | Tạo SubD từ mesh. | ⬜ Chưa hoàn thành |
| To Nurbs | `ClayooCreationToNurbs` | Chuyển SubD sang NURBS. | ⬜ Chưa hoàn thành |
| Selection Sets | `ClayooSelectionSelectionSets` | Quản lý các tập selection SubD. | ⬜ Chưa hoàn thành |
| Select Ring | `ClayooSelectionSelectRing` | Chọn dải cạnh ring. | ⬜ Chưa hoàn thành |
| Select Loop | `ClayooSelectionSelectLoop` | Chọn chuỗi cạnh loop. | ⬜ Chưa hoàn thành |
| Select UV | `ClayooSelectUV` | Chọn theo tọa độ UV. | ⬜ Chưa hoàn thành |
| Select All | `ClayooSelectionSelectAll` | Chọn tất cả thành phần SubD. | ⬜ Chưa hoàn thành |
| Select None | `ClayooSelectionSelectNone` | Bỏ chọn các thành phần SubD. | ⬜ Chưa hoàn thành |
| Paint Selection | `ClayooSelectionPaintSelection` | Chọn thành phần bằng thao tác tô. | ⬜ Chưa hoàn thành |
| Grow Selection | `ClayooSelectionGrowSelection` | Mở rộng vùng selection. | ⬜ Chưa hoàn thành |
| Shrink | `ClayooSelectionShrink` | Thu nhỏ vùng selection. | ⬜ Chưa hoàn thành |
| Invert Selection | `ClayooSelectionInvertSelection` | Đảo selection. | ⬜ Chưa hoàn thành |
| Select Creased | `ClayooSelectionSelectCreased` | Chọn các cạnh crease. | ⬜ Chưa hoàn thành |
| Select Naked | `ClayooSelectionSelectNaked` | Chọn các biên hở. | ⬜ Chưa hoàn thành |
| Select Coplanar | `ClayooSelectionSelectCoplanar` | Chọn các thành phần đồng phẳng. | ⬜ Chưa hoàn thành |
| Subdivision Level | `ClayooEditSubdivisionLevel` | Thay đổi mức subdivision. | ⬜ Chưa hoàn thành |
| Clayoo Library | `ClayooCreationClayooLibrary` | Mở thư viện đối tượng Clayoo/SubD. | ⬜ Chưa hoàn thành |
| Set Plane | `ClayooSelectionSetPlane` | Đặt mặt phẳng thao tác selection. | ⬜ Chưa hoàn thành |
| Reset Plane | `ClayooSelectionResetPlane` | Đặt lại mặt phẳng thao tác. | ⬜ Chưa hoàn thành |
| Create Isocurves | `ClayooCreateIsocurves` | Tạo isocurves theo workflow Clayoo. | ⬜ Chưa hoàn thành |

#### Emboss

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Đã làm chưa? |
|---|---|---|---|
| Matrix Art | `EmbossMatrixArt` | Tạo relief theo workflow Matrix Art. | ⬜ Chưa hoàn thành |
| Heightfield Image | `EmbossHeightfieldImage` | Tạo relief độ cao từ ảnh. | ⬜ Chưa hoàn thành |
| Texture Builder | `TextureBuilder` | Tạo texture hình học. | ⬜ Chưa hoàn thành |
| Clay Emboss | `EmbossClayEmboss` | Tạo emboss theo workflow Clay Emboss. | ⬜ Chưa hoàn thành |
| Sculpt | `EmbossSculpt` | Điêu khắc hình học. | ⬜ Chưa hoàn thành |
| Decimator | `EmbossDecimator` | Giảm độ phức tạp mesh của relief. | ⬜ Chưa hoàn thành |
| Clay Emboss Resources | `EmbossClayEmbossResources` | Quản lý tài nguyên cho workflow Clay Emboss. | ⬜ Chưa hoàn thành |

#### Builder

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Đã làm chưa? |
|---|---|---|---|
| Eternity | `BuilderEternity` | Tạo nhẫn eternity theo tham số. | ⬜ Chưa hoàn thành |
| Signet | `BuilderSignet` | Tạo nhẫn signet theo builder. | ⬜ Chưa hoàn thành |
| Signet Ring | `ClayooCreationSignetRing` | Tạo nhẫn signet bằng workflow SubD/Clayoo. | ⬜ Chưa hoàn thành |
| Ring | `ClayooCreationRing` | Tạo nhẫn bằng workflow SubD/Clayoo. | ⬜ Chưa hoàn thành |
| Text On Curve | `TextOnCurve` | Đặt chữ dọc đường cong. | ⬜ Chưa hoàn thành |
| Raised | `BuilderRaised` | Tạo chi tiết raised theo builder; tham số nghiệp vụ chưa xác minh. | ⬜ Chưa hoàn thành |
| Award Ring Builder | `BuilderAwardRingBuilder` | Tạo nhẫn award theo builder. | ⬜ Chưa hoàn thành |
| Rope | `Builder_Rope` | Tạo cấu trúc dây xoắn rope. | ⬜ Chưa hoàn thành |
| Jump | `BuilderJump` | Tạo vòng nối jump ring. | ⬜ Chưa hoàn thành |
| Free Form Knot | `BuilderFreeFormKnot` | Tạo nút thắt tự do. | ⬜ Chưa hoàn thành |
| Knot | `BuilderKnot` | Tạo nút thắt theo builder. | ⬜ Chưa hoàn thành |
| Pattern Builder | `BuilderPatternBuilder` | Tạo pattern theo builder. | ⬜ Chưa hoàn thành |
| Nautilus Builder | `BuilderNautilusBuilder` | Tạo hình xoắn Nautilus theo builder. | ⬜ Chưa hoàn thành |
| Mill Work | `BuilderMillWork` | Workflow Mill Work; quy tắc gia công cụ thể chưa xác minh. | ⬜ Chưa hoàn thành |
| Mill Area | `BuilderMillArea` | Workflow Mill Area; quy tắc vùng gia công chưa xác minh. | ⬜ Chưa hoàn thành |
| Mill Work C | `BuilderMillWorkC` | Biến thể Mill Work C; ý nghĩa chi tiết chưa xác minh. | ⬜ Chưa hoàn thành |

#### Tools

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Đã làm chưa? |
|---|---|---|---|
| Ring Rail | `BuilderRingRail` | Tạo đường dẫn cơ sở cho thân nhẫn. | ⬜ Chưa hoàn thành |
| Profile Placer | `BuilderProfilePlacer` | Đặt profile trên rail. | ⬜ Chưa hoàn thành |
| Outside Ring Rail | `BuilderOutsideRingRail` | Tạo rail ngoài của nhẫn. | ⬜ Chưa hoàn thành |
| Custom Rail | `BuilderCustomRail` | Tạo rail tùy chỉnh. | ⬜ Chưa hoàn thành |
| Four Profile | `BuilderFourProfile` | Tạo bộ bốn profile cho thiết kế. | ⬜ Chưa hoàn thành |
| Metal Weights | `BuilderMetalWeights` | Tính khối lượng kim loại theo mẫu và vật liệu. | ⬜ Chưa hoàn thành |
| Resize | `BuilderResize` | Thay đổi size nhẫn. | ⬜ Chưa hoàn thành |
| Surface Pullback | `BuilderSurfacePullback` | Kéo đường cong về mặt theo builder. | ⬜ Chưa hoàn thành |
| Bermark Library | `BuilderBermarkLibrary` | Mở thư viện Bermark trong catalog. | ⬜ Chưa hoàn thành |
| Stuller Findings Library | `BuilderStullerFindingsLibrary` | Mở thư viện findings Stuller trong catalog. | ⬜ Chưa hoàn thành |
| Smart Target | `gvSmartTarget` | Đặt hoặc sử dụng Smart Target. | ⬜ Chưa hoàn thành |
| Smart Target Add | `gvSmartTargetAdd` | Thêm Smart Target. | ⬜ Chưa hoàn thành |
| Smart Target On Crv End | `gvSmartTargetOnCrvEnd` | Đặt Smart Target ở đầu đường cong. | ⬜ Chưa hoàn thành |
| Smart Blend | `gvSmartBlend` | Tạo chuyển tiếp theo workflow Smart Blend. | ⬜ Chưa hoàn thành |
| Smart MSR | `gvSmartMSR` | Workflow Smart MSR; thuật toán cụ thể chưa xác minh. | ⬜ Chưa hoàn thành |
| Join History | `gvJoinHistory` | Ghép đối tượng có quan hệ history. | ⬜ Chưa hoàn thành |
| Image Trace | `BuilderImageTrace` | Trace đường cong từ ảnh. | ⬜ Chưa hoàn thành |
| Object Checker | `BuilderObjectChecker` | Kiểm tra đối tượng trước xuất hoặc sản xuất. | ⬜ Chưa hoàn thành |
| Mesh Repair | `BuilderMeshRepair` | Sửa mesh theo builder. | ⬜ Chưa hoàn thành |
| 3 d Printing | `Builder3dPrinting` | Chuẩn bị mẫu cho in 3D. | ⬜ Chưa hoàn thành |
| Profile | `BuilderProfile` | Tạo profile theo builder. | ⬜ Chưa hoàn thành |
| Profile Merge | `BuilderProfileMerge` | Gộp các profile. | ⬜ Chưa hoàn thành |
| Profile End Cap | `BuilderProfileEndCap` | Đóng đầu profile bằng end cap. | ⬜ Chưa hoàn thành |
| Custom Rail Advanced | `BuilderCustomRailAdvanced` | Tạo rail tùy chỉnh nâng cao. | ⬜ Chưa hoàn thành |
| Object On Crv | `BuilderObjectOnCrv` | Đặt đối tượng dọc đường cong. | ⬜ Chưa hoàn thành |
| Surface Inset | `BuilderSurfaceInset` | Tạo inset trên mặt. | ⬜ Chưa hoàn thành |
| Mesh Reducer | `BuilderMeshReducer` | Giảm số phần tử mesh. | ⬜ Chưa hoàn thành |
| Mesh | `BuilderMesh` | Tạo mesh theo builder. | ⬜ Chưa hoàn thành |
| Curve Transform Tool | `BuilderCurveTransformTool` | Biến đổi đường cong bằng công cụ builder. | ⬜ Chưa hoàn thành |
| Center Line | `BuilderCenterLine` | Tạo đường tâm. | ⬜ Chưa hoàn thành |
| Boolean Builder | `BuilderBooleanBuilder` | Thực hiện Boolean theo builder. | ⬜ Chưa hoàn thành |

#### Gems

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Đã làm chưa? |
|---|---|---|---|
| Gem Loader | `BuilderGemLoader` | Nạp mẫu đá từ thư viện gem. | ⬜ Chưa hoàn thành |
| Gem On Crv | `BuilderGemOnCrv` | Đặt đá dọc đường cong. | ⬜ Chưa hoàn thành |
| Gem On Crv Advanced | `BuilderGemOnCrvAdvanced` | Đặt đá dọc đường cong với tùy chọn nâng cao. | ⬜ Chưa hoàn thành |
| Gem Count On Crv | `BuilderGemCountOnCrv` | Đặt hoặc tính số đá trên đường cong. | ⬜ Chưa hoàn thành |
| Gem List On Crv | `BuilderGemListOnCrv` | Đặt danh sách đá trên đường cong. | ⬜ Chưa hoàn thành |
| Gem On Crv Multi | `BuilderGemOnCrvMulti` | Đặt nhiều loại đá dọc đường cong. | ⬜ Chưa hoàn thành |
| Gem Profile | `BuilderGemProfile` | Tạo profile của đá. | ⬜ Chưa hoàn thành |
| Gem Guides | `BuilderGemGuides` | Tạo đường hướng dẫn đặt đá. | ⬜ Chưa hoàn thành |
| Orient To Gem | `BuilderOrientToGem` | Định hướng đối tượng theo đá. | ⬜ Chưa hoàn thành |
| Halo | `BuilderHalo` | Tạo bố trí đá halo. | ⬜ Chưa hoàn thành |
| Gem On Surface | `BuilderGemOnSurface` | Đặt đá trên mặt. | ⬜ Chưa hoàn thành |
| Custom Gem | `BuilderCustomGem` | Tạo đá tùy chỉnh. | ⬜ Chưa hoàn thành |
| Emerald | `BuilderEmerald` | Tạo đá dạng emerald. | ⬜ Chưa hoàn thành |
| Baguette | `BuilderBaguette` | Tạo đá dạng baguette. | ⬜ Chưa hoàn thành |
| Taper Bag Channel | `BuilderTaperBagChannel` | Tạo channel với đá baguette thuôn. | ⬜ Chưa hoàn thành |
| Taper Bag Between Two Curves | `BuilderTaperBagBetweenTwoCurves` | Đặt baguette thuôn giữa hai đường cong. | ⬜ Chưa hoàn thành |
| Gem Between Two Curves | `BuilderGemBetweenTwoCurves` | Đặt đá giữa hai đường cong. | ⬜ Chưa hoàn thành |
| Cluster Builder Old | `BuilderClusterBuilderOld` | Tạo cụm đá theo workflow Cluster cũ. | ⬜ Chưa hoàn thành |
| Gem Reporter | `BuilderGemReporter` | Tạo báo cáo đá. | ⬜ Chưa hoàn thành |
| Gem Map | `BuilderGemMap` | Tạo bản đồ bố trí đá. | ⬜ Chưa hoàn thành |
| Pave Dialog | `BuilderPaveDialog` | Mở tùy chọn bố trí pavé. | ⬜ Chưa hoàn thành |
| Pave Builder | `BuilderPaveBuilder` | Tạo bố trí pavé. | ⬜ Chưa hoàn thành |
| Pave Azure Builder | `BuilderPaveAzureBuilder` | Tạo azure cho pavé. | ⬜ Chưa hoàn thành |
| Pave Prong Builder | `BuilderPaveProngBuilder` | Tạo chấu cho pavé. | ⬜ Chưa hoàn thành |
| Gem Springs | `BuilderGemSprings` | Workflow Gem Springs; quy tắc chi tiết chưa xác minh. | ⬜ Chưa hoàn thành |
| Gem Follow | `BuilderGemFollow` | Cho đá theo đối tượng hoặc đường tham chiếu. | ⬜ Chưa hoàn thành |
| Gem Control | `BuilderGemControl` | Điều khiển các tham số bố trí đá. | ⬜ Chưa hoàn thành |
| Match Attributes | `OthersMatchAttributes` | Sao chép thuộc tính giữa các đối tượng. | ⬜ Chưa hoàn thành |
| Save Style | `OthersSaveStyle` | Lưu style đối tượng. | ⬜ Chưa hoàn thành |
| Load Style | `OthersLoadStyle` | Nạp style đối tượng. | ⬜ Chưa hoàn thành |
| GVDGem Flow | `BuilderGVDGemFlow` | Flow đá theo workflow GVD. | ⬜ Chưa hoàn thành |
| GVDGem Splop | `BuilderGVDGemSplop` | Đặt đá lên mặt theo workflow Gem Splop. | ⬜ Chưa hoàn thành |
| Pave Sphere | `BuilderPaveSphere` | Tạo bố trí pavé trên cầu. | ⬜ Chưa hoàn thành |
| Gem Update | `BuilderGemUpdate` | Cập nhật bố trí hoặc tham số đá. | ⬜ Chưa hoàn thành |
| Gem Positioner | `BuilderGemPositioner` | Điều chỉnh vị trí đá. | ⬜ Chưa hoàn thành |

#### Settings — ổ đá và chấu

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Đã làm chưa? |
|---|---|---|---|
| Head | `BuilderHead` | Tạo ổ hoặc đầu chấu giữ đá. | ⬜ Chưa hoàn thành |
| Bezel | `BuilderBezel` | Tạo ổ bezel. | ⬜ Chưa hoàn thành |
| Clayoo Bezel | `ClayooCreationClayooBezel` | Tạo bezel theo workflow Clayoo. | ⬜ Chưa hoàn thành |
| Pull To Rail | `BuilderPullToRail` | Kéo cấu trúc ổ đá về rail. | ⬜ Chưa hoàn thành |
| Bezel Cutter | `BuilderBezelCutter` | Tạo khối cắt cho bezel. | ⬜ Chưa hoàn thành |
| Prong Adder | `BuilderProngAdder` | Thêm chấu giữ đá. | ⬜ Chưa hoàn thành |
| Prong Editor | `BuilderProngEditor` | Chỉnh sửa chấu. | ⬜ Chưa hoàn thành |
| Prong On Surface | `BuilderProngOnSurface` | Đặt chấu trên mặt. | ⬜ Chưa hoàn thành |
| Bead On Surface | `BuilderBeadOnSurface` | Đặt bead trên mặt. | ⬜ Chưa hoàn thành |
| Metal Piece | `BuilderMetalPiece` | Tạo chi tiết kim loại theo builder. | ⬜ Chưa hoàn thành |
| Bead On Crv | `BuilderBeadOnCrv` | Đặt bead dọc đường cong. | ⬜ Chưa hoàn thành |
| Channel Border Curve | `BuilderChannelBorderCurve` | Tạo đường biên cho channel. | ⬜ Chưa hoàn thành |

#### Cutters

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Đã làm chưa? |
|---|---|---|---|
| Gem Cutter | `BuilderGemCutter` | Tạo khối cắt chỗ đặt đá. | ⬜ Chưa hoàn thành |
| Gem Cutter Library | `BuilderGemCutterLibrary` | Mở thư viện gem cutters. | ⬜ Chưa hoàn thành |
| Channel Builder | `BuilderChannelBuilder` | Tạo channel theo builder. | ⬜ Chưa hoàn thành |
| Micro Prong Cutter | `BuilderMicroProngCutter` | Tạo khối cắt cho micro-prong. | ⬜ Chưa hoàn thành |
| Bright Cut Channel | `BuilderBrightCutChannel` | Tạo channel theo workflow bright cut. | ⬜ Chưa hoàn thành |
| Bright Cutter | `BuilderBrightCutter` | Tạo khối bright cutter. | ⬜ Chưa hoàn thành |
| Bezel Cutter | `BuilderBezelCutter` | Tạo khối cắt cho bezel. | ⬜ Chưa hoàn thành |
| Cutto Finger Rail | `OthersCuttoFingerRail` | Cắt đối tượng theo finger rail. | ⬜ Chưa hoàn thành |
| Planeand Cube Cutters | `BuilderPlaneandCubeCutters` | Tạo khối/mặt phẳng dùng để cắt. | ⬜ Chưa hoàn thành |
| Quad Flip | `BuilderQuadFlip` | Workflow Quad Flip; ý nghĩa hình học chi tiết chưa xác minh. | ⬜ Chưa hoàn thành |
| Boolean Builder | `BuilderBooleanBuilder` | Thực hiện Boolean theo builder. | ⬜ Chưa hoàn thành |

#### Render

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Đã làm chưa? |
|---|---|---|---|
| Render Vray Render Styler | `BuilderRenderVrayRenderStyler` | Thiết lập render style theo workflow V-Ray trong catalog. | ⬜ Chưa hoàn thành |
| Create Spotlight | `RenderCreateSpotlight` | Tạo đèn spotlight. | ⬜ Chưa hoàn thành |
| Create Point Light | `RenderCreatePointLight` | Tạo đèn điểm. | ⬜ Chưa hoàn thành |
| Create Directional Light | `RenderCreateDirectionalLight` | Tạo đèn định hướng. | ⬜ Chưa hoàn thành |
| Set Spot Lightto View | `RenderSetSpotLighttoView` | Căn spotlight theo hướng nhìn. | ⬜ Chưa hoàn thành |
| Create Rectangular Light | `RenderCreateRectangularLight` | Tạo đèn hình chữ nhật. | ⬜ Chưa hoàn thành |
| Create Linear Light | `RenderCreateLinearLight` | Tạo đèn dạng đường. | ⬜ Chưa hoàn thành |
| Props | `BuilderProps` | Thêm props cho scene. | ⬜ Chưa hoàn thành |
| Set Renderer | `RenderSetRenderer` | Chọn render engine. | ⬜ Chưa hoàn thành |
| Surface Environment Map | `AnalyzeSurfaceEnvironmentMap` | Hiển thị environment map để kiểm tra mặt. | ⬜ Chưa hoàn thành |
| Apply Edge Softening | `RenderEffectsApplyEdgeSoftening` | Áp dụng hiệu ứng làm mềm cạnh khi render. | ⬜ Chưa hoàn thành |
| Animation Builder | `BuilderAnimationBuilder` | Tạo animation theo builder. | ⬜ Chưa hoàn thành |
| Animation Builder Advanced | `BuilderAnimationBuilderAdvanced` | Tạo animation với tùy chọn nâng cao. | ⬜ Chưa hoàn thành |
| Matrix Movie Maker | `MatrixMovieMaker` | Tạo phim theo workflow Matrix Movie Maker. | ⬜ Chưa hoàn thành |
| Render Editor | `BuilderRenderEditor` | Mở trình chỉnh sửa render. | ⬜ Chưa hoàn thành |
| Batch Render | `BuilderBatchRender` | Render theo lô. | ⬜ Chưa hoàn thành |
| Layout Tools | `BuilderLayoutTools` | Tạo bố cục trình bày. | ⬜ Chưa hoàn thành |
| Four View Capture | `BuilderFourViewCapture` | Chụp bộ ảnh từ bốn view. | ⬜ Chưa hoàn thành |
| Render Scheduler | `BuilderRenderScheduler` | Lập lịch render theo catalog. | ⬜ Chưa hoàn thành |
| Alpha Erase | `gvAlphaErase` | Workflow Alpha Erase; thao tác alpha cụ thể chưa xác minh. | ⬜ Chưa hoàn thành |
| Save Render Window As | `RenderSaveRenderWindowAs` | Lưu ảnh trong cửa sổ render. | ⬜ Chưa hoàn thành |
| Web Viewer | `WebViewer` | Mở viewer phục vụ chia sẻ trên web. | ⬜ Chưa hoàn thành |
| Apply Displacement | `RenderEffectsApplyDisplacement` | Áp dụng displacement khi render. | ⬜ Chưa hoàn thành |

#### Core / project / selection

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Đã làm chưa? |
|---|---|---|---|
| Undo | `Undo` | Hoàn tác thao tác của project. | ✅ Đã làm (phạm vi hiện tại) |
| Redo | `Redo` | Làm lại thao tác đã hoàn tác. | ✅ Đã làm (phạm vi hiện tại) |
| Select All | `SelectAll` | Chọn các đối tượng của project hiện hành. | ✅ Đã làm (phạm vi hiện tại) |
| Select None | `SelectNone` | Bỏ selection hiện hành. | ✅ Đã làm (phạm vi hiện tại) |
| Delete | `Delete` | Xóa các đối tượng đang chọn, hỗ trợ Undo qua host. | ✅ Đã làm (phạm vi hiện tại) |
| Project Add | `ProjectAdd` | Tạo/thêm project từ sidebar. | ⬜ Chưa hoàn thành |
| Project Out | `ProjectOut` | Đưa dữ liệu ra project theo catalog. | ⬜ Chưa hoàn thành |
| Project In | `ProjectIn` | Nạp dữ liệu vào project theo catalog. | ⬜ Chưa hoàn thành |
| Project Save | `ProjectSave` | Lưu project từ sidebar. | ⬜ Chưa hoàn thành |
| Project Delete | `ProjectDelete` | Xóa project từ sidebar. | ⬜ Chưa hoàn thành |
| Project Manager | `ProjectManager` | Quản lý danh sách project. | ⬜ Chưa hoàn thành |

#### View và display bổ sung

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Đã làm chưa? |
|---|---|---|---|
| View Front | `ViewFront` | Chuyển camera sang hướng nhìn trước. | ✅ Đã làm (phạm vi hiện tại) |
| View Top | `ViewTop` | Chuyển camera sang hướng nhìn trên. | ✅ Đã làm (phạm vi hiện tại) |
| View Right | `ViewRight` | Chuyển camera sang hướng nhìn phải. | ✅ Đã làm (phạm vi hiện tại) |
| View Left | `ViewLeft` | Chuyển camera sang hướng nhìn trái. | ✅ Đã làm (phạm vi hiện tại) |
| View Rear | `ViewRear` | Chuyển camera sang hướng nhìn sau. | ✅ Đã làm (phạm vi hiện tại) |
| View Bottom | `ViewBottom` | Chuyển camera sang hướng nhìn dưới. | ✅ Đã làm (phạm vi hiện tại) |
| View Isometric | `ViewIsometric` | Chuyển camera sang hướng nhìn isometric. | ✅ Đã làm (phạm vi hiện tại) |
| Fit All | `FitAll` | Zoom để thấy toàn bộ đối tượng. | ✅ Đã làm (phạm vi hiện tại) |
| ViewportTabs | `InfoSettingsViewportTabsToggle` | Bật/tắt tabs viewport. | ✅ Đã làm (phạm vi hiện tại) |
| ShowGrid | `GridON` | Bật/tắt lưới viewport. | ✅ Đã làm (phạm vi hiện tại) |
| View Grid Options | `ViewGridOptions` | Thiết lập lưới viewport. | ⬜ Chưa hoàn thành |
| Preview Cutter ON | `PreviewCutterON` | Bật preview cutter. | ⬜ Chưa hoàn thành |
| Preview Shade ON | `PreviewShadeON` | Bật chế độ shaded preview. | ⬜ Chưa hoàn thành |
| Shade Selected Only ON | `ShadeSelectedOnlyON` | Chỉ shade các đối tượng đang chọn. | ⬜ Chưa hoàn thành |
| GVGem View 1 | `GVGemView_1` | Đổi hiển thị đá theo catalog. | ⬜ Chưa hoàn thành |
| GVSurface View 1 | `GVSurfaceView_1` | Đổi hiển thị surface theo catalog. | ⬜ Chưa hoàn thành |
| Dial Wireframe | `Dial_Wireframe` | Đổi viewport sang hiển thị wireframe. | ⬜ Chưa hoàn thành |
| Dial Shaded | `Dial_Shaded` | Đổi viewport sang hiển thị shaded. | ⬜ Chưa hoàn thành |
| Dial Working Shade | `Dial_Working Shade` | Đổi viewport sang working shade. | ⬜ Chưa hoàn thành |
| Dial Working Render | `Dial_Working Render` | Đổi viewport sang working render. | ⬜ Chưa hoàn thành |
| Dial Ghosted | `Dial_Ghosted` | Đổi viewport sang hiển thị ghosted. | ⬜ Chưa hoàn thành |
| Dial Tech Shade | `Dial_Tech Shade` | Đổi viewport sang technical shade. | ⬜ Chưa hoàn thành |

#### Snap / ràng buộc

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Đã làm chưa? |
|---|---|---|---|
| Osnap E | `ToolsObjectSnapEnd` | Bắt điểm đầu hoặc cuối cạnh/đường cong. | ✅ Đã làm (phạm vi hiện tại) |
| Near | `ToolsObjectSnapNear` | Bắt điểm gần nhất trên đối tượng. | ⬜ Chưa hoàn thành |
| Osnap P | `ToolsObjectSnapPoint` | Bắt đối tượng điểm hoặc vertex được hỗ trợ. | ✅ Đã làm (phạm vi hiện tại) |
| Osnap M | `ToolsObjectSnapMidpoint` | Bắt điểm giữa cạnh/đường cong được hỗ trợ. | ✅ Đã làm (phạm vi hiện tại) |
| Center | `ToolsObjectSnapCenter` | Bắt tâm đường tròn hoặc cung. | ⬜ Chưa hoàn thành |
| Intersection | `ToolsObjectSnapIntersection` | Bắt điểm giao của đối tượng. | ⬜ Chưa hoàn thành |
| Perpendicular To | `ToolsObjectSnapPerpendicularTo` | Bắt điểm tạo hướng vuông góc. | ⬜ Chưa hoàn thành |
| Tangent To | `ToolsObjectSnapTangentTo` | Bắt điểm tạo hướng tiếp tuyến. | ⬜ Chưa hoàn thành |
| Quadrant | `ToolsObjectSnapQuadrant` | Bắt các điểm một phần tư đường tròn. | ⬜ Chưa hoàn thành |
| Knot | `ToolsObjectSnapKnot` | Bắt knot của đường cong. | ⬜ Chưa hoàn thành |
| Snap Between | `SnapBetween` | Bắt điểm giữa hai tham chiếu. | ⬜ Chưa hoàn thành |
| Snap On Surface | `SnapOnSurface` | Bắt điểm trên mặt. | ⬜ Chưa hoàn thành |
| Snap On Polysurface | `SnapOnPolysurface` | Bắt điểm trên polysurface. | ⬜ Chưa hoàn thành |
| Grid Snap ON | `GridSnapON` | Bắt điểm theo lưới. | ⬜ Chưa hoàn thành |
| Ortho | `OrthoSnapON` | Ràng buộc hướng Ortho trong công cụ đang hỗ trợ. | ✅ Đã làm (phạm vi hiện tại) |
| Planar Snap ON | `PlanarSnapON` | Ràng buộc thao tác theo mặt phẳng. | ⬜ Chưa hoàn thành |
| Project Snap ON | `ProjectSnapON` | Chiếu điểm bắt về mặt phẳng thao tác. | ⬜ Chưa hoàn thành |

#### Properties / history / settings / layers

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Đã làm chưa? |
|---|---|---|---|
| Display Properties | `InfoSettingsDisplayProperties` | Thiết lập thuộc tính hiển thị. | ⬜ Chưa hoàn thành |
| Rhino Options | `RhinoOptions` | Mở bộ tùy chọn tương đương workflow Rhino. | ⬜ Chưa hoàn thành |
| GVObject Info | `GVObjectInfo` | Xem thông tin đối tượng theo workflow GV. | ⬜ Chưa hoàn thành |
| Properties | `ObjectProperties` | Mở inspector thuộc tính native FreeCAD. | ✅ Đã làm (phạm vi hiện tại) |
| All Object Info | `AllObjectInfo` | Xem thông tin của toàn bộ đối tượng. | ⬜ Chưa hoàn thành |
| Command History | `CommandHistory` | Xem lịch sử command. | ✅ Đã làm (phạm vi hiện tại) |
| Project Notes | `ProjectNotes` | Đọc và sửa ghi chú project. | ✅ Đã làm (phạm vi hiện tại) |
| Super Select | `SuperSelect` | Selection nâng cao theo catalog. | ⬜ Chưa hoàn thành |
| Box Edit | `InfoSettingsBoxEdit` | Chỉnh đối tượng qua hộp giới hạn. | ⬜ Chưa hoàn thành |
| Libraries | `InfoSettingsLibraries` | Quản lý thư viện. | ⬜ Chưa hoàn thành |
| Selection Filter | `InfoSettingsSelectionFilter` | Lọc loại đối tượng được chọn. | ⬜ Chưa hoàn thành |
| Design Report | `InfoSettingsDesignReport` | Tạo báo cáo thiết kế. | ⬜ Chưa hoàn thành |
| Gumball Alignment | `InfoSettingsGumballAlignment` | Căn hệ trục gizmo thao tác. | ⬜ Chưa hoàn thành |
| Relocate Gumball | `InfoSettingsRelocateGumball` | Di chuyển tâm gizmo thao tác. | ⬜ Chưa hoàn thành |
| Gumball ON | `InfoSettingsGumballON` | Bật gizmo thao tác. | ⬜ Chưa hoàn thành |
| Smart Targets Gumball ON | `InfoSettingsSmartTargetsGumballON` | Bật gizmo cho Smart Targets. | ⬜ Chưa hoàn thành |
| Rhino Smart Track ON | `RhinoSmartTrackON` | Bật tracking thông minh theo catalog. | ⬜ Chưa hoàn thành |
| Rhino History ON | `RhinoHistoryON` | Bật history tương đương workflow Rhino. | ⬜ Chưa hoàn thành |
| GVHistory Update ON | `GVHistoryUpdateON` | Bật cập nhật history GV. | ⬜ Chưa hoàn thành |
| GVHistory Record ON | `GVHistoryRecordON` | Bật ghi history GV. | ⬜ Chưa hoàn thành |
| GVClear History | `InfoSettingsGVClearHistory` | Xóa quan hệ history GV. | ⬜ Chưa hoàn thành |
| Layer Arrow | `LayerArrow` | Chọn layer thao tác trên sidebar. | ⬜ Chưa hoàn thành |
| Layer Lock | `LayerLock` | Khóa layer trên sidebar. | ⬜ Chưa hoàn thành |
| Layer Visibility | `LayerVisibility` | Bật/tắt visibility layer trên sidebar. | ⬜ Chưa hoàn thành |
| Layer Hide | `LayerHide` | Ẩn layer theo catalog. | ⬜ Chưa hoàn thành |
| Layer Show | `LayerShow` | Hiện layer theo catalog. | ⬜ Chưa hoàn thành |

#### 3DM chuyên biệt / keyboard

| Tên lệnh / mục menu | Mã catalog | Ý nghĩa | Đã làm chưa? |
|---|---|---|---|
| Import3dm | `FileImport3dm` | Nhập geometry 3DM hoặc giữ snapshot nguồn trong FCStd. | 🟡 Đã làm một phần — chưa full 3DM |
| Export3dm | `FileExport3dm` | Xuất geometry được chọn thành archive Rhino5; chưa full preservation. | 🟡 Đã làm một phần — chưa full 3DM |
| F6 | `F6` | Mở context menu độc lập của OM9 theo selection. | ✅ Đã làm (phạm vi hiện tại) |

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
