# Pyramid Solid=No — 2026-10-29

The extracted Pyramid page interleaves two columns and contains neighboring Tube text. The source ownership of its Solid description has not been verified; the recovered Matrix menu wrapper delegates to `_Pyramid` and does not establish this option. OM9 exposes Solid as an independently tested interpretation, initially Yes; No removes the base and validates that the result is an open shell with the expected lateral faces and boundary edges before committing. The option applies to center, circumscribed, edge and star construction. Changing it creates no history; a direct three-number shortcut is rejected when No is active because that shortcut creates a closed primitive.

Validation on standalone Rust with OCCT, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -p om9-app --test solid solid_pyramid_`: 11 passed, including open center, edge, circumscribed and star, negative height, invalid option, shortcut rejection, and undo.
- `cargo test -p om9-app --test command_options_interface pyramid_`: 7 passed, including Solid through keyboard, pointer and typed options.
- `cargo test -p om9-app --test solid_difference_ui viewport_pyramid_`: 5 passed, including open-shell viewport selection and undo.

The Matrix original's defaults and detailed interaction and a real desktop/GPU session remain unverified. This is partial command evidence, not a completion claim for Pyramid or Solid.
