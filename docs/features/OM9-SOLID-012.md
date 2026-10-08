# OM9-SOLID-012 — Box

Command `Box`, catalog/menu ID `OM9_SolidBoxCornertoCornerHeight`.
Rust owns input state and placement/dimension normalization; the native host
constructs and verifies one closed `Part::Feature` solid through Part/OpenCascade.

## Source evidence

Matrix 8 Book 1, printed pp.222–223 / PDF pp.232–233, was extracted and
visually inspected on 2026-10-06. Box continues onto p.223 and ends immediately
before Sphere. Source describes two base corners or a typed length, Enter using
length for width and width for height, and the 3Point workflow. Diagonal/Cube,
Vertical and Center are documented but outside this implemented slice.

Original source material remains in the owner's private reference archive.
Public implementation evidence and remaining requirements are summarized in the
[validation report](../validation/2026-10-06-solid-box-sphere.md).

## OpenMatrix9 decisions

Millimetres, finite coordinates bounded by 1e9 mm, nonzero dimensions of at least
1e-7 mm, document-root ownership and standalone snapshot geometry. Negative
extents shift the origin while preserving a right-handed coordinate frame.
Mouse height uses the closest point between the ray and the frozen base normal;
parallel rays require numeric input or a side/perspective view. Native geometry
uses one transaction and the authored Solid purple color.

Rust tests: `rust/tests/solid_session.rs`, `rust/tests/solid_commands.rs`.
Native tests: `tests/solid_commands_smoke.FCMacro`.

Full Matrix/Rhino compatibility remains unverified. Builder, associative
History and active-layer integration are not claimed. The selected command
does not take Boolean/Loft/Trim inputs; unrelated domain baseline requirements
are not applicable.
