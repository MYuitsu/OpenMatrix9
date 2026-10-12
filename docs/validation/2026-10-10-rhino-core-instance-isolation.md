# Rhino core instance isolation — 2026-10-10

Status: **validated bounded increment; all P0 remains in progress**. Parent661529f.

RCORE-01.T06 reuses the existing FreeCAD App::Link/ScaleVector, native archive
expansion and structural/working 3DM exporters. The new files are acceptance
fixtures only. No production Rust, C++, Python, command or solver changed.
Literal world-coordinate checks establish transform composition, while exact
source/member/sibling stamps establish isolation. Counts and shaded appearance
are insufficient to establish either property.

The host fixture creates one box(2,3,4), member Placement(1,2,3), and one shared
definition Placement(4,5,6). LinkTransform=True on both links. A has identity
rotation, Placement(100,200,300) and unit scale; B has Placement(20,30,40),
positive90° Z rotation and scale(-2,3,.5). Both target the same native definition.

For member coordinates p and q=p+(4,5,6), independent expected coordinates are
A=(100+qx,200+qy,300+qz) and B=(20−3qy,30−2qx,40+.5qz). All eight literal corners
must match bijectively within **1e-7mm**, with exactly eight native vertices.
A occupies x[105,107],y[207,210],z[309,313]; B occupies x[-10,-1],y[16,20],
z[44.5,46.5]. B's linear determinant is−3. The fixture changes only B by
(7,-11,13) in one transaction, then checks actual coordinates through Undo/Redo.

Native independent root matrices include the definition translation once:

```
A = [1,0,0,104; 0,1,0,205; 0,0,1,306; 0,0,0,1]
B = [0,-3,0,5; -2,0,0,22; 0,0,.5,43; 0,0,0,1]
B after edit = [0,-3,0,12; -2,0,0,11; 0,0,.5,56; 0,0,0,1]
```

Actual structural V5 archives retain one definition, one canonical local BRep
member and two distinct root instances sharing that definition. Actual import
matches leaves by root UUID, covering both roots exactly once. Working export
separately produces two independent native CAD leaves at their current positions;
it is explicitly a flattening route. Host canonical member geometry/placement/
UUID, definition placement/UUID/member order and A's complete current geometry/
placement/scale/target stamps remain exact through B edits and both exporters.

The native fixture serializes the full ON_Object payloads of the local member,
definition and sibling A. Each remains byte-identical after the B-only overlay.
The retained original native owners and input archive bytes remain unchanged.
Singular and projective structural matrices produce concrete transform errors,
leaving an existing destination and source byte-identical. Actual native IEEE
NaN and Infinity on detached ON_Point geometry reject before mutation; JSON
null/string input is not substituted for those native-number tests.

Host singular-scale rejection compares against the immediate deliberately invalid
input state: object count, Undo count, A, canonical member, B placement/scale and
existing output/source bytes remain unchanged. Undo restores the prior valid B.
The fixture saves FCStd but does not claim fresh-process cold reopening for T06.

## Validation and diagnostic rulings

| Check | Current result |
|---|---|
| Direct native T06 fixture | 146 checks; build/run0 |
| FreeCAD T06 | 69 checks; fresh process exit0 |
| Existing native regressions | 4 pass: Blocks, NativeTransform, Merge, ModelingExchange |
| Existing FreeCAD new-block regression | 29 checks; fresh process exit0 |
| Production/native module reuse | SHA25661708e51c37e9dc150d355b968a5be9ab29886858ae9ce0cea108fc2b87e5e09 |

The 98 application checks use the same frozen module as checkpoint661529f;
unchanged production-source hashes are checked against that checkpoint. Its
523Rust/Clippy and SDK proofs are historical reused evidence, not new runs.
There is no invented production RED or implementation change when existing
required behavior already passes. Acceptance artifacts and failed fixture
diagnostics are recorded separately in [fingerprinted evidence](2026-10-10-rhino-core-instance-isolation-evidence.json).

1. Optimal CAD bounds use pinned BRepBndLib::AddOptimal(false,false), which still
   pads numerical bounds with OCCT Precision::Confusion=1e-7. Bbox-only checks
   therefore declare that enclosure padding plus64ULP roundoff separately.
   The geometric corner oracle remains1e-7mm; observed residuals are about1e-14.
   This follows local BRepBndLib.hxx69–81, Precision.hxx165 and
   TopoShapePyImp.cpp2549–2572, not an increase in model tolerance.
2. Independent host structural exports allocate fresh output-layer UUIDs. Full
   raw inventories are retained, while exact geometry UUID/class/role/CRC/target/
   matrix records are compared separately. Definition semantic records remain
   exact. Geometry CRC is an observation, not a cryptographic payload proof;
   the direct native full-serialization checks provide that separate proof.
   Output-layer identity and general attribute parity are not claimed.
3. Native AddModelGeometryComponent clones the box before this fixture computes
   the standalone box's bbox. A first diagnostic baseline compared that model
   member's unprimed serialized cache
   against bounds necessarily computed by the structural writer:13 bytes differed
   in bbox fields and chunk CRC, while sibling/definition were exact. The authored
   model member's correct bbox is now computed and asserted before the source
   baseline. Pinned ON_Brep assignment copies its cache; this is an authored
   initialization-order issue, not a general clone-clears-cache claim.
   No payload bytes are removed or normalized in the exact comparison.
   This does not claim an unprimed cache remains unchanged through export.
4. Archive leaf names include their parent, e.g.A/Canonical. The first native
   fixture chose the wrong literal table by name; it now uses exact root UUIDs.
   Compiler/API/fixture-predicate failures are excluded from runtime acceptance.
5. Independent review strengthened the host oracles to require each selected
   root exactly once, and each working leaf A/B exactly once. The former64-check
   run had correct A/B data but weaker duplicate coverage; the current macro is
   accepted only by the fresh69-check process.

[Independent review](../reviews/2026-10-10-instance-isolation-review.md) records
the inspected source/artifact hashes and limitations. Only RCORE-01.T06 is
advanced. Existing607-entry command catalog/statuses remain unchanged; the
package manifest, README, capability lookup and live matrix remain synchronized.

## Remaining work

This authored two-instance fixture does not certify every representation,
nested/empty definition, finite-determinant overflow case, reference-owner
mutation, History, command UI, worker cancellation or whole-process memory
safety. The current archive expansion gate's finite-determinant qualification
remains explicit. No new Rhino5 runtime parity is claimed. T07/T08/T09/T10/T11
and the remaining P0 clauses require their own direct evidence and missing-code
implementation after the reuse audit.
