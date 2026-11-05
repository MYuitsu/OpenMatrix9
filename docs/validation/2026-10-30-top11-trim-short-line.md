# Top 11 mutual Trim: short open line

An open line of length 0.0005 mm, shorter than the template document tolerance, meets a second line at its endpoint. Mutual Trim now identifies it as an open line instead of treating coincident-within-tolerance endpoints as a closed seam. Picking the short line removes it, and completing the command then undoing restores it. The closed-circle seam regression remains in the suite.

Validation: `cargo test -j 2 -p om9-app --test top11 --test top11_options --test top11_brep` passed 80 tests with `CARGO_TARGET_DIR=L:\om9-target`.

The no-default-features Top 11 and options suites passed 65 tests. `tools/public_source_audit.py` returned zero errors. The artifact pruning tool removed no runs and retained older cited evidence.

This is a narrow topology correction. Closure of other curve types near tolerance, overlaps and tangencies, surface/BRep targets, and real desktop acceptance remain unresolved. The existing kernel `split_face` handles a single planar face crossed by a straight line, so it does not by itself satisfy general exact surface trimming or identify the selected face region.
