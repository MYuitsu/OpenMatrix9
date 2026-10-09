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
