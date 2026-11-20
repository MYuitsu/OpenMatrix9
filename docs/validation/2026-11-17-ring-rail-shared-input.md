# Ring Rail: direct core, shared add-on entry and VNM mm

Implementation target: `OM9-L3-04-02-001`. Ring Rail remains partial; this record covers sizing, circular geometry, placement, shared publication and recorded-feature editing. Replace Yes/No/Cancel, custom shapes/library, complete region tables and desktop acceptance remain open.

## Source and size policy

US input uses the inside-diameter column of [Stuller's Ring Blank Lengths Chart](https://www.stuller.com/articles/view/determining-ring-blank-lengths-in-b-and-s-gauge/) (published 2013-06-05, checked 2026-11-17). Its half-size values are frozen in `om9_core::ring_size::US_HALF_DIAMETERS_MM`; quarter-size diameters are OM9 interpolation between adjacent published half sizes, rounded to 0.01 mm. Integer hundredths are used for this interpolation so binary floating-point error does not reverse a half-cent tie. This is a named supplier convention; it is not claimed to be a universal US/ISO conversion or manufacturing tolerance.

For comparison, [Tiffany's guide](https://www.tiffany.com/size-guide/) publishes diameter values rounded to 0.1 mm and differs at some sizes; [GIA's chart](https://4cs.gia.edu/en-us/blog/how-to-determine-ring-size-tips-and-ring-size-chart/) also publishes quarter sizes to 0.1 mm. OM9 does not mix rows from those charts with Stuller's table. Only factual measurement values and source attribution are recorded; no supplier artwork, model library or PDF is imported into the repository.

VNM (Việt Nam) is explicit inner diameter in mm, with a separate inner-circumference-mm input choice. It is not an inferred Vietnamese ni-number table. New sessions default to VNM diameter 18 mm. Inch diameter/circumference and mm circumference convert to the same source diameter; geometric circumference is always πD. US accepts sizes 1–15 on the quarter-size grid and refuses unsupported sizes or regions. Switching systems uses the shared inverse conversion; selecting US snaps to the nearest supported quarter size. The existing direct-builder diameter domain (1–40 mm) is shared by new Ring Rail input and preview. The Diameter field now accepts exact finite values within that domain, without the former 0.1 mm step restriction that rejected chart diameters.

## Implementation and publication

`om9-core::ring_size` owns conversion and the canonical circular rail: its plane is world XZ (Front), it starts toward +Z and turns toward +X, with an explicit center in world mm. The direct builder and `RingRailProducer` consume this logic. The Tools add-on adapts the same curve into its ABI reply; the standalone Top11 command is a compatibility wrapper around that session rather than another sizing implementation.

In the Matrix host, both `TopIconRingRail` and `BuilderRingRail` resolve to `BuilderRingRail` and its one add-on panel/settings state. Explicit manifest aliases take priority over inferred catalog names, preventing `gvRingRail` from being overwritten by a Top11 feature name. Without the add-on, both icons dispatch to the existing direct RingRail builder.

The direct builder stores a `rail_frame=front` recipe parameter. Older producer requests without that marker retain their XY orientation on replay. New center fields, preview, source output and diameter handle use the same model values. Both adapters retain the host transaction/Undo flow; preview does not insert entities. Legacy US integer recipes retain their frozen diameter values.

## Validation status

The first coordinated fixture exposed catalog-name alias shadowing; a later run exposed quarter-size rounding at a half-cent tie. Both were corrected. Core ring_tools/builder_session/builder_controls/producers: 37 passed. The consolidated addon_session suite passed 23 with 2 ignored, including seven sizing cases, geometry/Undo parity and shared icon-target/direct-dispatch checks. A later assertion that starts the add-on from both icon targets is prepared but not yet rerun.

The related ring_tools suite passed 9 and failed its dependent native sweep case with a stale published result. The VNM producer was corrected to treat its stored `diameter_mm` as authoritative. Inspection also found that builder inputs used sorted IDs rather than selection order; this was corrected to use the existing ordered selection service. A native attempt predating that order correction was canceled and is not counted as a result. The focused corrected native chain then passed (1 test, 9 filtered, 526.64 s). That duration covers the dependent rail/profile/sweep fixture, not Ring Rail UI latency. This native run predates the subsequent history/F6/unit-publication batch below and is not a current-head certification of every dependent edit path. No desktop/GPU or installed native-DLL evidence is claimed.

The parity fixture checks VNM diameter/circumference, US quarter size, mm diameter/circumference and inch diameter/circumference; it compares canonical geometry and placement across add-on/direct, preview/source equality, no preview Undo, and one Undo/Redo. The final version also checks both menu icons enter the shared owner on each route. Packages run in-process through the real host JSON boundary, not an installed DLL. Build target is the existing `L:/om9-target`, always `-j 2`.

## Host-owned recipe, model units and recorded editing

Official Tools/Matrix Ring Rail add-ons now publish through the direct builder. The host reconstructs the canonical source from typed settings, verifies that the reply is the expected single circle and matches that source, and uses the existing builder transaction to publish the entity, recipe and result. Unexpected replacements/deletions/transforms/records refuse. The add-on session remains available until publication succeeds, and its normal completion supplies the final event. This replaces generic circle-only insertion, so recorded Ring Rail output from either route can be edited after reopening without the add-on installed.

New recipes capture `model_mm_per_unit` alongside `rail_frame`. Source coordinates and radius are converted to document model units while Diameter/Size and center values remain physical mm. `BuilderSession::from_history` restores frame, units and authoritative VNM diameter without letting parameter iteration order change them. Legacy recipes without frame/unit markers retain XY coordinates and their previous model scale. Reset restores recorded parameters and the same output/recipe IDs; its revision advances to reject old UI edits. Locked outputs/recipes use the existing read-only permission checks; those new Ring Rail lock branches have not been separately fixture-tested. An unchanged Apply closes without an Undo entry; a geometry-equal metadata update does not incorrectly increment the source geometry revision.

F6 context now offers Ring Rail with an empty selection, and Edit Ring Rail for a selected recorded Ring Rail. The actions use the existing direct dispatcher/History editor; permission and provider availability remain explicit. This is tested through F6 action snapshots and controller dispatch, not physical F6-key or desktop-pointer automation. Earlier add-on circles without a recipe are not implicitly reclassified. Automatic downstream recompute during F6 Apply is not proved by these fixtures.

Final batch for this section: core ring_tools/builder_session/builder_controls/producers **38 passed**; Tools and Top11 library suites **17 passed**; focused host Ring Rail fixtures **2 passed, 24 filtered**; app f6_context/history_updates/builder_controls/builder_panel **29 passed**. The host fixtures cover seven sizing modes plus four route/document-unit combinations (two tests, not eleven test cases in the runner). They check both icon starts, source/preview/Undo parity, one recorded recipe from both routes, cold `.om9` save/open, F6 edit action/dispatch, stable identity through Reset, edited diameter, Undo/Redo and unchanged Apply. No new large native sweep run was repeated for this metadata/UI batch. Existing target `L:/om9-target`, `-j 2`.

## Shared existing rail and atomic dependent update

The direct and official add-on routes now ask the same Yes/No/Cancel question.
The current-rail pointer is an ordinary persisted resource in the source transaction;
older drawings use an unambiguous recorded rail or explicit selection. Arbitrary
circles are not guessed to be Ring Rails. No keeps the old rail and inserts another;
Cancel leaves the document and Undo stack unchanged. Yes preserves the existing
entity/recipe identities and attributes, applying the proposed world placement.
Permissions, recipe locks, document/authority/epoch, source revision, command run
and frontend parameter revision guard the question. Old question tokens refuse.

When an edited/replaced rail feeds update-enabled History, the existing host History
worker prepares the source, root recipe, current pointer and dependent results in
one private candidate. The host registry rebuilds dependencies in topological order;
native representations travel beside canonical sources into the final preparation.
The displayed Ring Rail parameters are read-only during this operation; Reset/tab changes refuse, the prompt reports the dependent update, and inactive parameter options are hidden. The public
coordinator rechecks source/context and publishes source plus scene once. Cancel,
stale selection/source/context, quota failure or locked/failed dependency discards
the private result. The worker slot remains owned until its thread exits.

This also connects recorded Ring Rail F6 Apply to the worker. It does not certify
all feature producers: a producer without the caller-limited rebuild protocol
refuses atomically. Native sweep construction currently checks cancellation before
and after its OCCT call; that call is not claimed interruptible. Snapshot limits
and reservations are enforced at the existing boundaries, but complete native
temporary allocation/peak-RAM admission is not proved.

Focused batch: **39 passed**: `addon_session ring_rail_` 6 (24 filtered),
`join_history` 6, core `dependent_history_transaction` 3, `producers` 10 and
`builder_session` 14. The new host test covers both routes with two dependent
Outside Rail recipes, successful publication, cancellation, stale selection and a
locked second dependency (eight scenarios inside one runner test). It checks no
publication while pending, read-only parameters, stable IDs, current source stamps
in the scene, one Undo/Redo and source/History archive persistence. The preceding
five Ring Rail tests cover shared confirmation, cold mm/inch reopen, placement,
legacy/unrecorded sources, locks, stale parameters and late options.

Receipts: `build/standalone-validation/ui-artifacts/ring-rail-dependent-20261117/`.
Integration targets are built with `cargo rustc --test ... -j 2` in the existing
target, then run with its debug DLL directory on PATH. This preserves the user's
running executable and unsaved drawing. An attempted all-tests check is currently
red (319 errors in older CanonicalBrep test fixtures); that is not presented as a
green workspace regression. The focused production-library/integration targets
above build and pass.

The native rail/profile/sweep fixture now edits through F6's controller route and
the new worker, checking exactly one sweep revision and one Undo/Redo. It passed:
**1 test, 9 filtered, 389.04 seconds** (wrapper elapsed 389.58 seconds). It checks
that the public document is unchanged immediately after scheduling, the native
sweep source changes once, one Undo/Redo restores the full document and the chain
is clean. A later invalid profile leaves the committed sweep intact. This is the
whole fixture duration, not UI latency or a controlled speedup over the older run.
The executable predates only the subsequent prompt/Reset/tab interaction guards;
those guards are covered by the final six-host-test rerun (8.07 seconds).
Real pointer/F6-key/dialog and installed native add-on package acceptance remain
open. The left menu layout has separate [evidence](2026-11-17-left-main-menu-only.md).

Next: implement custom non-round rail/library
using the confirmed sizing policy and complete the verified-region/UI/package gates.
The complete Ring Rail command and the full 31-command plan remain incomplete.
