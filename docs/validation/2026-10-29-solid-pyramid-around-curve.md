# Pyramid AroundCurve — 2026-10-29

The extracted Pyramid page interleaves two columns and contains neighboring Tube text. Its AroundCurve and direction text also appears under Tube and mentions a base circle; the extraction does **not** establish that these are Pyramid options. The recovered Matrix menu binding invokes the native `_Pyramid` command but does not establish its options. AroundCurve here is an independently tested OM9 interpretation, not verified Matrix behavior. OM9 selects one curve and accepts a picked point or `OnCurve=0..1`; it uses the transformed tangent as the base normal. Inscribed, Circumscribed, Star, and Solid=No share that frame. A deleted or changed source hides preview and blocks commit; undo at center returns to the center stage. Edge combined with AroundCurve is explicitly unsupported, and arbitrary non-similarity curve placement does not support point picking (typed parameter remains available).

Standalone Rust with OCCT, `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`:

- `cargo test -p om9-app --test solid solid_pyramid_`: 13 passed, including translated line, circle tangent, typed/picked center, star open shell, invalid parameter, undo, mode switch and stale source.
- `cargo test -p om9-app --test command_options_interface pyramid_`: 8 passed, including keyboard, pointer and typed AroundCurve routes.
- `cargo test -p om9-app --test solid_difference_ui viewport_pyramid_`: 6 passed, including simulated curve and center picking.
- `cargo test -p om9-app --lib solid_input_regressions`: 56 passed.

Original Matrix interaction and defaults, direction constraint options, Edge combined with AroundCurve, and real desktop/GPU validation remain open. This is partial evidence, not completion of Pyramid or Solid.
