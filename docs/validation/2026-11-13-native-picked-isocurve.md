# NativeBrep selected-face ExtractIsocurve

## Scope

A root NativeBrep's selected mesh-face hit now retains its viewport ray and
source/scene/selection binding. ExtractIsocurve converts the retained source,
resolves an exact face by the local-space ray, checks the exact hit against the
original world/view pick, and re-resolves the same ray against the unchanged
source at commit. A nonfinite, ill-conditioned, stale, ambiguous or too-distant
selection is refused. A hit from a different cursor is refused. This is a
source-ray decision, not a persistent mapping from mesh-face index to CAD face.

## Evidence

`crates/om9-app/tests/isocurve.rs` exercises an authored NativeBrep box after
OM9 save/open: two distinct picked faces and translated placement produce
curves on the expected face, via CMD and menu dispatch, with Undo/Redo. Managed
memory refusal at start and commit preserves the document and Redo and allows
retry; changed selection or deleted source is refused. A deliberately wrong
display triangle covering a source hole is refused. A NativeBrep cylinder
after save/open is checked in two display profiles and orthographic/perspective
views with scale/shear placement; sampled curve points and Undo/Redo pass.

`crates/om9-app/tests/solid_difference_ui.rs` simulates Ctrl+Shift viewport
selection of a reopened NativeBrep box, then uses CMD or the Curve palette,
focuses the command input and presses Enter. Both routes produce the tested
face curve and pass Undo/Redo. This is egui simulation, not physical desktop.

One sequential debug run for the two small cylinder profiles measured start
6.94/5.93 ms and completion 24.21/22.68 ms. The samples are uncontrolled and
do not establish release, desktop, worst-case or larger-model latency. The
NativeBrep source conversion and source-ray work still run synchronously.

The full Isocurve integration group passed 50 tests, one timing probe ignored;
the filtered controller/UI tests passed 19 before the final hit-projection
guard, and the targeted egui viewport test passed for both routes. The
no-default-features app check and existing debug executable build passed;
the public-source audit reported zero errors. The user approved the scoped
guidance entry, and repository and installed copies match. Failures cover simulated managed
pressure, not actual allocator or native process exhaustion.

## Limits

Nested instances, arbitrary imported Rhino topology, tolerance edge cases,
very close competing faces, extreme affine transforms, physical desktop,
native peak memory and general performance are not certified. The viewport
pixel tolerance bounds how far a display hit may be from an exact source hit;
it is not a global proof that every approximate display identifies the user's
intended face. A native intersector call cannot be interrupted midway. A curve
already created from one fixture does not prove all display profiles or export
paths. Plan A remains open.

## Large NativeBrep mesh validation

An authored NativeBrep sphere with a 30,016-triangle display survived OM9
save/open, viewport picking, isocurve extraction and Undo/Redo. Two sequential
debug samples before this change took 1,169.75/1,126.15 ms to complete the
command. One instrumented sample attributed approximately 409 ms to preparing
the transaction and 557 ms to the subsequent admitted scene/commit path;
conversion and extraction occurred earlier. Instrumentation has been removed.

`Mesh::validate` now aggregates face capacity and node usage, instead of
rechecking the full multi-dimensional budget for every face. Every face is
still checked for length, index bounds and duplicate vertices; node and byte
ceilings are checked early, arithmetic remains checked, and the total is
admitted against the same limits. This affects all meshes and NativeBrep
displays, not only ExtractIsocurve. A focused capacity test checks spare face
allocations, byte/node refusal and malformed faces; six hatch/mesh accounting,
147 retained-validation and 29 atomic-transaction tests passed. The Isocurve
integration group passed 50 with two timing probes ignored.

After the change, three instrumented debug completion samples were
692.16/657.86/646.76 ms, and one final sample without instrumentation was
697.35 ms. These are sequential, uncontrolled samples on one machine; they
do not establish release, desktop, concurrent or worst-case performance. The
operation is still synchronous and can visibly pause the UI. Actual OOM,
native peak memory and physical desktop remain untested.

The NativeBrep viewport CMD/palette egui test, no-default-features check,
existing debug executable build and public-source audit also passed after the
change. No new build directory was created.

### Aggregate face capacities before byte conversion

The same validation now sums face index capacities with checked arithmetic,
checks that sum against the remaining byte ceiling, and converts the total to
bytes once. It continues to inspect every face and rejects over-budget input;
the exact-boundary, one-byte/one-node-over, spare-capacity and invalid-index
fixtures passed. This is a general `Mesh::validate` change. The 147 retained,
29 atomic-transaction, six hatch/mesh, 50 Isocurve and the NativeBrep viewport
CMD/palette tests passed; two Isocurve diagnostic probes were ignored in the
ordinary suite.

Three subsequent non-instrumented debug samples of the same sphere completed
in 622.67/611.63/599.16 ms, compared with one earlier non-instrumented sample
of 697.35 ms. These sequential measurements are not controlled for machine
load and do not establish release, physical desktop, worst-case or concurrent
latency. The operation remains synchronous. OOM and native peak remain open.
The app no-default-features check and existing debug build passed; the public
source audit reported zero errors. No new build directory was created.
