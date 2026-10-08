# Rhino → OpenMatrix9: lộ trình 5 phase

Ngày: 2026-10-08. Trạng thái: tài liệu lập kế hoạch đã viết; implementation/acceptance của scope mới chưa bắt đầu.
[Thiết kế chung](../specs/2026-10-08-rhino-modeling-five-phase-design.md).
Source lập kế hoạch: H:/FreeCAD-src/build/om9-dev.

## Quyết định của người dùng

- Mục tiêu là nhận geometry từ Rhino rồi vẽ/sửa tiếp trong FreeCAD.
- Không yêu cầu history Rhino/Matrix và render.
- Phase2 phải phân biệt object CAD/Mesh để tránh lấy toàn points của mesh lớn.
- File3DM Import/Export Selected trước; clipboard sau.
- Dùng Superpowers, chia đúng5 phase chi tiết.

## Năm mức hỗ trợ

| Phase | Deliverable | Gate thực tế | Status kế thừa |
|---|---|---|---|
| [1 — Nhận geometry](2026-10-08-rhino-modeling-phase-1-exchange.md) | Modeling import/export, current geometry, basic metadata và một runtime thống nhất | 3DM→FCStd→selected V5, input mới+sửa, nguồn bị xóa vẫn dùng được | File exchange/display có proof; modeling mode planned |
| [2 — Phân loại/snap/curves](2026-10-08-rhino-modeling-phase-2-classification-snap-curves.md) | CAD/Mesh/cloud/subd classification, bounded snap, curve commands/CV editing | Default snap reads0 mesh/cloud points; curves nhập sửa/xuất đúng | Basic snap/CAD curves có code; index/classifier/combined editor planned |
| [3 — BRep dựng tiếp](2026-10-08-rhino-modeling-phase-3-brep-modeling.md) | Imported surfaces/solids dùng Loft/Sweep/Trim/Join/Boolean | Nhẫn→cutter mới→cut→chi tiết mới→save/export/reimport | BRep edited profiles đã có scoped proof; workflow nhẫn planned |
| [4 — Blocks/references](2026-10-08-rhino-modeling-phase-4-blocks-references.md) | Independent copy/member editing, materialized blocks, exact reference curves | Copy sửa độc lập; nested transform/owner/selection đúng | Preservation graph có proof; modeling semantics còn thiếu |
| [5 — Geometry và bản cài](2026-10-08-rhino-modeling-phase-5-geometry-release.md) | Remaining geometry adapters, target-version matrix, primary integration | Mọi promised operation có evidence trên installed binary cuối | Native extra slices có proof; SubD/cage/morph/final integration open |

**Hết Phase3 là mốc thử dùng dựng CAD thông thường.** Đây không phải chứng nhận full Rhino SDK hoặc full archive preservation.

## Dependency và kế hoạch thực hiện

P1 → P2 → P3 → P4 → P5.
Mỗi phase có file map, API đề xuất, regression cases, lệnh chạy và exit gate.
Tổng16 task kế hoạch. Con số này không phải số prepared test batches, số geometry types hoặc phần trăm support.
Future native/host/Rhino batch count: unknown cho tới khi fixture matrix được liệt kê.

Các task mới chưa được chạy; tất cả checkbox còn mở. Existing proof chỉ tái dùng làm baseline có hash; không thay proof mới.
Gộp source chọn lọc từ Mod/OpenMatrix9 vào dev ở P2/P3; nghiệm thu lại trên binary đồng nhất, không ghép kết quả của nhiều runtime.
Không đổi execution ledger8 packages của full-exchange cũ. Scope mới có requirements/evidence riêng; làm measurement thật mới cập nhật ledger.

## Superpowers execution contract

Thiết kế và5 kế hoạch là kết quả của yêu cầu hiện tại. Đây là draft để review trước coding, không phải implementation đã được phê duyệt.
Phương thức thực hiện chưa được chọn; kế hoạch hỗ trợ superpowers:executing-plans cho inline hoặc superpowers:subagent-driven-development khi người dùng yêu cầu delegation.
Không spawn agents, ghi product code, cài dependency, commit/push hoặc thay skill ở lượt lập kế hoạch.
Đọc thiết kế chung và plan phase tương ứng trước khi bắt đầu. Có hidden complexity thì viết subproject design/proof mới, không tự hạ gate.

## Build/runtime verification khi thực hiện

Dùng matching MSVC Developer shell. Xác minh FREECAD_SDK_BUILD, ABI/Qt/Python và required core patches trước configure.
Ví dụ configure module từ source dev (chạy khi execution đã bắt đầu):

~~~powershell
rtk proxy cmake -S H:/FreeCAD-src/build/om9-dev -B H:/FreeCAD-src/build/om9-modeling-module -G Ninja -DCMAKE_BUILD_TYPE=RelWithDebInfo -DFREECAD_SOURCE_DIR=H:/FreeCAD-src -DFREECAD_SDK_BUILD=H:/FreeCAD-src/build/relWithDebInfo -DFREECAD_DEPENDENCY_PREFIX=H:/FreeCAD-src/.pixi/envs/default/Library -DOPENMATRIX9_RUNTIME_ROOT=H:/FreeCAD-src/build/om9-modeling-sdk -DFETCHCONTENT_SOURCE_DIR_OM9_OPENNURBS=H:/FreeCAD-src/build/dependencies/opennurbs
rtk proxy cmake --build H:/FreeCAD-src/build/om9-modeling-module --target OpenMatrix9Gui --parallel 4
~~~

Runtime root phải được stage đầy đủ matching FreeCAD exe/DLL/modules/scripts/helper trước khi chạy; standalone build riêng module không tự tạo full runtime.
P1.1 runner xác minh hashes và gọi existing tests/run_menu_smoke.ps1 bằng absolute FreeCADExe/DependencyPrefix, không dùng mặc địnhD:/.
Native tests configure cùng pinned SDK vào H:/FreeCAD-src/build/3dm-modeling-native bằng options hiện có trong tests/native/CMakeLists.txt; target mới được đăng ký trong từng task trước build.
Trên Unix/khác SDK thay paths/configuration và record platform, không coi command Windows là universal.

Runner mapping:
- Phase1: modeling_baseline_smoke, modeling_exchange_smoke.
- Phase2: modeling_snap_smoke, modeling_snap_performance_smoke, modeling_curve_workflow_smoke, modeling_curve_editor_smoke.
- Phase3: modeling_brep_smoke, modeling_surface_edit_smoke, modeling_ring_workflow_smoke.
- Phase4: modeling_blocks_smoke, modeling_references_smoke, modeling_block_edit_workflow_smoke.
- Phase5: modeling_native_geometry_smoke, modeling_target_smoke, rồi rerun toàn Phase1–4 trên installed candidate.

Không chạy test CAD trong nhiệm vụ viết kế hoạch. Phase close yêu cầu native + real FreeCAD + actual target khi được quảng bá, exit0 và result ok; không chỉ file tồn tại.

## Evidence mỗi phase

Lưu docs/validation/modeling-phase-N/requirements.json và report tương ứng: requirement ID, fixture scope/hash, source/module/script hashes, command, exit code, report, limitations, next action.
Phân biệt proposed / implemented-unverified / scoped-verified / phase-accepted.
Nếu guard chỉ là adapter chưa làm thì vẫn open. Unsupported-by-target phải có independent proof.
Source-deleted FCStd, one transaction, Undo/Redo, Cancel, no stale snapshots và atomic destination là gates xuyên suốt.

## Review tài liệu đã thực hiện

- Đúng5 phase; clipboard deferred theo user answer.
- History/render ngoài baseline nhưng geometric owner references và FreeCAD Undo vẫn trong phạm vi.
- Mesh/cloud default snap zero-read; CAD display-mesh cache không làm đổi kind.
- P2/P3 integration có file map và conflict warning.
- Gate3 là workflow CAD thực tế, không chỉ decoder/object count.
- Gate5 không đánh dấu missing promised geometry adapter là supported.
- Không thay status/proof lịch sử, không claim đã implement từ tài liệu.

