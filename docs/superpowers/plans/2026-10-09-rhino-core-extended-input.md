# Rhino core extended numeric input implementation plan

> For agentic workers: execute the existing approved P0 contracts using
> superpowers:subagent-driven-development or superpowers:executing-plans;
> run each independent RED/implementation/GREEN cycle before integration.

**Goal:** Extend the common Rust input grammar and route its typed values through
enabled point consumers, preserving their existing unit/frame/phase policies.
**Architecture:** Rust owns numeric parsing, tagged notation, conversion and phase
routing. The existing borrowed native point/length ABI remains unchanged. FreeCAD
fixtures validate the real command consumers; no native parser fallback is added.
**Tech stack:** Rust, existing FreeCAD/Qt adapter, Python application fixtures.
**Spec:** RCORE-02/03/04/05, especially RCORE-03 input/output clauses and C07/C09.
**Baseline:** local commit `349203d`; all P0 remains in progress.

## Evidence and decisions

The exact [Rhino 5 numeric-entry help](https://docs.mcneel.com/rhino/5/help/en-us/user_interface/unit_systems.htm)
was read on 2026-10-09. It documents fraction, compound feet/inches, polar,
spherical and surveyor forms, and world/relative coordinate prefixes. The local
guide mapping is PDF49–51 for coordinates, PDF22 for command options and PDF47–48
for constraints. One-shot angle is an event distinct from an accepted point;
live constrained picking remains a subsequent required P0 state/preview task.

Do not infer every prefix permutation. Preserve existing Cartesian semantics;
support documented non-Cartesian branches with explicit notation in provenance.
Existing original text/context/result fields remain; grammar becomes an honest
extended version. Angles are radians internally and never length-scaled.
Numerical bounds, ambiguous syntax rejection and unsupported locale are explicit
OM9 policies, not claims of exhaustive Rhino parity. Previously verified runtime
reports stay immutable.

## Shared interface and ownership

- `units::parse_angle(&str) -> Result<f64, UnitError>` returns radians; plain
  numbers/fractions denote degrees, and DMS remains dimensionally separate.
- `coordinates::NumericSyntax { Point, Scalar, AngleConstraint }` and
  `classify_numeric(&str)` replace comma-only routing after exact option handling.
- `PointInput` retains evaluated Cartesian components and adds typed source
  notation; radius/angle fields cannot be mislabeled as millimeter components.
- Unit implementer owns units.rs/tests; parser implementer owns coordinates.rs,
  coordinate_input.rs/tests; session implementer owns curve.rs/solid.rs/tests.
  Parent owns native fixtures, reference probes, integration and evidence.

## Tasks

- [x] Read mapped guide/help and record independent numeric oracles; use installed
  Rhino 5 for uncertain spherical/prefix behavior where runtime is available.
- [x] RED then implement bounded fractions/feet/inches and independent angle/DMS
  parsing. Preserve legacy accepted length spacing and per-family unit policies.
- [x] RED then implement explicit polar/spherical/surveyor notation, frame/base
  resolution, typed classification and bounded report serialization.
- [x] RED then migrate Curve/Solid numeric phase gates, preserving options,
  scalar construction, accepted previous point, Undo and recoverable errors.
- [x] Run shared/native geometry oracles on rotated/translated CPlane with mixed
  model units; verify rejected input leaves phase, objects and Undo unchanged.
- [x] Run full Rust and affected SDK/native regressions, independent review,
  source audit and exact spec/catalog/ledger synchronization; commit locally.

## Review focus

- Mixed signs, zero denominators, integer overflow and underflow must reject
  without accepting partial numeric values or changing session state.
- Polar/spherical radius, elevation and angle dimensions must stay separate;
  tangent singularities and final world-coordinate overflow must reject.
- World-relative forms bypass CPlane transform; local-relative forms add the
  last accepted point without adding the plane origin twice.
- Angle constraint/option input must never become an accepted point or scalar.
- Report notation/context must match actual geometry, with owned UTF-8 and
  atomic output buffers; no document/Qt pointers cross into stored Rust state.

All missing grammar/locale variants, live constraints, common session reduction,
snap source identity, CPlane modes, selection/layers and durable History remain
in the all-P0 matrix until each contract has complete evidence.
