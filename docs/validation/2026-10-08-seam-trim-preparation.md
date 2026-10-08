# BRep seam/pole applicability and cylinder-cap fix — 2026-10-08

Independent radius3mm sphere and radius3mm/height7mm capped cylinder fixtures
verify both seam sides, singular north/south trims with no3D edge,17 physical
mapping samples per side, native topology indices/winding, immutable sources,
and signed analytical volumes36π/63π at the original1e-6mm³ threshold.
211 native checks pass. A singular trim has no component3D edge by topology;
this does not make regular/seam PolyEdge references universally incompatible.

The capped-cylinder top edge has a negative proxy domain while its UV circle
has a different positive domain. OCCT SameParameter interpolation failed even
though the curves physically coincide. Imported pcurves now use the edge
interval. An affine-plane shortcut verifies the complete spline/control basis,
positive rational weights with a convex-hull error bound, and bounded planar
cross term. Independent Bezier span breakpoints may be reparameterized without
changing their loci; source data/knots/weights remain untouched. Other bases
use OCCT synchronization. No blanket exception suppression or tolerance change.

Actual FreeCAD65 checks pass for both fixtures: valid editable solid, independent
adaptive volume, copy/UUID/native-geometry retention, selected V5 reimport,
source-deleted FCStd reopen, real scale edit/export/reimport, Undo/Redo and
delete-copy/Undo. Edited volumes use the independent analytic scale³ factor.

Current binary regression50/50 native suites,38 installed runtime reports and
5,761 checks. Both rings retain0.001mm bounds and existing area/volume thresholds.
SDK commit eb92af3ba1806b0a34a99aba0d3bda83e3d46083 remains pinned.

One prepared Rhino5 batch contains4 files:2 source fixtures and2 actual edited
FreeCAD exports. GUI Open/SelBadObjects/SaveAs/reopen is pending; no actual
Rhino5 compatibility claim yet. After actual execution, decode native graph and
reimport saved files. Seven packages2–8 remain open; total future batch count
unknown. Full exchange is incomplete; primary source integration/push not done.
Evidence: seam-trim-20261008/native-results.json, freecad-final-results.json,
oracle.json and preparation.json. The UV-reference8 failure evidence remains
unchanged; this BRep adapter fix does not make those references V5-compatible.
