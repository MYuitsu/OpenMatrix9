# Top11 Split along an existing seam — 2026-10-31

The initial connected-boundary splitter always joined faces along their original common edge. A cutter coinciding with that seam therefore produced no split. A new regression demonstrated the failure through the Top11 controller before the fix.

The OCCT adapter now checks shared original edge segments against the supplied cutters and treats a fully coincident segment as a cutting boundary. Original seams that the cutter only crosses remain connected. An area-overlapping surface/BRep cutter is refused to avoid interpreting the perimeter of a coincident face as a cut. Edge comparisons are capped by the smaller of the node budget and 250,000; a budget failure does not publish document mutations. This work is synchronous, operation-local OCCT topology and no native handle escapes.

Windows x64, OCCT 8.0.1, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- Two joined faces split along their common seam with a line or a perpendicular BRep face yield two one-face pieces of 100 mm², with Undo. Before the fix the line case failed with no intersection.
- A cutter covering only half the seam does not disconnect the faces; a coplanar overlapping BRep cutter is rejected without a transaction and the command remains active. A seam cutter plus a crossing cutter yields four 50 mm² pieces in one Undo.
- The simulated viewport cutter-selection test passes for cuts both along and across the seam. The Top11/options/BRep/snapshot suites passed 114 tests before the additional four-piece fixture, which then passed separately. The kernel `--features 3dm-brep --test l2_01_10` suite passed 69 tests.

Limits: the overlap threshold and whole-edge comparison have only these planar fixtures; general near-coincident, partial-edge, periodic, non-manifold and solid-cutter geometry and comparison performance remain open. The conservative area-overlap refusal can also exclude valid solid cutters. Original Matrix closed-polysurface capping behavior, Isocurve/Direction/Shrink, 3DM, SDK plugin parity and physical M03-UI acceptance remain unverified.

SHA-256: `measurement_bridge.cpp` 3f44a235e3d8bca9b03cd84644ec5e39a314889f5460571f548f90ef6128cfaf; `tests/top11_brep.rs` 6442a5a5b329dad7094ed57efa8f8b205c6c248c92385f1a90491c50539792a6; `ui/chrome.rs` 595f9fbe5e4291496effd07ab47e37cc800b270fbe446f3d459776952f7a6883.
