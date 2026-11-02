# Sphere AroundCurve ambiguous source edge — 2026-10-29

When multiple BRep edge hits overlap during source selection, the existing ambiguity popup now retains each exact hit and labels its object, EdgeN and ID. Choosing a row revalidates command run, scene epoch, viewport, source and hit before accepting it. This permits different edges of one object and edges of different objects to be identified without collapsing them to root IDs. Canceled or restarted command contexts reject old candidates.

Standalone OCCT, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -j 2 -p om9-app --test solid_difference_ui viewport_sphere_aroundcurve`: 4 passed, including coincident boxes, choosing the second object's edge, center placement, and stale popup after cancel/restart. The choice itself calls the popup selection API after simulated viewport input, rather than clicking its row.
- `python tools/public_source_audit.py`: 14724 source files, 594 icon bindings, 593 SVGs, no errors.

The fixture covers coincident edges of distinct boxes; overlapping edges from the same object, actual popup row clicks, instance paths, desktop/GPU and original Matrix behavior remain unverified.
