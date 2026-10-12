# Development release notes — 2026-10-12

[Releases](https://github.com/MYuitsu/OpenMatrix9/releases) · [Tiếng Việt](../../README.vi.md) · [English](../../README.en.md)

This archive preserves the previous detailed README: command tables, development
checkpoints, native test counts and limitations. It contains Vietnamese and English
notes. Each checkpoint applies to its recorded revision and host; these development
results do not certify a downloaded release binary. Published version-specific
results belong to the corresponding GitHub Release.

## Detailed development record

# OpenMatrix9

<!-- common-command-mnemonics-20261012:start -->
2026-10-12: quy tắc gạch chân một ký tự áp dụng qua engine Rust chung cho toàn
bộ schema Command Options hiện có: Line/Polyline, InterpCrv, Rectangle, Circle,
Ellipse, Rebuild, Box/Sphere, Sweep1/History, Sweep2/History và Loft. Tùy chọn con
khai báo alias trong phase cha; tên đầy đủ vẫn click được. Alias trùng hoặc
disabled không được quảng cáo bằng gạch chân. Lệnh mới đăng ký schema để dùng
cùng resolver, renderer và binding CMD–Builder. Trạng thái toàn feature còn partial.
[Bằng chứng native](../../docs/validation/2026-10-12-common-command-mnemonics.md).
<!-- common-command-mnemonics-20261012:end -->

<!-- command-mnemonics-20261012:start -->
2026-10-12: chỉ gạch chân một ký tự phím tắt đã khai báo, không gạch chân cả
tên. Rust cung cấp mnemonic; cả tên vẫn click được, clipboard giữ nguyên.
Box dùng D/3/V/C; phím3 chỉ chọn3Point trước điểm đầu, sau đó vẫn nhập số3.
Focus,0 origin và lịch sử dễ đọc đã sửa vẫn giữ. Supported status còn partial.
[Bằng chứng native](../../docs/validation/2026-10-12-command-mnemonics.md).
<!-- command-mnemonics-20261012:end -->

<!-- box-command-usability-20261012:start -->
2026-10-12: Box icon trái/phải focus CMD; `0` ở bước nhập điểm là gốc CPlane
và chuyển vào lịch sử khi được nhận. Width/Height giải thích Enter/default mm.
Lỗi ghi riêng vào lịch sử, prompt sạch và draft sai vẫn sửa được; giới hạn
geometry giữ nguyên. Focus có guard generation sau cancel; common Curve input
evidence giữ điểm gốc. Đây là sửa supported slice, status vẫn partial.
[Bằng chứng native và giới hạn](../../docs/validation/2026-10-12-box-command-usability.md).
<!-- box-command-usability-20261012:end -->

<!-- matrix-phase-options-20261012:start -->
2026-10-12: engine Rust chung có question/options theo phase, span UTF-16,
ẩn/hiện độc lập gate và revision refresh. Box áp dụng first-corner → Diagonal
Cube → Width/Height defaults; click và D/d dùng cùng native solver. CMD xanh ghi
invocation thật, preceding prompt và input đã nhận; Up/Down recall, Esc và draft
lỗi giữ đúng lifecycle. Đây là supported slice, không hoàn tất mọi command,
Gem Builder hay associative History.
[API dùng chung](../../docs/command-options-adapter.md) · [Bằng chứng native](../../docs/validation/2026-10-12-box-matrix-command-options.md).
<!-- matrix-phase-options-20261012:end -->

<!-- common-command-options-20261012:start -->
2026-10-12: Command Line Options dùng schema/reducer Rust chung và binder Qt
cho các consumer Curve, Rebuild, Sweep/Loft hiện có; lệnh mới đăng ký schema
và adapter theo exact feature ID. Giữ RCORE-04/Builder Framework ở phạm vi
`partially_implemented`; không mở Gem solver hoặc command chưa hỗ trợ.
[API dùng chung](../../docs/command-options-adapter.md) ·
[Bằng chứng native và giới hạn](../../docs/validation/2026-10-12-command-line-options.md).
<!-- common-command-options-20261012:end -->




<!-- rhino-core-snapshot-history-disposal-20261012:start -->
FreeCAD replay kiểm tổng history/property records và dung lượng snapshot/tên trước khi sao chép; dùng lại bảng transaction descriptions với borrowed view đồng bộ. Hủy Undo/Redo dùng FlagToggler, chặn reentrant mutation và pop record trước destructor; nhánh visible Redo của failed preserving group cũng được bảo vệ.
**App273/plugin275/kernel240:3372/3372 kiểm tra hành vi qua34 host;33/34 host sạch log.** Một host close-object có startup Unhandled std::exception trước fixture, nên runtime gate chưa đạt toàn bộ. Snapshot38, normal-close finalizer16 và empty-group Redo5 đạt; các lượt diagnostic278/280 không cộng vào tổng. Rust toàn bộ lần gần nhất1037/0, không chạy lại cho thay đổi native này.
**RCORE-01 vẫn đang triển khai:** startup deadlock, peak/opaque allocation, lifecycle và phần còn thiếu của hợp đồng gốc; các checklist lịch sử cần đối chiếu theo evidence, không tự coi mọi fixture đã đạt là chưa làm.
[Bằng chứng snapshot/history](../../docs/validation/2026-10-12-rhino-core-snapshot-history-disposal.md).
<!-- rhino-core-snapshot-history-disposal-20261012:end -->

<!-- rhino-core-acceptance-offset-d3-20261012:start -->
Theo thiết kế người dùng duyệt, identity/Redo được giữ khi từ chối trước acceptance; destructor chạy sau acceptance dưới barrier và báo disposal diagnostic riêng. Circle History giữ publication hợp lệ khi logger ném lỗi.
**App247/Runtime254/kernel240:3313/3313 kiểm tra qua28 host**; boundary162, logger24. OCCT D3 helper/direct curve/adaptor RED4/12→GREEN12/12; đủ19 họ/183 query,36 D3 được sửa và45 offset khớp đạo hàm độc lập. **Rust toàn bộ1037 đạt/0 lỗi.**
**RCORE-01/Task2/3 vẫn đang triển khai:** aggregate budgets, global participants, lifecycle và toàn bộ hợp đồng gốc còn mở. Witness destructor-veto cũ giữ nguyên, đã được thay tiêu chí bởi thiết kế duyệt; các checkpoint dưới là lịch sử.
[Bằng chứng acceptance/D3](../../docs/validation/2026-10-12-rhino-core-acceptance-offset-d3.md).
<!-- rhino-core-acceptance-offset-d3-20261012:end -->

<!-- rhino-core-native-retirement-verdict-20261012:start -->
App227/Runtime229 dùng lại transaction, native Property và Python exception của FreeCAD. Finalizer chạy trước verdict của commit/cleanup; callback bị chặn không còn báo thành công. Giữ inverse nếu retirement trước đó đã bị từ chối.
**3145/3145 assertion qua28 host** trên runtime tương ứng, gồm Python identity/private inverse72/201, finalizer6, frozen source180, native journal54, verdict50, cleanup14 và inverse retention12.
**RCORE-01/Task2/3 vẫn đang triển khai.** Test gốc Python final release 5/8 và detached object 16/32 còn RED; Rust121 đạt/1 lỗi OffsetBezier D3. Các checkpoint phía dưới là bằng chứng lịch sử.
[Bằng chứng retirement verdict](../../docs/validation/2026-10-12-rhino-core-native-retirement-verdict.md).
<!-- rhino-core-native-retirement-verdict-20261012:end -->

<!-- rhino-core-native-value-snapshots-20261012:start -->
App191/Runtime193 giữ identity native cho Link/XLink, subname/shadow/partial flags và cây biểu thức/validator; dùng lại setters/Paste của FreeCAD. Frozen journal giữ thao tác rename và refresh các entry mới/canceled. Prepared phase và cleanup guard chặn callback trong merge unwind; commit retry bị từ chối trước null journal.
Đã kiểm chứng **2607/2607 assertion qua21 host**, gồm Link128, XLink112, expression60, rename32, merge-unwind32, retry12, native journal51 và hồi quy/cold reload.
**Còn RED:** detached-object disposal16/32 và Rust121pass/1fail (raw native offset D3). Python property identity/orphan Save, frozen-source preparation và toàn bộ **Task2/3/full RCORE-01 còn mở**.
[Bằng chứng native value snapshot](../../docs/validation/2026-10-12-rhino-core-native-value-snapshots.md).
<!-- rhino-core-native-value-snapshots-20261012:end -->

<!-- rhino-core-native-persistence-inverse-20261012:start -->
App154 giữ inverse bằng buffer native Save/Restore và seal merge source; mọi private Property được giải phóng/kiểm tra trước caller transfer.
Final recovery disposal **204/204**, original registered retirement **204/204**;32 callback/case mỗi cohort. Native inverse/source retirement/seal/budget/dynamic tests **40/40**.
Runtime156/App154 đạt **1988 kiểm tra/13 host sạch**, gồm semantic432,close4,modes109,Curve33,booking80,scope744 và cold/legacy/spatial.
**Discarded regular Redo vẫn RED12/28**:4 callback bị chặn và không báo accepted, nhưng original booking/Redo và paired cleanup chưa phục hồi. **Task2/3/full RCORE-01 còn mở.**
[Bằng chứng native persistence inverse](../../docs/validation/2026-10-12-rhino-core-native-persistence-inverse.md).
<!-- rhino-core-native-persistence-inverse-20261012:end -->

<!-- rhino-core-native-retirement-20261012:start -->
App147 có native factory/Paste inverse; original Copy retirement được kiểm tra trước caller merge.
Original retirement **204/204**, registered native subtype **204/204**;32 callback/case mỗi cohort, phục hồi và giữ caller history.
Matching Runtime149/App147: **1982 kiểm tra/13 host sạch**, gồm semantic432,close4,modes109,Curve33,booking80,capture34,scope744,cold modes62,legacy8/cold3,spatial65.
**Final recovery disposal vẫn RED128/204**:32 callback bị chặn và không báo accepted, nhưng rollback/paired cleanup chưa hoàn tất. **Task2/3/full RCORE-01 còn mở.**
[Bằng chứng native retirement](../../docs/validation/2026-10-12-rhino-core-native-retirement.md).
<!-- rhino-core-native-retirement-20261012:end -->

<!-- rhino-core-native-capture-booking-20261012:start -->
Native journal capture đã chuẩn bị metadata/copy trước khi ghi entry; **34/34** kiểm tra, gồm first/later/removal Copy failure và retry.
Regular booking không khôi phục Redo đã loại bỏ sau accepted replay/abort; **80/80** so sánh hành vi FreeCAD native ở0/1,plain/booked/active/preserving.
Runtime138/App141 đạt lại **432 semantic +4 owner-close +109 modes +33 Curve**; tổng **692 assertion/6 host0**;5 host sạch, capture34 có initial macro deadlock.
String scope744 vẫn có deadlock lúc xử lý macro khởi động, **không phải bằng chứng lỗi shutdown**. Retirement callbacks, global participants/budgets và full RCORE-01/Task2/3 còn mở.
[Bằng chứng native capture/booking](../../docs/validation/2026-10-12-rhino-core-native-capture-booking.md).
<!-- rhino-core-native-capture-booking-20261012:end -->

<!-- rhino-core-circle-native-savepoint-20261012:start -->
Circle History recipe-v2/legacy đã dùng native savepoint trước khi đọc input, qua rollback và paired cleanup hoặc verified commit.
Runtime131/App125 đạt **432/432 semantic**, **4/4 owner-close**, **109/109 Circle modes**, **33/33 Curve result**; 4 host thoát0 và không có unhandled shutdown/deadlock.
App DLL/import được kiểm tra hash và chọn riêng; dùng SDK/cache hiện có.
**RCORE-01/Task2/3 vẫn mở**: capture Copy failure, retirement callbacks, exact booking/Redo policy, native String-scope shutdown failure và các yêu cầu gốc còn lại.
[Bằng chứng Circle native savepoint](../../docs/validation/2026-10-12-rhino-core-circle-native-savepoint.md).
<!-- rhino-core-circle-native-savepoint-20261012:end -->

<!-- rhino-core-native-replay-scope-20261012:start -->
Document scope prototype có private journal/observer latch,owner/history guards,
checked rollback/error cleanup và staged merge. UndoMode Python stub được nối mode0/1 thật.
Named **744/744 assertions**,104 case;Curve33/Circle109 đạt trên App125.
**GUI shutdown vẫn có unhandled std::exception/resource deadlock: chưa clean runtime gate.**
Circle chưa tích hợp scope;atomic **RED355/432**,48 callback;SDK DLL/lib/PDB giữ nguyên.
**Savepoint/Task2/3 và toàn bộ RCORE-01 còn mở.**
[Bằng chứng native replay scope](../../docs/validation/2026-10-12-rhino-core-native-replay-scope.md).
<!-- rhino-core-native-replay-scope-20261012:end -->

<!-- rhino-core-native-property-merge-20261012:start -->
Native Transaction có staged property merge: giữ caller record/first-write/ownership,
chuẩn bị copy và kiểm stale trước transfer. Native API RED0/1→GREEN29/29;
App117 build0;Curve33+Circle109 = **171/171 final positive**,3 host exits0.
SDK DLL/lib/PDB giữ nguyên;không rebuild root hoặc clone runtime.
History atomic vẫn **RED355/432**,48 callback. **Document scope/booking/rollback/lifetime,
Task2/3 và toàn bộ RCORE-01 còn mở.** Không chạy lại Rust ở checkpoint native này.
[Bằng chứng native property merge](../../docs/validation/2026-10-12-rhino-core-native-property-merge.md).
<!-- rhino-core-native-property-merge-20261012:end -->

<!-- rhino-core-initial-curve-owned-contacts-20261012:start -->
Initial Tangent: giữ tiếp điểm solver native tách khỏi probe; Rust kiểm owned D0/D1,
circle/plane/tangent/FromFirst/fixed null và output analytic/Deformable trước publication.
Focused7/7;full Rust **989 đạt/1 lỗi D3 cũ**. Runtime112 build0,native **522/522**,
11 exits0;contact sources khớp,full current-source matching false do test wrapper và sửa song song.
Native savepoint đã được người dùng duyệt;baseline mới vẫn **RED355/432**,48 callback.
**RCORE-01/Task2/3 và mọi yêu cầu gốc còn mở.**
[Bằng chứng initial owned contacts](../../docs/validation/2026-10-12-rhino-core-initial-curve-owned-contacts.md).
<!-- rhino-core-initial-curve-owned-contacts-20261012:end -->

<!-- rhino-core-initial-curve-owned-parameters-20261012:start -->
Initial AroundCurve/Tangent: native fraction được giữ; Rust kiểm owned D0/unitD1,
điểm/hướng, selected edge và path trên branch Undo. RED1/3→GREEN5/5;
183 query/19 family admission;full Rust **982 đạt/1 lỗi D3 cũ**.
Runtime109 build0,native **417/417**,8 host exits0;1498 inputs match.
**RCORE-01/Task2/3 vẫn mở: initial solved contacts, generic snap,
atomic/lifetime và mọi yêu cầu gốc còn thiếu.**
[Bằng chứng initial owned parameters](../../docs/validation/2026-10-12-rhino-core-initial-curve-owned-parameters.md).
<!-- rhino-core-initial-curve-owned-parameters-20261012:end -->

<!-- rhino-core-initial-curve-owned-selection-20261012:start -->
Initial Curve: dùng chung Rust owned selection với History trước khi đổi session;
CAD FitPoints và native Vertex/Direction/fixed-point được ràng buộc nguồn.
Giữ subset indices cloud/mesh; tách local basis1e150 và world command1e9.
Rust RED1/4→GREEN5/5;full **977 đạt/1 lỗi D3 cũ**. Native local/world RED3/4→4/4.
Runtime106 build0,native **360/360**,6 host exits0;1496 inputs match.
**RCORE-01/Task2/3 vẫn mở: initial path/Tangent, generic edge/face snap,
atomic/lifetime và mọi yêu cầu gốc còn thiếu.**
[Bằng chứng initial owned selection](../../docs/validation/2026-10-12-rhino-core-initial-curve-owned-selection.md).
<!-- rhino-core-initial-curve-owned-selection-20261012:end -->

<!-- rhino-core-circle-history-owned-fit-geometry-20261012:start -->
Circle History: Rust kiểm selected geometry theo surface poles→curve poles→native vertices;
hỗ trợ FaceN, whole/prefix, rational/periodic NURBS/Bezier, cloud/mesh và world once.
Native Face RED62/70→GREEN70/70; cold10 family50/50. Rust focused7/7,
Curve result **56/56**, full **972 đạt/1 lỗi D3 cũ**. Runtime102 build0;
native **426/426**,6 host exits0;production inputs match,hai test files đổi sau build.
**RCORE-01/Task2/3 vẫn mở: initial native input/Tangent,full families,
atomic/lifetime,conversion/maps/budgets và mọi yêu cầu gốc còn thiếu.**
[Bằng chứng selected geometry](../../docs/validation/2026-10-12-rhino-core-circle-history-owned-fit-geometry.md).
<!-- rhino-core-circle-history-owned-fit-geometry-20261012:end -->

<!-- rhino-core-circle-history-owned-role-replay-20261012:start -->
Circle History: Rust dựng lại điểm/cấu hình replay từ record và nguồn đã chọn;
giữ whole FitPoints tăng điểm, prefix và numeric gaps; chặn sai Direction/radius/spans.
Role RED0/6→GREEN7/7; nhóm Curve result **50/50**. Full Rust **965 đạt/1 lỗi D3 cũ**.
Runtime97 build0, History/cold/spatial **236/236**,3 host exits0;1486 inputs match.
**RCORE-01/Task2/3 vẫn mở: selected CAD poles/vertices/Face provenance,
initial Tangent,atomic/lifetime và toàn bộ yêu cầu gốc còn thiếu.**
[Bằng chứng stored role replay](../../docs/validation/2026-10-12-rhino-core-circle-history-owned-role-replay.md).
<!-- rhino-core-circle-history-owned-role-replay-20261012:end -->

<!-- rhino-core-circle-history-owned-contact-binding-20261012:start -->
Circle History giữ fraction tiếp điểm native vàRust kiểm D0/D1 độc lập:
circle/plane/tangent/FromFirst;probe vàcontact vẫn riêng. Contact **5/5**,source23/23,
19 family/183 query admission. Full Rust **958 đạt/1 lỗi D3 native còn nguyên**.
Runtime93 build0,native History/cold/nonplanar **236/236**,3 host exits0.
**RCORE-01 vàTask2/3 vẫn mở;role khác,initial Tangent,atomic/lifetime vàyêu cầu gốc còn thiếu.**
[Bằng chứng owned contact binding](../../docs/validation/2026-10-12-rhino-core-circle-history-owned-contact-binding.md).
<!-- rhino-core-circle-history-owned-contact-binding-20261012:end -->

<!-- rhino-core-circle-history-owned-source-binding-20261012:start -->
Circle History đã nối owned Rust D0/D1 vào path/probe/Point source binding.
Ba lỗi nguồn cũ đã sửa;source **23/23**,19 family/183 query qua edge/world và admission.
Full Rust **953 đạt/2 lỗi**:D3 native cũ và ca mới thiếu kiểm tiếp điểm Tangent.
Runtime91 build0,native History/cold **171/171**,2 host exits0.
**RCORE-01 và Task2/3 vẫn đang triển khai;contact/atomic/lifetime và yêu cầu gốc còn mở.**
[Bằng chứng owned source binding](../../docs/validation/2026-10-12-rhino-core-circle-history-owned-source-binding.md).
<!-- rhino-core-circle-history-owned-source-binding-20261012:end -->

<!-- rhino-core-owned-derived-curve-evaluation-20261012:start -->
Owned Rust Taylor/trim/offset:183 query D0–D2 native khớp;45 D3 offset
khớp oracle toán70 chữ số,nhưng36 khác D3 host;giữ raw native D3 RED.
Focused10/11,full Rust **943 đạt/4 lỗi**. Runtime86 build0,native capture/History/cold
**229/229**,3 host exits0. Chưa nối source admission;mọi yêu cầu gốc còn mở.
**RCORE-01 và Task2/3 vẫn đang triển khai.**
[Bằng chứng derived evaluator](../../docs/validation/2026-10-12-rhino-core-owned-derived-curve-evaluation.md).
<!-- rhino-core-owned-derived-curve-evaluation-20261012:end -->

<!-- rhino-core-owned-curve-evaluation-20261012:start -->
Owned Rust curve evaluator: 138 điểm/đạo hàm OCCT trên14 family khớp;
6/6 focused, full Rust **939 đạt/3 lỗi nguồn còn nguyên**. Runtime79 build0;
oracle43 + History109 + cold62 = **214/214**,3 host exits0. Chưa nối evaluator
vào source admission; derived support/contact/maps/budgets/lifetime còn mở.
**RCORE-01 và Task2/3 vẫn đang triển khai.**
[Bằng chứng evaluator](../../docs/validation/2026-10-12-rhino-core-owned-curve-evaluation.md).
<!-- rhino-core-owned-curve-evaluation-20261012:end -->

<!-- rhino-core-circle-history-source-role-fixes-20261012:start -->
Circle History: Rust đã chặn role/path thiếu hoặc trùng, source Point giả vai curve,
span Point/Curve chồng nhau và fixed point ngoài circle/plane; giữ native tolerance.
Probe AroundCurve cũ và anchor replay được ghi riêng. Native **171/171**,2 exits0;
Rust toàn bộ **933 đạt/3 lỗi**, nhóm source **17/20**. **RCORE-01/Task2/3 vẫn mở.**
[Bằng chứng source role fixes](../../docs/validation/2026-10-12-rhino-core-circle-history-source-role-fixes.md).
<!-- rhino-core-circle-history-source-role-fixes-20261012:end -->

<!-- rhino-core-circle-history-source-binding-red-20261012:start -->
Circle History: Rust nguồn/parameter/constraint **RED4/10**,exit101;
ba payload dựng lại từ snapshot native thật đạt,nhưng sáu ca sai fraction/role/
path duplicate/điểm ngoài curve/đổi nguồn đầy đủ/fixed point ngoài circle vẫn được
nhận. Point oracle đầu bị chặn bởi document token khác đã sửa đúng;không bịa stamp.
Chỉ thêm test/data;production chưa đổi. **RCORE-01 vàTask2/3 vẫn mở.**
[Bằng chứng source binding](../../docs/validation/2026-10-12-rhino-core-circle-history-source-binding-red.md).
<!-- rhino-core-circle-history-source-binding-red-20261012:end -->

<!-- rhino-core-circle-history-lifetime-red-20261012:start -->
Circle History: callback đóng được document sở hữu trong replay; prefix
`after_close`/owner absent đã lưu. Recompute không trả về trong300s; runner kết
thúc đúng host thử nghiệm vàxác nhận terminal. **Đây là witness lỗi lifetime,
không phải suite đạt hay mã crash đã biết.** Có thiết kế native savepoint đề xuất;
production chưa đổi. **RCORE-01, Task2/3 vàmọi yêu cầu gốc vẫn mở.**
[Bằng chứng lifetime vàthiết kế](../../docs/validation/2026-10-12-rhino-core-circle-history-lifetime-red.md).
<!-- rhino-core-circle-history-lifetime-red-20261012:end -->

<!-- rhino-core-circle-history-atomic-state-20261012:start -->
Circle History: đo **trước Redo**,48 ca/ba trạng thái caller,
**RED355/432**, host exit0 vàkhông lỗi fixture. Metadata sai42/48; nguồn vertex
bị sửa3/48;16 booking chưa materialize phát sinh journal/pending mới. Redo gốc
thực thi48/48 sau caller abort. BREP khác do identity location45/48, không phải
nguồn đổi hình học. **RCORE-01 vàTask2/3 vẫn mở; chưa sửa production.**
[Bằng chứng rejection-time](../../docs/validation/2026-10-12-rhino-core-circle-history-atomic-state.md).
<!-- rhino-core-circle-history-atomic-state-20261012:end -->

<!-- rhino-core-circle-history-modes-20261012:start -->
Circle History, cùng Runtime59: **182/182 native bổ sung**, bốn host exit0.
All-mode109, cold62 với20 result, legacy8 vàlegacy cold3 đạt; không sửa solver
hoặc rebuild. **Atomic rejection RED50/80**: phát hiện16 callback vàxóa cặp
Shape/result, nhưng14 ca giữ metadata bị sửa; kiểm source/Undo stack còn lỗi.
Redo gốc vẫn thực thi16/16. **RCORE-01, Task2/3 vàmọi yêu cầu gốc còn mở.**
[Bằng chứng và lỗi tiếp theo](../../docs/validation/2026-10-12-rhino-core-circle-history-modes.md).
<!-- rhino-core-circle-history-modes-20261012:end -->

<!-- rhino-core-circle-history-result-20261012:start -->
Circle History Runtime59: **430/430 native**,14 actual host exits0;
full Rust916/0/0,plugin build0. Dùng lại solver;result/current input
snapshots được ghép với Shape. Callback33,Box Vertex/Mesh/Update=No/units8,
Undo/Redo24 vàcold FCStd12 đạt. Giữ RED58 vàexcluded missing-helper launch.
**RCORE-01 đang triển khai;Task2/3,mọi C01–C12/T01–T11 vàgiới hạn gốc còn mở.**
[Bằng chứng vàgiới hạn](../../docs/validation/2026-10-12-rhino-core-circle-history-result.md).
<!-- rhino-core-circle-history-result-20261012:end -->

<!-- rhino-core-circle-history-result-red-20261012:start -->
Circle History Runtime57: **RED12/36**,actual host exit0,không fixture exception.
Center/AroundCurve/FitPoints dùng lại solver và cập nhật Shape đúng,nhưng result
và input snapshots còn cũ;invalid source chưa xóa result. Freeze/detach/FCStd
chưa có cặp Shape/result hiện hành. Giữ riêng checkpoint focus1544/1544.
**RCORE-01 đang triển khai;Task2/3 và mọi yêu cầu gốc giữ nguyên.**
[Bằng chứng và bước sửa](../../docs/validation/2026-10-12-rhino-core-circle-history-result-red.md).
<!-- rhino-core-circle-history-result-red-20261012:end -->

<!-- rhino-core-curve-focus-result-20261012:start -->
Ellipse FromFoci/MarkFoci dùng lại solver, có common GeometryResult cho hai focus.
Runtime56 RED6/30 → Runtime57 focus30/30,cold12/12,focus callbacks130/130.
**1544/1544 native**,27 actual exits0;6 cold hosts/29 FCStd;build0;Rust885 và lượt sau900.
Curve hashes khớp;full current-source matching false do Gumball đổi song song;
inventory được chụp sau khi build bắt đầu,không phải compiler-input certificate.
**RCORE-01 đang triển khai;Task2/3 và mọi yêu cầu gốc giữ nguyên.**
Auxiliary History/Circle Rebuild,full lifetime/sharing/budgets/conversion/maps và chuột/render còn mở.
[Bằng chứng và phần còn mở](../../docs/validation/2026-10-12-rhino-core-curve-focus-result.md).
<!-- rhino-core-curve-focus-result-20261012:end -->

*Checkpoint Runtime56 bên dưới là lịch sử; xem Runtime57 ở trên.*
<!-- rhino-core-curve-booking-20261012:start -->
Curve native booking Runtime56: **1372/1372 native**,24 actual exits0;Rust882/0/0;build0.
Commit/abort/reopen callback120/120 giữ đúng journal, rollback/document và prior native Redo;
selected1416 inputs match,SDK unchanged;5 cold hosts/27 FCStd.
**RCORE-01 đang triển khai;Task2/3 và mọi yêu cầu gốc giữ nguyên.**
Foci common result/History/Rebuild, full lifetime/sharing/budgets/conversion/maps và chuột/render còn mở.
[Bằng chứng và phần còn mở](../../docs/validation/2026-10-12-rhino-core-curve-booking.md).
<!-- rhino-core-curve-booking-20261012:end -->

*Checkpoint Runtime55 bên dưới là lịch sử; native booking đã có Runtime56 ở trên.*
<!-- rhino-core-curve-publication-20261011:start -->
Checked Curve publication/cancellation: Runtime55 **1252/1252 native checks**,23 actual exits0;
Rust882/0/0; standalone build0;1415 selected inputs match;SDK unchanged.
Output210/source60/layer210/replacement60/hidden-layer50 plus662 regressions pass.
Uses existing solvers and host Redo-preserving transaction; prior native Redo and replacement Box retained.
**RCORE-01 đang triển khai;Task2/3 giữ nguyên.** Transaction booking/premature close,
foci common result/History/Rebuild, sharing/budgets/conversion/maps và chuột/render còn mở.
[Bằng chứng và phần còn mở](../../docs/validation/2026-10-11-rhino-core-curve-publication.md).
<!-- rhino-core-curve-publication-20261011:end -->

*Checkpoint RED52 dưới đây là lịch sử; xem Runtime55 ở trên.*
<!-- rhino-core-curve-callback-red-20261011:start -->
Checked Curve publication:Runtime52 real callback witness **RED42/210**,
actual host exit0,không fixture exception. Sáu lệnh vẫn commit output bị đổi
Shape/frame/construction/result/units/parent;42 callbacks đều chạy,mỗi ca mất prior Redo.
Main result662/662 và Rust880 giữ riêng theo phạm vi;dùng lại host transaction giữ Redo.
**RCORE-01 đang triển khai;Task2/3 và mọi yêu cầu gốc giữ nguyên.**
[Bằng chứng và bước sửa](../../docs/validation/2026-10-11-rhino-core-curve-callback-red.md).
<!-- rhino-core-curve-callback-red-20261011:end -->

<!-- rhino-core-curve-result-20261011:start -->
Sáu solver Curve hiện hữu có common GeometryResult cho output chính:
Rust replay toàn construction; original numeric input0/native sources1..N;
full output CAD master/topology/units, paired Shape/metadata và Undo/Redo/FCStd.
Runtime50 RED12/54→Runtime52 GREEN54/54; native sources33/cold58.
Matching662/662 checks/18 actual exits0;5 cold hosts/27 FCStd;Rust880/0/0;build0.
1407 selected inputs khớp sau build/native;SDK không đổi;offscreen không chứng nhận chuột/render.
Checked callback publication,foci/History/Rebuild,input sharing/budgets và conversion/maps còn mở.
**RCORE-01 đang triển khai;Task2/3 và mọi yêu cầu gốc giữ nguyên.**
[Bằng chứng và phần còn mở](../../docs/validation/2026-10-11-rhino-core-curve-result.md).
<!-- rhino-core-curve-result-20261011:end -->

<!-- rhino-core-curve-result-red-20261011:start -->
*Checkpoint RED lịch sử trước common result Runtime52; xem checkpoint mới phía trên.*
Task3 common Curve GeometryResult: native witness Runtime50 **RED12/54**,
actual host exit0,không có fixture exception. Sáu lệnh có geometry/one Undo đúng,
nhưng thiếu OM9GeometryResult/input snapshots/output CAD master và paired persistence.
Cohort bridge517/517 giữ riêng;dùng lại solver để bổ sung result và checked publication.
**RCORE-01 đang triển khai;Task2/3 và mọi yêu cầu gốc giữ nguyên.**
[Bằng chứng và bước tiếp](../../docs/validation/2026-10-11-rhino-core-curve-result-red.md).
<!-- rhino-core-curve-result-red-20261011:end -->

<!-- rhino-core-curve-native-bridge-20261011:start -->
Cầu nối native Circle/Ellipse giữ original text và full parent CAD/cloud,
attributes/units/frame/revision/ranges; Circle Undo loại cả source branch.
Dùng lại solver. Hai lỗi review: Rust RED0/1→GREEN18/18;
native capture/indexing RED8/12→GREEN12/12. Runtime50:517/517 checks,
15 actual exits0;4 cold hosts/18 FCStd;full Rust874/0/0;build0.
Curve inputs khớp;Gumball C++/Rust/tests đổi song song nên full current-source matching false;
Full Rust inputs khớp tại lúc ghi;SDK không đổi;chuột/render chưa nghiệm thu.
**RCORE-01 đang triển khai;Task2/3 và mọi yêu cầu gốc giữ nguyên.**
[Bằng chứng và phần còn mở](../../docs/validation/2026-10-11-rhino-core-curve-native-bridge.md).
<!-- rhino-core-curve-native-bridge-20261011:end -->

<!-- rhino-core-curve-native-admission-20261011:start -->
*Checkpoint Rust-only trước khi nối native; xem checkpoint mới phía trên.*

Rust đã có API nhận nguồn native Curve sở hữu dữ liệu, giữ original text/source
và branch Undo; solver hiện hữu được dùng lại. Năm lỗi review: RED8/13→GREEN13/13;
units/context thêm:16/16 và FFI1/1. Full Rust871/0/0, actual exit0;238 inputs khớp.
**C++ chưa nối API mới; chưa build/chạy native. Runtime45 vẫn RED9/18 về nguồn.**
**RCORE-01 đang triển khai; Task2/3 và mọi yêu cầu gốc giữ nguyên.**
[Bằng chứng và bước tiếp](../../docs/validation/2026-10-11-rhino-core-curve-native-admission.md).
<!-- rhino-core-curve-native-admission-20261011:end -->

<!-- rhino-core-curve-native-source-20261011:start -->
Native witness mới trên Runtime45: **RED9/18**, actual host exit0.
Circle/Ellipse AroundCurve và FitPoints sau Undo đạt hình học/Undo/FCStd,
nhưng thiếu original native text, identity/revision/frame/ranges và full source snapshots.
Giữ riêng cohort accepted-input466/466; đây là phần Task2 cần làm tiếp.
Gumball/lib.rs sửa song song; full current-source matching vẫn chưa xác nhận.
**RCORE-01 đang triển khai; toàn bộ yêu cầu gốc vẫn cần hoàn tất.**
[Bằng chứng và bước tiếp](../../docs/validation/2026-10-11-rhino-core-curve-native-source.md).
<!-- rhino-core-curve-native-source-20261011:end -->

<!-- rhino-core-curve-accepted-input-20261011:start -->
Curve dùng lại solver, giữ đầu vào gốc tại Rust entrypoint trước chuẩn hóa:
text/units/frame, điểm trước Ortho và batch FitPoints; Circle Undo phục hồi branch.
Chặn NaN/Infinity trước đổi state/log: review Rust RED5/6→GREEN6/6.
Full Rust844/0/0; standalone build0; Runtime45 offscreen466/466 checks,
12 actual exits0;3 cold hosts/15 FCStd;SDK không đổi. Nguồn Curve không đổi;
Gumball/lib.rs sửa song song, nên chưa xác nhận toàn bộ source hiện tại khớp binary.
Full native sources/input, Curve GeometryResult/checked publication, History/foci/
Rebuild, conversion/maps và chuột/render vẫn còn mở.
**RCORE-01 đang triển khai; toàn bộ yêu cầu gốc vẫn cần hoàn tất.**
[Bằng chứng và phần còn mở](../../docs/validation/2026-10-11-rhino-core-curve-accepted-input.md).
<!-- rhino-core-curve-accepted-input-20261011:end -->

<!-- rhino-core-curve-construction-20261011:start -->
*Checkpoint Runtime44 trước accepted-input log; giữ bằng chứng lịch sử theo phạm vi.*
Sáu lệnh Curve dùng lại solver, lưu construction độc lập cùng Shape.
Giữ options/plan/basis; phân biệt normalized input và display outline.
Sửa Close một lần và giới hạn điểm đã nhận: Rust RED3/5→GREEN5/5;
native review RED15/26→GREEN26/26. Full Rust838/0/0; standalone build0.
Runtime44 offscreen413/413 checks/9 actual exits0;2 cold hosts/11 FCStd;
442 producer inputs khớp; shared SDK không đổi.
Đầu vào gốc/native sources, Curve GeometryResult, checked publication và
History/foci/Rebuild cùng conversion/maps và chuột/render vẫn còn mở.
**RCORE-01 đang triển khai; toàn bộ yêu cầu gốc vẫn cần hoàn tất.**
[Bằng chứng và phần còn mở](../../docs/validation/2026-10-11-rhino-core-curve-construction.md).
<!-- rhino-core-curve-construction-20261011:end -->

<!-- rhino-core-result-output-budget-20261011:start -->
*Checkpoint Runtime42 trước Curve construction; giữ bằng chứng lịch sử theo phạm vi.*
GeometryResult chung kiểm node/depth trước serialize; giữ đầy đủ geometry.
Focused Rust RED1/4 → GREEN4/4; full Rust833/0/0, standalone build0.
Runtime42 offscreen:892/892 checks,18 actual exits0; Solid553 và5 consumer fixtures;
2 cold hosts/6 FCStd,437 producer inputs không đổi. Shared SDK giữ nguyên.
Aggregate allocation, Curve result, conversion/maps và chuột/render còn mở.
**RCORE-01 đang triển khai; toàn bộ yêu cầu gốc vẫn cần hoàn tất.**
[Bằng chứng và phần còn mở](../../docs/validation/2026-10-11-rhino-core-result-output-budget.md).
<!-- rhino-core-result-output-budget-20261011:end -->

<!-- rhino-core-solid-source-budget-20261011:start -->
*Checkpoint Runtime41 trước output node/depth gate; giữ bằng chứng lịch sử theo phạm vi.*
Solid chặn giới hạn source JSON trước nhận FitPoints; dùng lại solver FreeCAD.
Ca nguồn khác nhau: Runtime40 RED3/5 → Runtime41 GREEN5/5, giữ đủ nguồn/range.
Matching Runtime41:553/553 checks,13 actual exits0,2 cold hosts/6 FCStd;
Rust829/0/0, standalone retry build0,437 producer inputs không đổi.
Gate source cần thiết đã kiểm; full request/output budget và chuột/render còn mở.
**RCORE-01 đang triển khai; toàn bộ yêu cầu gốc vẫn cần hoàn tất.**
[Bằng chứng và phần còn mở](../../docs/validation/2026-10-11-rhino-core-solid-source-budget.md).
<!-- rhino-core-solid-source-budget-20261011:end -->

<!-- rhino-core-solid-shared-snapshot-20261011:start -->
*Checkpoint Runtime39 trước source-envelope gate; trạng thái RED và source drift dưới đây là lịch sử.*
Solid dùng lại solver FreeCAD, có GeometryResult và snapshot nguồn chung với
chỉ số/range đầy đủ. Matching Runtime39:548/548 checks,12 actual exits0, gồm2
cold hosts mở5 recipe FCStd và1 file54 refs/2 snapshot nguồn; Rust826/0/0, standalone build0. Budget native RED3/5 → GREEN5/5.
Các ca hiện tại chạy offscreen; chuột/viewport/render chưa nghiệm thu trên Runtime39.
Ca nguồn khác nhau còn RED3/5 trên Runtime39; giới hạn chung chưa hoàn tất.
Bốn file Gumball đổi sau build; bằng chứng trên thuộc binary Runtime39.
**RCORE-01 đang triển khai; mọi yêu cầu gốc/conversion/maps vẫn còn mở.**
[Bằng chứng và phần còn mở](../../docs/validation/2026-10-11-rhino-core-solid-shared-snapshot.md).
<!-- rhino-core-solid-shared-snapshot-20261011:end -->

<!-- rhino-core-solid-result-20261011:start -->
*Checkpoint Runtime38 trước native offscreen và sửa snapshot sharing; trạng thái chưa kiểm chứng bên dưới là lịch sử.*
Solid đã nối vào GeometryResult chung; FitPoints giữ nguồn/range và có giới hạn
tổng dữ liệu, kiểm nội dung native cùng root frame. Rust825/0/0 gồm9 kiểm tra mới;
standalone Runtime38 build exit0. **Native mới chưa kiểm chứng**: năm lượt startup
timeout bị loại, kể cả profile đồ họa riêng; các host thử nghiệm đã kết thúc. Runtime35/32 giữ riêng.
**RCORE-01 đang triển khai; mọi yêu cầu gốc/conversion/maps vẫn còn mở.**
[Bằng chứng và phần còn mở](../../docs/validation/2026-10-11-rhino-core-solid-result.md).
<!-- rhino-core-solid-result-20261011:end -->

<!-- rhino-core-solid-construction-20261011:start -->
*Checkpoint Runtime35 trước Solid GeometryResult; bằng chứng lịch sử có phạm vi.*
Rust giữ bản chụp đầu vào Solid độc lập, gồm các điểm 4Point/AroundCurve bị
solver rút gọn; Shape và metadata/units được lưu cùng transaction, chặn context stale.
Matching Runtime35:757 checks/8 actual exits0, gồm1 cold host mở5 FCStd.
Native67: RED12/67 →66/67 →GREEN67/67; Rust816 với221 source-identical files.
Caller703/8 Runtime32 là bằng chứng lịch sử riêng. Đây là input evidence;
full Solid GeometryResult/conversion/maps và yêu cầu gốc vẫn còn mở.
**RCORE-01 đang triển khai; chưa nghiệm thu toàn chương.**
[Bằng chứng và phần còn mở](../../docs/validation/2026-10-11-rhino-core-solid-construction.md).
<!-- rhino-core-solid-construction-20261011:end -->

<!-- rhino-core-solid-publication-20261011:start -->
*Checkpoint Runtime33 trước construction capture; bằng chứng lịch sử có phạm vi.*
Box/Sphere dùng lại solver FreeCAD; bổ sung rollback, generation, kiểm frame/
shape/metadata trước commit và giữ phiên lệnh mới khi callback thay phiên cũ.
Callback metadata RED144/150 → GREEN150/150; Solid hiện tại669 checks/6 exits0.
Hồi quy caller703/8 trên Runtime32 được ghi riêng, không phải rerun Runtime33.
Full Rust811 đạt ở checkpoint source-identical;220 source files không đổi.
Full Solid GeometryResult/conversion/maps, preview lease và yêu cầu gốc còn mở.
**RCORE-01 đang triển khai; chưa nghiệm thu toàn chương.**
[Bằng chứng và phần còn mở](../../docs/validation/2026-10-11-rhino-core-solid-publication.md).
<!-- rhino-core-solid-publication-20261011:end -->

<!-- rhino-core-surface-preview-20261011:start -->
*Checkpoint Surface/camera trước đợt Solid; giữ bằng chứng lịch sử theo phạm vi.*
Surface báo lỗi dọn preview/root lease ở4 viewport, giữ nguồn/Undo và cho tạo
lại Preview.48 ca đạt468/468; camera khi đóng document RED15/18 → GREEN18/18.
GUI mới:486 checks/2 actual exits0, không còn escaped camera exception.
Hồi quy plugin trước đó:649 checks/23 exits0, gồm5 cold hosts; chưa chạy lại
trên DLL GUI mới. Rust808 dùng lại với218 source hashes không đổi.
**RCORE-01 đang triển khai; toàn bộ yêu cầu gốc giữ nguyên.**
[Bằng chứng và phần còn mở](../../docs/validation/2026-10-11-rhino-core-surface-preview.md).
<!-- rhino-core-surface-preview-20261011:end -->

<!-- rhino-core-commit-publication-20261011:start -->
*Checkpoint Runtime24 trước sửa cleanup; xem checkpoint mới phía trên.*
Builder Create/Restore/Match giải phóng Python result khi commit lỗi, báo
KernelFailure kèm thao tác và nguồn; Sweep1/Sweep2/Loft plain/History giữ đủ tên
nguồn và chặn native exception thoát qua Qt. RED57/69 và60/72 → GREEN69/69 và72/72.
Matching Runtime24/API19: 649 checks/23 actual host exits0, gồm5 cold hosts;
full Rust808 đạt. Launcher GUI riêng dùng lại SDK; shared App vẫn API12.
**RCORE-01 đang triển khai; toàn bộ yêu cầu gốc giữ nguyên.**
[Bằng chứng và phần còn mở](../../docs/validation/2026-10-11-rhino-core-commit-publication.md).
<!-- rhino-core-commit-publication-20261011:end -->

<!-- rhino-core-isolated-publication-20261011:start -->
*Checkpoint native trước khi có matching GUI riêng; xem kiểm chứng caller mới phía trên.*
DLL native thử nghiệm riêng API19: global28/28, root46/46;
App602 đạt/2 bỏ qua/0 lỗi trong604 ca,7 disabled. Đã kiểm lỗi lifecycle,
activation, chuyển thuộc tính lồng và verdict cùng/khác ID bằng RED → GREEN.
SDK dùng chung vẫn API12; matching OM9 GUI/plugin chưa kiểm chứng.
**RCORE-01 đang triển khai; toàn bộ yêu cầu gốc giữ nguyên.**
[Bằng chứng và phần còn mở](../../docs/validation/2026-10-11-rhino-core-isolated-publication.md).
<!-- rhino-core-isolated-publication-20261011:end -->

<!-- rhino-core-global-close-20261011:start -->
*Checkpoint lịch sử API12 và candidate API13; xem DLL thử nghiệm đã kiểm phía trên.*
API12: global native boundary8/8, root46/46; App602 đạt/2 bỏ qua/0 lỗi trong604 ca.
Hai cohort RED mới đều0/4: tham gia muộn, commit rỗng mất Undo và đóng nhóm lồng.
Candidate API13 đã biên dịch object; DLL/runtime chưa kiểm chứng vì shared host
demo Gumball còn mở. Không nâng checkpoint Runtime21/API11 thành bằng chứng mới.
**RCORE-01 đang triển khai; toàn bộ yêu cầu gốc giữ nguyên.**
[Bằng chứng và phần còn mở](../../docs/validation/2026-10-11-rhino-core-global-close.md).
<!-- rhino-core-global-close-20261011:end -->

<!-- rhino-core-publication-booking-20261011:start -->
*Checkpoint lịch sử API11/Runtime21; xem global boundary và candidate mới phía trên.*
Builder/Surface dùng lại solver, giữ native Redo khi output-created callback
làm source stale; Builder18/18 và Surface30/30 đạt trên Runtime21/API11.
Sửa native booking tách transaction xóa cũ khỏi edit/group mới và từ chối ID
nhóm bị callback đóng. Builder History gốc43/43; tổng462 checks/16 host exits0,
native46/46; App602 đạt/2 bỏ qua/0 lỗi trong604 ca; Rust808 đạt.
Fixture global finalization riêng còn2 lỗi/3 ca: thiếu global abort pair và
before-commit callback vẫn sửa source/quan sát booking đã đóng.
**RCORE-01 đang triển khai; chưa nghiệm thu toàn chương.**
[Bằng chứng và lỗi còn mở](../../docs/validation/2026-10-11-rhino-core-publication-booking.md).
<!-- rhino-core-publication-booking-20261011:end -->

<!-- rhino-core-atomic-redo-20261011:start -->
*Checkpoint lịch sử API9/Runtime18. Phạm vi đã kiểm giữ nguyên; xem hồi quy booking và global finalization mới phía trên.*
Native giữ Redo thật qua API opt-in và checked rollback nhiều document;
OM9 Edit dùng lại solver, chuyển sang API này. Runtime18/API9: callback5/5,
Python23/23; tổng401 checks/13 actual host exits0, full Rust808 đạt.
Native43/43; App599 đạt/2 bỏ qua/0 lỗi trong601 ca thực thi.
Bằng chứng này thay trạng thái API/callback còn mở ở checkpoint cũ trong phạm vi
cùng luồng đã kiểm; worker/custom callbacks và toàn bộ hợp đồng gốc còn mở.
**RCORE-01 đang triển khai; chưa nghiệm thu toàn chương.**
[Bằng chứng và giới hạn](../../docs/validation/2026-10-11-rhino-core-atomic-redo.md).
<!-- rhino-core-atomic-redo-20261011:end -->

<!-- rhino-core-checked-rollback-20261011:start -->
*Checkpoint trước API9/Runtime18; trạng thái API/callback bên dưới là bằng chứng lịch sử. Xem checkpoint mới phía trên.*
Native `Transaction::applyChecked` phát hiện rollback bị từ chối hoặc chưa khôi phục đủ;
giữ hành vi `apply` mặc định và ownership native. Chín ca checked rollback đạt.
App executable: 569 đạt, 2 bỏ qua, 5 lỗi giữ Redo còn mở; exit1, không gọi cả suite đạt.
API giữ Redo, gate nhiều document và matching host/plugin vẫn chưa hoàn tất.
**RCORE-01 đang triển khai; toàn bộ yêu cầu gốc được giữ nguyên.**
[Bằng chứng và phần còn mở](../../docs/validation/2026-10-11-rhino-core-checked-rollback.md).
<!-- rhino-core-checked-rollback-20261011:end -->

<!-- rhino-core-edit-result-20261011:start -->
*Checkpoint trước API9/Runtime18; trạng thái API/callback bên dưới là bằng chứng lịch sử. Xem checkpoint mới phía trên.*
Rust admission và native CAD Edit GeometryResult đã triển khai trên solver sẵn có.
Runtime14/15 đã đạt các hồi quy CAD/lệnh/guard và cold reload; callback abort còn thất bại 2/5 kiểm tra Redo.
Runtime15: Dimension NaN/Infinity 10/10, callback font/đồ họa và lỗi đúng Explode/source 12/12; dùng lại kiểm finite Rust.
Full Rust 808 đạt; bằng chứng native giữ riêng theo plugin, không suy thành nghiệm thu toàn chương.
**RCORE-01 đang triển khai**, không đóng consumer/lifecycle hay yêu cầu gốc.
[Bằng chứng, lỗi đang mở và bước tiếp](../../docs/validation/2026-10-11-rhino-core-edit-result.md).
<!-- rhino-core-edit-result-20261011:end -->

<!-- rhino-core-surface-result-20261011:start -->
Loft/Sweep giữ solver hiện có, có GeometryResult EditSurface với recipe,
source snapshots/attributes/revision và CAD master khớp Shape được công bố.
History cập nhật/xóa cả Shape và result; freeze/detach, Undo/Redo và FCStd đạt.
Sửa và kiểm Placement/group cùng tên output duy nhất trước publication.
Matching848/21 actual host exits0; full Rust791 đạt.
Global conversion/maps, các consumer và lifecycle toàn family còn mở;
**RCORE-01 đang triển khai**. [Bằng chứng và giới hạn](../../docs/validation/2026-10-11-rhino-core-surface-result.md).
<!-- rhino-core-surface-result-20261011:end -->

<!-- rhino-core-surface-history-admission-20261011:start -->
Surface History dùng Rust kiểm JSON/options/group metadata trước native capture;
giữ solver Loft/Sweep và kiểm rail closure từ nguồn hiện tại. Dữ liệu hỏng giữ
raw/source/links/Redo, xóa output cũ và báo InvalidBasis kèm tên child.
GREEN55; matching698/13 actual host exits0; full Rust767 đạt.
Full results/conversion/maps/ownership mọi family còn mở; **RCORE-01 đang triển khai**.
[Bằng chứng và giới hạn](../../docs/validation/2026-10-11-rhino-core-surface-history-admission.md).
<!-- rhino-core-surface-history-admission-20261011:end -->

<!-- rhino-core-builder-conversion-20261011:start -->
Builder giữ native ModifiedShape topology map, template snapshot độc lập và
điểm đo/parameter queries cho edge và face có trim; Rust kiểm frame/domain/ID
và sai lệch trước publish. Query native không thực hiện được ghi unavailable
và diagnostic; không nâng tolerance hoặc xác nhận global parameter map.
315 checks/10 actual host exits0 trên cùng plugin, full Rust758 đạt.
Global conversion/maps và hợp đồng mọi family còn mở; **RCORE-01 đang triển khai**.
[Bằng chứng và giới hạn](../../docs/validation/2026-10-11-rhino-core-builder-conversion.md).
<!-- rhino-core-builder-conversion-20261011:end -->

<!-- layer-management-audit-20261011:start -->
LAYERS / **OM9-LAYER-001** (2026-10-11): `partially_implemented` /
`validated_supported_slice`. Panel current/Assign, lock/Region/edit, Hide/Show,
group visibility, Advanced và Lights đã nghiệm thu. Footer bỏ theo người dùng.
Rust **11/11**; native **panel 71/71, outputs 21/21, permission 5/5, modal 5/5, circle 51/51, mouse 51/51**. Plugin-only build/shared SDK;
Art/material và general hierarchy còn ngoài slice. [Report](../../docs/validation/2026-10-11-layer-panel-implementation.md).
<!-- layer-management-audit-20261011:end -->

<!-- rhino-core-builder-result-20261011:start -->
Builder có GeometryResult native đúng operation, recipe/slot/source revision,
CAD master/topology/validation và thuộc tính seed độc lập, giữ qua History/FCStd.
Create/restore/Match chuẩn bị cả Shape/result trước transaction; metadata hỏng
bị từ chối, recipe legacy ghi unavailable. 297 checks/9 actual host exits0
trên cùng plugin; full Rust747 đạt. Conversion/maps, thuộc tính còn lại và
hợp đồng mọi family còn mở; **RCORE-01 vẫn đang triển khai**.
[Bằng chứng và giới hạn](../../docs/validation/2026-10-11-rhino-core-builder-result.md).
<!-- rhino-core-builder-result-20261011:end -->

<!-- rhino-core-builder-output-publication-20261011:start -->
Builder Create/restore/Match dựng và kiểm toàn bộ output/slot trước transaction;
lỗi output cuối giữ source, links và executable Redo. Dùng lại solver/native
frame và policy Rust; dữ liệu dựng sẵn chỉ sống trong lần publish này.
28 output checks; matching196 checks/7 actual host exits0, gồm Builder cold
FCStd và prepared CAD; full Rust744 đạt. Results/conversion/maps/attributes,
aggregate budgets và hợp đồng toàn family còn mở; **RCORE-01 đang triển khai**.
[Bằng chứng và giới hạn](../../docs/validation/2026-10-11-rhino-core-builder-output.md).
<!-- rhino-core-builder-output-publication-20261011:end -->

<!-- startup-ui-cleanup-20261011:start -->
Startup UI (2026-10-11): bỏ hàng toolbar Import/Export 3DM thừa; Task đóng khi
khởi động, menu chữ và các lệnh Import/Export vẫn truy cập được. Launcher FreeCAD
1.1.4 đã cập nhật; sáu kiểm tra trên profile mới đạt, host thoát 0.
[Bằng chứng](../../docs/validation/2026-10-11-startup-ui-cleanup.md).
<!-- startup-ui-cleanup-20261011:end -->

<!-- rhino-core-builder-policy-20261011:start -->
Builder affine-template hiện có dùng request Rust độc lập, kiểm source/template,
recipe và kích thước trước copy/transaction; lỗi đầu vào giữ Redo.
61 policy checks; matching1484 checks/32 actual host exits0, gồm Builder và
Circle cold FCStd; full Rust744 đạt. Giữ solver/frame/History hiện hữu.
Results/maps, source-lock-cache và yêu cầu gốc còn mở; **RCORE-01 đang triển khai**.
[Bằng chứng và giới hạn](../../docs/validation/2026-10-11-rhino-core-builder-policy.md).
<!-- rhino-core-builder-policy-20261011:end -->

<!-- rhino-core-circle-history-policy-20261011:start -->
Circle History dùng request Rust từ tùy chọn đã lưu, kiểm trước copy/reader;
giữ độc lập với lệnh đang nhập, legacy AroundCurve và Mesh/cloud FitPoints.
1363 checks/27 actual host exits0 trên cùng plugin, gồm cold FCStd; full
Rust739 đạt. Output results/maps, consumer khác và yêu cầu gốc còn mở;
**RCORE-01 vẫn đang triển khai**. [Bằng chứng và giới hạn](../../docs/validation/2026-10-11-rhino-core-circle-history-policy.md).
<!-- rhino-core-circle-history-policy-20261011:end -->

<!-- rhino-core-cage-borrowed-admission-20261011:start -->
Basis Cage và saved binding được Rust kiểm trên dữ liệu mượn trước copy/input
buffer; native selection và recompute có lỗi typed kèm operation/object. Giữ
solver, imported basis, Undo/Redo và Release liên kết hỏng. 1322 checks/24
actual host exits0 trên cùng plugin; full Rust734 đạt. Results/maps, History,
native allocation khác và yêu cầu gốc còn mở; **RCORE-01 đang triển khai**.
[Bằng chứng và giới hạn](../../docs/validation/2026-10-11-rhino-core-cage-borrowed-admission.md).
<!-- rhino-core-cage-borrowed-admission-20261011:end -->

<!-- rhino-core-native-cage-policy:start -->
Cage hiện có dùng policy Rust chung cho declaration/capture/commit và request
độc lập cho API/recompute; Part/Mesh được kiểm trước copy/Topology extraction.
Giữ Create bằng bounds, Restore archive và Release binding hiện có. 1268 checks/
23 actual host exits0 trên cùng plugin; full Rust730 đạt. Rational-control/
binding allocation, results/maps, các consumer khác và yêu cầu gốc còn mở;
**RCORE-01 đang triển khai**. [Bằng chứng và giới hạn](../../docs/validation/2026-10-11-rhino-core-native-cage-policy.md).
<!-- rhino-core-native-cage-policy:end -->

<!-- rhino-core-native-solid-policy:start -->
Box/Sphere hiện có dùng request/options Rust chung trước declaration,
Preview, commit và capture Sphere; kiểm output scalar giữ kích thước có dấu,
CPlane và solver hiện hữu. 1071 checks/18 actual host exits0 trên cùng plugin;
full Rust721 đạt. Các hợp đồng kết quả/ownership gốc và cage/replay còn mở;
**RCORE-01 đang triển khai**. [Bằng chứng và giới hạn](../../docs/validation/2026-10-11-rhino-core-native-solid-policy.md).
<!-- rhino-core-native-solid-policy:end -->

<!-- rhino-core-native-curve-policy:start -->
Sáu Curve family hiện có dùng request/options Rust chung từ session ở
declaration/preview/commit và các capture Circle/Ellipse/FitPoints được nêu trong báo cáo.
680 checks/15 actual host exits0 trên cùng plugin; full Rust712 đạt.
Box/Sphere/cage, replay request và các hợp đồng output/ownership gốc còn mở;
**RCORE-01 đang triển khai**. [Bằng chứng và giới hạn](../../docs/validation/2026-10-11-rhino-core-native-curve-policy.md).
<!-- rhino-core-native-curve-policy:end -->

<!-- rhino-core-native-rebuild-policy:start -->
Rebuild hiện có dùng policy operation/type/options Rust chung trước enable,
copy và solve; kiểm nội dung Compound và giữ solver/source/placement/Undo/Redo.
383 checks/nine actual host exits0 trên cùng plugin; full Rust702 đạt.
Các family construction và hợp đồng output/ownership gốc còn mở;
**RCORE-01 đang triển khai**. [Bằng chứng và giới hạn](../../docs/validation/2026-10-11-rhino-core-native-rebuild-policy.md).
<!-- rhino-core-native-rebuild-policy:end -->

<!-- rhino-core-fit-point-input:start -->
Circle/Sphere và Circle History dùng reader điểm native có giới hạn Rust trước
copy, giữ placement/source/Undo/Redo và ngân sách typed+selected. GREEN45;
589 checks trên cùng plugin,7 actual host exits0;full Rust695 đạt.
Các policy construction/results và yêu cầu gốc còn mở; **RCORE-01 đang triển khai**.
[Bằng chứng và giới hạn](../../docs/validation/2026-10-11-rhino-core-fit-point-input.md).
<!-- rhino-core-fit-point-input:end -->

<!-- rhino-core-native-surface-diagnostics:start -->
Lỗi policy Surface đã ghi đúng operation và object/subelement nguồn;
RED5/6 → GREEN6/6. Bản plugin sau sửa đạt112 checks,4 actual host exits0;
637 checks ở checkpoint tích hợp trước dùng plugin cũ. **RCORE-01 còn đang triển khai.**
[Bằng chứng và giới hạn](../../docs/validation/2026-10-11-rhino-core-native-surface-diagnostics.md).
<!-- rhino-core-native-surface-diagnostics:end -->

<!-- rhino-core-native-surface-policy:start -->
RCORE-01 đã nối policy Surface Rust chung vào declaration/capture/construction
của Sweep1/Sweep2/Loft hiện có; 637 checks trên cùng plugin, 9 actual host exits0.
Full Rust 691 đạt với source fingerprint không đổi. Các family khác, kết quả/
conversion và yêu cầu gốc còn mở; **toàn chương vẫn đang triển khai**.
[Bằng chứng và giới hạn](../../docs/validation/2026-10-11-rhino-core-native-surface-policy.md).
<!-- rhino-core-native-surface-policy:end -->

<!-- rhino-core-native-construction-input:start -->
RCORE-01 bổ sung gate Rust hiện có trước copy input Surface/Rebuild;
12/12 kiểm tra đầu vào đạt, giữ source/Undo/Redo và selected Edge2 hợp lệ.
608 checks, 8 matching actual exits0 đã được ghi; các hồi quy còn lại
và shared construction policy/results/conversion vẫn đang triển khai.
**Toàn chương chưa hoàn tất.** [Bằng chứng và giới hạn](../../docs/validation/2026-10-10-rhino-core-native-construction-input.md).
<!-- rhino-core-native-construction-input:end -->

<!-- rhino-core-native-edit-capabilities:start -->
RCORE-01 đã nối registry Rust chung vào Join/Explode/Trim/Boolean hiện có,
kiểm input native và block/cage leaves trước copy, kiểm lại options trước solve.
679 Rust;4456 host checks;36 matching actual exits0, dùng lại solver FreeCAD.
Full native Edit results/conversion/maps, các consumer khác và yêu cầu gốc còn
mở. **Chương vẫn đang triển khai.** [Bằng chứng và giới hạn](../../docs/validation/2026-10-10-rhino-core-native-edit-capabilities.md).
<!-- rhino-core-native-edit-capabilities:end -->

<!-- rhino-core-prepared-discrete-results:start -->
RCORE-01 có GeometrySnapshot/result cho prepared mesh và PointCloud float64;
native/plain geometry sở hữu riêng, Rust kiểm schema/channel và đối chiếu mesh
archive theo triangle có hướng ở độ chính xác float32 trước transaction.
81 warm+8 cold+3701 hồi quy=3790 host checks;675 Rust;26 actual exits0.
Upstream conversion/maps/attributes, các consumer khác và yêu cầu gốc còn mở.
**Chương vẫn đang triển khai.** [Bằng chứng và giới hạn](../../docs/validation/2026-10-10-rhino-core-prepared-discrete-results.md).
<!-- rhino-core-prepared-discrete-results:end -->

<!-- rhino-core-prepared-result-snapshots:start -->
RCORE-01 có đủ trường GeometrySnapshot cho prepared CAD, native master OCCT V3
khôi phục độc lập, typed topology/support và frozen binding attributes do Rust
đóng gói.107 snapshot/warm-cold+95 metadata+3499 hồi quy=3701 host checks;
665 full Rust tests;24 actual host exits0. Upstream conversion/parameter maps,
mọi family/consumer và yêu cầu gốc còn mở. **Chương vẫn đang triển khai.**
[Bằng chứng và giới hạn](../../docs/validation/2026-10-10-rhino-core-prepared-result-snapshots.md).
<!-- rhino-core-prepared-result-snapshots:end -->

<!-- rhino-core-prepared-cad-results:start -->
RCORE-01 đưa validation/admission Rust vào commit CAD/previews; geometry và plain
metadata sở hữu riêng trước transaction. Metadata lỗi giữ Redo, diagnostic đúng
identity; reports native lưu cùng object/archive và giữ bytes qua cold FCStd.
154 mới +3434 hồi quy =3588 host checks;660 full Rust tests;22 host exits0.
Full output snapshots/upstream conversion, các family khác và yêu cầu gốc còn
mở. **Chương vẫn đang triển khai.** [Bằng chứng và giới hạn](../../docs/validation/2026-10-10-rhino-core-prepared-cad-results.md).
<!-- rhino-core-prepared-cad-results:end -->

<!-- rhino-core-topology-validation:start -->
RCORE-01 bổ sung vertex-link/face-orientation do Rust sở hữu và reports kiểm
native validity/solid/cavity/hướng world bằng OCCT hiện có.220 mới +67 surface
+3096 hồi quy =3383 kiểm tra FreeCAD;655 full Rust tests,17 host exits0.
Reports inspection phân biệt provenance với validity; uniform conversion/commit
consumers và mọi yêu cầu gốc còn lại vẫn mở. **Chương vẫn đang triển khai.**
[Bằng chứng và giới hạn](../../docs/validation/2026-10-10-rhino-core-topology-validation.md).
<!-- rhino-core-topology-validation:end -->

<!-- rhino-core-reference-contexts:start -->
RCORE-01 bổ sung reference qua group/link/array, tên/nhãn definition và alias
native có giới hạn Rust; giữ đúng identity/world frame bằng readers hiện có.
540 reference +2556 hồi quy =3096 kiểm tra FreeCAD;641 full Rust tests đạt,
15 host exits0. Configured traversal/ownership/cycles, attributes, certificates,
reports và các yêu cầu gốc vẫn mở. **Chương vẫn đang triển khai.**
[Bằng chứng và giới hạn](../../docs/validation/2026-10-10-rhino-core-reference-contexts.md).
<!-- rhino-core-reference-contexts:end -->

<!-- rhino-core-instance-arrays:start -->
RCORE-01 bổ sung occurrence array do Rust sở hữu, giữ index/identity/member path
và world frame qua Part/group/mesh/cloud readers hiện có.609 array +10 boundary
+1937 hồi quy =2556 kiểm tra FreeCAD;635 full Rust tests đạt,14 host exits0.
Reference contexts/attributes, manifold/solid/reports và các yêu cầu gốc vẫn mở.
**Chương vẫn đang triển khai.** Các checkpoint dưới đây là bằng chứng lịch sử.
[Bằng chứng và giới hạn](../../docs/validation/2026-10-10-rhino-core-instance-arrays.md).
<!-- rhino-core-instance-arrays:end -->

<!-- rhino-core-edge-geometry:start -->
RCORE-01 bổ sung bảng cạnh3D/vertex/tolerance do Rust sở hữu, nối với UV topology;
giữ standalone/loose geometry và world frame direct/group/link.921 mới +1016 hồi
quy =1937 kiểm tra FreeCAD;632 full Rust tests đạt,12 host exits0.
Flags/thử mẫu chưa chứng nhận correspondence toàn miền; manifold/solid/reports,
arrays/attributes và yêu cầu gốc vẫn mở. **Chương vẫn đang triển khai.**
Các checkpoint bên dưới là bằng chứng lịch sử.
[Bằng chứng và giới hạn](../../docs/validation/2026-10-10-rhino-core-edge-geometry.md).
<!-- rhino-core-edge-geometry:end -->

<!-- rhino-core-discrete-snapshots:start -->
RCORE-01 bổ sung mesh/cloud snapshot do Rust sở hữu: raw-local points, triangle
indices/bounds, native owner checks và world frame cho direct/group/link.
99 mới +6 cold loading +911 hồi quy =1016 kiểm tra FreeCAD;628 full Rust tests đạt.
Channels/archive float64/subelements, arrays/reference contexts, attributes,
manifold/reports và các yêu cầu gốc vẫn mở. **Chương vẫn đang triển khai.**
Các checkpoint bên dưới là bằng chứng lịch sử.
[Bằng chứng và giới hạn](../../docs/validation/2026-10-10-rhino-core-discrete-snapshots.md).
<!-- rhino-core-discrete-snapshots:end -->

<!-- rhino-core-instance-definitions:start -->
RCORE-01 có graph group/instance do Rust sở hữu, giữ ordered members, identity
của nested link và single/multiple Part subelements theo world frame native.
48 mới +863 hồi quy =911 kiểm tra FreeCAD;624 full Rust tests đạt.
Array/mesh/cloud definitions, reference contexts còn lại, attributes/userdata,
manifold/reports và các yêu cầu gốc vẫn mở. **Chương vẫn đang triển khai.**
Các checkpoint bên dưới là bằng chứng lịch sử.
[Bằng chứng và giới hạn](../../docs/validation/2026-10-10-rhino-core-instance-definitions.md).
<!-- rhino-core-instance-definitions:end -->

<!-- rhino-core-world-attributes:start -->
RCORE-01 có ma trận support-local → world đã kiểm tra, thuộc tính nguồn native
do Rust sở hữu và descriptor tách instance/definition cho nested direct App::Link.
78 mới +778 hồi quy =856 kiểm tra FreeCAD;620 Rust tests đạt. Group/member/array
instance, thuộc tính/userdata còn lại, manifold/solid và reports vẫn mở.
**Chương vẫn đang triển khai.** Các checkpoint dưới đây là bằng chứng lịch sử.
[Bằng chứng và giới hạn](../../docs/validation/2026-10-10-rhino-core-world-attributes.md).
<!-- rhino-core-world-attributes:end -->

<!-- rhino-core-derived-support:start -->
RCORE-01 có snapshot Rust-owned cho offset 3D/UV, offset surface, extrusion và
revolution; giữ chuỗi basis NURBS/Bezier/analytic và inner trim một/hai hướng.
91 derived +687 hồi quy =778 kiểm tra FreeCAD;610 Rust tests đạt. World/attributes,
manifold/solid, reports và yêu cầu còn lại vẫn mở. **Chương vẫn đang triển khai.**
Các checkpoint bên dưới là bằng chứng lịch sử.
[Bằng chứng và giới hạn](../../docs/validation/2026-10-10-rhino-core-derived-support.md).
<!-- rhino-core-derived-support:end -->

<!-- rhino-core-bezier-support:start -->
RCORE-01 có snapshot Bezier do Rust sở hữu cho curve/surface và UV pcurve;
dùng lại adapter native, giữ control net/weight, natural domain và type Bezier.
76 Bezier +611 hồi quy =687 kiểm tra FreeCAD; 603 Rust tests đạt. Offset/support
khác, world/attributes, manifold/solid và yêu cầu còn lại vẫn mở.
**Chương vẫn đang triển khai.** Các checkpoint bên dưới là bằng chứng lịch sử.
[Bằng chứng và giới hạn](../../docs/validation/2026-10-10-rhino-core-bezier-support.md).
<!-- rhino-core-bezier-support:end -->

<!-- rhino-core-analytic-support:start -->
RCORE-01 có snapshot analytic do Rust sở hữu cho 10 loại curve/surface và UV
pcurve; giữ tham số native, domain, location/orientation và phân biệt chiều UV
với frame XYZ. 174 analytic +437 hồi quy =611 kiểm tra FreeCAD; 598 Rust tests đạt.
Bezier/support khác, attributes/world composition, manifold/solid và các yêu
cầu còn lại vẫn mở. **Chương vẫn đang triển khai.** Checkpoint bên dưới là
bằng chứng lịch sử theo từng phạm vi và fingerprint, không phải trạng thái mới.
[Bằng chứng và giới hạn](../../docs/validation/2026-10-10-rhino-core-analytic-support.md).
<!-- rhino-core-analytic-support:end -->

<!-- rhino-core-owned-support:start -->
RCORE-01 có bản sao NURBS do Rust sở hữu cho mặt/pcurve và curve 3D; giữ poles,
weights, knots, periodic axes, domain đầy đủ riêng với trim, orientation và
native location. 54 kiểm tra mới +383 hồi quy FreeCAD; 594 Rust tests đạt.
Analytic/Bezier và các support khác, attributes/world composition, manifold/solid
và các yêu cầu còn lại vẫn mở. **Chương vẫn đang triển khai.**
[Bằng chứng và giới hạn](../../docs/validation/2026-10-10-rhino-core-owned-support.md).
<!-- rhino-core-owned-support:end -->

<!-- rhino-core-host-parameter-dependency:start -->
Đã nghiệm thu fixture T10 cho điểm phụ thuộc Circle/Line: dùng map openNURBS/OCCT
có sẵn, Rust kiểm điểm với source thật và chặn raw parameter chưa được chứng minh.
Query/basis/source identity được lưu FCStd và tính lại sau sửa/Undo/restore.
238 kiểm tra fixture +114 hồi quy FreeCAD;556 Rust và159 native query đạt.
Phạm vi Circle/Line và oracle lấy mẫu; **toàn bộ P0 vẫn đang triển khai.**
[Bằng chứng và giới hạn](../../docs/validation/2026-10-10-rhino-core-host-parameter-dependency.md).
<!-- rhino-core-host-parameter-dependency:end -->

<!-- rhino-core-delayed-publication:start -->
Đã nghiệm thu fixture T08 cho native Rebuild: kết quả được giữ qua lượt giao,
source/session/options hết hiệu lực bị chặn trước Coin/transaction;36 envelope
và36 tham chiếu Shape giải phóng đủ.173 kiểm tra fixture và72 hồi quy đạt.
Solver vẫn đồng bộ, delivery ở GUI queue; lifetime mọi worker vẫn còn mở.
**Toàn bộ P0 vẫn đang triển khai.** [Bằng chứng và giới hạn](../../docs/validation/2026-10-10-rhino-core-delayed-publication.md).
<!-- rhino-core-delayed-publication:end -->

<!-- rhino-core-subobject-ownership:start -->
Đã nghiệm thu fixture T11: sửa wrapper Curve/Surface sở hữu riêng, publish qua
native deep copy, giữ source/sibling, đúng Undo/Redo và phân biệt lifetime của
snapshot mạnh với request subobject.35 kiểm tra FreeCAD đạt; không viết lại editor.
**Toàn bộ P0 vẫn đang triển khai.** [Bằng chứng và giới hạn](../../docs/validation/2026-10-10-rhino-core-subobject-ownership.md).
<!-- rhino-core-subobject-ownership:end -->

<!-- rhino-core-periodic-persistence:start -->
Đã nghiệm thu T09 bằng curve periodic thật và các face có seam/lỗ; dùng lại
serializer FCStd hiện có.204 kiểm tra warm +102 kiểm tra cold đạt, cùng221 kiểm
tra đối chứng kernel. Type/topology/basis/tọa độ và inventory giữ nguyên.
**Toàn bộ P0 vẫn đang triển khai.** [Bằng chứng và giới hạn](../../docs/validation/2026-10-10-rhino-core-periodic-persistence.md).
<!-- rhino-core-periodic-persistence:end -->

<!-- rhino-core-hatch-input-gate:start -->
Đã bổ sung gate Rust cho dữ liệu NURBS của vòng Hatch trước đọc nguồn/copy/dựng
native; dùng lại editor và exporter hiện có.541 kiểm tra Rust,27 kiểm tra native,
5 bộ hồi quy native và269 kiểm tra FreeCAD đạt. Đây là một phần T07;
**toàn bộ P0 vẫn đang triển khai**. [Bằng chứng và giới hạn](../../docs/validation/2026-10-10-rhino-core-hatch-input-gate.md).
<!-- rhino-core-hatch-input-gate:end -->

<!-- rhino-core-instance-isolation:start -->
Dùng lại App::Link, mirror/scale không đều và các exporter hiện có; thêm fixture
kiểm đúng tọa độ, giữ nguyên member/definition/instance A khi chỉ sửa B.
T06 đã nghiệm thu:146 kiểm tra native,98 kiểm tra FreeCAD và4 bộ hồi quy native.
Không cần viết lại code instance. **Toàn bộ P0 vẫn đang triển khai.**
[Bằng chứng và giới hạn](../../docs/validation/2026-10-10-rhino-core-instance-isolation.md).
<!-- rhino-core-instance-isolation:end -->

<!-- rhino-core-polycurve-partition:start -->
Dùng lại wire, đảo hướng và FCStd của FreeCAD; sửa bước working 3DM để polycurve
giữ từng đoạn thay vì gộp thành một cạnh. Rust kiểm tra thứ tự, miền và điểm nối;
báo cáo có trước commit. T05 đã nghiệm thu với 403 kiểm tra native, 382 kiểm tra
FreeCAD và 523 Rust tests. **Toàn bộ P0 vẫn đang triển khai.**
[Bằng chứng và giới hạn](../../docs/validation/2026-10-10-rhino-core-polycurve-partition.md).
<!-- rhino-core-polycurve-partition:end -->

<!-- rhino-core-curve-parameters:start -->
Dùng lại openNURBS/OCCT cho báo cáo query tham số toàn miền: giữ native form1/2,
chặn output lỗi trước khi tính điểm và giữ basis/report độc lập. 502 Rust tests;
159 kiểm tra query native; 257 kiểm tra FreeCAD. Fixture RCORE-01.T01 đã nghiệm thu
với oracle lấy mẫu; T10/History vẫn thiếu tích hợp host. **Toàn bộ P0 đang triển khai.**
[Bằng chứng và giới hạn](../../docs/validation/2026-10-10-rhino-core-curve-parameters.md).
<!-- rhino-core-curve-parameters:end -->

<!-- rhino-core-kernel-contract:start -->
Dùng lại bộ chuyển đổi FreeCAD/OCCT/openNURBS; bổ sung validator basis Rust,
descriptor chỉ đọc và policy cho inspection/working 3DM. 480 Rust tests;
504 kiểm tra ứng dụng / 8 lần chạy, gồm mở FCStd trong tiến trình mới.
**P0 toàn bộ vẫn đang triển khai.** [Bằng chứng và phạm vi](../../docs/validation/2026-10-10-rhino-core-kernel-contract.md).
<!-- rhino-core-kernel-contract:end -->

<!-- rhino-core-extended-input:start -->
Bổ sung nhập polar/spherical/surveyor, fraction/feet-inch và góc DMS bằng Rust,
dùng lại bộ dựng hình FreeCAD/OM9 hiện có. Sửa mốc tương đối và Undo của Circle/Sphere.
**421 Rust tests; 1108 native checks / 21 runs; 49 Python pass,1 skip.**
P0 toàn bộ còn mở; [phạm vi và bằng chứng](../../docs/validation/2026-10-09-rhino-core-extended-input.md).

<!-- rhino-core-extended-input:end -->

<!-- rhino-core-point-consumers:start -->
Distance, Angle và PictureFrame đã dùng chung parser Rust, giữ tọa độ/đơn vị đầu vào và provenance.
CPlane sai scale/shear bị từ chối; PictureFrame kiểm giao dịch trước mutation.
**324 native checks / 9 runs; 364 Rust tests; 49 Python pass, 1 skip.**
P0 toàn bộ vẫn đang triển khai; [phạm vi và bằng chứng](../../docs/validation/2026-10-09-rhino-core-point-consumers.md).

<!-- rhino-core-point-consumers:end -->

<!-- rhino-core-tolerance-workers:start -->
Rust đã nối dung sai document/override vào Join curve và surface, lưu cutoff thực tế
và provenance cùng kết quả; backend không được tự nâng dung sai. Worker nhập 3DM
dùng 60% logical CPUs, bỏ trần cố định 4, kiểm override trước mutation.
**444 native checks / 8 runs; 347 Rust tests; 49 Python pass, 1 skip.**
P0 toàn bộ vẫn đang triển khai; [báo cáo và giới hạn](../../docs/validation/2026-10-09-rhino-core-tolerance-workers.md) ghi phạm vi chính xác.

<!-- rhino-core-tolerance-workers:end -->

<!-- rhino-core-object-foundations:start -->
Nền tảng P0 tiếp theo — **object identity và stale selection**: Rust registry
đã được nối vào Edit/Boolean, Rebuild và Surface tương tác. Lựa chọn bị thay thế
hoặc chỉnh sửa được chặn trước khi ghi kết quả; snapshot không sửa document.
[Phạm vi, kiểm thử và phần còn thiếu](../../docs/validation/2026-10-09-rhino-core-object-foundations.md). P0 vẫn đang triển khai;
các checkpoint bên dưới là bằng chứng riêng của từng phiên bản.
<!-- rhino-core-object-foundations:end -->

[Tiếng Việt — mục lục và tiến độ theo nhóm](../../README.vi.md) ·
[English — contents and group progress](../../README.en.md)

![OpenMatrix9 — Open-source Jewelry CAD Workbench](../../Resources/branding/introduction.png)

An experimental jewelry CAD workbench for FreeCAD. Rust owns command catalog,
state and behavior; C++/Qt provides native integration. Unsupported commands
remain visible and disabled.

<!-- rhino-core-p0-checkpoint:start -->
Nền tảng Rhino P0 — 2026-10-09: đã bổ sung context đơn vị/tolerance tường minh,
parser điểm dùng chung, CPlane theo viewport và menu, vòng đời snap một lần/repeat,
validation spline/3DM và tách History Record khỏi Update.
Đây là **triển khai một phần P0**; chưa hoàn tất registry/selection identity chung,
tolerance resolver cho mọi solver, History schema chung hoặc xuất 3DM giữ trạng thái đã sửa.
Phạm vi, kết quả kiểm thử và các phần còn thiếu nằm trong
[báo cáo P0](../../docs/validation/2026-10-09-rhino-core-p0.md).
Các kết quả 321 Rust, 23 Python và 657 native GUI assertions thuộc snapshot CAD/P0
trước gộp; không xác nhận phiên bản sau gộp. Bản native đó được build và thử ở
runtime riêng; ứng dụng đang mở chưa tự nhận module mới.
<!-- rhino-core-p0-checkpoint:end -->

<!-- main-merge-validation -->
Kiểm chứng sau gộp `main` — 2026-10-09: 334 kiểm thử Rust, 49 Python
(1 bỏ qua), 8 bộ native và 847 kiểm tra GUI đều qua. Xem
[báo cáo merge](../../docs/validation/2026-10-09-main-merge.md) cho phạm vi,
module đã dùng và các giới hạn còn lại. P0 vẫn chưa hoàn tất.
<!-- /main-merge-validation -->

This project was developed with Matrix9 as a functional reference. Compatibility
identifiers and familiar color roles remain. This public snapshot excludes
commercial artwork, extracted resource archives, Matrix manuals, decompiled source
and private design samples. The approved pinned Rhino 5 guide is documented in
[the reference policy](../../ref/rhino5/README.md). It does not claim clean-room development, affiliation
with the original vendors, or legal clearance of the implementation.

All **510 icon bindings** use independently authored SVGs. The exporter accepts
authored catalogs only and cannot restore legacy bitmaps.

## Ủng hộ phát triển / Donate

Nếu OpenMatrix9 hữu ích với bạn, bạn có thể donate để hỗ trợ phát triển và kiểm
thử các chức năng CAD, import/export 3DM và tương thích openNURBS.
Mọi đóng góp đều tự nguyện. Cảm ơn bạn đã hỗ trợ dự án!

- **PayPal:** gửi tới **`nguyenthaiduy277@gmail.com`** trong PayPal.
- **MoMo:** quét mã QR dưới đây. Tên người nhận: **NGUYEN THAI DUY**.

<img src="../../docs/images/donate/momo-qr.png" alt="Mã QR MoMo donate cho NGUYEN THAI DUY" width="300">

## Build và cài đặt

Xem [hướng dẫn build FreeCAD và OpenMatrix9 trên Windows](../../docs/build-windows.md):

- **Build cùng FreeCAD từ source**: clone workspace, tích hợp CMake, Pixi, build và chạy.
- **Build riêng OpenMatrix9**: dùng source/build SDK và dependencies của FreeCAD đã build.
- **Dùng với FreeCAD installer/portable**: yêu cầu ABI/SDK khớp, vị trí `.pyd` và resources, kiểm tra sau cài.

OpenMatrix9 có module native C++/Rust; chỉ clone vào `Mod` chưa đủ.
Bản cài FreeCAD thông thường không tự cung cấp SDK để build module này.

<!-- edit-options-validation -->
Edit cập nhật 2026-10-09: Trim ExtendLines/ApparentIntersections (curve,
orthographic/perspective frozen view), Join midpoint/tolerance, Explode nhóm/Mesh,
Boolean mặt/Mesh với các giới hạn rõ trong [native contract](../../docs/features/edit-native-contract.md).
Đã qua 115 core + 57 options kiểm tra native, Rust 140 và Python 23.
Lệnh vẫn `partially_implemented`; các kết quả trên chỉ xác nhận geometry slice trước nhánh mới bên dưới.
Audit lịch sử của lần kiểm chứng này báo 290 artifact tham chiếu/private có sẵn
trong workspace; không có file triển khai Edit bị gắn cờ. Báo cáo đó không xác nhận
trạng thái audit của phiên bản sau gộp.
<!-- /edit-options-validation -->

<!-- history-special-explode-validation -->
History/Explode cập nhật 2026-10-09: [Join History và policy native](../../docs/features/history-native-contract.md)
đã có Record/Update/Lock/warning/Clear; [Explode adapters](../../docs/features/edit-native-contract.md)
cho block qua CMD, text/dimension/cage native và retained 3DM đã có triển khai.
Final SDK retained3DM40/40 và native special25/25 pass.
History SDK83/83 + cold4/4 gồm grouped frames/invalid chain; core115/options57 pass.
Chi tiết trong [validation record](../../docs/validation/2026-10-09-edit-history-special-types.md).
Ordinary Edit phá links Join/Surface History liên quan; unsupported font/view-dependent
annotation/custom arrow bị từ chối atomically. Không xác nhận đầy đủ Matrix History.
<!-- /history-special-explode-validation -->


<!-- reusable-builder-cage-validation -->
### Reusable Builder API và Cage — 2026-10-09

`createBuilderRecord`, `restoreBuilderOutputs` và `matchBuilderAttributes` cung cấp
durable native record/output và versioned affine-template replay; parameters tồn
tại sau khi output bị xóa, restore là thao tác tường minh. Match API yêu cầu
same explicit OM9GemShape và áp toàn bộ supported records atomically. Đây là
nền tảng API; original Gem builder solvers, Match styles popup, sliders/handles,
Styles UI và .mss vẫn chưa có, các original command tương ứng vẫn disabled.
Rust sở hữu validation/recipe state; C++ là adapter bắt buộc cho FreeCAD document,
transactions/persistence/signals và OCCT geometry.

Cage/Create Cage/ReleaseFromCage có native menu/CMD cho World BoundingBox và
existing 3D cage, Global/Local trong giới hạn geometry đã công bố. Cage links
cập nhật khi global History Record/Update Off. 1D/2D controls, Accurate/Fast và
general BRep refit vẫn unsupported. Xem [Builder API](../../docs/features/builder-history-native-contract.md),
[Cage contract](../../docs/features/cage-command-native-contract.md) và
[bằng chứng native](../../docs/validation/2026-10-09-reusable-builder-cage.md).
<!-- /reusable-builder-cage-validation -->

## Tiến độ lệnh / Command progress

Cập nhật: **2026-10-09**. Bảng này mô tả phạm vi đang có trong source public.
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
| Explode | `TopIconExplode` | Native Explode: [phạm vi, tùy chọn và giới hạn](../../docs/features/OM9-TOP11-005.md). | 🟢 |
| Join | `TopIconJoin` | Native Join: [phạm vi, tùy chọn và giới hạn](../../docs/features/OM9-TOP11-008.md). | 🟢 |
| Split | `TopIconSplit` | Chia đối tượng bằng đối tượng cắt. | 🔴 |
| Trim | `TopIconTrim` | Native Trim: [phạm vi, tùy chọn và giới hạn](../../docs/features/OM9-TOP11-010.md). | 🟢 |
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
| Interpolate Points | `CurveFreeFormInterpolatePoints` | Vẽ B-spline qua điểm bằng chuột/CMD; Degree, Knots, Close/Sharp, Undo. Tangent chưa hỗ trợ. | 🟢 |
| Rectangle Cornerto Corner | `CurveRectangleCornertoCorner` | Hai góc, Center, 3Point, Vertical, Length/Width và Shift square; preview, Undo/Redo, FCStd. Rounded/Conic chưa hỗ trợ. Kiểm chứng 2026-10-09: [OM9-CURVE-004](../../docs/features/OM9-CURVE-004.md). | 🟢 |
| Circle Center Radius | `CurveCircleCenterRadius` | Cơ bản/nâng cao; Tangent 3D không đồng phẳng; History mọi nhánh gồm Deformable; layer hiện hành/màu/ẩn/khóa; Undo/Redo và cold FCStd. Bộ giải có giới hạn tìm nghiệm. [OM9-CURVE-005](../../docs/features/OM9-CURVE-005.md). | 🟢 |
| Ellipse From Center | `CurveEllipseFromCenter` | Center, Diameter/chuột phải, Corner, Vertical, FromFoci/MarkFoci, AroundCurve, Deformable; preview, layer, Undo/Redo và FCStd. Snapshot; History chưa hỗ trợ. [OM9-CURVE-006](../../docs/features/OM9-CURVE-006.md). | 🟢 |
| Arc Center Start Angle | `CurveArcCenterStartAngle` | Vẽ cung tròn từ tâm, điểm đầu và góc. | 🔴 |
| Arc Start End Direction | `CurveArcStartEndDirection` | Vẽ cung tròn theo điểm đầu, điểm cuối và hướng. | 🔴 |
| Curve Rebuild | `OthersCurveRebuild` | Dựng lại curve/polyline theo PointCount/Degree; preview độ lệch lấy mẫu, DeleteInput, Undo. Current layer chưa hỗ trợ. | 🟢 |
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
| Sweep1 Rail | `SurfaceSweepSweep1Rail` | Native Sweep1: Refit theo dung sai, History tùy chọn, Chain Edges, seams, Rebuild/Closed và Preview. [Phạm vi](../../docs/features/OM9-SURFACE-001.md). | 🟢 |
| Sweep2 Rails | `SurfaceSweepSweep2Rails` | Native Sweep2: G1/G2 cạnh mặt, Refit, Add Slash một profile, History và Maintain Height trong giới hạn host. [Phạm vi](../../docs/features/OM9-SURFACE-003.md). | 🟢 |
| Profile Sweep | `SurfaceSweepProfileSweep` | Quét mặt từ profile theo workflow Profile Sweep. | 🔴 |
| Planar Curves | `SurfacePlanarCurves` | Tạo mặt phẳng từ các đường biên đồng phẳng. | 🔴 |
| Plane Cornerto Corner | `SurfacePlaneCornertoCorner` | Tạo mặt phẳng bằng hai góc. | 🔴 |
| Surface Rebuild | `OthersSurfaceRebuild` | Dựng lại mặt với cấu trúc control points mới. | 🔴 |
| Blend Surface | `SurfaceBlendSurface` | Tạo mặt chuyển tiếp giữa các mặt. | 🔴 |
| Variable Blend Surfaces | `SurfaceVariableFilletBlendChamferVariableBlendSurfaces` | Tạo blend giữa các mặt với tham số biến thiên. | 🔴 |
| Loft | `SurfaceLoft` | Native Loft: Match Start/End Tangents, Refit, History và các style trong giới hạn host. [Phạm vi](../../docs/features/OM9-SURFACE-009.md). | 🟢 |
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
| Union | `SolidUnion` | Native BooleanUnion: [phạm vi, tùy chọn và giới hạn](../../docs/features/OM9-SOLID-003.md). | 🟢 |
| Difference | `SolidDifference` | Native BooleanDifference: [phạm vi, tùy chọn và giới hạn](../../docs/features/OM9-SOLID-001.md). | 🟢 |
| Intersection | `SolidIntersection` | Native BooleanIntersection: [phạm vi, tùy chọn và giới hạn](../../docs/features/OM9-SOLID-002.md). | 🟢 |
| Boolean Two Objects | `SolidBooleanTwoObjects` | Native Boolean2Objects: [phạm vi, tùy chọn và giới hạn](../../docs/features/OM9-SOLID-004.md). | 🟢 |
| Cap Planar Holes | `SolidCapPlanarHoles` | Đóng các lỗ đồng phẳng để tạo khối kín. | 🔴 |
| Extract Surface | `SolidExtractSurface` | Trích một mặt khỏi khối hoặc polysurface. | 🔴 |
| Fillet Edge | `SolidFilletEdgeFilletEdge` | Bo tròn các cạnh khối. | 🔴 |
| Text | `SolidText` | Tạo hình học chữ dạng khối. | 🔴 |
| Extrude Planar Curve Straight | `SolidExtrudePlanarCurveStraight` | Extrude đường kín đồng phẳng theo phương thẳng. | 🔴 |
| GVDAll Extrude | `SolidGVDAllExtrude` | Extrude theo workflow GVD. | 🔴 |
| Pipe | `SolidPipe` | Tạo ống theo đường dẫn. | 🔴 |
| Box Cornerto Corner Height | `SolidBoxCornertoCornerHeight` | Native Box: Corners/Diagonal/3Point/Vertical/Center/Cube, CMD/mouse, preview và persistence. [Phạm vi](../../OpenMatrix9_Codex_Spec_v1/specs/04-solid/om9-solid-012-box.md). | 🟢 |
| Sphere Center Radius | `SolidSphereCenterRadius` | Native Sphere: Center/Diameter, 2/3/4Point/Radius, Vertical, FitPoints, AroundCurve, planar Tangent. [Phạm vi](../../OpenMatrix9_Codex_Spec_v1/specs/04-solid/om9-solid-014-sphere.md). | 🟢 |
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
| Cage Edit | `TransformCageEditingCageEdit` | Native existing 3D Cage: Global/Local Mesh và exact Edge/rectangular Face; general BRep global affine-only. [Phạm vi/giới hạn](../../docs/features/cage-command-native-contract.md), [validation](../../docs/validation/2026-10-09-reusable-builder-cage.md). | 🟡 |
| Release From Cage | `ReleaseFromCage` | Native selected-only detach, giữ current geometry/control/unselected bindings; Undo và FCStd. [Validation](../../docs/validation/2026-10-09-reusable-builder-cage.md). | 🟡 |
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
| Create Cage | `TransformCageEditingCreateCage` | Native World BoundingBox với positive X/Y/Z extents; explicit counts/degrees, editable CageControl, không tự capture. [Contract](../../docs/features/cage-command-native-contract.md). | 🟡 |

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
| Join History | `gvJoinHistory` | Native curve [Join History](../../docs/features/history-native-contract.md); supported SDK83/cold4 đã kiểm chứng. | 🟢 |
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
| Match Attributes | `OthersMatchAttributes` | Original command/UI vẫn disabled; reusable native `matchBuilderAttributes` API có same-shape durable records. [Phạm vi API](../../docs/features/builder-history-native-contract.md). | 🔴 |
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
| Gumball Alignment | `InfoSettingsGumballAlignment` | CPlane/Object/World đã có; [spec mới](../../OpenMatrix9_Codex_Spec_v1/specs/01-core/om9-info-021-gumball-alignment.md) yêu cầu frame đồng bộ bốn view, chưa triển khai. | 🟡 |
| Relocate Gumball | `InfoSettingsRelocateGumball` | Đổi pivot bằng CMD/Ctrl đã có; [spec mới](../../OpenMatrix9_Codex_Spec_v1/specs/01-core/om9-info-022-relocate-gumball.md) yêu cầu Relocate/preview đồng bộ bốn view, chưa triển khai. | 🟡 |
| Gumball ON | `InfoSettingsGumballON` | Command On/Off và core MSR đã có; [spec theo ảnh/PDF](../../OpenMatrix9_Codex_Spec_v1/specs/01-core/om9-info-020-gumball.md) yêu cầu hiện cả bốn view và sửa Scale/Plane/Menu. Bản visual hiện chỉ 1/4 overlays. | 🟡 |
| Smart Targets Gumball ON | `InfoSettingsSmartTargetsGumballON` | Bật gizmo cho Smart Targets. | 🔴 |
| Rhino Smart Track ON | `RhinoSmartTrackON` | Bật tracking thông minh theo catalog. | 🔴 |
| Rhino History ON | `RhinoHistoryON` | [History policy](../../docs/features/history-native-contract.md): Record/Update/Lock/warning trên Join/Surface/Builder; Cage bindings cập nhật độc lập Record/Update. [Reusable validation](../../docs/validation/2026-10-09-reusable-builder-cage.md). | 🟢 |
| GVHistory Update ON | `GVHistoryUpdateON` | RCORE-09: Update suspend/resume các liên kết đã ghi, độc lập Record, trên supported Join/Surface/Builder; Cage bindings độc lập global policy. | 🟢 |
| GVHistory Record ON | `GVHistoryRecordON` | RCORE-09: Record chỉ điều khiển liên kết mới; Record Off không dừng descendant đã ghi khi Update On. Cage capture giữ persistent binding khi Record Off. | 🟢 |
| GVClear History | `InfoSettingsGVClearHistory` | Clear incoming native Join/Surface/Builder links hoặc selected Cage influence; durable Builder recipe được giữ. | 🟢 |
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
phải cùng một đơn vị đo. Danh mục giao diện: [MainMenu.ini](../../Resources/menu/MainMenu.ini);
đường dispatch: [Command.cpp](../../Gui/Command.cpp) và [Rust catalog](../../rust/src/ffi.rs).

Kiểm tra theo nhóm: [workspace](../../tests/workspace_smoke.FCMacro),
[curve](../../tests/curve_smoke.FCMacro), [views](../../tests/core_views_smoke.FCMacro),
[view controls](../../tests/core_view_controls_smoke.FCMacro),
[picture frame](../../tests/core_picture_frame_smoke.FCMacro),
[distance](../../tests/core_distance_smoke.FCMacro), [angle](../../tests/core_angle_smoke.FCMacro),
[snap tools](../../tests/core_snap_point_tools_smoke.FCMacro),
[keyboard](../../tests/core_keyboard_smoke.FCMacro), [notes](../../tests/core_notes_smoke.FCMacro).


## Trao đổi 3DM và openNURBS

Tiến độ openNURBS trong bản phát triển, cập nhật 2026-10-08. ✅ là phạm vi đã kiểm chứng; ⬜ là chức năng chưa hoàn thành. Code mới có trong nhánh này; checkout đang làm việc của người dùng được giữ nguyên.

Import 3DM trong bản phát triển đã được tăng tốc bằng tiến trình native riêng, giữ chữ ký và archive nguồn. [Số liệu và cách mở bản nhanh](../../docs/validation/2026-10-08-fast-3dm-import.md).

Menu **Import3dm → Working geometry (continue modeling)** tạo đối tượng CAD,
mesh và point cloud độc lập để tiếp tục thiết kế. **Export3dm → Working geometry**
xuất hình học hiện tại của phần chọn, gồm đối tượng đã sửa và đối tượng mới, sang
Rhino 5/V5. Nhóm `App::Part` được mở rộng đủ các thành viên; `PartDesign::Body`
xuất hình dạng kết quả cuối. Dữ liệu không được hỗ trợ bị từ chối trước khi sửa
document hoặc thay file đích.

Chế độ Working geometry chủ động bỏ history, render, material, texture, light,
layout và userdata. Chọn **Preserve source data** khi cần giữ các dữ liệu này.
Phạm vi và kiểm chứng bản gộp: [báo cáo modeling exchange](../../docs/validation/2026-10-08-modeling-public-integration.md).

|Chức năng|Ý nghĩa và phạm vi|Tiến độ|
|---|---|---|
|Import 3dm nhanh (Preserve)|Tối đa4 tiến trình native; giữ dữ liệu/UUID, kiểm tra lỗi trước transaction; đã đo nhẫn cụ thể, còn Geometry only tuần tự.|✅|
|Working geometry import/export|CAD/mesh/point cloud độc lập; import qua worker, export hình học hiện tại và đối tượng mới. Không giữ dữ liệu phụ của chế độ Preserve; kiểm chứng Rhino5 của bản gộp được ghi riêng.|✅|
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
|Hatch typed loop edits (API)|Sửa CV/weight/knot/radius/điểm/đường ghép, thêm/xóa biên; Undo/Redo, copy/block và FCStd trong phạm vi test; payload do plugin lưu.|✅|
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
|Full openNURBS|Còn references/UV, multi-shell, annotation, geometry, resources/document/version và integration. [Danh sách còn thiếu](../../docs/validation/2026-10-07-full-opennurbs-gap-audit.md).|⬜|

Lần Rhino5 trước khi sửa RDK:8 file, FreeCAD reimport25/25; đối chiếu nguồn/export40/41, chưa đạt toàn bộ. [Kết quả và vấn đề RDK](../../docs/validation/2026-10-07-rhino5-application-test.md).

Kiểm tra: native35/35, GUI1910/1910, nhẫn894/894; Rhino5 10/10 vòng mở/lưu/mở lại, nhập ngược33/33. Kiểm chứng hình học bổ sung50/50 ở ngưỡng0,001 mm; bbox Rhino5 cũ49/50 được giữ riêng vì bỏ sót điểm thật. Hướng+2 trong phạm vi một solid đã kiểm chứng; full openNURBS vẫn đang hoàn thiện. Xem [báo cáo](../../docs/validation/2026-10-07-rhino5-ring-bounds.md).

Full regression trước đó: FreeCAD1901/1901; native30/30. Rust không đổi, giữ bằng chứng75/75. [Bằng chứng và giới hạn](../../docs/validation/2026-10-07-trim-domain-correspondence.md).

Payload archive/Hatch được plugin lưu trong FCStd, không cần bản sửa core `PropertyFileIncluded`. Project cũ được chuyển có kiểm tra hash và hỗ trợ Undo/Redo; xem [kiểm chứng trên core gốc](../../docs/validation/2026-10-08-plugin-payload-storage.md). Binary OM9 vẫn phải khớp SDK/ABI của bản FreeCAD đang dùng; xem [hướng dẫn build](../../docs/build-windows.md).

Giao diện trường số Hatch loop qua104 kiểm tra, thao tác hàng CV/knot/point/segment qua103; full regression1479/1479. Còn tạo/đổi kiểu native và rationality, surface/reference/plugin, pattern/render và Rhino5 thực tế. [Phạm vi và bằng chứng](../../docs/validation/2026-10-07-3dm-hatch-loop-rows.md).

CurveOnSurface giữ nguyên nguồn trong FCStd. 30 profile không có C3 đã qua Rhino5 thực tế và giữ đúng trường native khi nhập ngược; những loại con chưa kiểm chứng vẫn bị chặn. Bản sửa SDK chỉ thay hàm Read trong file build sinh ra, nguồn SDK gốc vẫn nguyên vẹn. Mất đối tượng có C3 là tương thích reader Rhino5 đã quan sát, không phải kết luận rằng định dạng V5 không chứa được C3.

### Rhino5 bounds follow-up

10/10 mở/lưu/mở lại; nhập ngược33/33. Kiểm chứng hình học bổ sung50/50 ở ngưỡng0,001 mm; bbox Rhino cũ49/50 được giữ riêng. [Báo cáo và giới hạn phép đo](../../docs/validation/2026-10-07-rhino5-ring-bounds.md). Full openNURBS vẫn đang hoàn thiện.

### Kiểm toán full openNURBS — 2026-10-07

Danh mục chuẩn hóa có131 khai báo nguồn,128 lớp runtime đã đối chiếu trong FreeCAD; phân loại abstract/helper/obsolete/concrete riêng. Năm dòng comment và ba lớp obsolete không được SDK build đã được ghi rõ. Báo cáo cũ128/123 giữ nguyên làm lịch sử, không còn là inventory đầy đủ. Không dùng số lớp hay số assertions để tính phần trăm hoàn thành. Regression hiện tại42/42 native; Rhino5 GUI35/35 profile, nhập ngược API/GUI qua484 kiểm tra FreeCAD. Bốn mẫu shell cong qua46 kiểm tra FreeCAD và29 kiểm tra nhập ngược file Rhino đã lưu; phép đo Rhino chính xác8/8 giữ ngưỡng1e-6 mm³, báo cáo mặc định1/4 giữ riêng; `Shape.Volume` mặc định còn sai số tích phân được ghi riêng, kiểm chứng dùng tích phân thích nghi với ngưỡng thể tích giữ nguyên. [Capability từng trục](../../docs/3dm-capabilities.json) · [Capability từng thuộc tính](../../docs/3dm-capability-slices.json) · [Audit hiện tại](../../docs/3dm-current-support-audit.json) · [Phạm vi đã kiểm chứng và phần còn thiếu](../../docs/validation/2026-10-07-opennurbs-packages-1-3.md).


Xem [phạm vi nhánh mới và cách kiểm tra](../../docs/public-update-2026-10-08.md).

The project follows its existing **LGPL-2.1-or-later** declaration; the license
text is in [LICENSE](../../LICENSE). Separate dependency notices remain applicable.

### Surface advanced và History — 2026-10-09

Refit, Sweep2 G1/G2, Add Slash và Loft tangent matching đã có native geometry trong [giới hạn host](../../docs/features/surface-advanced-options.md). Bật History trong options để sửa rail/profile và tự dựng lại mặt; CMD `gvSweepHistory` / `gvSweep2History` tạo variant002/004. Native type giữ dependency qua Undo/Redo/FCStd, kể cả mở file trong tiến trình mới. **530/530 kiểm tra native**, 164 Rust và23 Python đã qua; [bằng chứng](../../docs/validation/2026-10-09-surface-constraints-history.md).

### Nguồn Rhino 5 cho core

[Hướng dẫn và bảng tra Rhino 5](../../ref/rhino5/README.md) ánh xạ tài liệu gốc,
Command Help và các bước kiểm chứng sang 12 chương
[00-rhino-core](../../OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/README.md).
Đọc nguồn theo phần chức năng trước khi triển khai và chạy fixture trước khi
xác nhận tương đương. Đây là ngoại lệ cho tài liệu Rhino 5 được người dùng
cho phép; code decompile, tài nguyên và tài liệu Matrix vẫn ở kho riêng.

<!-- gumball-core-20261011:start -->
OM9-INFO-020/021/022: `partially_implemented` / `validated_supported_slice`.
Native 13 handles, Move/Rotate/Scale/Copy/planar Extrude, numeric/modifiers,
CPlane/Object/World, Relocate and single-owner view-state cache implemented.
Actual Qt/DPI, batch callback rollback and FCStd/retained 3DM cold proof;
unsupported owners/frames and shared-host failed-transaction Redo loss remain explicit.
[Evidence, reproducible suite and limits](../../docs/validation/2026-10-11-gumball-core.md).
Final fresh review: three Important findings fixed by TDD; native432/16 and Rust808/115 passed.
<!-- gumball-core-20261011:end -->

<!-- gumball-four-view-spec-20261011:start -->
2026-10-11: đã triển khai Gumball đồng bộ bốn viewport: Command On/Off,
first-click handles, shared preview/batch, bounds Scale, plane hai màu và menu
ring; core settings/Snappy/Strength/orbit và native Box/Sphere/Link property Scale.
Giữ ba ID `partially_implemented`: generic owners, Mesh edge/vertex/light/camera
frames và host old-Redo rollback còn mở; chưa claim full original-host parity.
[Cách dùng Gumball](../../docs/gumball-user-guide.md) · [Native evidence](../../docs/validation/2026-10-11-gumball-four-view-completion.md).
<!-- gumball-four-view-spec-20261011:end -->

## Additional development checkpoint

<!-- rhino-core-notification-lock-20261012:start -->
Đã sửa startup resource-deadlock tại FreeCAD NotificationArea: dựng nội dung dưới mutex rồi thả khóa trước khi mở popup; cảnh báo Qt có thể vào logger bình thường. Popup thật vẫn giữ thông báo và warning, không tắt thông báo để né lỗi.
**App273/plugin275/GUI287/kernel240:3375/3375 kiểm tra qua35 host sạch log, tất cả exit0.** Fixture popup RED2/3→GREEN3/3, stack native xác nhận nguyên nhân. Giữ các lỗi startup274/278/284 và các crash cũ làm evidence lịch sử. Rust toàn bộ lần gần nhất1037/0, không chạy lại cho thay đổi GUI native này.
**RCORE-01 vẫn đang triển khai:** còn các phần chưa có evidence đầy đủ về conversion, consumer/result, ownership/preservation, aggregate bounds và lifecycle/cache; T01-T11 đã có accepted slices, không mở lại toàn bộ chỉ vì checkbox lịch sử.
[Bằng chứng notification lock](../../docs/validation/2026-10-12-rhino-core-notification-lock.md).
<!-- rhino-core-notification-lock-20261012:end -->

## Published README before redesign

<!-- published-readme-snapshot:fc9fb83f5d5db5c1b55fb35d7790e58c24f5ac36 -->

Preserved from public main at commit `fc9fb83f5d5db5c1b55fb35d7790e58c24f5ac36`. Version and host limitations
below belong to that historical checkpoint.

# OpenMatrix9

<!-- om9-install-entry:start -->
**Jewelry CAD for FreeCAD · Windows x64**

### Cài đặt nhanh / Install in 3 steps

1. Cài **[FreeCAD 1.1.4 chính thức](https://github.com/FreeCAD/FreeCAD/releases/tag/1.1.4)** / Install official FreeCAD 1.1.4 and open it once.
2. Mở **[trang Releases](https://github.com/MYuitsu/OpenMatrix9/releases)**. Trong bản **0.3.4-202610120606**, mở mục **Assets** và tải **OpenMatrix9-0.3.4-202610120606-SetupLauncher-Windows-x64.zip**. / Open Releases, expand Assets and download the SetupLauncher ZIP for this version.
3. Giải nén ZIP SetupLauncher vào một thư mục rồi bấm **Start-OM9.cmd**: launcher cài OM9, tạo icon **OpenMatrix9** trên Desktop và mở workbench. / Extract the SetupLauncher ZIP, run Start-OM9.cmd, then use the Desktop icon afterwards.

Giữ thư mục **OpenMatrix9**, **launcher-manifest.json** và các file **Start-OM9** cùng nhau sau khi giải nén gói SetupLauncher. Kiểm tra hai ZIP bằng **SHA256SUMS.txt** trong cùng release. Không cần Rust, Visual Studio hay quyền Administrator để dùng OM9. / Keep the bundled OpenMatrix9 folder, manifest and Start-OM9 files together.

**Kiểm thử và build:** Gumball đã đạt **30 ca / 955 kiểm tra trên máy** với FreeCAD stock. Theo yêu cầu ngày 12/10/2026, GitHub Actions chỉ build và đóng gói; CI không chạy lại bộ test. Gói tải xuống ghi rõ commit, SHA256 và `tests_executed: false`. [Bằng chứng và quy trình](../../docs/validation/2026-10-12-gumball-main-release.md).

**Bản phát hành:** Releases cung cấp ZIP SetupLauncher để cài tự động, ZIP plugin để cài thủ công và **SHA256SUMS.txt**. Các bản thử được ghi **Pre-release**. Artifact trong Actions dành cho kiểm tra build; tải bản cài đặt ở Releases.

[Hướng dẫn tiếng Việt](../../INSTALL.vi.md) · [English install guide](../../INSTALL.en.md) · [Tiến độ](../../README.vi.md) · [Build from source](../../docs/build-windows.md)
<!-- om9-install-entry:end -->

<!-- rhino-core-host-parameter-dependency:start -->
Đã nghiệm thu fixture T10 cho điểm phụ thuộc Circle/Line: dùng map openNURBS/OCCT
có sẵn, Rust kiểm điểm với source thật và chặn raw parameter chưa được chứng minh.
Query/basis/source identity được lưu FCStd và tính lại sau sửa/Undo/restore.
238 kiểm tra fixture +114 hồi quy FreeCAD;556 Rust và159 native query đạt.
Phạm vi Circle/Line và oracle lấy mẫu; **toàn bộ P0 vẫn đang triển khai.**
[Bằng chứng và giới hạn](../../docs/validation/2026-10-10-rhino-core-host-parameter-dependency.md).
<!-- rhino-core-host-parameter-dependency:end -->

<!-- rhino-core-delayed-publication:start -->
Đã nghiệm thu fixture T08 cho native Rebuild: kết quả được giữ qua lượt giao,
source/session/options hết hiệu lực bị chặn trước Coin/transaction;36 envelope
và36 tham chiếu Shape giải phóng đủ.173 kiểm tra fixture và72 hồi quy đạt.
Solver vẫn đồng bộ, delivery ở GUI queue; lifetime mọi worker vẫn còn mở.
**Toàn bộ P0 vẫn đang triển khai.** [Bằng chứng và giới hạn](../../docs/validation/2026-10-10-rhino-core-delayed-publication.md).
<!-- rhino-core-delayed-publication:end -->

<!-- rhino-core-subobject-ownership:start -->
Đã nghiệm thu fixture T11: sửa wrapper Curve/Surface sở hữu riêng, publish qua
native deep copy, giữ source/sibling, đúng Undo/Redo và phân biệt lifetime của
snapshot mạnh với request subobject.35 kiểm tra FreeCAD đạt; không viết lại editor.
**Toàn bộ P0 vẫn đang triển khai.** [Bằng chứng và giới hạn](../../docs/validation/2026-10-10-rhino-core-subobject-ownership.md).
<!-- rhino-core-subobject-ownership:end -->

<!-- rhino-core-periodic-persistence:start -->
Đã nghiệm thu T09 bằng curve periodic thật và các face có seam/lỗ; dùng lại
serializer FCStd hiện có.204 kiểm tra warm +102 kiểm tra cold đạt, cùng221 kiểm
tra đối chứng kernel. Type/topology/basis/tọa độ và inventory giữ nguyên.
**Toàn bộ P0 vẫn đang triển khai.** [Bằng chứng và giới hạn](../../docs/validation/2026-10-10-rhino-core-periodic-persistence.md).
<!-- rhino-core-periodic-persistence:end -->

<!-- rhino-core-hatch-input-gate:start -->
Đã bổ sung gate Rust cho dữ liệu NURBS của vòng Hatch trước đọc nguồn/copy/dựng
native; dùng lại editor và exporter hiện có.541 kiểm tra Rust,27 kiểm tra native,
5 bộ hồi quy native và269 kiểm tra FreeCAD đạt. Đây là một phần T07;
**toàn bộ P0 vẫn đang triển khai**. [Bằng chứng và giới hạn](../../docs/validation/2026-10-10-rhino-core-hatch-input-gate.md).
<!-- rhino-core-hatch-input-gate:end -->

<!-- rhino-core-instance-isolation:start -->
Dùng lại App::Link, mirror/scale không đều và các exporter hiện có; thêm fixture
kiểm đúng tọa độ, giữ nguyên member/definition/instance A khi chỉ sửa B.
T06 đã nghiệm thu:146 kiểm tra native,98 kiểm tra FreeCAD và4 bộ hồi quy native.
Không cần viết lại code instance. **Toàn bộ P0 vẫn đang triển khai.**
[Bằng chứng và giới hạn](../../docs/validation/2026-10-10-rhino-core-instance-isolation.md).
<!-- rhino-core-instance-isolation:end -->

<!-- rhino-core-polycurve-partition:start -->
Dùng lại wire, đảo hướng và FCStd của FreeCAD; sửa bước working 3DM để polycurve
giữ từng đoạn thay vì gộp thành một cạnh. Rust kiểm tra thứ tự, miền và điểm nối;
báo cáo có trước commit. T05 đã nghiệm thu với 403 kiểm tra native, 382 kiểm tra
FreeCAD và 523 Rust tests. **Toàn bộ P0 vẫn đang triển khai.**
[Bằng chứng và giới hạn](../../docs/validation/2026-10-10-rhino-core-polycurve-partition.md).
<!-- rhino-core-polycurve-partition:end -->

<!-- rhino-core-curve-parameters:start -->
Dùng lại openNURBS/OCCT cho báo cáo query tham số toàn miền: giữ native form1/2,
chặn output lỗi trước khi tính điểm và giữ basis/report độc lập. 502 Rust tests;
159 kiểm tra query native; 257 kiểm tra FreeCAD. Fixture RCORE-01.T01 đã nghiệm thu
với oracle lấy mẫu; T10/History vẫn thiếu tích hợp host. **Toàn bộ P0 đang triển khai.**
[Bằng chứng và giới hạn](../../docs/validation/2026-10-10-rhino-core-curve-parameters.md).
<!-- rhino-core-curve-parameters:end -->

<!-- rhino-core-kernel-contract:start -->
Dùng lại bộ chuyển đổi FreeCAD/OCCT/openNURBS; bổ sung validator basis Rust,
descriptor chỉ đọc và policy cho inspection/working 3DM. 480 Rust tests;
504 kiểm tra ứng dụng / 8 lần chạy, gồm mở FCStd trong tiến trình mới.
**P0 toàn bộ vẫn đang triển khai.** [Bằng chứng và phạm vi](../../docs/validation/2026-10-10-rhino-core-kernel-contract.md).
<!-- rhino-core-kernel-contract:end -->

<!-- rhino-core-extended-input:start -->
Bổ sung nhập polar/spherical/surveyor, fraction/feet-inch và góc DMS bằng Rust,
dùng lại bộ dựng hình FreeCAD/OM9 hiện có. Sửa mốc tương đối và Undo của Circle/Sphere.
**421 Rust tests; 1108 native checks / 21 runs; 49 Python pass,1 skip.**
P0 toàn bộ còn mở; [phạm vi và bằng chứng](../../docs/validation/2026-10-09-rhino-core-extended-input.md).

<!-- rhino-core-extended-input:end -->

<!-- rhino-core-point-consumers:start -->
Distance, Angle và PictureFrame đã dùng chung parser Rust, giữ tọa độ/đơn vị đầu vào và provenance.
CPlane sai scale/shear bị từ chối; PictureFrame kiểm giao dịch trước mutation.
**324 native checks / 9 runs; 364 Rust tests; 49 Python pass, 1 skip.**
P0 toàn bộ vẫn đang triển khai; [phạm vi và bằng chứng](../../docs/validation/2026-10-09-rhino-core-point-consumers.md).

<!-- rhino-core-point-consumers:end -->

<!-- rhino-core-tolerance-workers:start -->
Rust đã nối dung sai document/override vào Join curve và surface, lưu cutoff thực tế
và provenance cùng kết quả; backend không được tự nâng dung sai. Worker nhập 3DM
dùng 60% logical CPUs, bỏ trần cố định 4, kiểm override trước mutation.
**444 native checks / 8 runs; 347 Rust tests; 49 Python pass, 1 skip.**
P0 toàn bộ vẫn đang triển khai; [báo cáo và giới hạn](../../docs/validation/2026-10-09-rhino-core-tolerance-workers.md) ghi phạm vi chính xác.

<!-- rhino-core-tolerance-workers:end -->

<!-- rhino-core-object-foundations:start -->
Nền tảng P0 tiếp theo — **object identity và stale selection**: Rust registry
đã được nối vào Edit/Boolean, Rebuild và Surface tương tác. Lựa chọn bị thay thế
hoặc chỉnh sửa được chặn trước khi ghi kết quả; snapshot không sửa document.
[Phạm vi, kiểm thử và phần còn thiếu](../../docs/validation/2026-10-09-rhino-core-object-foundations.md). P0 vẫn đang triển khai;
các checkpoint bên dưới là bằng chứng riêng của từng phiên bản.
<!-- rhino-core-object-foundations:end -->

[Tiếng Việt — mục lục và tiến độ theo nhóm](../../README.vi.md) ·
[English — contents and group progress](../../README.en.md)

![OpenMatrix9 — Open-source Jewelry CAD Workbench](../../Resources/branding/introduction.png)

An experimental jewelry CAD workbench for FreeCAD. Rust owns command catalog,
state and behavior; C++/Qt provides native integration. Unsupported commands
remain visible and disabled.

<!-- rhino-core-p0-checkpoint:start -->
Nền tảng Rhino P0 — 2026-10-09: đã bổ sung context đơn vị/tolerance tường minh,
parser điểm dùng chung, CPlane theo viewport và menu, vòng đời snap một lần/repeat,
validation spline/3DM và tách History Record khỏi Update.
Đây là **triển khai một phần P0**; chưa hoàn tất registry/selection identity chung,
tolerance resolver cho mọi solver, History schema chung hoặc xuất 3DM giữ trạng thái đã sửa.
Phạm vi, kết quả kiểm thử và các phần còn thiếu nằm trong
[báo cáo P0](../../docs/validation/2026-10-09-rhino-core-p0.md).
Các kết quả 321 Rust, 23 Python và 657 native GUI assertions thuộc snapshot CAD/P0
trước gộp; không xác nhận phiên bản sau gộp. Bản native đó được build và thử ở
runtime riêng; ứng dụng đang mở chưa tự nhận module mới.
<!-- rhino-core-p0-checkpoint:end -->

<!-- main-merge-validation -->
Kiểm chứng sau gộp `main` — 2026-10-09: 334 kiểm thử Rust, 49 Python
(1 bỏ qua), 8 bộ native và 847 kiểm tra GUI đều qua. Xem
[báo cáo merge](../../docs/validation/2026-10-09-main-merge.md) cho phạm vi,
module đã dùng và các giới hạn còn lại. P0 vẫn chưa hoàn tất.
<!-- /main-merge-validation -->

This project was developed with Matrix9 as a functional reference. Compatibility
identifiers and familiar color roles remain. This public snapshot excludes
commercial artwork, extracted resource archives, Matrix manuals, decompiled source
and private design samples. The approved pinned Rhino 5 guide is documented in
[the reference policy](../../ref/rhino5/README.md). It does not claim clean-room development, affiliation
with the original vendors, or legal clearance of the implementation.

All **510 icon bindings** use independently authored SVGs. The exporter accepts
authored catalogs only and cannot restore legacy bitmaps.

## Ủng hộ phát triển / Donate

Nếu OpenMatrix9 hữu ích với bạn, bạn có thể donate để hỗ trợ phát triển và kiểm
thử các chức năng CAD, import/export 3DM và tương thích openNURBS.
Mọi đóng góp đều tự nguyện. Cảm ơn bạn đã hỗ trợ dự án!

- **PayPal:** gửi tới **`nguyenthaiduy277@gmail.com`** trong PayPal.
- **MoMo:** quét mã QR dưới đây. Tên người nhận: **NGUYEN THAI DUY**.

<img src="../../docs/images/donate/momo-qr.png" alt="Mã QR MoMo donate cho NGUYEN THAI DUY" width="300">

## Build và cài đặt

Xem [hướng dẫn build FreeCAD và OpenMatrix9 trên Windows](../../docs/build-windows.md):

- **Build cùng FreeCAD từ source**: clone workspace, tích hợp CMake, Pixi, build và chạy.
- **Build riêng OpenMatrix9**: dùng source/build SDK và dependencies của FreeCAD đã build.
- **Dùng với FreeCAD installer/portable**: yêu cầu ABI/SDK khớp, vị trí `.pyd` và resources, kiểm tra sau cài.

OpenMatrix9 có module native C++/Rust; chỉ clone vào `Mod` chưa đủ.
Bản cài FreeCAD thông thường không tự cung cấp SDK để build module này.

<!-- edit-options-validation -->
Edit cập nhật 2026-10-09: Trim ExtendLines/ApparentIntersections (curve,
orthographic/perspective frozen view), Join midpoint/tolerance, Explode nhóm/Mesh,
Boolean mặt/Mesh với các giới hạn rõ trong [native contract](../../docs/features/edit-native-contract.md).
Đã qua 115 core + 57 options kiểm tra native, Rust 140 và Python 23.
Lệnh vẫn `partially_implemented`; các kết quả trên chỉ xác nhận geometry slice trước nhánh mới bên dưới.
Audit lịch sử của lần kiểm chứng này báo 290 artifact tham chiếu/private có sẵn
trong workspace; không có file triển khai Edit bị gắn cờ. Báo cáo đó không xác nhận
trạng thái audit của phiên bản sau gộp.
<!-- /edit-options-validation -->

<!-- history-special-explode-validation -->
History/Explode cập nhật 2026-10-09: [Join History và policy native](../../docs/features/history-native-contract.md)
đã có Record/Update/Lock/warning/Clear; [Explode adapters](../../docs/features/edit-native-contract.md)
cho block qua CMD, text/dimension/cage native và retained 3DM đã có triển khai.
Final SDK retained3DM40/40 và native special25/25 pass.
History SDK83/83 + cold4/4 gồm grouped frames/invalid chain; core115/options57 pass.
Chi tiết trong [validation record](../../docs/validation/2026-10-09-edit-history-special-types.md).
Ordinary Edit phá links Join/Surface History liên quan; unsupported font/view-dependent
annotation/custom arrow bị từ chối atomically. Không xác nhận đầy đủ Matrix History.
<!-- /history-special-explode-validation -->


<!-- reusable-builder-cage-validation -->
### Reusable Builder API và Cage — 2026-10-09

`createBuilderRecord`, `restoreBuilderOutputs` và `matchBuilderAttributes` cung cấp
durable native record/output và versioned affine-template replay; parameters tồn
tại sau khi output bị xóa, restore là thao tác tường minh. Match API yêu cầu
same explicit OM9GemShape và áp toàn bộ supported records atomically. Đây là
nền tảng API; original Gem builder solvers, Match styles popup, sliders/handles,
Styles UI và .mss vẫn chưa có, các original command tương ứng vẫn disabled.
Rust sở hữu validation/recipe state; C++ là adapter bắt buộc cho FreeCAD document,
transactions/persistence/signals và OCCT geometry.

Cage/Create Cage/ReleaseFromCage có native menu/CMD cho World BoundingBox và
existing 3D cage, Global/Local trong giới hạn geometry đã công bố. Cage links
cập nhật khi global History Record/Update Off. 1D/2D controls, Accurate/Fast và
general BRep refit vẫn unsupported. Xem [Builder API](../../docs/features/builder-history-native-contract.md),
[Cage contract](../../docs/features/cage-command-native-contract.md) và
[bằng chứng native](../../docs/validation/2026-10-09-reusable-builder-cage.md).
<!-- /reusable-builder-cage-validation -->

## Tiến độ lệnh / Command progress

Cập nhật: **2026-10-09**. Bảng này mô tả phạm vi đang có trong source public.
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
| Explode | `TopIconExplode` | Native Explode: [phạm vi, tùy chọn và giới hạn](../../docs/features/OM9-TOP11-005.md). | 🟢 |
| Join | `TopIconJoin` | Native Join: [phạm vi, tùy chọn và giới hạn](../../docs/features/OM9-TOP11-008.md). | 🟢 |
| Split | `TopIconSplit` | Chia đối tượng bằng đối tượng cắt. | 🔴 |
| Trim | `TopIconTrim` | Native Trim: [phạm vi, tùy chọn và giới hạn](../../docs/features/OM9-TOP11-010.md). | 🟢 |
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
| Interpolate Points | `CurveFreeFormInterpolatePoints` | Vẽ B-spline qua điểm bằng chuột/CMD; Degree, Knots, Close/Sharp, Undo. Tangent chưa hỗ trợ. | 🟢 |
| Rectangle Cornerto Corner | `CurveRectangleCornertoCorner` | Hai góc, Center, 3Point, Vertical, Length/Width và Shift square; preview, Undo/Redo, FCStd. Rounded/Conic chưa hỗ trợ. Kiểm chứng 2026-10-09: [OM9-CURVE-004](../../docs/features/OM9-CURVE-004.md). | 🟢 |
| Circle Center Radius | `CurveCircleCenterRadius` | Cơ bản/nâng cao; Tangent 3D không đồng phẳng; History mọi nhánh gồm Deformable; layer hiện hành/màu/ẩn/khóa; Undo/Redo và cold FCStd. Bộ giải có giới hạn tìm nghiệm. [OM9-CURVE-005](../../docs/features/OM9-CURVE-005.md). | 🟢 |
| Ellipse From Center | `CurveEllipseFromCenter` | Center, Diameter/chuột phải, Corner, Vertical, FromFoci/MarkFoci, AroundCurve, Deformable; preview, layer, Undo/Redo và FCStd. Snapshot; History chưa hỗ trợ. [OM9-CURVE-006](../../docs/features/OM9-CURVE-006.md). | 🟢 |
| Arc Center Start Angle | `CurveArcCenterStartAngle` | Vẽ cung tròn từ tâm, điểm đầu và góc. | 🔴 |
| Arc Start End Direction | `CurveArcStartEndDirection` | Vẽ cung tròn theo điểm đầu, điểm cuối và hướng. | 🔴 |
| Curve Rebuild | `OthersCurveRebuild` | Dựng lại curve/polyline theo PointCount/Degree; preview độ lệch lấy mẫu, DeleteInput, Undo. Current layer chưa hỗ trợ. | 🟢 |
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
| Sweep1 Rail | `SurfaceSweepSweep1Rail` | Native Sweep1: Refit theo dung sai, History tùy chọn, Chain Edges, seams, Rebuild/Closed và Preview. [Phạm vi](../../docs/features/OM9-SURFACE-001.md). | 🟢 |
| Sweep2 Rails | `SurfaceSweepSweep2Rails` | Native Sweep2: G1/G2 cạnh mặt, Refit, Add Slash một profile, History và Maintain Height trong giới hạn host. [Phạm vi](../../docs/features/OM9-SURFACE-003.md). | 🟢 |
| Profile Sweep | `SurfaceSweepProfileSweep` | Quét mặt từ profile theo workflow Profile Sweep. | 🔴 |
| Planar Curves | `SurfacePlanarCurves` | Tạo mặt phẳng từ các đường biên đồng phẳng. | 🔴 |
| Plane Cornerto Corner | `SurfacePlaneCornertoCorner` | Tạo mặt phẳng bằng hai góc. | 🔴 |
| Surface Rebuild | `OthersSurfaceRebuild` | Dựng lại mặt với cấu trúc control points mới. | 🔴 |
| Blend Surface | `SurfaceBlendSurface` | Tạo mặt chuyển tiếp giữa các mặt. | 🔴 |
| Variable Blend Surfaces | `SurfaceVariableFilletBlendChamferVariableBlendSurfaces` | Tạo blend giữa các mặt với tham số biến thiên. | 🔴 |
| Loft | `SurfaceLoft` | Native Loft: Match Start/End Tangents, Refit, History và các style trong giới hạn host. [Phạm vi](../../docs/features/OM9-SURFACE-009.md). | 🟢 |
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
| Union | `SolidUnion` | Native BooleanUnion: [phạm vi, tùy chọn và giới hạn](../../docs/features/OM9-SOLID-003.md). | 🟢 |
| Difference | `SolidDifference` | Native BooleanDifference: [phạm vi, tùy chọn và giới hạn](../../docs/features/OM9-SOLID-001.md). | 🟢 |
| Intersection | `SolidIntersection` | Native BooleanIntersection: [phạm vi, tùy chọn và giới hạn](../../docs/features/OM9-SOLID-002.md). | 🟢 |
| Boolean Two Objects | `SolidBooleanTwoObjects` | Native Boolean2Objects: [phạm vi, tùy chọn và giới hạn](../../docs/features/OM9-SOLID-004.md). | 🟢 |
| Cap Planar Holes | `SolidCapPlanarHoles` | Đóng các lỗ đồng phẳng để tạo khối kín. | 🔴 |
| Extract Surface | `SolidExtractSurface` | Trích một mặt khỏi khối hoặc polysurface. | 🔴 |
| Fillet Edge | `SolidFilletEdgeFilletEdge` | Bo tròn các cạnh khối. | 🔴 |
| Text | `SolidText` | Tạo hình học chữ dạng khối. | 🔴 |
| Extrude Planar Curve Straight | `SolidExtrudePlanarCurveStraight` | Extrude đường kín đồng phẳng theo phương thẳng. | 🔴 |
| GVDAll Extrude | `SolidGVDAllExtrude` | Extrude theo workflow GVD. | 🔴 |
| Pipe | `SolidPipe` | Tạo ống theo đường dẫn. | 🔴 |
| Box Cornerto Corner Height | `SolidBoxCornertoCornerHeight` | Native Box: Corners/Diagonal/3Point/Vertical/Center/Cube, CMD/mouse, preview và persistence. [Phạm vi](../../OpenMatrix9_Codex_Spec_v1/specs/04-solid/om9-solid-012-box.md). | 🟢 |
| Sphere Center Radius | `SolidSphereCenterRadius` | Native Sphere: Center/Diameter, 2/3/4Point/Radius, Vertical, FitPoints, AroundCurve, planar Tangent. [Phạm vi](../../OpenMatrix9_Codex_Spec_v1/specs/04-solid/om9-solid-014-sphere.md). | 🟢 |
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
| Cage Edit | `TransformCageEditingCageEdit` | Native existing 3D Cage: Global/Local Mesh và exact Edge/rectangular Face; general BRep global affine-only. [Phạm vi/giới hạn](../../docs/features/cage-command-native-contract.md), [validation](../../docs/validation/2026-10-09-reusable-builder-cage.md). | 🟡 |
| Release From Cage | `ReleaseFromCage` | Native selected-only detach, giữ current geometry/control/unselected bindings; Undo và FCStd. [Validation](../../docs/validation/2026-10-09-reusable-builder-cage.md). | 🟡 |
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
| Create Cage | `TransformCageEditingCreateCage` | Native World BoundingBox với positive X/Y/Z extents; explicit counts/degrees, editable CageControl, không tự capture. [Contract](../../docs/features/cage-command-native-contract.md). | 🟡 |

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
| Join History | `gvJoinHistory` | Native curve [Join History](../../docs/features/history-native-contract.md); supported SDK83/cold4 đã kiểm chứng. | 🟢 |
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
| Match Attributes | `OthersMatchAttributes` | Original command/UI vẫn disabled; reusable native `matchBuilderAttributes` API có same-shape durable records. [Phạm vi API](../../docs/features/builder-history-native-contract.md). | 🔴 |
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
| Gumball Alignment | `InfoSettingsGumballAlignment` | Căn hệ trục gizmo thao tác. | 🟢 |
| Relocate Gumball | `InfoSettingsRelocateGumball` | Di chuyển tâm gizmo thao tác. | 🟢 |
| Gumball ON | `InfoSettingsGumballON` | Bật gizmo thao tác. | 🟢 |
| Smart Targets Gumball ON | `InfoSettingsSmartTargetsGumballON` | Bật gizmo cho Smart Targets. | 🔴 |
| Rhino Smart Track ON | `RhinoSmartTrackON` | Bật tracking thông minh theo catalog. | 🔴 |
| Rhino History ON | `RhinoHistoryON` | [History policy](../../docs/features/history-native-contract.md): Record/Update/Lock/warning trên Join/Surface/Builder; Cage bindings cập nhật độc lập Record/Update. [Reusable validation](../../docs/validation/2026-10-09-reusable-builder-cage.md). | 🟢 |
| GVHistory Update ON | `GVHistoryUpdateON` | RCORE-09: Update suspend/resume các liên kết đã ghi, độc lập Record, trên supported Join/Surface/Builder; Cage bindings độc lập global policy. | 🟢 |
| GVHistory Record ON | `GVHistoryRecordON` | RCORE-09: Record chỉ điều khiển liên kết mới; Record Off không dừng descendant đã ghi khi Update On. Cage capture giữ persistent binding khi Record Off. | 🟢 |
| GVClear History | `InfoSettingsGVClearHistory` | Clear incoming native Join/Surface/Builder links hoặc selected Cage influence; durable Builder recipe được giữ. | 🟢 |
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
phải cùng một đơn vị đo. Danh mục giao diện: [MainMenu.ini](../../Resources/menu/MainMenu.ini);
đường dispatch: [Command.cpp](../../Gui/Command.cpp) và [Rust catalog](../../rust/src/ffi.rs).

Kiểm tra theo nhóm: [workspace](../../tests/workspace_smoke.FCMacro),
[curve](../../tests/curve_smoke.FCMacro), [views](../../tests/core_views_smoke.FCMacro),
[view controls](../../tests/core_view_controls_smoke.FCMacro),
[picture frame](../../tests/core_picture_frame_smoke.FCMacro),
[distance](../../tests/core_distance_smoke.FCMacro), [angle](../../tests/core_angle_smoke.FCMacro),
[snap tools](../../tests/core_snap_point_tools_smoke.FCMacro),
[keyboard](../../tests/core_keyboard_smoke.FCMacro), [notes](../../tests/core_notes_smoke.FCMacro).


## Trao đổi 3DM và openNURBS

Tiến độ openNURBS trong bản phát triển, cập nhật 2026-10-08. ✅ là phạm vi đã kiểm chứng; ⬜ là chức năng chưa hoàn thành. Code mới có trong nhánh này; checkout đang làm việc của người dùng được giữ nguyên.

Import 3DM trong bản phát triển đã được tăng tốc bằng tiến trình native riêng, giữ chữ ký và archive nguồn. [Số liệu và cách mở bản nhanh](../../docs/validation/2026-10-08-fast-3dm-import.md).

Menu **Import3dm → Working geometry (continue modeling)** tạo đối tượng CAD,
mesh và point cloud độc lập để tiếp tục thiết kế. **Export3dm → Working geometry**
xuất hình học hiện tại của phần chọn, gồm đối tượng đã sửa và đối tượng mới, sang
Rhino 5/V5. Nhóm `App::Part` được mở rộng đủ các thành viên; `PartDesign::Body`
xuất hình dạng kết quả cuối. Dữ liệu không được hỗ trợ bị từ chối trước khi sửa
document hoặc thay file đích.

Chế độ Working geometry chủ động bỏ history, render, material, texture, light,
layout và userdata. Chọn **Preserve source data** khi cần giữ các dữ liệu này.
Phạm vi và kiểm chứng bản gộp: [báo cáo modeling exchange](../../docs/validation/2026-10-08-modeling-public-integration.md).

|Chức năng|Ý nghĩa và phạm vi|Tiến độ|
|---|---|---|
|Import 3dm nhanh (Preserve)|Tối đa4 tiến trình native; giữ dữ liệu/UUID, kiểm tra lỗi trước transaction; đã đo nhẫn cụ thể, còn Geometry only tuần tự.|✅|
|Working geometry import/export|CAD/mesh/point cloud độc lập; import qua worker, export hình học hiện tại và đối tượng mới. Không giữ dữ liệu phụ của chế độ Preserve; kiểm chứng Rhino5 của bản gộp được ghi riêng.|✅|
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
|Hatch typed loop edits (API)|Sửa CV/weight/knot/radius/điểm/đường ghép, thêm/xóa biên; Undo/Redo, copy/block và FCStd trong phạm vi test; payload do plugin lưu.|✅|
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
|Full openNURBS|Còn references/UV, multi-shell, annotation, geometry, resources/document/version và integration. [Danh sách còn thiếu](../../docs/validation/2026-10-07-full-opennurbs-gap-audit.md).|⬜|

Lần Rhino5 trước khi sửa RDK:8 file, FreeCAD reimport25/25; đối chiếu nguồn/export40/41, chưa đạt toàn bộ. [Kết quả và vấn đề RDK](../../docs/validation/2026-10-07-rhino5-application-test.md).

Kiểm tra: native35/35, GUI1910/1910, nhẫn894/894; Rhino5 10/10 vòng mở/lưu/mở lại, nhập ngược33/33. Kiểm chứng hình học bổ sung50/50 ở ngưỡng0,001 mm; bbox Rhino5 cũ49/50 được giữ riêng vì bỏ sót điểm thật. Hướng+2 trong phạm vi một solid đã kiểm chứng; full openNURBS vẫn đang hoàn thiện. Xem [báo cáo](../../docs/validation/2026-10-07-rhino5-ring-bounds.md).

Full regression trước đó: FreeCAD1901/1901; native30/30. Rust không đổi, giữ bằng chứng75/75. [Bằng chứng và giới hạn](../../docs/validation/2026-10-07-trim-domain-correspondence.md).

Payload archive/Hatch được plugin lưu trong FCStd, không cần bản sửa core `PropertyFileIncluded`. Project cũ được chuyển có kiểm tra hash và hỗ trợ Undo/Redo; xem [kiểm chứng trên core gốc](../../docs/validation/2026-10-08-plugin-payload-storage.md). Binary OM9 vẫn phải khớp SDK/ABI của bản FreeCAD đang dùng; xem [hướng dẫn build](../../docs/build-windows.md).

Giao diện trường số Hatch loop qua104 kiểm tra, thao tác hàng CV/knot/point/segment qua103; full regression1479/1479. Còn tạo/đổi kiểu native và rationality, surface/reference/plugin, pattern/render và Rhino5 thực tế. [Phạm vi và bằng chứng](../../docs/validation/2026-10-07-3dm-hatch-loop-rows.md).

CurveOnSurface giữ nguyên nguồn trong FCStd. 30 profile không có C3 đã qua Rhino5 thực tế và giữ đúng trường native khi nhập ngược; những loại con chưa kiểm chứng vẫn bị chặn. Bản sửa SDK chỉ thay hàm Read trong file build sinh ra, nguồn SDK gốc vẫn nguyên vẹn. Mất đối tượng có C3 là tương thích reader Rhino5 đã quan sát, không phải kết luận rằng định dạng V5 không chứa được C3.

### Rhino5 bounds follow-up

10/10 mở/lưu/mở lại; nhập ngược33/33. Kiểm chứng hình học bổ sung50/50 ở ngưỡng0,001 mm; bbox Rhino cũ49/50 được giữ riêng. [Báo cáo và giới hạn phép đo](../../docs/validation/2026-10-07-rhino5-ring-bounds.md). Full openNURBS vẫn đang hoàn thiện.

### Kiểm toán full openNURBS — 2026-10-07

Danh mục chuẩn hóa có131 khai báo nguồn,128 lớp runtime đã đối chiếu trong FreeCAD; phân loại abstract/helper/obsolete/concrete riêng. Năm dòng comment và ba lớp obsolete không được SDK build đã được ghi rõ. Báo cáo cũ128/123 giữ nguyên làm lịch sử, không còn là inventory đầy đủ. Không dùng số lớp hay số assertions để tính phần trăm hoàn thành. Regression hiện tại42/42 native; Rhino5 GUI35/35 profile, nhập ngược API/GUI qua484 kiểm tra FreeCAD. Bốn mẫu shell cong qua46 kiểm tra FreeCAD và29 kiểm tra nhập ngược file Rhino đã lưu; phép đo Rhino chính xác8/8 giữ ngưỡng1e-6 mm³, báo cáo mặc định1/4 giữ riêng; `Shape.Volume` mặc định còn sai số tích phân được ghi riêng, kiểm chứng dùng tích phân thích nghi với ngưỡng thể tích giữ nguyên. [Capability từng trục](../../docs/3dm-capabilities.json) · [Capability từng thuộc tính](../../docs/3dm-capability-slices.json) · [Audit hiện tại](../../docs/3dm-current-support-audit.json) · [Phạm vi đã kiểm chứng và phần còn thiếu](../../docs/validation/2026-10-07-opennurbs-packages-1-3.md).


Xem [phạm vi nhánh mới và cách kiểm tra](../../docs/public-update-2026-10-08.md).

The project follows its existing **LGPL-2.1-or-later** declaration; the license
text is in [LICENSE](../../LICENSE). Separate dependency notices remain applicable.

### Surface advanced và History — 2026-10-09

Refit, Sweep2 G1/G2, Add Slash và Loft tangent matching đã có native geometry trong [giới hạn host](../../docs/features/surface-advanced-options.md). Bật History trong options để sửa rail/profile và tự dựng lại mặt; CMD `gvSweepHistory` / `gvSweep2History` tạo variant002/004. Native type giữ dependency qua Undo/Redo/FCStd, kể cả mở file trong tiến trình mới. **530/530 kiểm tra native**, 164 Rust và23 Python đã qua; [bằng chứng](../../docs/validation/2026-10-09-surface-constraints-history.md).

### Nguồn Rhino 5 cho core

[Hướng dẫn và bảng tra Rhino 5](../../ref/rhino5/README.md) ánh xạ tài liệu gốc,
Command Help và các bước kiểm chứng sang 12 chương
[00-rhino-core](../../OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/README.md).
Đọc nguồn theo phần chức năng trước khi triển khai và chạy fixture trước khi
xác nhận tương đương. Đây là ngoại lệ cho tài liệu Rhino 5 được người dùng
cho phép; code decompile, tài nguyên và tài liệu Matrix vẫn ở kho riêng.

<!-- gumball-core-20261011:start -->
OM9-INFO-020/021/022: `partially_implemented` / `validated_supported_slice`.
Native 13 handles, Move/Rotate/Scale/Copy/planar Extrude, numeric/modifiers,
CPlane/Object/World, Relocate and single-owner view-state cache implemented.
Actual Qt/DPI, batch callback rollback and FCStd/retained 3DM cold proof;
unsupported owners/frames and shared-host failed-transaction Redo loss remain explicit.
[Evidence, reproducible suite and limits](../../docs/validation/2026-10-11-gumball-core.md).
Final fresh review: three Important findings fixed by TDD; native432/16 and Rust808/115 passed.
<!-- gumball-core-20261011:end -->

<!-- gumball-four-view-spec-20261011:start -->
2026-10-11: đã triển khai Gumball đồng bộ bốn viewport: Command On/Off,
first-click handles, shared preview/batch, bounds Scale, plane hai màu và menu
ring; core settings/Snappy/Strength/orbit và native Box/Sphere/Link property Scale.
Tại baseline ngày 2026-10-11, ba ID còn `partially_implemented`: generic
owners, Mesh edge/vertex/light/camera frames và host old-Redo rollback còn mở.
[Cách dùng Gumball](../../docs/gumball-user-guide.md) · [Native evidence](../../docs/validation/2026-10-11-gumball-four-view-completion.md).
<!-- gumball-four-view-spec-20261011:end -->

<!-- gumball-core-completion-20261012:start -->
2026-10-12: OM9-INFO-020/021/022 `implemented` / `validated_supported_slice`;
`approved_scope_complete=true` for the approved OM9 core scope. Exact30 native
processes / 955 checks passed against one plugin/host.
Approved OM9 core scope completed against the exact30 native suite: native primitive dimensions/expressions, standard FreeCAD Link/Scale with native OM9 Compound-derived helpers preserving admitted definitions/dependencies/subelements/Body trees and presentation across Undo/Redo/FCStd, bounded native input settling before fresh gestures after Undo with unchanged history and callback cancellation, actual Mesh Ctrl+Shift component picking, typed document Camera/Light physical frames, shared four-view Relocate and process-local cache. Native host limits and unsupported graph/frame cases remain explicit. Clayoo/SmartTargets, Rhino internal roll, full original-host parity, full Render and live host camera widgets are excluded. Check the exact candidate commit's GitHub Actions run for its stock-host validation result.
[Final native evidence](../../docs/validation/2026-10-12-gumball-core-completion.md).
<!-- gumball-core-completion-20261012:end -->
