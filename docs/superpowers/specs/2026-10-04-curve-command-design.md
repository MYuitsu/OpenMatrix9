# Curve commands and shared mouse/CMD workflow

Status: design approved; implementation in progress. Initial Line/Polyline interaction flows validated; both specs remain PARTIAL. See docs/validation/2026-10-04-curve-commands.md.

User update (2026-10-04): the design was approved. Use the existing `specs/02-curve` files as the implementation reference; do not reread PDFs or gate implementation on PDF verification. This instruction supersedes the manual-review steps below. Keep any unspecified behavior explicitly identified as an implementation decision.

## User requirements

Implement the Curve menu using the original Matrix9 command names and behavior first. Menu clicks and the command frame must invoke the same implementation. Verify geometry and both interaction paths. Defer renaming handlers and refactoring until the Curve group is stable.

## Observed baseline

The checkout at `D:/FreeCAD-src/Mod/OpenMatrix9`, HEAD `d4bc265`, contains a Rust menu/state/FFI layer and native C++/Qt workbench. `rust/src/ffi.rs` assigns menu command identifiers from icon keys and dispatches only mapped native workspace/file/view commands. `Gui/Command.cpp` dispatches those native commands. There is no Curve modeling handler or Matrix command input frame in this baseline. The existing progress ledger covers menu/workspace validation, not Curve geometry.

The source catalog has 58 Curve features in `ref/matrix9/OpenMatrix9_Codex_Spec_v1/specs/02-curve/README.md`. Its command column is evidence to verify, not an already implemented registry. The Matrix8 Book1 PDF and local FreeCAD executable exist. The prior chat supplied requirements but did not supply an implemented Curve subsystem.

## Proposed architecture

Extend the current native workbench. Rust owns command identities, command resolution, prompts, options, validated inputs, and session transitions. C++/Qt adapts viewport mouse events, the command frame, selection, previews, and document lifecycle. A narrow native geometry adapter creates and edits FreeCAD Part geometry through the host's OpenCascade integration. Geometry operations remain traceable to exact OM9 IDs; Rhino command strings are not geometry implementations.

Use one active command session per workbench. Both sidebar/menu actions and typed command text enter the same Rust resolver and session. Both viewport picks and typed coordinates enter the same input validator. Keep C++ host actions on the GUI thread. Do not evaluate command input as Python or shell code.

Alternative considered: forward commands to Draft tools. This could reduce initial code but would not by itself reproduce Matrix prompts, options, or lifecycle. Alternative considered: implement all behavior in Python. This would conflict with the established Rust ownership requirement. The proposed Rust/native adapter preserves current ownership and enables semantic tests separately from the GUI.

## Naming and menu identity

Public names remain those recovered from Matrix and confirmed against the spec/manual/native evidence, for example `Polyline`, `Line`, `InterpCrv`, `Rectangle`, `Circle`, and `Curve`. Keep existing icon resource keys separate from command text so artwork identifiers cannot accidentally become public command names. Do not change names to `_om9` during this work.

Keep menu labels and order from the original menu resources. Process feature contracts in OM9-CURVE ID order without reordering the original UI. Some catalog entries contain an option sequence (`Sketch O`, `Sketch N`, `Offset Normal`) or no recovered command text. Resolve their original invocation before registering them; do not invent names or treat an option sequence as a new command.

## Session and host lifecycle

The session represents idle, waiting for input, preview, commit, cancel, and failure states. Each feature contract determines applicable transitions; this state list does not impose undocumented preview behavior on Matrix commands.

Show the original English prompts and options once verified from source. Accept command text, option text, coordinates and numeric inputs only where the selected command supports them. Keep point conversion relative to the active construction plane and use host length units consistently. Confirm coordinate syntax, relative input, defaults and option behavior in each feature contract before implementation.

Treat transient geometry as preview, separate from document objects. A successful operation uses one native document transaction. Esc/cancel removes callbacks and preview without committing geometry. Invalid input retains the appropriate prompt and cannot create partial objects. Closing or switching the active document, deactivating the workbench, or losing the source objects cancels the session safely. Successful-only command history is updated at commit, not at invocation.

Use native FreeCAD Undo/Redo and FCStd persistence. Preserve selection and source objects unless the original operation requires changes. Document host choices separately from recovered Matrix behavior.

## Implementation sequence

1. Read the existing `OM9-CURVE-001` and `OM9-CURVE-002` specifications and record implementation contracts under `docs/features/`. Do not reread PDFs. Distinguish spec-defined behavior from host implementation choices for unspecified details.
2. Introduce the shared resolver/session, CMD widget, viewport input and native geometry adapter with these first commands. Validate basic input and every recovered option before giving a command PASS. A working straight segment alone is PARTIAL when other options remain unsupported.
3. Proceed through OM9-CURVE-003 to OM9-CURVE-058, using each existing spec and implementing the complete applicable contract. Introduce additional adapter operations as required by the selected feature rather than building speculative geometry abstractions.
4. Run Curve and existing workspace/menu regression after each verified group. Refactor only after functional equivalence is established; preserve public invocation names and rerun the same acceptance fixtures.

## Acceptance and evidence

For each feature, store mouse-path and CMD-path results independently. Use identical geometry fixtures for both paths and compare topology, endpoints, dimensions, orientation, continuity and tolerances as applicable. Tests must verify output geometry, not merely command availability or an opened menu.

Exercise Enter, Esc, options, invalid inputs, preselection/postselection, Undo/Redo, document switching, workbench lifecycle and save/reload where applicable. Test viewport clicks through actual Qt input events and coordinate conversion, not direct calls into commit logic. Test CMD through keyboard input and submission, not direct resolver calls. Inspect native runtime screenshots; distinguish simulated Qt input from manual desktop interaction in reports.

Rust semantic tests, formatting, Clippy and release build precede native compilation/linking and FreeCAD activation. Native runtime evidence is recorded in `docs/validation/` with exact environment, commands, artifacts and limitations. Mark feature results PASS only when the full verified contract and both interaction paths pass; use PARTIAL for incomplete behavior and FAIL for exercised incorrect behavior. Unexecuted tests remain untested.

## Scope and review

This design covers the Curve command subsystem and its necessary command frame. It does not require a four-viewport redesign, unrelated sidebar changes, or immediate handler renaming. Exact per-feature algorithms, defaults and options are established from source during each feature contract, not guessed by this architectural proposal.

Review decision requested: approve the shared Rust session plus native Qt/FreeCAD adapter and implementation order above. After design approval, create the implementation plan before product code changes.
