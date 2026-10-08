# OM9-SOLID-014 — Sphere

Command `Sphere`, catalog/menu ID `OM9_SolidSphereCenterRadius`.
Rust owns center/radius validation; the native host creates and verifies one
closed `Part::Feature` solid with an analytic spherical face.

## Source evidence

Matrix 8 Book 1 printed p.223 / PDF p.233 was extracted and visually inspected
on 2026-10-06. The section ends at the Ellipsoid heading on the same page.
It describes picking the center then entering or picking the radius. The manual
also describes 2Point, 3Point, 3Point with Radius, Tangent, AroundCurve, 4Point
and FitPoints; these extended construction workflows are not implemented.

Public implementation evidence and remaining requirements are summarized in the
[validation report](../validation/2026-10-06-solid-box-sphere.md).
Original source references remain in the owner's private archive.

## OpenMatrix9 decisions

Millimetres; positive radius at least 1e-7 mm. Radius picked from a point is
the Euclidean distance in world coordinates. Center coordinates and the entire
sphere must lie within ±1e9 mm. Output is a native analytic solid, rather than
a triangulated mesh; mesh bounding boxes are not used as evidence of exact
center or radius. Standalone document-root output has feature/command metadata,
one native transaction and the authored Solid purple color.

Shared lifecycle: scene-only unpickable final-step preview, Cancel/Esc cleanup,
Undo input, successful-only command history, Undo/Redo, FCStd persistence,
document switch/close cancellation and workbench cancellation.

Rust tests: `rust/tests/solid_session.rs`, `rust/tests/solid_commands.rs`.
Native tests: `tests/solid_commands_smoke.FCMacro`.

The recorded Mesh mention does not mean this command creates mesh output.
Builder/Styles/associative History and active-layer integration are not claimed.
Full Matrix/Rhino compatibility remains unverified.
