# Đặc tả chuẩn hóa Command và tham số theo Polyline

Ngày: 2026-10-26. Đối tượng thực hiện: GPT Luna hoặc người triển khai tiếp theo.

**Trạng thái: đặc tả công việc tiếp theo, chưa phải chứng nhận các thay đổi dưới đây đã được triển khai.** Người dùng yêu cầu viết mô tả chi tiết; lượt viết tài liệu này không chạy migration toàn bộ ứng dụng, không sửa skill và không phát hành binary.

## 1. Kết quả phải đạt

Khi chuyển giữa các lệnh, Command luôn lấy prompt, tham số, giá trị, phím tắt và bước nhập từ **lệnh đang sở hữu input**. Mọi lệnh có tham số phải có cùng chất lượng tương tác như Polyline: bấm chữ gạch chân hoặc click đều cập nhật đúng tham số; nhập giá trị có bước rõ ràng; lỗi giữ dữ liệu để sửa; Enter/Escape/Undo có kết quả xác định.

“Giống Polyline” là giống hợp đồng tương tác. Không gắn `PersistentClose`, `Mode`, `Helpers`, `Length` hoặc `BothSides` vào lệnh không sở hữu chúng. Không thay thuật toán dựng hình chỉ để làm giao diện trông giống nhau.

Ba lỗi cần ngăn tận gốc:

1. Đang ở `Start of polyline` nhưng hiện `BothSides=No Length=` của Line trước đó.
2. Nhấn phím tắt/click chỉ đổi chữ hoặc đẩy một chuỗi `Radius=` thiếu giá trị vào parser và báo lỗi.
3. Dùng dữ liệu/ảnh của binary cũ rồi kết luận source mới vẫn lỗi, hoặc báo hoàn tất chỉ vì lệnh mở được.

## 2. Đọc nguồn theo đúng thứ tự

1. `AGENTS.md`, `C:/Users/nguye/.codex/RTK.md`, `docs/openmatrix9-progress.json`.
2. `skills/openmatrix9-workflow/SKILL.md`; các reference Rust-first, milestone target, progress.
3. `OM9_Milestones/M01 - CAD Shell, Viewport/L1.json`; owner `OM9-L2-03-10` và L3 `OM9-L3-03-10-003`; command-engine owner `OM9-L2-01-02` cho input/dispatch/session.
4. `skills/openmatrix9-ui/SKILL.md`, `skills/openmatrix9-workspace-contract/SKILL.md`, reference Command/Curve, `skills/openmatrix9-build-validation/SKILL.md`.
5. `crates/om9-app/src/command_options.rs`, `polyline_options.rs`, `session/command_options.rs`, `session/polyline.rs`, `ui/command.rs`, `transcript.rs`.
6. `COMMAND-MATRIX.vi.md`, `OWNER-CARDS.vi.md`, `TASKS.vi.md`, `ACCEPTANCE.csv` trong thư mục này.
7. Trước mỗi family: đọc đúng `session/<owner>.rs`, parser và test của family đó, rồi resolve exact feature/milestone bằng workflow router. Không tải toàn bộ milestone hay Ghidra export vào context.

Milestone cục bộ đủ để tra semantics đã chuẩn hóa. Không đọc lại PDF chỉ vì gặp một option. QymCAD chỉ là mẫu routine khi thật sự cần; công việc này mặc định không cần kernel hay mẫu QymCAD mới.

Các skill cũ mô tả một slice Polyline thẳng và chưa xác nhận Arc/Helpers. Mã và evidence ngày 2026-10-25 đã bổ sung Arc/Direction/Center; Helpers mới có flag, chưa chứng minh tracking. Giữ khác biệt lịch sử này rõ ràng, không xóa Arc đang có, không tự sửa skill để hợp thức hóa trạng thái.

## 3. Hiện trạng đã đối chiếu với source

| Thành phần | Đã có | Chưa được suy ra là hoàn tất |
|---|---|---|
| `PolylineOptionInterface` | Typed ID/value, snapshot theo stage, activation qua owner | Không chứng minh mọi lệnh đã có value-entry giống Polyline |
| `CommandOptionInterface` | `command_options`, `resolve_command_option`, `activate_command_option` | Không có typed current value độc lập trong record chung; hiện chủ yếu là `label` |
| Adapter chung | 37 provider + Line + Polyline = 39 nhóm khai báo | 39 nhóm không phải toàn bộ số lệnh hay số hàm trong ứng dụng |
| Value entry chung | `CommandOptionEffect::Draft(String)` cho nhiều owner | Đây là cầu nối nhập `Name=`, chưa phải owner substage đầy đủ |
| UI | Polyline có inline token; các owner khác còn hàng button chung | Chưa thống nhất hoàn toàn prompt inline theo chuẩn đích |
| Identity | `CommandOptionId { owner, name }` | Chưa có run/generation trong ID nên cần bảo vệ snapshot cũ qua phiên mới |
| Controller | 57 field `_pending`; 19 field ngoài registry option | Số field không phải số command; cần audit các branch và command không dùng `_pending` |
| Runtime | Debug `28915c56…` đã kiểm tra; release `f6b75cee…` cũ hơn | Không mặc định executable mang tên `om9.exe` là bản mới |

Số liệu source và khai báo token đầy đủ nằm trong `source-inventory.json`. Các số này là inventory ngày viết, không phải tỷ lệ hoàn thành. Khi nguồn đổi, đối chiếu SHA256 và cập nhật inventory trước khi kết luận thiếu/thừa.

Evidence có sẵn:

- `docs/validation/2026-10-25-polyline-parameter-interface/README.md`.
- `docs/validation/2026-10-25-command-parameter-interface/README.md`.
- `docs/validation/2026-10-26-polyline-option-owner/README.md`.
- Một test Box workflow cũ thất bại cả trên baseline; hai test native fixture từng bị ignore. Không gọi chúng là đã sửa bởi migration này.

## 4. Phạm vi và phân loại tất cả lệnh

Mỗi command được phát hiện phải có đúng một phân loại, một owner và lý do:

| Phân loại | Việc phải làm |
|---|---|
| `OPTIONS` | Expose tham số đang khả dụng qua contract chung; đủ metadata, stage và ba đường kích hoạt |
| `POSITIONAL_INPUT` | Expose bước nhập hiện tại qua cùng command snapshot; giữ thứ tự đối số của owner, không bịa option có thể sửa tùy ý |
| `IMMEDIATE_ACTION` | Thực thi một lần qua dispatcher; không tạo hàng option rỗng giả hoặc reuse owner trước |
| `SPECIALIZED_SURFACE` | File dialog, Layers, Gumball numeric, plugin JSON hoặc editor riêng: ghi owner chuyên biệt và lý do; Command không chặn focus/input của nó |
| `UNSUPPORTED` | Vẫn visible-disabled với lý do; không bật nút hoặc giả lập thành công để đủ bảng |

Không dừng ở 39 nhóm. Audit cả 19 pending ngoài registry, các variant của Top11, nhánh Solid ngoài Box/Sphere, `dispatch_effect`/`submit_effect`, menu Curve/Surface/Solid, toolbar/F6, File/View/Layers và plugin entry points. Danh sách phát hiện thêm phải được ghi thành hàng mới. Không thể công bố “mọi lệnh xong” khi còn `UNCLASSIFIED` hoặc branch chưa xem.

Không bắt mọi command phải có option. `ZoomExtents`, Undo tài liệu hoặc một immediate action không cần mang tham số Polyline. Plugin `PluginRun ... JSON` không được tự đổi protocol JSON sang phím tắt chữ nếu SDK không định nghĩa điều đó.

## 5. Owner và vòng đời phiên lệnh

### 5.1. Active owner khác repeat slot

`active_feature()` hiện có fallback sang lệnh có thể repeat. Vì vậy không dùng hàm đó để quyết định option nào đang được vẽ. Repeat slot chỉ phục vụ empty Enter ở trạng thái idle; không phải nguồn active owner.

Tạo hoặc mở rộng một truy vấn rõ nghĩa, ví dụ `active_command_input_snapshot()`. Đây là **API đích đề xuất**, chưa tồn tại nguyên dạng. Snapshot phải xác định được:

- Feature/command ID thật, owner domain, một run ID/generation.
- Document ID và context/revision/epoch cần kiểm theo kiến trúc hiện có.
- Stage hiện tại, prompt nền, cách nhận input và điều kiện finish/cancel.
- Danh sách option được owner cho phép ở stage này.

Ở idle: active command là `None`, option list rỗng dù repeat slot có Line. Khi có specialized pending: pending đó thắng repeat slot. Không dùng thứ tự “provider nào có token trước thì coi là active” làm giải pháp lâu dài cho hai pending cùng sống.

### 5.2. Không rewrite toàn bộ session trong một lần

Bắt đầu bằng owner resolver/adapter trên các pending hiện có. Giữ identifiers, dispatcher và geometry handlers. Sau đó chuyển từng family. Nếu phát hiện hai owner tương tác cùng active, xử lý qua lifecycle/context hiện có và báo lỗi xác định; không âm thầm trộn snapshot hoặc chọn một bên tùy thứ tự Vec.

Khi commit/cancel, đóng document, thay document, context stale hoặc bắt đầu một run mới: clear parameter-entry của run cũ. Snapshot/click ID cũ phải bị từ chối mà không mutate run mới, kể cả command mới có cùng tên/owner.

## 6. Record tham số đích

Giữ contract hiện có để migration không đứt build. Có thể thêm trường/type bằng adapter; không đổi tên mọi handler. Mỗi option cần biểu diễn các mục sau, không nhất thiết dùng đúng tên Rust dưới đây:

| Mục | Yêu cầu |
|---|---|
| `id` | Identity ổn định trong owner; không lấy display label có giá trị làm identity |
| `owner/run` | Dùng để kiểm snapshot còn thuộc đúng phiên; same-name khác run không tự hợp lệ |
| `name` | Tên Matrix/command hiện có: `Length`, `Radius`, `Degree`… |
| `kind` | Toggle, Choice, Number/Integer, Point/Vector, Text, Action; phân biệt số nguyên bằng type/constraints |
| `current_value` | Typed value hoặc `Unset`; được đọc từ owner, không parse ngược `label` |
| `unit_semantics` | Length/angle/dimensionless/count/fraction/point/vector/text; conversion do owner quy định |
| `shortcut` | Chữ đang được quảng bá thật; có thể không có nếu không còn accelerator an toàn |
| `availability` | Available, hidden theo stage, hoặc disabled có reason cho capability thiếu; không activate option hidden |
| `activation` | Toggle/cycle/set choice, begin value entry, begin point entry, hay owner action |
| `constraints/default` | Bounds/default hợp lệ đã được owner xác định; không lấy mặc định từ chuỗi prompt |

Kiểu giá trị đích có thể gồm `Flag(bool)`, `Choice(stable_key)`, `Number(Option<f64>)`, `Integer(Option<usize>)`, `Point(Option<Point3>)`, `Vector(Option<Point3>)`, `Text(Option<String>)`, `None`. Đây là thiết kế đề xuất; tránh lưu bản sao state lâu dài trong UI.

Choice không được làm sai giá trị hiện tại: `Size=Area` là một lựa chọn thuộc tham số Size; nhấn nó chọn Area, không toggle thành Diameter. Nếu vẫn giữ nhiều nút choice riêng, record phải biết lựa chọn nào đang selected. `Approx` và `Deformable` đang là alias cùng flag ở Circle/Ellipse, không hiển thị chúng thành hai bool đối nghịch.

## 7. Máy trạng thái nhập liệu chung

Sơ đồ dưới là mô hình hành vi, không bắt buộc viết thành một enum khổng lồ thay mọi owner:

```text
Idle --start command--> AwaitPrimaryInput
AwaitPrimaryInput --toggle/choice--> AwaitPrimaryInput(updated snapshot)
AwaitPrimaryInput --select numeric/text option--> AwaitOptionValue(option, return_stage)
AwaitPrimaryInput --select point/vector option--> AwaitOptionPoint(option, return_stage)
AwaitOptionValue --valid value--> owner applies --> return_stage or owner next stage
AwaitOptionPoint --valid pick/text--> owner applies --> return_stage or owner next stage
Any input stage --invalid--> same stage, draft/state retained, explicit error
Any command stage --Escape--> owner cancellation --> Idle
Owner valid final input/Enter/action --commit succeeds--> Idle or owner's documented continuation
```

### 7.1. Chọn một option

Điều kiện keyboard: command run hợp lệ, option available, draft rỗng, không composing/paste, không popup hay editor khác sở hữu input, không Ctrl/Alt/Command chord, không key-repeat activation. Điều kiện click cũng phải giữ draft và IME. Drag chọn chữ không được trở thành click option.

Activation nhận ID và run/context; resolve lại snapshot mới nhất; kiểm option còn available; rồi gọi owner. UI không gán `pending.radius`, `pending.mode` hoặc geometry trực tiếp.

### 7.2. Toggle và Choice

- Toggle: một lần nhấn/click đổi đúng một lần; snapshot mới lấy lại owner value ngay cùng chu kỳ cập nhật UI. Giữ phím không đảo liên tục.
- Choice dạng cycle: chỉ cycle nếu owner vốn có hành vi đó, ví dụ Polyline Mode Line ↔ Arc.
- Choice dạng set: chọn giá trị cụ thể, ví dụ Arc Center, Circle 3Point, Sphere Solution=2. Chọn lại cùng value không tự nhảy sang value khác.
- Không commit geometry chỉ vì đổi flag/mode, trừ action/last-input đã được owner quy định rõ là commit.
- Khi mode đổi, owner quyết định point/parameter nào giữ/xóa. Không tự giữ điểm sai frame hay xóa toàn bộ point chain chỉ để refresh label.

### 7.3. Nhập số hoặc text theo chuẩn Polyline

Đích cuối: chọn `Length`/`Radius`/`Degree`/`File` đi vào owner-owned value-entry. Prompt chỉ rõ tham số đang nhập; draft ban đầu rỗng. Ví dụ sau khi chọn Turns: `Turns <3>:` rồi nhập `2`; không gửi `Turns=` thiếu value để thử parser.

`CommandOptionEffect::Draft("Turns=")` hiện có được giữ tạm trong migration. Family chỉ được đánh dấu `PARITY_DONE` sau khi đã chuyển sang stage rõ ràng hoặc đã chứng minh một protocol owner tương đương cần giữ vì đặc thù. Không để half-migration khiến keyboard dùng substage, click dùng prefix, typed name dùng parser thứ ba.

Giữ đường trực tiếp `Turns=2`, `Degree=3`, `Radius=5` để script/CMD dùng. Chuẩn hóa tên/alias đã khai báo, không lowercase nội dung text/path hay phần giá trị. `Name=` không có value chỉ mở entry khi đó là option hợp lệ; không success-no-op. Empty Enter trong entry chỉ nhận default khi owner khai báo default hợp lệ; nếu thiếu required value thì ở lại entry và báo lỗi, không rơi xuống finish/repeat.

Một value có thể là input cuối và commit ngay, ví dụ radius Circle sau center. Cả ba route phải giữ hành vi này. Không ép mọi owner phải thêm Enter thứ hai như Polyline, vì Polyline là lệnh multi-point có finish riêng.

Khi đã ở entry: input thường thuộc tham số đang nhập, không được fallback sang command-name dispatcher. Assignment đầy đủ cho chính tham số đó có thể được nhận qua cùng parser; assignment sang tham số khác bị từ chối rõ và giữ entry/draft cho đến khi hoàn tất hoặc hủy. Với Text/File, nội dung trông như tên command vẫn là nội dung. Menu File hay action ngoài editor vẫn theo owner/lifecycle riêng đã có, không suy ra quyền thay document từ chuỗi người dùng đang nhập vào Text.

### 7.4. Nhập điểm và vector

Point option như Center nhận click viewport hoặc tọa độ text qua cùng owner. Direction/Axis có thể là vector hoặc hai điểm tùy owner; metadata phải phân biệt, không mặc định mọi `Direction` là Point3. `AngleIsocurve Direction=U/V` là Choice, khác `Project Direction=x,y,z`.

Parse đủ token: sai số lượng, token rác, NaN/Inf, zero vector không hợp lệ phải bị từ chối. Không dùng `filter_map(parse.ok())` để bỏ token sai rồi vô tình nhận `1,bad,2,3` thành ba số. CPlane, units, snap và transform vẫn thuộc input/geometry owner hiện có; UI không nhân/chia hoặc đổi trục lần thứ hai.

### 7.5. Lỗi và hủy

Giá trị sai giữ nguyên stage, điểm đã nhận, current value cũ và draft sai để sửa; ghi lỗi có tên tham số. Không append prompt giả như đã success. Lỗi không tạo object, Undo unit, document revision hoặc xóa Redo.

Escape khi command đang nhập hủy command theo owner, gồm cả entry con. Không tự đổi thành cơ chế hai Escape để thoát entry rồi thoát command. Trường hợp completion popup idle có quy tắc Escape riêng đã được duyệt: Escape đầu đóng popup và giữ draft.

Escape ở đây là keyboard/control event. Chuỗi `Cancel` nhập vào numeric/point entry chỉ có nghĩa control nếu owner đã hỗ trợ token đó; nội dung `Cancel` trong Text/File vẫn là nội dung text, không tự xóa run bởi generic alias lookup.

## 8. Prompt và layout đích

Sử dụng bố cục Polyline: prompt nền + `(Tên=GiáTrị … Action)` + dấu nhắc + draft. Một ví dụ hình thức, giá trị do owner cung cấp:

```text
Start of polyline (PersistentClose=No Mode=Line Helpers=No):
Next point of polyline (PersistentClose=No Mode=Line Helpers=No Length=5 Undo):
Start of line (BothSides=No):
Center of circle (…các mode/flag hợp lệ…):
Turns <3>:
```

Không coi các ví dụ này là danh sách option mặc định của mọi stage. Đặc biệt `Length` của Line ở bước nào phải theo owner contract; không sao chép vào ví dụ Start of Line chỉ vì adapter hiện đang expose rộng.

Yêu cầu renderer:

1. Tạo text prompt và range của từng token từ cùng snapshot trong một lần format; trả về cả chuỗi lẫn span. Không dùng `find("Close")` trên chuỗi có `PersistentClose`.
2. Gạch chân đúng accelerator được record quảng bá, có thể là chữ giữa tên khi trùng initial. Dùng Unicode-safe byte/character conversion; không slice bừa `byte + 1` với chữ đa byte.
3. Click target bám đúng span token; stable widget ID chứa option + run; không chỉ dùng x/y cũ hoặc display value.
4. Không để hàng floating button cũ tồn tại đồng thời với inline prompt. Khi family chuyển xong, loại bỏ render path cũ của family đó qua contract chung.
5. Giữ một transcript/editor và một draft. History/prompt protected; Ctrl+A/C, selection kéo từ history xuống live row, paste và IME vẫn chạy.
6. Live row luôn pinned, không wrap; history scroll riêng. Prompt dài dùng horizontal scrolling của cùng dòng, caret/draft vẫn nhìn được; token ngoài viewport vẫn truy cập được bằng scroll. Không đẩy button sang tọa độ ngoài Command hoặc vẽ chồng toolbar.
7. Tôn trọng theme hiện hành trong mã và reference được duyệt. Không lấy mô tả màu cũ trong một skill để tự đổi theme green/gold đang chạy.
8. Sau activation thành công, refresh current prompt từ owner. Old history là lịch sử bất biến, không sửa ngược các dòng đã ghi. Nếu ghi lịch sử option, chỉ ghi một echo canonical và prompt tiếp theo theo quy ước hiện có.

## 9. Phím tắt, nhập tên và focus

Resolve theo thứ tự: full declared name/fixed choice → alias đã khai báo → accelerator duy nhất trong stage. Không dùng first-letter search trên mọi lệnh hoặc trên repeat slot. Nếu xung đột không giải được, không quảng bá một chữ sai; vẫn cho full name và click.

Giữ shortcut gốc đã chứng minh khi có. Có thể dùng chữ khác trong cùng tên để giải collision, nhưng phải xác định ổn định cho stage và gạch chân đúng chữ đó. Không dùng phím số để chặn nhập số/tọa độ: `2Point` không được nuốt số `2` của một input số. Không cấp accelerator C cho Cancel cạnh Center/Close; Escape đảm nhận Cancel.

M trong Polyline phải đi tới Mode thay vì Mid snap khi option accelerator hợp lệ. Khi draft không rỗng hoặc editor khác có focus, không áp accelerator và cũng không cho global snap nuốt chữ đang gõ. Tương tự E/End khi trùng option. Không thay global shortcuts ngoài phạm vi xung đột đã chứng minh.

Paste cả tên option không được kích hoạt trước Enter; IME preedit/commit và Enter xác nhận candidate không được chạy command/option. Một frame có Paste/IME hoặc multi-character text không được diễn giải như từng accelerator. Up/Down giữ history recall khi không có completion popup hợp lệ. Ctrl+A/C/V/Z trong editor giữ ý nghĩa chỉnh/copy text; không biến Ctrl+Z thành document Undo vô điều kiện.

## 10. Đơn vị và validation

Tách validation input với validation geometry cuối nhưng dùng cùng owner cho mọi route. Mọi số thực phải finite; count/index/degree phải là số nguyên thật, không cast/truncate từ float. Giữ dấu âm hợp lệ của Distance/Height theo từng owner; không áp luật “mọi số phải >0”.

Line lưu `length_mm` nhưng snapshot phải hiển thị theo unit đã chụp của session; ví dụ tài liệu inch, `Length=2.5` dựng đường 63.5 mm và vẫn hiển thị 2.5. Không hiển thị 63.5 rồi hiểu đó là inch. Radius/area/angle/vector/count có dimension khác nhau; không đưa tất cả qua length parser.

Không mở rộng hỗ trợ suffix unit cho một owner đang chỉ parse f64 bằng cách đoán. Trước tiên ghi gap, tra exact milestone/unit owner, rồi thêm conversion ở domain boundary nếu được contract yêu cầu. Kiểm conversion đúng một lần; giữ lựa chọn unit/session và World Point/CPlane conventions hiện có.

Giới hạn Number/Integer được mô tả trong OWNER-CARDS là mốc audit từ mã, không thay budget/tolerance của domain. Quan hệ `point_count > degree`, domain U/V của surface, top radius có thể bằng 0 hay không, amount âm… phải theo exact owner và solver. Không đặt một validator toàn cục có cùng min/max cho tất cả lệnh.

Text/File: giữ case, khoảng trắng, Unicode và đường dẫn có space. File pick/load chỉ xảy ra khi owner cần; không dùng generic option click để mở/truyền file không liên quan. Không để parse command alias chạy trên nội dung Text.

Giữ numeric locale theo protocol owner hiện hành; không tự thay dấu phẩy thành dấu chấm vì dấu phẩy còn ngăn tọa độ XYZ. Entry mới không được bypass input budget hiện tại của `submit_effect` (4096 byte UTF-8) hoặc allocation budgets của domain; kiểm text quá dài/Unicode và báo lỗi, không cắt cụt giá trị âm thầm.

## 11. Undo, preview và giao dịch

- Đổi option hoặc nhận point tạm không tự tạo document transaction. Preview có thể cập nhật, nhưng không được publish object thật để giả preview.
- Pending Undo/Undo option đi tới owner của command, xóa input/constraint tạm đúng semantics; không gọi document Undo để xóa một object cũ.
- Commit theo ranh giới của từng command: Polyline một wire trong một transaction khi finish; command multi-copy hoặc surface batch có quy tắc riêng đã tồn tại. Không ép mọi command một object hoặc một Undo nếu contract khác.
- Hủy/lỗi giữ document và Redo. Commit thành công phải kiểm actual geometry, scene, label/layer/selection và Undo/Redo, không chỉ có chuỗi success.
- Khi mode/value đổi, loại bỏ preview cache stale của đúng run; không xóa scene/document cache của command khác.

## 12. Ba ví dụ triển khai bắt buộc trước khi nhân rộng

### 12.1. Line → Polyline

1. Tạo Line thành công; repeat slot vẫn trỏ Line.
2. Mở Polyline bằng menu, lặp lại bằng CMD trong test khác.
3. Ở first point, snapshot/click target chỉ gồm P/M/H; không có BothSides hoặc Length.
4. Chọn first point: Length/Undo xuất hiện; BothSides vẫn vắng.
5. Mode Arc: Length vắng, Direction/Center có; C không nhầm Close.
6. Escape: quay Idle, danh sách active option rỗng. Line cũ và Undo count không bị xóa.

Giữ test `line_repeat_slot_never_leaks_options_into_polyline`; bổ sung stage Arc và stale run khi cần, không xóa test để làm migration qua.

### 12.2. Rectangle Square và Length

Square là toggle của owner Rectangle; S/click/full name phải đổi cùng một flag. `Length` chưa hợp lệ trước corner thì không expose như một option đang available. Sau corner, chọn Length → value-entry → nhập 5 → owner cập nhật width/height đúng mode hiện hành. `-5` báo lỗi, giữ entry/draft; không tạo Rectangle trước khi owner đủ input. Height ở một mode có thể commit khác mode khác: kiểm parser, không tự thêm Enter.

### 12.3. Helix Turns

Helix đang pending; snapshot có Turns=3 theo owner. T/click/`Turns` chọn cùng parameter và mở prompt nhập Turns với draft rỗng. Nhập 2 cập nhật Turns=2, trở lại stage trước. Axis và center/radius đã nhận không bị reset ngoài semantics owner. Zero/âm/NaN bị từ chối. Chọn Axis nhận vector hợp lệ qua đúng coordinate path; T không được coi là tọa độ hay command mới.

## 13. Cách chuyển từng owner, từng bước nhỏ

Mỗi family thực hiện cùng checklist sau:

1. Chụp baseline source/test/runtime; xác định feature IDs/aliases và cách gọi menu/CMD/toolbar đang có.
2. Liệt kê stage; ở mỗi stage ghi required input, parameter values, availability, default, finish/cancel/Undo và transaction boundary.
3. Đối chiếu token provider với parser. Option parser từ chối vì chưa đủ point thì provider không được quảng bá nó available. Parser hỗ trợ tham số thật nhưng metadata thiếu thì bổ sung có kiểm chứng.
4. Bổ sung typed snapshot owner. Getter/read snapshot không mutate owner, không dựng geometry, không gọi kernel.
5. Đưa activation/submit/pick qua owner contract; value-entry thuộc Controller/owner, không thêm `MatrixApp.current_radius` hay state song song trong egui memory.
6. Render bằng formatter/span chung. Loại bỏ nhánh UI per-command vừa thay; không xóa compatibility API còn có caller mà chưa migrate.
7. Kiểm cả ba route, các stage, invalid/retry/cancel, actual result và Undo. Ghi gaps rõ.
8. Hoàn tất một gói và record evidence rồi mới sang gói tiếp theo; không đánh dấu cả family done chỉ từ một toggle đầu tiên.

Nếu chỉ thay layout mà owner chưa hỗ trợ value-entry, đánh dấu `BRIDGE_ONLY`, không `PARITY_DONE`. Nếu solver thiếu, giữ `UNSUPPORTED`/reason và đưa ra blocker của capability đó; tiếp tục gói độc lập được phép, không giả thuật toán.

## 14. Acceptance và bằng chứng

`ACCEPTANCE.csv` là các ca chuẩn dùng lại, không phải log đã chạy. Mỗi ca phải được instantiate với owner/stage/input cụ thể. Với mỗi parameter available, kiểm keyboard, pointer, typed full name; kiểm `Name=value` khi parser hỗ trợ. Count scenario khác count test function và count command.

Test phải khẳng định owner state/geometry/Undo/focus/draft, không chỉ khẳng định label bằng chuỗi hardcode. State fixture chỉ chứng minh metadata; ít nhất một geometry/commit test phù hợp cho mỗi command được tuyên bố migrated. Các mode đặc biệt cần test riêng; Center circle không chứng minh Tangent/AroundCurve/FitPoints.

Native Windows: kiểm đúng PID/path/hash/profile của executable mới, menu và CMD, inline click, shortcut, invalid/retry, cancel và geometry phù hợp. Chụp ảnh own app vào `implementation/evidence/...`, có provenance. Không lấy screenshot người dùng hoặc ảnh commercial làm evidence runtime authored. Không điều khiển/đóng phiên người dùng có bản vẽ chưa lưu.

Lệnh kiểm tra mẫu, sau khi resolve đúng target:

```powershell
# Mọi shell command phải bắt đầu bằng rtk. Set CARGO_TARGET_DIR qua env của runner.
rtk proxy cargo test -p om9-app --test command_options_interface -j 2
rtk proxy cargo test -p om9-app --test polyline_options_interface --test polyline_mode_regression --test pinned_command -j 2
rtk proxy cargo test -p om9-app --test <test-owner-thuc-te> -j 2
rtk proxy cargo clippy -p om9-app --lib -j 2 -- -D warnings
rtk proxy cargo build -p om9-app --bin om9 -j 2
rtk proxy python tools/public_source_audit.py
rtk git diff --check
rtk proxy python tools/prune_build_artifacts.py --apply
```

`<test-owner-thuc-te>` là chỗ cần thay bằng test tồn tại, không phải lệnh copy nguyên. `CARGO_TARGET_DIR=L:/om9-target`, không build trong checkout, không tạo lại `standalone/`. Chỉ fmt file/chunk đang sửa; giữ dirty changes có sẵn. Native adapter mới nằm ngoài scope mặc định; nếu cần phải có constraint và validation tương ứng, không tự chọn C++/Python cho logic session.

Report binary cũ/mới rõ: nếu chỉ debug mới mà release vẫn cũ thì ghi đúng, không nói “bản người dùng đang dùng đã cập nhật”. Không thay/force-close binary đang có user document để build đè. Không publish/commit/stage ngoài yêu cầu.

## 15. Điều kiện hoàn tất toàn bộ yêu cầu

Một hàng command chỉ `PARITY_DONE` khi có: phân loại/owner, stage table đầy đủ, metadata/value/shortcut chính xác, value-entry phù hợp, ba route tương đương, không rò owner/run, unit/validation đúng, commit/cancel/Undo và evidence đúng binary. Immediate/Specialized/Unsupported dùng trạng thái kết thúc riêng có lý do; không giả `PARITY_DONE` cho code không chạy.

Toàn chương trình chỉ có thể được báo hoàn tất khi mọi entry point đã reconcile và mọi hàng có trạng thái kết thúc có bằng chứng, không còn unknown bị bỏ quên. Các blocker geometry ngoài scope, test baseline hỏng, test ignore và manual gate chưa ký phải được báo riêng; không biến số test pass thành phần trăm xong của tất cả lệnh.

Ghi vào `docs/openmatrix9-progress.json`, runtime_implementation của exact milestone owner, README nhóm/root và report. Giữ planning/source content và ID cũ. Khi cần thay nghiệp vụ ngoài chuẩn tương tác đã yêu cầu, ghi before/after và đề xuất skill sau khi code/verify; chỉ cập nhật skill khi có consent đúng thay đổi. Tài liệu này tự nó không sửa skill và không ký M01-UI/M03-UI.
