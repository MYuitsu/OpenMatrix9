# OM9-BUILDERCORE-001 — shared Command Options

<!-- common-command-mnemonics-20261012:start -->
2026-10-12: quy tắc gạch chân một ký tự áp dụng qua engine Rust chung cho toàn
bộ schema Command Options hiện có: Line/Polyline, InterpCrv, Rectangle, Circle,
Ellipse, Rebuild, Box/Sphere, Sweep1/History, Sweep2/History và Loft. Tùy chọn con
khai báo alias trong phase cha; tên đầy đủ vẫn click được. Alias trùng hoặc
disabled không được quảng cáo bằng gạch chân. Lệnh mới đăng ký schema để dùng
cùng resolver, renderer và binding CMD–Builder. Trạng thái toàn feature còn partial.
[Bằng chứng native](../validation/2026-10-12-common-command-mnemonics.md).
<!-- common-command-mnemonics-20261012:end -->

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

Status: partially implemented. The existing BuilderRecord/History foundation
is extended by a common Rust option model and native Qt bindings.

Rebuild and Sweep/Loft scalar controls share accepted state with CMD. The
generic binder handles checkbox toggles, choices and raw numeric controls,
blocks signal loops, preserves invalid edits, and rejects a Builder change to
the same unresolved CMD draft. Declared choice children drive control
availability. Native builders validate a temporary result before accepting
parameters and replacing preview.

Enter in a numeric field accepts that value without clicking the dialog's
default OK button. Esc cancels the pending level or raw edit while keeping
accepted state. Commit remains blocked by unresolved CMD or numeric widget
drafts. Existing native selection/source, document lifecycle, publication and
transaction guards remain in their adapters.

[API for other commands](../command-options-adapter.md) ·
[Native tests and limitations](../validation/2026-10-12-command-line-options.md) ·
[Earlier BuilderRecord foundation](../validation/2026-10-09-reusable-builder-cage.md).

Original Gem-specific solvers, profile/Styles UI, viewport sliders/handles,
Reset semantics and `.mss` exchange are not enabled by schema registration.
Private CMD/Builder scopes are reducer/ABI tested; native coverage uses the
existing shared controls and existing native input-identity controls.
