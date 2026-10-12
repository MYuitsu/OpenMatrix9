---
id: OM9-BUILDERCORE-001
name: Builder Framework
command: null
domain: 01-core
module: Builders vs. Commands
kind: builder
implementation_status: partially_implemented
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
implementation_validation_status: validated_supported_slice
implementation_verified_at: '2026-10-09'
implementation_validation_record: ../../../docs/validation/2026-10-09-reusable-builder-cage.md
---

# OM9-BUILDERCORE-001 — Builder Framework

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

Alias tương thích: `Chưa có alias command`. Nhóm: `01-core`. Trạng thái: `partially_implemented`.

## Hợp đồng thiết kế

**Hợp đồng thiết kế `OM9-BUILDERCORE-001` — Builder Framework.**

Builder là phiên có UI riêng, In preview lọc type input, Options/Mode, profile preview, graphic controls, slider và viewport handles cùng CMD. In nhận selection đúng loại hoặc F6 tự nạp; Builder không có input dùng Start. Mode đổi loại tạo và highlight lựa chọn. Slider cho đơn vị mm, degree hoặc percent tùy control; Shift drag bước 0.5, +/- bước 0.10 theo tương thích được mô tả. Nhập số trực tiếp thay kéo. Handles hover hiện tên: arrow scale, cube position, orb vị trí trên curve, orb+halo rotation; chữ/symbol còn tùy Builder. Profile preview mở browser, hover xem tạm, click áp; Edit mở editor profile riêng với curve/snaps/grid. Reset trở về defaults của mode đang chọn; Enter commit và bỏ handles/preview, Esc hủy, X đóng UI. Options CMD và UI có phần riêng nên không coi chúng luôn tương đương.

<!-- builder-command-line-options-contract-20261012:start -->
<a id="command-line-options"></a>

### Command Line Options và đồng bộ CMD–Builder — 2026-10-12

Bổ sung theo yêu cầu người dùng và ảnh tham chiếu Command Line Options.
Áp dụng [hợp đồng chung RCORE-04](../00-rhino-core/04-command-engine.md#command-line-options)
cho cùng một Builder session: toggle, tùy chọn con, nhập số và controls đều
đi qua state/validation chung. Đây là **hợp đồng mục tiêu chưa nghiệm thu
native**, không mở rộng supported slice BuilderRecord đã kiểm chứng bên dưới.
Các chi tiết về routing/draft/Enter/Esc là quyết định OM9; defaults và cách
dựng geometry vẫn do spec từng Builder xác lập, không suy từ ảnh.

#### Schema và binding của từng Builder

Rust sở hữu schema và accepted parameter state theo exact feature ID. Mỗi
option có key ổn định, kiểu thao tác, aliases/mnemonic, current value, parent
và children nếu có, phase/gate, units/ranges/dependencies cho số và chính sách
lưu giá trị. Qt control/slider/handle và CMD chỉ là các đường nhập tới state đó.

| Phạm vi option | Hợp đồng binding |
|---|---|
| CMD và Builder | Hai phía map cùng option key, đọc cùng current value và gate; cùng input hợp lệ cho cùng parameters và preview. |
| Chỉ CMD | Có trong live prompt khi đúng phase; vẫn cập nhật shared state/preview và gates của controls liên quan, không bắt buộc tạo control riêng. |
| Chỉ Builder | Có trong controls/Options List khi đúng phase; dùng cùng validation/session, không tự tạo alias hoặc option CMD chưa được feature khai báo. |

Tập option không phải bản sao bắt buộc giữa hai phía. Chỉ các mappings đã
được feature khai báo mới được dùng; tên giống nhau chưa đủ để nối hai controls.
Nhánh chưa có adapter/solver phải disabled có lý do ở nơi được hiển thị.
`Mode`, `Reset` và `Close` phải giữ kiểu/ý nghĩa của Builder cụ thể; không
dùng quy tắc action Close của Polyline cho một toggle Close của Builder.

#### Các thao tác bắt buộc

1. **Toggle:** click CMD hoặc control đổi đúng một lần giữa hai trạng thái đã
   khai báo. Nhập giá trị tường minh đặt giá trị đó, không đảo thêm. CMD label,
   check state và preview cùng cập nhật; snapshot refresh không phát lại click.
2. **Tùy chọn con:** click option cha mở các choices/children hợp lệ và hiện
   lựa chọn hiện hành. Mở/hover không tự nhận giá trị; click hoặc nhập child
   chọn nhánh. Child cần số/cấp con tiếp tục chờ đúng phase. Esc đóng từng
   cấp và giữ accepted value; không thay bằng lựa chọn đầu danh sách.
3. **Nhập số:** click option CMD mở prompt có units, current value và constraints;
   nhập số rồi Enter chỉ nhận parameter, cập nhật preview và quay về prompt cha.
   Ô số Builder dùng cùng parser/validator; raw draft tách khỏi giá trị đã nhận.
   Sai grammar/units/range hoặc NaN/Infinity giữ giá trị/preview cũ, giữ draft
   và báo lỗi. Enter rỗng theo current/default đã công bố; Esc bỏ draft giữ
   accepted value. Không suy Roll=0 hoặc các trạng thái Off trong ảnh là defaults.
4. **Đồng bộ:** mỗi accepted event tạo cùng option revision cho CMD, controls,
   dependent gates và preview. UI refresh không tạo event/transaction lần hai.
   Khi CMD có draft cho option, sửa chính option bằng Builder bị chặn với
   feedback tới khi nhận/hủy draft; không âm thầm ghi đè. Đổi option khác có
   thể làm draft mất điều kiện: giữ draft và validation lại trước khi nhận.

Ví dụ tên GemShape/PlacementMethod có thể mở lựa chọn, Flip/Mirror có thể
là toggle, Roll có thể nhận số; mỗi mapping phải lấy từ schema/spec của
exact command. Đây là ví dụ tương tác, không xác nhận `gvGemOnCrvAdvanced`
đã hỗ trợ, không bổ sung defaults và không thay solver của feature đó.

#### Preview, Mode/Reset và kết thúc

Thay toggle/choice/number chỉ đổi parameters của session và preview tạm;
không tạo document object hoặc Undo document. Mode/Reset đi qua cùng reducer,
cập nhật values/choices/gates ở hai phía theo defaults đã xác lập của mode;
option/draft mất điều kiện phải được xử lý có feedback, không rơi về mode
hay giá trị khác âm thầm. Slider/handle dùng cùng units/ranges và shared
parameters; không được giữ một bản accepted value riêng với CMD.

Mở/đóng panel không sinh session mới hoặc reset accepted values. Phân biệt
đóng/ẩn panel để tiếp tục pick với Cancel hoặc thao tác đóng phiên do feature
khai báo: chuyển panel→pick→panel giữ state/focus. X chỉ đóng UI thì không
được coi là Cancel; nếu feature xác lập X đóng cả phiên thì teardown như Cancel.
Khi đang ở choice/number, Enter nhận input của phase đó; chỉ Enter tại prompt
chính hoặc action Commit hợp lệ mới chốt toàn Builder trong một transaction.
Geometry, parameters và metadata/History được ghi theo scope từng feature;
Esc ở prompt chính, Cancel/đóng phiên, document-close hoặc deactivate dọn preview,
draft và callbacks. Callback/preview cũ phải bị loại theo generation/revision.

Nghiệm thu phải dùng một Builder có controls và command được hỗ trợ thật.
API tạo BuilderRecord hoặc fixture schema không thay bằng chứng native cho
sliders, Options List, lựa chọn con và đồng bộ CMD–Builder.
<!-- builder-command-line-options-contract-20261012:end -->

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `active_document` | `document` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Mode` | `choice` | Loại kết quả do Builder riêng khai báo | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `ControlValue` | `number` | Giá trị trong đơn vị control | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Action` | `choice` | Start, Reset, Commit hoặc Cancel | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |

## Hướng dẫn triển khai

1. Rust: khai báo state/operation plan theo `OM9-BUILDERCORE-001`, kiểm tra phase selection, parameter kinds/units và capability; mọi số cần hữu hạn.
2. C++/Qt–FreeCAD: Rust state machine phân biệt Idle/Input/Preview/EditingProfile/Commit; mỗi Builder khai báo schema riêng và profile asset do OM9 sở hữu; Qt handles/slider đồng bộ qua state events; Part commit đúng transaction.
3. Tích hợp `Builder Framework` ở native command router; UI/CMD cùng handler. Python hỗ trợ workbench/fixture và native runtime checks theo ID, không đăng ký command trùng.
4. Tách preview/report/view state khỏi geometry document; commit transaction chỉ cho tác động được yêu cầu, cleanup event handlers khi panel/document đóng; serializer giữ identity và fields đúng scope.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-BUILDERCORE-001",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "ControlValue", kind: ParameterKind::Number, required: false },
        Parameter { name: "Action", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Sai type không khởi tạo preview; cùng giá trị nhập số và handle cho cùng geometry; hover profile không commit; Enter một transaction, Esc không để geometry tạm.
- Hủy panel/phiên giữ dữ liệu trước Apply; lỗi input/adapter không ghi kết quả một phần. Kiểm tra NaN/Infinity và document switch/close; không giữ callbacks tới object đã xóa.
- Đối với thay document: Undo/Redo và save/reload giữ kết quả cùng metadata/links. Đối với view/read-only: geometry/count không đổi. Đối với file output: path/format lỗi không ghi file hỏng hoặc đổi model ngoài yêu cầu.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

<!-- builder-command-line-options-fixtures-20261012:start -->
### Nghiệm thu Command Line Options — đã kiểm supported slice

Các tên dưới đây là fixture của `OM9-BUILDERCORE-001`, không phải feature IDs
mới. Ghi thêm exact ID của Builder consumer và schema/options đã kiểm;
áp dụng [RCORE-04.T14–T23](../00-rhino-core/04-command-engine.md#command-line-options-fixtures).

| Fixture | Điều kiện đạt |
|---|---|
| OM9-BUILDERCORE-001.CMD-T01 | Click/tên/mnemonic/explicit value và control cho cùng toggle state; hai click khôi phục state; update widget không phát toggle lần hai. |
| OM9-BUILDERCORE-001.CMD-T02 | Parent mở choices không đổi parameter; child click/typed/numeric và Esc từng cấp đúng phase, không auto-select; unsupported child bị chặn có lý do. |
| OM9-BUILDERCORE-001.CMD-T03 | CMD nhập số+Enter và ô số/slider/handle với cùng quantity cho cùng parameters và native preview; sai input giữ state cũ; Enter nhận số không commit toàn Builder. |
| OM9-BUILDERCORE-001.CMD-T04 | Shared/CMD-only/Builder-only giữ đúng visibility, identity và gates; thay parameter shared đồng bộ hai phía; draft đang nhập được bảo vệ và không có feedback loop. |
| OM9-BUILDERCORE-001.CMD-T05 | Mode/Reset đổi values/choices/dependencies ở hai phía; draft invalid báo rõ; panel→pick→panel giữ session; preview từ generation/revision cũ bị loại. |
| OM9-BUILDERCORE-001.CMD-T06 | Enter/Commit tại phase chính tạo một transaction geometry+parameters+metadata theo consumer; Undo/Redo và save/reload đúng scope; các thay đổi option trước commit không tạo document object. |
| OM9-BUILDERCORE-001.CMD-T07 | Esc từng cấp giữ accepted value; Esc chính/Cancel/đóng phiên/document-close/deactivate hủy và dọn callbacks/preview; X theo scope đã khai báo của feature; không late commit hoặc successful history giả. |

- [x] Toggle và explicit set được kiểm chứng với native Loft/Rebuild và các shared controls.
- [x] Tùy chọn con và nhập số/units/errors/Enter/Esc được kiểm chứng trên native consumer hiện có.
- [ ] Binding CMD–Builder, visibility riêng và Mode/Reset được kiểm chứng native.
- [ ] Preview/commit/cancel, draft protection và stale-result rejection được kiểm chứng native.
<!-- builder-command-line-options-fixtures-20261012:end -->

<!-- command-options-implementation-20261012:start -->
Kết quả cuối: 30 reducer/ABI tests; 3 native fixtures mới 26+25+15 checks.
CMD–Builder shared controls đã đồng bộ hai chiều; các checkbox nghiệm thu rộng
bao gồm private UI/Reset/exhaustive stale lifecycle vẫn giữ chưa đánh dấu.

2026-10-12: common Rust schema/reducer + Qt binder đã có cho native Rebuild
và Sweep/Loft scalar controls. Lệnh khác khai báo schema, current values,
gates và adapter; không thêm tên lệnh vào reducer. Native input identity
controls giữ guards hiện có. Đây là supported slice của Builder Framework;
private visibility, Reset/profile/handles và Gem geometry solvers vẫn chưa
được claim hoàn tất native.
[Adapter API](../../../docs/command-options-adapter.md) ·
[Bằng chứng và giới hạn](../../../docs/validation/2026-10-12-command-line-options.md).
<!-- command-options-implementation-20261012:end -->

## OpenMatrix9 implementation notes

**Verified:** 2026-10-09. **Status:** partially_implemented; validated_supported_slice.

### Native API foundation

`OpenMatrix9Gui.createBuilderRecord(gem, featureId, parametersJson, seedObjects, scaleToGem=False)` copies same-document native Part seed geometry into a durable gem-local affine template. It preserves the inputs and returns a native BuilderRecord whose OutputNames identify separate BuilderOutput slots. Multiple independent records can belong to one gem. `restoreBuilderOutputs(record)` explicitly recreates missing owned slots; deleting output geometry does not trigger implicit resurrection, and occupied unrelated names reject.

The registered recipe is schema 1, evaluator `om9.affine-template`, version 1. Raw settings JSON is retained; it is metadata, not an arbitrary setting/cutter geometry solver. Affine scale/offset and optional current/initial gem dimension ratios evaluate the copied templates in current global gem frames. Rust validates schema/version, finite values, positive dimensions, feature identity and replay budgets. C++ is required for FreeCAD native document properties/transactions/signals/persistence and OCCT seed/result geometry. Rust never retains document/kernel pointers.

Global Record/Update/Lock/Clear integrates with durable output links. Under the RCORE-09 policy revision (2026-10-09), Record controls new links only: new outputs created with Record Off remain unrecorded, while existing recorded outputs update whenever Update is On. Update independently suspends/resumes existing links, independent output edits detach, and Undo/Redo/FCStd preserve the declared record/output scope. The original source contract above remains unchanged. This is API-only infrastructure: original per-builder commands, arbitrary Gem solvers, In/Start/Mode UI, sliders, handles, profile editor, Styles selection and .mss interchange remain disabled or unsupported. See [Builder API contract](../../../docs/features/builder-history-native-contract.md).

[Matching native validation](../../../docs/validation/2026-10-09-reusable-builder-cage.md) records the accepted API/command scope and limits. Original design requirements above remain broader than this implemented slice.

## Acceptance checklist

- [ ] Hợp đồng feature và điều kiện hình học được kiểm chứng trước khi triển khai.
- [ ] Inputs and selection state documented.
- [ ] Parameters/defaults/units documented.
- [ ] Output geometry documented.
- [ ] Preview/commit/cancel behavior tested if applicable.
- [ ] History/dependencies tested if applicable.
- [ ] Undo/redo tested.
- [ ] Save/reload persistence tested.
- [ ] Automated tests use feature ID `OM9-BUILDERCORE-001`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-BUILDERCORE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-BUILDERCORE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-BUILDERCORE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-VIEW](../../ENGINEERING_CONTRACTS.md#om9-view): kiểm chứng cho `OM9-BUILDERCORE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-DISPLAY](../../ENGINEERING_CONTRACTS.md#om9-display): kiểm chứng cho `OM9-BUILDERCORE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-BUILDERCORE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-BUILDERCORE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-BUILDERCORE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANNOTATE](../../ENGINEERING_CONTRACTS.md#om9-annotate): kiểm chứng cho `OM9-BUILDERCORE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-BUILDERCORE-001` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
