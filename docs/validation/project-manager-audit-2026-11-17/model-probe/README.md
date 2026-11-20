# Project Manager audit probe

This diagnostic binary compiles the production `project_manager.rs` module by path without modifying it. It records six observed gaps; exit zero means the diagnostic ran, not that the product meets the manual. It does not link the full app test module, open a window or mutate user documents.

Repeat with `CARGO_TARGET_DIR=L:/om9-target` and `cargo run --manifest-path docs/validation/project-manager-audit-2026-11-17/model-probe/Cargo.toml -j 2 --offline`. The audit originally ran the equivalent harness under ignored `build/project-manager-audit-2026-11-17/model-probe`. Only source-module relative paths differ in this archived copy.
