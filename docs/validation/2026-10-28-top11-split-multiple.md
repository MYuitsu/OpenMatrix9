# Top 11 multi-curve Split — 2026-10-28

Scope: standalone Windows x64, curve targets with cutting curves. The operation computes all affected pieces before committing a single document transaction; unhit targets are preserved. Cutters can hit different targets. Different placements and surface/BRep targets are not covered by this implementation.

Regression: `split_multiple_curves_is_one_undo_and_preserves_unhit_target` first failed with `split/trim need exactly one curve`, then passed. It checks two split lines, an unhit third line, Undo and Redo. `split_multiple_curves_accepts_cutters_that_hit_different_targets` checks two independently hitting cutters and Undo.

With `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`, `cargo test -p om9-app --test top11 --test top11_options --test top11_brep` passed 52 tests (28 + 10 + 14), including the existing Rotate and BRep Join cases. The separately run `cargo test -p om9-app --lib top11` passed 4 tests. `cargo build -p om9-app --bin om9 -j 2`, scoped rustfmt check, and `python tools/public_source_audit.py` passed. Artifact pruning removed no runs because retained runs are cited.

Actual UI exploration used a fresh OM9 window (process 48068), created two target lines and a cutter through CMD, started Split from the Top 11 toolbar without preselection, selected both targets and the cutter in the viewport, and confirmed that the two sides of one resulting line could be selected separately. Undo/Redo through the actual UI is **not verified**: after exploratory Undo attempts, CMD Undo reported `missing Undo entry`; the exact history state needs a clean, instrumented replay. There is no human acceptance sign-off. Automated controller Undo/Redo is separate evidence and does not close this UI gap.

SHA-256 at validation: `om9.exe` 56f3e32ccb1983c4eced294a1043d29136330ef1b33d3ff7c78e5f56a271b051; `crates/om9-app/src/session/top11.rs` e8ae2bbbc5e9c7b5bab5f122ee135aa73549b7fa723304cd499d7680a3cde931; `crates/om9-app/tests/top11.rs` 1b0da7753376cd00c1b9c04a8429f514564e8b017ac8726018c753831d84f44e.

Remaining: clean UI Undo/Redo replay, multi-target Trim, general and closed curve Split Point, different placements, surface/BRep Split/Trim, broader Top 11 edge cases and manual acceptance. Do not treat this record as completion of the Top 11 group.
