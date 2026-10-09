# Ordinary polycurve partition — independent native review, 2026-10-10

Requested under `superpowers:requesting-code-review`, against baseline `04d64cc`
and Task 3 / Review focus in the polycurve partition plan. Scope: the native
adapter, archive/merge/staging routing, CMake integration and native fixture
fingerprinted below. The application macro and persisted warm/cold evidence were
also inspected. Rust portable policy/FFI has a separate independent review.
Only this report was edited; the reviewer did not build, execute a native/GUI
process, edit production/tests, mutate a matrix, or publish anything.

## Assessment

No blocking source defect was found in the admitted flat ordinary-owned
polycurve slice. The final native run passed 403 checks and the matching
SDK built successfully. Matching warm 69-check and separate-process cold
37-check application reports establish the authored open two-segment T05
fixture described below. They do not close every polycurve family, nesting,
reference ownership or overall P0.

Safe Rust owns the copied numerical/joint/occurrence policy and the 4096-segment,
1 MiB logical-report limits. Native copies, containers, allocator overhead and
aggregate worker use are not included in that logical budget. Native C++ is a
permanent adapter exception for openNURBS owners,
OCCT construction/traversal and Qt serialization. It reuses the existing child
curve converters and does not introduce a geometry solver.

## Admission, metadata and errors

The importer requires the ordinary `ON_PolyCurve` native RTTI class ID. Before accessing
parent parameters, Domain, closure/evaluation or CRC, it bounds native segment
count, container capacity, parameter count and nonnull arrays, then admits every
nonnull child. Allowed native child IDs are Line, Arc, NURBS and Polyline;
nested polycurves, proxies/reference graphs and other reported native classes are
explicitly unsupported. No recursive flattening or reference-owner guessing
occurs. Archive preserve mode retains the unsupported object with an issue;
working mode reports rejection. Invalid numerical/storage data reject rather
than silently becoming editable output. A class ID is not a proof of arbitrary
C++ dynamic-type or virtual-method safety; the live native owner's validity
obligation remains explicit.

Raw NURBS children use the shared `checkedBasis` storage/count/value path before
their Domain or parent CRC traversal. Polyline children have bounded point and
parameter-array envelopes before metadata/evaluation. Count is limited before
`count+1`, indexing or vector reservation. The source row's finite increasing
child/parent domains are validated before evaluating their endpoints; all actual
source endpoints and geometric joints are validated before `MakeWire` begins.
The domain-only stage uses zero endpoints on both current/previous rows, so it
does not mistake a fabricated origin for a real joint. The translated fixture
checks this boundary at a nonzero joint.

The admitted original remains caller-owned, live and immutable throughout the
synchronous call. Arithmetic/container guards do not establish readability of
forged/dangling pointers or prove unique native child ownership. Archive
deserialization and existing instance clone/transform operations happen before
this admission slice and remain separate boundaries. Shared basis extraction
uses bounded scratch before full Rust value validation; this is not literal
rejection before every native allocation required by full T07.

Partial native construction is local and unwinds on rejection. Native kernel
exceptions are caught outside Rust calls. Status errors return through the
value ABI before C++ throws. No document/native owner or allocator crosses the
56/96/64/40-byte input/source/native/summary ABI. Returned wire geometry is
independent; reports contain copied arrays/vectors and no child pointers.

## Ordered occurrences, direction and export ownership

`wireRows` counts every edge occurrence with `TopExp_Explorer`, bounds that
count, and separately traverses the ordered wire with `BRepTools_WireExplorer`.
It requires exact coverage, valid Forward/Reversed directions, finite domains,
oriented supporting-curve endpoints and actual shared vertex identities at
each joint. Partial, disconnected or branched traversal cannot publish a
subset. Empty input rejects before `.front()`/`.back()` access. Native location
is applied once to evaluated endpoints; reversed occurrence endpoints are
swapped separately from the increasing supporting domain.

Each admitted source segment is converted independently with the unchanged
single-edge factory. No whole-polycurve `GetNurbForm`, Join, midpoint repair,
`AppendAndMatch`, `CloseGap`, `RemoveNesting` or fitting fallback is used.
`MakeWire` may reconcile native vertex topology under kernel tolerance; it does
not establish original vertex identity, and the report only records the native
shared-vertex observation and declared endpoint/joint residuals. The source
segment curves are not edited. A gap beyond the declared bound rejects, with
source CRC unchanged.

Export traverses every oriented occurrence and calls existing `exportCurve` on
that occurrence. The child remains in a `unique_ptr` until `Append(c.get())`
returns success; only then is ownership released to the polycurve. Pinned
openNURBS `Append` delegates to `Insert`; ordinary false returns do not retain
the child. A failed export publishes no partially owned output. Successful
export revalidates source rows, complete coverage and closure before returning.
This does not certify allocation-failure recovery inside the native SDK.

Source parent spans and child supporting domains are separate observations.
The native fixture authors parent `[2,3],[3,7]` while its children use `[0,5]`
and `[11,19]`; native polycurve reversal preserves the corresponding negated
parent intervals separately. Export can construct new parent domains and reverse
child domains. Working roundtrip may represent reversed geometry with Forward
topological flags on reversed supporting bases; the oriented endpoints/order
remain the relevant observation. No original/global/export parameter mapping
or identical-domain preservation is asserted.

## Caller routing and conversion-report scope

Compared with `04d64cc`, `Gui/ThreeDmCurves.cpp` is unchanged after normalized
line comparison. Hatch still calls its original single-edge `importCurve`.
The new archive dispatch applies only to exact ordinary polycurves; standalone
PolyEdge and other existing reference routes retain their explicit handling.
Both working wire export and preserve-mode replacement now reuse
`exportWireChecked`, eliminating their earlier pre-success pointer release.

Working V5 output is reread and checked against the expected wire occurrence
count, oriented endpoints/joints and closure before the existing host destination
replacement step. `verifyWirePartition` measures partition/endpoints; it does
not compare all basis values or certify continuous locus deviation. The fixture
adds separate sampled geometry and rational-weight checks.

Import stores copied conversion facts in `ExchangeItem`, and staging serializes
them before host commit. Archive facts explicitly declare source-file units,
`scale_to_mm` and `observed_before_unit_scale`; they must not be interpreted as
post-scale world coordinates. Native reports otherwise leave units and source
identity unknown. Global locus, parameterization/map, topology, attributes and
deviation remain null. These prepared facts are not durable History or complete
preserve-mode userdata/replacement evidence.

## Numbered T05 evidence and limits

The authored fixture is line A=(-5,0,0) to J=(0,0,0), followed by a rational
quarter-circle R10 centered at (0,10,0), J to B=(10,10,0). The native source
has an independently authored rational quadratic basis; the application fixture
uses existing Part Line/Arc/NURBS/Wire APIs. Both retain two distinct ordered
occurrences and nonunit middle arc weight `sqrt(1/2)`.

Recommendation: the numbered open mixed line/rational-arc **T05 fixture is
verified with its declared sampled scope**. Native reversal and reverse export
retain arc-reversed then line-reversed geometric traversal, endpoints and joint.
Warm application checks source/sibling byte fingerprints, one Undo/Redo and
double reversal. Fresh-process FCStd reopening preserves the reversed oriented
ledger and rational basis, with maximum serialized numeric difference
`8.151490101317698e-17`, below the separately declared `1e-12` serialization
bound. Working V5 prepare/commit keeps one native CAD curve Wire with two ordered
occurrences and one shared joint. Prepared source/native segment facts, no-heal
policy, units and unknown global fields are checked before commit. Working
import Undo/Redo also passes. The imported working Wire is saved to its own
FCStd; cold reopening verifies its exact input-file hash, typed CAD
classification, ordered endpoints/joint and rational basis against the warm
imported ledger. Sampled R10 radius residuals are
`1.7763568394002505e-15 mm`, under the unchanged `1e-7 mm` length bound.

No segments are merged/healed, so T05's conditional warning for a merge is not
triggered. Continuous/global deviation, closed-loop direction proof for every
family, arbitrary repeated occurrences, nested/reference mutation, tangent
continuity, durable subelement identity/History and complete host ownership
remain open. The native fixture covers empty/incomplete container, nested child,
gapped source and branched/uncovered wire rejection. Added raw-child negatives
exercise an over-limit NURBS pole count and null CV buffer through this actual
polycurve entry, both with zero Domain/CRC reads. The reviewer requested that
the fixture arm metadata traps before corrupting fields; the final overrides
throw before calling the real SDK Domain/CRC, and restore fields/disarm before
ordinary destruction. Thus an ordering regression gives a deterministic failed
assertion rather than an unsafe SDK buffer read. This improves the test oracle
without changing production admission code.

Final native controls also preserve a two-child all-line polycurve, read an
actual nested V5 archive as retained with an explicit reason in preserve mode,
reject it in working mode, and reject branched wire export without changing
existing destination bytes. The final warm host imports that exact unsupported
archive into an already populated document: the named rejection precedes
commit, and object count, Undo count, existing CAD bytes and original
source/sibling bytes remain unchanged. These are concrete failure controls for
the authored cases, not certification of every unsupported owner graph.

## Execution evidence inspected

Native/SDK paths are relative to `H:/FreeCAD-src/build/`.

| Artifact | Result / SHA-256 |
|---|---|
| `p0-polycurve-native-red-exit.json` | stub RED build 0/run 1; `3313dc32f71ff3daaeeccd3d668b14e63a386ee597cfdb3874ec7d2ebdbba686` |
| `p0-polycurve-native-red-results.json` | failed with unimplemented partition, three setup checks; `d2534ff739b93620d020ec4436a380608ecb978118e99c6ca381eeec87521ded` |
| `p0-polycurve-native-compile-green-exit.json` | initial minimal GREEN build 0/run 0; `974c7faadcfb1f4b55e824c6de3180598c762f2f54e8d7781af5a50e22d0bf8d` |
| `p0-polycurve-native-compile-green-results.json` | initial 15 checks, historical scope; `d1830f514906b6f55cc211a7a327db5b3fc92c44277be4ad28b22d39cabc1edb` |
| `p0-polycurve-native-acceptance-exit.json` | final build 0/run 0; `cbd8089cc5388c22276b9080028a53fbea00a9d493ea5fd834a8eb30ba562a2c` |
| `p0-polycurve-native-acceptance-results.json` | 403/403, armed raw-child gates, V5 reread/staging and negative controls; `676c2f28583ad3ab08533013d427c91a32ed5841e8c7dc9aaec861cf6bbb9af9` |
| `p0-polycurve-sdk-final-unlocked-exits.json` | configure 0/build 0; `ccb53b82a54cba7f912c89f4925b328dec9ae78688c67e8be66e9cef7f99fe30` |
| `p0-polycurve-sdk-final-unlocked-build.log` | final worker/module linked; `d892502eb3850475d3241dd512d523dd3ad8a0c57605f95466d0136a1ef9a221` |
| `p0-polycurve-native-regressions-exits.json` | build/query/kernel/CTest all 0; `b1f4b5bf3f37f79adb80121de12766847d6d323bb92f613fe5cf5d143745019b` |
| `p0-polycurve-native-regressions-ctest.log` | seven selected CTests passed; `260d160416c944719b847f02efa7a4cff0e3dfdec293e55bce2998f089910db2` |
| `rcore-native/ThreeDmPolycurveTests.exe` | final native executable; `9010c1a2bce891f3c17daf98902f05f1b04864c34bde561ac3c59ba74e280db8` |
| `reusable-history-runtime/bin/OpenMatrix9Gui.pyd` | module used by both final host reports; `61708e51c37e9dc150d355b968a5be9ab29886858ae9ce0cea108fc2b87e5e09` |
| `polycurve-fixtures/nested-unsupported.3dm` | exact native-authored archive used by host negative import; `d014e153e47741acbe641c968816524df5f35cbdfbdcebf57b46fa932a2fdf54` |

The final regression query/kernel runs pass 159/59 checks respectively. Selected
CTest coverage is ModelingExchange, TrimInteriorMapping, MultiShellSolid,
NativeBasisGate, Merge, Geometry and HatchLoopEdit. An intermediate
`p0-polycurve-sdk-final-*` link attempt failed with LNK1168 because a fixture
process held the module open; it is excluded as successful build proof. The
parent identified/stopped that fixture process, rebuilt in
`sdk-final-unlocked`, and ran the final application evidence on the new module.

Application paths are relative to `H:/FreeCAD-src/Mod/OpenMatrix9/`.

| Artifact | Result / SHA-256 |
|---|---|
| `build/merge_replay-1/b0866f4b6a5e469db65b7e7519216fa2/results.json` | final warm 69/69, `ok=true`; `33b09d6a67f6271934fcecde58730b29ac5bec82f2a55915d0e9fde663d1d1a8` |
| `build/merge_replay-1/154e7694cf9042ea9532869efe50a6f3/results.json` | final separate-process cold 37/37, `ok=true`; `ab67b19137a12fb6a148c1dafa759a0b06afb804a0714527ca74ef4bd2509777` |
| `build/merge_replay-1/b0866f4b6a5e469db65b7e7519216fa2/native-baseline.json` | cold comparison input; `d745204a8b79d2f883cca3df7544338589b632d20d5cfc63161a9ab75a1adf9c` |
| `build/merge_replay-1/b0866f4b6a5e469db65b7e7519216fa2/mixed-polycurve.FCStd` | identical saved file reopened cold; `ffbe1ce7b12c0f2e9c5f9de0438c7dd101bf18594d537ca6efe5009803ba3a1f` |
| `build/merge_replay-1/b0866f4b6a5e469db65b7e7519216fa2/reversed-polycurve.3dm` | actual working V5 output; `ff800aaed899f43b15e858829af238fe1fe10af39a26f85e5ab93fbce0006a69` |
| `build/merge_replay-1/b0866f4b6a5e469db65b7e7519216fa2/working-prepared.brep` | exact staged import output; `b0ff579a2efb6430a84924f3b190c99b858879f77d10ee322ef4611225765735` |
| `build/merge_replay-1/b0866f4b6a5e469db65b7e7519216fa2/imported-polycurve.FCStd` | actual working imported wire saved/reopened cold; `31abb753591059a9873ef4addbfc328bd23bf61961dd92996c4ac32e06fed5e0` |
| `build/merge_replay-1/148ef6d764b1400f853f8226ea2f7412/results.json` | final-module modeling regression 32/32, `ok=true`; `22f4d21fd360bc1a9b621bdcb93fedb736dc9221713da83384ccab887f297a8e` |

Historical routed native 384-check, raw-gate/trapped 392-check,
warm 52/66-check and earlier cold 26/37-check reports remain scoped observations.
They are superseded by the 403-check acceptance and latest warm/cold evidence,
and are not presented as execution of the later module.

These artifacts identify this reviewed source/runtime snapshot. Later style,
test or module rebuilds require their own final fingerprints and applicable
replay; earlier evidence must not be labeled as execution of a later binary.

## Reviewed source fingerprints

Paths are relative to `H:/FreeCAD-src/Mod/OpenMatrix9/`.

| File | SHA-256 |
|---|---|
| `Gui/PolycurveContract.h` | `7717c9c89e0d95920eb7883b9a295178eeea179688d4a1c9c105a4c90dfa84ba` |
| `Gui/PolycurveContract.cpp` | `6cfda52fd00700686059d87baf244412017eb895fe1c7479c945fb132e879a00` |
| `Gui/ThreeDmArchive.h` | `e4f2c2eea43cdbb0443421e34623f9fc08c397d81ac05795a48abc8bb9b3d78f` |
| `Gui/ThreeDmArchive.cpp` | `5c18f465c7141847a2337a41fc7d940714601c99ed3d8c393fa189f5224ebcd0` |
| `Gui/ThreeDmMerge.cpp` | `8690af306b4f3d0c9e07e432328f803fe2dc088a779859130fb7325e33b14d83` |
| `Gui/ThreeDmStaging.cpp` | `70457375be4ba1cbb7dd1b23466b1237df1e1a828232e2ccf46a506f4f55903d` |
| `cmake/OpenNURBS.cmake` | `bec4936dee0e990372a0b246360f4807f3f0255a557677f1d5b64a46f4eb6b33` |
| `tests/native/rhino_core_polycurve_partition.cpp` | `f3b5fe1bae21965088dc2d4b72868c0f8949b0798391b8f682e1c3e79a0c411a` |
| `tests/native/CMakeLists.txt` | `e0ee4689438ab5fca63ef175444541d8a9108ad1796d8cbbe10e88fdfb6e036b` |
| `tests/rhino_core_polycurve_smoke.FCMacro` | `80831233ab8fecedb9b16e91481fecb9243c5cdba5b3ac1dc6876038b52e0096` |
| `docs/superpowers/plans/2026-10-10-rhino-core-polycurve-partition.md` | `df7df83a9c5e0fc412667bd5cff5fe37e108025220f8b76b28d9bba7ba5af0d5` |

Pinned local primary native reference: openNURBS
`eb92af3ba1806b0a34a99aba0d3bda83e3d46083`, particularly
`H:/FreeCAD-src/build/dependencies/opennurbs/opennurbs_polycurve.cpp` for
Append/Insert, parent parameter construction, reversal and closure. No Rhino
runtime, global deviation, complete preserve-mode semantics or whole-process
native memory-safety certification is supported by this review.
