---
id: OM9-MATRIXTOOLS-008
name: View Manager
command: gvViewManager
domain: 16-matrix-tools
module: Matrix Tools
kind: manager
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-MATRIXTOOLS-008 — View Manager

Alias tương thích: `gvViewManager`. Nhóm: `16-matrix-tools`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

View Manager lưu camera và mức zoom để gọi lại. AddCurrentView chụp trạng thái view đang hoạt động và đặt tên; DeleteView xóa saved view; LoadView nhận tên hoặc lựa chọn saved view rồi áp dụng camera vào viewport đích theo policy Match Name/Fit Match/Current Viewport. Khi chưa có saved view, không đưa người dùng vào luồng load rỗng. Tên trùng, overwrite, projection và dữ liệu camera không hợp lệ cần quy tắc host riêng.

Saved view cần chứa projection/camera transform/target/zoom cùng schema version và units khi có khoảng cách. Nạp chỉ đổi view state, không biến đổi object geometry. Cancel ở bước nhập tên/chọn view giữ camera và thư viện. Định dạng lưu dùng schema OpenMatrix9, không giả lập thư viện renderer hoặc yêu cầu file binary thương mại.

### Chi tiết thao tác qua command và saved-view store

Các chi tiết dưới đây đọc được trong thân managed; chưa chạy Rhino hoặc native
acceptance. Command và panel ViewManager có các đường tương tác khác nhau.
Command luôn đưa AddCurrentView/DeleteView vào lựa chọn, chỉ thêm LoadView khi
store có NamedViews, và nhận tên bằng GetString. Panel có danh sách saved views,
nút Save/Delete và lựa chọn viewport đích; danh sách được sắp theo friendly name.

Save từ command truyền active view cho helper. Helper tạo NamedView trong document
qua RhinoScript, lấy ViewInfo tương ứng rồi thêm vào archive 3DM phiên bản 5.
Metadata trong dictionary của layer đầu lưu tên hiển thị, tên viewport nguồn và
thumbnail. Thumbnail 35×35 được chụp từ active view bằng mode Shaded; tham số
ViewInfo của hàm thumbnail chưa được dùng để render một camera riêng. Sau khi ghi
store, helper xóa NamedView vừa tra trong document, đồng bộ danh sách và refresh UI.
Không có cleanup bảo đảm khi lỗi giữa chuỗi thao tác; không suy tên tạm sẽ an toàn
khi trùng NamedView đã có trong document, hoặc archive đã lưu đủ renderer state.

Kiểm tên trùng ở danh sách managed không phân biệt hoa/thường. Khi Save từ nút UI,
helper hỏi overwrite và chỉ tiếp tục sau Yes, đồng thời gọi nhánh Delete trước.
Command truyền bScript=true nên bỏ qua cả câu hỏi và nhánh Delete này, rồi tiếp
tục thêm entry. Vì thao tác NamedView/archive bên dưới còn do API native xử lý,
không kết luận đường command có cùng collision/overwrite hoặc dedup như nút UI.
OM9 phải công bố collision policy riêng và dùng identity bền thay tên làm khóa.

Panel khởi tạo lựa chọn đích là Match Name. Match Name tìm viewport model có tên
khớp tên nguồn đã lưu, không phân biệt hoa/thường; không có thì dùng active view.
Fit Match cũng thử tên trước, sau đó tìm view perspective hoặc cùng nhóm hướng
camera theo các trục ±X/±Y/±Z; không có thì dùng active view. Current Viewport dùng
active view trực tiếp. Vì vậy source Save là active view, còn đích Load có thể là
view khác. OM9 hiển thị đích đã resolve trước khi Apply và kiểm lifetime/revision.

Apply tìm saved view theo tên rồi nạp ViewInfo vào viewport đích bằng API có cờ
includeTraceImage=false, sau đó phục hồi tên cũ của viewport đó. Giá trị bool trả
về của API chưa được kiểm tra. Nhánh tìm thấy view return trước đoạn cuối đặt
active view và gọi redraw, nên thân helper chưa bảo đảm cùng một đường focus/redraw
cho mọi kết quả; không suy native đã apply thành công chỉ vì helper trả về.
Biến Perspective/fallback-active riêng của command chỉ giữ/khôi phục tên, không
thay source Save hay quy tắc chọn đích trong helper.

Store thiếu có thể được tạo trước khi chọn action. LoadViews đọc NamedViews cùng
metadata/thumbnail, dựng lại danh sách rồi ghi archive; bước đồng bộ sau đó còn
ghi archive lần nữa. Đồng bộ loại NamedViews không còn trong danh sách managed
và xóa start-section comments. Delete theo tên xóa entry trong danh sách rồi gọi
đồng bộ; tên không có chỉ return. Chưa thấy đồng bộ xóa metadata dictionary dư.
Do đó luồng đọc/list gốc không phải read-only và Cancel không bảo đảm chưa có ghi file.

Write không được kiểm tra bool trong các helper này. Command cũng bỏ qua bool
của Save, còn Delete/Apply không báo lỗi khi tên không tồn tại; command có thể
trả Success dù không đạt tác động được yêu cầu. OM9 chọn policy rõ: đọc/list không
ghi store; Save/Delete ghi nguyên tử sau xác nhận; kiểm kết quả I/O/apply trước
completion; cleanup NamedView tạm và phục hồi state khi lỗi/cancel. Đây là yêu cầu
OM9 cần nghiệm thu, không phải bảo đảm đã thấy trong luồng gốc.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `active_viewport` | `viewport` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Action` | `choice` | AddCurrentView/DeleteView/LoadView. | Chưa xác định; không tự điền. |
| `Name` | `text` | Tên khi thêm hoặc chọn saved view. | Chưa xác định; không tự điền. |
| `SavedView` | `reference` | Identity view khi xóa/nạp. | Chưa xác định; không tự điền. |

## Hướng dẫn triển khai

1. Rust giữ action cùng camera snapshot có kiểu; validate danh sách rỗng, tên và state hữu hạn trước host. Phân biệt save/delete I/O với load camera.
2. C++/Qt lấy camera của viewport active, serialize qua OM9 view-store adapter. Ghi cập nhật store nguyên tử; xóa đúng identity, không đổi geometry document.
3. Load resolve viewport đang còn sống, kiểm tra projection và camera frame rồi áp dụng toàn bộ trạng thái; lỗi/cancel giữ camera cũ. Capture không được nhầm view perspective phụ với active view đang lưu.
4. Đăng ký command/menu/CMD trong C++ native; Rust giữ request/state. Python đăng ký workbench và fixture. Giữ nhánh thiếu adapter disabled/unsupported.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-MATRIXTOOLS-008",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Action", kind: ParameterKind::Choice, required: true },
        Parameter { name: "Name", kind: ParameterKind::Text, required: false },
        Parameter { name: "SavedView", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Lưu hai camera với zoom/projection khác nhau rồi Load khôi phục đúng viewport đích theo Match Name/Fit Match/Current Viewport và fallback đã công bố.
- Store rỗng không cho Load; Cancel/name collision/file corruption giữ camera/store theo policy.
- Delete đúng view, saved views đọc lại sau restart; object placement/geometry không đổi.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

Supported slice dưới đây là hợp đồng để triển khai; trạng thái native giữ nguyên
và các ca runtime vẫn cần nghiệm thu. Mẫu Rust chỉ phân loại ViewState; save/delete
store, metadata và cleanup tạm cần adapter riêng, chưa được mẫu đó bảo đảm.

### Inputs

Active viewport snapshot cho Save; action, saved-view identity/name, store revision
và target policy cho Load. Target resolver giữ viewport ID/generation, tên nguồn
và loại projection/hướng camera để chọn đích đúng Match Name/Fit Match/Current.

### Parameters and defaults

Add/Delete luôn có ở command path; Load chỉ có khi store không rỗng. Panel gốc
khởi tạo Match Name; Fit Match thử tên rồi loại view/hướng trục, Current dùng active;
các nhánh không tìm được match fallback active. UI Save hỏi overwrite, command Save
bỏ nhánh này với bScript=true. Collision/overwrite và schema camera của OM9 phải
công bố riêng, không lấy khác biệt giữa hai entrypoint làm default ngầm.

### Output

Save/Delete thay store; Load thay view đích đã resolve, giữ tên viewport và object
geometry. Schema OM9 lưu camera/projection/target/zoom, tên nguồn, thumbnail tùy chọn
và identity. Luồng gốc dùng NamedViews, dictionary metadata và thumbnail Shaded;
không có bằng chứng runtime rằng mọi renderer property đã được lưu và phục hồi.

### Preview / commit / cancel

Theo policy OM9, read/list không ghi store; Save/Delete staging và ghi nguyên tử
sau xác nhận. Capture không đè NamedView hiện có; cleanup dữ liệu tạm cả khi lỗi.
Apply validate toàn bộ snapshot trước mutation, giữ tên và refresh viewport đích
sau khi API báo thành công. Cancel/lỗi giữ camera/store theo checkpoint. Luồng gốc
có thể tạo store trước action, ghi hai lần khi LoadViews/đồng bộ và thiếu cleanup
bảo đảm, nên không dùng nó làm bằng chứng transaction/cancel đã đúng.

### History / dependency model

Saved-view store có schema version và identity riêng; camera history khác document
geometry Undo. Tên nguồn và thumbnail là metadata cần đi cùng view; xóa view phải
dọn metadata theo identity. Việc helper gốc dùng NamedView tạm trong document
không chứng minh Undo hoặc phục hồi đúng khi trùng tên/lỗi giữa chừng.

### Error / invalid-input behavior

Store hỏng, tên thiếu/trùng, projection invalid, resource thiếu hoặc viewport đã
đóng phải báo lỗi và giữ state cũ. Không dùng command Success thay cho kết quả
Write/Apply. Kiểm riêng: Save UI/command cùng tên, Load đủ ba target modes, list
không ghi file theo policy OM9, lỗi sau tạo NamedView tạm, xóa metadata và focus/redraw
của viewport đích. Các ca này chưa chạy.

## Acceptance checklist

- [ ] Hợp đồng feature và điều kiện hình học được kiểm chứng trước khi triển khai.
- [ ] Inputs and selection state documented.
- [ ] Parameters/defaults/units documented.
- [ ] Output geometry documented.
- [ ] Preview/commit/cancel behavior tested if applicable.
- [ ] History/dependencies tested if applicable.
- [ ] Undo/redo tested.
- [ ] Save/reload persistence tested.
- [ ] Automated tests use feature ID `OM9-MATRIXTOOLS-008`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-MATRIXTOOLS-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-MATRIXTOOLS-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-MATRIXTOOLS-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-MATRIXTOOLS-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-MATRIXTOOLS-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-MATRIXTOOLS-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-MATRIXTOOLS-008` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
