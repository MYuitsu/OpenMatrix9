# Torus final tube-radius wire preview — 2026-10-29

At the tube-radius stage, OM9 now shows two 64-segment inner/outer rings and four 32-segment section circles. Center, 2Point, 3Point, 3Point Radius and AroundCurve use the same Rust center/ring/tube calculation for preview and final BRep. A zero or excessive tube radius suppresses preview, and a stale AroundCurve source suppresses it. Preview is temporary and creates no transaction.

Windows `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: nine focused Torus controller tests passed. New fixtures check 256 preview edges against an independent torus equation for Center, 2Point, 3Point and the Radius branch, then compare committed volume; AroundCurve preview disappears after source deletion. Four egui viewport workflows passed, including a Center preview availability check and Undo. This is a bounded wire approximation; early input stages, actual GPU/desktop display, original Matrix preview and general numerical limits remain unverified.
