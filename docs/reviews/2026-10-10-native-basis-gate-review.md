# Selected native NURBS basis gates — independent review, 2026-10-10

Requested under `superpowers:requesting-code-review`, against baseline `fcdb0ac`.
Scope: `Gui/NurbsBasisAdapter.h/.cpp`, the new gates in
`Gui/ThreeDmCurves.cpp` and `Gui/ThreeDmSurfaces.cpp`,
`tests/native/nurbs_basis_adapter.cpp`, and their borrowed Rust boundary in
`rust/src/nurbs_basis_ffi.rs`. The separately implemented native descriptor
companion is excluded. Only this report was edited; the reviewer did not build,
run native/GUI processes or mutate production source.

## Assessment

No blocking defect found in the selected direct-basis gates or borrowed ABI.
The additional native envelope/profile tests have been source-reviewed; final
native and matching SDK/application evidence passed after the Rust policy move.
The implementation protects selected raw and converted basis branches; it does
not satisfy literal rejection before every native allocation or all-consumer
RCORE-01.T07. The missing consumers and allocation boundaries below must remain
explicit in acceptance evidence.

## Boundary and storage checks

- Rust owns degree/count/cardinality, knot/domain, finite Euclidean poles,
  positive weights, homogeneous-overflow and logical-payload policy. C++ only
  reads native metadata/storage, extracts owned basis vectors, calls the shared
  gate and retains the existing geometry converters. Native extraction is the
  necessary permanent OCCT/openNURBS adapter exception to Rust-first.
- For openNURBS curves, malformed signed order/count fields are mapped without
  calling `Degree`/`KnotCount` first. The first Rust count gate bounds degree and
  poles before native validity traversal. Dimension/rational flags and positive
  minimum strides are checked before computing the last reachable CV span.
  Owned CV/knot capacities are checked before `IsValid` or extraction reads.
- Surface dimensions are positive native `int` values on the asserted 64-bit
  ABI, so their first product fits `size_t`; Rust checks exact flattened product
  and the total-pole limit before native index/stride traversal. After the
  `degree<=25`, `total poles<=1e6` policy, stride products with positive `int`
  strides cannot overflow the 64-bit arithmetic. The final positive reachable
  span is additionally bounded by `INT_MAX`, protecting later native signed
  offset arithmetic.
- Native CV and knot pointers must be nonnull, double-aligned, have a bounded
  readable span and not wrap address arithmetic. Negative owned capacities
  reject. Capacity zero is the documented SDK convention for externally managed
  storage, not proof of its capacity: the native owner must guarantee that the
  allocation is live, initialized, sufficiently large and immutable throughout
  validation. Forged/dangling pointers cannot be certified by arithmetic guards.
- Native validity runs after envelope guards, then the exact distinct-knot count
  and logical bytes are checked before extraction vectors reserve storage.
  Raw storage scans remain native and bounded; the reusable value policy runs
  in Rust on the independent owned scratch snapshot.
- Scratch snapshots store only `double`, `size_t`, arrays and vectors. They hold
  no FreeCAD/document/Python/OCCT pointers. Native source objects are caller-owned
  and must not be mutated concurrently. Neither extraction nor preflight
  clamps, normalizes, copies or changes the source geometry.

## Knot conventions and selected gate placement

openNURBS omits the two outer knots. The adapter duplicates the stored first and
last knot while collapsing adjacent equal values into multiplicities. This
restores expanded structural cardinality without expanding the full knot vector.
Periodic openNURBS ghost poles are validated in this expanded **nonperiodic
structural view**, retaining every stored pole. This is explicitly distinct from
native periodicity preservation and must not be used as a semantic periodic
descriptor. OCCT compact periodic axes preserve their native periodic flags and
are validated under compact periodic cardinality. Domains and both U/V grids
are retained; surface poles and weights use U-major ordering.

Direct raw ON curves/surfaces are checked before `GetNurbForm` copying. Converted
forms are checked before clamping and again afterward, before OCCT output pole,
weight and knot arrays are constructed. Export checks direct native B-splines
and their converted basis before ON output construction; periodic-to-nonperiodic
changes receive the later output-input gate. 2D pcurves share the curve gate
with an explicit zero Z component. The existing trim/seam pairing and surface
parameter mapping are reused, with a gate after the revolution affine UV pole
mapping.

The reviewer identified the revolution profile branch as a gap: `rev->IsValid`
and a temporary `GetNurbForm` could read/copy a raw ON NURBS profile before the
curve adapter gate. The repaired source checks a nonnull profile and gates raw
ON NURBS profile storage before those operations. The independent revolution
fixture exercises a valid baseline, an oversized profile count and a null CV
pointer through the production importer. Source review and the final passing
native suite close this selected gap.

## ABI, errors and budgets

By-value C++ records mirror Rust `#[repr(C)]`; C++ asserts 64-bit field sizes,
alignments and every field offset for limits, axes, curve/surface inputs and
surface counts. Vectors are initialized and aligned before being borrowed by a
single synchronous call. Neither side retains pointers, frees the other's
allocation or exposes native objects through this ABI. Rust checks limits and
counts before creating slices, then checks nonnull/alignment/address ranges;
zero weight count requires a null pointer. The remaining live-allocation and
immutability obligations remain the caller's unsafe contract.

Rust status values return before C++ throws `ExchangeError`; native exceptions
do not cross into Rust. The validator has checked arithmetic and no intentional
panic paths. Release `panic=abort` remains the crate policy, not an error recovery
or whole-process memory-safety guarantee. Existing C++ operation boundaries
retain responsibility for catching native kernel exceptions.

The selected OM9 native policy is degree at most 25, total poles at most 1e6,
distinct knots per axis at most 1e6, logical input bytes at most 64 MiB and
absolute Euclidean coordinate at most 1e150. It is different from generated
spline limits; a policy pass does not guarantee every backend operation can
solve the basis. The 64 MiB guard excludes native source padding/copies,
container/allocator overhead, converter temporaries and aggregate worker use.
It cannot be presented as the global 60% memory scheduler or aggregate budget.
The policy now comes from Rust `om9_basis_native_limits` through the same
asserted by-value `Limits` record, instead of duplicate C++ constants. It returns
only constants with no native object, allocation or retained state; the C++
static policy copy does not introduce a native lifetime dependency. Final fresh
native/module linkage and execution evidence is recorded below.

## Explicit remaining allocation/consumer limits

- Archive parsing/deserialization and BRep copying/validity checks already occur
  before these selected gates.
- Proxy curves/trims and native trimmed wrappers can read/copy their underlying
  basis before the converted-form gate. `exportCurve` performs the existing
  native `Transformed` copy before its direct curve gate.
- Non-NURBS `GetNurbForm`, native clamping/convention conversion and some wrapper
  conversion operations allocate native temporaries before the post-conversion
  policy can reject their output counts.
- The selected adapter intentionally allocates bounded owned extraction scratch
  before Rust value validation. Invalid values can therefore reject before
  output construction without rejecting before **every** native allocation.
- Downstream pcurve `alignTrimDomain`, trim reversal and `SameParameter`, as well
  as unrelated NURBS consumers, are not certified by the current gate placement.
  Post-transform basis safety must not be generalized from the initial gate.
- These native tests do not establish FreeCAD transaction/Undo/Redo rejection,
  async close/cancel/stale worker lifetime, full clone/subobject ownership or
  Rhino parity. Final matching SDK/application replay exercises normal geometry
  transfer and descriptor reads; it does not prove all those rejection/lifetimes.

## Evidence and follow-up

Root reports native build `622fed` exit 0, basis run `0833a2` exit 0/PASS,
exact-kernel run `2c7bc4` exit 0 with 59/59 checks and three regression CTests.
The independent periodic-source/shared-sibling converter baseline also passed
before new gates, so no production copy repair is inferred from symbol names.
The separate portable FFI reviewer reports 38 passing tests. These observations
are scoped evidence, not certification of all allocation or pointer ownership.

The original native adapter suite already exercises rational extraction,
nonfinite poles/knots, invalid weights, owned short capacities, malformed counts,
degree/coordinate boundaries, homogeneous overflow, generated-policy separation,
periodic ON/OCCT conventions, 2D extraction and source/sibling immutability.
The extended native test source uses RAII to restore every temporary ownership,
capacity and pointer change before native destruction, including exception
unwinding. Capacity-zero tests keep the actual allocations live and restore
their original ownership. Null, misaligned and aligned-wrapping CV/knot addresses
exercise rejection before native reads; the CV cases additionally call the
production curve/surface import entrypoints. CRC checks protect source
immutability. No dangling pointer is dereferenced and these cases do not prove
general allocation accessibility. Root reports extended basis run `88abab`
exit 0/PASS before the subsequent Rust policy move; the final frozen-source
proof below supersedes that historical run.

## Final matching evidence and hashes

Root reports final SDK build `450c52` exit 0 and native basis run `f9a819` exit
0/PASS. The reviewer read `H:/FreeCAD-src/build/p0-native-basis-run.log`
(`RCORE-01 native NURBS basis adapter PASS`) and its exit JSON (`exit=0`),
without running or overwriting the artifacts. Final exact-kernel rerun `9122ba`
is `ok=true`, 59/59 checks, including unchanged periodic NURBS source/shared
sibling basis after export. Geometry, TrimInteriorMapping and MultiShellSolid
CTest results are also passing in the persisted native log.

Final application evidence uses module SHA-256
`b80631e87580fb0c33efa63fb65151edbab1ac2b22d1a75bbcb05c2c35df660c`:
exact-kernel warm `1bf94f2737bf48abb8b9040071ba4103` passed 142/142 and separate
cold `975f60237386484981f2ce57e5e6d1b1` passed 79/79. Companion
`1efa76209858406d966eefef052da9b4` passed 142/142 including native CAD working
export/import in both display states, with two separate passing 64-check
exact-input adaptive sphere volume oracles. Its semantics and diagnostic
sampling are reviewed separately. These results confirm matching normal-use
native/application integration, without extending the selected gate to full T07.

Root reports that the dependency-cache repair restored the existing pinned openNURBS revision
`eb92af3`; it was not a converter/source-version upgrade or geometry rewrite.

| Source/artifact | SHA-256 |
|---|---|
| `Gui/NurbsBasisAdapter.h` | `7bc203cf0c371419a03ab3ec437bc9048fe589b1638b7ab46fd0daa831211ba3` |
| `Gui/NurbsBasisAdapter.cpp` | `a8bf47f5e287112a52078d5616517333a4963a79b232b3ee40c44a37729a800e` |
| `Gui/ThreeDmCurves.cpp` | `1a1b17536f363f0289d9c05b6f49a719bd786a8fb79baa710614b86c5c36d3c9` |
| `Gui/ThreeDmSurfaces.cpp` | `cb32c4550bf39f0db99be18a4b67cd8d6dabebab69dd5e4dd5793c5b85081626` |
| `rust/src/nurbs_basis.rs` | `50d4fe44e2a325ff2f614c1f06d263d42cc1f16fc3504e4240863f395e41e916` |
| `rust/src/nurbs_basis_ffi.rs` | `c2c6f1ea5afd97fc1ce41993e37bd4bd85a6a6f8c27d1753a13646966ba3be58` |
| `tests/native/nurbs_basis_adapter.cpp` | `9d2bde109b051c3550db3c2f4c3279c1b961ded3770a426ba9ec2b1e85366374` |
| `H:/FreeCAD-src/build/rcore-native/NurbsBasisContractTests.exe` | `5da153554d5ed4e54cb008769edbd6b904bfb1e664f02120640f01f012e1288d` |
| Final native basis log | `e1d11ee8b1c3fb512abb8510ae5188c716588355b82bd9c1dd4ec7e64d2a8be5` |
| Final native basis exit JSON | `5a05677ae6e255b8cfee5614f17eeefc62ece77150be523e03e2965696017dee` |

No source blocker remains within this review's direct-basis/profile scope.
Full RCORE-01.T07/T08/T09/T11 and literal pre-allocation rejection remain open.
