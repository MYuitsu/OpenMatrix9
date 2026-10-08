# Expanded owning nested UV V5 admission — 2026-10-08

Actual Rhino5.14.522.8390 API read/write/reread passes all90 independently
generated controls: six outer surfaces, five parameter classes and three inner
UV maps (sheared bilinear, rational bilinear, nonrational bicubic). Inputs and
the active document are unchanged. Actual FreeCAD reimport361 checks confirms
identical native child trees, UUID, dependencies and geometry CRC after Rhino
write, including exact saved archive bytes after source-deleted FCStd reopening.
Exact report, command log and90 saved files are frozen in
`rhino5-expanded-nested-20261008/`.

Native selected-writer tests first failed with the existing profile rejection.
After admitting this bounded family,359 checks pass: unchanged/moved output
preserves every UV field and UUID, with17 independently composed physical
samples at unchanged1e-9mm tolerance.48 native-valid negative controls and48
direct writer controls still reject C3, parameters outside the unit domain,
unsupported quadratic maps, deeper nesting, UV CVs outside outer domains,
negative surface/parameter weights and unverified rational bicubic maps;
protected destination bytes remain unchanged.

The profile admits exactly one owning nested CurveOnSurface, no C3, with a2D
Line/Arc/Nurbs/Polyline/PolyCurve parameter and2D unit-domain single-span NURBS
UV surface: order/CV counts2x2 (nonrational or positive rational weights), or
4x4 nonrational. CVs are finite, inside outer domains, and increase in X with
the first grid index and Y with the second. Parameter curves require positive
rational weights, including PolyCurve segments; the complete parameter bounding
box must lie inside the inner domains. Existing full child traversal limits
apply before arbitrary curve validity/bounding-box work. The admission pointer
identifies only that UV child; deeper nests and unknown native children remain
refused. This is an exchange/placement profile, not global correspondence,
invertibility or a general UV editor/CAD-preview certification.

Actual FreeCAD V5 lifecycle passes900 checks across90 cases: exact unchanged
export, transformed native staging versus public moved V5 fields, distinct
copy UUIDs, deleting originals, Undo/Redo and export after source-deleted FCStd
reopen. Full native42/42 and30 serial runtime reports with3,465 checks pass on
the freshly built/linked/installed module. Both original rings retain347+547
checks with0.001mm bounds and existing area/volume thresholds. Prior29-report/
2,925-check proof is archived in
`opennurbs-checkpoint-20261008-before-expanded-v5/`.

Ruling: admit only positive-weight bounded single-span UV families supported
by actual target and exact decoded-tree evidence — no optional child removal
or implicit meshing. Conservative unsupported-property guards can refuse valid
native data; that data remains preserved in the project rather than declared
incompatible with the format without actual target evidence.

The180 unchanged/moved OM9 writer outputs and independent oracle are frozen in
`rhino5-expanded-writer-20261008/`. Actual GUI Open/SaveAs/reopen and decoded
native reimport of those files remain pending. Metadata preflight checks only
API names/arity. Original restricted GUI12/12 and73 reimport proof remains
separate; its command log does not establish absence of modal warnings.

Full reference remap/global correspondence, general shell applicability,
remaining geometry/annotation/resources/document/history packages and final
integration remain open. SDK stays pinned; primary source is neither integrated
nor pushed. Test counts do not represent a completion percentage.

Final verification: refreshed coverage6/6, GUI mass replay2/2 and evidence binding3/3 Python tests pass. Production source and installed binary hashes agree;180 frozen writer files/oracle/script unchanged. Actual GUI probe dispatched; do not run another application while the user may execute it.
