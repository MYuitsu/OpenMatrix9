# Gumball integration into main — 2026-10-12

The user selected integration into `main`, local verification, then publication
on GitHub Releases. This extends the earlier candidate-only delivery. CI remains
build/package only. Version remains `0.3.4-202610120606` for completion of the
same Gumball release task; no further spec completion or major increment occurs.

Inputs are candidate `a28c4473321dfc5e4746b9e1c69295c4cb428e8d` and main
`27af1e1351ad12f855affbd21f81bba3e8a1569f`. Their common ancestor is
`f5dc335e330c2ba07cb8dc724b13f4109bb8e956`.

Main's release freeze `89008e0` replaced the earlier CAD development tree with
the accepted layer-session runtime. Its missing Cage/History/Units/CPlane/Curve
modules do not establish intended removal from the current Gumball scope.
Integration must retain the candidate CAD modules and main's genuine Layer,
Phase2/3, stock-host and launcher work. The current Layers panel contract and
Gumball guards remain the acceptance criteria; an older release UI cannot
silently replace them.

The candidate's 43 progress entries remain intact. Main's 47-entry ledger is
preserved byte-for-byte as [historical evidence](2026-10-12-main-pre-gumball-progress.json).
Historical release receipts, stock-host measurements and the openNURBS roadmap
remain separate evidence; they do not certify the merged runtime.

Release preparation checks on this integration checkout pass: 8 packaging,
4 build-record, 3 timestamp-version, 11 public-source-audit regression tests;
27 launcher, 29 updater, 2 portable-host and 17 isolated-renderer checks.
The source audit reports 511 icon bindings, 510 SVG files and no errors.

The completed merged-source local validation uses one official FreeCAD 1.1.4
host (`7b38c5d5aa2c74f830539c1c5c44cbf6e3e8e157292c718f9ee24ac6d3e12cf6`) and one rebuilt plugin
(`e2a65fa874b294ad3e873188d63d1ac6513f7a02b1d4397f6b4fbd49ceb61717`). The standalone plugin target reused the prepared stock SDK;
no FreeCAD root build or complete host copy was performed.

| Local cohort | Processes | Passing checks |
| --- | ---: | ---: |
| gumball | 30 | 955 |
| integration | 5 | 160 |
| preservation | 11 | 449 |

Rust: **846 passing tests** in 121 result groups, locked manifest, debug profile.
Native totals: **46 processes / 1564 checks**.
[Compact evidence and hashes](2026-10-12-gumball-main-local-evidence.json).

Integration preserves the candidate's legacy sidebar and genuine main
canonical Layer/clipboard and Phase2/3 APIs. Exact native factory witnesses
exclude legacy groups/settings from model inventory without trusting markers on
geometry. Explicit Selectable intent persists independently from visibility and
lock. Bounded Rust refresh only updates verified legacy RGB/own restrictions;
canonical rows, palette, IDs and paths remain unchanged. Actual nondefault
Ellipse MarkFoci PointColor now matches snapshot/export metadata and FCStd.
Three old-DLL RGB metadata failures were reproduced before the native fix.

Native HistorySettings also has no geometry view provider. Its exact final
native class, four static Bool members and absence of dynamic payload establish
metadata identity. Canonical inventory and collection exclude that helper;
geometry with policy-name fields remains a model and augmented native settings
reject preflight. Actual Circle History, legacy group membership and FCStd
coexistence pass. Phase3 mutation guards remain active.

The actual Surface and Boolean foreign-transaction regressions were reproduced
on the initial merge. Both restore main's pre-open editable/ownership guard;
commit and abort are conditional on retaining the same transaction. Surface
and BRep algorithms, source guards and History behavior are unchanged.

The Ctrl+Shift Mesh pointer regression was a fixture observer-order defect:
focus reinstalls OM9's consuming Qt filter ahead of a previously installed
observer. Focus now settles before installing the observer. All original
assertions and geometry/pixel bounds remain unchanged; the same-DLL diagnostic
passed all 85 pointer checks across the four native viewports.

Rebuild sampling, construction, deviation, preview and commit use native OCCT
without Python Part constructors. Existing deferred guards remain: the expanded
223-check fixture proves immediate target/nested Link revision propagation,
including edit-restore before recompute. The proposed extra Link guard was
withdrawn after actual tests and host signal tracing; no speculative production
patch remains. Polycurve keeps segmentation and periodic exchange preserves
native basis/world placement; comparison is sampled at the recorded tolerance,
not a proof over every parameter.

Selected FCStd fixtures: unchanged metadata/Undo/Unicode/geometry assertions pass with a fixture
setup workaround. Stock Windows ZipWriter checks ambient CRT errno even when
the stream did not fail. Original Notes save reproduced errno13; its archive
had valid CRC/XML. A plain stock document with no OM9 object reproduces the
same false error; clearing only preexisting errno permits save/reopen. The
fixture records and clears that value before save, continues to propagate real
save failures, and retains every original oracle. No production host save fix
or clean empty-document stock-save behavior is claimed. Untouched red and
controlled five-check host reproduction remain in private local evidence.

Stock native transactions use one application-wide transaction ID: opening a
transaction elsewhere commits the existing foreign draft. A fresh ordinary
PartWorkbench baseline with OM9 unloaded proves pending=True to False before
new geometry, retains the draft on abort and restores it on Undo. The original
dev multi-pending fixture is preserved; the new stock creation/copy/Part-UI
fixture explicitly commits the foreign draft first and verifies independent
committed state and Undo. This is the already documented single-active-document
stock scope, not a new multi-document transaction capability.

The candidate's 43 records are preserved, with this one integration entry added.
Main's 47 records remain byte-exact historical evidence. These selected checks
do not certify every historical item, the full P0/core catalog, original
Rhino/Matrix parity, Clayoo/SubD or SmartTargets.

Publication uses the matching version tag on main. CI only builds, records
source/binary SHA256 and packages: `validation_kind: build_only`,
`tests_executed: false`. Local DLL hashes and CI hashes have separate authority.
The exact tagged Action and public asset digests are inspected after publication.
Historical candidate [Action 38200167049](https://github.com/MYuitsu/OpenMatrix9/actions/runs/38200167049)
does not certify this merged source. Its obsolete draft 410144789 is reconciled
before tagged publication; original candidate files remain backed up locally.
