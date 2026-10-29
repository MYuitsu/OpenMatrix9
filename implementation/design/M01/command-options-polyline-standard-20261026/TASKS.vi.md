# Kế hoạch triển khai từng gói cho GPT Luna

Đây là 13 **gói công việc dự kiến** P00..P12, không phải 13 batch test đã chạy và không phải toàn bộ số batch geometry tương lai. Mỗi gói là một checkpoint nhỏ; tiếp tục đến hết phạm vi được giao, không tự kết thúc nhiệm vụ chỉ vì một gói đã xong. Không tự spawn agent hoặc tạo chat khác khi người dùng chưa yêu cầu.

## Quy ước trạng thái và checkpoint

Trạng thái một command/mode: `UNCLASSIFIED`, `AUDITED`, `BRIDGE_ONLY`, `MIGRATING`, `PARITY_DONE`, `VERIFIED_POSITIONAL`, `VERIFIED_IMMEDIATE`, `VERIFIED_SPECIALIZED`, `UNSUPPORTED_WITH_REASON`, `BLOCKED_WITH_REASON`.

Mỗi checkpoint ghi: gói/owner/mode đã làm; source hash; files changed; tests/scenarios/fixture counts tách riêng; native executable identity; lỗi baseline/ignore; các hàng còn mở; file/method tiếp theo; hành động cụ thể tiếp theo. Khi context bị compact, đọc checkpoint và tiếp tục, không restart hoặc coi kế hoạch là evidence.

Không dùng checkbox của tài liệu này như kết quả implementation. Tạo execution ledger/report riêng khi được giao thực thi. Giữ source inventory gốc ngày 2026-10-26 để trace; nếu nguồn đổi, tạo inventory refresh có timestamp và giải thích delta.

## P00 — Khóa scope và inventory thật

**Đọc:** `SPEC.vi.md` §§1–4, `COMMAND-MATRIX.vi.md`, `source-inventory.json`, `session.rs` dispatch/submit/prompt/cancel/is_running, menu registries và Top11/File/View/plugin entry points.

**Làm:**

1. Kiểm SHA256 source inventory với checkout hiện tại. Đọc `git status` và ghi dirty baseline; không reset thay đổi khác.
2. Tạo execution table mỗi command/alias/mode: exact feature ID, owner, route, classification, stage table path, tests, status, reason.
3. Reconcile 39 nhóm, 57 pending fields và 19 field ngoài registry. Line là base `session`, không nằm trong count `_pending`; Solid/Top11 là nhiều command/variant chung type. Không cộng các số này thành tổng số lệnh.
4. Tìm cả command không có pending qua catalog/menu/dispatcher. Alias không tự được chứng nhận chỉ vì ID cùng tên gần giống.
5. Đọc exact milestone bằng router cho family đầu tiên; ghi nguồn nào là contract, nguồn nào chỉ là code fact/unknown.

**Bằng chứng kết thúc:** bảng không mất owner gốc; mọi hàng bổ sung được phân loại hoặc ghi AUDIT_REQUIRED rõ; không production code thay đổi trong P00. Nếu tổng command chưa xác định, nói tổng unknown, không tính phần trăm.

## P01 — Khóa oracle Polyline và regressions chuyển lệnh

**File/test:** `session/polyline.rs`, `polyline_options.rs`, `tests/polyline_options_interface.rs`, `polyline_mode_regression.rs`, `command_options_interface.rs`, `pinned_command.rs`.

**Làm:** chạy baseline đúng default-feature target, `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`. Ghi stage/value của P/M/H/L/C/D/U; Line–Arc–Line fixture; Close/PersistentClose collision; Length invalid/retry; source owner vs repeat slot. Giữ test Line→Polyline cả menu và CMD.

**Thêm ca thiếu nếu cần:** same-command new-run stale option; chuyển Polyline Length entry sang Cancel/Idle/Line; Line→Circle/Rectangle; unavailable options nhận ID cũ; selection/document context stale. Không dùng label assertion đơn lẻ thay geometry/state assertion.

**Kết thúc:** baseline và known failures tách rõ; không đổi semantics Polyline đang có để giảm số case cần làm.

## P02 — Active owner, run identity và typed snapshot

**File dự kiến:** `command_options.rs`, `session/command_options.rs`, `session.rs`; có thể tách module command-input mới nếu giúp ownership rõ. Không rewrite toàn bộ session/geometry.

**Làm theo thứ tự:**

1. Thêm active-input owner query thật; không fallback repeat slot. Capture exact feature ID lúc bắt đầu command, không suy lại từ mode label.
2. Thêm run ID/generation theo Controller lifecycle; không tăng run ID mỗi frame. Commit/cancel/new-run/context replacement làm snapshot cũ mất hiệu lực. Có thể thêm input-stage generation để từ chối stale pick/click sau chuyển stage.
3. Mở rộng record chung có typed current value, input/unit semantics, availability/constraints/default cần thiết. Giá trị derive từ owner, không tạo cache mutable trong MatrixApp.
4. Bảo vệ activation bằng owner/run/context và snapshot mới. ID name giống nhau ở Sphere/Box hoặc run A/run B không phải quyền activate run mới.
5. Adapter tương thích cho Polyline và các provider chưa migrate; status BRIDGE_ONLY. Không xóa API legacy còn caller trước khi chuyển xong caller.

**Tests bắt buộc:** idle với repeat Line vẫn zero options; owner selected đúng cho mọi pending fixture; stale cross-owner và same-owner new-run không mutate; snapshot getter không side effect; selected choice typed values đúng.

## P03 — Value/point entry thuộc owner

**File:** Controller input module, `session/command_options.rs`, `ui.rs::submit_command_option`, `submit_draft`; Polyline entry làm oracle.

**Mẫu xử lý dưới đây là pseudocode, không phải API sẵn có để copy rồi gọi hàm chưa tồn tại:**

```rust
fn activate_option(handle) -> Result<UiEffect> {
    validate_current_owner_run_and_stage(handle)?;
    let option = fresh_owner_snapshot().lookup(handle.id)?;
    match option.activation {
        Toggle | SetChoice(_) | CycleChoice => owner_apply_option(option),
        BeginValue => owner_begin_value_entry(option.id),
        BeginPoint => owner_begin_point_entry(option.id),
        Action => owner_apply_action(option.id),
    }
}

fn submit_command_draft(text) -> Result<UiEffect> {
    if owner_is_entering_text_value() { return owner_submit_value(text); }
    if owner_is_entering_numeric_or_point_value() { return owner_submit_value(text); }
    if active_owner_exists() {
        // Full option/alias/assignment/primary point, all interpreted by current owner.
        return owner_submit_input(text);
    }
    idle_command_dispatch_or_repeat(text)
}
```

**Chi tiết không được bỏ:**

- Owner lưu `parameter_id`, `return_stage`/input context cần thiết; không duplicate current number/geometry trong UI. Point entry đi qua input pick pipeline hiện có.
- Thêm effect cho “đã bắt đầu input” nếu cần. MatrixApp chỉ clear draft/focus và refresh prompt; effect không tự sinh geometry. Mọi match của enum phải được cập nhật để compile ngay.
- Giữ `Draft(prefix)` tạm cho owner chưa chuyển, ghi rõ bridge; không đánh dấu xong khi chỉ đổi tên effect.
- Same-parameter `Name=value` và bare value dùng cùng validator. Tên parameter/command khác trong entry không thoát ngầm; Text không đi qua alias lookup.
- Empty Enter default/required, NaN/Inf, số nguyên, negative/zero, Unicode text/path, invalid retry và Esc phải tách test.
- Last-input có thể commit. Result effect phải phân biệt applied-and-still-running, committed/finished và failed; không append running prompt sau finish.

**Hoàn tất P03:** ít nhất Polyline, một scalar owner (Helix Turns), một integer owner (Degree), một point/vector owner, một text owner có ba route tương đương. Family còn lại giữ bridge rõ, không tuyên bố tất cả xong.

## P04 — Formatter/span và một đường render

**File:** `ui/command.rs`, `transcript.rs`, UI audit getters/tests. Giữ một editor và draft.

1. Formatter nhận active snapshot, tạo prompt text + ordered token spans + shortcut spans cùng lúc. Có thể dùng một struct view-model transient; không parse display string thành owner state.
2. Cập nhật protected transcript prompt từ formatted result; owner/domain prompt nền không mất nghĩa. History không bị sửa ngược.
3. Vẽ underline/click target theo spans. Test `PersistentClose`/`Close`, current value chứa chuỗi giống tên option, Unicode và ký tự nhiều byte.
4. Render một path cho tất cả owner migrated. Gỡ floating button row cũ khi path mới đã thay; owner bridge vẫn dùng cùng formatter nếu đủ metadata, không tạo widget source thứ hai.
5. Giữ pinned row/unwrapped/hscroll/IME/caret/selection. Token dài không được đẩy click target ngoài vùng Command hay che toolbar; resize/minimum width/DPI phải kiểm.
6. Dùng run-aware stable IDs; clear hit map khi owner/stage đổi. Không để rect hoặc pointer-down từ run cũ activate run mới.

**Tests:** `pinned_command`, `command_tokens`, `command_shortcuts`, `command_transcript`, IME/focus/held-key regressions; native prompt dài Circle và Text Unicode. Không kiểm screenshot similarity thay cho click/selection behavior.

## P05 — Line, Rectangle, Arc

Thứ tự nhỏ: Line → Rectangle default/Square → Rectangle modes → Arc/Arc Direction modes. Dùng OWNER-CARDS §3.

Mỗi bước điền stage table, migrate entry và values, gỡ branch/bridge cũ, kiểm tất cả parameters của mode rồi actual geometry/Undo. Giữ unit regression của Line. Ghi rõ availability trước/sau first point; không đưa BothSides sang command khác. Tests targets: `command_options_interface`, `command_options_and_repeat`, `rect`, `arc`, `command_sessions`, `command_state_machine` khi phần đó bị chạm.

## P06 — Circle và Ellipse

Đọc OWNER-CARDS §4. Tách mode/size kind/size number/approximation/constraints. Không làm đồng thời toàn bộ complex tangent geometry nếu đang chỉ sửa metadata; solver nào chưa có proof thì ghi limitation và giữ disabled/reason đúng.

Mốc nhỏ: Circle Center size conversions → Circle numeric/choice collisions → Vertical/2Point/3Point → FitPoints/AroundCurve/Tangent stages → Ellipse basic modes → foci/around-curve/deformable. Mỗi mốc chạy `circle` hoặc `ellipse` tests và new three-route cases cho parameters vừa chuyển.

Đặc biệt test Approx/Deformable alias, selected Size choice, Degree/PointCount relation, malformed Point token, Curve source lifetime và C/D/A/P collisions. Không biến set choice thành toggle bằng generic helper.

## P07 — Point-chain families, Polygon, Helix, Spiral

Owners: interp, control_curve, continue_curve, continue_interp, through_points, polygon, helix, spiral. Dùng OWNER-CARDS §§5–6.

Tách mỗi owner thành checkpoint; integer bounds, odd Degree chỉ của InterpCrv, Closed/Open và Circumscribed/Inscribed set semantics, signed/zero Height khác nhau, Axis vector input. Tests theo tên file thật trong `crates/om9-app/tests`; thêm menu/CMD invoke path khi chưa có evidence.

## P08 — Curve modifiers và query owners

Owners: rebuild, fit, offset_curve, project, blend, pull, intersect, crv2view, arc_blend, fillet_corners, corner, extend_curve, divide, to_lines, tween_curves, crv_seam, curve_soft_edit, offset_on_surface, offset_normal.

Theo OWNER-CARDS §7, xử lý tối đa một family đang sửa tại một thời điểm để giữ diff kiểm được. Bảo vệ selection/source stamp; validate parameter và result. Cancel-only owners vẫn có snapshot active/input nhưng không nhận thêm tham số giả. Audit values đọc sai field, set-vs-toggle, signed distance, integer budget và current source topology index.

## P09 — Registered surface-related owners

Owners: isocurve, angle_isocurve, cross_section, extrude_surface, heightfield. Theo OWNER-CARDS §8.

Phải có fixture surface đúng. U/V và Direction=U/V không phải world XYZ. Không tự clamp Angle/domain; resolve contract discrepancy rồi mới chọn implementation. Text File path giữ whitespace/case; không dùng global command dispatch khi đang entry. Native backend unavailable phải giữ failure rõ, không mock cho pass.

## P10 — Solid và positional chain

Thứ tự: Box → Sphere basic modes → sphere constraints/solutions → Text/Boolean2Objects → other positional primitives/topology operations. Theo OWNER-CARDS §§9–10 và approved Solid references.

- `solid` là một provider group nhưng nhiều feature/variant. Bảng coverage phải tách Box/Sphere và mọi mode được nhận supported.
- Positional chain dùng current slot/entry protocol chung, không cấp quyền edit bất kỳ previous slot khi owner chưa có.
- Giữ Text Unicode/digits, Boolean choice shortcuts, exact topology indices và signed height.
- Chạy `solid`, `solid_workflows` với known baseline failure/fixture ignores được báo đúng. Không sửa test/skip để giấu failure mới. Nếu một known skip được dùng, nêu tên và baseline evidence trong lệnh/report.

## P11 — 19 extra pending owners và catalog sweep cuối

Đọc OWNER-CARDS §§11–12 và `additional_pending_owners` trong JSON. Với từng field, tìm mọi feature/variant đi qua nó; điền stage table từ parser thực. Migrate OPTIONS/POSITIONAL có UI Command; chuyên biệt/immediate/unsupported kết thúc bằng reason/evidence đúng loại.

Top11 Duplicate/Mirror/Split/Trim/Move/Rotate phải test point-vs-object input, flag availability, copies transaction boundaries. Sweep History/Add Slash giữ recipe/IDs/source stamps và option state qua entry. Mỗi nhánh surface ngoài registry resolve exact feature trước khi đổi unit/domain/constraints.

Chạy đối chiếu lại toàn bộ menu/toolbar/F6/CMD/File/View/Layers/Gumball/plugin categories. Không còn một nhánh cũ gọi parser trực tiếp từ UI mà không có lý do/classification được ghi. Không tự migrate giao thức editor/SDK riêng nếu thiếu contract; ghi VERIFIED_SPECIALIZED/UNSUPPORTED_WITH_REASON có căn cứ.

## P12 — Nghiệm thu thực và bàn giao

1. Tất cả discovered entries có trạng thái kết thúc hoặc blocker cụ thể; báo số unknown nếu còn, không nói tất cả đã xong khi có AUDIT_REQUIRED.
2. Chạy appropriate tests cho final code, Clippy, scoped fmt, public-source audit, diff check; build default features với target dir ngoài checkout và `-j 2`.
3. Trước native input, xác định đúng PID/path/SHA256/profile. Không dựa vào title `OpenMatrix9 — Untitled`; binary cũ vẫn có title đó. Không đóng phiên chưa lưu hay build đè executable đang được người dùng dùng.
4. Replay menu và CMD, thông số bằng key/click/text, integer/numeric/point/text retry, actual geometry, cancel và Undo/Redo. Chụp own-app evidence và provenance; test runtime không phải human sign-off.
5. Ghi debug/release/package/install trạng thái riêng. Có debug mới không đồng nghĩa user release đã được cập nhật; không publish nếu chưa được giao.
6. Cập nhật scoped milestone runtime_implementation, root/nhóm README và live progress ledger. Record implementation uncertainty, known baseline failures, ignores, remaining packages và exact next action.
7. Nếu implementation thay nghiệp vụ ngoài những gì đã được giao, trình before/after và proposed skill text sau verify; chỉ ghi skill theo consent. Không dùng việc có tài liệu này làm human UI gate approval.

## Mẫu báo cáo cuối

```text
Target/source/binary identity:
Đã migrated: command/mode list + parity behavior cụ thể.
Đã classified ngoài option registry: immediate/positional/specialized/unsupported + reasons.
Test functions / parameter-route scenarios / native fixtures: tách số, không cộng trùng.
Geometry/Undo/context/focus đã chứng minh:
Known baseline failures / new failures / ignored fixtures:
Debug / release / installed binary đang ở trạng thái nào:
Chưa xong / unknown / human gates:
Files/report/checkpoint để tiếp tục:
```
