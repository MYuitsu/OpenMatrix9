# Rhino 5 .3dm import/export — design for review

Status: approved by user; implemented and verified on the current SDK.

## Intent and source

User request: reread `specs/01-core` and add Rhino 5 .3dm import and export to OpenMatrix9. Working interpretation: update the specification, implement, build and verify the workbench feature. The user approved both this design and the implementation plan.

The 130 Core source-behavior entries were reviewed, with detailed File workflow review. Existing contracts remain authoritative:

- `OM9-FILE-004`: Import adds geometry to the current project.
- `OM9-FILE-001`: Export Selected exports selected objects through a file-type/name dialog.
- `OM9-FILE-003`, `005`, `006`: original Open/Save refer to .3dm; current implementation delegates to native FreeCAD Open/Save. This change adds exchange support without silently changing document Save semantics.
- `OM9-LAYER-001`: layers organize properties, visibility and locking.
- Rhino compatibility distinguishes curves, surfaces, polysurfaces, solids and meshes; mesh is not a substitute for NURBS. Geometry, trim loops and closedness require numerical/topological evidence.

Original source-derived text and imported manuals/spec bodies will be preserved. New exchange rules are explicit OpenMatrix9 decisions.

## User workflow

Add `OM9-FILE-012` (Rhino 5 3DM Exchange) to Core as an extension linking Import and Export Selected. Add File menu actions `Import Rhino 5 (.3dm)` and `Export Selected Rhino 5 (.3dm)`, reachable through the same native handlers from Command. Import requires an active editable document. Export requires supported selection in the active document.

Import reads and validates before committing, then inserts all converted objects in one Undo transaction. Cancel or conversion failure leaves existing geometry unchanged. Export writes selected whole objects using world placement to archive version 5 through a temporary sibling file and replaces the destination only after successful validation. Cancel/failure does not create a successful-history entry. Empty selection and subelement-only selection receive a clear message.

## Approach and alternatives

Recommended: native openNURBS for file I/O and Rhino geometry, with C++ conversion to/from OpenCASCADE and FreeCAD. Rust owns command identity, policy, supported-type decisions and success state. C++ owns file dialogs, kernel geometry conversion and document transactions. Python is restricted to test/automation or host bindings, not the product behavior/conversion implementation.

Alternatives: rhino3dm Python can access 3dm geometry and metadata more quickly, but a full trimmed-BRep conversion still needs a converter and adds product conversion logic outside the Rust/native architecture. Mesh-only exchange is simpler but does not meet the Core geometry contract for editable CAD geometry.

Primary library reference: https://github.com/mcneel/opennurbs. McNeel's rhino3dm reference: https://github.com/mcneel/rhino3dm. Dependency version and compatible build/license packaging must be pinned during implementation.

## Geometry and metadata contract

Implement exchange for points, line/polyline/arc/circle and NURBS curves, NURBS surfaces, trimmed BRep faces and joined shells/solids, and polygon meshes. Preserve topology and trims; do not silently tessellate CAD geometry. Convert extrusions through their BRep representation. Place geometry in world coordinates, retaining transformed object placement.

Normalize imported length units to FreeCAD millimeters using the archive's unit system. Export millimeters with explicit tolerance metadata. Unitless/custom units require a user-selected scale rather than a guessed unit. Preserve object names, layer paths, object/layer color, visibility and lock metadata through FreeCAD properties/groups; layer behavior must be reconciled with existing layer UI rather than claimed from metadata alone.

Block instances, annotation, plugin objects and Matrix builder/history records require separate support decisions. Unsupported input must produce a preflight report and fail the operation without silently dropping objects. Unsupported export must fail before replacing any destination. Exact Rhino/Matrix parametric history and materials/textures are not promised by geometry exchange.

## Validation and deliverable

First write failing semantic and native integration tests. Test actual file archive version 5; import into a populated document; selected-only export; millimeter/inch scaling; names/colors/layers; placements; NURBS weights/knots; trimmed face with a hole; solid closedness/volume; mesh counts; malformed file; unsupported objects; cancellation; rollback; one-step Undo/Redo and FCStd save/reopen.

Use independently created openNURBS fixtures as well as export/read-back; a converter validating only its own round trip is insufficient. Menu and Command must execute the same handler. Build OpenMatrix9 against the current H: FreeCAD SDK and run in FreeCAD with an isolated test profile. Opening the result in an actual Rhino 5 installation remains a separate interoperability check when Rhino 5 is available.

Deliver source, updated Core extension/index and feature records, dependency/license packaging, native build, packaged module and test evidence. Preserve previous build validation documents and unrelated user changes. Record only verified support in the progress ledger.

## Review decision

Confirm this CAD-preserving scope and native approach before implementation. After design approval, prepare the implementation plan and execute inline unless the user explicitly requests delegation. No product dependencies or source code have been changed at this stage.

## Implementation rulings

Native dialogs and transactions remain in C++. Python is a FreeCAD host-binding/staging adapter only; openNURBS and OCC conversion remain native. Public native methods are `import3dm(path, documentName, customUnitMm=0)` and `export3dm(path, documentName, objectNames)`. Rhino archive version 5 is represented by openNURBS as version 50 on readback. Hidden locked objects retain the lock as an explicit user string. FreeCAD triangle storage retains original quad topology in exchange metadata until edited. Numerical conversion uses source archive tolerance, demonstrated by the independent extrusion fixture.
