# Restricted nested GUI acceptance and expanded UV controls — 2026-10-08

Actual Rhino5.14.522.8390 GUI Open/SaveAs/reopen passes all twelve OM9 writer
outputs: six unchanged and six moved. There are zero SelBadObjects results;
source/oracle/saved file hashes bind the report to frozen inputs. Seventeen
interior/end samples retain the original 1e-9 mm threshold, maximum deviation
2.282e-14 mm. The command log does not establish absence of modal warnings.

Actual FreeCAD reimport passes 73 checks: complete native UV children, source
UUID, dependency graph and geometry CRC equal before/after GUI SaveAs. External
source deletion and FCStd reopen retain the exact saved bytes and source record.
Rhino adds two opaque plugin tables (16592d58-4a2f-401d-bf5e-3b87741c1b1b and
5dc0192d-73dc-44f5-9141-8e72542e792d). These remain preserved; selected export
refuses unknown closure atomically. No payload is stripped. Exact sources,
SaveAs files, oracle, script, command log, returned JSON and reimport proof are
in `rhino5-nested-writer-20261008/`.

Next controls cover six outer surfaces, five UV parameter classes
(Line/Arc/Nurbs/Polyline/PolyCurve) and three UV maps (sheared bilinear, rational
bilinear and nonrational bicubic), 90 independent fixtures. Native generation
compares seventeen interior/endpoints using explicit raw-curve/UV-surface/model-
surface composition, verifies validity and complete decoded child fields after
native V5 serialization. These fixtures use direct SDK writing solely to test
the target; public OM9 V5 guards remain enabled. No production policy changed.

Actual FreeCAD retention passes 721 checks, including exact source records,
native staging placement with unchanged UV children, separate copy UUIDs,
deleting originals, Undo/Redo, source-deleted FCStd and atomic public V5 refusal.
Actual Rhino5 read/write/reread is pending. Frozen fixtures, independent oracle,
script, metadata preflight and runtime proof are in
`rhino5-expanded-nested-20261008/`. Preflight proves names/arity only.

Ruling: retain the V5 guard for expanded UV maps until actual target evidence —
native serialization cannot establish Rhino5 reader fidelity. If this guard is
too conservative, it delays export; original data remains intact in the project.

Fresh regression after extending the independent generator: 42/42 native,
29 serial FreeCAD reports with 2,925 checks. The preceding 27-report/2,131-check
checkpoint is archived in `opennurbs-checkpoint-20261008-before-expanded-uv/`.
Production C++/Rust/Python and installed module are unchanged this step; source
and binary hashes remain bound. Both rings retain their prior fresh 894 checks
and 0.001 mm bounds/area/volume limits. No need to rebuild unchanged binaries or
rerun unrelated host cases. Coverage/test refresh is recorded in the ledger.

This completes the actual GUI acceptance of the restricted nested profile.
General nested maps and reference copy/remap/global mapping, general shell
applicability, packages4–7 and final integration remain open. SDK remains pinned;
primary checkout is neither integrated nor pushed.

Final refreshed coverage6/6, GUI measurement replay2/2 and evidence binding3/3 tests pass after the new property slices. Expanded ninety-file Rhino probe dispatched; do not run another application while it may be active. Actual target result remains pending.

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
