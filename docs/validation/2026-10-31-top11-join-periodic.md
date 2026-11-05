# Top11 Join: certified periodic NURBS — 2026-10-31

Status: partial automated evidence; not complete Top11 or M03-UI acceptance.

When Join needs to move a seam endpoint on a full-domain periodic NURBS with a certified repeated-pole/weight and repeated-knot basis, it materializes a nonperiodic equivalent using homogeneous knot insertion, then moves the requested seam endpoint to the common midpoint. Opening one end intentionally changes the formerly closed segment near that end; the other endpoint remains in place. The periodic flag is removed because the adjusted segment is no longer periodic. Uncertified periodic representations are refused when endpoint adjustment is required. Coincident endpoints requiring no adjustment retain the existing path.

The core fixture exercises one rational quadratic periodic basis, both seam ends and both orientations, comparing 201 sample positions before adjustment within 1e-10; a modified, uncertified basis is refused. The app fixture joins the closed curve to a near-seam line, exercising both orientations and CMD/toolbar, checks midpoint and sampled displacement <= 0.000401 for a 0.0008 gap, .om9 roundtrip and Undo/Redo. This evidence does not prove arbitrary periodic representations or continuity beyond the sampled cases.

Windows x64, CARGO_TARGET_DIR=L:\om9-target, -j 2: five core restriction unit tests, 48 core curve_ops/top11 integration tests, 165 app Top11/options/BRep/snapshot tests, and 94 no-default Top11/options tests passed. The no-default build reports existing unrelated warnings.

Uncertified/trimmed periodic representations, discontinuous boundaries, extreme precision, aggregate memory, broader 3DM interoperability and physical M03-UI acceptance remain open. No native or openNURBS measurement was performed.

Source SHA-256 at validation: curve_ops.rs `dfe834c29229e9e235dfe391c4a620455caedede17a7c981e9ffe7bb1defdc26`; app tests/top11.rs `3ad7500ccb90dadc37361064d435a6c75cdddec78925cfcc192ab3f3314dfa47`.

Update: [trimmed periodic evidence](2026-10-31-top11-join-trimmed-periodic.md) covers certified repeated-basis curves with a trimmed active interval.
