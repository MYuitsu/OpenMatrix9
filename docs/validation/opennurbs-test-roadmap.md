Layer session curve-lifecycle candidate — 2026-10-10: **native curve/Rebuild nhận current layer qua Rust** đạt2 small fixtures,8 FreeCAD reports/179 checks, Rust232/47 groups, native8/8 và Qt13 slots. One Undo/Redo/Cancel/FCStd/foreign transaction, observed legacy own lock/hidden/ByObject được kiểm chứng; all creation routes/complete provenance/guards/whole receive/actual Matrix acceptance còn mở. Full exchange vẫn false. [Bằng chứng](layer-session/lifecycle-slice.json).

Layer session live-panel candidate — 2026-10-10: **Rust-owned32 default palette/live layer UI** đạt2 small fixtures (32/36 layers),5 FreeCAD reports/139 checks,11 Qt behavior tests (+init/cleanup), Rust225/46 groups và focused native8/8. Exact fullpath/custom màu/assign/CMD/lazy Undo được kiểm chứng; lifecycle/guards/whole receive/actual Matrix9/heavy acceptance còn mở. Full exchange vẫn false. [Bằng chứng](layer-session/panel-controller-slice.json).

Layer session multi-source candidate — 2026-10-10: **gộp palette theo fullpath** đạt1 native fixture/8 output cases,18 native checks và5 report FreeCAD/109 checks (retained31 + worker15 + clipboard19 + file12 + exchange32). Rust210 tests/45 result groups (có docs0), focused native8/8. Hai namespace/cùng nguồn và hai file độc lập dùng chung6-layer palette; giữ current block/curve, own flags, ByParent, native layer note/pattern; lỗi global binding bảo vệ target. Controller/Undo, native member/provenance, serialize-once/performance và actual Matrix9 còn mở. [Bằng chứng](layer-session/multi-retained-slice.json).

Layer session retained file candidate — 2026-10-10: **Rust-owned retained palette** đạt1 fixture/5 output cases,12 native checks và5 report FreeCAD/97 checks (retained19 + worker15 + clipboard19 + file12 + exchange32). Rust207 tests/45 result groups (có docs0), focused native8/8. Full6-layer palette, empty layers, current block/new curve, ByParent member nguyên native và palette-only không cần file nguồn đã kiểm chứng. Canonical multi-source/member binding, GUI/controller/Undo và actual Matrix9 còn mở; full exchange vẫn false, batch tương lai chưa xác định. [Bằng chứng](layer-session/retained-writer-slice.json).

Layer session worker candidate — 2026-10-10: **clipboard metadata qua worker** đạt1 fixture pair (2 file3DM nguồn,4 layers/2 objects),30 checks native staging và4 report FreeCAD/78 checks. Rust201 tests/43 result groups (có docs0), focused native7/7. Có kiểm capture thật→native preparation, digest/tag/context và bảo vệ staging; chưa có nhận layer vào document hoặc nghiệm thu Matrix9. Tiếp theo retained-export/instance context rồi controller/persistence/Undo/guards. Gói2–8 còn mở; không thêm batch prepared chưa chạy trong slice này, tổng batch tương lai chưa xác định. [Bằng chứng](layer-session/clipboard-worker-slice.json).

Layer session candidate — 2026-10-10: bộ **clipboard carrier native/host** đạt1/1 case (2 file3DM nguồn),23 checks Qt/Windows và16 checks FreeCAD dùng SDK chung. Rust201 tests/43 result groups (có nhóm docs0 test); focused native7/7. Đã kiểm chứng hai format đi cùng nhau, metadata sai bị từ chối, khôi phục clipboard cũ và giữ owner mới; chưa có commit layer vào document hoặc nghiệm thu Matrix9. Tiếp theo metadata qua worker trước GUI commit, rồi controller/Undo/guards. Các gói full openNURBS2–8 vẫn mở; không có thêm batch chưa chạy đã được chuẩn bị trong slice này, tổng batch tương lai chưa xác định. [Bằng chứng](layer-session/clipboard-native-slice.json).

Phase3 **implementation_verified=true / application_accepted=true (accepted_scoped)**: 18/18 requirement groups;15 fixture Rhino5 / 599 checks, 14 OM9 reports / 504 checks. Actual two-way clipboard11 fixtures /496 Rhino /218 host +120 saved reread checks on the same module. Rust152 tests; native10 suites; tools72 tests; format/strict Clippy PASS; one independent whole-change review resolved. Source/runtime isolated at om9-phase3-dev/sdk; no primary integration. Full openNURBS remains incomplete: seven broader packages open, zero prepared Phase3 batches pending, future total unknown. [Summary](modeling-phase-3/summary.json).

Historical checkpoints below (counts are separate, not cumulative):

Checkpoint hiện tại: **51/51 suite native,39 báo cáo FreeCAD/5.806 kiểm tra**.

Checkpoint bổ sung Phase1 clipboard2026-10-09: **Rhino11/11 file/496 kiểm tra;19 báo cáo FreeCAD/510 kiểm tra;48 suite native,80 Rust/58 Python đạt** trên runtime clipboard riêng. Native preparation nhẫn12.20→3.98s; Mesh20k binding1.854→0.0492s. Mức luồng mặc định60% (14/24), giảm theo task/RAM. Hai phép đo scalar của Rhino5 trên một BRep127 mặt được ghi là unavailable; toàn bộ11 file SaveAs vẫn đạt oracle nhập ngược bounds/topology/area/volume.0 bộ đã chuẩn bị còn chờ trong phạm vi bổ sung;7 gói full openNURBS vẫn mở, tổng bộ tương lai chưa xác định. Tiếp theo Phase2 classification/snap/curve editing với gate ứng dụng. [Bằng chứng](2026-10-09-modeling-phase-1-clipboard.md). Checkpoint51 suite cũ bên trên là lịch sử, không phải số suite được chạy lại trong bộ clipboard.
Hai file nhẫn giữ bounds0,001mm và area/volume cũ;tools42/42 đạt.

Bộ BRep seam/cực và edit mới:**Rhino5 4/4 đạt,0 đối tượng lỗi**.
Native decode1800 và nhập ngược FreeCAD45 kiểm tra đạt. Rhino chèn knot
vào4 đường UV của trụ sau edit; cơ sở chung chứng minh hình học tương đương,
sai lệch CV tối đa4,44×10⁻¹⁶; không tuyên bố raw UV byte-identical.
UV-reference8 vẫn đóng với incompatibility có phạm vi và giữ chặn xuất V5.

**0 bộ đã chuẩn bị đang chờ;7 đợt(2–8) chưa đóng. Tổng số bộ test còn lại
chưa xác định.** Tiếp theo các tương tác component/owner-edit/transform và
phạm vi correspondence còn thiếu. Chưa full. [Bằng chứng](2026-10-08-seam-trim-complete.md).

| Đợt | Phạm vi | Trạng thái |
|---|---|---|
| 1 | Chuẩn hóa coverage | Hoàn tất |
| 2 | CurveOnSurface / PolyEdge | Đang làm |
| 3 | BRep / solid | Đang làm |
| 4 | Các loại hình học còn lại | Còn mở |
| 5 | Annotation / styles / layout | Còn mở |
| 6 | Resources / block liên kết | Còn mở |
| 7 | Document / merge / payload | Còn mở |
| 8 | Nghiệm thu / tích hợp / build cuối | Còn mở |

**Còn 7 đợt chưa đóng. Tổng số bộ test còn lại chưa xác định**, vì chưa liệt kê
hết fixture theo loại dữ liệu, thuộc tính và tương tác của các đợt còn lại.
Khi một ma trận được xác định, thêm vào hàng đợi và báo số mới; nếu phát hiện
lỗi mới cần regression, giải thích việc tăng số bộ. Không hứa một số hữu hạn
mà chưa có ma trận đầy đủ.

Mẫu báo cáo sau mỗi lần đo:

- **Vừa đo:** tên bộ, đợt, số file đã hoàn tất / đạt / không đạt; lỗi script nếu có.
- **Đã chứng minh:** hình học/dữ liệu/tham chiếu nào; dẫn báo cáo.
- **Còn lại:** số bộ đã chuẩn bị, danh sách tiếp theo, số đợt chưa đóng;
  tổng số bộ chưa biết thì ghi rõ.
- **Tiếp theo:** bộ sẽ chạy hoặc phần phải sửa trước khi chạy; lý do thêm test nếu có.

Full chỉ được xác nhận khi không còn loại/thuộc tính trong phạm vi thiếu adapter
hoặc `unverified`, mọi incompatibility V5 có bằng chứng, và binary cuối qua
nghiệm thu. Kết quả một bộ đo không tự đóng toàn bộ đợt.

## Copy/remap regression completed — 2026-10-08

18 fixture độc lập:36/36 trường hợp native và18/18 vòng FreeCAD đạt;2.611 kiểm
tra native và612 FreeCAD. Regression binary45/45 suite,34 báo cáo/5.343 kiểm tra;
tools42/42. Bộ này kiểm chứng graph nội bộ SDK80 và từ chối V5 an toàn, không
đổi12+6 trường hợp Rhino5 không tương thích thành đạt.0 bộ còn trong hàng chờ
đã chuẩn bị tại checkpoint này;7 đợt (2–8) chưa đóng; tổng bộ test tương lai chưa
xác định. Tiếp theo ma trận seam/cực/đường khép kín, rồi singular trim mapping.

## Seam/cực/periodic mới — 2026-10-08

1 bộ đã chuẩn bị đang chờ Rhino5:8 fixture,8/8 native(229 kiểm tra) và8/8 FreeCAD(32 kiểm tra) đạt. Chưa có kết quả Rhino hoặc kiểm tra decode sau SaveAs. Bộ này bổ sung phạm vi seam và hai đầu cực còn thiếu trong ma trận trước; không tính là chạy lại bộ cũ.7 đợt(2–8) chưa đóng;tổng bộ còn lại chưa xác định.

Seam preparation checkpoint2026-10-08:8/8 native(229 checks),8/8 FreeCAD(32 checks), actual Rhino5 pending. Current regression46/46 suites,35 reports/5375 checks; same verified production binary.1 prepared pending batch,7 packages(2–8) open,total future count unknown. Source/fixture bindings verified; no primary integration or push. [Evidence](2026-10-08-seam-profiles-preparation.md).

## UV-reference owner/copy/remap — 2026-10-08

8/8 fixture native895 và FreeCAD272 kiểm tra đạt. Đã sửa dựng graph lúc ghi và giữ nguyên copy-count userdata; dữ liệu nguồn bất biến. Bộ này bổ sung UV2D tham chiếu owner riêng, chưa có bằng chứng Rhino.1 bộ đã chuẩn bị đang chờ,7 đợt chưa đóng,tổng số bộ còn lại chưa xác định. Regression48/48 native;37 báo cáo/5696 kiểm tra;tools42/42 đạt. [Bằng chứng](2026-10-08-uv-reference-remap.md).

## Actual Rhino5 UV-reference8 — 2026-10-08

Đã đọc8/8,0 đạt/8 không đạt:8 owner hợp lệ nhưng8 root UnsetPoint,8 SaveAs thất bại;không có bản lưu để nhập ngược. Native507 kiểm tra nguồn/graph/từ chối V5 atomic đạt. Bộ đóng với incompatibility có phạm vi.0 bộ chuẩn bị đang chờ,7 đợt chưa đóng,tổng còn lại chưa xác định. [Bằng chứng](2026-10-08-uv-reference-rhino5.md).


Native Qt menu regression (`native-qt-exchange-user-ring-crash`): 2 fixtures (new oval Import and independent box Export),8 QAction checks +18 UI checks passed after GIL fix. Actual ring source preserved;52 host geometries valid.0 prepared Rhino batches,7 packages open,total future batches unknown. [Evidence](2026-10-08-native-menu-gil-fix.md).


`oval-wireframe-display-diagnosis`:1 fixture, read-only native basis and GUI comparison completed; display polygon-line limitation confirmed, no renderer fix claimed.0 prepared Rhino batches,7 packages open,total future batches unknown. [Evidence](2026-10-08-oval-display-diagnostic.md).

## Wireframe CAD — 2026-10-08

Chẩn đoán polygon-line tại checkpoint trên đã được xử lý có phạm vi: BRep dùng
native CAD edges +trimmed isocurves; giữ geometry/source và mode theo view.
Ma trận CAD13 fixture và hồi quy mục tiêu:8 báo cáo FreeCAD/105 kiểm tra đạt
trên runtime mới, gồm menu Qt và hai file nhẫn.0 bộ Rhino mới đã đo.
Checkpoint geometry51 suites cũ không được tính là chạy lại trên binary này.
0 bộ đã chuẩn bị đang chờ,7 đợt chưa đóng; tổng bộ còn lại chưa xác định.
[Kết quả và giới hạn](2026-10-08-cad-wireframe.md).

## Import nhanh — 2026-10-08

Bộ `fast-preservation-import-and-regressions` thuộc đợt8: một nhẫn đo2 lượt
paired, một nguồn hỗn hợp19 record và targeted regressions;12 báo cáo/
151 kiểm tra đạt. Import trung vị 25.85→13.71s;
bao gồm display 29.23→16.82s.
0 bộ Rhino chờ,7 đợt chưa đóng,tổng bộ tương lai chưa xác định.
[Kết quả, giới hạn](2026-10-08-fast-3dm-import.md).

2026-10-08 Working modeling Phase1: complete_scoped;9 runtime reports135 checks,10 targeted native suites,11 SDK20130711 cases. Actual Rhino5 UI unverified. Evidence: modeling-phase-1/summary.json and2026-10-08-modeling-phase-1.md. Prepared full-archive queue remains0;7 packages remain unclosed; total future full openNURBS batches unknown. Phase2–5 not completed by this checkpoint.

2026-10-08 Working modeling Phase1: complete_scoped;9 runtime reports143 checks,10 targeted native suites,11 SDK20130711 cases. Actual Rhino5 UI unverified. Evidence: modeling-phase-1/summary.json and2026-10-08-modeling-phase-1.md. Prepared full-archive queue remains0;7 packages remain unclosed; total future full openNURBS batches unknown. Phase2–5 not completed by this checkpoint.

2026-10-09 Phase1 actual application accepted:10 actual Rhino5 completed cases (ring369 checks);4 actual FreeCAD saved-output lifecycle reports230 checks.19 working-mode requirements now proved. Diagnostic125 excluded from accepted count;7 full-exchange packages remain open. No Phase2–5/full/primary integration claim. Evidence: modeling-phase-1/application-evidence.json.

## Phase2 accepted — 2026-10-09

P2.1–P2.4 and actual Rhino5/FreeCAD curve application gate accepted on the final matching manifest. 24 bound host reports/437 checks; Rhino3 fixtures/125 checks; native49 suites; Rust88; Python58. One final review and one branch-loss fix pass; two Minor findings deferred. Prepared pending batches0;7 full openNURBS packages still open; total future batch count unknown. Next: Phase3 BRep modeling. See modeling-phase-2/summary.json and final-review.md.

2026-10-09 Phase2 Rust migration accepted_scoped: Rust sở hữu basis/session/Join–Rebuild decisions/index/cache/budget/ranking; Qt/C++ giữ bridge native, bỏ production Python PointsOn. Rust112/Python58/native50;30 host reports507 checks; actual Rhino5 3 fixtures125 checks +saved reread55 checks. Runtime riêng om9-phase2-rust-sdk; baseline lịch sử được giữ. Một final review, mọi Critical/Important đã giải quyết; xem modeling-phase-2-rust/final-review.md. Skill Rust-first7 skill/17 bản hợp lệ. Prepared pending0,7 full openNURBS packages open, tổng future batches unknown; Phase3–5 chưa hoàn tất.

## Rhino5 Phase2 manual entrypoint — 2026-10-09

Batch kiểm chứng entrypoint từ Rhino có bản vẽ chưa lưu: 3 fixture đạt125 kiểm tra Rhino,35 host OM9,55 nhập lại SaveAs;8/8 bảo toàn phiên gọi đạt;tools58/58. Runtime Rust và report SaveAs cùng lần chạy được bind bằng hash. [Cách chạy và bằng chứng](2026-10-09-rhino5-phase2-manual-verification.md).0 batch đã chuẩn bị còn chờ;7 gói full openNURBS vẫn mở;tổng batch tương lai chưa xác định. Tiếp theo người dùng chạy script trên Rhino của mình;phạm vi sản phẩm tiếp theo Phase3 BRep modeling.

Lần chạy thủ công của người dùng đã PASS3/3 cùng runtime. Mở rộng batch entrypoint bằng1 assertion STA: RED bắt đúng cảnh báo Thread.Sleep; sau đổi timed Join, GREEN9/9 và warnings=[];geometry125/35/55 vẫn đạt,tools58/58. Không cộng retry thành batch mới;0 chuẩn bị còn chờ,7 gói mở,tổng tương lai chưa xác định.

## Phase1/2 performance baseline — 2026-10-09

3 batch hiện hữu đo lại/4 dataset chính/48 kiểm tra đạt: mesh20k bulk48,84ms,nhẫn29 root60%3,448s,warm snap p95 CAD10k0,0203ms/million mesh-cloud0,0667ms. Đây là baseline, chưa code tối ưu mới. [Audit và thiết kế bounded đề xuất](2026-10-09-phase12-performance-audit.md).0 bộ chuẩn bị còn chờ;7 gói mở;tổng tương lai chưa biết. Tiếp theo duyệt thiết kế compact JSON và shared Rust snap cache trong API hiện hữu.

## Phase1/2 bounded performance verified —2026-10-09

Compact request11.38%smaller; shared safeRust cache4096~60xcore/dense8192~18%core. No wholeCopyPastewallgain claim; RAMmetadata+0.625MiB. Rust114/tools60/native50,38hostreports,actualRhino14fixtures621checks,bothsaved175.0prepared pending;7packagesopen,totalfutureunknown. [Evidence](2026-10-09-phase12-performance-complete.md).

## User Rhino5 Phase1/2 verifier —2026-10-09

Existing scoped14fixture batch replayed through one manual entry; actualRhino621,38hostreports,saved175,caller-session safety verified. Diagnostic2fixture85checks and retained failing probes are separate attempts, not new implementation completion.0preparedpending;7packagesopen;futuretotalunknown. [Evidence](2026-10-09-rhino5-phase12-user-script.md).

## Clipboard fixture regression / user verifier replay — 2026-10-09

User failure retained; captured native busy-lock RED, stable-fixture80 shortcuts GREEN, targeted6reports/97checks. Fresh full manual entry14fixtures621Rhino checks,38hostreports/770checks,saved175,caller10/10 unchanged. Product/runtime hashes unchanged.0preparedpending for this correction;7packagesopen;futuretotalunknown. [Evidence](2026-10-09-rhino5-phase12-user-script.md).

## User verifier retry0820a4cc — external clipboard replacement

Phase2:3fixtures125Rhino/35host/55saved checks PASS. Clipboard8/11 fixtures complete; current-extrusion producer13/13 PASS, sequence11368Rhino payload replaced by sequence11383Explorer file clipboard. Latest execution FAIL retained; previous full PASS remains historical.1existing14fixture replay prepared/pending;7packagesopen;futuretotalunknown. Next: run same entrypoint while avoiding other Copy/Cut. [Evidence](2026-10-09-rhino5-phase12-user-script.md).

## User replay ca077635 — PASS

14fixtures/621Rhino,38OM9 reports/770checks,175saved reread. External-clipboard replay complete; historical failures retained.0prepared replays for this correction. Slow Copy profiling is separate;7packagesopen;futuretotalunknown. [Evidence](2026-10-09-rhino5-phase12-user-script.md).

## Phase1/2 timing instrumentation — 2026-10-09

3fixtures/204Rhino,3OM9reports/104checks,54events/12CopyPaste totals PASS;66tools/12readerPASS. Full updated14fixture entry prepared/pending; prior ca077 full proof retained. Product unchanged, no speedup claimed.7packagesopen;futuretotalunknown. [Evidence](2026-10-09-phase12-copy-timing.md).

## Rhino5 live reader correction — 2026-10-09

User9974caller TypeError and child script-change rejection retained as FAIL; reported14/621/38/770/175positive assertion counts are not full acceptance. Actual Rhino reader RED→GREEN:1log fixture/12checks;tools67/reader12PASS.1existing14fixture full replay prepared/pending;7packagesopen;futuretotalunknown. [Evidence](2026-10-09-phase12-copy-timing.md).

## Rhino5 live reader correction — 2026-10-09

User9974caller TypeError and child script-change rejection retained as FAIL; reported14/621/38/770/175positive assertion counts are not full acceptance. Actual Rhino reader RED→GREEN:1log fixture/12checks;tools67/reader12PASS.1existing14fixture full replay prepared/pending;7packagesopen;futuretotalunknown. [Evidence](2026-10-09-phase12-copy-timing.md).


Full timing-entry replay 4f741966: PASS 14 fixtures / 621 Rhino / 38 OM9 reports / 770 OM9 / 175 saved reread checks. Logging: 198 events, 44 command totals, no errors. Ring OM9 Copy 16.146s (native encode/validate 14.925s); finer native and publication profiling remains next. Zero prepared replays for the reader correction; seven full-openNURBS packages open, future total batches unknown. See [timing evidence](2026-10-09-phase12-copy-timing.md).


Phase3 current-state audit 2026-10-09: written Rust-owned design awaits review. All 18 requirement groups remain unproven on a Phase3 runtime. Accepted Phase1/2 baseline source/runtime hashes rechecked; no product change or measurement batch. Surface/Edit integration, stale surface input checks and open-shell Boolean policy remain required. Zero executable Phase3 batches prepared; seven full-openNURBS packages open; total future batches unknown. [Audit](modeling-phase-3/current-state-audit.json).

Phase3 implementation: gate15 fixture/599 Rhino/480 OM9 checks PASS (pre-final attempt4); clipboard11/496 Rhino/218 OM9 +120 saved reread PASS on matching Phase3 runtime. Invalid-CAD supplemental check17 PASS. Final15 fixture gate running; one whole-change review and18-group audit pending. One executable batch prepared/running; seven broader packages open; total future batches unknown.

Final Phase3 batch15: PASS, no stopped cases. Exact native mass/bounds/topology and actual app lifecycle verified. Clipboard regression11: PASS, all exact Rhino saved files reread. Prepared pending batches0; unclosed broader packages7; future total unknown.

Matrix Rust palette gate `matrix-rust-palette-native-gate-20261010`: one prepared native batch, five geometry fixtures (two Matrix + three OM9), full palettes/empty/nested layers and two injected failure stages. Preparation Rust9 + managed ABI + IronPython3/harness10 PASS, not native acceptance. Recovery30c4 actual PASS removed five objects/30 layers, preserved observed34 layers. Seven broader packages remain unclosed; complete future batch count unknown. [Preparation](layer-session/matrix-rust-palette-preparation.json).

Native matrix-rust-palette-native-gate-20261010 accepted for user-approved OM9-only Undo scope: all16 checks PASS on run7c9, summary cache bug reproduced/fixed. Original report preserved and acceptance recorded separately. Original layer-session Tasks1-5/primary UI deployment and broader packages remain incomplete. [Proof](layer-session/palette-native-acceptance.json).


## Stock FreeCAD 1.1.4 compatibility — 2026-10-10

10 native runtime reports, 217 checks passed, including eight BRep fixtures. No fresh Rhino/Matrix cases. Full exchange remains incomplete; seven packages remain open/in progress, and the total future batch count remains unknown. Release packaging and user-folder discovery are the next verification steps. See [evidence](2026-10-10-stock-freecad-1.1.4.md).
