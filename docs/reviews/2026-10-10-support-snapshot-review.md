# RCORE-01 owned NURBS support review — 2026-10-10

A fresh reviewer inspected the Rust owned model/JSON ABI, native support reader,
topology integration and host fixture read-only. A follow-up inspected the final
3D curve integration and legacy face-companion reuse. No actionable implementation
defect was found. The reviewer did not build or launch native tests; accepted
runtime evidence is recorded separately.

The reviewer checked synchronous native-buffer borrowing, validation before
Rust cloning, caller-owned output, two-pass consistency, short-output rejection
atomicity, rational weights, U-major surfaces, compact periodic axes, separate
full/restricted domains and aggregate budget rejection. OCCT `UVBounds` was
inspected for source-cache writes; no new mutation finding was identified.
Qt numeric roundtrips retained the finite values examined; signed zero normalizes
to zero, so this is geometric numeric preservation rather than bitwise JSON
preservation of IEEE sign-zero bits.

The initial runtime fixture did not explicitly assert native periodic axes,
distinct trim/full domains, reversed uses, nonidentity locations or unsupported
outcomes. Direct surface JSON ABI coverage was also missing. The final fixture
adds those native controls, a rational 3D circle, a restricted periodic 3D edge,
an over-budget surface and exact BRep/Undo/Redo invariants. Rust tests add direct
rational surface JSON copying/rejection atomicity and actual serialized-limit
rejection of a valid basis containing long subnormal decimals.

The final integration keeps legacy `basis` fields compatible and removes the
second C++ pole/axis JSON builder. A standalone curve uses the same Rust-owned
serializer as pcurves. A face reuses the already captured owned payload without
another native extraction. The final follow-up found no actionable regression.

Ruling: the aggregate bound counts unique copied supporting payloads, not the
complete expanded Qt descriptor or every native allocation. Free-boundary views
repeat edge-use/pcurve JSON; legacy/root aliases can also repeat the transported
payload. Neither this bound nor the companion basis cap is a global allocator
certificate. Caller pointer accessibility/lifetime/nonoverlap remains an ABI
obligation, not a property that the envelope validator can prove.

Analytic/Bezier/offset/extrusion/revolution supports remain explicitly uninspected;
this increment makes no implicit conversion, world-frame composition, solid
certificate, parameter-map or full chapter completion claim.
