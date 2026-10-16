> **Current target: specs v2 (2026-10-13).** [OM9-SOLID-012](../../OpenMatrix9_Codex_Spec_v2/specs/04-solid/om9-solid-012-box.md)
> Use standalone OM9 and Matrix9 UI contract for new work. The record below is legacy FreeCAD/source evidence; its v1 links, native passes and host limitations do not certify the new runtime.

# OM9-SOLID-012 — Box

<!-- box-default-video:start -->
2026-10-12, người dùng làm rõ video minh họa **Box mặc định**, không phải một trong bốn option. Box mở ngay ở First corner; mặc định hai góc đáy đối diện rồi Height (3 lần chọn). Diagonal/3Point/Vertical/Center là lựa chọn tùy chọn. 3Point riêng vẫn là hai đầu cạnh rồi Width/Height. Quy tắc này thay thế yêu cầu chọn phương pháp bắt buộc trước đó.

[Bằng chứng mặc định và preview theo video](../validation/2026-10-12-box-default-video.md).
<!-- box-default-video:end -->


<!-- box-documented-workflow-20261012:start -->
2026-10-12, sửa theo yêu cầu người dùng: Box bắt buộc chọn phương pháp; bỏ nhánh
Box thường/Corners. 3Point và Vertical nhận hai điểm đầu cạnh rồi Width/Height,
không nhận chiều dài thay điểm cuối. Center nhận tâm rồi góc hoặc Length toàn phần.
`0` chỉ là điểm gốc sau khi chọn phương pháp; Width/Height vẫn có Enter default.
Điều này thay thế các ghi chú Corners/default và Length ở endpoint bên dưới.
[Bằng chứng Rust/native](../validation/2026-10-12-box-documented-workflow.md).
<!-- box-documented-workflow-20261012:end -->

<!-- command-mnemonics-20261012:start -->
2026-10-12: chỉ gạch chân một ký tự phím tắt đã khai báo, không gạch chân cả
tên. Rust cung cấp mnemonic; cả tên vẫn click được, clipboard giữ nguyên.
Box dùng D/3/V/C; phím3 chỉ chọn3Point trước điểm đầu, sau đó vẫn nhập số3.
Focus,0 origin và lịch sử dễ đọc đã sửa vẫn giữ. Supported status còn partial.
[Bằng chứng native](../validation/2026-10-12-command-mnemonics.md).
<!-- command-mnemonics-20261012:end -->

<!-- box-command-usability-20261012:start -->
2026-10-12: Box icon trái/phải focus CMD; `0` ở bước nhập điểm là gốc CPlane
và chuyển vào lịch sử khi được nhận. Width/Height giải thích Enter/default mm.
Lỗi ghi riêng vào lịch sử, prompt sạch và draft sai vẫn sửa được; giới hạn
geometry giữ nguyên. Focus có guard generation sau cancel; common Curve input
evidence giữ điểm gốc. Đây là sửa supported slice, status vẫn partial.
[Bằng chứng native và giới hạn](../validation/2026-10-12-box-command-usability.md).
<!-- box-command-usability-20261012:end -->

<!-- matrix-phase-options-20261012:start -->
2026-10-12: engine Rust chung có question/options theo phase, span UTF-16,
ẩn/hiện độc lập gate và revision refresh. Box áp dụng first-corner → Diagonal
Cube → Width/Height defaults; click và D/d dùng cùng native solver. CMD xanh ghi
invocation thật, preceding prompt và input đã nhận; Up/Down recall, Esc và draft
lỗi giữ đúng lifecycle. Đây là supported slice, không hoàn tất mọi command,
Gem Builder hay associative History.
[API dùng chung](../command-options-adapter.md) · [Bằng chứng native](../validation/2026-10-12-box-matrix-command-options.md).
<!-- matrix-phase-options-20261012:end -->

<!-- rhino-core-extended-input:start -->
## Extended numeric input — validated slice

Extended typed point input through existing native geometry; no all-option or all-P0 completion. Legacy Solid mm/cm/in callback remains; actual accepted relative base and Undo are repaired.

[Runtime/source evidence](../validation/2026-10-09-rhino-core-extended-input.md). Native geometry is reused; live constraints, locale and other missing contracts remain open.

<!-- rhino-core-extended-input:end -->

Native Diagonal/3Point/Vertical/Center and oriented Cube Box; required method selection removes implicit Corners. 3Point/Vertical require two edge points then pick/type Width and Height; Center accepts center then corner or full Length. Enter defaults, signed extents, frozen CPlane, CMD/mouse, right-click shortcut, preview and persistence. Single-character D/3/V/C links, native focus and accepted-input history retained. Associative History/layer/Styles integration remains unverified.

[Contract, options and limits](../../OpenMatrix9_Codex_Spec_v1/specs/04-solid/om9-solid-012-box.md).

[Native validation](../validation/2026-10-09-solid-box-sphere-options.md).

Rust owns ordered state and construction math; C++/Qt owns native references, Part/OCCT geometry, scene preview and one transaction. Snapshot results preserve input geometry and persist through native Undo/Redo and FCStd reload.

## Earlier source evidence

Matrix 8 Book 1, printed pp.222–223 / PDF pp.232–233, was extracted and visually checked in the 2026-10-06 core implementation. The current normalized spec engineering contract is preserved; host solver choices and remaining compatibility limits are separated in its implementation notes. Original reference inputs remain unchanged.
