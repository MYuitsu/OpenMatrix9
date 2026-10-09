# RCORE-01.T11 authored subobject ownership review

Independent review against OM9 baseline `422098b`, completed 2026-10-10.
Scope: the new native host fixture and existing FreeCAD getter/copy/lifetime and
OM9 request-guard implementations. No product or test code was changed by this
reviewer; no build, GUI process, test execution, or delegation was performed.
The runtime qualification below comes from independently inspected persisted
artifacts, including process completion, rather than a reviewer rerun.

No blocking finding remains for the **authored RCORE-01.T11 fixture**. Recommend
closing that numbered fixture with the restrictions below. A publication
ownership finding was demonstrated in the real native host and resolved by an
explicit deep-copy boundary in the fixture. The result does not close all-family
ownership, borrowed raw-pointer lifetime, worker cancellation / release /
sanitizer requirements, or the wider RCORE-01 chapter.

## Ownership and native source evidence

The three mutable owners are stated before the corresponding edits: the detached
`Part.BSplineCurve` and `Part.BSplineSurface` getter results, two new standalone
`Part::Feature` outputs, and the explicitly replaced canonical Wire `Shape`
property. The fixture uses one identity-frame Wire containing an authored line
and clamped degree-2 BSpline, an untrimmed clamped degree-2 x degree-2 tensor
BSpline Face, and two translated App::Link instances of the same App::Part
definition. It exercises existing native facilities without a new converter or
ownership abstraction.

- `TopoShapeEdgePyImp.cpp:968` constructs `GeomBSplineCurve(adapt.BSpline())` and
  a new BSplineCurvePy; `TopoShapeFacePyImp.cpp:922` does the corresponding surface
  construction. Both returned wrappers are marked not tracking their source.
  The decisive geometric detachment is `Geometry.cpp:1477` / `5293`: `setHandle`
  copies the OCCT curve / surface with `Copy()`, rather than retaining the source
  support as its mutable handle.
- `BSplineCurvePyImp.cpp:438` and `BSplineSurfacePyImp.cpp:599` mutate the wrapper's
  own geometry handle with `SetPole`. The generated BSpline wrappers delegate
  destruction to their parents; generated `GeometryPy.cpp:1075` deletes its
  handled geometry when the Python wrapper dies. These paths do not require a
  document object or an Edge/Face pointer to perform the detached edit.
- `PropertyTopoShape.cpp:179` obtains a const Python shape wrapper;
  `TopoShape.cpp:550` constructs it around `new TopoShape(*this)`.
  `TopoShape.h:2977`'s ShapeProtector derives from TopoDS_Shape. The locally
  installed OCCT `TopoDS_Shape.hxx:327` stores an `occ::handle<TopoDS_TShape>`.
  Consequently the ordinary Shape value is a **strong shallow shape alias**.
  `IsSame` compares the TShape handle and location, and ignores orientation;
  runtime `isSame == true` establishes this topology identity, not a borrowed
  raw-pointer relationship or independent mutable support.
- `TopoShapePyImp.cpp:193` routes `copy(copyGeom=True, copyMesh=False)` through
  `makeElementCopy`; `TopoShapeExpansion.cpp:3576` passes those flags to
  `BRepBuilderAPI_Copy`. Runtime `isSame == false` confirms distinct topology
  for these explicit copies. No in-place mutation through the shallow shape
  alias is attempted.
- Publication requires a separate ownership boundary: native `Geometry.cpp:737`
  / `5061` and the actual Python-call routes `GeometryCurvePyImp.cpp:103` /
  `GeometrySurfacePyImp.cpp:111` pass their own support handles to MakeEdge /
  MakeFace without `Copy()`. `PropertyTopoShape.cpp:191/208` assigns a TopoShape
  value, whose assignment at `TopoShapeExpansion.cpp:294` retains the shape
  handle. The final fixture deliberately publishes
  `toShape().copy(copyGeom=True, copyMesh=False)` rather than assuming toShape
  itself creates independently mutable support.
- `Gui/CoreObjects.cpp/.h` are unchanged from `422098b` (baseline/current lines
  compared). Guard capsules own value snapshots with document session, native
  object identity, generation/revision/epoch, ancestors, units and world frame;
  they do not own an Edge/Face raw geometry pointer. Document deletion closes
  and erases the old session, property changes invalidate source revisions, and
  Undo/Redo invalidate the document epoch. `verifyObject` checks these facts
  before looking up the source by saved name. The subelement string itself is
  not a resolved or durably remapped native subshape.

## Accepted final runtime evidence

Final acceptance folder:
`H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/7612fd53179f4117b25f0599366c59dd`.
Its `results.json` records **35/35 PASS**, `ok: true`, and no error. The manifest,
actual current macro, results-reported loaded module, and current module file
agree: macro SHA256 `aee66cb542276a3c9a521011c608865dd6d2b867066f8a1e21964f834e9a9a5c`,
module SHA256 `8b03c89907672d0579544a1fce934632c231ad5b70e23612fc09bd3c6b790598`.
`process.json` records PID 50796, `wait_completed: true`, `exit_code: 0`, started
2026-10-09T21:05:01.6628607Z and observed 2026-10-09T21:05:05.9421857Z, using
`H:/FreeCAD-src/build/relWithDebInfo/bin/FreeCAD.exe`. Process exit alone is not
the acceptance oracle; both the process record and the 35 check results were
inspected.

The independent quadratic midpoint after moving pole 2 to (2,2,2) is (2,1,1).
The independent tensor midpoint after moving the center pole to (1,1,4) is
(1,1,1). Both use the unchanged absolute distance bound 1e-7. Persisted full
bases show only the intended pole edits; original curve and surface midpoints
remain (2,1,0) and (1,1,0). Source/definition member order and both link placement
and native world-shape fingerprints compare exactly before/after detached edits.
The source-before/source-after Wire and Face BRep files also compare byte for
byte, independently of the reported hashes. All four shallow/deep native shape
fingerprints remain exact; the object/property-name inventory and Undo count are
unchanged, and both source request guards remain valid.

Publishing with explicit deep geometry copies creates exactly two standalone
outputs and one Undo entry. Retained-wrapper re-edits to curve pole z6 / surface
center z12 produce the independent expected midpoint z3 in those wrappers.
Both published output midpoints remain at z1; their full native bases and raw
BRep hashes remain exactly at the previous published values. Independently read
output-before/output-after-reedit BRep pairs compare byte for byte. The wrappers
are then reset for subsequent lifecycle controls. One Undo removes both outputs
while preserving the sources/siblings, and Redo restores their exact native
shape hashes. This is a fixture transaction exercising FreeCAD, rather than
evidence of an unimplemented OM9 face-edit command.

Explicit canonical Wire replacement then changes the Wire and both shared
instance world shapes as intended. The unrelated Face and published output
hashes remain exact. The old Edge2 guard rejects, the independent Face1 guard
remains valid, and the detached curve still evaluates correctly. Undo restores
the original source/siblings and invalidates the old Face request through the
document epoch.

After closing the original document, an empty same-name document rejects both
old requests. Detached native support owners and all four strong shallow/deep
shape snapshots remain usable with the expected values / exact retained hashes;
their evaluation and stale validation create no object or Undo entry. The fixture
then recreates the same CanonicalWire/CanonicalFace names with equivalent native
geometry in that new document session. Both old requests still reject, while
fresh Edge2 and Face1 guards validate and the recreated inventory / zero Undo
remain unchanged. Check names using "resolves" refer to successful **guard
validation**, not an independent native subshape resolution call.

All six persisted rejection errors are concrete `Source object changed or was
replaced: <name>; cancel and reselect before accepting` errors. An optional
review observation that `reject()` formerly accepted any exception was addressed
by requiring that exact stale-source prefix; final acceptance is the matching
rerun above. It cannot pass on a missing API or unrelated exception.

## Resolved publication ownership finding

The earlier 32-check fixture did not re-edit retained getter wrappers after
publication, leaving standalone-output isolation unproved. Native source review
identified the direct handle-toShape boundary above. The real host negative
control `4315b76011404021bc171cc29a0c663a` (macro
`74be74d04844b8e6da24ae90791aa4e486c4c9d5c9cd3bcd5a38603a1aefd2a0`)
passed 14 checks and failed check 15, `published owners are isolated from
retained mutable supports`. Its persisted output hashes changed from
`2243bc8f0d41211738e6e466ff584dfaaa48690ac2ce15c5aac11cae4e3d813e` /
`572295360a2ef8a9e4ef2801a9bc289c56aac878347616591309ead1b5af71dd` to
`d230639f8a7c824e7830184ca1b432019e9146f5905069f9faf4b6507db9ee39` /
`ea1ed20bbe7d760f227212efbcd6fb527d30f4a769f206e16177ca454c8c8a5d`;
both output native midpoints moved to z3. This is direct runtime evidence of
shared mutable support for these un-copied publication calls, not a theoretical
alias warning.

Using the existing explicit deep-copy API at both publication sites yielded
34/34 PASS in `e6ecd78aa5964bb1a8d6970900cc9f0b` (macro
`4d48cf38bc9999aa3cb6a779a643f45dfaafa22aa7a1d008523f6e3c70b4c573`).
The final 35-check rerun additionally persists output raw-byte pairs and requires
unchanged complete native bases and midpoint values during retained-wrapper
re-edit. All three runs use the same module fingerprint. The RED process exited
0 but `ok: false`; it is preserved and excluded from PASS counts. This repairs
fixture use of the existing ownership APIs and requires no product change.

## Recreation oracle and preserved failed attempts

The initial `b92d935448a04199ad61de445759ce71` run passed 27 checks with macro
`8944dc1bd6d21d6d1a43f73541e256ae35d5e1d6188ce24c3f9cf8fd00f165e9`.
It lacked recreated-name positive controls and is historical, not current-source
acceptance. Extended attempts `ed038e2218414ca3b91ca1c8a217b990` and
`800eb173160541f0afc093f8159ac2f2` each passed 27 checks and failed check 28 when
requiring a fresh owner's raw BRep serialization to equal that of the retained
owner. Both have process exit 0 but `ok: false`; neither is counted as PASS.
Their macro fingerprints are respectively `91857e35c0ea60cff6af5fda11a80335fd62ca52269f2ae543a4862160cde632`
and `d24f161a080d56cda72c40fea4eb3070508ba46599e127405affb8210b1083b7`.

The second attempt persisted both retained/recreated BReps. Independent raw
diff inspection found added identity TopLoc records (and child use-location
indexes referring to them) and the fresh top-level owner's Free flag changing
`0101000` to `1101000`. Curve/surface, pole, knot, weight and vertex geometric
payloads are unchanged. This is distinct from an existing source unexpectedly
changing: all original source/sibling/retained bytes continue to use strict
equality. No BRep content was normalized and no product fix or numerical bound
change was made to obtain GREEN.

The recreated-owner predicate compares exact actual/expected shape type,
placement, vertex coordinates, edge classes/domains/orientations/endpoints/full
BSpline bases, ordered wire-use orientations, face orientations/domains and full
surface bases. The entire actual/expected signature is persisted and equal in
the final run. For this known line + quadratic Wire and untrimmed tensor Face,
it establishes the intended recreated geometry while allowing different native
owner serialization. It is **not a general BRep equivalence certificate**:
arbitrary pcurves, trim incidence/use identity, tolerances, flags and all other
representations are outside that predicate's coverage.

`af0fcccc519746258284a9063cfe6b98` passed the expanded 32 checks with macro
`399a1ab06cc45687a5d698a34d2d101a7a2b9cc8704129ea802540f7f5a5fac7`; it is
superseded by the final rejection-prefix hardening rerun. It is not combined
with the final run to inflate the acceptance check count.

The prefix-hardened `ae66bd14b32f4714b740b0c8119fa176` run also passed 32 checks
with macro `39a18de5ecfa36d78606ec50cb777c92b92e6f028bbed69aaf657b4d74f610f0`;
its publication coverage was superseded by the ownership RED/GREEN and final
35-check fixture above. No superseded run is presented as current-source proof.

## Explicit limits

The literal T11 requires owner identification before mutation, conditional
source/sibling changes, and distinct standalone/subobject lifecycle reporting;
the authored fixture supplies each. Its detached Curve/Surface values and strong
shape snapshots survive close because they own native state. Document-bound
Part::Feature objects and guarded subelement requests have different lifetimes;
the fixture does not assert that those document objects survive close.

Still open: every geometry family and native adapter, editing a trimmed Face or
loop, mutation through raw borrowed handles, native thread transfer/release,
worker cancel/close races, ASAN or leak proof, lock checks on real command commit,
durable subelement remapping, general global topology/deviation certification,
and complete T07/T08/T10/History contracts. A saved FCStd is fingerprinted here,
but no cold reopen or cross-process ownership assertion is inferred from save.
The shared-definition stamps cover authored members, source CAD, link owner
names and matrices/world CAD; they are not a serialized certificate for every
document property or UUID. No new production code was required for the authored
acceptance slice.

## Inspected SHA256 fingerprints

Source and native ownership provenance (generated wrapper files are inspected
build outputs, not new production changes):

| File | SHA256 |
|---|---|
| `H:/FreeCAD-src/Mod/OpenMatrix9/tests/rhino_core_subobject_ownership_smoke.FCMacro` | `aee66cb542276a3c9a521011c608865dd6d2b867066f8a1e21964f834e9a9a5c` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/docs/superpowers/plans/2026-10-10-rhino-core-subobject-ownership.md` | `d67b25222ec8fb770ee9cace31f2a44ef31e5cd5ff23f867c51f0e9c23a21c61` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/Gui/CoreObjects.cpp` | `fedbdfba5593f89d5b4bf2f6148aabd58112e4f2c7bb5cc43e105d3c8acaa98e` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/Gui/CoreObjects.h` | `3800b6922ad57be6bd4bb2a93a7ba8c034f8ee16a610637bf955a80c496feb03` |
| `H:/FreeCAD-src/src/Mod/Part/App/TopoShapeEdgePyImp.cpp` | `8d005d88c40011ad7d366426b00e217b03e9e6f3199fd86c97474045c923f0c9` |
| `H:/FreeCAD-src/src/Mod/Part/App/TopoShapeFacePyImp.cpp` | `324ed0b8642b0261ff6fc981997fa8523589eac647827ca5aab85dfdc3faa0f2` |
| `H:/FreeCAD-src/src/Mod/Part/App/Geometry.cpp` | `6f739f21a06fac39937b77b55071df46b64261500c49f907d4ff166bb551afa9` |
| `H:/FreeCAD-src/src/Mod/Part/App/GeometryCurvePyImp.cpp` | `93218013d9250e8d9d0bdcff53884e3c8a304bfa92d270ceaf76dbea3294abf8` |
| `H:/FreeCAD-src/src/Mod/Part/App/GeometrySurfacePyImp.cpp` | `3c925d1c1cf91d93e50fffece55196d42f95b9de1e7047ed80e664081876fa45` |
| `H:/FreeCAD-src/src/Mod/Part/App/BSplineCurvePyImp.cpp` | `cbfb4984dbce3418a9b41bd6509aa5e2eab1210d39316ef87d2c8dccd5f80657` |
| `H:/FreeCAD-src/src/Mod/Part/App/BSplineSurfacePyImp.cpp` | `fb13e31d9b095a55bae499c28f6d724edeaa7315ea163e82114a7ab7eb4d9a95` |
| `H:/FreeCAD-src/src/Mod/Part/App/TopoShapePyImp.cpp` | `db0f1f3fb0fb55e13017c64466d1f916821340810057259a203053c37d4eab0e` |
| `H:/FreeCAD-src/src/Mod/Part/App/TopoShapeExpansion.cpp` | `ddbe6e24ffe6fc80927016851ad026de3ad2976be7bbc1fc003d9746a8d5d945` |
| `H:/FreeCAD-src/src/Mod/Part/App/TopoShape.cpp` | `2f7de58f556e3ce2cb0877ebdb1a83536d1e73be2d8a2b9243056083f0619e5e` |
| `H:/FreeCAD-src/src/Mod/Part/App/TopoShape.h` | `cf0354bb7fc9c37e82dc37b5fde187860e69b6170c3fe9688482c69cbe2731b4` |
| `H:/FreeCAD-src/src/Mod/Part/App/PropertyTopoShape.cpp` | `da3e5682155e33872c742adeaf6798c376d328239065abe9d16663a191c52ee7` |
| `H:/FreeCAD-src/.pixi/envs/default/Library/include/opencascade/TopoDS_Shape.hxx` | `9535f666d49437f72151a484daf9f60d065ac59d0dc765866211427e94085f65` |
| `H:/FreeCAD-src/build/relWithDebInfo/src/Mod/Part/App/GeometryPy.cpp` | `9304a2b9837fe9562af1d41c2b155fc73e72f91edaf92711ac2bf6a624dbe0ae` |
| `H:/FreeCAD-src/build/relWithDebInfo/src/Mod/Part/App/BSplineCurvePy.cpp` | `de6368ebfad12e73672c7e79ccc1323d4e9d1e1edbbe0699f18e6d34f46fe7a6` |
| `H:/FreeCAD-src/build/relWithDebInfo/src/Mod/Part/App/BSplineSurfacePy.cpp` | `7fa0ce5a2def346097a73b57cd874d1a1872e89c21c9f735c5ddbd08a5af8ff8` |
| `H:/FreeCAD-src/build/relWithDebInfo/src/App/ComplexGeoDataPy.cpp` | `3bc6d2cdd1e29829c0b9ae25dfabcfd64e158c95c5565944d87d9aba219921cb` |
| `H:/FreeCAD-src/build/reusable-history-runtime/bin/OpenMatrix9Gui.pyd` | `8b03c89907672d0579544a1fce934632c231ad5b70e23612fc09bd3c6b790598` |

Final acceptance artifacts:

| File | SHA256 |
|---|---|
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/7612fd53179f4117b25f0599366c59dd/merge-native-module.json` | `cbc2e84e70bbdedeb9b0451aa58ad4be5caa8d9083c963cdef72ad62f0fc3eea` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/7612fd53179f4117b25f0599366c59dd/process.json` | `80fbe34c4298ccd3b1a18f3deddbe3990f3745b76d35ed32d982f461cd056390` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/7612fd53179f4117b25f0599366c59dd/results.json` | `46ae2aa00ec17b2c73993d2dcb2ddb3aeb6cebcc719f1ad0d88e6016c9e88834` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/7612fd53179f4117b25f0599366c59dd/source-before-CanonicalWire.brep` | `0078f2910b0507139b7e4e9817eeb36c1ac2e3de78efbbbcc11c9d4228cb5dec` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/7612fd53179f4117b25f0599366c59dd/source-after-CanonicalWire.brep` | `0078f2910b0507139b7e4e9817eeb36c1ac2e3de78efbbbcc11c9d4228cb5dec` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/7612fd53179f4117b25f0599366c59dd/source-before-CanonicalFace.brep` | `38c59852032cd783b35ccb1802c57e78dbf84c5241708d57cbd0e1c8d504bebd` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/7612fd53179f4117b25f0599366c59dd/source-after-CanonicalFace.brep` | `38c59852032cd783b35ccb1802c57e78dbf84c5241708d57cbd0e1c8d504bebd` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/7612fd53179f4117b25f0599366c59dd/recreated-CanonicalWire.brep` | `07a2d7ea1e980c9cb044aeee000834839d73e2e1952cdc21919fd18f098a20ae` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/7612fd53179f4117b25f0599366c59dd/retained-CanonicalWire.brep` | `0078f2910b0507139b7e4e9817eeb36c1ac2e3de78efbbbcc11c9d4228cb5dec` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/7612fd53179f4117b25f0599366c59dd/recreated-CanonicalFace.brep` | `d19497c192db980df191d8acadb9f31d02cc7d8b22b878bf8eb1975c0d65704f` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/7612fd53179f4117b25f0599366c59dd/retained-CanonicalFace.brep` | `38c59852032cd783b35ccb1802c57e78dbf84c5241708d57cbd0e1c8d504bebd` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/7612fd53179f4117b25f0599366c59dd/SubobjectOwner.FCStd` | `b1de2f638bdfed4c70fe1aeee7cb6eb3e513137cfb4c806647bca6c1be89570f` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/7612fd53179f4117b25f0599366c59dd/output-before-reedit-OwnedCurve.brep` | `2243bc8f0d41211738e6e466ff584dfaaa48690ac2ce15c5aac11cae4e3d813e` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/7612fd53179f4117b25f0599366c59dd/output-after-reedit-OwnedCurve.brep` | `2243bc8f0d41211738e6e466ff584dfaaa48690ac2ce15c5aac11cae4e3d813e` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/7612fd53179f4117b25f0599366c59dd/output-before-reedit-OwnedFace.brep` | `572295360a2ef8a9e4ef2801a9bc289c56aac878347616591309ead1b5af71dd` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/7612fd53179f4117b25f0599366c59dd/output-after-reedit-OwnedFace.brep` | `572295360a2ef8a9e4ef2801a9bc289c56aac878347616591309ead1b5af71dd` |

Historical / failed diagnostic artifacts, excluded from current acceptance:

| File | SHA256 |
|---|---|
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/ed038e2218414ca3b91ca1c8a217b990/results.json` | `2efabdc93ae1d6d46a8ebe7ed6f1b447f17a377661c4c3c7484e16831657f901` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/ed038e2218414ca3b91ca1c8a217b990/process.json` | `dab56eed815ede5bdce47010db1d9e218f70d46f0001e4ac500a9cb1c3a0b80b` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/ed038e2218414ca3b91ca1c8a217b990/merge-native-module.json` | `5f9a38901741011ce09c1c0a56c2fcd0db1b3cbf838f0d58db2b26c11afcc160` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/800eb173160541f0afc093f8159ac2f2/results.json` | `30c848e64a1ea8e7ff4b0112196ba9cff2e83e8718f6595ed752974fb161f422` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/800eb173160541f0afc093f8159ac2f2/process.json` | `ac04c6fbdb71f901401e10ac402e96f51611e3ad334c2c3e031c2c976acf8151` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/800eb173160541f0afc093f8159ac2f2/merge-native-module.json` | `cac27d11d6677b2b82b449a6709514685696df383b4711d4989c971f609d34b8` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/800eb173160541f0afc093f8159ac2f2/retained-CanonicalWire.brep` | `0078f2910b0507139b7e4e9817eeb36c1ac2e3de78efbbbcc11c9d4228cb5dec` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/800eb173160541f0afc093f8159ac2f2/recreated-CanonicalWire.brep` | `07a2d7ea1e980c9cb044aeee000834839d73e2e1952cdc21919fd18f098a20ae` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/800eb173160541f0afc093f8159ac2f2/retained-CanonicalFace.brep` | `38c59852032cd783b35ccb1802c57e78dbf84c5241708d57cbd0e1c8d504bebd` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/800eb173160541f0afc093f8159ac2f2/recreated-CanonicalFace.brep` | `d19497c192db980df191d8acadb9f31d02cc7d8b22b878bf8eb1975c0d65704f` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/b92d935448a04199ad61de445759ce71/results.json` | `d25cfd0618648bd2020e25f2a5b7a88d217e73af931085e189292c83e9fd6a30` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/b92d935448a04199ad61de445759ce71/process.json` | `36e3be2c9e548a41a5d42411032c23ff17ecaf9aaa53ba5021ffbb337210e757` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/b92d935448a04199ad61de445759ce71/merge-native-module.json` | `0a0edb104bac3baf5ed667fd90a23ecb08c8a6230224d8a0e758205a78992284` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/af0fcccc519746258284a9063cfe6b98/results.json` | `85919e55e7e9b255984b186f70226dfc55d578bdcf50b051218bbb02a91e1d80` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/af0fcccc519746258284a9063cfe6b98/process.json` | `70f365f1cbd66dc01f21b5368827b0ed861daf3f72a47d0e8e28c77d41224d8e` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/af0fcccc519746258284a9063cfe6b98/merge-native-module.json` | `1bf303e2e68d6e011fc984f71a572f6ce5dccd704406b79ba09a7f0c382c472d` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/ae66bd14b32f4714b740b0c8119fa176/merge-native-module.json` | `b7a453f7af4033ddabea7c74831b0ed8be34b5aa947db330cbb318504fa33e10` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/ae66bd14b32f4714b740b0c8119fa176/process.json` | `05de56185a6fc6f69cf965284503a94e87a6c159851145d0d2909e07c6cad45d` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/ae66bd14b32f4714b740b0c8119fa176/results.json` | `b66da9e9d47f0f7e03b3e42a46505f7393dfb3775a353c6d8c646056fc4d6343` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/4315b76011404021bc171cc29a0c663a/results.json` | `27ec61c24d749025a4505e99d253498bc0a7e22886fcea55408aaa1d8770f288` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/4315b76011404021bc171cc29a0c663a/process.json` | `4cca7b2d2cd281062d5ddfdf5f0c2c968669dddc19762ef91895fd4aff7b0b46` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/4315b76011404021bc171cc29a0c663a/merge-native-module.json` | `e3e105622967258a8ca8eac92d31d90a2fc98f55ee8ef520e712968caf513646` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/e6ecd78aa5964bb1a8d6970900cc9f0b/results.json` | `13003b62f43415b0d5759b4561ec03c389c5aeb396f87a24871d9808d8f352f5` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/e6ecd78aa5964bb1a8d6970900cc9f0b/process.json` | `2c9f6e6c19c01c2a93e6a509dcc082a4d66147e316aad42d45fb34684e77d449` |
| `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/e6ecd78aa5964bb1a8d6970900cc9f0b/merge-native-module.json` | `1493efea325afe78a7c77f9c8f2964cf33fc06d9b3d79ee4a48e7754c0381e03` |
