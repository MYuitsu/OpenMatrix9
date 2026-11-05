# Top11 Join: trimmed NURBS with unclamped backing — 2026-10-31

Status: partial automated evidence, not complete Top11 or M03-UI acceptance.

Nonperiodic NURBS may have unclamped support knots. Exact knot insertion at an interior active boundary creates a clamped endpoint; Join needs this only at the endpoint being moved. The opposite support endpoint may remain unclamped. A requested seam at an unclamped support boundary is still explicitly refused. As before, the midpoint adjustment changes geometry near the seam; interval restriction itself does not refit the curve.

Three unit tests cover clamped and unclamped rational multispan backing, orientations, both interior trim boundaries and one interior boundary. The newly included cases compare 101 points per interval with error under 1e-10 and verify refusal of the untrimmed unclamped boundary. The app fixture covers three backing/trim configurations, two orientations and both CMD and toolbar (12 combinations), tests midpoint, sampled displacement <= 0.000401 for gap 0.0008, archive roundtrip and Undo/Redo. These fixtures are not proof for all NURBS or precision scales.

Windows x64, CARGO_TARGET_DIR=L:\om9-target, -j 2: three core unit tests passed, 48 core integration tests passed, 164 app Top11/options/BRep/snapshot tests passed, 93 no-default Top11/options tests passed (existing unrelated warnings).

Unclamped support endpoint adjustment, periodic NURBS, discontinuous trim boundary, extreme precision, aggregate memory, general geometry, broader 3DM interoperability and physical M03-UI acceptance remain open. No native or openNURBS measurement was performed.

Source SHA-256 at validation: curve_ops.rs `3bbcc7ee93681b584d4bae1c3d1681eff0d74b166cce6c35cd2f69ad611c5e6e`; top11.rs `3baa79024f4170a34bb698ccd0bcb09d9af8e9efa3f6b5f5cc2fb05dc56cfde7`; app tests/top11.rs `3fda571f90ee2139295f8aa537e2d3477ec38fa35f02a18094a92b2892ef74de`.

Update: the support-boundary refusal is superseded for the nonperiodic cases in [support-boundary evidence](2026-10-31-top11-join-support-boundary.md).
