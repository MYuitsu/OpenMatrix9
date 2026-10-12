# RCORE-04 — Command Line Options

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

Status: partially implemented; reusable Rust engine with native consumer bindings.
The four requested interactions are toggle, nested choice, numeric input and
CMD–Builder synchronization. The engine accepts schemas by exact feature ID;
new commands register descriptors and native adapters without adding command
names or geometry rules to the reducer.

Rust owns accepted values, draft/cursor phases, aliases, scopes, units/domains,
relations, revisions and candidate tokens. Qt renders typed spans and sends
events. Native construction rejects a provisional candidate without replacing
accepted parameters or the preceding valid preview. Numeric Enter accepts the
option; main Enter/OK commits through the existing native transaction.

The initial consumers are Polyline, Line, InterpCrv, Rectangle, Circle, Ellipse,
Rebuild, Sweep1/Sweep1 History, Sweep2/Sweep2 History and Loft. Rebuild uses
dynamic registration, demonstrating the common path independently of built-in
schema families. Circle/Ellipse size actions retain existing geometric input
semantics; portable parameter options use the common reducer.

[Adapter API and example](../command-options-adapter.md) ·
[Native evidence and exact limits](../validation/2026-10-12-command-line-options.md).

This does not mark all of RCORE-04 complete or enable unrelated geometry
commands/Gem solvers. Transparent commands, full repeat/macro semantics and
all original per-builder workflows retain their existing status.
