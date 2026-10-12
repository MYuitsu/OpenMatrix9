---
id: RCORE-04
title: Command engine và tương tác xuyên lệnh
priority: P0
evidence_level: source_inspected
runtime_validation: not_run
evidence_scope: initial_source_audit_2026-10-09
date: 2026-10-09
---

# RCORE-04 — Command engine và vòng đời tương tác

<!-- common-command-mnemonics-20261012:start -->
2026-10-12: quy tắc gạch chân một ký tự áp dụng qua engine Rust chung cho toàn
bộ schema Command Options hiện có: Line/Polyline, InterpCrv, Rectangle, Circle,
Ellipse, Rebuild, Box/Sphere, Sweep1/History, Sweep2/History và Loft. Tùy chọn con
khai báo alias trong phase cha; tên đầy đủ vẫn click được. Alias trùng hoặc
disabled không được quảng cáo bằng gạch chân. Lệnh mới đăng ký schema để dùng
cùng resolver, renderer và binding CMD–Builder. Trạng thái toàn feature còn partial.
[Bằng chứng native](../../../docs/validation/2026-10-12-common-command-mnemonics.md).
<!-- common-command-mnemonics-20261012:end -->

<!-- command-mnemonics-20261012:start -->
2026-10-12: chỉ gạch chân một ký tự phím tắt đã khai báo, không gạch chân cả
tên. Rust cung cấp mnemonic; cả tên vẫn click được, clipboard giữ nguyên.
Box dùng D/3/V/C; phím3 chỉ chọn3Point trước điểm đầu, sau đó vẫn nhập số3.
Focus,0 origin và lịch sử dễ đọc đã sửa vẫn giữ. Supported status còn partial.
[Bằng chứng native](../../../docs/validation/2026-10-12-command-mnemonics.md).
<!-- command-mnemonics-20261012:end -->

<!-- box-command-usability-20261012:start -->
2026-10-12: Box icon trái/phải focus CMD; `0` ở bước nhập điểm là gốc CPlane
và chuyển vào lịch sử khi được nhận. Width/Height giải thích Enter/default mm.
Lỗi ghi riêng vào lịch sử, prompt sạch và draft sai vẫn sửa được; giới hạn
geometry giữ nguyên. Focus có guard generation sau cancel; common Curve input
evidence giữ điểm gốc. Đây là sửa supported slice, status vẫn partial.
[Bằng chứng native và giới hạn](../../../docs/validation/2026-10-12-box-command-usability.md).
<!-- box-command-usability-20261012:end -->

<!-- matrix-phase-options-20261012:start -->
2026-10-12: engine Rust chung có question/options theo phase, span UTF-16,
ẩn/hiện độc lập gate và revision refresh. Box áp dụng first-corner → Diagonal
Cube → Width/Height defaults; click và D/d dùng cùng native solver. CMD xanh ghi
invocation thật, preceding prompt và input đã nhận; Up/Down recall, Esc và draft
lỗi giữ đúng lifecycle. Đây là supported slice, không hoàn tất mọi command,
Gem Builder hay associative History.
[API dùng chung](../../../docs/command-options-adapter.md) · [Bằng chứng native](../../../docs/validation/2026-10-12-box-matrix-command-options.md).
<!-- matrix-phase-options-20261012:end -->

<!-- rhino-core-delayed-publication:start -->
## Bổ sung đã kiểm chứng — Rebuild result publication — 2026-10-10

Native Rebuild giữ kết quả qua lượt giao GUI và kiểm lại source/session/options
trước Coin/transaction. Cancel, đóng document, đổi option, request mới hơn và
shutdown không publish kết quả cũ; 173 fixture checks và 72 hồi quy đạt.
Solver hiện vẫn chạy đồng bộ; queue chỉ kiểm chứng ranh giới publish này, không
chứng nhận worker/thread-safety hoặc mọi command family.
[Bằng chứng và giới hạn](../../../docs/validation/2026-10-10-rhino-core-delayed-publication.md).
**RCORE-04 và toàn bộ P0 vẫn đang triển khai.**
<!-- rhino-core-delayed-publication:end -->

<!-- rhino-core-extended-input:start -->
## Bổ sung đã kiểm chứng — extended numeric input

Bổ sung nhập polar/spherical/surveyor, fraction/feet-inch và góc DMS bằng Rust,
dùng lại bộ dựng hình FreeCAD/OM9 hiện có. Sửa mốc tương đối và Undo của Circle/Sphere.
**421 Rust tests; 1108 native checks / 21 runs; 49 Python pass,1 skip.**
P0 toàn bộ còn mở; [phạm vi và bằng chứng](../../../docs/validation/2026-10-09-rhino-core-extended-input.md).

- [x] Grammar/routing và retry của supported slice đã kiểm chứng.
- [ ] Live constraints, locale và toàn bộ yêu cầu xuyên family còn mở.

<!-- rhino-core-extended-input:end -->

<!-- rhino-core-point-consumers:start -->
## Bổ sung đã kiểm chứng — shared point consumers

Distance, Angle và PictureFrame đã dùng chung parser Rust, giữ tọa độ/đơn vị đầu vào và provenance.
CPlane sai scale/shear bị từ chối; PictureFrame kiểm giao dịch trước mutation.
**324 native checks / 9 runs; 364 Rust tests; 49 Python pass, 1 skip.**
P0 toàn bộ vẫn đang triển khai; [phạm vi và bằng chứng](../../../docs/validation/2026-10-09-rhino-core-point-consumers.md).

Đây là slice Cartesian và lifecycle của ba consumer. Các hợp đồng grammar, session, view identity và History còn lại vẫn mở; không đóng toàn bộ fixture xuyên mọi command.

<!-- rhino-core-point-consumers:end -->

<!-- rhino-core-object-foundations:start -->
## Bổ sung nền tảng object snapshot — 2026-10-09

Edit, Rebuild và Surface tương tác giữ snapshot tại thời điểm nhận input và
kiểm lại trước transaction. Snapshot gồm session/generation/revision/epoch,
unit context, world placement và các placement owner có giới hạn độ sâu.
Source stale yêu cầu Cancel/chọn lại; chỉnh option không tự làm mới token.
Fixture kiểm cả commit thật và không có output/Undo dư khi bị từ chối.
Chưa thống nhất CommandRequest/CommandSession, view/CPlane revision, worker
cancellation hoặc late-result rejection cho mọi family.

[Bằng chứng và giới hạn](../../../docs/validation/2026-10-09-rhino-core-object-foundations.md); P0 vẫn `partially_implemented`.
Các phần cập nhật cũ và bảng audit bên dưới giữ bằng chứng lịch sử, không phải tổng kết checkpoint này.
<!-- rhino-core-object-foundations:end -->

## Cập nhật triển khai P0 — 2026-10-09

Các nhận định nguồn/metadata bên dưới thuộc baseline audit ban đầu. Đợt triển
khai mới tách repeat candidate khỏi transcript và dùng whitelist command thành
công; failed/cancelled/view/History operations không thay candidate. Native
repeat kiểm lại availability trước dispatch, không ghi success trước commit.
Curve session có guard khi unit context đổi; malformed CPlane được xử lý ở các
native consumer đã nối, hủy trạng thái tạm và không tạo geometry.

Rust regression và các native suites theo slice được theo dõi trong
[báo cáo P0](../../../docs/validation/2026-10-09-rhino-core-p0.md); native snap/repeat
đã qua 10 checks và Line/Polyline qua 27 checks. Không đóng toàn bộ T01–T13 từ repeat
whitelist hoặc một macro thành công.

**P0 còn thiếu:** `CommandRequest/CommandSession` chung cho mọi family/entrypoint;
document generation, source/view/CPlane revision snapshot thống nhất; cancellation
và late worker commit rejection xuyên family; transparent commands, full macro
grammar, mnemonic collisions và repeat/Undo exclusions đầy đủ theo phase.

## Phạm vi và nguồn

Hợp đồng dùng chung cho `OM9-CMD`, `OM9-IFACE-001`, `OM9-MAIN-001`,
`OM9-F6-001`, menu, sidebar, CMD, shortcut và command đang chờ điểm/selection.
Không tạo thêm feature ID; prompt/completion không được bật một capability chưa hỗ trợ.
Nguồn Rhino: User's Guide Rhino 5 PDF 11–23 / trang in 3–15,
đặc biệt PDF 21–22 về Undo, options, repeat và command window; xem [audit][audit].
PDF phân biệt prompt và lịch sử chữ; option có thể nhập tên, mnemonic hoặc click.
Enter/Space/right-click viewport lúc idle có thể repeat; Undo/Delete thuộc ví dụ
không repeat. Policy list này cần mapping rõ, không lấy toàn bộ history làm repeat list.

Baseline đọc source: FreeCAD HEAD `21d36cfa1eb110a1d0667050ff31706298805bbd`;
OM9 HEAD `52e887ab706dcd5d80e23979fb1f1b62d8d869b0`, ngày 2026-10-09.
Checkout có thay đổi người dùng. Đây là source inspection, không phải clean snapshot,
lần audit nguồn ban đầu không chạy FreeCAD/Rhino, build hoặc native acceptance; evidence triển khai mới tách riêng ở phần cập nhật.

## Input/output và capability gate

- Input chuẩn hóa thành `CommandRequest`: exact feature ID, alias gốc, entrypoint,
  document/view ID, selection snapshot, capability version và input event.
  Cùng ID và input hợp lệ qua menu/CMD/F6 phải vào cùng executor/session.
- Alias là tên giao diện, không là identity. Alias trùng Rebuild curve/surface,
  Dim ngang/dọc cần context hoặc lựa chọn explicit; không chọn command đầu catalog.
  Khi context chưa đủ, hiện các lựa chọn thực sự khả dụng và giữ input.
- Gate xét operation, options, representation, selection, edit/read-only state,
  backend và version; lý do disabled phải đọc được. Completion có thể liệt kê
  command disabled nhưng không được gọi nó thành công hoặc tự chọn fallback.
- Parser phase trả typed event: option, number, point, selection, accept, cancel,
  undo-step, navigation hoặc unsupported. Tên option không bị nuốt bởi parser số.
  Text editor, IME và ô tên giữ quyền nhập Space/chữ/phím tắt riêng.
- Output là effect hữu hạn: Waiting, PreviewChanged, SelectionRequested,
  CommitPlan, Completed, Cancelled, RecoverableError hoặc FatalError.
  “Đã dispatch” không đồng nghĩa “đã commit”; command chưa hoàn thành không vào
  successful execution history dùng cho repeat/icon history.

<!-- command-line-options-contract-20261012:start -->
<a id="command-line-options"></a>

## Command Line Options — hợp đồng bổ sung 2026-10-12

Bổ sung theo yêu cầu người dùng và ảnh tham chiếu Command Line Options: cùng
cơ chế cho tool đơn giản như Polyline và cho Builder; có tùy chọn click,
toggle, lựa chọn con và nhập giá trị. Các quy tắc vận hành chi tiết dưới đây
là quyết định thiết kế OM9. Ảnh không xác lập defaults, đơn vị, miền số hoặc
phạm vi native đã hỗ trợ. **Đây là yêu cầu triển khai/nghiệm thu, chưa phải
bằng chứng runtime.** Các supported slices và trạng thái đã ghi vẫn giữ nguyên.

### Mô hình option và cách hiển thị

Safe Rust sở hữu schema option, giá trị đã nhận, phase và validation; Qt chỉ
hiển thị snapshot và chuyển thao tác tới session. Schema phải mô tả:

| Dữ liệu | Hợp đồng |
|---|---|
| Identity | Exact feature ID, `option_key` ổn định và parent key nếu là tùy chọn con; label/alias không thay identity. |
| Kiểu thao tác | `toggle`, `choice`, `number` hoặc `action`; kiểu thuộc feature, không suy từ chữ hiển thị. `Close` của Polyline là action kết thúc wire, không phải toggle như Close của một Builder khác. |
| Tên và giá trị | Tên/alias gốc, mnemonic theo phase, giá trị đã nhận và biểu diễn hiện hành; dùng `Option=Value` khi có giá trị. On/Off, Yes/No hoặc cặp khác phải do feature khai báo. |
| Lựa chọn con | Parent/child keys, các giá trị hợp lệ, thứ tự hiển thị và capability của từng nhánh; child có thể là choice, toggle, number hoặc action. |
| Số | Đơn vị, miền/ràng buộc liên option, grammar và default nếu đã được feature xác lập; giữ raw draft tách khỏi parsed quantity và accepted value. |
| Phạm vi và gate | Hiển thị tại CMD, Builder hoặc cả hai; phase hợp lệ, enabled/disabled và lý do theo capability thực tế. |
| Lifetime | Chính sách giữ giá trị trong command, trong phiên hoặc qua lần mở lại do từng feature công bố; không mặc định mọi option là preference lâu dài. |

Live prompt đọc tên và giá trị từ snapshot hiện hành. Vùng click và mnemonic
phải sinh từ schema của phase, không hard-code danh sách tên option hoặc
đoán option bằng regex trong transcript. Lịch sử vẫn là chữ có thể chọn/copy;
click vào prompt cũ không phát sự kiện sửa session hiện tại. Option chưa hỗ
trợ giữ trạng thái disabled với lý do, không tự chọn giá trị hay backend khác.

Tên đầy đủ, alias/mnemonic đã đăng ký và click phải giải về cùng option key.
Nhập tên rồi Enter kích hoạt option; `Option=value` đặt giá trị trực tiếp
nếu schema hỗ trợ cú pháp đó. Alias/mnemonic chữ không phân biệt hoa/thường;
collision trong phase phải yêu cầu lựa chọn rõ hoặc tên đầy đủ, không chọn
option đầu tiên. Không thay tên command gốc hoặc chiếm phím của IME/text field.

### Toggle, tùy chọn con và nhập số

| Loại | Click hoặc nhập tên option | Nhận giá trị và quay về phase cha |
|---|---|---|
| Toggle hai trạng thái | Một click hợp lệ đảo đúng một lần giữa hai giá trị đã khai báo và cập nhật prompt/preview. | Nhập giá trị tường minh đặt đúng giá trị đó; đặt lại cùng giá trị không đảo lần nữa. Không commit document chỉ vì toggle. |
| Choice / tùy chọn con | Mở prompt danh sách con, hiển thị lựa chọn hiện hành và các nhánh khả dụng; mở danh sách không tự chọn mục đầu. | Click child hoặc nhập tên/mnemonic/value hợp lệ nhận lựa chọn. Nếu child cần số hoặc một cấp con nữa, tiếp tục phase tương ứng; chỉ cập nhật option khi đủ input hợp lệ. |
| Number | Chuyển sang chờ số, hiện tên option, đơn vị, giá trị hiện hành và miền/ràng buộc đã xác lập. | Nhập số rồi Enter nhận giá trị, cập nhật preview và trở về phase cha; Enter này không đồng thời kết thúc command/Builder. |
| Action | Thực hiện đúng action của feature, ví dụ Undo bước, Reset hoặc Close wire. | Quyền kết thúc/commit tùy action và phase; không áp dụng quy tắc toggle chỉ vì action có tên giống một option ở lệnh khác. |

Trong danh sách con, hover/highlight chỉ đổi lựa chọn đang trỏ; chưa nhận
giá trị. Enter chỉ nhận lựa chọn hợp lệ được chỉ định/đang chọn rõ; không có
lựa chọn thì tiếp tục chờ. Esc đóng cấp con hiện tại, giữ accepted value và
trở về prompt cha; Esc tiếp theo ở phase command hủy theo lifecycle chung.

Khi chờ số, parser dùng grammar và quantity context của feature, kiểm hữu
hạn, đơn vị, miền, enum liên quan và ràng buộc chéo trước khi nhận. Sai số,
NaN/Infinity, sai đơn vị hoặc ngoài miền giữ accepted value/preview trước đó,
giữ draft và báo lỗi có thể sửa; không clamp hoặc thay bằng default âm thầm.
Enter rỗng chỉ nhận current/default đã hiển thị nếu feature cho phép và giá
trị còn hợp lệ; nếu không có thì tiếp tục chờ. Esc bỏ draft số, giữ accepted
value và quay về phase cha. Khi còn chờ số/choice, Enter không đi sang commit.

Click option chỉ kích hoạt khi không kéo chọn text và không có draft chưa
nhận. Draft không rỗng được giữ nguyên kèm feedback yêu cầu nhận/hủy draft
trước khi chuyển option; không ghi đè input. Up/Down/Tab nếu dùng cho lựa
chọn con phải theo phase đó, không mở command-name completion hoặc recall
history. Pan/zoom và viewport navigation vẫn không phải option/point input.

### Một trạng thái chung cho CMD và Builder

CMD và Builder controls/Options List có thể có tập option khác nhau. Với
option chung, hai phía phải map cùng feature ID/option key và gửi sự kiện
có kiểu tới cùng Rust session/reducer/validator, đọc cùng accepted snapshot.
Không duy trì một giá trị CMD và một giá trị Builder độc lập. Quy tắc binding
chi tiết ở [Builder Framework](../01-core/om9-buildercore-001-builder-framework.md#command-line-options).

Sau mỗi input hợp lệ, accepted value, prompt, control hiển thị, các gate
phụ thuộc và preview cùng phản ánh một revision option. Update widget từ
snapshot không được phát ngược một user event, toggle lần hai hoặc transaction
thứ hai. Input lỗi giữ state trước đó ở cả hai phía. Nếu CMD có draft chưa
nhận cho một option, thao tác Builder sửa chính option đó bị chặn với feedback
cho tới khi draft được nhận/hủy; không âm thầm ghi đè draft. Thay option khác
có thể làm draft mất điều kiện hợp lệ: giữ draft và yêu cầu validation lại.

Mở/đóng Builder panel không tạo command session thứ hai hoặc reset values.
Đổi Mode/Reset dùng cùng reducer, cập nhật tập option/choices/gates và xử lý
draft không còn hợp lệ có thông báo. Option chỉ có ở một phía vẫn dùng state
chung nếu tham gia cùng feature; không bắt buộc bổ sung control tương ứng.

Toggle/choice/number chỉ sửa session và preview, không tạo geometry/Undo
document. Enter tại prompt chính chỉ commit khi không còn input pending và
mọi điều kiện của feature đã đạt; geometry, parameters và metadata được ghi
trong một transaction theo hợp đồng feature. Esc/close document/deactivate
dọn draft, choice và preview; không để callback sống hoặc success history giả.
Callback/preview result phải kiểm session generation và option revision;
kết quả từ values/Mode cũ không được thay preview mới hoặc commit về sau.

### Khoảng cách với implementation hiện có

`CommandConsole::optionAt`/`stylePrompt` hiện nhận diện một tập tên cố định;
click `Mode` gửi `Mode=Line`. Line/Polyline có một số toggle và nhập Length,
nhưng các bằng chứng đó chưa chứng nhận schema option chung, lựa chọn con
hoặc đồng bộ mọi Builder. API BuilderRecord cũng không thay thế Builder UI.
Ví dụ GemShape, PlacementMethod, Roll, Flip, Mirror và TaperDiameter trong
ảnh phải được nối theo spec feature/capability; tên xuất hiện không bật
`gvGemOnCrvAdvanced` khi command đó vẫn chưa được triển khai.
<!-- command-line-options-contract-20261012:end -->

## State machine và quy tắc phím

1. `Idle → Preparing`: resolve ID/gate, chụp selection, document revision,
   cài callback thuộc session và tạo prompt đầu. Gate thất bại giữ document nguyên vẹn.
2. `Preparing → Selecting/Collecting`: preselection chỉ được dùng nếu đúng type,
   cardinality và lock policy. Thiếu input chuyển postselection; input sai có lý do,
   không silently bỏ đối tượng rồi thao tác trên tập khác ngoài ý người dùng.
   Preselection trong workflow Rhino là tập objects đã chọn trước khi gọi lệnh;
   `Gui::Selection::setPreselect` của FreeCAD chủ yếu là hover highlight.
   API hover không đủ chứng minh preselection/postselection của command đã đúng.
3. `Collecting → Preview`: mỗi accepted input tạo checkpoint cho session Undo;
   hover chỉ cập nhật preview, không đẩy checkpoint và không tạo document object.
   Prompt nêu phase, defaults OM9 được công bố và options thật sự hỗ trợ.
4. Enter nhận option/default/kết thúc tập input tùy phase; khi chưa đủ số điểm
   phải tiếp tục chờ. Space/right-click phải dùng bảng event theo phase/focus;
   không tự coi mọi right-click là commit vì nó còn có context/navigation role.
5. Esc khi đang chọn completion có thể chỉ đóng completion trước; hành vi tầng
   kế tiếp phải nhất quán và có fixture. Esc của command dọn session/preview;
   Escape lúc idle xử lý selection theo RCORE-06, không sửa geometry.
6. `Preview → Committing`: revalidate identity, revision, constraints, ownership,
   editability và backend. Mở một transaction cho toàn geometry+metadata+History.
   Lỗi abort toàn transaction; không có output một phần hoặc undo entry rỗng.
7. `Completed → Idle`: ghi successful history sau commit thành công, tháo callback,
   xóa preview và đặt repeat candidate theo command repeatability policy.
8. `RecoverableError`: giữ points/options hợp lệ và cho Undo/option/Cancel sửa;
   `FatalError`, document close hoặc workbench deactivate dọn tài nguyên một lần.
   Worker trả muộn phải bị loại bằng session generation, không hồi sinh command.

Pan/zoom/orbit và chuyển viewport trong phase pick là navigation event,
không mặc định tạo một point/commit/cancel. RCORE-03 quyết định frame latch;
RCORE-05 quyết định marker/point cuối. Modal host task chặn command phải báo lý do.
Một lệnh mới khi session đang hoạt động cần policy reject, suspend hoặc cancel-old;
không để hai controller cùng sở hữu mouse callback và active transaction.

## Bốn loại lịch sử cần giữ riêng

| Loại | Dữ liệu và invariant |
|---|---|
| Session Undo | Checkpoint points/options/frame/selection tạm; không gọi Undo document để lùi một hover/point |
| Document Undo/Redo | Transaction geometry+metadata+dependency graph đã commit; giữ nguyên identity policy |
| Transcript/input history | Prompt, chuỗi nhập, lỗi và kết quả; có thể lưu input thất bại, không coi là successful execution |
| Associative History | Graph source/parameters/output của RCORE-09; transcript và Python macro không thay thế graph |

Repeat tái khởi động command có input/options theo policy đã chốt;
không tự replay pointer selection đã stale. Repeated failure không trở thành success.
Macro/RhinoScript/plugin engine là capabilities độc lập, có parser/runtime/backend
riêng. Python console hoặc alias giống Rhino không chứng minh chạy RhinoScript/plugin.

## FreeCAD có gì, OM9 cần nối gì

Các nhãn dưới đây là khả năng đọc được từ source, không phải runtime pass.

| Capability ID | Năng lực | FreeCAD | Bằng chứng / giới hạn | OM9 hiện tại / việc cần bổ sung |
|---|---|---|---|---|
| RCORE-04.C01 | Command registry/menu/action | UI+API | [Command][fc-command] `CommandBase`, `Command`, `CommandManager`; QAction activation | [Command.cpp][om9-command] gate/dispatch theo index; cần request thống nhất mọi entrypoint |
| RCORE-04.C02 | isActive và quyền host | UI+API | `Command::isActive`, host GUI command lifecycle | `om9NativeCommandAvailable` xét document/control và các family; giữ slice explicit |
| RCORE-04.C03 | Atomic transaction/Undo | API | `openCommand`, `commitCommand`, `abortCommand` nối App transaction | Curve commit đã có; phải nghiệm thu mỗi family và metadata/History đi cùng |
| RCORE-04.C04 | Console nhập lệnh | Một phần | [PythonConsole][fc-console] `InteractiveInterpreter::runSource` compile/execute Python | [CommandConsole][om9-console] là UI riêng; cần grammar/options/session Rhino ở Rust |
| RCORE-04.C05 | Prompt/options/completion | Một phần | PythonConsole có Python completion/history, không chứng minh command options Rhino | `optionAt`, `chooseCompletion`, `setPrompt`, Rust [completion][om9-completion]; cần typed option model |
| RCORE-04.C06 | Preselection/postselection | UI+API | [SelectionSingleton][fc-selection] có selected/preselected objects và gates | Các controller phải normalize filter/cardinality cùng RCORE-06 |
| RCORE-04.C07 | Session checkpoint và Esc | Một phần | Host hỗ trợ command/task callback; mỗi workbench tự có lifecycle | [Curve Session][om9-session] và [CurveController][om9-controller] có input/cancel; chưa kiểm xuyên family |
| RCORE-04.C08 | Repeat từ success history | Một phần | Host command registry không tự cung cấp semantics repeat Rhino cho mọi command | `CurveController::submit` input rỗng gọi history đầu; cần exclusion/focus/phase fixtures |
| RCORE-04.C09 | Navigation trong active pick | Một phần | Native view có event/navigation API; hành vi phụ thuộc tool | Chưa chứng minh pan/orbit/Space/right-click cùng policy cho mọi controller |
| RCORE-04.C10 | RhinoScript/Rhino plugin runtime | Chưa thấy trong phạm vi rà | Rà Command/PythonConsole chỉ thấy Python host execution, không Rhino runtime | Alias/completion không phải implementation; giữ unsupported capability rõ |
| RCORE-04.C11 | Close/deactivate/stale cleanup | Chưa kiểm chứng | Host document/view lifecycle là nền tảng cần tích hợp | Curve có `validDocument`, cancel và preview cleanup; cần worker/native replay |

## Bằng chứng OM9 và giới hạn kết luận

`CurveController::submit` chuyển input sang Solid/Surface/Edit/History đang active,
thử các command core, rồi gửi active curve vào Rust; hiện vẫn là logic định tuyến
ở C++ cần từng bước đưa quyết định thuần dữ liệu sang Rust.
Input history trong controller ghi chuỗi không rỗng trước xử lý; điều này hợp lý
cho transcript nhưng không được dùng như bằng chứng success.
`CurveController::result` gọi `om9_sidebar_record_execution(...,true)` sau commit;
đó là bằng chứng nguồn cho đường curve, không chứng minh tất cả route đều giống nhau.
`CommandConsole::keyPressEvent` phân biệt completion Escape với cancelled callback.
Không chỉnh sửa các semantics hiện có trong đợt tài liệu này.

## Kế hoạch adapter, lỗi và ownership

Safe Rust sở hữu command catalog/capability model, alias resolution, event reducer,
session checkpoints, repeatability và typed commit plan. UI đọc một snapshot prompt.
C++ giữ QAction/Qt event hookup, document transaction, native selection và solver calls;
Python chỉ registration hoặc API host bắt buộc. Không gửi raw command string vào
Python thay cho typed adapter khi đang có native/Rust contract phù hợp.
Native giữ document/Qt handle, Rust giữ ID+generation; callback teardown idempotent.
Mọi FFI trả status/error có ownership string rõ, không để panic/exception qua ABI.
Phụ thuộc RCORE-01 capability/input types; RCORE-02 units; RCORE-03 parser/frame;
RCORE-05 pick; RCORE-06 selection; RCORE-09 transaction graph; RCORE-12 workers.

## Chi tiết bổ sung từ code decompile

Các chi tiết dưới đây vẫn ở mức `source_inspected`, chưa kiểm chứng runtime.
“Thân managed” chỉ nhánh xử lý, tham số hoặc giá trị gán thấy trực tiếp;
“SDK” chỉ hợp đồng trong chú thích đi kèm code. Phần giao cho native vẫn
cần kiểm chứng riêng. Giữ nguyên đánh giá FreeCAD và phạm vi native OM9
đã nghiệm thu ở các phần trên.

| Hành vi và dữ liệu | Giới hạn và yêu cầu tích hợp |
|---|---|
| Style tách cờ Hidden, ScriptRunner, Transparent, DoNotRepeat và NotUndoable. SDK mô tả transparent chạy lồng, không sửa geometry, không thêm/xóa view; ví dụ còn gồm selection commands. Getter có cờ cho transparent, Nothing và Undo riêng; SDK mô tả transparent được phép mặc định. | Default và hạn chế transparent ở đây là chú thích SDK; các setter chuyển native. Không thấy runtime enforcement, danh sách exhaustive hoặc policy Ctrl+Z trong thân này. Hidden nói về command completion, không đồng nghĩa disabled capability OM9. |
| OnRunCommand khởi tạo Failure, giải command/document, chuyển mode dương thành Scripted, bắt và báo exception. GetCommandStack duyệt IDs native đến Guid.Empty, trả null nếu rỗng; SDK xác định active command nằm cuối danh sách. | Không chứng minh exception tự rollback document. Session lồng cần stack/ownership rõ; không suy InCommand thành chỉ một controller duy nhất hoặc đồng nghĩa đang commit. |
| Cả hai overload RunScript chặn event watcher bằng exception trước native. SDK phân biệt script runner hoàn tất script trước khi trả về với lời gọi ngoài command trả về trước khi script chạy. | Chưa đọc engine macro native; bool trả về không đủ chứng minh geometry đã commit. Callback UI cần dispatch thích hợp, không chạy script ngay trong event watcher. |
| Undo enabled và recording active là hai truy vấn. SDK dành BeginUndoRecord cho thay đổi ngoài command và mô tả trả 0 nếu không mở được record. AddCustomUndoEvent từ chối description rỗng hoặc handler null trong managed, chỉ giữ callback sau khi native trả serial khác 0. | Không thấy transaction rollback tự động hay bảo đảm mọi plugin data được Undo. AcceptUndo trong getter là option input riêng; không thay hợp đồng transaction geometry+metadata của OM9. |
| SDK mô tả hai nguồn repeatability: khi UseNeverRepeatList bật, danh sách cấu hình thay cờ không-repeat từng command. Managed SetList ghép tên rồi gửi native; CommandNames đọc và tách danh sách hiện tại. | Chưa có danh sách default từ runtime. Không đóng cứng Undo/Delete thành toàn bộ exclusion list hoặc lấy transcript/MRU làm danh sách repeat hợp lệ. |

Hệ quả cho engine/UI: phân loại command transparent là capability riêng; đang
collect input có thể cho view/snap/selection command chạy lồng theo gate của phase.
Không biến mọi lệnh mới thành cancel-old. `EnableTransparentCommands` ở getter và
`Style.Transparent` ở command là hai điều kiện khác nhau. Repeat cần nguồn policy
và version cấu hình; không thay history success OM9 bằng NeverRepeatList Rhino.
Các quy tắc lỗi/rollback và preselection nghiêm ngặt phía trên là contract OM9,
không suy từ exception handling hoặc preselect defaults của Rhino: getter có
thể bỏ phần preselection không hợp lệ hoặc chuyển sang postselect tùy cấu hình.

| Fixture bổ sung — chưa chạy | Expected invariant / câu hỏi cần ghi nhận |
|---|---|
| RCORE-04.T10 | Đang pick chạy transparent view/snap/selection command, rồi quay về phase; kiểm stack và callback owner. Tắt transparent tại getter phải chặn entrypoint đó mà không mất input |
| RCORE-04.T11 | Repeat cùng command khi dùng style flag và khi cấu hình NeverRepeatList: ghi nguồn policy thực tế; không repeat lệnh disabled hoặc selection stale |
| RCORE-04.T12 | Thay modeless property ngoài command: kiểm BeginUndoRecord trả serial/0, End chỉ record đã mở; Undo phục hồi plugin metadata cùng geometry, AcceptUndo phase không gọi nhầm document Undo |
| RCORE-04.T13 | RunScript từ script runner, ngoài command và event watcher: ghi thời điểm completion/commit; nhánh watcher bị từ chối trước native, không ghi dispatch thành success |

## Fixtures cần nghiệm thu

Tại baseline audit nguồn các ca dưới đây **chưa chạy**; nghiệm thu mới phải kiểm từng invariant geometry/state ngoài ảnh UI.

| Fixture | Expected invariant |
|---|---|
| RCORE-04.T01 | Cùng Circle/Line input qua menu, CMD, F6 và shortcut cho cùng exact ID, world geometry và một transaction |
| RCORE-04.T02 | Esc ở selection/point/option/preview và completion mở; teardown đúng tầng, không output/undo entry dư |
| RCORE-04.T03 | Nhập option bằng tên/mnemonic/click trong phase số; không parse nhầm option thành numeric failure hoặc đổi phase |
| RCORE-04.T04 | Idle Enter/Space/right-click repeat command được phép; Undo/Delete/error/cancel không thay bằng một “successful command” giả |
| RCORE-04.T05 | Undo điểm khác Ctrl+Z document; Redo geometry khôi phục metadata/History; transcript vẫn phản ánh input lỗi |
| RCORE-04.T06 | Preselection sai type, alias Rebuild trùng, backend unavailable: giải thích rõ, không tự chọn family đầu hoặc mutate một phần |
| RCORE-04.T07 | Pan/zoom/orbit/chuyển viewport giữa hai điểm: session còn active; navigation không tăng point count |
| RCORE-04.T08 | Đóng document/deactivate trong preview hoặc worker, nhận kết quả muộn: không crash, không object mới, không callback còn sống |
| RCORE-04.T09 | IME/text field gõ tên có khoảng trắng và Ctrl+C/V: focus owner đúng, không kích command/repeat/clipboard geometry |

<!-- command-line-options-fixtures-20261012:start -->
<a id="command-line-options-fixtures"></a>

## Fixtures Command Line Options — đã kiểm supported slice

Các ca này bổ sung T01–T13, phải ghi exact feature ID và supported options
của command/Builder thực sự đang kiểm. Fixture giả lập schema chỉ xác nhận
reducer; nghiệm thu native phải kiểm state, preview, geometry và transactions.

| Fixture | Expected invariant |
|---|---|
| RCORE-04.T14 | Click/tên/mnemonic toggle cho cùng state; hai click khôi phục ban đầu; đặt giá trị tường minh hai lần không đảo; prompt và preview theo accepted value. |
| RCORE-04.T15 | Mở choice không tự chọn/đổi preview; chọn child bằng click/tên cho cùng value; child số và nhiều cấp đúng phase; Esc từng cấp giữ giá trị cũ; Enter thiếu lựa chọn tiếp tục chờ. |
| RCORE-04.T16 | Nhập số hợp lệ, units và Enter chỉ nhận option; sai grammar/NaN/Infinity/range giữ draft, state và preview cũ; Enter rỗng theo default/current đã công bố, không commit khi pending. |
| RCORE-04.T17 | Với option chung, cùng giá trị qua CMD và Builder control/slider cho cùng parameters và preview/geometry; refresh widget không phát event vòng hoặc toggle/transaction lần hai. |
| RCORE-04.T18 | CMD-only, Builder-only và shared options có đúng visibility/gate; disabled child có lý do và không dispatch; tên option gần giống ở hai feature không bị map chung. |
| RCORE-04.T19 | Drag/copy transcript, click prompt cũ, draft không rỗng, IME và mnemonic collision không sửa nhầm state; Builder sửa option có CMD draft bị chặn, Enter/Esc xử lý xong draft mới nhận thay đổi. |
| RCORE-04.T20 | Mode/Reset cập nhật options/choices/gates ở cả hai phía; values/draft không hợp lệ được báo rõ; kết quả preview của revision cũ bị loại, không phục hồi giá trị cũ. |
| RCORE-04.T21 | Enter ở prompt chính commit một transaction khi đủ input; Undo/Redo và save/reload giữ geometry/parameters/metadata theo feature; thao tác option trước commit không tạo object hay Undo document. |
| RCORE-04.T22 | Esc, đóng panel/document và deactivate tại choice/number/preview dọn đúng phạm vi; đóng panel không reset session đang hợp lệ; cancel/document-close không output, callback hoặc late commit. |
| RCORE-04.T23 | `Close` action của Polyline đóng/kết thúc theo điều kiện điểm; `Close` toggle ở Builder dùng cặp state riêng; không suy kiểu hành vi từ label và không giả lập command disabled để báo pass native. |

- [x] Toggle/choice/number/action theo schema được kiểm chứng native trong supported slice Curve/Rebuild/Sweep/Loft.
- [x] Tùy chọn con, Enter/Esc theo phase và invalid-input recovery được kiểm chứng với consumer hiện có.
- [ ] Đồng bộ CMD–Builder, draft conflicts và loại stale preview được kiểm chứng.
- [ ] Gate, atomic commit, cancellation và persistence theo feature được kiểm chứng.
<!-- command-line-options-fixtures-20261012:end -->

<!-- command-options-implementation-20261012:start -->
Kết quả cuối: 30 reducer/ABI tests; 3 native fixtures mới 26+25+15 checks.
CMD–Builder shared controls đã đồng bộ hai chiều; các checkbox nghiệm thu rộng
bao gồm private UI/Reset/exhaustive stale lifecycle vẫn giữ chưa đánh dấu.

2026-10-12: có engine Rust chung, đăng ký schema động theo exact feature ID
và binder Qt dùng lại cho lệnh khác; không đóng khung vào OffsetBezier.
Native Rebuild dùng API đăng ký chung, Curve và Sweep/Loft dùng các schema
consumer. Toggle, choice/child-number và CMD–Builder được kiểm qua controls,
preview/Part geometry, Enter/Esc, raw numeric drafts và transactions thực.
Giữ `partially_implemented`: private scopes mới chứng minh ở reducer/ABI;
Reset, Gem solvers và các phần RCORE khác chưa được nâng trạng thái.
[API đăng ký](../../../docs/command-options-adapter.md) ·
[Native evidence và baseline failures](../../../docs/validation/2026-10-12-command-line-options.md).
<!-- command-options-implementation-20261012:end -->

## Còn cần bằng chứng tương thích

Chưa chốt exhaustive mnemonic collisions, runtime transparent commands và danh sách
repeat exclusion mặc định; SDK đã mô tả cờ transparent, gate của getter và
danh sách cấu hình không repeat. Còn thiếu
Space/right-click theo mọi phase, macro quoting hoặc RhinoScript semantics.
Negative claim chỉ trong các file Command/PythonConsole và OM9 được dẫn ở đây;
không phải kết luận mọi workbench/add-on FreeCAD đều thiếu khả năng đó.

[audit]: ../../../docs/reviews/2026-10-09-rhino-core-gap-audit.md
[fc-command]: ../../../../../src/Gui/Command.cpp
[fc-console]: ../../../../../src/Gui/PythonConsole.cpp
[fc-selection]: ../../../../../src/Gui/Selection/Selection.h
[om9-command]: ../../../Gui/Command.cpp
[om9-console]: ../../../Gui/CommandConsole.cpp
[om9-completion]: ../../../rust/src/command_completion.rs
[om9-session]: ../../../rust/src/curve.rs
[om9-controller]: ../../../Gui/CurveController.cpp
