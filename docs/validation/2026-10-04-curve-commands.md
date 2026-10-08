# Curve implementation checkpoint — 2026-10-04

Status: IN PROGRESS. OM9-CURVE-001 and OM9-CURVE-002 are PARTIAL; this is not completion of the 58 Curve specs.

References are the local `ref/matrix9/OpenMatrix9_Codex_Spec_v1/specs/02-curve` files. User instruction supersedes PDF recheck cues; no PDF was reread.

## Implemented

Original `Line` and `Polyline` names start one Rust session through the menu/sidebar or command frame. Typed coordinates and viewport mouse points use that session. Native Part geometry commits as one transaction, with success-only history. Line supports two points and BothSides; Polyline supports straight segments, Enter, Close, PersistentClose on commit and Length. Coordinates accept relative input and mm/cm/in suffixes.

Mouse projection uses physical pixels and world XY. Near-parallel rays produce an error without discarding the command. Document deletion/switch signals cancel immediately; polling provides a fallback. Workbench activation preserves one CMD dock.

## Validation

- Rust: 19 tests; format, Clippy with warnings denied and release build passed.
- Python tooling: 24 tests passed.
- Matching FreeCAD SDK: MSVC C++ compile/link passed after the final projection change.
- Native Curve: 26 assertions passed at each of 100%, 150%, 200% DPI. Covers keyboard and Qt mouse event simulation, independent ray projection, equivalent geometry, invalid coordinates, Enter/Esc, Length, Undo/Redo, FCStd reload, rapid document switch, same-name replacement, parallel-ray recovery and workbench switches.
- Native workspace regression: 64 assertions passed after final code.
- Native original menu regression: 31 assertions passed after final code.

Reports:

- `build/curve_smoke-1/0e399e45289542429edda59eed1abb68/results.json`
- `build/curve_smoke-1.5/a701d51f4a724811af583e176f8a7fcc/results.json`
- `build/curve_smoke-2/f7f26491926a4fab9cc97e4698b6ebe9/results.json`
- `build/workspace_smoke-1/206219497c92412ba59373f361300337/results.json`
- `build/smoke-1/c81455787de841b99dcf467879ca7419/results.json`

Review found four issues: uncaught Base exception, DPI conversion, points bypassing a pending Length prompt, and polling missing rapid document changes. These were fixed and covered by regression tests. Front-view test additionally required selecting the active MDI viewport and disabling camera animation so it tests the final orientation. Ray-direction tolerance guards numerical near-parallel intersections.

## Remaining

No transient preview, snapping or Matrix construction-plane parity is claimed. Polyline Arc/Direction/Center/Helpers and automatic cursor-near-start closing remain unsupported. Advanced Line options remain unsupported. World XY, numerical limits and typed Part adapter are documented host decisions. Tests simulate native Qt events; manual desktop interaction and full OpenGL visual fidelity remain unverified. Next: complete remaining OM9-CURVE-001/002 behavior, then OM9-CURVE-003 `InterpCrv`; defer broad refactoring.
