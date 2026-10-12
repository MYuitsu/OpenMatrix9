# Command Line Options Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans for native inline execution, or superpowers:subagent-driven-development if the user selects that method. Steps use checkbox syntax for tracking. Execution awaits the required human plan review.

**Goal:** Triển khai toggle, tùy chọn con, nhập số và đồng bộ CMD–Builder bằng trạng thái Rust chung, có bằng chứng native về controls, preview, geometry và transaction.

**Architecture:** Safe Rust sở hữu option schema, alias resolution, phase, accepted values, validation và generation/revision. Qt hiển thị snapshot và chuyển sự kiện có kiểu; các controller native dùng snapshot đã nhận để gọi solver/preview/commit hiện hữu. Không thêm solver Gem hoặc một Builder giả chỉ dùng cho fixture.

**Tech Stack:** Rust 2024/std, C ABI có ownership rõ, C++/Qt6, FreeCAD/Part/OCCT/Coin và Python/PySide6 cho fixture ứng dụng.

**Spec:**

- [RCORE-04 Command Line Options](../../../OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/04-command-engine.md#command-line-options).
- [Builder Framework](../../../OpenMatrix9_Codex_Spec_v1/specs/01-core/om9-buildercore-001-builder-framework.md#command-line-options).
- Các supported slices của Curve, Rebuild và Surface trong Spec v1 giữ nguyên điều kiện geometry và defaults đã công bố.

## Global Constraints

- Rust sở hữu business logic/state/validation; C++ chỉ nối Qt, FreeCAD, OCCT/Coin và lifetime native. Không giữ document/widget/kernel pointers trong Rust.
- Giữ exact feature IDs, original command names/aliases, unsupported gates, protected transcript, IME và command completion khi idle.
- Chỉ toggle/choice/number đã nhận mới đổi session/preview. Enter của option không đồng thời commit command; Close action của Polyline khác Close toggle của Builder.
- CMD và Builder có thể có tập option riêng; shared option dùng cùng key/state/validator. Không giữ hai accepted values độc lập.
- Input sai giữ accepted values và preview trước đó; draft không bị ghi đè. Snapshot refresh không phát lại user event.
- Một transaction cho geometry+parameters+metadata theo feature. Cancel không output/Undo document; stale generation/revision không publish.
- Dùng duy nhất SDK `H:/FreeCAD-src/build/relWithDebInfo`, cache plugin `H:/FreeCAD-src/build/openmatrix9-history-cage` và target `OpenMatrix9Gui` qua `build/build-om9-plugin.ps1`.
- Runtime đề xuất `H:/FreeCAD-src/build/command-options-runtime-20261012`; launcher `build/run-om9-plugin.ps1`, profile riêng từng run. Không build/configure FreeCAD root hoặc sao chép nguyên host.
- Worker default vẫn 60% logical CPUs, làm tròn xuống, tối thiểu một. Không cleanup recursive, không đổi branch hoặc ghi đè thay đổi khác.
- Không thêm dependencies Rust mới cho option UI. FFI dùng owned data phía Rust và caller-owned buffers phía Qt, có bounds và error status.

## Review Focus

1. CMD đang có draft số trong khi Builder sửa cùng option: chặn thao tác Builder, giữ draft và accepted value; thử lại sau Enter/Esc.
2. Choice có mnemonic collision, child disabled và tên giống ở hai feature: giải theo key/phase, không tự chọn first item hoặc fallback.
3. Option cập nhật controls gây vòng signal hoặc hai toggle: một event/revision, QSignalBlocker khi render snapshot.
4. Native solver từ chối candidate hoặc callback tới sau Cancel/Mode change: không mất preview/accepted values cũ và không late commit.
5. Enter/Esc có focus ở CMD, ô số, choice hoặc viewport: đúng phase, không lấy phím của IME hoặc chạy history/repeat sai.

Mỗi mục được kiểm ở Rust và/hoặc native trong các task sở hữu hành vi bên dưới.

## Baseline và phạm vi consumer

- Checkout: `H:/FreeCAD-src/Mod/OpenMatrix9`, branch tại thời điểm lập kế hoạch `codex/rhino-core-object-foundations`; có nhiều thay đổi đang làm từ các công việc khác. Không commit/stage cả checkout.
- CMake cache đã đối chiếu đúng source checkout và SDK chung; runtime hiện hành là `initial-parameters-runtime109-20261012`. Runtime mới phải được chọn qua script có build-cache mutex.
- `CommandConsole` hiện dùng regex cố định cho sáu tên và biến click Mode thành Mode=Line. `CurveController::acceptInput` lấy draft trước khi biết input hợp lệ, còn `cancelInput` hủy toàn command; cần sửa theo option phase.
- `SurfaceController` có native options UI và solver thật cho Sweep1/Sweep2/Loft. Controls hiện ghi thẳng vào C++ `SurfaceOptions`; CMD chưa có đường nhập shared option values.
- CoreRebuild có controls Degree/PointCount/DeleteInput và preview/publication hiện hữu. Tích hợp vào schema chung, giữ guards và transaction hiện có.
- Native consumers dùng Curve đang hỗ trợ và Surface/Rebuild controls thật. Các option chưa có geometry adapter vẫn disabled; không bật gvGemOnCrvAdvanced hoặc suy defaults từ ảnh.
- Baseline `rtk cargo test --manifest-path Mod/OpenMatrix9/rust/Cargo.toml` dừng ở lib tests: 120 pass, 1 fail. Lỗi `support_snapshot::evaluation_tests::owned_curve_jets_match_native_value_and_three_derivatives_for_nineteen_families` với OffsetBezier parameter 0.1, derivative 3; rerun exact test cũng fail. Chưa sửa product code Command Line Options khi lỗi này xuất hiện; integration suites chưa chạy sau lib failure. Kiểm tra lại baseline lúc bắt đầu execution và ghi riêng nếu còn lỗi, không báo full Rust pass khi test này vẫn fail.

## File Structure

| File | Trách nhiệm |
|---|---|
| `rust/src/command_options.rs` | Schema/value/event/cursor, reducer, typed candidate và snapshot; không có native pointers. |
| `rust/src/command_option_schemas.rs` | Schema và bindings của supported Curve/Rebuild/Surface options, visibility/gate theo context. |
| `rust/src/command_options_ffi.rs` | C ABI, bounded UTF-8/buffers, generation/revision và candidate acceptance. |
| `rust/src/lib.rs`, `curve.rs`, `surface.rs` | Module registration và nối option model vào command session owner hiện có. |
| `Gui/CommandOptions.h/.cpp` | Qt bridge cho typed snapshots/events, prompt spans và control bindings. |
| `Gui/CommandConsole.h/.cpp`, `CurveController.h/.cpp` | Schema-driven live option input/click/keyboard; bảo vệ draft và transcript. |
| `Gui/SurfaceController.h/.cpp`, `CoreRebuild.cpp` | Controls đọc shared snapshot; native preview/commit dùng accepted parameters. |
| `Gui/RustBridge.h`, `Gui/CMakeLists.txt` | ABI declarations và plugin source registration. |
| `rust/tests/command_options.rs`, `command_options_ffi.rs` | Semantic và ABI regression. |
| `tests/command_line_options_smoke.FCMacro`, `builder_command_options_smoke.FCMacro` | Native UI, geometry, preview, persistence và lifetime. |
| `docs/features/RCORE-04.md`, `docs/features/OM9-BUILDERCORE-001.md` | Implementation records, native exceptions/ownership và supported scope của hạ tầng mới. |

## Task 1: Rust option reducer, schemas và ABI

**Interfaces:**

- Define `OptionKey = String`; `OptionValue::{Toggle(bool), Choice(String), Number(f64)}`; `OptionKind::{Toggle, Choice, Number, Action}`.
- Define owned `OptionSpec` with key, parent, labels/aliases/mnemonic, kind, scope, choices, numeric rule and gate reason. `OptionContext` carries generation, unit scale and portable phase/capability facts.
- Define `OptionEvent::{Activate(OptionKey), Submit(String), Set(OptionKey, OptionValue), Escape, Select(i32)}`; `OptionSnapshot` contains prompt spans, visible choices, accepted values, pending key, generation/revision and errors.
- `OptionSession::new(schema: Vec<OptionSpec>, values: BTreeMap<OptionKey, OptionValue>, context: OptionContext) -> Result<Self, OptionError>`.
- `OptionSession::dispatch(&mut self, event: OptionEvent) -> Result<OptionOutcome, OptionError>`; outcome separates cursor-only navigation, candidate value change, action and unhandled command input.
- Candidate contains base generation/revision plus typed values; `accept(candidate)` checks the same identity before publication. Native rejection discards candidate without changing accepted values; immutable snapshot is the renderer/solver source.
- C ABI functions in `command_options_ffi.rs`: `om9_options_open`, `om9_options_event`, `om9_options_accept`, `om9_options_snapshot`, `om9_options_close`. Handles and revisions are `u64`; value-kind/event/status discriminants are `u32`, number is `f64`; strings are bounded caller-owned UTF-8 buffers. Concrete mirrored structs and safety contracts are defined together in `RustBridge.h` before use.

- [ ] Write tests first: `toggle_click_once_explicit_set_idempotent`, `choice_nested_escape_keeps_accepted`, `choice_collision_and_disabled_rejected`, `number_enter_does_not_finish`, `invalid_number_preserves_values_and_draft`, `generation_and_revision_reject_stale_candidate`.
- [ ] Run the new Rust tests and preserve RED output for the missing production module/behavior before implementation.
- [ ] Implement reducer and static per-consumer schemas; reuse `units::parse_length(raw, mm_per_unit)` for length quantities, integer validation for counts and feature-specific rules for scalars.
- [ ] Test existing values/ranges: Surface pointCount 2–256 and tolerance 1e-7–1e6 mm; native cross constraints remain enforced. Empty Enter uses only a displayed, valid current/default allowed by the descriptor. Do not invent source defaults.
- [ ] Bind supported option keys to original Curve/Surface/Rebuild session values; legacy native structures become derived snapshots, with no independent mutable accepted copy.
- [ ] Add ABI tests for malformed UTF-8, null/short buffers, wrong discriminants, invalid handles, stale tokens and close/reopen; check layout agreement and no retained foreign pointers.
- [ ] Run `rtk cargo test --manifest-path Mod/OpenMatrix9/rust/Cargo.toml --test command_options` and `--test command_options_ffi`; expected all new semantic/ABI tests pass. Keep the unrelated baseline test status separate.

## Task 2: Schema-driven Command surface và Curve consumers

**Interfaces:**

- `CommandOptionsBridge::snapshot()` returns a `CommandOptionSnapshot` containing QString prompt text, keyed click spans and pending phase. `submit`, `activate`, `escape`, `select` return typed outcome/error; this is Qt/C ABI adaptation, not a second reducer.
- `CommandConsole::setOptionSnapshot(const CommandOptionSnapshot&)` renders the current live prompt; option callbacks carry a stable key, not an inferred regex string.
- Curve session owns the accepted option model; native CurveController consumes typed actions/accepted snapshot through the existing geometry path.

- [ ] Add native RED fixtures using the real console: typed and clicked option aliases, Mode choice with disabled Arc, Length numeric Enter, nested Esc and invalid draft preservation. Old plugin must fail because the option behavior is absent, not because module import failed.
- [ ] Replace regex/hard-coded click mapping with schema spans. Historical prompt text remains copyable and cannot activate the current session; dragging selection is never an option click.
- [ ] Validate before consuming a submitted draft; on recoverable error preserve its text. Route Esc to child/number phase first; only the main phase cancels the command.
- [ ] Implement Up/Down/Tab for option choices with no command completion/history during active option input. Preserve idle completion, IME, clipboard and transcript protections.
- [ ] Bind currently supported live options of Line/Polyline/InterpCrv/Rectangle/Circle/Ellipse from their actual session/capability, preserving geometry and unsupported modes. Close/Undo are typed actions; explicit value set is not a toggle.
- [ ] Run `command_line_options_smoke.FCMacro`; check RCORE-04.T14/T15/T16/T18/T19/T23 against native Curve state/preview/wire, not prompt text alone.
- [ ] Regress `matrix_grid_command_smoke.FCMacro`, `command_completion_smoke.FCMacro`, `curve_smoke.FCMacro`, `curve_spline_smoke.FCMacro` and the existing native option suites of the Curve consumers touched.

## Task 3: Real CMD–Builder bindings và native lifecycle

**Interfaces:**

- Surface/Rebuild controls send the same typed Set/Activate events as CMD. Qt refresh from snapshot uses `QSignalBlocker`; C++ solver receives a typed copy of accepted parameters.
- Native candidate validation builds a temporary shape/preview before replacing the previous valid preview or accepting values; failure leaves previous values/preview intact. Publication rechecks option/session generation/revision and existing document/source guards.

- [ ] Add native RED fixtures on Loft/Sweep options and Rebuild controls: CMD Preview toggle, Style choice, section Rebuild/Refit child numbers, Rebuild Degree/PointCount, UI→CMD and CMD→UI propagation.
- [ ] Move portable option validation/gates to Rust; initialize from existing supported defaults/settings and identity context. Bind every currently supported Surface scalar/toggle/choice control and Rebuild Degree/PointCount/DeleteInput. Input-specific seam/reverse/slash actions retain source identity and geometry guards; do not silently expose an unsupported option.
- [ ] Represent section simplification choices and their PointCount/Refit Tolerance children in the common schema. Use actual solver parameters and existing control names, not test-only choices or a fake Builder.
- [ ] Protect the CMD draft: attempts by Builder to change the same pending option are rejected with feedback and the widget is restored from accepted state. A change to another option revalidates any dependent draft before accepting it.
- [ ] Update dependent gates, current values, prompt and controls atomically after acceptance; block signal loops. CMD-only and Builder-only options follow descriptor scopes while sharing the owning session.
- [ ] Route Enter/OK/Commit through the same readiness check. Choice/number pending cannot commit via a button or viewport Enter. Preserve the feature's X/panel-close semantics; panel hide/show for pick does not reset values or create a second session.
- [ ] Native fixture compares UI and CMD paths on equivalent input documents: accepted parameters, actual Part geometry, scene preview, transaction count and stored SurfaceOptions/Rebuild metadata must agree.
- [ ] Verify RCORE-04.T17/T20/T21/T22 and Builder CMD-T01–T07, including invalid inputs, Mode/Reset where the consumer actually supports them, cancellation, document close/deactivate, stale callback rejection, Undo/Redo and cold FCStd reopening.
- [ ] Regress existing `surface_commands_smoke.FCMacro`, `surface_options_smoke.FCMacro`, `surface_history_smoke.FCMacro` and Rebuild/publication suites affected by the integration.

## Task 4: Build, independent review và evidence synchronization

- [ ] Run full `rtk cargo test --manifest-path Mod/OpenMatrix9/rust/Cargo.toml`, formatting checks on changed Rust files, targeted Clippy and release build. Name all baseline/new failures; do not report full-suite success if any test remains red.
- [ ] Build only the plugin with `rtk proxy pwsh -NoProfile -ExecutionPolicy Bypass -File H:/FreeCAD-src/build/build-om9-plugin.ps1 -SourceDirectory H:/FreeCAD-src/Mod/OpenMatrix9 -BuildDirectory H:/FreeCAD-src/build/openmatrix9-history-cage -RuntimeDirectory H:/FreeCAD-src/build/command-options-runtime-20261012`.
- [ ] Run the Command fixture: `rtk proxy pwsh -NoProfile -ExecutionPolicy Bypass -File H:/FreeCAD-src/build/run-om9-plugin.ps1 -PluginRuntime H:/FreeCAD-src/build/command-options-runtime-20261012 -Macro H:/FreeCAD-src/Mod/OpenMatrix9/tests/command_line_options_smoke.FCMacro -OutputDirectory H:/FreeCAD-src/build/shared-host-tests/command-line-options-20261012/run01`.
- [ ] Run the Builder fixture: `rtk proxy pwsh -NoProfile -ExecutionPolicy Bypass -File H:/FreeCAD-src/build/run-om9-plugin.ps1 -PluginRuntime H:/FreeCAD-src/build/command-options-runtime-20261012 -Macro H:/FreeCAD-src/Mod/OpenMatrix9/tests/builder_command_options_smoke.FCMacro -OutputDirectory H:/FreeCAD-src/build/shared-host-tests/builder-command-options-20261012/run01`. If a run directory already exists, preserve it and select the next unused run ordinal for the fresh run; record the actual command.
- [ ] Require actual host exit 0, loaded module path/hash, source fingerprints, fixture results and native screenshots. Read every failed check and fix within the changed behavior before rerunning; preserve reports/profiles/models.
- [ ] Review the change against both specs and the ten RCORE/seven Builder fixture requirements; for native execution use the final independent reviewer required by the execution skill, after the user selects that method.
- [ ] Run applicable Python tool tests, `rtk proxy python tools/public_source_audit.py` and scoped `git diff --check`. Stage/checkpoint only task-owned hunks if committing; no push/publication.
- [ ] Record ownership/native exceptions and supported scopes in the two feature implementation records; record commands, hashes, outputs, baseline failures and limits in `docs/validation/2026-10-12-command-line-options.md` and the implementation ledger.
- [ ] Update the two specs' implementation notes/acceptance items only for evidence actually obtained; synchronize affected consumer records, FEATURES.json/YAML, IMPLEMENTATION_STATUS, README and MANIFEST without upgrading unrelated/full RCORE completion.

## Completion Criteria

- Four requested interaction classes work through the same Rust-owned state on actual native consumers, with no fake Builder/solver acceptance.
- Every new fixture category has native evidence or a precise unsupported consumer/gate identified; schema-only tests are never labeled native UI/geometry proof.
- Original geometry behavior, idle completion, draft/IME/copy and cancellation regressions pass for the touched consumers.
- Existing baseline failures are resolved by the responsible source change or reported by exact test name; they are never hidden by a filtered green run.
- Final report links runtime evidence and updated specs, states the supported consumer scope and any remaining limits, and does not claim all RCORE-04/Builder/Gem implementations are complete.

## Plan Review

- Spec coverage: all four interactions plus phase, draft, aliases/collision, visibility/gates, native publication, transactions and lifecycle have task/test ownership above.
- Execution recommendation: Native inline in the existing checkout. Rust↔Qt↔controller interfaces are tightly coupled and the current source has shared in-progress changes; sequential ownership minimizes concurrent edits. An isolated worktree would need the current approved spec and dependencies preserved deliberately.
- Human approved Native inline execution in this chat on 2026-10-12. The engine must be reusable by other commands, as clarified by the user. Dynamic exact-ID schema registration and a shared native widget binder implement that clarification. Runtime/Rust output may use C: when H: is full; the shared SDK and plugin cache are reused.

## Final execution status — 2026-10-12

Tasks 1–4 are complete in the supported scope recorded by the validation report.
Runtime/Rust output is on C, matching existing App141 is selected with the same
shared SDK, and no full host/root build was made. The original detailed plan
checkboxes remain an unedited planning record; they must not imply exhaustive
Reset/private-widget/focus/macro implementation. The two feature specs and
validation report identify actual coverage and remaining broad requirements.
Thirty reducer/ABI tests and 13 native suites / 692 checks pass; Ellipse Layer
swatch and OffsetBezier derivative failures are pre-feature baselines. One final
independent review's five Important findings were repaired in one RED→GREEN pass.
No commits, merge or publication: preserve unrelated shared work and artifacts.
