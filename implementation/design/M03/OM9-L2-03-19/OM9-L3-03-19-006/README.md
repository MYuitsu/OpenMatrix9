# OM9-L3-03-19-006 — Mesh Reduce Basic

Status: AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING.

Services:
- `crates/om9-core/src/utilities.rs`: bounds, NURBS mesh generation, weld/degenerate repair, hole filling, self-intersection removal, dihedral feature-edge preservation, QEM edge-collapse with normal-flip checks, standard Mesh.corner_uvs field, PBR/material binding, adjustable feature angle, iterative self-intersection repair, adjustable feature angle, iterative self-intersection repair.
- `crates/om9-app/src/utilities.rs`: health entries, B-Rep kernel naked edges, mesh boundary/non-manifold edges, duplicate grouping.
- `crates/om9-app/src/session.rs`: select bad/naked, center, group/ungroup, mesh from NURBS, mesh reduce, health summary.
- `crates/om9-app/src/ui.rs`: separate Object Health Report dock window with per-object rows.

Evidence: `cargo test -p om9-core --test utilities` and `cargo test -p om9-app --test utilities`.
Manual gate M03-UI remains unsigned.
