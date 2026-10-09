# Surface constraints and History implementation plan

> Execute authorized continuation task with independent Refit and History work
> and native constraint verification. No commit/publication requested.

Goal: add Sweep2 G1/G2 and Add Slash, section Refit, Loft endpoint tangent
matching and associative Surface History; synchronize validated scopes into spec_v1.

Architecture: Rust owns bounded fitting and paired rail parameter policy.
C++/Qt owns typed OCCT constraints, geometric certificates, selection and native
document dependency execution. History uses a native persistent Part feature.

Spec: OM9-SURFACE-001/002/003/004/009 and OM9-HISTORY-001 in
OpenMatrix9_Codex_Spec_v1. Source Book1 PDF180–186 and193–195.

## Boundaries

Preserve stable IDs, native transactions, private sources and unrelated chats.
G1/G2 need explicit supporting face-edge refs; no metadata-only enforcement.
Refit fitting samples propose a curve; geometric acceptance needs a separate
positive-weight NURBS convex-hull bound. Reject any uncertified candidate.
Add Slash stores ordered pairs of normalized rail arc fractions and drives
actual section frames, not a display-only line. History stores all original
subreferences, options and placement dependencies; errors invalidate stale geometry.
Host restrictions must be stated without claiming complete proprietary parity.

## Tasks

- [x] Rust Refit: adaptive count with tolerance/4 candidate sampling, finite input
  bounds, periodic closure, unattainable tolerance and no stale published output.
- [x] Native Refit: adaptive trimmed-curve chord hull bound in both directions;
  UI finite millimetre tolerance; commit/persistence and non-modification tests.
- [x] Rust slash mapping: strictly increasing interior pairs, piecewise parameter
  map and finite validation. Native exact arc sampling, table and two-rail picks.
- [x] G1/G2 and tangent matching: native filling constraints for eligible open
  single-edge patches, supporting faces, independent contact/normal/curvature checks.
- [x] History: native SurfaceHistory type with serialized ordered inputs/options,
  source/parent placement recompute, direct child edit detach, invalid source error,
  cycle rejection, native Undo and FCStd restore/source edit.
- [x] Integrate CMD aliases 002/004 and plain Loft History checkbox; preserve
  snapshot behavior when History is off. History closed-rail default per PDF183.
- [x] Build isolated matching SDK module; native advanced options, old options and
  baseline Surface regressions; Rust/Python/source audit and independent review.
- [x] Update exact specs/catalog/status/README/ledger/manifest only after evidence.

Completion evidence: [530 native checks, 164 Rust tests and 23 Python tests](../../validation/2026-10-09-surface-constraints-history.md).
The hull acceptance uses coordinate-scaled numerical padding; it is not a
formal floating-point certificate. Dedicated History menu/F6 parity and all
geometric combinations outside the documented supported slice remain open.

## Review focus

Reversed/grouped support faces, rational curves, rail slash crossing/out-of-order,
source deletion and child edit detach, FCStd reconstruction with every option.
Tests must measure geometry rather than only UI state or metadata.
