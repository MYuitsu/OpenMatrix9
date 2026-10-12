# RCORE-01 topology validation — 2026-10-10

**Scope:** Owned vertex-link/oriented surface observations and bounded native OCCT CAD inspection reports, separate from representation provenance and global embedding certificates. Full original RCORE-01 remains in progress.

Rust owns vertex-link multigraphs built from canonical edge-endpoint germs.
Distinct endpoint germs on closed edges, parallel edges and self loops are
retained. Each recorded vertex requires one connected path or cycle; branching,
vertex pinches and wholly collapsed loops reject the manifold observation.
Mixed collapsed pole/seam loops remain supported. Shared-edge face orientation
is checked independently. Validated topology row/count/range limits precede
the owned graph. The existing synchronous topology Summary ABI is52 bytes.

Native C++ is necessary for live FreeCAD/OCCT topology and validity APIs; it
retains no native owner in Rust or JSON. Existing InputTree bounds64 depth,
16384 nodes/child totals and support budgets before analyzer/classifier calls.
`BRepCheck_Analyzer(shape,true,false,true)` requests geometric controls and the
exact curve-on-surface method for SameParameter edges, with parallelism off.
`BRepClass3d_SolidClassifier.PerformInfinitePoint` observes outward/inward
orientation. Native Precision::Confusion is algorithm precision, explicitly
separate from document tolerance. No geometry repair or tolerance change occurs.
Contracts were read in the matching SDK headers; primary
[analyzer documentation](https://occt3d.com/dev/doc/refman/html/class_b_rep_check___analyzer.html)
and [classifier documentation](https://occt3d.com/dev/doc/refman/html/class_b_rep_class3d___solid_classifier.html)
describe the reused native APIs.

Rust combines44-byte scalar observations into a32-byte tri-state Summary:
unknown stays unknown; a known failing prerequisite wins over uncertainty.
`topology.validation.valid_solid` requires a native Solid root, faces/shells,
native validity, closed/manifold/oriented boundaries, no loose/repeated geometry
and inspected orientation. Cavity/multi-shell validity comes from the native
analyzer, not a shell-count heuristic. Shells and compound wrappers remain
distinct roots; this inspection does not silently convert them to Solids.
Legacy `topology.solid` retains its declared native-shape-type provenance;
consumers requiring validity must use the validation report. Generic full
topology certificates remain unknown. Reports identify operation, input
identity/revision, methods/scope, reasons and source/conversion status.

World orientation is derived from immutable native orientation and the checked
current world determinant sign on every reframe. It never flips a previous
world value. Mirrored/nonuniform nested links, double mirrors, groups and arrays
were exercised. Supporting UV and source geometry remain unchanged.

**655 full Rust tests and3383 native checks passed; all17 matching host exits0.**
220 native validity/solid checks +67 surface-complex checks +3096 regressions
share plugin SHA256 `7b2bcc9cfebdda20d97e385d6524ac0792d2b97f3bd12786b04d75aa144a4fe5`. Shared FreeCAD host SHA256 remained
unchanged. Only standalone OpenMatrix9Gui was rebuilt using the shared SDK,
existing plugin cache and14 workers.

Native fixtures include box/sphere/cone/torus, a cavity Solid and its Compound
wrapper, reversed Solid, open shell/invalid Solid, inconsistent face direction,
vertex pinch, loose edge/vertex, repeated face, invalid cavity direction,
detached shells and standalone curve. Native validity queries, actual shared
vertex identity, cavity inside/outside points and source BRep/history checks
provide independent evidence. Owned reports survive replacement/document close.

Meaningful RED evidence reproduced missing fields, all six initial scalar
policy failures and a reviewed collapsed-loop false positive. The initial
Hollow fixture incorrectly called a Compound returned by cut a Solid; the
corrected fixture tests its contained Solid and wrapper separately. Production
policy was unchanged. [Review and correction](../reviews/2026-10-10-topology-validation-review.md).
[Machine evidence](2026-10-10-rhino-core-topology-validation.json) records exact
source, SDK, runtime, harness and result fingerprints.

**The full original chapter remains in progress.** Inspection reports do not
yet replace every operation/commit consumer or provide all conversion reports.
Native BRepCheck validity is not a global mathematical self-intersection or
embedding certificate. Every other original requirement remains in the
[closure audit](../reviews/2026-10-10-rcore01-closure-audit.md).
