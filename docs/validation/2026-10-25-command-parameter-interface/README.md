# Shared command parameter interface — 2026-10-25

Scope: all existing Command option declarations (37 registered provider groups, plus Line and Polyline). Commands which only request positional input, and commands without declared options, retain their existing input flow. This work does not implement additional geometry capabilities or promote milestone acceptance gates.

`CommandOptionInterface` in `crates/om9-app/src/command_options.rs` exposes a snapshot of available `CommandOption` records, resolves names/accelerators, and activates a `CommandOptionId`. Records carry owner-scoped IDs, labels derived from current owner state, input kinds, advertised shortcuts and activation. The adapter lives in `session/command_options.rs`; Polyline's existing typed contract remains compatible.

The Command UI consumes this interface for every owner. The former per-command rendering/dispatch ladder is removed. Keyboard, pointer and typed-name activation use the same controller contract. Completed assignments go through `Controller::submit_effect` and the existing owner parser; only declared assignment names and fixed choices are canonicalized without altering values.

Before this change, most tokens underlined a letter without a common keyboard handler; numeric tokens could submit an incomplete assignment and produce an error. Now toggles/choices update immediately through the owner, while numeric/point/text options stage the appropriate `Name=` draft. Invalid values retain the draft and owner state. Polyline retains its dedicated numeric/point stages and dynamic prompt values.

Initial-letter accelerators are reserved first; collisions use a free alphabetic letter in the option's own name, underlined on the button. Cancel uses Escape. Number keys remain coordinate/numeric input. IME, paste, nonempty drafts, other focused editors, and held-key suppression protect text entry. Declared accelerators take precedence over one-shot snap keys. Wrapped option rows start at the Command area's left edge so long prompts cannot push them offscreen.

## Evidence and limits

Validation results, build identity and native observations are recorded beside this report. The new UI regression test contains 36 activation scenarios across 12 commands, plus three routes which create a radius-5 circle and verify one document Undo. The existing Polyline test retains its 24 parameter/route scenarios. These are scenarios inside Rust tests, not additional test-function counts.

The owner-state contract fixture covers all 37 registered provider groups: declaration coverage, unique IDs/shortcuts, safe value staging, resolution and cancellation without document/Undo mutation. Existing command integration suites validate their geometry/parser behavior separately.

The first broader run exposed `solid_workflows::remaining_primitive_menu_prompts_produce_analytic_volumes`: menu Box followed by the single input `2 3 4` leaves the interactive Box awaiting its next point, so the old test's expected document transaction is absent. The byte-exact pre-expansion sources reproduce the same assertion at line 248; see [baseline result](baseline-solid-workflows.json), [baseline log](baseline-solid-workflows.log), and [initial owner run](test-command-owners-initial.log). The final regression run explicitly skips this one known baseline failure and runs the other owner tests. This change does not alter Box coordinate interpretation to satisfy that unrelated assertion.

The running user release is kept intact. A separately built debug executable is used for the new native verification. No release publication, installer update or reusable skill edit is performed. Existing geometry limitations (including Polyline Helpers tracking, continuously closed preview, Center arc restrictions and exact mixed-Polycurve 3DM re-export) remain open. Full human UI acceptance and macOS verification remain unsigned.

## Proposed reusable rule (not recorded in skills)

All commands with declared options use `CommandOptionInterface` for current values, stage availability, shortcuts, pointer activation and typed names. Toggles/choices use the owner parser; value options request input before validation. Accelerators remain unique in the current snapshot and protect IME, drafts, other editors and numeric input.

This proposal requires the user's separate consent under the repository business-rule policy.

## Recorded results

- 225 distinct app/Command tests passed (224 in the main batch, plus the new unit-display regression in the final seven-test interface rerun; repeated tests are not added twice).
- 241 existing owner integration tests passed across 40 targets; the one baseline failure is explicitly filtered and two native fixture tests remain ignored. Total distinct passing tests: 466.
- Final interface tests: seven; owner contract tests: two. The Circle fixed-choice test also verifies selected-value refresh on all three routes. The Line snapshot test verifies inch display and a physical 63.5 mm line from Length=2.5.
- Clippy `-D warnings`, scoped formatting, default-feature debug build, public-source audit and diff checks passed. [Commands](commands.json), [source/build identity](source.json).
- Native Windows debug: Circle Deformable click/D, Helix T → Turns=2, Rectangle menu → Square S/click, and clean Escape cancellation were observed. [Native record](native-debug.json), [screenshots](../../../implementation/evidence/M01/command-parameter-interface-20261025/provenance.json).
- Active binary: `L:/om9-target/debug/om9.exe`; SHA256 `28915c56c1b2ab659624360939356c8a10dfcf8fafb0e96765ee5779cae76494`. Profile: `build/standalone-validation/ui-artifacts/command-parameter-interface-debug-20261025`. The new debug window is left idle for evaluation.
