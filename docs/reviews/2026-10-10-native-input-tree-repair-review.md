# Independent review — native input-tree topology repair

**Verdict: approved for the bounded RCORE-01.T07 increment.** T07 and all P0
requirements remain open.

The reviewer verified that out-of-range `ON_BrepEdge::m_ti` references reject in
preflight before the orientation copy. An invalid face-loop reference is rejected
by `ON_Brep::IsValid()` on the independent copy before `SolidOrientation()` is
called. The pinned OpenNURBS 5 behavior supports the tombstone ruling: cleared
edge/trim back-pointers fail `IsCorrupt`, and the previous copy-plus-validation
route rejects the same state.

The reviewed validation records are 64/64 native checks, 21/21 checks in a fresh
FreeCAD host process with exit code 0, and 572 Rust tests across 73 suites. The
reviewer independently matched the four recorded source fingerprints and all
664 package manifest entries. The reviewer did not rebuild or rerun tests.

The existing Line/Polyline commands and `Part.makePolygon` construction are
reused. The review does not certify archive decode/member staging, process-wide
allocation, analytic/mesh/cloud/cage paths, arbitrary native owners, worker
lifetime, full T07, or all-P0 completion.

Current implementation fingerprints and process/module hashes are recorded in
[`2026-10-10-rhino-core-native-input-tree-followup.json`](../validation/2026-10-10-rhino-core-native-input-tree-followup.json).
