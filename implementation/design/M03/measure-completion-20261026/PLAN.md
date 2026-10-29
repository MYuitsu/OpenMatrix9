# Measure completion — active work, 2026-10-26

User requests the L2-03-20 functional gaps resolved and explicitly chose implementation of all 13 annotation/dimension commands (not ownership-only planning).

## Work and evidence boundaries

- Extend read-only measurement owner: canonical BRep edge length/area/closed volume, surface isocurve length, pointwise curvature radius, object Distance/Angle, continuous/orthographic point modes, dynamic feedback, grouped ObjectSummary and known/unknown gem carat totals; add app menu access.
- Preserve M03 L2-03-20's five existing L3 and exclusion of drafting. Add a distinct M03 model-space annotation owner (L2-03-21), with one exact L3 for each of the 13 existing OM9-MEASURE IDs. Preserve IDs/icons; no unsupported command becomes enabled before implementation tests.
- Rust owns annotation semantics and state. Versioned typed `om9.annotation` entity extensions plus canonical line/point geometry preserve model identity, layer placement, ordinary transaction/Undo and archive persistence. Native OCCT remains limited to canonical geometry measurement calls.
- Implement seven dimension creators, Leader, Text Block, Edit Dimension, Edit Text, Dimension Options and Recenter. UI derives lines/arrow/text overlays from the same typed persisted annotation. No paper-space/plot layout or claim of full Rhino annotation compatibility.
- Retain RED→GREEN independent numerical/geometry/lifecycle tests; run focused existing measure/UI/rectangle tests, native tests for the added adapter, archive roundtrip and real-window checks. Record incomplete gates rather than promoting from source presence.
- Update exact milestone records, README Limits, traceability and live ledger only with actual evidence. Do not modify reusable skill business rules without the requested post-implementation consent (prior Rectangle approval is unrelated).

This file is a working plan, not completion evidence. Results will live under `docs/validation/2026-10-26-measure-completion/`.

## Steering 2026-10-27

User additionally requests M03 OM9-L2-04-01 corrections: 019 solid grips missing; 025 allegedly lacks inverse surface routines (actual current curve_surface_ops has closest_point_on_surface and pullback_curve, must reconcile); 037 contract UNRESOLVED_BLOCK_ENABLE; ArrayCrv lacks spacing mm/Roadlike; SetPt edits only placement origin; no drag preview. Preserve original Measure+13-annotation implementation objective while addressing these compatible additions. Native measurement adapter files added and cargo check kernel now passes; app Measure/annotation implementation has not yet been written.

## Verified checkpoint 2026-10-27

The earlier app-not-written checkpoint is superseded. Measure, 13 annotations and Transform corrections are implemented with scoped evidence: 169 distinct tests and default build pass. Three Transform commands remain blocked as documented. See docs/validation/2026-10-26-measure-completion/README.md and receipt.json. Actual desktop helper remains inaccessible after unlock; M03-UI is unsigned. Skill proposal is prepared but reusable rules await separate consent.
