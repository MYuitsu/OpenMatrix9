# OM9-SOLID-002 — BooleanIntersection

Source engineering contract: `specs/04-solid/om9-solid-002-boolean-intersection.md`. Matrix 8 Book 1 PDF pp.223–224
re-read through continuation to the next command heading on 2026-10-09.
Original PDF remains an ignored reference, excluded from runtime resources.

Implemented slice: Ordered solid BRep, surface-area BRep or closed Mesh sets; validated common results and persistence. Mixed categories, edge-only surfaces and invalid/empty results reject.

Invocation: stable menu/F6 `OM9_SolidIntersection` and CMD `BooleanIntersection`. The
[shared native Edit contract](edit-native-contract.md) distinguishes manual
facts from host options/defaults, geometry types, tolerance, lifecycle and limits.

Validation: options `build/edit_options_smoke-1/19b793add82d48de9e92ffcbf2c43343/results.json` — 57/57;
core `build/edit_commands_smoke-1/4ebfad5e7e844c999ca94dc0a4493dea/results.json` — 115/115. Fixtures use exact feature IDs.
Both FreeCAD processes exited 0. Matching SDK build passed; Rust 140 tests
and Python 23 tests passed. Current regression reports and audit limitations
are in [implementation ledger](../openmatrix9-progress.json).

Status: `partially_implemented`, `validated_supported_slice`. Source edits do
not recompute snapshot results. Live Matrix History, annotation/block/cage
adapters, document-tolerance compatibility and exhaustive topology remain open.
