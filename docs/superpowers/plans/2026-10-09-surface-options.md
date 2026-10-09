# Surface options implementation plan

**Goal:** Extend the native Sweep1, Sweep2 and Loft option workflows requested by the owner; synchronize only verified behavior into Spec v1.

**Architecture:** Keep the Rust command session and validation layer. Extend the existing C++/Qt dialog and typed Part adapter; use an isolated OCC helper for the additional Loft construction policies. All results remain transient until one successful document transaction.

**Spec:** `OpenMatrix9_Codex_Spec_v1/specs/03-surface/om9-surface-{001,003,009}-*.md`.
Manual checked on 2026-10-09: Book 1 PDF 180–183, 184–186 and 193–195. The original command defaults established there are Freeform/Normal and Do Not Simplify; other initial settings below are explicit host choices.

**Constraints:** Preserve IDs, source contracts, world placement, original inputs, atomic commit and source-reference persistence. Do not change History variants. Reject unsupported solver combinations with an editable error rather than silently ignoring an option. Keep unrelated concurrent changes.

## Deliverables and verification

- [ ] Rust semantic tests first: Maintain Height transport; explicit style/closed-option validation; chained rail state and Undo.
- [ ] Add Maintain Height for the single-profile two-rail transport, exposing and rejecting unsupported multi-profile combinations explicitly.
- [ ] Add section Rebuild (existing Rust fitter), common Preview toggle/button, and supported Closed Sweep on closed rails with at least two sections.
- [ ] Add typed native Loft policies: Loose control-net construction, Tight centripetal interpolation, Uniform parameterization, Developable pair construction with a conservative geometry check. Validate topology, profile preservation where applicable and failed-build cleanup.
- [ ] Add Chain Edges input grouping, Automatic/Natural seam alignment and multi-edge seam handling without changing source geometry.
- [ ] Extend native option tests through the real menu/CMD/dialog, with numeric geometry assertions, cancellation, invalid settings, Undo/Redo and FCStd persistence. Confirm existing surface fixtures still pass.
- [ ] Review changed geometry/state paths; run relevant Rust/Python/source checks and matching SDK build/runtime.
- [ ] Update three feature records, Spec v1 notes/checklists, JSON/YAML, README and ledger; refresh manifest according to current package rules.

## Remaining solver work must stay explicit

Tangency/curvature constraints, Refit's certified error guarantee, Global Shape Blending, Simple Sweep/Refit Rail, and general Add Slash correspondence require distinct geometry algorithms. They must not be presented as enabled merely because their controls or catalog entries exist. Road-like frames and miter behavior may be enabled only when native fixtures demonstrate their source contract. Developable must not silently return an arbitrary ruled surface.
