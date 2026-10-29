# OM9-L2-01-10 — 02-be-occt-boolean-fillet-shell-ops

- **Milestone:** M03 (Geometry 3DM)
- **Owner:** `OM9-L2-01-10` — nine L3 owners, 50 L4 records
- **Delivery date:** 2026-10-22
- **Status:** `AUTOMATED_TARGET_VALIDATED` (`manual_test_gate: AUTO_ONLY`)
- **Kernel:** OCCT 8.0.1, pinned at `native/occt/pin.json` (`V8_0_1`, `b8f597c…`)
- **Crate:** `crates/om9-kernel` — `src/editing.rs` (safe Rust) over `src/editing_bridge.cpp`
  and the shared `src/brep_snapshot.hxx`

## What was delivered

| L3 | Capability | Rust / native entry point |
|---|---|---|
| `OM9-L3-01-10-001` | B-Rep boolean union | `editing::boolean(.., BooleanOp::Union)`, `om9_occt_boolean` |
| `OM9-L3-01-10-002` | B-Rep boolean difference | `editing::boolean(.., BooleanOp::Difference)` |
| `OM9-L3-01-10-003` | B-Rep boolean intersection | `editing::boolean(.., BooleanOp::Intersection)`, typed `BooleanOutcome::Empty` |
| `OM9-L3-01-10-004` | Curve and face split | `editing::split_face`, `om9_occt_split_face` |
| `OM9-L3-01-10-005` | Planar cap and close solid | `editing::cap_planar`, `om9_occt_cap` |
| `OM9-L3-01-10-006` | Fillet and chamfer | `editing::fillet`, `editing::chamfer`, `om9_occt_fillet/chamfer` |
| `OM9-L3-01-10-007` | Offset face and thicken shell | `editing::thicken`, `editing::offset_shape` |
| `OM9-L3-01-10-008` | Face extract, unjoin, rejoin | `editing::extract_face`, `remove_face`, `sew`, `sew_all` |
| `OM9-L3-01-10-009` | Geometry health and naked edges | `editing::geometry_health` over the exact B-Rep measure |

## Ownership and rules

- Safe Rust owns validation (operand requirements, selection indices, tolerances, budgets), the
  result oracles and the typed error contract. C++ only maps admitted data onto OCCT algorithms
  and contains exceptions.
- Inputs are immutable `Brep` values: a failed operation cannot mutate them and never returns a
  partial body. A boolean with no face result is `BooleanOutcome::Empty`, not a zero-face shape.
- Selection is by index into the canonical OCCT map order (the same order as the published
  `faces`/`edges`). `edge_endpoints` is a display-derived selection helper; acceptance always
  measures the exact B-Rep.
- Every measured value comes from `creation::measure` (canonical source), including volume, area,
  topology counts and naked (free boundary) edges. The tessellation is never consulted for health.

## Measured acceptance (this checkout)

`cargo test -p om9-kernel --features 3dm-brep --test l2_01_10 m03_01_10_receipt -- --exact --nocapture`

| Fixture | Measured |
|---|---:|
| Union A∪B (5 mm overlap) | 1499.999999999999 mm³ |
| Difference A−B | 500.000000000000 mm³ |
| Intersection A∩B | 500.000000000000 mm³ |
| Split 10×20 face at x=5 | 200.000000000000 mm² (two 100 mm² pieces) |
| Cap open 10³ box | 1000.000000000000 mm³, closed, 0 naked edges |
| Fillet one vertical edge r=1 | 997.853981633974 mm³ |
| Chamfer one vertical edge 1 mm | 995.000000000000 mm³ |
| Thicken 10×20 face 2 mm | 400.000000000000 mm³ |
| Offset 10×20 face 2 mm | 200.000000000000 mm² |
| Extract a 10×20 face | 600.000000000000 mm² |
| Closed box naked edges | 0 |
| Open shell naked edges | 4 |

Negative fixtures (open-shell operands, tampered sources, trim misses, degenerate or non-planar
loops, zero value, unknown index, oversized fillet radius, thickening a solid) all return a typed
error; the input body is unchanged. An oversized fillet fails in the kernel (`Kernel`), the rest are
refused before the call (`Invalid`).

## Verification commands

```
cargo test -p om9-kernel --features 3dm-brep --test l2_01_10
cargo test -p om9-kernel --features 3dm-brep --test l2_01_09
cargo test -p om9-kernel --features 3dm-brep
cargo check --workspace --all-targets -j 2
cargo clippy -p om9-kernel --features 3dm-brep --all-targets -- -D warnings
cargo fmt --all -- --check
python tools/public_source_audit.py
python tools/om9_milestone_dashboard.py
```

### Runtime DLLs

The test executable links OCCT dynamically, so `TKernel.dll`, `TKShHealing.dll` and their
`tbb12.dll`/`jemalloc.dll` dependencies must be resolvable. Either add these directories to `PATH`
or keep the DLLs beside the test executable (`<target>/debug/deps`):

- `build/standalone-deps/occt-8.0.1-sdk/bin`
- `build/standalone-deps/occt-8.0.1-3rdparty/3rdparty-vc14-64/tbb-2021.13.0-x64/bin`
- `build/standalone-deps/occt-8.0.1-3rdparty/3rdparty-vc14-64/jemalloc-vc14-64/bin`

## Evidence layout

- `implementation/design/M03/OM9-L3-01-10-00X/{api.md,source_crosswalk.json}`
- `implementation/fixtures/M03/OM9-L3-01-10-00X/{positive.json,negative.json}`
- `implementation/evidence/M03/OM9-L3-01-10-00X/` — one receipt per L4 (implementation, regression,
  integration-stress where planned, acceptance and the AUTO_ONLY review record).

## Deliberate scope limits

- Curve split is delivered for a straight trim line on one planar face; general curve/face
  intersections are a separate owner task.
- Capping refuses a non-planar naked loop by name instead of approximating it.
- macOS execution is not verified.
- The planning tree keeps `SPEC_ONLY_NOT_IMPLEMENTED`; delivery is recorded through
  `runtime_implementation` plus this evidence package.

## Solid partition foundation — 2026-10-28

The native adapter partitions one closed BRep solid by one closed BRep solid;
Rust validates the compound and extracts separately measured pieces. This is
not the full Boolean Split command. [Evidence and remaining work](../../../../docs/validation/2026-10-28-solid-audit.md).

The subsequent Rust helper partitions multiple closed single-solid targets and
cutters and deduplicates equivalent cells; command, transaction and UI remain open.

Create Solid now has an OCCT MakerVolume adapter for intersecting BRep
surfaces and region extraction, with bounded Rust input/output validation.
The Solid audit records tested cases and unclosed topology and runtime gates.
