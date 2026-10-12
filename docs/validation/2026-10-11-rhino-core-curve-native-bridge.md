# RCORE-01 — Native Curve source bridge, 2026-10-11T22:26:14+07:00

**Status: in_progress. Task2/3 and the complete original chapter remain open.**

The six existing Rust/FreeCAD constructors remain in use. A minimum C++ bridge
captures full parent CAD or discrete source values, attributes, units, root/world
frames, identity/revision and ordered accepted ranges before sending bounded owned
JSON synchronously to the Rust admission API. No document/OCCT/Qt pointer enters
Rust. Original `Curve=`, `OnCurve=0.2500`, native point/direction and `Selection`
text are retained. Circle Undo restores its source/resource branch; removed
FitPoints sources are absent from the completed evidence. Circle/Ellipse path,
tangent and Circle History source checks include full content/attribute digests.

This is a native adapter exception to Rust-first: only FreeCAD/OCCT/Qt capture
and host liveness checks require C++; Rust owns admission/state/schema/budgets.
Empty units remain the existing legacy millimeter context with absent tolerance,
rather than inventing one: focused Rust RED16/17→GREEN17/17.

Two fresh read-only review findings were reproduced and fixed:

| Finding | Actual RED → GREEN evidence |
|---|---|
| Constructor metadata advertised a nonexistent numeric snapshot collection | Independent Rust consumer failed0/1, then native-admission18/18. The descriptor now says `source_snapshots_one_based`; indices1..N resolve actual native owners in that collection |
| Initial native capture errors omitted actual command/source and type | Runtime49 RED8/12, actual host exit0. Label length65537 triggered the real native attribute budget. Runtime50 GREEN12/12: `InvalidBasis` names Circle/Ellipse and Source.Edge1; rejected input creates no geometry/Undo event, repaired source retries the same session and creates correct geometry once |

The capture boundary preserves explicit typed native errors, maps OCCT/Base
exceptions to KernelFailure, uses topology-stage errors for untyped topology
failures and input admission errors for budgets. Original details remain in the
diagnostic. These tests certify the exercised budget boundary; they do not prove
every possible native exception path.

## Matching verification

- Final full Rust **874 passed/0 failed/0 ignored**, actual exit0 in125 suites.
- Standalone OpenMatrix9Gui build **exit0**; runtime50 is separate from shared
  binaries. No FreeCAD root build or full runtime copy was made. Shared SDK
  fingerprints remain unchanged.
- Native source witness from historical Runtime45 RED9/18 is **GREEN18/18** on
  runtime50 with unchanged physical/source expectations. Circle/Ellipse recover
  world center(10,20,35), radii2 or5/3. The source is a full two-edge compound
  of total length120, not only the selected Edge1. FitPoints retains all local
  cloud XYZ and selected world XYZ after branch Undo.
- Matching **517/517 checks,15 distinct actual host exits0**, serially: capture
  errors12, native sources18/cold21, accepted24/cold13, construction42/cold19,
  basis26/cold27, binding59, policy36, Circle History33, context16, Solid61,
  Edit110. Four cold hosts reopen18 FCStd archives from matching warm runs.
- The native source cold host reconstructs complete CAD parent masters and
  independently transforms cloud local XYZ via parent×native location. It also
  resolves sources using the advertised collection/base, not an assumed index.
- Immediately after build, the pre-build1401-input inventory changed only in two Gumball test files:
  `rust/tests/gumball_view.rs` and new `tests/gumball_mesh_frame_smoke.FCMacro`.
  During final Rust/native verification and recording, more Gumball C++/Rust
  sources/tests changed; the machine checkpoint lists every observed drift.
  Full current-source/plugin matching is therefore **false**. Every remaining
  frozen source/resource input matches, including all recorded Curve sources;
  results belong to preserved runtime50. Full Rust was rerun after freezing
  the1402-input inventory: every Rust/Cargo input matched immediately after that
  run, with only GumballGeometry.cpp drift in the postrun comparison. All
  Rust/Cargo inputs also matched at recording. The874-pass Rust run covers that
  current Rust checkpoint; runtime50 native evidence covers its earlier binary.
  Future concurrent edits are not covered by either checkpoint.
  Native fixture copies match current source. These expanded inventories are
  not exact compiler-input certificates. Cache/path/hard-link, missing Vulkan header and Ninja recovery
  warnings remain recorded; none was suppressed or treated as acceptance.

Runtime49's source GREEN18/cold18 and matching regressions are historical
development evidence; the totals above come only from fresh runtime50 runs.
The older Rust-only Runtime48 checkpoint and Runtime45 RED remain separate.

## Remaining required work

- All original RCORE-01 requirements remain required: adapters1–7, C01–C12, T01–T11, every enabled consumer, GeometryRef/Snapshot/Result, rational/periodic basis, trims/pcurves/seam occurrences, conversion and complete mapping, independent family read/edit/retain/export.
- Task2 remains open: generic native pick/snap sources for Line/Polyline/Rectangle/InterpCrv and original inputs across every path; full native subset indices, shared full parent snapshots across different selection observations, aggregate allocation/traversal/worker/cancellation bounds.
- Curve OM9CurveConstruction is immutable construction evidence, not a common GeometryResult or current-validity certificate. Exact output CAD master/topology/basis/units, numeric/native input snapshots, auxiliary Ellipse foci, Circle History and Rebuild results remain required.
- Curve checked GUI publication must still revalidate generation/revision/context, complete source content/units/placement/locks/ownership, output and metadata at callback barriers, with real native Undo/Redo preservation. Current source digest guards do not establish every publication barrier.
- Global analytic-to-NURBS deviation/locus and parameter mappings plus full topology maps remain unproven. Native validity and independent physical dimensions do not certify these contracts.
- Offscreen native evidence covers the named command/key/kernel/Undo/FCStd paths only. Actual mouse, preview lifetime in every viewport, OpenGL/render acceptance and all original lifecycle/cache requirements remain open.
- Matching producer inventory includes source/resources/tests selected by extensions, not an exact compiler dependency certificate. Production deployment/ABI acceptance remains pending; the isolated accepted App/GUI host reuses the one SDK.

## Next action

Finish Task2 generic native pick/snap provenance, subset/shared-snapshot and complete budget contracts; add common ConstructCurve GeometryResult and checked publication using existing solvers, then pair direct/foci/History/Rebuild outputs and audit every original RCORE-01 requirement.

[Machine checkpoint](2026-10-11-rhino-core-curve-native-bridge.json) records
source/plugin/SDK fingerprints, actual exits and all selected artifact hashes.
