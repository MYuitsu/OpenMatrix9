# openNURBS packages 1–3: current evidence and remaining work

The approved eight-package plan is still in progress. These are verified slices, not a percentage or a claim of full exchange. Primary source has not been integrated or pushed. SDK remains pinned at `eb92af3ba1806b0a34a99aba0d3bda83e3d46083`.

## Coverage discovery

The authoritative normalized catalog is `docs/3dm-capabilities.json`. Complete discovery finds131 source registrations:130 macro registrations across four variants and one manual ClassId. Linked standalone and actual isolated FreeCAD both expose128 matching classes. Three obsolete userdata registrations are excluded by pinned SDK CMake. Five historical entries were commented registrations. The old128-row catalog and historical123-name audit remain as original evidence; neither was complete current discovery.

Classification is98 concrete,10 abstract,2 helper and21 obsolete compatibility registrations. Base CurveProxy/SurfaceProxy Read and Write return false, so direct archival exchange is not applicable; concrete owners/subclasses have their own adapters. Owned BRep subobjects and legacy factory upgrades are identified separately. Every row carries source/runtime evidence and six separate capability axes. Scoped reports do not establish class-wide completion.

## CurveOnSurface and PolyEdge

Native trim mapping uses OCCT physical projection followed by independent native UV/surface evaluation. Nonlinear parameterization, reversal, incorrect interiors, ambiguous branches and immutable source were tested. Closed disk seams now distinguish identical physical endpoints using owning topology. Angular ArcCurve/type2 NURBS parameters are tested separately. The latest mapping suite passes152 checks. This does not certify a global continuous map for every singular/seam surface.

The32 curved domain fixtures retain15 verified interior samples and explicit numerical scope in the inventory. FreeCAD176 checks verify source ownership, invalid-reference atomicity and exact FCStd source retention after external source deletion.

Actual Rhino5.14.522.8390 read four coherent controls. The no-C3 plane/line control retained its UUID/domain and all17 physical samples with zero deviation. All three controls containing `m_c3`, including geometrically exact approximations and references, lost the CurveOnSurface object. Exact source files, native oracle and application JSON are archived in `rhino5-coherent-reference-20261007/`.

**This is observed Rhino5 reader incompatibility, not proof that the V5 archive format cannot encode `m_c3`.** Native/repaired readers do retain it. OM9 keeps the complete source and refuses actual-Rhino5 export rather than remove the optional child or change the native class.

The writable profile now includes five2D UV curve types (Line/Arc/Nurbs/Polyline/PolyCurve) on Nurbs/Plane/Rev/Sum/Extrusion surfaces without C3 and with safe userdata. All30 coherent child profiles, including2D rational surfaces, passed actual Rhino5 File3dm read/write/reread and exact decoded child comparisons on native reimport. Selected unchanged/moved exports pass71 native checks. Actual Rhino GUI Open/SaveAs/reopen passed35/35 profiles with zero bad objects. Fresh FreeCAD484 checks across35 API and35 GUI outputs cover exact native values and the supported coupled placement, Undo/Redo, independent UUID copy/delete and FCStd reopening paths. The separate23-check C3 guard suite remains green. Nested surface curves remain refused pending their own evidence.

Rhino GUI SaveAs adds three opaque plugin user tables. Full saved bytes survive FCStd reopening; selected export refuses unknown dependency closure atomically. These tables are not stripped. Frozen GUI evidence is in `rhino5-gui-exchange-profiles-20261007/`. The user's True-only answer does not establish the absence of modal warnings.

Six independent owning nested UV CurveOnSurface controls now preserve exact decoded child fields and17 independently composed interior points natively. FreeCAD36 checks cover source identity, target guard atomicity, coupled model placement retaining UV fields, Undo/Redo and source-deleted FCStd. Fixtures and oracle are frozen in `rhino5-nested-profiles-20261007/`. Actual Rhino5 retention is pending; no nested V5 writer profile has been enabled.

## Multi-shell and cavities

The converter validates all closed shells, complete face ownership, separation, containment forest and alternating cavity winding. It retains disjoint solids, outward cavities and islands in cavities. It does not discard shells. Native32 checks cover analytical signed volumes48/784/792mm³, actual SDK+2 append-built sources, reflections and immutable CRC/cache.

An inward multi-shell complement with holes has no verified valid editable OCCT representation in the current converter. Preserve import retains the native BRep with an explicit `representation_issue`; geometry-only import refuses it. Complete homogeneous shell winding can still resolve+2 to-1 and a reflection can produce a valid outward editable solid. Mixed material directions remain unresolved instead of receiving an invented global sign.

FreeCAD46 checks verify six box fixtures, topology/sign, explicit native-only status and FCStd source retention. Native-only host placement is separate from immutable class-level source applicability. Inward V5 preservation still refuses known target sign reversal before changing destination. Actual Rhino5 API and GUI lifecycle passed all three positive shell fixtures; native reimport retained UUIDs, full graph CRC and exact48/784/792mm³ volumes. Frozen evidence is in `rhino5-exchange-profiles-20261007/` and `rhino5-gui-exchange-profiles-20261007/`.

Four additional independent analytical fixtures cover disjoint cylinders and cylindrical, spherical and toroidal cavities. Native and actual FreeCAD verification retain both shells and the complete native graph through FCStd reopening after external source deletion. Singular pole trims now have degenerate host edges; periodic seam trims share one edge with both pcurves in the native edge domain. Closed standalone periodic faces receive individually checked shells. This establishes these fixtures, not arbitrary touching or intersecting shells.

Actual Rhino5 GUI read/save/reopen completed all four valid outward solids with zero bad objects, but the strict default-volume comparison passed only1/4. Three deviations are4.260479e-6,3.807502e-6 and7.038183e-6mm3 against the unchanged1e-6mm3 limit. First-read and resaved Rhino measurements are identical. Actual FreeCAD reimport29/29 verifies all saved UUIDs/native graph CRC, two valid closed shells, adaptive analytical volume and complete source-deleted FCStd archive retention. Frozen original failed report and exact inputs/saved files are in `rhino5-curved-shells-20261007/`.

The actual Rhino5 read-only precision diagnostic now passes8/8 source/saved measurements. Brep.GetVolume(relativeTolerance,absoluteTolerance) at1e-13 agrees with independent analytical volumes within1.993e-10mm3;1e-12→1e-13 convergence and source→SaveAs volume deltas are zero for all four fixtures. Inputs and active document remain unchanged. Its initial ObjectAttributes.Id failure and corrected ObjectId script are retained separately. Installed DLL/AST preflight establishes API names/arity only; the actual returned JSON establishes numerical results.

`combined-acceptance.json` binds that separate precise measurement to the original GUI lifecycle using source/saved file hashes, oracle hashes and decoded UUID/topology/volume values. It does not rewrite the original False report. The common GUI measurement adapter now requests1e-13 precision and records default volume/error separately, retaining1e-6mm3 acceptance. Two replay tests cover all four measured cases and deliberate wrong-volume rejection; these are adapter tests, not another actual Rhino GUI run. Three evidence-binding tests also reject changed saved bytes and altered numeric measurements despite a True flag.

Native-only definition members now use host representation applicability, rather than immutable class-level editable applicability, when composing placement. Actual FreeCAD19 checks cover canonical/selected members, independent copy, promoted proxy, Undo/Redo, modified payload refusal, source-deleted FCStd and atomic export guards. The independent real member fixture is native V5. Reflected internal SDK80 staging and public explicit Geometry only V5 export decode as two separate valid CAD cavities at the exact expected bounds and784mm³ each. Copy UUIDs remain separate. Public preservation retains its inward-sign refusal. This is actual FreeCAD writer/reader proof, not actual Rhino5 application proof for this member profile.

Curved FreeCAD volume checks use adaptive integration at1e-13 and keep the original1e-6mm³ analytical threshold. Default `Shape.Volume` reports a separate quadrature discrepancy (about0.9% for these rational cylinders); it was not corrected or substituted silently. The read-only adaptive measurement API records both values and rejects invalid precision.

FCStd's fixed-decimal BRep writer changes tiny nonzero values on save. Comparing pre-save text hashes incorrectly staged replacement of an unchanged native BRep. New imported CAD objects retain a hidden, read-only Part shape baseline; both properties use the same FreeCAD persistence. Exact decoded geometry comparisons ignore only IEEE signed zero and OCC's Checked cache flag, with every nonzero digit retained. The baseline is never reset during save or restore. FreeCAD46 curved checks now include a1e-9 scale edit, Undo/Redo, edited FCStd reopening and native graph preservation for unchanged sources. Older projects retain their original signature contract; no unverified legacy migration is performed. The additional baseline increases stored CAD data; the immutable source archive remains separate and unchanged.

## Regression and limits

Fresh native regression passed42/42, including mapper152 checks, selected UV/surface profile71 checks and the curved shell suite. Actual isolated FreeCAD reports: registry3, box shells46, V5 guard23, trim domains176, API/GUI reimport484, curved shells46 and actual Rhino curved reimport29 checks. Current reports and binary/source hashes are in `opennurbs-current-proofs-20261007/`. The final serial registry rerun now passed after all builds/copies completed; it supersedes the earlier launcher/module-copy overlap. Catalog generation, coverage Python6/6 and current audit passed at the preceding checkpoint and are refreshed separately when evidence changes; none imply class-wide exchange completion. C: test-temp exhaustion was resolved with process-local TMP/TEMP on H:; no tolerance or512MiB production boundary was relaxed. Historical35-suite/1910-GUI evidence is not relabeled as a fresh run.

Package1 is complete for source/runtime/property evidence normalization. Still open: complete reference copy/remap/current-edit paths; further singular/global mapping applicability; nested no-C3 target profiles; general curved-shell/tangent/touching cases and legacy CAD baseline migration; packages4–7; final review, source integration and installed application acceptance. Both user ring fixtures and original bounds evidence remain unchanged;0.001mm bounds and existing area/volume thresholds are unchanged.

Latest serial checkpoint after member fixes and independent fixture extensions:
native42/42, actual isolated FreeCAD26 reports/2082 checks, including native-only
members19, nested owning UV36 and both original rings347+547=894. Fifteen broader
member/block/copy/reference suites are included, not relabeled historical runs.
Coverage6, GUI measurement replay2 and evidence-binding3 Python tests pass.
Current production hashes and installed Python policy agree with captured proof.

## Checkpoint 2026-10-08: restricted nested UV

Actual Rhino5 API passes all six owning nested UV controls. Complete decoded
native child trees, UUID, dependencies and CRC remain identical after Rhino
write; actual FreeCAD reimport passes 25 checks and preserves exact archive bytes
through source-deleted FCStd reopen. The selected V5 writer now admits only the
verified Line / non-rational axis-aligned bilinear 2D unit-domain UV profile
inside outer domains. Six outer surface controls retain 17 independent interior
and end samples; optional C3, rational/sheared/higher-order and deeper maps remain
guarded. This supersedes the preceding nested-pending checkpoint for this scope.

Native profile tests pass 143 checks, including 30 native-valid unsupported
property controls and 30 direct atomic writer refusals. Actual nested FreeCAD
lifecycle passes 60 checks: placement, stable independent copies, deletion,
Undo/Redo and V5 export after source-deleted FCStd reopening. That export exposed
and reproduced unstable top-level-copy UUIDs. The writer now uses the existing
namespace/source/host UUID-v5 contract; repeated/mixed selection identity is
stable and collisions with geometry/layer components refuse atomically.

Fresh final regression: 42/42 native suites; 27 serial isolated FreeCAD reports
with 2,131 passing checks, including both original rings (347 + 547 = 894).
Ring bounds remain 0.001 mm and area/volume limits are unchanged. Coverage 6,
GUI measurement replay 2 and evidence binding 3 Python tests pass. Current
source/module hashes and installed Python policy agree with captured proof.
Counts describe tests, not a percentage of openNURBS completion.

Twelve unchanged/moved OM9 writer files, independent oracle and exact GUI probe
are frozen in `rhino5-nested-writer-20261008/`. Actual Rhino5 GUI Open/SaveAs,
decoded reimport of these outputs, global reference mapping/remap, general shell
applicability, packages 4–7 and final review/integration remain pending.
GUI API metadata preflight checks names/arity only. The first File3dm-only
checker misclassified document RhinoObjects; its failed report is retained
alongside the scoped GUI check and two negative controls. No actual Rhino GUI
execution is inferred from that check. Primary source is not integrated or pushed.

See [profile evidence and limitations](2026-10-08-opennurbs-nested-uv-profile.md)
and `opennurbs-current-proofs-20261007/summary.json`.

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

## Expanded nested V5 admission — 2026-10-08

Actual Rhino5 API90/90 and complete decoded native/FreeCAD reimport361 passed.
Old selected V5 profile rejection observed RED; bounded positive-weight single-
span2x2 / nonrational4x4 UV family now passes native359 and public FreeCAD900
lifecycle checks.48 unsupported controls plus48 direct protected-target refusals
remain. Fresh native42/42, rebuilt/installed module and serial30 reports/3,465
checks include both original rings894 at unchanged0.001mm bounds. Source/module
hashes are bound to current proof; preceding29/2925 checkpoint is archived.

This supersedes the earlier90-case public-V5 guard-pending scope only for the
family above. GUI Open/SaveAs of180 unchanged/moved writer outputs remains
pending; no global mapping/editor/full support claim. Primary not integrated or
pushed. [Evidence](2026-10-08-opennurbs-expanded-v5-admission.md).
