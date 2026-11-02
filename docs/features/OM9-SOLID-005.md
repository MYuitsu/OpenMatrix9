# OM9-SOLID-005 — Cap Planar, partial standalone implementation

The normalized M03 reference describes selecting surfaces or polysurfaces and
pressing Enter to cap planar closed boundaries. Menu and `Cap` CMD now stage
one or more BRep inputs; pending Undo removes the last selection and Cancel
leaves the document unchanged. Enter prepares every result before committing a
single transaction, preserving object identity and placement. A failure leaves
all inputs and history unchanged and retains the selection for correction.

The current OCCT adapter handles tested non-overlapping planar loops,
including two opposite box boundaries and a circular single-edge boundary,
resulting in a closed solid. A later checkpoint also covers a tested nested
coplanar pair forming an annular cap and a face with two holes. For a tested
shell with planar and nonplanar boundaries, planar caps are added while the
nonplanar opening remains; a shape with no planar loop is rejected. General
nested topology, intersecting loops, arbitrary curved boundaries and other
input categories remain open. Its hardcoded
`1e-7` tolerance is not a documented Matrix default. Native intermediate peak
memory, F6, SDK/History integration, actual desktop interaction and the M03
human UI gate remain unverified. See
`docs/validation/2026-10-28-solid-audit.md` and
`docs/validation/2026-10-28-solid-cap-source-map.md` for evidence.
