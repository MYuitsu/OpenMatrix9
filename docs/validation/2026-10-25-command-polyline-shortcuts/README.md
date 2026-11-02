# Polyline Command shortcut correction — 2026-10-25

Target: Windows x64 standalone Rust OM9. Owners: OM9-L3-03-10-003, OM9-L3-02-04-001 and OM9-L3-03-16-001. User request: Command option initials such as M must change their parameter values.

## Cause and correction

Both existing M bindings assigned `Mode=Line` unconditionally. A global one-shot Mid snap handler also consumed M before Command. The raw-input and Command handlers duplicated option routing, and click hit-testing searched for plain option names in prompts containing combining underline characters.

The first real-window replay additionally exposed a prompt-layout panic when changing Arc back to Line after three points: searching `Close` from the beginning matched the substring inside `PersistentClose`. Label layout and click hit-testing now search forward through the ordered option list. The retained [initial stderr](native-before-fix-stderr.log) records that failure; `switching_back_after_an_arc_keeps_close_distinct_from_persistent_close` reproduces the complete transition and checks that clicking Close creates one mixed curve rather than toggling PersistentClose.

Mode now toggles in the session parser, keyboard and inline clicks submit the same option, and Mode owns M during Polyline. Plain prompt text and UI underline formatting keep labels searchable/copyable. Success updates the live prompt/history; errors retain their feedback. Nonempty drafts, paste, IME and another editor's focus are protected.

Arc mode now produces analytic circular segments within one Polycurve. It uses the previous end tangent, or requests a Direction when none exists. Direction/Center inputs use the shared point path. Preview samples the actual segments; the scene cache and admission plan support bounded mixed line/arc tessellation while preserving straight corners and refusing gaps. Enter commits one transaction; Undo/Redo restores the whole result.

## Automated evidence

See [command receipts](commands.json) and adjacent logs. The geometry oracle is an open line from (0,0,0) to (10,0,0), a quarter-circle with center (10,5,0), radius 5 mm and initial +X tangent ending at (15,5,0), then a line to (20,5,0). Menu and CMD produce identical segments. Tests cover real egui key/text and pointer events, prompt/draft state, analytic geometry, serialization, Undo/Redo, cancellation, invalid arcs, scene gaps and exact display allocation boundaries.

| Batch | Result |
|---|---|
| Expanded app library/Command/menu regression before the native-found layout fix | 216 passed, 0 failed |
| Full scene suite, including mixed arc admission/gap tests | 106 passed, 0 failed |
| Regression after the native-found layout fix | 19 passed, 0 failed; overlaps the app batch, includes 9 Mode regression cases |
| Clippy after the final fix, app + scene libraries, `-D warnings` | Passed |
| Final default-feature MSVC release build, external target directory, `-j 2` | Passed |

The first Clippy run caught a needless borrow after changing the prompt lifetime; it was corrected. The retained initial failure is not an open error. Counts above are batches, not an additive unique-test total.

Scoped rustfmt and the final public source audit pass. The first final-audit attempt correctly rejected screenshots under `docs/validation`; the own-app captures were moved to the existing `implementation/evidence` scope with provenance/hashes, without changing audit policy. Artifact pruning ran with `--apply`: no removable old run remained; cited historical runs were retained by the tool.

## Real Windows replay

The corrected debug window passed menu start, viewport line/arc picks, M in both directions at three points, Close, and typed document Undo/Redo. The final release window separately passed menu start, line/arc viewport picks, M in both directions and Close. The release binary is `L:/om9-target/release/om9.exe`, SHA-256 `924b96be04efca0cc03d969b6d7eba48d1a2761b80bd0aee5b0596904e6c7461`.

Receipts: [debug replay](native-debug-replay.json), [release replay](native-release-replay.json), [source fingerprints](source.json). Screenshots: [release Arc prompt](../../../implementation/evidence/M03/command-polyline-shortcuts-20261025/native-release-mode-arc.png), [Line after Arc](../../../implementation/evidence/M03/command-polyline-shortcuts-20261025/native-release-line-after-arc.png), [committed mixed curve](../../../implementation/evidence/M03/command-polyline-shortcuts-20261025/native-release-closed.png), [debug document Undo](../../../implementation/evidence/M03/command-polyline-shortcuts-20261025/native-debug-undo.png), [debug document Redo](../../../implementation/evidence/M03/command-polyline-shortcuts-20261025/native-debug-redo.png). The earlier `native-mode-arc.png` belongs to the first, superseded release and is not final acceptance evidence.

Owned isolated profiles: `build/standalone-validation/ui-artifacts/command-polyline-shortcuts-20261025`, `build/standalone-validation/ui-artifacts/command-polyline-shortcuts-debug-20261025`, and `build/standalone-validation/ui-artifacts/command-polyline-shortcuts-release-20261025`. The final release window was left open for the user. No human M01/M03 gate was signed by this replay.

## Limits

- Center constructs minor arcs; antipodal/semicircle Center endpoints and major Center arcs are not certified.
- Helpers changes its existing flag/prompt; dynamic tracking remains unfinished.
- PersistentClose retains the existing restart-after-commit behavior; continuously closed preview is unfinished.
- Whole-object picking is retained for mixed Polycurves; there is no invented global native parameter or new edge subobject identity.
- Exact mixed-Polycurve 3DM re-export, macOS, and full M01/M03 manual acceptance are not certified.
- No native adapter, SDK ABI, reusable skill business rule, publication or user session was changed.

## Proposed skill update — pending user consent

Before: the workspace skill lists M/M=L as straight Line only and Arc as unfinished.

Proposed addition: “Trong Polyline, khi draft trống và không nhập IME, M hoặc bấm Mode đổi Line/Arc bằng cùng handler và cập nhật prompt ngay. Mode ưu tiên trước phím bắt điểm Mid. Arc tạo đoạn cung giải tích trong cùng Polycurve; Enter ghi một transaction, Undo/Redo áp dụng cả đường. Giữ draft có nội dung; không bắt phím từ editor khác. Direction/Center chỉ xuất hiện trong Arc, Length chỉ trong Line sau điểm đầu. Helpers tracking và PersistentClose preview liên tục vẫn chưa hoàn tất.”

This proposal has not been recorded in a source or installed skill.
