# OM9-FILE-012 — actual Rhino5 signed geometry and member safeguard

Status: **partial / in_progress**. Full openNURBS128/16/6 and native
renderer/resource/plugin/history semantics remain incomplete. Public repository
has not been integrated or pushed by this follow-up.

## Actual Rhino5 outcome

`H:\FreeCAD-src\build\rhino5-retest-20261007-145055\application-test-20261007-145132`: Rhino5.14.522.8390, harness4, all9 file lifecycles True, no invalid
leaves or Unicode exceptions. The user directly confirmed no warnings/errors;
`user-confirmation.json` retains that observation separately from command logs.
Both exported ring diamonds retain volume about -1.257183756385 mm³, matching
original -1.257183756391. Both rings and four samples pass FreeCAD reimport and
FCStd reopen. The added native preserved member fails: -2880 becomes +2880 mm³.
Actual aggregate analysis remains **43/46**, FreeCAD reimport **28/29, process1**.
These raw failures and the old runtime hashes remain immutable evidence.

Two strict source/export bounds errors remain **0.001614619745 mm >0.001 mm**:
UUIDs502f1501-11ed-4233-950b-2f9f97a5fdcf and
f82df7e7-7ec2-4edf-b880-3fadf26c3c70. Independent4227 trim-aware bidirectional
samples max9.07302011443e-6 mm provide sampled physical correspondence; they do
not certify unsampled extrema or turn strict bounds failures into passes.

## Bounded repair after actual member failure

Public Rhino5 preservation now refuses every known inward BRep, including
definition members, before opening output. Target bytes and source snapshots
remain intact. Exact retained class/UUID/userdata/history graphs are not silently
replaced with synthetic definitions. Full native preservation of inward members
therefore remains unsupported.

Explicit Geometry only on placed blocks assembles the current graph in a
separate **internal SDK80** stage. Existing closure, UUID/class, native payload,
attribute, resource refusal, source hash and atomic readback checks remain.
Single-source, sourceless-new and multi-source recursion propagate a fixed
purpose; public preservation has no JSON switch to bypass its V5 safeguard.
Flattened geometry then uses the reflected outward-member codec to create V5
output carrying the original physical position and signed placement. Geometry
only omits source tables, original definitions/history/userdata as documented.
Editable CAD proxy odd/even parity and physical-parent placement remain covered.

## Standalone SDK +2 repair

The former outward fallback is replaced with retained native face/trim winding.
Infinity classification establishes IN=-1 or OUT=+1 on a valid converted solid,
without reversing unknown solids. An independent copy protects native caches
and payloads. Resolve direction before affine transformation and apply signed
determinant parity; identity import retains winding and export/reimport retains
the resulting direction. Known +/-1 and public/internal export purposes remain.

The public V5 guard also resolves unknown inward roots/members before writing.
A native-valid BRep outside the one-solid converter, such as disconnected
shells, cannot currently be classified by this path and therefore refuses
preservation with the object UUID and classification reason. Its source remains
valid and unmodified. This is a bounded capability restriction, not arbitrary
native unknown-solid preservation or a claim of full openNURBS support.

Accessible official pinned McNeel GitHub/raw and OCCT 8 sources, rationale and
an offline-readable summary:
`docs/validation/2026-10-07-opennurbs-orientation-sources.md`.

## Fresh verification after +2 repair

- Test-first RED: signed unknown import and public unsafe preservation failed
  in native33/35 before the repair. Separate UUID/context test failed when its
  wrapper was temporarily removed. Both RED logs retained in the new pack.
- Final native **35/35 process0**. Independent +2 inward/outward sources, shear,
  nonuniform scale, odd/even reflections, export/reimport and source CRC/cache
  immutability pass. Native-valid unclassified multi-shell refusal protects target.
- Final FreeCAD **1910/1910 in57 suites**, user rings **894/894**, process0,
  source/install/binary/reader SHA guards. Nested/shared full signed volumes
  `[-2880.0, -720.0, -576.0, 24.0]` mm³ now pass, including the formerly wrong +576 -> -576.
- Source SDK and original fixtures remain unchanged. Existing OCCT deprecated
  alias warnings and optional missing Vulkan headers do not prevent the build.
- New ten-file pack `H:\FreeCAD-src\build\rhino5-retest-20261007-160440` contains two unchanged originals and eight fresh
  V5 geometry exports. All eight independently decode as genuine RDK3 in the
  pristine SDK2013 reader. Ninth case tests safe inward member -2880/+120;
  tenth tests independent +2 block sources and edited geometry
  -2880/-720/-576/+24. **Actual Rhino5 lifecycle now passes all ten cases**; direct user
  confirmation reports True and no warnings/errors.
- The previous actual43/46 and host28/29 process1 evidence remains untouched;
  earlier ring sign/RDK success does not certify these fresh block cases.

Read-only reviewer found no stronger correctness issue; requested explicit
converter scope and UUID/context. Both are documented and regression-tested.
Next: strict bounds/extrema and incomplete native multi-shell/resource/plugin semantics.

## Actual ten-file Rhino5 verification

Report: `H:/FreeCAD-src/build/rhino5-retest-20261007-160440/application-test-20261007-160529/results.json`.
Ten actual open/SaveAs/reopen lifecycles pass. Direct user observation reports
True with no RDK warning or other error; retained separately in
`user-confirmation.json`. All source and saved-output hashes remain verified.

The safe Geometry-only member retains -2880/+120 mm³ in Rhino. The SDK+2
nested/shared case retains -2880/-720/-576/+24 mm³, including the previously
wrong reflected sign. Eight Rhino-saved exports import into the same guarded
FreeCAD runtime and persist through FCStd: **33/33 process0**.

The first host reimport produced31/33 because duplicated source names received
generated FreeCAD label suffixes in a different insertion order. Its output,
log and analysis remain in `identity-matching-red`. The harness now records
native UUID/name from the same preparation/transaction as import_file and
compares geometry occurrences by UUID/kind/bounds. Native names, layers,
visibility, lock, color, topology and all old metric thresholds remain checked.
Generated host labels still must persist exactly through FCStd save/reopen.
Fourteen differential checks reject UUID/name, signed volume, bounds and
metadata corruption. This changes the test pairing; no production runtime or
geometry was changed after the native35/35, GUI1910/1910 and rings894/894 run.
Read-only review confirms the two native objects have distinct stable UUIDs
but the same native name; only enumeration/suffix assignment changes. The
harness does not certify order-independent generated host suffixes.

Aggregate analysis is **49/50 process1**, with the one failed bounds assertion
covering the same two source UUIDs and deviations0.001614619745 mm >0.001 mm.
No threshold was relaxed. The old43/46 actual and28/29 host RED artifacts remain
unchanged. Exact inward/native preservation, unsupported native+2 multi-shell
and full openNURBS semantics remain partial.

rhino3dm source audit also finds no separate +2 solver in its inspected getter
path; see the pinned references in `2026-10-07-opennurbs-orientation-sources.md`.

## Ring bounds diagnosis follow-up

Actual pristine Rhino points disprove containment by the original bbox. Supplementary empirical geometry analysis **50/50 process0**. Raw bbox analysis remains **49/50 process1**, retained verbatim. See `2026-10-07-rhino5-ring-bounds.md`. Converter geometry and original fixtures are unchanged.
