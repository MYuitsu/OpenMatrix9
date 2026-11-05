# Top11 Join: unclamped NURBS support boundary — 2026-10-31

Status: partial automated evidence, not full Top11 or M03-UI acceptance.

For nonperiodic NURBS, a Join endpoint at an unclamped support boundary is now made clamped using homogeneous knot insertion and cropping of inactive control data. The edited boundary pole can then move to the midpoint. Restriction preserves the active curve before the intentional seam adjustment; it is not a refit. The opposite boundary need not be clamped. Ordinary interior knot insertion retains its strict parameter checks.

Four unit tests cover clamped and unclamped support, one- and two-sided trims, degrees two and three, several boundary knot multiplicities, reversed records and both endpoints. The support-boundary fixture compares 201 sample points per orientation and endpoint, each coordinate within 1e-10 before adjustment. The app Join fixture covers four backing/trim configurations, both orientations and CMD/toolbar (16 combinations), checking midpoint, 101 sample positions, .om9 roundtrip and Undo/Redo. These fixtures do not prove arbitrary numerical scales or all geometric cases.

Windows x64, CARGO_TARGET_DIR=L:\om9-target, -j 2: four core unit tests, 48 core curve_ops/top11 integration tests, 164 app Top11/options/BRep/snapshot tests and 93 no-default Top11/options tests passed. The no-default build reports existing unrelated warnings.

Periodic NURBS, discontinuous trim boundaries, extreme precision, aggregate memory, wider geometry, 3DM interoperability and physical M03-UI acceptance remain open. No native or openNURBS measurement was performed.

Source SHA-256: curve_build.rs `3c95f9714a6553387e07f8c8f43ed532a1468274441a2f6b5ddf161d299b26e3`; curve_ops.rs `9804c92282075f5f9d75c63598d46aed774dd30f19808776687dd9bd63735a98`; app tests/top11.rs `92a0df86741d039f9b6d0a91a1457732438b521c42fd2dd4d3ff3cbf23d855aa`.

Update: [periodic evidence](2026-10-31-top11-join-periodic.md) covers certified repeated-basis periodic curves; other periodic forms remain open.
