# Initial public OM9 plugin SDK — 2026-10-21

Implemented an independent Rust SDK, versioned C ABI, native `.om9p` loader,
package inspector/builder, standalone SDK ZIP and AI migration skill. The
Windows development host executes a separately built DLL without Rhino.

The [API inventory](../../sdk/API-INVENTORY.md) distinguishes exported services,
future portable/native candidates and fixture-only functions. The
[design record](../../implementation/design/M01/PLUGIN-SDK.md) records ownership,
FFI lifetime, publication and platform boundaries. Developer instructions are in
[the SDK guide](../../sdk/README.md).

## Measured validation

Builds use `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`, rustc 1.98.1, Windows x64.
The app build for this SDK batch uses `--no-default-features`: it exercises the
real standalone Controller/document/scene/archive path, without the selected
OCCT BRep adapters. No FreeCAD or Rhino host is started.

| Batch | Result | What it establishes |
|---|---|---|
| Core/scene/SDK/plugin-host regression | 509 passed, 73 suites | Existing portable components, SDK ABI/error handling, package/bundle validation, permissions, preparation budgets and exact polyline display/picking; includes both orthographic and perspective/near-plane cases. |
| Controller plugin regressions | 4 passed | One batched publication/Undo, scene consistency, invalid/budget/parser refusal preserving state. |
| Actual packaged DLL integration | 3 passed | Live CMD service loads a real `.om9p`; line/circle/polyline geometry, selection move/delete, one Undo/Redo unit, stable IDs, cold archive roundtrip and cancellation preserving old Redo. |
| CLI cold replay | PASS, three objects | Pack/inspect, `--plugin-run`, fresh `--inspect`, equal canonical document hash; canceled/unknown commands leave the existing `.om9` file byte-identical. |
| SDK outside repository | PASS | Extract SDK ZIP outside checkout, build its example offline with no OM9 core/app sources, package that DLL and execute it in OM9 to produce three objects. |
| Source/style/skill checks | PASS | Focused cargo fmt, Git diff whitespace, source audit (0 errors), new skill validation and identical installed skill copy. |

These batches overlap and are not additive completion metrics. The test source
declares actual DLL tests ignored by default; this batch explicitly supplies
`OM9_TEST_PLUGIN_PACKAGE` and executes them with `--ignored`.

Reproduce the runtime/developer distribution batch after building the three
development packages:

```text
cargo build -p om9-app -p om9-plugin-host -p om9-hello-plugin --no-default-features -j 2
python tools/validate_plugin_sdk.py --target-dir L:/om9-target --output <new-run-directory>
```

`tools/validate_plugin_sdk.py` refuses an existing run directory and writes
observed commands, exit codes, captured output and artifact hashes. The run is
`build/standalone-validation/ui-artifacts/20261021-plugin-sdk/native/observed.json`;
the permanent [JSON receipt](2026-10-21-plugin-sdk.json) retains the measured
results and artifact fingerprints outside ignored build storage.

## Broad regression exception

The earlier five-package batch including all app integration tests stopped at
`crates/om9-app/tests/dock_layout_default.rs`:

- `d1_reset_after_every_placement_kind_recreates_the_baseline`;
- `n1_to_n8_a_layout_that_cannot_be_honoured_resets_to_the_baseline`.

Both expect a 360 px left dock after reset but observe 300 px. The dock-layout
implementation and these tests have no diff in this SDK change. They were not
altered or waived. The saved broad-batch log is
`build/standalone-validation/ui-artifacts/20261021-plugin-sdk/broad-regression.log`.
This checkpoint does not claim a fully green app/workspace regression suite.

## Deliverables and remaining scope

The standalone SDK source ZIP is
`build/sdk/openmatrix9-sdk-0.1.0.zip`; its Rust/manifests/header contents match
the ZIP used in the external build. The corresponding Windows package tool is
`build/sdk/om9-plugin-windows-x64.exe`, and the independently built example is
`build/sdk/hello-windows-x64.om9p`. Their measured hashes are in the JSON receipt.
No artifact has been published or uploaded.

The new `openmatrix9-plugin-port` skill is available in the repository and
installed under `C:/Users/nguye/.codex/skills/openmatrix9-plugin-port`. It maps
actual available APIs and reports missing capabilities; it does not promise
automatic recovery or execution of every binary-only `.rhp`.

This is experimental SDK ABI v1/developer release 0.1.0, not full Rhino SDK
coverage. Native code remains trusted in-process code. No interactive SDK menu,
panels, asynchronous/interactive point input or persistent plugin-storage API
is accepted here. Full UI pointer/DPI review was not performed. macOS library
format/target handling has structural tests only; macOS product adapters and
runtime execution remain unverified. No native kernel measurement batch was
performed or promoted by this SDK work.

Next exact reference: `sdk/API-INVENTORY.md`, candidate future services and
platform boundaries. Extend one required capability at its existing owner and
verify it through a real plugin before advertising it in the SDK.

## Surface update — round 39 (2026-10-21)

The public SDK now covers the services below; each has its own owner receipt and the
measured batch listed there. This section only records the current surface so the original
record stays readable.

| Area | Services |
|---|---|
| Document/units | `document_info`, `document_units`, `parse_length`, `parse_angle`, `parse_coordinates` |
| Objects | `add_point`, `add_line`, `add_polyline`, `add_circle`, `add_object`, `add_arc`, `add_circle_on_frame`, `add_ellipse`, `add_rectangle`, `add_polygon`, `add_interpolated_curve`, `add_nurbs_curve`, `add_mesh`, `query_objects`, `object_info`, `object_attributes`, `set_object_attributes`, `rename_object`, `set_object_placement`, `object_placement`, `object_geometry`, `object_bounds`, `object_extensions`, `set_object_extension`, `clear_object_extension`, `delete_object` |
| Layers | `layers`, `layer_create`, `layer_update`, `set_current_layer`, `clear_current_layer`, `layer_order`, `assign_objects_to_layer` |
| Curves | `curve_info`, `curve_definition`, `curve_semantics`, `evaluate_curve`, `curve_segments`, `evaluate_curve_segment` |
| Measurement | `measure_distance`, `measure_angle`, `measure_curve_length`, `measure_curve_radius`, `measure_mesh_area`, `object_summary`, `report_measurement` |
| Selection | `selected_objects`, `set_selection`, `add_to_selection`, `remove_from_selection`, `toggle_selection` |
| Project | `project_state`, `save_project`, `save_project_as` |
| History/read | `last_change` (plugin commands plus host Undo/Redo) |
| Records | `groups`, `block_definitions` |
| Parameters | `validate_parameter`, `apply_parameters`, `declare_parameters` |

Fresh batch `20261021-plugin-sdk-surface`: 10 commands PASS, 3 objects, cold `--inspect` hash
equal, cancel/unknown command preserved the `.om9` bytes, SDK ZIP built offline, 32 native
tests passed. Source total for this run: 32 native tests; workspace suites: 13 SDK + 61
host + 403 core + 5 controller.