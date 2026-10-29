# Box menu variants and recovered macros — 2026-10-29

The recovered Matrix command crosswalk identifies four Box variant IDs and their `_Box` macros. OM9 now exposes authored icons for Diagonal, 3 Points, Vertical Base and Center, dispatching each to the existing Box session. `_Box _Diagonal`, `_Box _3Point`, `_Box _P`, `_Box _Vertical` and `_Box _Center` start the same respective modes. Mode dispatch checks target availability and refuses to replace another active command.

Standalone Rust with OCCT, `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`:

- `cargo test -p om9-app --test solid solid_box_`: 24 passed, including command/variant geometry and undo on world-XY examples and refusal to replace a running command.
- `cargo test -p om9-app --test chrome_actions`: 27 passed, including simulated clicks on all four icons and other Solid icon routes after pagination changed.
- Solid icon audit: 37 authored SVG entries, no errors or shared geometry; 24/32/64px dark review sheet visually inspected. Public-source audit: no errors.

This proves binding and the stated examples only. Rotated construction planes, full signed-direction/preview behavior, original Matrix implementation and real desktop/GPU interaction remain open. Do not treat the Box command as complete on this evidence.
