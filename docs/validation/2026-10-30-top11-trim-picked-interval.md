# Top 11 Trim: picked interval (2026-10-30)

Target: standalone Rust OM9, Windows; checkout base `89ef2676b597ebc5ab38fa5c6acf0ad47a2f261b` plus working-tree changes. The old implementation compared the pick with interval midpoints. For a line from 0 to 100 cut at 10, a pick at 11 incorrectly retained 10..100. The core regression failed with those endpoints before the fix.

The shared Trim owner now projects the pick onto each cut piece and removes the nearest piece. The existing curve projection is exact for lines and uses bounded sampling and refinement for other curves; global closest-point correctness on pathological curves is not established. A pick exactly on a cutting boundary is ambiguous and currently resolves to the first equal-distance interval.

Validation: `CARGO_TARGET_DIR=L:\om9-target`, `cargo test -j 2 -p om9-core --test top11`: 18 passed. `cargo test -j 2 -p om9-app --test top11`: 29 passed; the added CMD/geometry/Undo regression passed separately. `python tools/public_source_audit.py`: zero errors. The UI pointer route, other curve representations, macOS and manual M03-UI acceptance remain unverified. This does not complete Top 11.
