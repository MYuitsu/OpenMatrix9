# Box document units and retry — 2026-10-29

Box point coordinates already passed through the document-unit parser, but numeric Length/Width/Height and direct three-number Box dimensions were treated as millimeters. These values now convert from the document unit exactly once; Enter defaults reuse already converted values. Default/Diagonal, 3Point, Vertical and Cube reject a single numeric value in place of their second point; Center keeps its approved numeric Length path. If a Height operation fails because the selected base is degenerate, the pending session remains available for Undo and correction.

Standalone Rust with OCCT, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -p om9-app --test solid solid_box_`: 35 passed, including centimeter default, 3Point signed Width, Center dimensions and preview, inline dimensions, Enter default, degenerate base retry and Undo.
- `cargo test -p om9-app --test solid_difference_ui viewport_box_`: 2 world-XY simulated viewport regressions passed.

The unit fixture covers centimeters; other units, all option UI routes with non-mm units, rotated viewport input and real desktop/GPU remain open. No full Box or Solid completion is claimed.
