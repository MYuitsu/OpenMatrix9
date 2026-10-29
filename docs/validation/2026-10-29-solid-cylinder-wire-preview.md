# Cylinder wire preview — 2026-10-29

At the height prompt, Cylinder now previews two 64-segment rings joined by four generators, using the captured base frame and the same radius and signed height as the committed cylinder. At zero height only the base ring remains. Preview does not change the document; the existing source-stamp checks suppress it if a referenced curve is no longer valid.

On Windows with `CARGO_TARGET_DIR=L:/om9-target` and `-j 2`:

- `cargo test -p om9-app --test solid solid_cylinder_`: 9 passed, including positive-height Center and negative-height 2Point previews, closed volume, open topology and atomic Undo.
- `cargo test -p om9-app --test solid_difference_ui viewport_cylinder_`: 5 passed. The simulated viewport Center workflow also checks the preview edge count and unchanged document before committing.
- `cargo test -p om9-app --test solid_workflows truncated_cone_`: 25 passed, covering the shared cone path.

These are controller and simulated UI checks, not evidence of GPU rendering or original Matrix behavior. The complete Cylinder command and the broader Solid inventory remain open.
