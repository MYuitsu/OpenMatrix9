# Native Edit remaining options

**Goal:** Continue the existing Edit implementation against the current Spec v1
contracts, as requested: Trim/Join/Explode/Boolean options, not new commands.

**Architecture:** Rust validates session options and retains stable IDs. The
existing C++ typed Part/Mesh adapters calculate geometry on snapshots, render
transient previews and commit one transaction. Freeze the active projection at
Trim start; projected intersections cut original curves at corresponding
parameters so output retains its depth and analytic geometry.

**Spec:** OM9-TOP11-005/008/010 and OM9-SOLID-001..004 in
`OpenMatrix9_Codex_Spec_v1/specs/`. The current package contains engineering
contracts; original manuals remain in ignored local reference paths; the old registry is absent.
PDF84–87 and223–224 were re-read through continuations on 2026-10-09.

## Constraints and checks

- Preserve existing commands and default 3D Trim behavior, dependency guards,
  input revision checks, cancellation, document switching, Undo and persistence.
- Finite mm tolerance shown to the user; do not enlarge it silently.
- Apparent intersections apply to curves only. Freeze camera projection and
  preserve the original curve parameters/depth. Reject degenerate projection.
- ExtendLines uses imaginary straight cutters; never output extended geometry.
- Mesh and BRep route separately; mixed Boolean input is rejected.
- Explode preserves group membership and world placement; annotations/blocks
  require dedicated adapters, not anonymous geometry substitution.
- Keep unrelated work in the shared checkout. No publication or skill writes.

## Tasks

- [x] Add native failing fixtures for both Trim options, a curved projected
  cutter, frozen camera, tolerance midpoint Join, grouped Explode and additional
  Boolean geometry. Probe native Mesh operations before claiming support.
- [x] Add validated Rust option state and FFI; test invalid values, kind gating,
  and session reset. Expose checkboxes/numeric tolerance through shared UI/CMD.
- [x] Implement Trim extensions/projection and midpoint curve Join. Preserve
  fragment provenance, native analytic outputs and option metadata.
- [x] Implement grouped Explode and native Mesh connectivity if runtime proves
  the API; add BRep surface Boolean routing and native Mesh Boolean routing only
  when validity/analytic-volume checks pass.
- [x] Run complete Rust/Python suites, native Edit regressions/options fixtures,
  source audit and independent review. Record limitations by name.
- [x] Synchronize per-feature records, README, Spec notes/checklists, metadata,
  status summary, manifest and ledger from actual reports.
