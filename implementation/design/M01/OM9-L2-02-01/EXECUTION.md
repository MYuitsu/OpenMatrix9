# `OM9-L2-02-01` — execution record (2026-10-19)

**Status:** `AUTOMATED_TARGET_VALIDATED_REVIEW_PENDING`
**Target:** standalone Rust OM9 · **Workspace:** `H:/OpenMatrix9` · **Head:** `b5a15dae9ece3ba4a01e7c26fe60b051c9837e2a`
**Active scope:** 10 L3 × 5 L4 = 50 tasks, all `VERIFIED_WITH_EVIDENCE`.

Machine-readable records: `crosswalk.json`, `l4-matrix.json`, `oracles.json`, `api.md`.

## 1. What this pass added

The semantic rules did not exist as one owner. This pass added
`crates/om9-core/src/rhino_semantics.rs`:

- `SemanticKind` / `entity_semantic_kind` with Point, Line, Circle, Polyline, Polycurve, NURBS
  curve/surface, trimmed surface, extrusion, mesh, cloud, B-Rep and instance kinds.
- `classify_curve` (closed/planar under an explicit tolerance, endpoints, degenerate refusal) and
  `inspect_nurbs_curve` (degree/pole/weight/knot counts, rationality).
- `classify_surface` (verbatim domain, periodic/seam policy, closed-vs-periodic distinction) and
  `SurfaceSemantics::require_orientation` (U/V swap refusal).
- `TrimmedSurface` (underlying + outer/inner UV loops, validate, extension persistence,
  `to_untrimmed()` always refused).
- `DisplayIsocurve` (display-only, not pickable, no `Geometry` conversion) and `BoundaryEdge`
  (pickable topology).
- `classify_shell` (Surface / Polysurface / Solid; open-solid and self-overlap refusal).
- `Extrusion` + `ExtrusionConversion` (closed planar polyline -> closed portable `BrepBody`, source
  preserved as provenance, persistence via extension).
- `is_exact_geometry` / `require_exact_nurbs` (mesh never exact NURBS; no implicit conversion).
- `InstanceSemantic` / `block_instances` / `set_instance_transform` (shared definition, independent
  occurrence identity and transform).

Model integration:
- `om9.trim.surface` and `om9.extrusion` are registered known slots
  (`crates/om9-core/src/model.rs:100-111`).
- `Document::validate` calls `rhino_semantics::validate_extension` for known slots
  (`model.rs:716`), so a damaged semantic payload fails archive decode/encode.

Tests added:
- `crates/om9-core/tests/rhino_semantics.rs` (11 tests for L3-001..010).
- `crates/om9-archive/tests/rhino_semantics_persistence.rs` (4 persistence tests for trim/extrusion,
  damaged payload, mesh-vs-BRep across the codec).

## 2. L4 loop per L3

| L3 | -01 audit | -02 contract | -03 implement | -04 integrate | -05 acceptance |
| --- | --- | --- | --- | --- | --- |
| 001 Point/Polyline | `rhino_semantics.rs:33-75`, `geometry.rs:340-430` | tests `:83`, `:134` | `:118-225` | core test | 11 pass |
| 002 NURBS curve | `geometry.rs:187-292`, `:230-273` | test `:156` | `:243-273` | core test | 11 pass |
| 003 open/closed/planar | `:118-225` policy | test `:191` | `:199-225` | core test | 11 pass |
| 004 surface UV/seam | `geometry.rs:294-338`, `:276-382` | test `:242` | `:331-382` | core test | 11 pass |
| 005 trimmed surface | `:449-532`, `model.rs:100-111,716` | tests `:328`, archive `:62` | `:457-532` | archive persistence | 56 pass |
| 006 edge/isocurve | `:537-615`, `surface_kernel.rs:196-259` | test `:391` | `:544-615` | core test | 11 pass |
| 007 solid/polysurface | `brep_kernel.rs:91-152`, `:619-684` | test `:523` | `:647-684` | core test | 11 pass |
| 008 extrusion | `:687-937`, `model.rs:102-111` | tests `:611`, archive `:101` | `:782-895` | archive persistence | 56 pass |
| 009 mesh vs NURBS | `:940-963`, `geometry.rs:455-499` | tests `:756`, archive `:165` | `:940-963` | archive persistence | 56 pass |
| 010 instance/block | `:965-1020`, `model.rs:632-650` | test `:774` | `:972-1020` | core test | 11 pass |

## 3. Commands and observed results

All commands used `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`. Logs are in
`build/standalone-validation/ui-artifacts/m01-l2-02-01-20261019/`.

| # | Command | Result |
| --- | --- | --- |
| 1 | `cargo test -p om9-core -j 2` | exit 0, **385 passed** (374 prior + 11 new), 0 failed |
| 2 | `cargo test -p om9-archive -j 2` | exit 0, **56 passed** (52 prior + 4 new), 0 failed |
| 3 | `cargo test -p om9-exchange -j 2` | exit 0, **97 passed**, 0 failed |
| 4 | `cargo test -p om9-app --features gpu-sphere --test file_lifecycle --test cold_reopen --test cold_layers --test brep_selection -j 2` | exit 0, **38 passed** |
| 5 | `cargo clippy -p om9-core -p om9-archive --all-targets -j 2 -- -D warnings` | exit 0 |
| 6 | `rustfmt --check --edition 2024` on the three touched files | exit 0 |
| 7 | `cargo test --workspace --no-fail-fast -j 2` | exit 101: **1,199 passed / 170 suites**, 2 known unrelated failures (`mesh_fill`, `ui_events`); see §4 |
| 8 | `python tools/validate_m00_execution.py --require-ready` | exit 1, `NOT READY` (pre-existing M00 records; no QymCAD code imported) |

New test files:

- `crates/om9-core/tests/rhino_semantics.rs`: `point_line_polyline_and_polycurve_have_distinct_kinds`,
  `a_degenerate_line_is_refused_and_never_becomes_a_point_or_polygon`,
  `cubic_nurbs_curve_reports_degree_knots_weights_and_rationality`,
  `closed_and_planar_rules_use_the_stated_tolerance_not_raw_equality`,
  `surface_domains_seams_periodicity_and_orientation_are_explicit`,
  `trimmed_surface_keeps_the_underlying_surface_and_never_promotes_it`,
  `isocurves_are_display_only_and_boundary_edges_are_pickable_topology`,
  `closed_shell_is_a_solid_while_open_shells_are_surface_or_polysurface`,
  `extrusion_stays_lightweight_and_converts_to_a_closed_brep_with_provenance`,
  `a_mesh_and_an_exact_box_never_share_a_semantic_kind`,
  `block_instances_share_a_definition_but_not_identity_or_transform`.
- `crates/om9-archive/tests/rhino_semantics_persistence.rs`:
  `trimmed_surface_semantics_survive_save_and_reopen_without_becoming_untrimmed`,
  `extrusion_semantics_survive_save_and_reopen_and_still_convert`,
  `a_damaged_trim_payload_is_refused_before_publication`,
  `mesh_and_brep_keep_distinct_kinds_across_the_codec`.

## 4. Workspace regression

`cargo test --workspace --no-fail-fast -j 2` result is recorded in `workspace-all-final.log`
(**1,199 passed / 170 suites**). Two failures
were already present in this working tree before this pass and are unrelated to the semantic module
(`om9-app --test mesh_fill` and `om9-app --test ui_events` still expect the removed CPU painter; the
ledger records them at `docs/openmatrix9-progress.json:12939`). Every other target passed; the exact
counts are in `2026-10-19-l2-02-01-checks.json`.

## 5. Explicit deviations / residuals

- Geometric face-face self-intersection needs OCCT; the portable classifier refuses open shells and
  coincident duplicate faces, which is the topology evidence available without a kernel. The
  limitation is documented on `classify_shell`.
- Extrusion conversion is limited to closed planar **polyline** profiles. A NURBS-profile extrusion
  is refused (`Unsupported`) rather than approximated; that path belongs to a kernel-backed L3.
- Trim and extrusion parameters are persisted in validated known extension slots instead of new
  `Geometry` enum variants. This keeps the archive schema stable and still makes
  `Document::validate` refuse damaged payloads.
- No external reviewer signed the record; every L3 is `AUTO_ONLY`.
- The global M00 gate is NOT READY (pre-existing); this work imported no QymCAD code.

## 6. Evidence pointers

- Design: `implementation/design/M01/OM9-L2-02-01/`
- Per-L3/L4 evidence: `implementation/evidence/M01/OM9-L3-02-01-001/` … `-010/`
- Checks receipt: `implementation/evidence/M01/2026-10-19-l2-02-01-checks.json`
- Logs: `build/standalone-validation/ui-artifacts/m01-l2-02-01-20261019/`
- Milestone runtime records: `L2.json` and each `L3-*/l3.json` `runtime_implementation`
- Ledger: `docs/openmatrix9-progress.json` (`m01-l2-02-01-rhino-nurbs-semantics-2026-10-19`)
