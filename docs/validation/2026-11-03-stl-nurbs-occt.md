# NURBS surface STL conversion

Follow-up: angular export control is now wired into this conversion; see [STL angular profile](2026-11-03-stl-angular-profile.md). The limitations below reflect the earlier delivery.

P3a partial: STL export converts a validated NURBS surface to an OCCT face with the existing `surface_to_face` adapter, then applies the requested placement-scaled deflection and exports the face tessellation. The old sampled 8–128 grid has been removed. Surface reversal and mirrored placement are handled independently. Without OCCT the export reports unsupported rather than silently using the former sampled grid. The user-authored document is not changed.

Windows, default features including OCCT/openNURBS, `CARGO_TARGET_DIR=L:/om9-target`, Cargo `-j 2`: the 20-test `stl_export` suite passed, including planar open boundaries, a curved NURBS surface with coarse/fine requests, reversed and mirrored surface orientation, and existing BRep and STL regressions. The curved test was rerun with an additional check comparing each encoded triangle center to the analytic surface vertically at a 0.001 mm request; it passed. This is sampled evidence for one surface, not a global error or coverage certificate. No native desktop interaction or release performance was measured.

The native conversion uses OCCT's face construction and its existing construction tolerance; periodic and trimmed fixtures, nonuniformly scaled NURBS, seams, thin details and independent surface coverage need additional verification. Angular export control is not yet wired through this or retained 3DM BReps; capture/background export, managed memory, GPU budget, checkpoints and remaining plan gates are open. The existing OM9 session was left intact.
