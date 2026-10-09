# Rhino core: authored subobject mutation owners and lifetime

Date: 2026-10-10. Baseline `422098b` on
`codex/rhino-core-object-foundations`. **The authored RCORE-01.T11 fixture is
verified; complete P0 remains in progress.** No production code was changed.

FreeCAD already has the required copies and request guards. The native Edge.Curve
and Face.Surface getters create independent geometry wrappers; `copyGeom=True`
copies native shape geometry. An `App::Link` can share a definition while a
Rust-backed OM9 object snapshot tracks document session, source revision,
replacement and Undo epoch. The fixture exercises these paths without adding
another editor, serializer or ownership model.

The authored definition contains a Wire with a Line and clamped degree2 BSpline
subcurve, plus an untrimmed clamped degree2×2 BSpline Face. Both have unit weights
and identity source frame; two translated App::Link instances share that exact
definition. Exact source CAD, member names, sibling world CAD/link placements,
native shallow/deep snapshots and document/property/Undo inventory are captured.
A strong native Shape alias has the same topology identity; explicit deep copies
have different topology identity. Neither is a borrowed raw pointer.

Mutation owners are named before each operation. The detached subcurve middle
pole changes from(2,2,0) to(2,2,2), giving midpoint(2,1,1). The detached Surface
center pole changes from(1,1,0) to(1,1,4), giving midpoint(1,1,1). Independent
quadratic/tensor Bernstein weights at0.5 make these analytic expectations.
Source points remain(2,1,0)/(1,1,0). Source/siblings/native snapshots remain
byte-exact, no object/property/Undo is added and source subobject guards stay valid.

Publication explicitly promotes geometry through existing
`toShape().copy(copyGeom=True,copyMesh=False)` into two new standalone Part::Feature
owners in one transaction. This boundary matters: `toShape()` alone retains the
wrapper's mutable support. A real fixture RED demonstrates subsequent retained
wrapper edits changing those directly published shapes. The existing native deep
copy fixes that ownership assumption; no production defect or replacement
algorithm was inferred. Retained wrapper poles are then changed again to z6/z12,
giving midpoint z3, while published raw bytes, full native basis and midpoint z1
remain unchanged. One Undo removes both outputs, and Redo restores their geometry.

The fixture separately names the canonical Wire Shape property as the mutation
owner for an intentional source replacement. Both shared-definition instances
then change as intended; the independent Face and standalone outputs stay exact.
The old Edge request rejects with the concrete stale-source reason, while the
independent Face request remains valid until Undo changes the document epoch.
Undo restores source/sibling geometry and rejects the previous Face token.

After document close, the detached support wrappers and strong native shallow/deep
shape snapshots remain usable. Document/subelement requests reject on same-name
recreation. The new session first has no objects, then recreates both source names
and equivalent native geometry: old requests still reject and fresh Edge/Face
requests validate. Request evaluation adds no object, property or Undo. This
distinguishes retained native geometry lifetime from semantic document ownership.

## Evidence

The [evidence file](2026-10-10-rhino-core-subobject-ownership-evidence.json) pins
source, module, macro, actual result/process, raw BRep, native-source review and
prior reused verification hashes. The final fresh native FreeCAD process passes
35 checks and exits0 on module `8b03c899…`; all six stale errors match the expected
source-change prefix. Exact source-before/source-after and published-before/after
re-edit BReps are retained and equal. The
[independent review](../reviews/2026-10-10-subobject-ownership-review.md) checks the
native getter/copy/toShape routes and the actual scoped runtime evidence.

The publication alias RED is preserved at
`build/merge_replay-1/4315b76011404021bc171cc29a0c663a`:14 prior checks pass,
the output-isolation assertion fails and recorded native output midpoints become
z3. Native deep copy yields34 passing checks; an added explicit basis/midpoint/raw
file witness gives the final35. Earlier27/32 passing runs lacked this stronger
boundary and are not counted as acceptance or additional assertions.

Two excluded recreation attempts demanded byte identity between distinct native
topological owners. Raw diagnostics show new composed identity TopLoc records,
use-location indexes and owner Free flags, with geometry unchanged. The final
recreation control compares exact native placement/vertices/edge class/domains/
orientations/endpoints/BSpline basis, ordered Wire orientations and Face domain/
orientation/surface basis. The raw files remain unmodified. This predicate is
appropriate for the authored Line+quadratic Wire and untrimmed tensor Face;
source/sibling and retained-snapshot byte-equality assertions stay strict.

Parent Rust541/native27/5CTest/SDK proof and current module are reused, explicitly
not rerun: product code remains unchanged. The fixture-only native host/API use is
not portable business logic implemented in Python.

## Remaining scope

T11's authored owner/isolation and standalone-versus-subobject lifecycle fixture
is complete. This does not certify arbitrary raw borrowed handles, native
mutators in every backend, general trim/topology equivalence, all-family edit
ownership, callbacks/leak counts, sanitizers or global absence of UAF. The tested
standalone evaluation after close is not delayed solver completion/release proof.
No hole/seam was silently discarded: the edited Face is explicitly untrimmed.

Full T07 allocation coverage, T08 delayed result publication/lifetime and T10
guarded host parameter dependency, plus remaining explicit P0 contracts and
chapters, stay open. Continue with the actual result-publication boundary using
existing native solvers, owned result buffers and the established source guards.
