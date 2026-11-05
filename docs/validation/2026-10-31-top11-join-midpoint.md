# Top11 Join: midpoint continuity — 2026-10-31

Status: PARTIAL_CHECKS_PASSED. The normalized Matrix reference states that joined curve ends are replaced by their midpoint. Previously OM9 accepted ends within the document tolerance but retained the gap; the new line fixture failed before this change.

After ordering in world coordinates, Rust adjusts each tolerated pair to its midpoint before one atomic publication. The tested path covers lines, circular arcs (converted to rational NURBS when adjusted) and clamped nonperiodic NURBS. Nested polycurve endpoint handling is implemented but not independently tested. A near-closing seam is handled too. The operation validates the adjusted curve and checks endpoint residual against one percent of document tolerance. A required adjustment to a periodic or trimmed NURBS is currently refused instead of committing a false seam; a no-adjustment join is not blocked by that restriction. Geometry may change near the seam and an adjusted arc changes representation.

Fixtures check independently placed lines through CMD and toolbar, midpoint and Undo/Redo, a nearly closed loop, rational NURBS with both orientation flags, arc endpoints and preservation of unmodified poles/weights, and atomic refusal for a trimmed NURBS gap. The existing world-Join fixture includes `.om9` round trip. A synthetic egui viewport fixture selects two lines, joins at their midpoint and undoes. Windows x64, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`: core curve_ops/top11 48 passed; app Top11/options/BRep/snapshot 164 passed; no-default app 96 passed; one default-feature viewport fixture passed.

General periodic/trimmed endpoint adjustment, high-magnitude numerical precision, downstream continuity/fairness, dense join performance, 3DM and physical M03-UI acceptance remain open. No openNURBS measurement batch was performed. Full Top11 remains incomplete.

Source SHA-256: `crates/om9-core/src/top11.rs` 58a5e3de406a06fa9f50e83efc52e2865ff3316d4dc719f54ec0a703d52feec5; `crates/om9-app/tests/top11.rs` 31ffa63a0f10435a66b069dbe6b1d81b073557a0195a38b3c023081d93e3317a; `crates/om9-app/src/ui/chrome.rs` 660e7564a97d7383de2ed77dd07f09c3016f283d434ccf64ddc2cc540f0c1028.

Update: the earlier trimmed-NURBS refusal is superseded for the clamped-backing cases in [trimmed NURBS evidence](2026-10-31-top11-join-trimmed-nurbs.md). Other limits remain.
