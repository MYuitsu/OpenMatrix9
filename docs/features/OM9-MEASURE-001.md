# OM9-MEASURE-001 — Angle

Status: in progress. Rust and native four-point menu/mouse/CMD measurement verified; TwoObjects and required O-Snaps remain incomplete.

Local source requires four prompts: Start of first line, End of first line, Start of second line, End of second line. Result appears in feedback window. Source explicitly requires O-Snaps to existing geometry. TwoObjects is an unnormalized option cue and must remain tracked.

OM9 geometry choice: angle of directed world-space line vectors in degrees, 0..180, using normalized vectors and atan2(cross magnitude,dot) for accuracy near parallel directions. The lines may have different origins and lengths. Coordinates must be finite within ±1e9 mm; lines of length ≤1e-12 mm are rejected. These numeric/default choices are documented as OM9 decisions, not recovered Matrix defaults.

ABI om9_measure_angle accepts 12 readable doubles (first line start/end then second line start/end), returns NaN on null pointer or invalid/degenerate input. Caller owns array. Four Rust tests cover geometry, invalid inputs, small angles and ABI layout. Stub failed two semantic tests before implementation. Current native report requirement, Undo/model invariance and snap workflow remain unverified.

Plan: docs/superpowers/plans/2026-10-05-core-angle.md. Source: local specs/01-core/om9-measure-001-angle.md. No PDF reread. End Snap source read: includes curve endpoints, polyline interior vertices, closed seams and surface/polysurface corners; limiting it to open edges would not satisfy OM9-SNAP-002.
Angle OM9-MEASURE-001 four-point directed 3D line math and ABI added. Four Rust tests passed; Clippy all-targets -D warnings passed, session55904 exit zero. Normalized atan2 preserves tiny angles and scale invariance; rejects invalid coordinates/zero-length lines. Native Angle, TwoObjects and explicitly required O-Snaps remain outstanding. Distance feature front status corrected to reflect native verification.

Angle native original command mapping AnalyzeAngle -> Angle and four-point session integrated in existing read-only measurement controller. Rust release and native controller/Workbench/CurveController build/link succeeded. 14 native checks and exit zero: build/angle-isolated-1/ed250692c87e4039b1d02a75d5887a4e/results.json. Four exact prompts, spatial distinct-origin 90-degree result, menu/CMD sharing, first/second degenerate endpoint retries, Esc, four mouse points, no model/Undo and actual report history pass. TwoObjects reports unimplemented; O-Snaps not present. Distance regression passed all 24 checks with exit zero: build/distance-isolated-1/dd3ce1e800464aa2931fc166f795acab/results.json. Keep Angle in progress.
