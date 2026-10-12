# Implementation status — 2026-10-10

<!-- box-default-video:start -->
2026-10-12, người dùng làm rõ video minh họa **Box mặc định**, không phải một trong bốn option. Box mở ngay ở First corner; mặc định hai góc đáy đối diện rồi Height (3 lần chọn). Diagonal/3Point/Vertical/Center là lựa chọn tùy chọn. 3Point riêng vẫn là hai đầu cạnh rồi Width/Height. Quy tắc này thay thế yêu cầu chọn phương pháp bắt buộc trước đó.

[Bằng chứng mặc định và preview theo video](../docs/validation/2026-10-12-box-default-video.md).
<!-- box-default-video:end -->


<!-- box-documented-workflow-20261012:start -->
2026-10-12, sửa theo yêu cầu người dùng: Box bắt buộc chọn phương pháp; bỏ nhánh
Box thường/Corners. 3Point và Vertical nhận hai điểm đầu cạnh rồi Width/Height,
không nhận chiều dài thay điểm cuối. Center nhận tâm rồi góc hoặc Length toàn phần.
`0` chỉ là điểm gốc sau khi chọn phương pháp; Width/Height vẫn có Enter default.
Điều này thay thế các ghi chú Corners/default và Length ở endpoint bên dưới.
[Bằng chứng Rust/native](../docs/validation/2026-10-12-box-documented-workflow.md).
<!-- box-documented-workflow-20261012:end -->

<!-- common-command-mnemonics-20261012:start -->
2026-10-12: quy tắc gạch chân một ký tự áp dụng qua engine Rust chung cho toàn
bộ schema Command Options hiện có: Line/Polyline, InterpCrv, Rectangle, Circle,
Ellipse, Rebuild, Box/Sphere, Sweep1/History, Sweep2/History và Loft. Tùy chọn con
khai báo alias trong phase cha; tên đầy đủ vẫn click được. Alias trùng hoặc
disabled không được quảng cáo bằng gạch chân. Lệnh mới đăng ký schema để dùng
cùng resolver, renderer và binding CMD–Builder. Trạng thái toàn feature còn partial.
[Bằng chứng native](../docs/validation/2026-10-12-common-command-mnemonics.md).
<!-- common-command-mnemonics-20261012:end -->

<!-- command-mnemonics-20261012:start -->
2026-10-12: chỉ gạch chân một ký tự phím tắt đã khai báo, không gạch chân cả
tên. Rust cung cấp mnemonic; cả tên vẫn click được, clipboard giữ nguyên.
Box dùng D/3/V/C; phím3 chỉ chọn3Point trước điểm đầu, sau đó vẫn nhập số3.
Focus,0 origin và lịch sử dễ đọc đã sửa vẫn giữ. Supported status còn partial.
[Bằng chứng native](../docs/validation/2026-10-12-command-mnemonics.md).
<!-- command-mnemonics-20261012:end -->

<!-- box-command-usability-20261012:start -->
2026-10-12: Box icon trái/phải focus CMD; `0` ở bước nhập điểm là gốc CPlane
và chuyển vào lịch sử khi được nhận. Width/Height giải thích Enter/default mm.
Lỗi ghi riêng vào lịch sử, prompt sạch và draft sai vẫn sửa được; giới hạn
geometry giữ nguyên. Focus có guard generation sau cancel; common Curve input
evidence giữ điểm gốc. Đây là sửa supported slice, status vẫn partial.
[Bằng chứng native và giới hạn](../docs/validation/2026-10-12-box-command-usability.md).
<!-- box-command-usability-20261012:end -->

<!-- matrix-phase-options-20261012:start -->
2026-10-12: engine Rust chung có question/options theo phase, span UTF-16,
ẩn/hiện độc lập gate và revision refresh. Box áp dụng first-corner → Diagonal
Cube → Width/Height defaults; click và D/d dùng cùng native solver. CMD xanh ghi
invocation thật, preceding prompt và input đã nhận; Up/Down recall, Esc và draft
lỗi giữ đúng lifecycle. Đây là supported slice, không hoàn tất mọi command,
Gem Builder hay associative History.
[API dùng chung](../docs/command-options-adapter.md) · [Bằng chứng native](../docs/validation/2026-10-12-box-matrix-command-options.md).
<!-- matrix-phase-options-20261012:end -->

<!-- common-command-options-20261012:start -->
2026-10-12: Command Line Options dùng schema/reducer Rust chung và binder Qt
cho các consumer Curve, Rebuild, Sweep/Loft hiện có; lệnh mới đăng ký schema
và adapter theo exact feature ID. Giữ RCORE-04/Builder Framework ở phạm vi
`partially_implemented`; không mở Gem solver hoặc command chưa hỗ trợ.
[API dùng chung](../docs/command-options-adapter.md) ·
[Bằng chứng native và giới hạn](../docs/validation/2026-10-12-command-line-options.md).
<!-- common-command-options-20261012:end -->


<!-- gumball-four-view-spec-20261011:start -->
2026-10-11: đã triển khai Gumball đồng bộ bốn viewport: Command On/Off,
first-click handles, shared preview/batch, bounds Scale, plane hai màu và menu
ring; core settings/Snappy/Strength/orbit và native Box/Sphere/Link property Scale.
Giữ ba ID `partially_implemented`: generic owners, Mesh edge/vertex/light/camera
frames và host old-Redo rollback còn mở; chưa claim full original-host parity.
[Cách dùng Gumball](../docs/gumball-user-guide.md) · [Native evidence](../docs/validation/2026-10-11-gumball-four-view-completion.md).
<!-- gumball-four-view-spec-20261011:end -->



<!-- rhino-core-notification-lock-20261012:start -->
Đã sửa startup resource-deadlock tại FreeCAD NotificationArea: dựng nội dung dưới mutex rồi thả khóa trước khi mở popup; cảnh báo Qt có thể vào logger bình thường. Popup thật vẫn giữ thông báo và warning, không tắt thông báo để né lỗi.
**App273/plugin275/GUI287/kernel240:3375/3375 kiểm tra qua35 host sạch log, tất cả exit0.** Fixture popup RED2/3→GREEN3/3, stack native xác nhận nguyên nhân. Giữ các lỗi startup274/278/284 và các crash cũ làm evidence lịch sử. Rust toàn bộ lần gần nhất1037/0, không chạy lại cho thay đổi GUI native này.
**RCORE-01 vẫn đang triển khai:** còn các phần chưa có evidence đầy đủ về conversion, consumer/result, ownership/preservation, aggregate bounds và lifecycle/cache; T01-T11 đã có accepted slices, không mở lại toàn bộ chỉ vì checkbox lịch sử.
[Bằng chứng notification lock](../docs/validation/2026-10-12-rhino-core-notification-lock.md).
<!-- rhino-core-notification-lock-20261012:end -->

<!-- rhino-core-snapshot-history-disposal-20261012:start -->
FreeCAD replay kiểm tổng history/property records và dung lượng snapshot/tên trước khi sao chép; dùng lại bảng transaction descriptions với borrowed view đồng bộ. Hủy Undo/Redo dùng FlagToggler, chặn reentrant mutation và pop record trước destructor; nhánh visible Redo của failed preserving group cũng được bảo vệ.
**App273/plugin275/kernel240:3372/3372 kiểm tra hành vi qua34 host;33/34 host sạch log.** Một host close-object có startup Unhandled std::exception trước fixture, nên runtime gate chưa đạt toàn bộ. Snapshot38, normal-close finalizer16 và empty-group Redo5 đạt; các lượt diagnostic278/280 không cộng vào tổng. Rust toàn bộ lần gần nhất1037/0, không chạy lại cho thay đổi native này.
**RCORE-01 vẫn đang triển khai:** startup deadlock, peak/opaque allocation, lifecycle và phần còn thiếu của hợp đồng gốc; các checklist lịch sử cần đối chiếu theo evidence, không tự coi mọi fixture đã đạt là chưa làm.
[Bằng chứng snapshot/history](../docs/validation/2026-10-12-rhino-core-snapshot-history-disposal.md).
<!-- rhino-core-snapshot-history-disposal-20261012:end -->

<!-- rhino-core-acceptance-offset-d3-20261012:start -->
Theo thiết kế người dùng duyệt, identity/Redo được giữ khi từ chối trước acceptance; destructor chạy sau acceptance dưới barrier và báo disposal diagnostic riêng. Circle History giữ publication hợp lệ khi logger ném lỗi.
**App247/Runtime254/kernel240:3313/3313 kiểm tra qua28 host**; boundary162, logger24. OCCT D3 helper/direct curve/adaptor RED4/12→GREEN12/12; đủ19 họ/183 query,36 D3 được sửa và45 offset khớp đạo hàm độc lập. **Rust toàn bộ1037 đạt/0 lỗi.**
**RCORE-01/Task2/3 vẫn đang triển khai:** aggregate budgets, global participants, lifecycle và toàn bộ hợp đồng gốc còn mở. Witness destructor-veto cũ giữ nguyên, đã được thay tiêu chí bởi thiết kế duyệt; các checkpoint dưới là lịch sử.
[Bằng chứng acceptance/D3](../docs/validation/2026-10-12-rhino-core-acceptance-offset-d3.md).
<!-- rhino-core-acceptance-offset-d3-20261012:end -->

<!-- rhino-core-native-retirement-verdict-20261012:start -->
App227/Runtime229 dùng lại transaction, native Property và Python exception của FreeCAD. Finalizer chạy trước verdict của commit/cleanup; callback bị chặn không còn báo thành công. Giữ inverse nếu retirement trước đó đã bị từ chối.
**3145/3145 assertion qua28 host** trên runtime tương ứng, gồm Python identity/private inverse72/201, finalizer6, frozen source180, native journal54, verdict50, cleanup14 và inverse retention12.
**RCORE-01/Task2/3 vẫn đang triển khai.** Test gốc Python final release 5/8 và detached object 16/32 còn RED; Rust121 đạt/1 lỗi OffsetBezier D3. Các checkpoint phía dưới là bằng chứng lịch sử.
[Bằng chứng retirement verdict](../docs/validation/2026-10-12-rhino-core-native-retirement-verdict.md).
<!-- rhino-core-native-retirement-verdict-20261012:end -->

<!-- rhino-core-native-value-snapshots-20261012:start -->
App191/Runtime193 giữ identity native cho Link/XLink, subname/shadow/partial flags và cây biểu thức/validator; dùng lại setters/Paste của FreeCAD. Frozen journal giữ thao tác rename và refresh các entry mới/canceled. Prepared phase và cleanup guard chặn callback trong merge unwind; commit retry bị từ chối trước null journal.
Đã kiểm chứng **2607/2607 assertion qua21 host**, gồm Link128, XLink112, expression60, rename32, merge-unwind32, retry12, native journal51 và hồi quy/cold reload.
**Còn RED:** detached-object disposal16/32 và Rust121pass/1fail (raw native offset D3). Python property identity/orphan Save, frozen-source preparation và toàn bộ **Task2/3/full RCORE-01 còn mở**.
[Bằng chứng native value snapshot](../docs/validation/2026-10-12-rhino-core-native-value-snapshots.md).
<!-- rhino-core-native-value-snapshots-20261012:end -->

<!-- rhino-core-native-persistence-inverse-20261012:start -->
App154 giữ inverse bằng buffer native Save/Restore và seal merge source; mọi private Property được giải phóng/kiểm tra trước caller transfer.
Final recovery disposal **204/204**, original registered retirement **204/204**;32 callback/case mỗi cohort. Native inverse/source retirement/seal/budget/dynamic tests **40/40**.
Runtime156/App154 đạt **1988 kiểm tra/13 host sạch**, gồm semantic432,close4,modes109,Curve33,booking80,scope744 và cold/legacy/spatial.
**Discarded regular Redo vẫn RED12/28**:4 callback bị chặn và không báo accepted, nhưng original booking/Redo và paired cleanup chưa phục hồi. **Task2/3/full RCORE-01 còn mở.**
[Bằng chứng native persistence inverse](../docs/validation/2026-10-12-rhino-core-native-persistence-inverse.md).
<!-- rhino-core-native-persistence-inverse-20261012:end -->

<!-- rhino-core-native-retirement-20261012:start -->
App147 có native factory/Paste inverse; original Copy retirement được kiểm tra trước caller merge.
Original retirement **204/204**, registered native subtype **204/204**;32 callback/case mỗi cohort, phục hồi và giữ caller history.
Matching Runtime149/App147: **1982 kiểm tra/13 host sạch**, gồm semantic432,close4,modes109,Curve33,booking80,capture34,scope744,cold modes62,legacy8/cold3,spatial65.
**Final recovery disposal vẫn RED128/204**:32 callback bị chặn và không báo accepted, nhưng rollback/paired cleanup chưa hoàn tất. **Task2/3/full RCORE-01 còn mở.**
[Bằng chứng native retirement](../docs/validation/2026-10-12-rhino-core-native-retirement.md).
<!-- rhino-core-native-retirement-20261012:end -->

<!-- rhino-core-native-capture-booking-20261012:start -->
Native journal capture đã chuẩn bị metadata/copy trước khi ghi entry; **34/34** kiểm tra, gồm first/later/removal Copy failure và retry.
Regular booking không khôi phục Redo đã loại bỏ sau accepted replay/abort; **80/80** so sánh hành vi FreeCAD native ở0/1,plain/booked/active/preserving.
Runtime138/App141 đạt lại **432 semantic +4 owner-close +109 modes +33 Curve**; tổng **692 assertion/6 host0**;5 host sạch, capture34 có initial macro deadlock.
String scope744 vẫn có deadlock lúc xử lý macro khởi động, **không phải bằng chứng lỗi shutdown**. Retirement callbacks, global participants/budgets và full RCORE-01/Task2/3 còn mở.
[Bằng chứng native capture/booking](../docs/validation/2026-10-12-rhino-core-native-capture-booking.md).
<!-- rhino-core-native-capture-booking-20261012:end -->

<!-- rhino-core-circle-native-savepoint-20261012:start -->
Circle History recipe-v2/legacy đã dùng native savepoint trước khi đọc input, qua rollback và paired cleanup hoặc verified commit.
Runtime131/App125 đạt **432/432 semantic**, **4/4 owner-close**, **109/109 Circle modes**, **33/33 Curve result**; 4 host thoát0 và không có unhandled shutdown/deadlock.
App DLL/import được kiểm tra hash và chọn riêng; dùng SDK/cache hiện có.
**RCORE-01/Task2/3 vẫn mở**: capture Copy failure, retirement callbacks, exact booking/Redo policy, native String-scope shutdown failure và các yêu cầu gốc còn lại.
[Bằng chứng Circle native savepoint](../docs/validation/2026-10-12-rhino-core-circle-native-savepoint.md).
<!-- rhino-core-circle-native-savepoint-20261012:end -->

<!-- rhino-core-native-replay-scope-20261012:start -->
Document scope prototype có private journal/observer latch,owner/history guards,
checked rollback/error cleanup và staged merge. UndoMode Python stub được nối mode0/1 thật.
Named **744/744 assertions**,104 case;Curve33/Circle109 đạt trên App125.
**GUI shutdown vẫn có unhandled std::exception/resource deadlock: chưa clean runtime gate.**
Circle chưa tích hợp scope;atomic **RED355/432**,48 callback;SDK DLL/lib/PDB giữ nguyên.
**Savepoint/Task2/3 và toàn bộ RCORE-01 còn mở.**
[Bằng chứng native replay scope](../docs/validation/2026-10-12-rhino-core-native-replay-scope.md).
<!-- rhino-core-native-replay-scope-20261012:end -->

<!-- rhino-core-native-property-merge-20261012:start -->
Native Transaction có staged property merge: giữ caller record/first-write/ownership,
chuẩn bị copy và kiểm stale trước transfer. Native API RED0/1→GREEN29/29;
App117 build0;Curve33+Circle109 = **171/171 final positive**,3 host exits0.
SDK DLL/lib/PDB giữ nguyên;không rebuild root hoặc clone runtime.
History atomic vẫn **RED355/432**,48 callback. **Document scope/booking/rollback/lifetime,
Task2/3 và toàn bộ RCORE-01 còn mở.** Không chạy lại Rust ở checkpoint native này.
[Bằng chứng native property merge](../docs/validation/2026-10-12-rhino-core-native-property-merge.md).
<!-- rhino-core-native-property-merge-20261012:end -->

<!-- rhino-core-initial-curve-owned-contacts-20261012:start -->
Initial Tangent: giữ tiếp điểm solver native tách khỏi probe; Rust kiểm owned D0/D1,
circle/plane/tangent/FromFirst/fixed null và output analytic/Deformable trước publication.
Focused7/7;full Rust **989 đạt/1 lỗi D3 cũ**. Runtime112 build0,native **522/522**,
11 exits0;contact sources khớp,full current-source matching false do test wrapper và sửa song song.
Native savepoint đã được người dùng duyệt;baseline mới vẫn **RED355/432**,48 callback.
**RCORE-01/Task2/3 và mọi yêu cầu gốc còn mở.**
[Bằng chứng initial owned contacts](../docs/validation/2026-10-12-rhino-core-initial-curve-owned-contacts.md).
<!-- rhino-core-initial-curve-owned-contacts-20261012:end -->

<!-- rhino-core-initial-curve-owned-parameters-20261012:start -->
Initial AroundCurve/Tangent: native fraction được giữ; Rust kiểm owned D0/unitD1,
điểm/hướng, selected edge và path trên branch Undo. RED1/3→GREEN5/5;
183 query/19 family admission;full Rust **982 đạt/1 lỗi D3 cũ**.
Runtime109 build0,native **417/417**,8 host exits0;1498 inputs match.
**RCORE-01/Task2/3 vẫn mở: initial solved contacts, generic snap,
atomic/lifetime và mọi yêu cầu gốc còn thiếu.**
[Bằng chứng initial owned parameters](../docs/validation/2026-10-12-rhino-core-initial-curve-owned-parameters.md).
<!-- rhino-core-initial-curve-owned-parameters-20261012:end -->

<!-- rhino-core-initial-curve-owned-selection-20261012:start -->
Initial Curve: dùng chung Rust owned selection với History trước khi đổi session;
CAD FitPoints và native Vertex/Direction/fixed-point được ràng buộc nguồn.
Giữ subset indices cloud/mesh; tách local basis1e150 và world command1e9.
Rust RED1/4→GREEN5/5;full **977 đạt/1 lỗi D3 cũ**. Native local/world RED3/4→4/4.
Runtime106 build0,native **360/360**,6 host exits0;1496 inputs match.
**RCORE-01/Task2/3 vẫn mở: initial path/Tangent, generic edge/face snap,
atomic/lifetime và mọi yêu cầu gốc còn thiếu.**
[Bằng chứng initial owned selection](../docs/validation/2026-10-12-rhino-core-initial-curve-owned-selection.md).
<!-- rhino-core-initial-curve-owned-selection-20261012:end -->

<!-- rhino-core-circle-history-owned-fit-geometry-20261012:start -->
Circle History: Rust kiểm selected geometry theo surface poles→curve poles→native vertices;
hỗ trợ FaceN, whole/prefix, rational/periodic NURBS/Bezier, cloud/mesh và world once.
Native Face RED62/70→GREEN70/70; cold10 family50/50. Rust focused7/7,
Curve result **56/56**, full **972 đạt/1 lỗi D3 cũ**. Runtime102 build0;
native **426/426**,6 host exits0;production inputs match,hai test files đổi sau build.
**RCORE-01/Task2/3 vẫn mở: initial native input/Tangent,full families,
atomic/lifetime,conversion/maps/budgets và mọi yêu cầu gốc còn thiếu.**
[Bằng chứng selected geometry](../docs/validation/2026-10-12-rhino-core-circle-history-owned-fit-geometry.md).
<!-- rhino-core-circle-history-owned-fit-geometry-20261012:end -->

<!-- rhino-core-circle-history-owned-role-replay-20261012:start -->
Circle History: Rust dựng lại điểm/cấu hình replay từ record và nguồn đã chọn;
giữ whole FitPoints tăng điểm, prefix và numeric gaps; chặn sai Direction/radius/spans.
Role RED0/6→GREEN7/7; nhóm Curve result **50/50**. Full Rust **965 đạt/1 lỗi D3 cũ**.
Runtime97 build0, History/cold/spatial **236/236**,3 host exits0;1486 inputs match.
**RCORE-01/Task2/3 vẫn mở: selected CAD poles/vertices/Face provenance,
initial Tangent,atomic/lifetime và toàn bộ yêu cầu gốc còn thiếu.**
[Bằng chứng stored role replay](../docs/validation/2026-10-12-rhino-core-circle-history-owned-role-replay.md).
<!-- rhino-core-circle-history-owned-role-replay-20261012:end -->

<!-- rhino-core-circle-history-owned-contact-binding-20261012:start -->
Circle History giữ fraction tiếp điểm native vàRust kiểm D0/D1 độc lập:
circle/plane/tangent/FromFirst;probe vàcontact vẫn riêng. Contact **5/5**,source23/23,
19 family/183 query admission. Full Rust **958 đạt/1 lỗi D3 native còn nguyên**.
Runtime93 build0,native History/cold/nonplanar **236/236**,3 host exits0.
**RCORE-01 vàTask2/3 vẫn mở;role khác,initial Tangent,atomic/lifetime vàyêu cầu gốc còn thiếu.**
[Bằng chứng owned contact binding](../docs/validation/2026-10-12-rhino-core-circle-history-owned-contact-binding.md).
<!-- rhino-core-circle-history-owned-contact-binding-20261012:end -->

<!-- rhino-core-circle-history-owned-source-binding-20261012:start -->
Circle History đã nối owned Rust D0/D1 vào path/probe/Point source binding.
Ba lỗi nguồn cũ đã sửa;source **23/23**,19 family/183 query qua edge/world và admission.
Full Rust **953 đạt/2 lỗi**:D3 native cũ và ca mới thiếu kiểm tiếp điểm Tangent.
Runtime91 build0,native History/cold **171/171**,2 host exits0.
**RCORE-01 và Task2/3 vẫn đang triển khai;contact/atomic/lifetime và yêu cầu gốc còn mở.**
[Bằng chứng owned source binding](../docs/validation/2026-10-12-rhino-core-circle-history-owned-source-binding.md).
<!-- rhino-core-circle-history-owned-source-binding-20261012:end -->

<!-- rhino-core-owned-derived-curve-evaluation-20261012:start -->
Owned Rust Taylor/trim/offset:183 query D0–D2 native khớp;45 D3 offset
khớp oracle toán70 chữ số,nhưng36 khác D3 host;giữ raw native D3 RED.
Focused10/11,full Rust **943 đạt/4 lỗi**. Runtime86 build0,native capture/History/cold
**229/229**,3 host exits0. Chưa nối source admission;mọi yêu cầu gốc còn mở.
**RCORE-01 và Task2/3 vẫn đang triển khai.**
[Bằng chứng derived evaluator](../docs/validation/2026-10-12-rhino-core-owned-derived-curve-evaluation.md).
<!-- rhino-core-owned-derived-curve-evaluation-20261012:end -->

<!-- rhino-core-owned-curve-evaluation-20261012:start -->
Owned Rust curve evaluator: 138 điểm/đạo hàm OCCT trên14 family khớp;
6/6 focused, full Rust **939 đạt/3 lỗi nguồn còn nguyên**. Runtime79 build0;
oracle43 + History109 + cold62 = **214/214**,3 host exits0. Chưa nối evaluator
vào source admission; derived support/contact/maps/budgets/lifetime còn mở.
**RCORE-01 và Task2/3 vẫn đang triển khai.**
[Bằng chứng evaluator](../docs/validation/2026-10-12-rhino-core-owned-curve-evaluation.md).
<!-- rhino-core-owned-curve-evaluation-20261012:end -->

<!-- rhino-core-circle-history-source-role-fixes-20261012:start -->
Circle History: Rust đã chặn role/path thiếu hoặc trùng, source Point giả vai curve,
span Point/Curve chồng nhau và fixed point ngoài circle/plane; giữ native tolerance.
Probe AroundCurve cũ và anchor replay được ghi riêng. Native **171/171**,2 exits0;
Rust toàn bộ **933 đạt/3 lỗi**, nhóm source **17/20**. **RCORE-01/Task2/3 vẫn mở.**
[Bằng chứng source role fixes](../docs/validation/2026-10-12-rhino-core-circle-history-source-role-fixes.md).
<!-- rhino-core-circle-history-source-role-fixes-20261012:end -->

<!-- rhino-core-circle-history-source-binding-red-20261012:start -->
Circle History: Rust nguồn/parameter/constraint **RED4/10**,exit101;
ba payload dựng lại từ snapshot native thật đạt,nhưng sáu ca sai fraction/role/
path duplicate/điểm ngoài curve/đổi nguồn đầy đủ/fixed point ngoài circle vẫn được
nhận. Point oracle đầu bị chặn bởi document token khác đã sửa đúng;không bịa stamp.
Chỉ thêm test/data;production chưa đổi. **RCORE-01 vàTask2/3 vẫn mở.**
[Bằng chứng source binding](../docs/validation/2026-10-12-rhino-core-circle-history-source-binding-red.md).
<!-- rhino-core-circle-history-source-binding-red-20261012:end -->

<!-- rhino-core-circle-history-lifetime-red-20261012:start -->
Circle History: callback đóng được document sở hữu trong replay; prefix
`after_close`/owner absent đã lưu. Recompute không trả về trong300s; runner kết
thúc đúng host thử nghiệm vàxác nhận terminal. **Đây là witness lỗi lifetime,
không phải suite đạt hay mã crash đã biết.** Có thiết kế native savepoint đề xuất;
production chưa đổi. **RCORE-01, Task2/3 vàmọi yêu cầu gốc vẫn mở.**
[Bằng chứng lifetime vàthiết kế](../docs/validation/2026-10-12-rhino-core-circle-history-lifetime-red.md).
<!-- rhino-core-circle-history-lifetime-red-20261012:end -->

<!-- rhino-core-circle-history-atomic-state-20261012:start -->
Circle History: đo **trước Redo**,48 ca/ba trạng thái caller,
**RED355/432**, host exit0 vàkhông lỗi fixture. Metadata sai42/48; nguồn vertex
bị sửa3/48;16 booking chưa materialize phát sinh journal/pending mới. Redo gốc
thực thi48/48 sau caller abort. BREP khác do identity location45/48, không phải
nguồn đổi hình học. **RCORE-01 vàTask2/3 vẫn mở; chưa sửa production.**
[Bằng chứng rejection-time](../docs/validation/2026-10-12-rhino-core-circle-history-atomic-state.md).
<!-- rhino-core-circle-history-atomic-state-20261012:end -->

<!-- rhino-core-circle-history-modes-20261012:start -->
Circle History, cùng Runtime59: **182/182 native bổ sung**, bốn host exit0.
All-mode109, cold62 với20 result, legacy8 vàlegacy cold3 đạt; không sửa solver
hoặc rebuild. **Atomic rejection RED50/80**: phát hiện16 callback vàxóa cặp
Shape/result, nhưng14 ca giữ metadata bị sửa; kiểm source/Undo stack còn lỗi.
Redo gốc vẫn thực thi16/16. **RCORE-01, Task2/3 vàmọi yêu cầu gốc còn mở.**
[Bằng chứng và lỗi tiếp theo](../docs/validation/2026-10-12-rhino-core-circle-history-modes.md).
<!-- rhino-core-circle-history-modes-20261012:end -->

<!-- rhino-core-circle-history-result-20261012:start -->
Circle History Runtime59: **430/430 native**,14 actual host exits0;
full Rust916/0/0,plugin build0. Dùng lại solver;result/current input
snapshots được ghép với Shape. Callback33,Box Vertex/Mesh/Update=No/units8,
Undo/Redo24 vàcold FCStd12 đạt. Giữ RED58 vàexcluded missing-helper launch.
**RCORE-01 đang triển khai;Task2/3,mọi C01–C12/T01–T11 vàgiới hạn gốc còn mở.**
[Bằng chứng vàgiới hạn](../docs/validation/2026-10-12-rhino-core-circle-history-result.md).
<!-- rhino-core-circle-history-result-20261012:end -->

<!-- rhino-core-circle-history-result-red-20261012:start -->
Circle History Runtime57: **RED12/36**,actual host exit0,không fixture exception.
Center/AroundCurve/FitPoints dùng lại solver và cập nhật Shape đúng,nhưng result
và input snapshots còn cũ;invalid source chưa xóa result. Freeze/detach/FCStd
chưa có cặp Shape/result hiện hành. Giữ riêng checkpoint focus1544/1544.
**RCORE-01 đang triển khai;Task2/3 và mọi yêu cầu gốc giữ nguyên.**
[Bằng chứng và bước sửa](../docs/validation/2026-10-12-rhino-core-circle-history-result-red.md).
<!-- rhino-core-circle-history-result-red-20261012:end -->

<!-- rhino-core-curve-focus-result-20261012:start -->
Ellipse FromFoci/MarkFoci dùng lại solver, có common GeometryResult cho hai focus.
Runtime56 RED6/30 → Runtime57 focus30/30,cold12/12,focus callbacks130/130.
**1544/1544 native**,27 actual exits0;6 cold hosts/29 FCStd;build0;Rust885 và lượt sau900.
Curve hashes khớp;full current-source matching false do Gumball đổi song song;
inventory được chụp sau khi build bắt đầu,không phải compiler-input certificate.
**RCORE-01 đang triển khai;Task2/3 và mọi yêu cầu gốc giữ nguyên.**
Auxiliary History/Circle Rebuild,full lifetime/sharing/budgets/conversion/maps và chuột/render còn mở.
[Bằng chứng và phần còn mở](../docs/validation/2026-10-12-rhino-core-curve-focus-result.md).
<!-- rhino-core-curve-focus-result-20261012:end -->

*Checkpoint Runtime56 bên dưới là lịch sử; xem Runtime57 ở trên.*
<!-- rhino-core-curve-booking-20261012:start -->
Curve native booking Runtime56: **1372/1372 native**,24 actual exits0;Rust882/0/0;build0.
Commit/abort/reopen callback120/120 giữ đúng journal, rollback/document và prior native Redo;
selected1416 inputs match,SDK unchanged;5 cold hosts/27 FCStd.
**RCORE-01 đang triển khai;Task2/3 và mọi yêu cầu gốc giữ nguyên.**
Foci common result/History/Rebuild, full lifetime/sharing/budgets/conversion/maps và chuột/render còn mở.
[Bằng chứng và phần còn mở](../docs/validation/2026-10-12-rhino-core-curve-booking.md).
<!-- rhino-core-curve-booking-20261012:end -->

*Checkpoint Runtime55 bên dưới là lịch sử; native booking đã có Runtime56 ở trên.*
<!-- rhino-core-curve-publication-20261011:start -->
Checked Curve publication/cancellation: Runtime55 **1252/1252 native checks**,23 actual exits0;
Rust882/0/0; standalone build0;1415 selected inputs match;SDK unchanged.
Output210/source60/layer210/replacement60/hidden-layer50 plus662 regressions pass.
Uses existing solvers and host Redo-preserving transaction; prior native Redo and replacement Box retained.
**RCORE-01 đang triển khai;Task2/3 giữ nguyên.** Transaction booking/premature close,
foci common result/History/Rebuild, sharing/budgets/conversion/maps và chuột/render còn mở.
[Bằng chứng và phần còn mở](../docs/validation/2026-10-11-rhino-core-curve-publication.md).
<!-- rhino-core-curve-publication-20261011:end -->

*Checkpoint RED52 dưới đây là lịch sử; xem Runtime55 ở trên.*
<!-- rhino-core-curve-callback-red-20261011:start -->
Checked Curve publication:Runtime52 real callback witness **RED42/210**,
actual host exit0,không fixture exception. Sáu lệnh vẫn commit output bị đổi
Shape/frame/construction/result/units/parent;42 callbacks đều chạy,mỗi ca mất prior Redo.
Main result662/662 và Rust880 giữ riêng theo phạm vi;dùng lại host transaction giữ Redo.
**RCORE-01 đang triển khai;Task2/3 và mọi yêu cầu gốc giữ nguyên.**
[Bằng chứng và bước sửa](../docs/validation/2026-10-11-rhino-core-curve-callback-red.md).
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
[Bằng chứng và phần còn mở](../docs/validation/2026-10-11-rhino-core-curve-result.md).
<!-- rhino-core-curve-result-20261011:end -->

<!-- rhino-core-curve-result-red-20261011:start -->
*Checkpoint RED lịch sử trước common result Runtime52; xem checkpoint mới phía trên.*
Task3 common Curve GeometryResult: native witness Runtime50 **RED12/54**,
actual host exit0,không có fixture exception. Sáu lệnh có geometry/one Undo đúng,
nhưng thiếu OM9GeometryResult/input snapshots/output CAD master và paired persistence.
Cohort bridge517/517 giữ riêng;dùng lại solver để bổ sung result và checked publication.
**RCORE-01 đang triển khai;Task2/3 và mọi yêu cầu gốc giữ nguyên.**
[Bằng chứng và bước tiếp](../docs/validation/2026-10-11-rhino-core-curve-result-red.md).
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
[Bằng chứng và phần còn mở](../docs/validation/2026-10-11-rhino-core-curve-native-bridge.md).
<!-- rhino-core-curve-native-bridge-20261011:end -->

<!-- rhino-core-curve-native-admission-20261011:start -->
*Checkpoint Rust-only trước khi nối native; xem checkpoint mới phía trên.*

Rust đã có API nhận nguồn native Curve sở hữu dữ liệu, giữ original text/source
và branch Undo; solver hiện hữu được dùng lại. Năm lỗi review: RED8/13→GREEN13/13;
units/context thêm:16/16 và FFI1/1. Full Rust871/0/0, actual exit0;238 inputs khớp.
**C++ chưa nối API mới; chưa build/chạy native. Runtime45 vẫn RED9/18 về nguồn.**
**RCORE-01 đang triển khai; Task2/3 và mọi yêu cầu gốc giữ nguyên.**
[Bằng chứng và bước tiếp](../docs/validation/2026-10-11-rhino-core-curve-native-admission.md).
<!-- rhino-core-curve-native-admission-20261011:end -->

<!-- rhino-core-curve-native-source-20261011:start -->
Native witness mới trên Runtime45: **RED9/18**, actual host exit0.
Circle/Ellipse AroundCurve và FitPoints sau Undo đạt hình học/Undo/FCStd,
nhưng thiếu original native text, identity/revision/frame/ranges và full source snapshots.
Giữ riêng cohort accepted-input466/466; đây là phần Task2 cần làm tiếp.
Gumball/lib.rs sửa song song; full current-source matching vẫn chưa xác nhận.
**RCORE-01 đang triển khai; toàn bộ yêu cầu gốc vẫn cần hoàn tất.**
[Bằng chứng và bước tiếp](../docs/validation/2026-10-11-rhino-core-curve-native-source.md).
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
[Bằng chứng và phần còn mở](../docs/validation/2026-10-11-rhino-core-curve-accepted-input.md).
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
[Bằng chứng và phần còn mở](../docs/validation/2026-10-11-rhino-core-curve-construction.md).
<!-- rhino-core-curve-construction-20261011:end -->

<!-- rhino-core-result-output-budget-20261011:start -->
*Checkpoint Runtime42 trước Curve construction; giữ bằng chứng lịch sử theo phạm vi.*
GeometryResult chung kiểm node/depth trước serialize; giữ đầy đủ geometry.
Focused Rust RED1/4 → GREEN4/4; full Rust833/0/0, standalone build0.
Runtime42 offscreen:892/892 checks,18 actual exits0; Solid553 và5 consumer fixtures;
2 cold hosts/6 FCStd,437 producer inputs không đổi. Shared SDK giữ nguyên.
Aggregate allocation, Curve result, conversion/maps và chuột/render còn mở.
**RCORE-01 đang triển khai; toàn bộ yêu cầu gốc vẫn cần hoàn tất.**
[Bằng chứng và phần còn mở](../docs/validation/2026-10-11-rhino-core-result-output-budget.md).
<!-- rhino-core-result-output-budget-20261011:end -->

<!-- rhino-core-solid-source-budget-20261011:start -->
*Checkpoint Runtime41 trước output node/depth gate; giữ bằng chứng lịch sử theo phạm vi.*
Solid chặn giới hạn source JSON trước nhận FitPoints; dùng lại solver FreeCAD.
Ca nguồn khác nhau: Runtime40 RED3/5 → Runtime41 GREEN5/5, giữ đủ nguồn/range.
Matching Runtime41:553/553 checks,13 actual exits0,2 cold hosts/6 FCStd;
Rust829/0/0, standalone retry build0,437 producer inputs không đổi.
Gate source cần thiết đã kiểm; full request/output budget và chuột/render còn mở.
**RCORE-01 đang triển khai; toàn bộ yêu cầu gốc vẫn cần hoàn tất.**
[Bằng chứng và phần còn mở](../docs/validation/2026-10-11-rhino-core-solid-source-budget.md).
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
[Bằng chứng và phần còn mở](../docs/validation/2026-10-11-rhino-core-solid-shared-snapshot.md).
<!-- rhino-core-solid-shared-snapshot-20261011:end -->

<!-- rhino-core-solid-result-20261011:start -->
*Checkpoint Runtime38 trước native offscreen và sửa snapshot sharing; trạng thái chưa kiểm chứng bên dưới là lịch sử.*
Solid đã nối vào GeometryResult chung; FitPoints giữ nguồn/range và có giới hạn
tổng dữ liệu, kiểm nội dung native cùng root frame. Rust825/0/0 gồm9 kiểm tra mới;
standalone Runtime38 build exit0. **Native mới chưa kiểm chứng**: năm lượt startup
timeout bị loại, kể cả profile đồ họa riêng; các host thử nghiệm đã kết thúc. Runtime35/32 giữ riêng.
**RCORE-01 đang triển khai; mọi yêu cầu gốc/conversion/maps vẫn còn mở.**
[Bằng chứng và phần còn mở](../docs/validation/2026-10-11-rhino-core-solid-result.md).
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
[Bằng chứng và phần còn mở](../docs/validation/2026-10-11-rhino-core-solid-construction.md).
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
[Bằng chứng và phần còn mở](../docs/validation/2026-10-11-rhino-core-solid-publication.md).
<!-- rhino-core-solid-publication-20261011:end -->

<!-- rhino-core-surface-preview-20261011:start -->
*Checkpoint Surface/camera trước đợt Solid; giữ bằng chứng lịch sử theo phạm vi.*
Surface báo lỗi dọn preview/root lease ở4 viewport, giữ nguồn/Undo và cho tạo
lại Preview.48 ca đạt468/468; camera khi đóng document RED15/18 → GREEN18/18.
GUI mới:486 checks/2 actual exits0, không còn escaped camera exception.
Hồi quy plugin trước đó:649 checks/23 exits0, gồm5 cold hosts; chưa chạy lại
trên DLL GUI mới. Rust808 dùng lại với218 source hashes không đổi.
**RCORE-01 đang triển khai; toàn bộ yêu cầu gốc giữ nguyên.**
[Bằng chứng và phần còn mở](../docs/validation/2026-10-11-rhino-core-surface-preview.md).
<!-- rhino-core-surface-preview-20261011:end -->

<!-- rhino-core-commit-publication-20261011:start -->
*Checkpoint Runtime24 trước sửa cleanup; xem checkpoint mới phía trên.*
Builder Create/Restore/Match giải phóng Python result khi commit lỗi, báo
KernelFailure kèm thao tác và nguồn; Sweep1/Sweep2/Loft plain/History giữ đủ tên
nguồn và chặn native exception thoát qua Qt. RED57/69 và60/72 → GREEN69/69 và72/72.
Matching Runtime24/API19: 649 checks/23 actual host exits0, gồm5 cold hosts;
full Rust808 đạt. Launcher GUI riêng dùng lại SDK; shared App vẫn API12.
**RCORE-01 đang triển khai; toàn bộ yêu cầu gốc giữ nguyên.**
[Bằng chứng và phần còn mở](../docs/validation/2026-10-11-rhino-core-commit-publication.md).
<!-- rhino-core-commit-publication-20261011:end -->

<!-- rhino-core-isolated-publication-20261011:start -->
*Checkpoint native trước khi có matching GUI riêng; xem kiểm chứng caller mới phía trên.*
DLL native thử nghiệm riêng API19: global28/28, root46/46;
App602 đạt/2 bỏ qua/0 lỗi trong604 ca,7 disabled. Đã kiểm lỗi lifecycle,
activation, chuyển thuộc tính lồng và verdict cùng/khác ID bằng RED → GREEN.
SDK dùng chung vẫn API12; matching OM9 GUI/plugin chưa kiểm chứng.
**RCORE-01 đang triển khai; toàn bộ yêu cầu gốc giữ nguyên.**
[Bằng chứng và phần còn mở](../docs/validation/2026-10-11-rhino-core-isolated-publication.md).
<!-- rhino-core-isolated-publication-20261011:end -->

<!-- rhino-core-global-close-20261011:start -->
*Checkpoint lịch sử API12 và candidate API13; xem DLL thử nghiệm đã kiểm phía trên.*
API12: global native boundary8/8, root46/46; App602 đạt/2 bỏ qua/0 lỗi trong604 ca.
Hai cohort RED mới đều0/4: tham gia muộn, commit rỗng mất Undo và đóng nhóm lồng.
Candidate API13 đã biên dịch object; DLL/runtime chưa kiểm chứng vì shared host
demo Gumball còn mở. Không nâng checkpoint Runtime21/API11 thành bằng chứng mới.
**RCORE-01 đang triển khai; toàn bộ yêu cầu gốc giữ nguyên.**
[Bằng chứng và phần còn mở](../docs/validation/2026-10-11-rhino-core-global-close.md).
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
[Bằng chứng và lỗi còn mở](../docs/validation/2026-10-11-rhino-core-publication-booking.md).
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
[Bằng chứng và giới hạn](../docs/validation/2026-10-11-rhino-core-atomic-redo.md).
<!-- rhino-core-atomic-redo-20261011:end -->

<!-- rhino-core-checked-rollback-20261011:start -->
*Checkpoint trước API9/Runtime18; trạng thái API/callback bên dưới là bằng chứng lịch sử. Xem checkpoint mới phía trên.*
Native `Transaction::applyChecked` phát hiện rollback bị từ chối hoặc chưa khôi phục đủ;
giữ hành vi `apply` mặc định và ownership native. Chín ca checked rollback đạt.
App executable: 569 đạt, 2 bỏ qua, 5 lỗi giữ Redo còn mở; exit1, không gọi cả suite đạt.
API giữ Redo, gate nhiều document và matching host/plugin vẫn chưa hoàn tất.
**RCORE-01 đang triển khai; toàn bộ yêu cầu gốc được giữ nguyên.**
[Bằng chứng và phần còn mở](../docs/validation/2026-10-11-rhino-core-checked-rollback.md).
<!-- rhino-core-checked-rollback-20261011:end -->

<!-- rhino-core-edit-result-20261011:start -->
*Checkpoint trước API9/Runtime18; trạng thái API/callback bên dưới là bằng chứng lịch sử. Xem checkpoint mới phía trên.*
Rust admission và native CAD Edit GeometryResult đã triển khai trên solver sẵn có.
Runtime14/15 đã đạt các hồi quy CAD/lệnh/guard và cold reload; callback abort còn thất bại 2/5 kiểm tra Redo.
Runtime15: Dimension NaN/Infinity 10/10, callback font/đồ họa và lỗi đúng Explode/source 12/12; dùng lại kiểm finite Rust.
Full Rust 808 đạt; bằng chứng native giữ riêng theo plugin, không suy thành nghiệm thu toàn chương.
**RCORE-01 đang triển khai**, không đóng consumer/lifecycle hay yêu cầu gốc.
[Bằng chứng, lỗi đang mở và bước tiếp](../docs/validation/2026-10-11-rhino-core-edit-result.md).
<!-- rhino-core-edit-result-20261011:end -->

<!-- gumball-core-20261011:start -->
OM9-INFO-020/021/022: `partially_implemented` / `validated_supported_slice`.
Native 13 handles, Move/Rotate/Scale/Copy/planar Extrude, numeric/modifiers,
CPlane/Object/World, Relocate and single-owner view-state cache implemented.
Actual Qt/DPI, batch callback rollback and FCStd/retained 3DM cold proof;
unsupported owners/frames and shared-host failed-transaction Redo loss remain explicit.
[Evidence, reproducible suite and limits](../docs/validation/2026-10-11-gumball-core.md).
Final fresh review: three Important findings fixed by TDD; native432/16 and Rust808/115 passed.
<!-- gumball-core-20261011:end -->


<!-- rhino-core-surface-result-20261011:start -->
Loft/Sweep giữ solver hiện có, có GeometryResult EditSurface với recipe,
source snapshots/attributes/revision và CAD master khớp Shape được công bố.
History cập nhật/xóa cả Shape và result; freeze/detach, Undo/Redo và FCStd đạt.
Sửa và kiểm Placement/group cùng tên output duy nhất trước publication.
Matching848/21 actual host exits0; full Rust791 đạt.
Global conversion/maps, các consumer và lifecycle toàn family còn mở;
**RCORE-01 đang triển khai**. [Bằng chứng và giới hạn](../docs/validation/2026-10-11-rhino-core-surface-result.md).
<!-- rhino-core-surface-result-20261011:end -->

<!-- rhino-core-surface-history-admission-20261011:start -->
Surface History dùng Rust kiểm JSON/options/group metadata trước native capture;
giữ solver Loft/Sweep và kiểm rail closure từ nguồn hiện tại. Dữ liệu hỏng giữ
raw/source/links/Redo, xóa output cũ và báo InvalidBasis kèm tên child.
GREEN55; matching698/13 actual host exits0; full Rust767 đạt.
Full results/conversion/maps/ownership mọi family còn mở; **RCORE-01 đang triển khai**.
[Bằng chứng và giới hạn](../docs/validation/2026-10-11-rhino-core-surface-history-admission.md).
<!-- rhino-core-surface-history-admission-20261011:end -->

<!-- rhino-core-builder-conversion-20261011:start -->
Builder giữ native ModifiedShape topology map, template snapshot độc lập và
điểm đo/parameter queries cho edge và face có trim; Rust kiểm frame/domain/ID
và sai lệch trước publish. Query native không thực hiện được ghi unavailable
và diagnostic; không nâng tolerance hoặc xác nhận global parameter map.
315 checks/10 actual host exits0 trên cùng plugin, full Rust758 đạt.
Global conversion/maps và hợp đồng mọi family còn mở; **RCORE-01 đang triển khai**.
[Bằng chứng và giới hạn](../docs/validation/2026-10-11-rhino-core-builder-conversion.md).
<!-- rhino-core-builder-conversion-20261011:end -->

<!-- layer-management-audit-20261011:start -->
LAYERS / **OM9-LAYER-001** (2026-10-11): `partially_implemented` /
`validated_supported_slice`. Panel current/Assign, lock/Region/edit, Hide/Show,
group visibility, Advanced và Lights đã nghiệm thu. Footer bỏ theo người dùng.
Rust **11/11**; native **panel 71/71, outputs 21/21, permission 5/5, modal 5/5, circle 51/51, mouse 51/51**. Plugin-only build/shared SDK;
Art/material và general hierarchy còn ngoài slice. [Report](../docs/validation/2026-10-11-layer-panel-implementation.md).
<!-- layer-management-audit-20261011:end -->

<!-- rhino-core-builder-result-20261011:start -->
Builder có GeometryResult native đúng operation, recipe/slot/source revision,
CAD master/topology/validation và thuộc tính seed độc lập, giữ qua History/FCStd.
Create/restore/Match chuẩn bị cả Shape/result trước transaction; metadata hỏng
bị từ chối, recipe legacy ghi unavailable. 297 checks/9 actual host exits0
trên cùng plugin; full Rust747 đạt. Conversion/maps, thuộc tính còn lại và
hợp đồng mọi family còn mở; **RCORE-01 vẫn đang triển khai**.
[Bằng chứng và giới hạn](../docs/validation/2026-10-11-rhino-core-builder-result.md).
<!-- rhino-core-builder-result-20261011:end -->

<!-- rhino-core-builder-output-publication-20261011:start -->
Builder Create/restore/Match dựng và kiểm toàn bộ output/slot trước transaction;
lỗi output cuối giữ source, links và executable Redo. Dùng lại solver/native
frame và policy Rust; dữ liệu dựng sẵn chỉ sống trong lần publish này.
28 output checks; matching196 checks/7 actual host exits0, gồm Builder cold
FCStd và prepared CAD; full Rust744 đạt. Results/conversion/maps/attributes,
aggregate budgets và hợp đồng toàn family còn mở; **RCORE-01 đang triển khai**.
[Bằng chứng và giới hạn](../docs/validation/2026-10-11-rhino-core-builder-output.md).
<!-- rhino-core-builder-output-publication-20261011:end -->

<!-- startup-ui-cleanup-20261011:start -->
Startup UI (2026-10-11): bỏ hàng toolbar Import/Export 3DM thừa; Task đóng khi
khởi động, menu chữ và các lệnh Import/Export vẫn truy cập được. Launcher FreeCAD
1.1.4 đã cập nhật; sáu kiểm tra trên profile mới đạt, host thoát 0.
[Bằng chứng](../docs/validation/2026-10-11-startup-ui-cleanup.md).
<!-- startup-ui-cleanup-20261011:end -->

<!-- rhino-core-builder-policy-20261011:start -->
Builder affine-template hiện có dùng request Rust độc lập, kiểm source/template,
recipe và kích thước trước copy/transaction; lỗi đầu vào giữ Redo.
61 policy checks; matching1484 checks/32 actual host exits0, gồm Builder và
Circle cold FCStd; full Rust744 đạt. Giữ solver/frame/History hiện hữu.
Results/maps, source-lock-cache và yêu cầu gốc còn mở; **RCORE-01 đang triển khai**.
[Bằng chứng và giới hạn](../docs/validation/2026-10-11-rhino-core-builder-policy.md).
<!-- rhino-core-builder-policy-20261011:end -->

<!-- rhino-core-circle-history-policy-20261011:start -->
Circle History dùng request Rust từ tùy chọn đã lưu, kiểm trước copy/reader;
giữ độc lập với lệnh đang nhập, legacy AroundCurve và Mesh/cloud FitPoints.
1363 checks/27 actual host exits0 trên cùng plugin, gồm cold FCStd; full
Rust739 đạt. Output results/maps, consumer khác và yêu cầu gốc còn mở;
**RCORE-01 vẫn đang triển khai**. [Bằng chứng và giới hạn](../docs/validation/2026-10-11-rhino-core-circle-history-policy.md).
<!-- rhino-core-circle-history-policy-20261011:end -->

<!-- rhino-core-cage-borrowed-admission-20261011:start -->
Basis Cage và saved binding được Rust kiểm trên dữ liệu mượn trước copy/input
buffer; native selection và recompute có lỗi typed kèm operation/object. Giữ
solver, imported basis, Undo/Redo và Release liên kết hỏng. 1322 checks/24
actual host exits0 trên cùng plugin; full Rust734 đạt. Results/maps, History,
native allocation khác và yêu cầu gốc còn mở; **RCORE-01 đang triển khai**.
[Bằng chứng và giới hạn](../docs/validation/2026-10-11-rhino-core-cage-borrowed-admission.md).
<!-- rhino-core-cage-borrowed-admission-20261011:end -->

<!-- rhino-core-native-cage-policy:start -->
Cage hiện có dùng policy Rust chung cho declaration/capture/commit và request
độc lập cho API/recompute; Part/Mesh được kiểm trước copy/Topology extraction.
Giữ Create bằng bounds, Restore archive và Release binding hiện có. 1268 checks/
23 actual host exits0 trên cùng plugin; full Rust730 đạt. Rational-control/
binding allocation, results/maps, các consumer khác và yêu cầu gốc còn mở;
**RCORE-01 đang triển khai**. [Bằng chứng và giới hạn](../docs/validation/2026-10-11-rhino-core-native-cage-policy.md).
<!-- rhino-core-native-cage-policy:end -->

<!-- rhino-core-native-solid-policy:start -->
Box/Sphere hiện có dùng request/options Rust chung trước declaration,
Preview, commit và capture Sphere; kiểm output scalar giữ kích thước có dấu,
CPlane và solver hiện hữu. 1071 checks/18 actual host exits0 trên cùng plugin;
full Rust721 đạt. Các hợp đồng kết quả/ownership gốc và cage/replay còn mở;
**RCORE-01 đang triển khai**. [Bằng chứng và giới hạn](../docs/validation/2026-10-11-rhino-core-native-solid-policy.md).
<!-- rhino-core-native-solid-policy:end -->

<!-- rhino-core-native-curve-policy:start -->
Sáu Curve family hiện có dùng request/options Rust chung từ session ở
declaration/preview/commit và các capture Circle/Ellipse/FitPoints được nêu trong báo cáo.
680 checks/15 actual host exits0 trên cùng plugin; full Rust712 đạt.
Box/Sphere/cage, replay request và các hợp đồng output/ownership gốc còn mở;
**RCORE-01 đang triển khai**. [Bằng chứng và giới hạn](../docs/validation/2026-10-11-rhino-core-native-curve-policy.md).
<!-- rhino-core-native-curve-policy:end -->

<!-- rhino-core-native-rebuild-policy:start -->
Rebuild hiện có dùng policy operation/type/options Rust chung trước enable,
copy và solve; kiểm nội dung Compound và giữ solver/source/placement/Undo/Redo.
383 checks/nine actual host exits0 trên cùng plugin; full Rust702 đạt.
Các family construction và hợp đồng output/ownership gốc còn mở;
**RCORE-01 đang triển khai**. [Bằng chứng và giới hạn](../docs/validation/2026-10-11-rhino-core-native-rebuild-policy.md).
<!-- rhino-core-native-rebuild-policy:end -->

<!-- rhino-core-fit-point-input:start -->
Circle/Sphere và Circle History dùng reader điểm native có giới hạn Rust trước
copy, giữ placement/source/Undo/Redo và ngân sách typed+selected. GREEN45;
589 checks trên cùng plugin,7 actual host exits0;full Rust695 đạt.
Các policy construction/results và yêu cầu gốc còn mở; **RCORE-01 đang triển khai**.
[Bằng chứng và giới hạn](../docs/validation/2026-10-11-rhino-core-fit-point-input.md).
<!-- rhino-core-fit-point-input:end -->

<!-- rhino-core-native-surface-diagnostics:start -->
Lỗi policy Surface đã ghi đúng operation và object/subelement nguồn;
RED5/6 → GREEN6/6. Bản plugin sau sửa đạt112 checks,4 actual host exits0;
637 checks ở checkpoint tích hợp trước dùng plugin cũ. **RCORE-01 còn đang triển khai.**
[Bằng chứng và giới hạn](../docs/validation/2026-10-11-rhino-core-native-surface-diagnostics.md).
<!-- rhino-core-native-surface-diagnostics:end -->

<!-- rhino-core-native-surface-policy:start -->
RCORE-01 đã nối policy Surface Rust chung vào declaration/capture/construction
của Sweep1/Sweep2/Loft hiện có; 637 checks trên cùng plugin, 9 actual host exits0.
Full Rust 691 đạt với source fingerprint không đổi. Các family khác, kết quả/
conversion và yêu cầu gốc còn mở; **toàn chương vẫn đang triển khai**.
[Bằng chứng và giới hạn](../docs/validation/2026-10-11-rhino-core-native-surface-policy.md).
<!-- rhino-core-native-surface-policy:end -->

<!-- rhino-core-native-construction-input:start -->
RCORE-01 bổ sung gate Rust hiện có trước copy input Surface/Rebuild;
12/12 kiểm tra đầu vào đạt, giữ source/Undo/Redo và selected Edge2 hợp lệ.
608 checks, 8 matching actual exits0 đã được ghi; các hồi quy còn lại
và shared construction policy/results/conversion vẫn đang triển khai.
**Toàn chương chưa hoàn tất.** [Bằng chứng và giới hạn](../docs/validation/2026-10-10-rhino-core-native-construction-input.md).
<!-- rhino-core-native-construction-input:end -->

<!-- rhino-core-native-edit-capabilities:start -->
RCORE-01 đã nối registry Rust chung vào Join/Explode/Trim/Boolean hiện có,
kiểm input native và block/cage leaves trước copy, kiểm lại options trước solve.
679 Rust;4456 host checks;36 matching actual exits0, dùng lại solver FreeCAD.
Full native Edit results/conversion/maps, các consumer khác và yêu cầu gốc còn
mở. **Chương vẫn đang triển khai.** [Bằng chứng và giới hạn](../docs/validation/2026-10-10-rhino-core-native-edit-capabilities.md).
<!-- rhino-core-native-edit-capabilities:end -->

<!-- rhino-core-prepared-discrete-results:start -->
RCORE-01 có GeometrySnapshot/result cho prepared mesh và PointCloud float64;
native/plain geometry sở hữu riêng, Rust kiểm schema/channel và đối chiếu mesh
archive theo triangle có hướng ở độ chính xác float32 trước transaction.
81 warm+8 cold+3701 hồi quy=3790 host checks;675 Rust;26 actual exits0.
Upstream conversion/maps/attributes, các consumer khác và yêu cầu gốc còn mở.
**Chương vẫn đang triển khai.** [Bằng chứng và giới hạn](../docs/validation/2026-10-10-rhino-core-prepared-discrete-results.md).
<!-- rhino-core-prepared-discrete-results:end -->

<!-- rhino-core-prepared-result-snapshots:start -->
RCORE-01 có đủ trường GeometrySnapshot cho prepared CAD, native master OCCT V3
khôi phục độc lập, typed topology/support và frozen binding attributes do Rust
đóng gói.107 snapshot/warm-cold+95 metadata+3499 hồi quy=3701 host checks;
665 full Rust tests;24 actual host exits0. Upstream conversion/parameter maps,
mọi family/consumer và yêu cầu gốc còn mở. **Chương vẫn đang triển khai.**
[Bằng chứng và giới hạn](../docs/validation/2026-10-10-rhino-core-prepared-result-snapshots.md).
<!-- rhino-core-prepared-result-snapshots:end -->

<!-- rhino-core-prepared-cad-results:start -->
RCORE-01 đưa validation/admission Rust vào commit CAD/previews; geometry và plain
metadata sở hữu riêng trước transaction. Metadata lỗi giữ Redo, diagnostic đúng
identity; reports native lưu cùng object/archive và giữ bytes qua cold FCStd.
154 mới +3434 hồi quy =3588 host checks;660 full Rust tests;22 host exits0.
Full output snapshots/upstream conversion, các family khác và yêu cầu gốc còn
mở. **Chương vẫn đang triển khai.** [Bằng chứng và giới hạn](../docs/validation/2026-10-10-rhino-core-prepared-cad-results.md).
<!-- rhino-core-prepared-cad-results:end -->

<!-- rhino-core-topology-validation:start -->
RCORE-01 bổ sung vertex-link/face-orientation do Rust sở hữu và reports kiểm
native validity/solid/cavity/hướng world bằng OCCT hiện có.220 mới +67 surface
+3096 hồi quy =3383 kiểm tra FreeCAD;655 full Rust tests,17 host exits0.
Reports inspection phân biệt provenance với validity; uniform conversion/commit
consumers và mọi yêu cầu gốc còn lại vẫn mở. **Chương vẫn đang triển khai.**
[Bằng chứng và giới hạn](../docs/validation/2026-10-10-rhino-core-topology-validation.md).
<!-- rhino-core-topology-validation:end -->

<!-- rhino-core-reference-contexts:start -->
RCORE-01 bổ sung reference qua group/link/array, tên/nhãn definition và alias
native có giới hạn Rust; giữ đúng identity/world frame bằng readers hiện có.
540 reference +2556 hồi quy =3096 kiểm tra FreeCAD;641 full Rust tests đạt,
15 host exits0. Configured traversal/ownership/cycles, attributes, certificates,
reports và các yêu cầu gốc vẫn mở. **Chương vẫn đang triển khai.**
[Bằng chứng và giới hạn](../docs/validation/2026-10-10-rhino-core-reference-contexts.md).
<!-- rhino-core-reference-contexts:end -->

<!-- rhino-core-instance-arrays:start -->
RCORE-01 bổ sung occurrence array do Rust sở hữu, giữ index/identity/member path
và world frame qua Part/group/mesh/cloud readers hiện có.609 array +10 boundary
+1937 hồi quy =2556 kiểm tra FreeCAD;635 full Rust tests đạt,14 host exits0.
Reference contexts/attributes, manifold/solid/reports và các yêu cầu gốc vẫn mở.
**Chương vẫn đang triển khai.** Các checkpoint dưới đây là bằng chứng lịch sử.
[Bằng chứng và giới hạn](../docs/validation/2026-10-10-rhino-core-instance-arrays.md).
<!-- rhino-core-instance-arrays:end -->

<!-- rhino-core-edge-geometry:start -->
RCORE-01 bổ sung bảng cạnh3D/vertex/tolerance do Rust sở hữu, nối với UV topology;
giữ standalone/loose geometry và world frame direct/group/link.921 mới +1016 hồi
quy =1937 kiểm tra FreeCAD;632 full Rust tests đạt,12 host exits0.
Flags/thử mẫu chưa chứng nhận correspondence toàn miền; manifold/solid/reports,
arrays/attributes và yêu cầu gốc vẫn mở. **Chương vẫn đang triển khai.**
Các checkpoint bên dưới là bằng chứng lịch sử.
[Bằng chứng và giới hạn](../docs/validation/2026-10-10-rhino-core-edge-geometry.md).
<!-- rhino-core-edge-geometry:end -->

<!-- rhino-core-discrete-snapshots:start -->
RCORE-01 bổ sung mesh/cloud snapshot do Rust sở hữu: raw-local points, triangle
indices/bounds, native owner checks và world frame cho direct/group/link.
99 mới +6 cold loading +911 hồi quy =1016 kiểm tra FreeCAD;628 full Rust tests đạt.
Channels/archive float64/subelements, arrays/reference contexts, attributes,
manifold/reports và các yêu cầu gốc vẫn mở. **Chương vẫn đang triển khai.**
Các checkpoint bên dưới là bằng chứng lịch sử.
[Bằng chứng và giới hạn](../docs/validation/2026-10-10-rhino-core-discrete-snapshots.md).
<!-- rhino-core-discrete-snapshots:end -->

<!-- rhino-core-instance-definitions:start -->
RCORE-01 có graph group/instance do Rust sở hữu, giữ ordered members, identity
của nested link và single/multiple Part subelements theo world frame native.
48 mới +863 hồi quy =911 kiểm tra FreeCAD;624 full Rust tests đạt.
Array/mesh/cloud definitions, reference contexts còn lại, attributes/userdata,
manifold/reports và các yêu cầu gốc vẫn mở. **Chương vẫn đang triển khai.**
Các checkpoint bên dưới là bằng chứng lịch sử.
[Bằng chứng và giới hạn](../docs/validation/2026-10-10-rhino-core-instance-definitions.md).
<!-- rhino-core-instance-definitions:end -->

<!-- rhino-core-world-attributes:start -->
RCORE-01 có ma trận support-local → world đã kiểm tra, thuộc tính nguồn native
do Rust sở hữu và descriptor tách instance/definition cho nested direct App::Link.
78 mới +778 hồi quy =856 kiểm tra FreeCAD;620 Rust tests đạt. Group/member/array
instance, thuộc tính/userdata còn lại, manifold/solid và reports vẫn mở.
**Chương vẫn đang triển khai.** Các checkpoint dưới đây là bằng chứng lịch sử.
[Bằng chứng và giới hạn](../docs/validation/2026-10-10-rhino-core-world-attributes.md).
<!-- rhino-core-world-attributes:end -->

<!-- rhino-core-derived-support:start -->
RCORE-01 có snapshot Rust-owned cho offset 3D/UV, offset surface, extrusion và
revolution; giữ chuỗi basis NURBS/Bezier/analytic và inner trim một/hai hướng.
91 derived +687 hồi quy =778 kiểm tra FreeCAD;610 Rust tests đạt. World/attributes,
manifold/solid, reports và yêu cầu còn lại vẫn mở. **Chương vẫn đang triển khai.**
Các checkpoint bên dưới là bằng chứng lịch sử.
[Bằng chứng và giới hạn](../docs/validation/2026-10-10-rhino-core-derived-support.md).
<!-- rhino-core-derived-support:end -->

<!-- rhino-core-bezier-support:start -->
RCORE-01 có snapshot Bezier do Rust sở hữu cho curve/surface và UV pcurve;
dùng lại adapter native, giữ control net/weight, natural domain và type Bezier.
76 Bezier +611 hồi quy =687 kiểm tra FreeCAD; 603 Rust tests đạt. Offset/support
khác, world/attributes, manifold/solid và yêu cầu còn lại vẫn mở.
**Chương vẫn đang triển khai.** Các checkpoint bên dưới là bằng chứng lịch sử.
[Bằng chứng và giới hạn](../docs/validation/2026-10-10-rhino-core-bezier-support.md).
<!-- rhino-core-bezier-support:end -->

<!-- rhino-core-analytic-support:start -->
RCORE-01 có snapshot analytic do Rust sở hữu cho 10 loại curve/surface và UV
pcurve; giữ tham số native, domain, location/orientation và phân biệt chiều UV
với frame XYZ. 174 analytic +437 hồi quy =611 kiểm tra FreeCAD; 598 Rust tests đạt.
Bezier/support khác, attributes/world composition, manifold/solid và các yêu
cầu còn lại vẫn mở. **Chương vẫn đang triển khai.** Checkpoint bên dưới là
bằng chứng lịch sử theo từng phạm vi và fingerprint, không phải trạng thái mới.
[Bằng chứng và giới hạn](../docs/validation/2026-10-10-rhino-core-analytic-support.md).
<!-- rhino-core-analytic-support:end -->

<!-- rhino-core-owned-support:start -->
RCORE-01 có bản sao NURBS do Rust sở hữu cho mặt/pcurve và curve 3D; giữ poles,
weights, knots, periodic axes, domain đầy đủ riêng với trim, orientation và
native location. 54 kiểm tra mới +383 hồi quy FreeCAD; 594 Rust tests đạt.
Analytic/Bezier và các support khác, attributes/world composition, manifold/solid
và các yêu cầu còn lại vẫn mở. **Chương vẫn đang triển khai.**
[Bằng chứng và giới hạn](../docs/validation/2026-10-10-rhino-core-owned-support.md).
<!-- rhino-core-owned-support:end -->

<!-- rhino-core-topology-snapshot:start -->
RCORE-01 có snapshot topology chung và kiểm tra Rust cho mặt/vòng biên/cạnh seam,
pcurve domain/UV endpoints, occurrence của mặt dùng chung và thành phần biên hở.
383 kiểm tra FreeCAD đạt trên cùng plugin. Đây là bước triển khai có giới hạn;
supporting geometry, world composition, chứng nhận manifold/solid và các yêu cầu
còn lại vẫn mở. **Chương vẫn đang triển khai.**
[Bằng chứng và giới hạn](../docs/validation/2026-10-10-rhino-core-topology-snapshot.md).
<!-- rhino-core-topology-snapshot:end -->

<!-- rhino-core-import-commit-gate:start -->
RCORE-01 đã sửa numeric preflight trước commit CAD/previews, chặn worker và
container Python tùy biến. Plugin cuối đạt 55 gate checks, 32 modeling và
19 preservation checks; property payload và Undo/Redo được đối chiếu thật.
**Chương vẫn đang triển khai** theo
[audit toàn bộ yêu cầu](../docs/reviews/2026-10-10-rcore01-closure-audit.md).
[Bằng chứng và giới hạn](../docs/validation/2026-10-10-rhino-core-import-commit-gate.md).
<!-- rhino-core-import-commit-gate:end -->

<!-- rhino-core-native-input-tree-followup:start -->
Đã thêm preflight cây NURBS cùng mesh/PointCloud native trước copy/chuyển đổi,
`Duplicate`, `BrepForm`, `IsValid` và `IsoSurface` được bao phủ. 92 native checks,
6 archive/block CTests, bộ Explode, 572 Rust tests và 21 FreeCAD host checks đạt.
Dùng lại adapter Mesh/PointCloud, Line/Polyline và `Part.makePolygon`. Đây là
phần giới hạn của RCORE-01.T07; decode trong `ONX_Model::Read`, allocator toàn
tiến trình, analytic còn lại, cage-edit và worker vẫn mở. **Toàn bộ P0 còn đang triển khai.**
[Bằng chứng và giới hạn](../docs/validation/2026-10-10-rhino-core-native-mesh-cloud.md).
<!-- rhino-core-native-input-tree-followup:end -->

## Đồng bộ tiến độ Spec nhóm 00 — 2026-10-10

RCORE-01–07/09–10 tiếp tục ở trạng thái triển khai một phần: chỉ các lát cắt có
bằng chứng riêng được xác nhận; các hợp đồng P0 còn lại vẫn mở. RCORE-08/11/12
(P1/P2) chưa được runtime nghiệm thu trong phạm vi P0 hiện tại.
[Bảng tiến độ theo chương và các giới hạn](specs/00-rhino-core/README.md).

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
| [OM9-LAYER-001](specs/01-core/om9-layer-001-layer-management.md) | Layer Management | Một phần / validated_supported_slice | Panel + supported output integration đã kiểm native; Art/material/general hierarchy ngoài slice. |
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

<!-- gumball-four-view-spec-20261011:start -->
2026-10-11: đã triển khai Gumball đồng bộ bốn viewport: Command On/Off,
first-click handles, shared preview/batch, bounds Scale, plane hai màu và menu
ring; core settings/Snappy/Strength/orbit và native Box/Sphere/Link property Scale.
Tại baseline ngày 2026-10-11, ba ID còn `partially_implemented`: generic
owners, Mesh edge/vertex/light/camera frames và host old-Redo rollback còn mở.
[Cách dùng Gumball](../docs/gumball-user-guide.md) · [Native evidence](../docs/validation/2026-10-11-gumball-four-view-completion.md).
<!-- gumball-four-view-spec-20261011:end -->

<!-- gumball-core-completion-20261012:start -->
2026-10-12: OM9-INFO-020/021/022 `implemented` / `validated_supported_slice`;
`approved_scope_complete=true` for the approved OM9 core scope. Exact30 native
processes / 955 checks passed against one plugin/host.
Approved OM9 core scope completed against the exact30 native suite: native primitive dimensions/expressions, standard FreeCAD Link/Scale with native OM9 Compound-derived helpers preserving admitted definitions/dependencies/subelements/Body trees and presentation across Undo/Redo/FCStd, bounded native input settling before fresh gestures after Undo with unchanged history and callback cancellation, actual Mesh Ctrl+Shift component picking, typed document Camera/Light physical frames, shared four-view Relocate and process-local cache. Native host limits and unsupported graph/frame cases remain explicit. Clayoo/SmartTargets, Rhino internal roll, full original-host parity, full Render and live host camera widgets are excluded. Local stock suite:30 processes/955 checks. GitHub Actions only builds/packages per the user request on 2026-10-12; it makes no runtime-test claim. See [policy](../docs/validation/2026-10-12-local-tests-ci-build-only.md).
[Final native evidence](../docs/validation/2026-10-12-gumball-core-completion.md).
<!-- gumball-core-completion-20261012:end -->

2026-10-12 multi-selection fix0.3.5: Shift/window whole objects share one Gumball, including Object Alignment;143 new regression checks and31 native processes/1099 checks,848 Rust pass locally. Component guards, common-pivot transforms and one Undo retained. [Evidence](../docs/validation/2026-10-12-gumball-multi-selection.md). CI build/package only.
