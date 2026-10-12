# Rhino core Hatch input preflight

Baseline32a72a6. Continue approved all-P0 plan; exact T07 remains open globally.
Reuse existing native Hatch editor/constructor/topology/export and shared Rust
NURBS basis validator. Demonstrated gap: native NURBS constructed before malformed
CV/knot/weight checks; archive copies precede current-field validation.

Safe Rust parses bounded owned loop input, validates every nested NURBS with the
existing native basis policy, and returns status through a synchronous byte ABI.
Retain boolean values in the existing bounded JSON parser. C++ serializes values,
calls preflight before archive reads/copies/native constructors, then retains its
native source identity, class, metadata, closure and topology validation. Python
only provides host fixture/API glue. No new editor or solver.

- [x] Capture actual public boundary RED: malformed NURBS versus nonexistent
  source archive proves source access currently precedes basis rejection; output,
  document and Undo remain unchanged. Valid basis must still reach source access.
- [x] Portable tests cover boolean distinction, rational/nonrational row width,
  positive finite weights/homogeneous overflow, knot count/order/multiplicity,
  exact active domain, nested/later-row failure and bounded malformed ABI.
- [x] Implement Rust preflight and call from loop/current-field application,
  public boundary and writer selected/dependency/independent-copy preflight.
- [x] Verify RED→GREEN, complete Rust/Clippy, native Hatch regression and matching
  SDK + fresh host replay; independent review, scoped report/spec sync/commit.

Review focus: omitted outer knots/expanded ghost poles, rational bool and tuple
width, later invalid child before earlier native construction, native read/copy
ordering, numeric limits versus aggregate budget, unchanged geometry/Undo/file
sentinels, ABI lifetime/UTF8/length/no unwind. Raw archived parser/proxy/trim and
all-family worker allocations remain full T07 gaps; do not mark T07 verified.
