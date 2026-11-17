# Larger trimmed surface — scoped support

Trim metadata repeats the underlying NURBS surface. The former shared 16 KiB
name/extension bound prevented a 28×28 planar surface with a simple outer loop
and hole from entering a default document. The trim slot now has a separate
128 KiB default bound; unrelated names and extension payloads keep their prior
limit. Tighter configured name limits below 16 KiB remain in force, and the
trim bound never exceeds the document byte budget. Existing resource and
validation admission still applies. The representation and archive schema are
unchanged, but an older build with the 16 KiB bound can reject newly saved
documents containing larger trim payloads.

The fixture has a payload above 16 KiB and below 128 KiB. It was verified for:

- document insertion with Undo/Redo and `.om9` encode/decode;
- scene display and selection on material versus the hole, including an
  incremental scene update compared with a complete rebuild;
- ExtractWireframe after reopen, eight source-boundary endpoints and one
  Undo/Redo; simulated managed pressure preserves the document and Redo;
- ExtractIsocurve after reopen, splitting at the hole and Undo/Redo;
- STL after reopen, checking area and points on material, inside the hole and
  outside, with document and Redo retained;
- real 3DM export/reimport using the compatible P1 Release adapter, checking
  material, hole and outside rays while retaining document and Redo.

The scene's incremental bound previously counted all NURBS pole arrays as
trim edges; a 28×28 source falsely exceeded the point budget. A nonallocating
preflight now counts arrays in the two top-level trim-loop fields. If keys are
escaped, repeated or structurally uncertain it falls back to the old more
conservative count. Tests cover ordinary and uncertain inputs and compare the
incremental result with a full scene rebuild.

The core library suite passed 377 tests (one ignored); the full archive and
scene suites passed; isocurve 26, wireframe 77 (four diagnostics ignored), and
the complete STL suite of 67 passed; eight actual 3DM file fixtures passed.
Other extension limits, the 128 KiB trim cap and rejection without mutation for
an oversized trim were checked. This does not establish all NURBS surfaces,
arbitrary trim loops, desktop, actual physical out-of-memory, native temporary
peak or independent Rhino compatibility. The existing debug native 3DM adapter
still rejects the packet schema and was not replaced while user sessions were
active. ExtractIsocurve's pending surface is now admitted under managed memory;
physical memory pressure remains unverified. [Follow-up](2026-11-13-isocurve-pending-admission.md).
## Display up to 1,024 trim edges

Before this change a planar 708-edge outer loop plus a four-edge hole failed
scene preflight with `Budget("bytes")`: the raw crossing preflight could not
represent the edge count and fell back to a much larger worst-case estimate.
The display path now accepts at most 1,024 combined trim edges with its own
bounded topology check of at most 523,776 unordered pairs. The existing
707-edge limit for boundary-extraction commands is unchanged. The raw
preflight and builder use the same display edge limit, and the admitted bound
is checked against built display usage in the tests.

Plane fixtures of 712 and 1,024 total edges built trimmed display meshes;
triangle centroids did not enter the hole, and output usage stayed within the
preflight estimate. After save/reopen, the 712-edge plane used trimmed display
instead of fallback; simulated clicks selected material but not the hole or
outside. A 712-edge cylindrical surface with a hole allowed Surface Snap to a
rear plane through the hole in simulated Wireframe and Shaded, followed by
Line and Undo/Redo. These are geometric/egui checks, not a GPU-present or
physical desktop acceptance test; arbitrary curved-display approximation,
near-boundary numerical cases and native/process peak remain open.

To retain source-based fallback for the new range just beyond the display
limit, its bounded topology limit is now 1,536 edges / 1,178,880 pairs.
Reopened 1,025-edge planar and cylindrical fallback fixtures passed the
selection, overlap and Surface Snap regressions. Five warm debug timings on
the existing machine were 23.18–23.38 ms for the 1,025-edge convex no-hole
case and 63.39–63.75 ms for the 1,536-edge concave no-hole case; these measure
the helper only, not end-to-end frame latency or a worst-case bound.

Validation: scene library 46 passed; fallback-filtered app library 11 passed
with one timing diagnostic ignored in that batch (the diagnostic was then run
separately); selection sequence 9, Surface Snap UI 28, rubber-band 10 and
Wireframe 87 passed, with eight Wireframe tests ignored. Five filtered core
tests, the no-default check, existing debug build and public source audit
passed. The focused Surface Snap test was extended after the full UI batch
and passed again with eight variants; the broader batch count predates that
test-only extension.

### Offscreen GPU plane and cylinder

Two authored 712-edge fixtures (708-edge outer loop and four-edge hole) were
saved to OM9 and reopened by the headless renderer: one plane and one cylinder.
For each, the appropriate orthographic view was rendered on an available GPU
and compared with an empty-scene render at three small pixel patches. The hole
and exterior matched the background, while the material patch differed by
more than the test threshold. Both GPU tests ran and passed without the
no-adapter skip. The input and output files were in unique temporary folders
and cleaned up. This checks selected pixels in two offscreen views, not every
pixel of either surface, a window swapchain, interaction on desktop, 1,024-edge
GPU output, general curved trim, or native/process peak.

### Surface Snap on 1,024-edge trim material

A reopened 712-edge planar trim used the new display mesh, but Surface Snap
still failed with `trim topology exceeds verification limit`: source
refinement retained the previous 707-edge verifier. It now uses the same
bounded 1,024-edge topology verifier as the display path.

Reopened plane fixtures of 712 and 1,024 edges and a 1,024-edge cylinder
fixture have material and hole probes. Simulated Wireframe/Shaded interaction
snapped to material from the source and did not snap to the hole in the
tested cases. Continuing Line from the valid snap and Undo/Redo passed;
drawing and Redo were checked before the Line commit. The 30-test Surface
Snap UI batch passed; after it, the cylindrical hole variants were added and
the focused test passed again. The no-default check, existing debug build and
public source audit passed. These are authored fixtures and simulated input,
not all trim shapes, physical desktop, actual OOM or native peak.
