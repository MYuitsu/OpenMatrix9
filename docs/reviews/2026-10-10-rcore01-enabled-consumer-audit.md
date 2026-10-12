# RCORE-01 enabled consumer integration audit — 2026-10-10

Original authority: adapter items1/3/5/6 and RCORE-01.C10 in
[the chapter](../../OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/01-object-model-kernel.md).
This is current source evidence and a continuation map, not runtime acceptance.
Full RCORE-01 remains in progress.

`Command.cpp::RustCommand::isActive` calls `om9NativeCommandAvailable`.
That function routes OM9 controllers before falling back to existing FreeCAD
commands. Existing source, Rust session engines and native kernel operations
must be reused; the missing integration is not an absent Line/Polyline solver.

| Consumer | Existing decision path inspected | Shared contract integration |
|---|---|---|
| Descriptor/owned snapshot reads | `CoreGeometryContract.cpp`, `om9_geometry_operation(1,...)` | Rust Inspect capability is already consumed. |
| Independent working 3DM | `ThreeDmModeling.cpp`, `ThreeDmPython.cpp::write3dm`, prepared result assembly | Working-V5 type/options registry is consumed; results still declare upstream conversion unknown. |
| Curve creation/Rebuild | `CurveController.cpp::available/start`, existing Rust curve session and `CoreRebuild` | Availability checks controller/document/view/edit permissions; shared operation/type/options registry integration is not evidenced there. Rebuild has its own source/generation/result gates. |
| Surface construction | `SurfaceController.cpp::available/start` and existing surface session | Controller availability and solver-specific validation exist; shared operation registry integration is not evidenced in availability. |
| Edit/Boolean | `EditController.cpp::available/start/rebuildTrim`, `EditGeometry.cpp::editInput/buildEdit` and special native leaf adapters | Shared Rust NativeEdit/Boolean type/options policy is now consumed before declaration, native copy and solve. Native tree/Mesh budgets and block/cage preflight reuse existing adapters. Matching command regressions are linked below; full GeometryResult/conversion reporting and aggregate budgets remain open. |
| Box/Sphere | `SolidController.cpp::available/start`, existing solid session | Existing input/frame/permission policy is reused; shared operation registry does not yet describe creation/backend/options. |
| Cage | `CageController.cpp::available/start`, Rust `om9_cage_command_available` | The existing Rust availability policy is not a shared geometry operation/type/options decision. |
| Selection, view, snaps, history and host command fallback | Remaining `om9NativeCommandAvailable` branches | Inventory their geometry-producing effects before assigning a kernel operation. A navigation command needs no invented geometry solver. |

The current `geometry_contract.rs::operation_capability` supports Inspect,
ImportWorkingV5/ExportWorkingV5 and native Edit/Boolean options. Registry
declaration alone does not establish all command integrations. Native context/permission checks alone cannot establish the
original registry requirement, and unit tests of unsupported enum values cannot
prove existing enabled controller paths use the decision.

Next implementation should describe the actual supported native operation
families/options in Rust, reuse their current type/source validators and route
both enablement and execution through the shared decision. Preserve commands
that legitimately prompt for missing input; reject unsupported concrete input
types/options before preview/native solve/publication with current identity.
Validate through actual host command/controller paths, including a valid
existing operation, an unsupported type/options case and cancellation/Undo.
Do not disable working commands just to make an unsupported-only test pass.

Evidence inspected: `Gui/Command.cpp`, `Gui/CurveController.cpp`,
`Gui/SurfaceController.cpp`, `Gui/EditController.cpp`, `Gui/SolidController.cpp`,
`Gui/CageController.cpp`, `Gui/CoreGeometryContract.cpp`,
`Gui/ThreeDmModeling.cpp`, `Gui/ThreeDmPython.cpp`,
`rust/src/geometry_contract.rs`, `rust/src/prepared_discrete.rs` and
`rust/tests/geometry_contract.rs`. This map does not claim exhaustive coverage
of every solver call site or close any unverified original fixture.

## Native Edit runtime follow-up

679 Rust;4456 host checks;36 actual exits0 on the matching plugin.
[Scope and remaining requirements](../validation/2026-10-10-rhino-core-native-edit-capabilities.md).
Continue creation/Surface/Solid/Rebuild/cage operation policies on their existing
solvers, then uniform output result/conversion admission. Edit integration does
not close every enabled geometry consumer.
