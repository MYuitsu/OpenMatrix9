# Native Builder conversion source review — 2026-10-11

Scope: current uncommitted native Builder lineage/observations, independent Rust
result validation and publication integration. Original RCORE-01 is still open.

An independent read-only reviewer found one P2 issue: unavailable surface
projections, and native curve/surface projector exceptions, could reject an
otherwise valid shape or clear a valid History output. Rust initially accepted
unavailable queries only for edges.

The fix records bounded source-parameter or source-UV failures with explicit
KernelFailure diagnostics and unavailable-query evidence. Native query exceptions
are caught locally. Rust checks failure values, domains, reason/kind combinations
and aggregate attempts, and sets recorded_queries_complete=false. Successful
measurements with excessive residuals still reject publication, as do source,
native geometry/topology and byte/count budget failures. No tolerance is raised.

Re-review confirmed the P2 fix on source and identified no other independent
important defect in this addition. The reviewer did not run builds or FreeCAD.
Portable surface-failure RED13/14 became GREEN14/14. Native surface-failure
injection remains untested; final matching runtime evidence is maintained in
[the conversion validation report](../validation/2026-10-11-rhino-core-builder-conversion.md).

Global deviation bounds, full-domain parameter maps, all-family result/lifecycle
contracts and complete attributes/preservation remain required original work;
this review does not close the chapter or certify those requirements.
