# M01 OM9-L3-01-01-002 Workbench Docking Regions (2026-10-16)

Scope: standalone Rust OM9, module OM9-L2-01-01. Only what was run is listed. Full logs: `implementation/evidence/M01/OM9-L3-01-01-002/`; verification note: `.agents/tasks/m01-l3-002-docking/verification.md`.

## Implemented

- `standalone/crates/om9-app/src/workbench_regions.rs`: pure region model (left rail, center with four viewports or one, optional right dock, command input, toolbar, status bar), explicit `RegionError`/`RegionNote`, `compute_regions`, `compute_regions_for`, `tile_views`, `RegionPlan::validate`, and two seam traits toward OM9-L2-01-02/03 tested against stubs.
- `ui.rs`: the frame validates a plan each pass, tiles the viewports through `tile_views`, and shows errors and notices; `chrome.rs`: panel-size sync pauses while the check has an error.
- Artefacts: `source_crosswalk.json`, `api.md`, `positive.json`, `negative.json`, red/target/regression logs, `ui-review.json`.

## Measured

- Red run then green: lib 21 and integration 8 new tests pass; `cargo test --workspace`, `cargo check --workspace --all-targets`, `cargo build --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check` exit 0.
- `om9.exe --ui-audit` at 1600x1000 (right dock shown) and 800x600 (right dock yielded): exit 0, screenshots and receipts hashed in the regression log.

## Not passed / not done

- `python tools/public_source_audit.py` exits 1 on 11721 pre-existing errors in private reference and untracked trees; none in files touched here.
- `tools/validate_m01_spec.py` is absent: not run.
- M01-UI manual gate OPEN, reviewer NOT_SIGNED; DPI and multi-monitor not captured.
- Legal reuse PENDING_HUMAN_M00_APPROVAL; M00 NOT_PASS; OM9-L2-01-02/03 not implemented.
- No `planning_state` or skill file changed; no commit.
