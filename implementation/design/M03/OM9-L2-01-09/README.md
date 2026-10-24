# OM9-L2-01-09 — 01-be-occt-geometry-creation-ops

- **Milestone:** M03 (Geometry 3DM)
- **Owner:** `OM9-L2-01-09` — nine L3 owners, 47 L4 records
- **Delivery date:** 2026-10-22
- **Status:** `AUTOMATED_TARGET_VALIDATED` (`manual_test_gate: AUTO_ONLY`)
- **Kernel:** OCCT 8.0.1, pinned at `native/occt/pin.json` (`V8_0_1`, `b8f597c…`)
- **Crate:** `crates/om9-kernel` — `src/creation.rs` (safe Rust) over `src/creation_bridge.cpp`
  and the shared `src/brep_snapshot.hxx`

## What was delivered

| L3 | Capability | Rust / native entry point |
|---|---|---|
| `OM9-L3-01-09-001` | Analytic solid primitive factories | `creation::primitive`, `om9_occt_primitive` |
| `OM9-L3-01-09-002` | Curve to solid extrusion | `creation::extrude`, `creation::extrude_wire`, `om9_occt_extrude` |
| `OM9-L3-01-09-003` | Revolve profile and rail revolve | `creation::revolve`, `om9_occt_revolve` |
| `OM9-L3-01-09-004` | Pipe from curve | `creation::pipe`, `om9_occt_tube` |
| `OM9-L3-01-09-005` | Planar boundary surface generator | `creation::planar` (via `om9_occt_planar`) |
| `OM9-L3-01-09-006` | Loft through cross-sections | `creation::loft`, `om9_occt_loft` |
| `OM9-L3-01-09-007` | Sweep single and double rail | `creation::sweep1`, `creation::sweep2` (sections → `om9_occt_loft`) |
| `OM9-L3-01-09-008` | Surface from edge curves | `creation::surface_from_edges`, `om9_occt_surface` |
| `OM9-L3-01-09-009` | General profile to B-Rep adapter | `creation::profile_to_brep`, `creation::CreationSession` |

## Ownership and rules

- Safe Rust owns validation (planarity, simplicity, winding, frame continuity, rail separation,
  budgets), the frame/section math, the generational handle session and the measured oracles.
- C++ is limited to the OCCT construction call plus exception containment. No `TopoDS_Shape`, OCCT
  handle or topology index crosses into safe Rust.
- Every operation is one complete call returning one owned `Brep`; a refused input returns
  `CreationError` before a body exists.
- Measurement is from the canonical OCCT source (volume, area, topology counts, closedness,
  optimal bounds), never from the display mesh.

## Measured acceptance (this checkout)

`cargo test -p om9-kernel --features 3dm-brep --test l2_01_09 m03_01_09_receipt -- --exact --nocapture`

| Fixture | Measured |
|---|---:|
| Box 10×20×30 | 6000.000000 mm³ |
| Sphere r=5 | 523.598775598299 mm³ |
| Cylinder r=5 h=10 | 785.398163397449 mm³ |
| Cone r=5 h=10 | 261.799387799150 mm³ |
| Torus R=10 r=2 | 789.568352087151 mm³ |
| Ellipsoid 3/4/5 (2026-10-23) | 251.327412284846 mm³ (authored rational net; adaptive integration) |
| Pyramid 4 sides R=10 h=10 | 666.666666666667 mm³ |
| Extrusion 10×10×30 | 3000.000000 mm³ |
| Open extrusion side area | 1200.000000 mm² |
| Revolve annulus r=5..7 h=10 | 753.982236861551 mm³ |
| Pipe r=2 h=10 | 125.663706143592 mm³ |
| Planar 10×20 face | 200.000000 mm² |
| Loft 3 squares | 1000.000000 mm³ |
| Sweep1 2×3 × 10 | 60.000000 mm³ |
| Sweep2 rails 3 apart, height 2 | 60.000000 mm³ |
| Filled unit square | 1.000000 mm², deviation 0 mm |
| Profile face 10×20 | 200.000000 mm² |

Negative fixtures (zero/NaN sizes, degenerate or self-intersecting loops, crossing revolutions,
zero tangents, non-perpendicular profiles, coincident rails, unsupported span, boundary gaps,
released handles) all return a typed `CreationError` and publish nothing.

## Verification commands

```
cargo test -p om9-kernel --features 3dm-brep --test l2_01_09
cargo test -p om9-kernel --features 3dm-brep
cargo check --workspace --all-targets -j 2
cargo clippy -p om9-kernel --features 3dm-brep --all-targets -- -D warnings
cargo fmt --all -- --check
python tools/public_source_audit.py
python tools/om9_milestone_dashboard.py
```

### Runtime DLLs

The test executable links OCCT dynamically, so `TKernel.dll` (and its `tbb12.dll` /
`jemalloc.dll` dependencies) must be resolvable at run time. They are not copied by `cargo`;
either add these directories to `PATH` before running, or keep the DLLs beside the test
executable (`<target>/debug/deps`):

- `build/standalone-deps/occt-8.0.1-sdk/bin`
- `build/standalone-deps/occt-8.0.1-3rdparty/3rdparty-vc14-64/tbb-2021.13.0-x64/bin`
- `build/standalone-deps/occt-8.0.1-3rdparty/3rdparty-vc14-64/jemalloc-vc14-64/bin`

After a target-directory clean the deployed copies are gone; without one of the two steps the
test process exits with `0xc0000135` (`STATUS_DLL_NOT_FOUND`) before running a single test.

## Evidence layout

- `implementation/design/M03/OM9-L3-01-09-00X/{api.md,source_crosswalk.json}`
- `implementation/fixtures/M03/OM9-L3-01-09-00X/{positive.json,negative.json}`
- `implementation/evidence/M03/OM9-L3-01-09-00X/` — one receipt per L4 (implementation, regression,
  integration-stress where planned, acceptance and the AUTO_ONLY review record).

## Deliberate scope limits

- Planar inner loops (holes) are a separate owner task; the delivered planar face admits one loop.
- QymCAD role naming (`Roles`/`GeoName`) is not reused; stable role names remain an M04
  history/naming concern.
- macOS execution is not verified.
- The planning tree keeps `SPEC_ONLY_NOT_IMPLEMENTED`; delivery is recorded through
  `runtime_implementation` plus this evidence package.

Ellipsoid correction (2026-10-23): [exact rational net and independent geometry evidence](../../../../docs/validation/2026-10-23-exact-ellipsoid.md). The earlier volume deficit was dominated by fixed quadrature and was insufficient to certify a surface approximation. Historical receipts remain preserved.

## Solid follow-up — 2026-10-23

Exact curve extrusion and one-transaction ExtrudeAll, Text/Boolean menu prompts, and span-aware volume integration are verified. Kernel 71 passed; app 191 passed; native Text/Boolean/ExtrudeAll saved files cold-measured. The earlier Ellipsoid-only checkpoint above is historical.

[Latest report](../../../../docs/validation/2026-10-23-solid-ui-followup.md) · [Human worksheet](../../../../implementation/evidence/M03/solid-ui-followup-2026-10-23/M03-UI-CHECKLIST.md). M03-UI remains unsigned.
