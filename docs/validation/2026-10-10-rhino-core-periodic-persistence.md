# Rhino core: authored periodic and seam/hole FCStd persistence

Date: 2026-10-10. Baseline `83a806c` on
`codex/rhino-core-object-foundations`. **RCORE-01.T09 is verified for the authored
four-object fixture; all P0 remains in progress.** No production code changed.

FreeCAD already provides the native periodic curve constructor, OCCT BRep and
FCStd persistence. The existing kernel macro provides rational-circle and
seam/hole face geometry and predicates. This increment adds the missing true
periodic curve and fresh persistence evidence; no command or serializer was
rewritten. Legacy kernel predicates are copied unchanged into the standalone
fixture, with their exact source hash and AST comparison retained.

The existing kernel fixture first passes142 warm and79 cold checks on the current
module. Its exact saved FCStd supplies the three geometry inputs: rational R10
circle, square20 with holeR3 and open cylinderR4/H7 seam face. Deep native geometry
copies populate a dedicated document with these three features and one authored
periodic curve. Source geometry and inventory stay byte-exact during the copy
operation after target-document initialization. Source layer/attribute copying
is outside this geometric fixture's scope.

The periodic curve is a native compact degree2 BSpline with poles
[(0,0,0),(1,2,0),(2,0,0)], knots[0,1,2,3], multiplicities[1,1,1,1], weights[1,1,1]
and periodicity true. Its full domain is[0,3]; a closed nonperiodic circle is not
a substitute. Translation(7,-5,2) is retained and applied once through native
world extraction. Local and world native basis, placement, topology, bounds,
raw BRep and exact object/property inventory are recorded before assertions.

The independent locus oracle samples257 points on each cyclic quadratic span
using `Q_i(s)=(1-s)^2 P_i/2+(-s^2+s+1/2)P_(i+1)+s^2 P_(i+2)/2`.
Four analytic extrema witnesses give775 points per stage. Point-to-native-curve
distance must be≤1e-7mm before saving, after warm reopen and in the fresh process.
No native starting-span ordinal or source-to-target parameter map is assumed.
Independent bounds are local x[1/3,5/3],y[0,3/2],z0 and world
x[22/3,26/3],y[-5,-7/2],z2. OCCT bbox numerical allowance1e-7+64ULP(10mm)
is recorded separately from the locus bound and1e-12 persistence field comparison.
This is sampled deviation evidence, not a certified global continuous maximum.

The existing face predicates retain a hole whose center is excluded by native UV
trim classification and distanceR3. The seam face has a seam edge with two ordered
use occurrences and exactly two free circular boundary components. The fixture
compares supporting classes, complete available curve/surface basis, domains,
orientations and ordered loop/use graph. It does not infer a solid from periodic
U or require a prior allowed 3DM conversion to preserve analytic periodicity.

## Evidence

The [machine-readable evidence](2026-10-10-rhino-core-periodic-persistence-evidence.json)
pins source, module, macro, saved file, raw result/BRep and review hashes.

| Fresh FreeCAD process | Checks |
| --- | --- |
| Existing kernel warm control | 142 passed |
| Existing kernel cold control | 79 passed |
| Authored periodic warm/save/reopen | 204 passed |
| Authored periodic fresh-process restore | 102 passed |

All four distinct process IDs and exit0 are persisted by the launcher, with UTC
launch/completion observations. The runs use module `8b03c899…`:527 checks total, including306
authored T09 checks. The actual FCStd hash is checked before both reopens, and the
cold process checks the same module hash. Type/topology/basis/world bounds and
placement survive. All four exact object names/types/property lists and zero Undo
remain unchanged; descriptor/geometry inspection creates no extra object, property
or Undo and changes no CAD bytes. Warm same-name close/reopen rejects all four
old-session guards and accepts new ones; the cold process accepts fresh guards.

Rust541/native27/5CTest/SDK build proof from the parent Hatch checkpoint is reused,
explicitly not rerun: product source and module bytes remain unchanged. These are
not additional new T09 assertion counts. The separate
[fixture review](../reviews/2026-10-10-periodic-persistence-review.md) assesses the
literal acceptance scope and final matching artifacts.

Retained excluded attempts show two fixture issues, not persistence failures.
The first source-copy baseline was captured before target-document initialization.
Exact diagnostics show the circle root's native Free flag changes1101100→0101100
when that document is created, before any copy. Object/property/Undo inventories
and geometric records are unchanged. The final copy baseline starts after this
initialization, with no BRep fields normalized or removed. A later attempt read
`source.Name` after closing that document and raised a Python ReferenceError;
the fixture now retains the name before close. Earlier203/101 successful runs
are superseded by the final exact inventory comparison, not counted twice.
The final204/102 runs were repeated with persisted process provenance; earlier
matching results without that manifest are not counted as additional evidence.

## Remaining scope

This closes T09's specified authored periodic-curve plus seam/hole persistence
fixture. All-family FCStd semantics, arbitrary periodic 3DM representation,
layer/attribute roundtrip, semantic subelement remapping, parameter-map continuity
and general topology tolerance policy are not inferred from it. Default
`Shape.Area` remains an observation; this increment adds no new adaptive area
certificate. The previous T02 native area proof remains separate.

Same-name session rejection proves the tested synchronous identity guard behavior,
not T08 delayed worker cancellation or handle/UAF release. T07 full native
allocation coverage, T10 guarded host dependency and T11 subobject mutation owner
and standalone/subobject lifecycle remain required. Continue reuse and direct
evidence for these contracts and the remaining P0 chapters.
