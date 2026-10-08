# 3DM preservation foundation — 2026-10-06

Preserve mode stores the immutable source archive inside FCStd, keeps `(import_namespace,source_uuid)` identities and records unsupported objects as labeled retained App features. Editable geometry uses native openNURBS/OCC conversion. Geometry-only behavior remains available and retains embedded block expansion.

Import mode is exposed through the shared menu/CMD handler. Registered File Open/Import defaults to preservation. Whole preserved content cannot use the legacy geometry writer. Explicit editable-object `export_file(..., geometry_only=True)` reports that source tables, retained records, history and userdata are omitted. Merged preservation export is the next implementation package.

Native inventory reports class/component identity, definition membership, applicable dependencies, geometry/attribute user strings and userdata summaries, resources without opening external paths, and native document/settings diagnostic reports. Full semantic reconstruction of every document/resource class remains unverified in the coverage matrix. Raw source preservation does not establish editable support or Rhino5 writability for those classes.

Verified: Rust68 tests/18 suites; native7/7; source limits512MiB and host manifest32MiB; malformed source/manifest, invalid modes, explicit units; mixed BRep/Point/PointCloud/TextDot/block inventory; attribute user text and light-only retention; FCStd reopen after input/staging removal; exact snapshot hashes; namespace collisions; one Undo/Redo; binding-error rollback; guarded export; menu preservation and unitless scale prompt. Final primary SDK build/link and host processes exited0.

Both immutable ring fixtures pass geometry exchange (101/135 objects;347/547 checks) with unchanged adaptive measurement acceptance. Preservation mode passes identity graph, valid CAD, complete stored manifest and exact snapshot checks, including FCStd reopen for both files.

Fresh reviewer identified unitless message mismatch, missing RenderLight binding, and attribute/dependency/resource inventory omissions. These were corrected and corresponding failing native/runtime cases pass. Actual Rhino5 application interoperability remains untested.

Commands: `rtk cargo test --manifest-path rust/Cargo.toml`; `rtk proxy cmd /c build/3dm-preservation-regressions.cmd`; `rtk proxy cmd /c build/3dm-module-final.cmd`; `rtk proxy powershell -NoProfile -ExecutionPolicy Bypass -File build/run-final-foundation-host.ps1`. Geometry rings ran through `build/run-foundation-rings.ps1`.

Evidence directory: `H:/FreeCAD-src/build/validation/3dm-preservation-foundation-20261006`.
