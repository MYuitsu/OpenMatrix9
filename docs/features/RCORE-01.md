# RCORE-01 shared basis and geometry contracts

<!-- rhino-core-host-parameter-dependency:start -->
Đã nghiệm thu fixture T10 cho điểm phụ thuộc Circle/Line: dùng map openNURBS/OCCT
có sẵn, Rust kiểm điểm với source thật và chặn raw parameter chưa được chứng minh.
Query/basis/source identity được lưu FCStd và tính lại sau sửa/Undo/restore.
238 kiểm tra fixture +114 hồi quy FreeCAD;556 Rust và159 native query đạt.
Phạm vi Circle/Line và oracle lấy mẫu; **toàn bộ P0 vẫn đang triển khai.**
[Bằng chứng và giới hạn](../validation/2026-10-10-rhino-core-host-parameter-dependency.md).
<!-- rhino-core-host-parameter-dependency:end -->

<!-- rhino-core-delayed-publication:start -->
Đã nghiệm thu fixture T08 cho native Rebuild: kết quả được giữ qua lượt giao,
source/session/options hết hiệu lực bị chặn trước Coin/transaction;36 envelope
và36 tham chiếu Shape giải phóng đủ.173 kiểm tra fixture và72 hồi quy đạt.
Solver vẫn đồng bộ, delivery ở GUI queue; lifetime mọi worker vẫn còn mở.
**Toàn bộ P0 vẫn đang triển khai.** [Bằng chứng và giới hạn](../validation/2026-10-10-rhino-core-delayed-publication.md).
<!-- rhino-core-delayed-publication:end -->

<!-- rhino-core-subobject-ownership:start -->
Đã nghiệm thu fixture T11: sửa wrapper Curve/Surface sở hữu riêng, publish qua
native deep copy, giữ source/sibling, đúng Undo/Redo và phân biệt lifetime của
snapshot mạnh với request subobject.35 kiểm tra FreeCAD đạt; không viết lại editor.
**Toàn bộ P0 vẫn đang triển khai.** [Bằng chứng và giới hạn](../validation/2026-10-10-rhino-core-subobject-ownership.md).
<!-- rhino-core-subobject-ownership:end -->

<!-- rhino-core-periodic-persistence:start -->
Đã nghiệm thu T09 bằng curve periodic thật và các face có seam/lỗ; dùng lại
serializer FCStd hiện có.204 kiểm tra warm +102 kiểm tra cold đạt, cùng221 kiểm
tra đối chứng kernel. Type/topology/basis/tọa độ và inventory giữ nguyên.
**Toàn bộ P0 vẫn đang triển khai.** [Bằng chứng và giới hạn](../validation/2026-10-10-rhino-core-periodic-persistence.md).
<!-- rhino-core-periodic-persistence:end -->

<!-- rhino-core-hatch-input-gate:start -->
Đã bổ sung gate Rust cho dữ liệu NURBS của vòng Hatch trước đọc nguồn/copy/dựng
native; dùng lại editor và exporter hiện có.541 kiểm tra Rust,27 kiểm tra native,
5 bộ hồi quy native và269 kiểm tra FreeCAD đạt. Đây là một phần T07;
**toàn bộ P0 vẫn đang triển khai**. [Bằng chứng và giới hạn](../validation/2026-10-10-rhino-core-hatch-input-gate.md).
<!-- rhino-core-hatch-input-gate:end -->

<!-- rhino-core-instance-isolation:start -->
Dùng lại App::Link, mirror/scale không đều và các exporter hiện có; thêm fixture
kiểm đúng tọa độ, giữ nguyên member/definition/instance A khi chỉ sửa B.
T06 đã nghiệm thu:146 kiểm tra native,98 kiểm tra FreeCAD và4 bộ hồi quy native.
Không cần viết lại code instance. **Toàn bộ P0 vẫn đang triển khai.**
[Bằng chứng và giới hạn](../validation/2026-10-10-rhino-core-instance-isolation.md).
<!-- rhino-core-instance-isolation:end -->

<!-- rhino-core-polycurve-partition:start -->
Dùng lại wire, đảo hướng và FCStd của FreeCAD; sửa bước working 3DM để polycurve
giữ từng đoạn thay vì gộp thành một cạnh. Rust kiểm tra thứ tự, miền và điểm nối;
báo cáo có trước commit. T05 đã nghiệm thu với 403 kiểm tra native, 382 kiểm tra
FreeCAD và 523 Rust tests. **Toàn bộ P0 vẫn đang triển khai.**
[Bằng chứng và giới hạn](../validation/2026-10-10-rhino-core-polycurve-partition.md).
<!-- rhino-core-polycurve-partition:end -->

<!-- rhino-core-curve-parameters:start -->
Dùng lại openNURBS/OCCT cho báo cáo query tham số toàn miền: giữ native form1/2,
chặn output lỗi trước khi tính điểm và giữ basis/report độc lập. 502 Rust tests;
159 kiểm tra query native; 257 kiểm tra FreeCAD. Fixture RCORE-01.T01 đã nghiệm thu
với oracle lấy mẫu; T10/History vẫn thiếu tích hợp host. **Toàn bộ P0 đang triển khai.**
[Bằng chứng và giới hạn](../validation/2026-10-10-rhino-core-curve-parameters.md).
<!-- rhino-core-curve-parameters:end -->

Implementation target: Safe Rust owns structural validation, explicit limits,
representation/operation policy, count budgets and copied facts. Native C++ owns
FreeCAD object resolution, OCCT/openNURBS storage envelopes and value extraction;
Python files are host acceptance fixtures. Existing solvers are reused.

The descriptor API is read-only schema1 JSON with existing identity guards, units
and separate native/world provenance. Supporting facts and restriction domains
are explicitly scoped. Native shape-type solidity is not manifold/orientation
validation. Uninspected facts and over-budget arrays stay null with reasons.

Borrowed basis ABI requires live initialized immutable aligned readable arrays
for the call; no pointers are retained. Count/byte/degree gates run before value
inspection. Native exceptions remain native and Python catches inspection errors.
Each side owns/frees its own storage. Release panic=abort remains a limitation.

Generated spline limits and broader native/companion policies remain separate.
Registry branches are Inspect/ImportWorkingV5/ExportWorkingV5 only, with unsupported
options rejected. Legacy command IDs/capabilities are preserved. Conversion facts
are not fabricated from permission or successful construction.

No descriptor read creates objects/properties/transactions. Existing exchange
transactions and Undo/Redo are reused; exact geometry FCStd cold reload is tested.
Generic all-consumer ownership, mappings, async cancellation, permissions and
full rational input allocation safety remain open.

[Verification and remaining clauses](../validation/2026-10-10-rhino-core-kernel-contract.md).
