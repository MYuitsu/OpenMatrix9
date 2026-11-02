# Box document-unit routes — 2026-10-29

Follow-up validation of the Box unit conversion, without a product behavior change. The measured fixtures use millimeter, centimeter, meter, inch, foot and micrometer documents; each exercises inline dimensions, 3Point numeric width/height and Center numeric length/width/height. A separate inch fixture uses the shared Width option through simulated keyboard, pointer and typed command paths and measures the resulting BRep.

Standalone Rust with OCCT, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -p om9-app --test solid solid_box_numeric_routes_scale_for_all_document_units`: one test, 18 unit/route fixtures, passed.
- `cargo test -p om9-app --test command_options_interface box_width_non_mm_uses_shared_option_routes`: one test, three simulated input routes, passed.

These results do not cover every Box option, rotated viewport input, real desktop/GPU, or the full Solid inventory.
