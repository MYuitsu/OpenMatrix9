# Nested UV CurveOnSurface: restricted Rhino5 exchange

The actual Rhino5.14.522.8390 File3dm probe passed all six owning nested UV
controls on2026-10-08. Source/read/write/reread UUIDs and domains are retained;
17 independently composed interior/end samples have maximum deviation
6.154e-15mm at the unchanged1e-9mm threshold. Source hashes remain unchanged and
the active document identity/object inventory is unchanged.

Actual isolated FreeCAD reimport passes25 checks. The complete decoded native
child trees, dependencies, geometry CRC and UUIDs match before/after Rhino write.
Rhino-written archive bytes and immutable source records survive external source
deletion and FCStd reopening. Frozen source/saved files, original oracle, exact
script and returned JSON are in `rhino5-nested-profiles-20261007/`; the folder date
records fixture preparation, not the date of the user's actual run.

The V5 writer now admits one owning nested parameter CurveOnSurface: its parameter
is a2D LineCurve and its surface is a non-rational2D NURBS with two CVs/order2 in
both axes, unit knot domains and an axis-aligned positive rectangular CV grid.
The grid lies inside the outer surface domains and the line endpoints lie inside
the inner surface domains. Optional C3, rational/sheared/higher-order UV maps,
different child classes, deeper nesting and other nesting locations remain
guarded. Child userdata, metadata/numeric/node/depth/cycle limits still use the
same bounded traversal; the admission pointer identifies only the verified UV
child. No native child is removed or converted to mesh.

Native writer tests observed the old rejection before enabling this profile.
Selected unchanged/moved V5 outputs keep the complete UV subtree and independent
physical samples. Thirty native-valid unsupported-property controls still
refuse, including C3, Arc, rational UV, deeper nesting and shear; their direct
writer counterparts leave protected destination bytes unchanged.

The expanded actual FreeCAD profile suite passes60 checks, covering exact fields,
coupled public V5 placement, independent UUID copies, deleting originals,
Undo/Redo and source-deleted FCStd with export after reopen.

That final operation exposed a separate top-level-copy identity defect: C++ had
called ON_CreateUuid on every export. Geometry remained identical but copy UUIDs
changed. Copies now use the same namespace/source/host UUID-v5 identity contract
as existing member copies. Source identity and document properties are not
mutated. Native tests reproduce the old instability and verify repeated/mixed
selection identity, plus atomic refusal of collisions with source geometry or
layer components. Old random output UUIDs were not stored in projects and cannot
be reconstructed from those projects.

This is scoped API/native/FreeCAD evidence. Actual Rhino GUI Open/SaveAs and the
OM9 unchanged/moved writer outputs require their next separate target run. Full
reference remap/global mapping and the remaining approved packages remain open.
Primary source is not integrated or pushed; SDK and all existing tolerances stay
unchanged. Final regression/binary evidence is recorded in the current proof
summary and execution ledger.

## Final regression and pending GUI proof

Fresh final binary passes 42 native suites and 27 serial FreeCAD reports with
2,131 checks, including 894 original-ring checks at unchanged 0.001 mm bounds.
The V5 nested suite includes 143 native checks; all 11 coverage/measurement/
evidence Python tests pass. Captured source hashes and installed Python bytes
agree. These counts do not measure full exchange completion.

Twelve OM9 unchanged/moved writer outputs and the independent point oracle are
frozen in `rhino5-nested-writer-20261008/`; actual GUI Open/SaveAs and decoded
reimport remain pending. Metadata preflight is scoped to GUI RhinoObject and
ObjectTable aliases, with two negative controls. The earlier File3dm-only
checker failure is preserved as a metadata-scope failure, not evidence of an
actual Rhino API failure. Source SDK, application tolerances and primary checkout
remain unchanged.

## Actual nested GUI acceptance and expanded controls — 2026-10-08

Restricted nested GUI12/12 now passes; FreeCAD73/73 verifies full native children,
UUID/dependencies/CRC and source-deleted FCStd. Two GUI plugin tables remain
preserved with atomic unsafe-closure refusal. Modal warnings are not established
by the command log. This supersedes the earlier pending GUI entry.

Ninety independent sheared/rational/bicubic nested controls pass native generation
and FreeCAD721 retention checks. Public V5 export remains guarded pending actual
Rhino5 API proof. Native42/42 and runtime29 reports/2,925 checks are captured;
counts describe tests, not completion percentage. Production binary unchanged.
Full exchange remains incomplete, primary source not integrated/pushed.

See [evidence and limits](2026-10-08-opennurbs-expanded-nested-uv.md).
