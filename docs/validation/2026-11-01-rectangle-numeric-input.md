# Rectangle numeric input — 2026-11-01

The supplied Matrix9 screenshot shows `Rectangle`, first corner `20,20`, then
`10` for length and `10` for width; its prompt also offers Enter to use length
as width. The supplied OM9 screenshot shows the first numeric value rejected.

Standalone Rust Corners mode now accepts a positive finite bare length after the
first corner, followed by a positive finite bare width or Enter for equal sides.
The two dimensions commit once through the rectangle owner. Existing corner and
explicit-option paths remain in place. Other modes are outside this change.

On Windows, source base `89ef2676` plus the working-tree change, with default
features and `CARGO_TARGET_DIR=L:/om9-target`, `cargo test -p om9-app --lib
rectangle -j 2` passed 3 tests. The new tests cover actual CMD coordinates,
menu dispatch, non-origin bounds, numeric width, Enter default, and invalid
width retry. Before the fix the numeric regression failed as expected. Public
source audit reported zero errors.

The running `om9.exe` was locked by Windows; `cargo test --test rect` could not
replace it (os error 5). No new executable or actual desktop interaction was
verified. The existing running application therefore does not contain this fix.
