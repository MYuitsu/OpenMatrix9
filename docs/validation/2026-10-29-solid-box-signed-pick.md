# Box signed height on world XY — 2026-10-29

The approved Box contract requires signed height and preview/commit agreement. Previously a point below the base was converted to a positive height; Center dimensions preview also ignored the sign, while default and Center corner preview did not use the committed second base point. OM9 now keeps signed point heights and uses the chosen world-XY base for height preview. Center numeric Length/Width accepts negative Height. Invalid zero height retains the session and document; commit errors also restore the pending point-based session.

Standalone Rust with OCCT, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -p om9-app --test solid solid_box_`: 26 passed, including signed preview and commit for default and Center corner/dimensions, invalid retry and undo.
- `cargo test -p om9-app --test solid_difference_ui viewport_box_negative_height_stays_below_base`: passed for simulated viewport pick and undo.

This covers the world-XY slice. Rotated CPlane, picked height and preview of Vertical/3Point, full document-unit behavior and actual desktop/GPU validation remain open; no claim of full Box compliance is made.
