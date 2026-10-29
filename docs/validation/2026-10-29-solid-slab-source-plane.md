# Slab development source plane and InCPlane — 2026-10-29

For a preselected or subsequently selected open line/polyline, the hidden Slab session derives a planar normal from the placed source. Collinear input uses the captured construction-plane normal projected perpendicular to the source; an ambiguous or nonplanar source is refused. `InCPlane=Yes` uses the captured CPlane normal only when every source segment is compatible; incompatible cases are rejected rather than projected. `InCPlane=No` restores the source-derived normal. Offset pointer rays intersect the selected source plane; height input and preview use its normal. These are partial OM9 choices, not proof of original Matrix behavior.

Standalone OCCT, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -j 2 -p om9-app --test solid solid_slab_`: 9 passed, including a YZ-plane L-shaped source, measured BRep volume and Undo, invalid InCPlane refusal and valid XY toggle.
- `cargo test -j 2 -p om9-app --test solid_difference_ui viewport_slab_development_`: 6 passed, including simulated offset on the YZ source plane and height picked in a second view.
- `cargo check -j 2 -p om9-app --no-default-features`: passed with 12 warnings.
- `python tools/public_source_audit.py`: 14738 source files, 594 icon bindings, 593 SVGs, no errors.

The Slab user command remains disabled. This does not support projection of an incompatible source into the CPlane, arbitrary smooth/closed curves, changing CPlane mid-session, all orientation or ambiguity cases, desktop/GPU or original Matrix behavior.
