# RCORE-01 owned Bezier support review — 2026-10-10

A fresh reviewer inspected the current increment read-only against its plan:
private Rust Bezier wrappers/JSON, shared native basis ABI, descriptor form,
native extraction through the existing checked adapters, and host tests.
No actionable implementation defect was found. The reviewer did not run
builds/tests or alter files; runtime evidence is recorded separately.

Canonical Bezier checks precede owned cloning. Degree/pole cardinality,
two-clamped-knot shape, full domain [0,1] and nonperiodicity prevent arbitrary
NURBS from receiving a Bezier tag. Shared ABI validation bounds initialized
buffers before borrowing, and rejected output remains untouched. Native
temporary control arrays live through synchronous Rust query/copy; no kernel
handle or input pointer escapes. Existing analytic/NURBS tag values remain,
with Bezier represented distinctly as form3. Restrictions, native-local
controls, location and topological orientation remain separate observations.

The initial host fixture covered a rational surface only. The reviewer flagged
native non-rational surface weight omission/rational=false as a P3 coverage
gap, not a defect. The final fixture adds a polynomial surface with explicit
original controls, null weights, rational=false and independent Bernstein/native
samples. A rational curve control/weight assertion was also added.
A final read-only follow-up verified the updated fixture and saved 76-check
report with host exit0. No actionable issue or remaining material uncertainty
was found for this increment; the earlier coverage gap is closed.

These read-only snapshots do not certify conversion reports, parameter maps,
world placement composition, solid/manifold validity or raw decoder allocation.
The full original RCORE-01 chapter and its remaining requirements stay open.
