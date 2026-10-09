# Read-only geometry companion — independent review, 2026-10-10

Scope: `Gui/CoreGeometryContract.h/.cpp`, the value-only descriptor/budget ABI in
`rust/src/geometry_contract_ffi.rs`, relevant portable fact semantics in
`rust/src/geometry_contract.rs`, and
`tests/rhino_core_geometry_descriptor_smoke.FCMacro`. Review requested under
`superpowers:requesting-code-review`; baseline `fcdb0ac`. Only this report was
edited. The reviewer did not run/build GUI or native processes or change source.

## Assessment / resolved findings

Final review includes explicit CAD modeling export/import in both display states
and both exact-input native adaptive-volume replays. The earlier 120-check run
remains historical classification evidence. Default `Shape.Volume` failed the
added accuracy predicate; that value remains diagnostic and its replacement
adaptive oracle uses the exact saved imported bytes and the unchanged bound.

**Resolved [P1]: supporting facts and restriction domains require separate
explicit scopes.** The original `curveFacts` and `surfaceFacts` unwrapped
supporting geometry and obtained its closed/periodic flags while reporting the
edge-restricted or face-restricted
domain. A trimmed half-circle can therefore have a full supporting circle's
`closed=true` beside the restricted edge interval, and a partial cylinder face
can have supporting `periodic U=true` beside a nonperiodic restricted interval.
`domain_scope` alone does not define which geometry the other facts describe.
The original full-circle/full-cylinder fixtures cannot detect this ambiguity.

Root accepted the finding. The concrete source repair now uses typed
`Fact<GeometryFactScope>` and `Fact<DomainScope>`; native support flags are
explicitly `supporting_geometry`, while observed intervals are
`edge_restriction`/`face_uv_restriction`. The value-only Rust gate rejects
unscoped known flags and incompatible scope/representation/domain combinations,
then returns the typed scopes to JSON. The reviewer checked the matching
104-byte input/72-byte summary, alignment and field offsets. Portable tests
exercise scope retention/rejection. Root reports scope RED `9cd8a7` (10 pass,
2 fail) and GREEN `460069` (27 tests). The final native application fixture and
matching SDK replay are now checked below.

Edge `restriction_facts.closed` separately uses `BRep_Tool::IsClosed(edge)` and
is labeled `native_topology_endpoints`; restricted periodicity stays unknown.
Local OCCT primary headers define this closure as identical vertices, not a
Rhino geometric endpoint tolerance comparison. Face restricted closed/periodic
direction facts remain unknown. The independently authored half-circle edge and
half-cylinder face retain supporting closed/periodic facts beside restriction
intervals of width pi. The edge is explicitly open by native endpoint topology;
face restriction direction facts remain unknown. Owned values survive document
close, and same-process reopen preserves these facts/domains while invalidating
old guards. The final 142-check host run passes every predicate, closing this finding
within the companion's stated scope.

**Resolved T04 fixture source mutation:** the initial diagnostic changed the
serialized BRep hash while analytic radius, stamp and guard stayed unchanged.
Raw diagnostic `fbd3f12a999a446aaf5af91c0a793492` isolates a single face TShape
flag change, `0101000` to `0111000`, after explicit source `Shape.tessellate(0.8)`.
Observation reads and display events alone leave the source bytes unchanged.
Final fixture sampling uses an independent native geometry/topology copy and
retains the original strict source hash predicate, with no flag normalization
or tolerance enlargement. The final run passes both display modes.

## Sound boundaries already inspected

- `geometryDescriptor` is a synchronous GUI-thread Python API. It captures and
  verifies the existing object snapshot before native inspection and verifies
  it again before returning. Document restoration/missing references reject;
  existing object/subelement reference limits are reused. No command, source
  property, transaction or persistent document object is added.
- Rust receives copied scalar identity/representation/domain flags, never host
  pointers. Native handles stay on the C++ stack. The result is newly owned
  Unicode JSON containing primitive values and arrays; future document close
  does not give the saved JSON a native lifetime dependency.
- The native basis helper is reused. Rust owns companion count/budget policy;
  counts are checked before independent basis scratch and JSON pole/knot arrays.
  Over-budget bases remain explicitly uninspected with no truncated arrays.
  The 8192-pole/16384-knot-per-axis companion policy is separate from the broader
  native conversion policy and from aggregate worker memory guarantees.
- Object stamp fields are emitted as decimal strings, preserving the unsigned
  64-bit identity domain without JSON double precision loss. Existing unit
  context and finite world placement are copied as provenance. Native support
  location, world provenance and supporting poles remain separate; world
  coordinates and transform composition are explicitly unknown.
- Native shape type distinguishes CAD edge/wire/face/shell/solid from mesh and
  point cloud. A periodic/closed supporting surface does not make a face solid.
  `topology.solid` is labeled `native_shape_type` evidence only; manifold,
  closure, orientation, complete trim graph and preservation stay unknown. A
  declared SOLID type alone is not validated manifold/oriented solidity.
- Native exceptions are caught before returning through the Python boundary;
  Rust value-only calls do not receive native exceptions or objects. Static
  record layout checks include the new scope fields and match their Rust
  `repr(C)` records; final fresh native/module linkage and application replay
  are recorded below.

## T04 primary source and raw diagnostic cross-check

FreeCAD `TopoShapePy::exportBrepToString` calls
`TopoShape::exportBrep(std::ostream&)`. That function already constructs
`BRepTools_ShapeSet(Standard_False)` and uses format 1
(`src/Mod/Part/App/TopoShape.cpp:958–971`), so a changed serialized hash cannot
simply be attributed to included display triangulation bytes. The persisted
before/after raw BRep files differ only in the third flag of one `Fa` record:
`0101000` becomes `0111000`. All other serialized geometry, tolerance,
representation, orientation and topology text is identical. The matching
[OCCT 8.0.1 primary writer](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/V8_0_1/src/ModelingData/TKBRep/TopTools/TopTools_ShapeSet.cxx)
emits flags in Free/Modified/Checked/Orientable/Closed/Infinite/Convex order, so
the third bit is exactly Checked. Installed `Standard_Version.hxx` identifies
OCCT 8.0.1; local `TopoDS_TShape.hxx` exposes this separate internal check flag.

`TopoShapePy::tessellate` calls the native face mesh path, so the radius/stamp
checks cannot independently prove that every serialized CAD field stayed the
same. The failing raw diagnostic proves explicit source meshing was intrusive
to that strict serialized-hash condition even though the geometry stayed the
same. It remains RED, rather than being retroactively accepted.

`Shape.copy(copyGeom=True,copyMesh=False)` routes through native
`TopoShape::makeElementCopy` to `BRepBuilderAPI_Copy`. Local OCCT primary headers
define copied geometry/topology with triangulation omitted for these flags.
The fixture uses that fresh copy for both independent Mesh input sampling and
density samples; it does not mutate the document Shape or assign the copy back.
Raw source checkpoints before/after sampling and display events, and after
observation reads, remain byte-identical in the final report. No alternate
hash, ignored field or normalized Checked flag is used.

The density RED `488686c9d8a54167b7a61b3cdb110862` already passed strict source
hash/radius/revision/guard predicates in both modes, but both 0.8 and 0.05 linear
tolerances produced 4002 vertices/8000 facets. FreeCAD
`TopoShape.cpp::defaultAngularDeflection` uses `min(0.1,linearTolerance*5+0.005)`
and passes it to `BRepMesh_IncrementalMesh`; both old values therefore request
0.1 radians. The final fine tolerance 0.01 requests 0.055 radians and records
13360 vertices/26716 facets versus coarse 4002/8000. This tightens sampling
resolution and demonstrates actual refinement of independent native samples.

The document sphere's `ViewObject.Deviation` changes from 0.4 to 0.05 and its
DisplayMode from Wireframe to Shaded. Primary `ViewProviderExt.cpp::onChanged`
invalidates the rendering cache and calls `updateVisual` for visible Deviation
changes. The fixture records requested/actual mode and density properties,
strict CAD hash, radius, stamp, guard and CAD companion classification. Sample
facet counts are explicitly from independent geometry copies, not measured
viewport mesh-cache counts. The final extension separately exercises the native
CAD working export/import route in each display state; sampling copies never
become its export input.

## Fixture scope and remaining evidence

The companion macro verifies provenance, owned values, rational curve basis,
U-major NURBS surface basis, native CAD/mesh separation, explicit uninspected
limits/representations, revision-scoped subelements, read rejection immutability,
copied-data isolation and document close/reopen identity invalidation. Its
close/reopen sequence is in one FreeCAD process; it is not a separate cold
startup. World transforms are recorded but their composition is not certified.

Root reports final SDK build `450c52` exit 0 and companion run `828efb` exit 0.
The reviewer independently read
`build/merge_replay-1/1efa76209858406d966eefef052da9b4/results.json`:
`ok=true`, 142/142 host predicates, no failures. Report SHA-256 is
`fccfbc378ce164d6e82f6db07741682e6173f83e36449912604c0abb9df4321b`;
FCStd SHA-256 is `2a1c08f5714f3eb6c88c0b4ccc785ffa25e05214a545513e97697034ed1e8c1c`.
Its native module SHA-256 matches the module on disk:
`b80631e87580fb0c33efa63fb65151edbab1ac2b22d1a75bbcb05c2c35df660c`.
Root associates the run with the current macro hash below. The host report
honestly leaves its external adaptive-volume work pending; `ok=true` alone is
not volume acceptance. The separate matching native results close that work:

| Exact-input native result | Result | SHA-256 |
|---|---|---|
| `H:/FreeCAD-src/build/p0-kernel-sphere-wireframe.json` | 64/64, `ok=true` | `a5dd54fbfb4b13ab8866e39d8d20ea153bbd646533378de406aaba434dceb0ea` |
| `H:/FreeCAD-src/build/p0-kernel-sphere-shaded.json` | 64/64, `ok=true` | `644d4bcd3cab8eb5b9adeec0d2d82b571e0096f865b49b85fd7e0fdee65e62df` |

Each native input hash matches its required host BRep and the current 3529-byte
file on disk: `726132b9ce7ca59bc89619fbbd434fff433b2d5a8f44df6afee1223175aedf91`.
Both pass the unchanged `1e-5 mm³` bound: adaptive volume residual is
`9.009895052258798e-9 mm³` and `Eps=1e-14`/`1e-16` pair difference is
`2.842170943040401e-14 mm³`. The native oracle requires a valid solid and checks
signed volume against `36π`, so reverse orientation is not hidden by absolute
volume. Default `Shape.Volume=113.04787279841815` remains a recorded observation
with error `0.04946273081439756`, not accepted numerical accuracy.

Each working route returns one `Part::Feature`, no `Mesh::Feature`, native Solid
with one solid, OM9 CAD kind 3 and `native-cad` representation. Existing native
capabilities report the OCCT/openNURBS working backend; preserved attributes are
still uninspected. Original source strict BRep/radius/stamp/guard predicates pass
after export. Imported NURBS sphere radius evidence is explicitly a 17×9 grid
(153 finite samples, max radial residual `1.3322676295501878e-15 mm` within its
declared `1e-6 mm` sample bound), not a certified global conversion deviation.
Worker settings restore in `finally`; independent imported documents close and
their saved descriptors remain owned plain values.

T04's numbered fixture invariants are covered at this native/application
descriptor boundary: density/display properties change; type, strict CAD hash,
revision and radius remain unchanged; native CAD classification remains solid
with legacy CAD kind 3 rather than becoming Mesh. Explicit native CAD modeling
round trips in both modes add actual route evidence. This covers the numbered
T04 fixture assertions, not every modeling solver's dispatch. No full
T07/T08/T09/T11, Rhino parity, parameter map or global native
memory-safety claim follows. T01 remains sampled, as the exact-kernel review
records.

## Final source and diagnostic hashes

| Source/report | SHA-256 |
|---|---|
| `Gui/CoreGeometryContract.h` | `382a305dbafeb8348b206341d012460850b2d704b0f3e9328e1fcbce11f1a0ce` |
| `Gui/CoreGeometryContract.cpp` | `0a4a042b2e8e51e3d2e774c825a905ccb82d67cc4969f144b748bf133b9e8b8c` |
| `rust/src/geometry_contract.rs` | `25d4ad12b4a17228aa492fe17e548c660334649f79df3e3f658880bbf473f59e` |
| `rust/src/geometry_contract_ffi.rs` | `3547370f686124e3a522a3630073328b6d4c94acf31d63662df82a3a2869770b` |
| `rust/tests/geometry_contract_ffi.rs` | `e3c5d223662a943a4a09ec0e582b7e44eedebbe3fd10c9bf8fe3518dbb190bf7` |
| `tests/rhino_core_geometry_descriptor_smoke.FCMacro` | `3e383da77b66f25edaabd41c885178c2d756b09d30ee42267002472a445fd169` |
| Final 142-check host companion report | `fccfbc378ce164d6e82f6db07741682e6173f83e36449912604c0abb9df4321b` |
| Source-meshing RED `fbd3f12a999a446aaf5af91c0a793492/results.json` | `e2093e405c9832c3d9d98729aa7b4ec92db0aeda171cbbb016ac43b61ee68e34` |
| Density-refinement RED `488686c9d8a54167b7a61b3cdb110862/results.json` | `9b24df2a7d5aa62a32ad5b31f2b7a08276db4eca66c4ece063105343ae5f2b61` |

The two RED reports are preserved diagnostic evidence, not accepted final runs.
No blocking source or numbered T04 fixture defect remains within this reviewed
scope; the explicitly unknown topology/coordinate/preservation fields remain
unknown.
