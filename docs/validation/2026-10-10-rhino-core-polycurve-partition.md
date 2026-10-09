# Rhino core polycurve partition — 2026-10-10

Status: **validated bounded increment; all P0 remains in progress**. Parent `04d64cc`.

The missing behavior was working 3DM import: an ordinary `ON_PolyCurve` went
through whole-curve GetNurbForm and became one OCCT edge. The existing FreeCAD
wire, reversal, Line/Polyline commands, transactions and FCStd persistence are
reused. A separate flat ordinary-polycurve adapter now converts each admitted
child with the existing curve factory and constructs a native wire. The legacy
single-edge `importCurve` and Hatch callers remain unchanged.

Safe Rust owns finite/domain/budget/ordinal/endpoint/joint/count validation and
copied report semantics. The ABI passes values only, without pointers, callbacks,
native owners or allocators. C++ owns live native curves/wires, storage-envelope
extraction, OCCT construction and serialization; this is the permanent native
adapter exception. Python supplies host fixtures and existing bootstrap only.

Admission permits exact owned Line/Arc/NURBS/Polyline children. It checks native
container envelopes and every raw child before parent metadata or CRC. Nested,
reference, missing or unverified children are explicitly retained or rejected.
Every source segment is admitted before MakeWire. Parent intervals and each
child's supporting domain remain separate; a raw exported wire has no invented
global parent domain. There is no Join healing, midpoint replacement,
AppendAndMatch, RemoveNesting, fitting or whole-polycurve NURBS fallback.

The completed wire is inspected again. Oriented curve endpoints, actual shared
native vertices and traversal coverage are separate facts. Ordered traversal
must cover every edge occurrence; a branch/disconnected subset cannot export
partially. Both archive and replacement export reuse one checked wire exporter.
Each unique_ptr transfers ownership only after Append succeeds. Actual V5
reread checks partition and oriented endpoints instead of just object count.

Copied conversion observations are in the prepared payload before the host's
existing atomic commit. The archive report explicitly marks source-file units,
the scale to millimetres and observation before unit scaling. These observations
are not live permissions or a durable source/EdgeN identity. Global locus,
parameterization, topology, parameter map, attributes and deviation remain null.

## Authored T05 and measured scope

The asymmetric fixture has line A=(-5,0,0)→J=(0,0,0), then an exact rational
quadratic R10 quarter arc J→B=(10,10,0), centered at (0,10,0). Its poles are
[(0,0,0),(10,0,0),(10,10,0)] and weights [1,√½,1]. Reversal must give arc B→J,
then line J→A. Native source child domains [0,5] and [11,19] differ from parent
spans [2,3] and [3,7]; native source reversal also checks [-7,-2] parent intervals.

Independent endpoint, line and 33-point arc-radius observations use the unchanged
**1e−7 mm** geometric bound. Rational degree, poles and nonunit weights are
checked. Source/sibling fingerprints, shared joint, complete occurrence count,
one Undo/Redo, double reversal, FCStd reopening and actual working V5 exchange
are verified. The committed imported wire also survives Undo/Redo and fresh
FCStd reopening with typed CAD classification and its copied basis/ledger.

Cold serialization initially differed in five near-zero floating values, with
maximum 8.151490101317698e−17 and no categorical change. Finite serialized values
therefore use an explicit **1e−12** comparison bound; degree, multiplicities,
orientation and counts remain exact. This is not a bit-identical float claim.
Sampled geometric checks do not establish a continuous/global deviation bound.

**RCORE-01.T05 is verified for this authored open two-segment fixture.** No segment
merge or healing occurs, so its conditional merge warning is not triggered.
Broader polycurve family/attribute parity and the full P0 contracts stay open.

## Verification

| Check | Result |
|---|---|
| Full Rust suite / Clippy | 523 passed; exit 0 / 0; existing warnings only |
| Focused portable contract and value ABI | 13 + 8 passed; independent review |
| Python tooling | 49 passed, 1 skipped; exit 0 |
| Native polycurve fixture | 403 checks, build/run 0 |
| Existing native query / exact kernel | 159 / 59 checks passed |
| Native regression suites | 7 passed: basis, geometry, trim mapping, multiple shells, modeling exchange, merge, Hatch loop edit |
| Matching FreeCAD module | Configure/build 0; SHA-256 `61708e51c37e9dc150d355b968a5be9ab29886858ae9ce0cea108fc2b87e5e09` |
| Fresh application processes | 382 checks in 6 processes, all exit 0 |

The application checks are polycurve warm 69, cold 37, modeling exchange 32,
existing Line/Polyline 27, spline editing 41 and trim domains 176. Native negatives
include source gaps without repair, missing parent storage, nested retention/
working rejection, uncovered branch export and unchanged destination bytes.
Armed raw-child traps prove over-budget and null-CV children are rejected before
Domain/CRC. Actual nested working import preserves document objects, Undo count,
existing CAD bytes, original source and sibling when rejected.

The original working-import RED is retained in immutable run
`d0725451270c4c2cb8a529e9da2dba20`: 34 baseline checks pass, then the actual staged
BRep has one Edge instead of a two-occurrence Wire. Genuine Rust stub RED has
13 failures; value-ABI stub RED has 6 failures/2 passes. Native stub build succeeds
and execution rejects missing implementation. Test compile/API-name mistakes,
overstrict cold float equality, a misspelled modeling fixture and the consequent
module-lock link failure are recorded separately and excluded from acceptance.
The harness now checks that a macro exists before launching a test process.

[Evidence and exact fingerprints](2026-10-10-rhino-core-polycurve-partition-evidence.json),
[portable review](../reviews/2026-10-10-polycurve-rust-review.md) and
[native review](../reviews/2026-10-10-polycurve-native-review.md) preserve the
source/runtime linkage. The 607-entry JSON/YAML catalog and command support
statuses are unchanged; package manifest, capability lookup and live acceptance
matrix are synchronized.

## Remaining limits

The 4096-segment/1MiB Rust policy bounds logical owned report data, excluding
native copies, container overhead and aggregate worker budgets. Native callers
must provide live immutable owners; pointer/capacity envelopes do not prove
foreign pointer validity, aliased child ownership, archive deserialization or
prior instance cloning/transform safety. Early raw-child gates are partial T07
evidence, not full allocation/commit-path coverage.

Arbitrary closed/nested/PolyEdge/reference graphs, global parameter maps,
continuous locus proofs, userdata/History and all-family preservation remain
unverified. No new Rhino 5 runtime parity or whole-process memory-safety claim is
made. The next P0 slice reuses existing instance/transform support for the exact
T06 definition/sibling/world-coordinate fixture.
