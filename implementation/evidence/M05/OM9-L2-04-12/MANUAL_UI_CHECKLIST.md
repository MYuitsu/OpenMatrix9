# M05-UI manual review — OM9-L2-04-12

- **Gate:** `M05-UI`
- **Status:** **OPEN — no reviewer signature**
- **Owner:** `om9-app` pavé windows + `om9-core::gem_pave`
- **Date prepared:** 2026-10-28

This checklist is the human step the automated evidence cannot perform. Run it against the
standalone application built from this checkout; record the build hash and the reviewer in the
signature block. A skipped step is a FAIL for the gate, not a silent pass.

## Preparation

1. Build the app with `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`.
2. Start the app with an empty `Untitled` document.
3. Create a plane/surface and a few gems (or load a fixture) so the selection cases below exist.
4. Keep `OM9_PAVE_STYLE_LIBRARY` unset for the session-only style case, then set it to a
   writable JSON path for the persistence case.

## Auto Pavé

1. Select one surface and run `BuilderPaveDialog` / `gvPaveDialog`.
2. Change Pattern, Spacing, Angle, Flip, Taper and Equalize; the preview must update without
   writing the document.
3. Select one open curve together with the surface, reopen Auto Pavé; the curve must appear as
   the attractor and the preview must densify near it.
4. Select one closed curve (rectangle/circle) together with the surface, reopen; the preview
   must stay inside the curve.
5. Apply; Undo/Redo must move the whole layout and the `om9.gem.pave` history record together.
6. Cancel a second run; no stone and no feature may remain.

## Manual Pavé

1. Select the surface and run `BuilderPaveBuilder` / `gvPaveBuilder`.
2. Type `Count=12` and Apply; exactly twelve stones must be inserted.
3. Reopen the surface's Manual builder; the stored count must be shown and the preview must
   keep twelve stones.
4. Apply a saved style that came from Manual; the count and parameters must return.

## Pavé Azure

1. Select a surface and at least one placed gem; run `BuilderPaveAzureBuilder`.
2. Toggle Hexagonal, change Angle/Depth/Size/Through/Seat/MinSpacing; four closed profile
   curves per stone must preview and Apply in one step.
3. Cancel; no curve may be inserted.
4. Confirm that no metal cutting solid is expected here (M06/M07 owns the solid).

## Pavé Prong

1. Select touching gems; run `BuilderPaveProngBuilder`.
2. Change Fillet/Height/Drop/Taper/Scale/Shared/Unshared/Center min/max/Touch; markers must
   move without a document write.
3. Apply; every marker is one closed curve in the same transaction.
4. Try a selection without a gem; the command must refuse visibly.

## Pavé Sphere

1. With no surface selected, run `BuilderPaveSphere`.
2. Change Count, X/T, Removal, Min size, Relax amount/strength; the preview must update.
3. Drag the **Radius** viewport handle and the **Count** viewport handle; the window values and
   the preview must move by the same field (no divergence between drag and typed value).
4. Apply; Undo must remove all sphere stones.

## Gem Springs

1. Select a surface and at least two overlapping gems; run `BuilderGemSprings`.
2. Change Spacing/MinSpacing/Iterations/Strength/Resize; stones must separate while staying on
   the surface.
3. Apply and Undo; the previous positions must return exactly.

## Region / Preview / History

1. In any pavé builder change Density (spacing), Border and Gap (min spacing); the preview must
   reflect each change and never mutate the document before Apply.
2. Toggle Stones/Framework preview; the framework must remain a preview only.
3. Apply a layout, edit the host surface, reopen Auto/Manual and press `Refresh`; child `GemId`s
   must be preserved, changed UVs updated, and an out-of-domain station must be reported as a
   conflict instead of silently dropped.
4. Delete the attractor/edge source curve and press `Refresh` again; the refresh must fail with
   a visible stale-source error and must not lose the document.

## Result

- Build / binary hash: ______________________________
- Date: ______________________________
- Reviewer: ______________________________
- Result: **PASS / FAIL** (circle one)
- Defects / notes:

```
```

The gate may be changed from OPEN only by this signed human review. Automated tests are not a
substitute for this page.
