# Normalize the selected feature

Use the ID from `FEATURES.json` in the selected checkout. Resolve the package from its feature lookup/catalog: `OpenMatrix9_Codex_Spec_v1/` or the legacy `ref/matrix9/OpenMatrix9_Codex_Spec_v1/`, whichever exists and matches that checkout. Read `IMPLEMENTATION_RULES.md` and the package's available source/evidence guidance before filling implementation decisions; use `SOURCES.md` when that package provides it.

Record the following in `docs/features/<ID>.md`, derived separately from the original package:

| Contract field | Required evidence/decision |
|---|---|
| Identity | Exact ID/name/command/domain/spec path; source manual and printed/PDF page range, including command continuation pages up to the next feature heading |
| Evidence | Quote or summarize narrowly with file/page/line; separate inference and design choice |
| Inputs | Supported object types, selection order, preconditions, empty/invalid selection |
| Parameters | Meaning, units, ranges, default values and why supported; explicit `TODO_EVIDENCE` for missing facts |
| Outputs | Geometry/object type, topology, ownership, placement, orientation and dependencies |
| Lifecycle | Invocation and applicable preview/update/commit/cancel; do not assume every command is a builder |
| History | Applicable dependency recomputation, stable IDs and update propagation |
| Errors | Failed operations, no-op/cancel, cleanup and useful user feedback |
| Undo/save | Transaction boundary, undo/redo, document serialization and reload |
| Validation | Geometry/state invariants, fixture/manual comparison, tolerances and live FreeCAD checks |
| Continuation | Current verified status, unresolved question and smallest next document/function |

Builder preview geometry must be separated from committed document geometry. Cancellation must leave the document in the intended pre-command state. If the original lifecycle is unproven, distinguish a chosen host interaction design from source evidence.

Source dependency cues such as “History”, “Mesh” and “Builder” are detected terms. Verify actual dependencies before choosing architecture or marking a framework applicable.

## Conditional reference reads

| Condition | Read next |
|---|---|
| Interactive builder confirmed | `specs/01-core/om9-buildercore-001-builder-framework.md` |
| Source or behavior uses History | `specs/01-core/om9-history-001-history-workflow.md` |
| Styles presets are part of selected feature | `specs/01-core/om9-style-001-styles-framework.md` |
| Clayoo feature | `specs/14-subd/README.md`, then exact ID spec |
| T-Splines feature | `specs/06-tsplines/README.md`, then exact ID spec |
| Original code resolves a missing behavior | Native mapping skill, selected container/event/VA |

## Example: Gem Loader

Helper `feature OM9-GEM-001` resolves `gvLoader`, `specs/10-gems/om9-gem-001-gem-loader.md`, Matrix 8 Book 2, `matrix_8_manual_book2.pdf`, printed p.402/PDF p.176.

The supported workflow cues select the loader, shape and size. Cut Type, Keep Original Size, Gem Shape, Gem Size, On Surface and Custom Gem Sizes are option names. They do not supply values, units, placement rules or cancellation behavior. The catalog marks it `kind: command`; a detected Builder mention does not change that fact.

The Gem Loader section continues on PDF p.177/printed p.403. Read both p.176–177 for its parameter/UI description; do not stop at the catalog's first page. If implementing On Surface, resolve the separate OM9-GEM-011 spec rather than treating the branch as fully described here.

The package intentionally excludes its PDFs. Local files discovered during retrieval testing are registered in `docs/openmatrix9-reference-locations.json`; validate paths when used and update the project registry if files move. This machine has the Book 2 and Addendum PDFs in Downloads, even though they are absent under `ref`. If a cited file is absent, record the exact missing page; do not invent values to claim fidelity. Menu wiring, feature identity and independent infrastructure may proceed without that page.

## Distinct domains

Clay Edit is `OM9-SUBD-001`/`ClayEdit`, not a T-Splines HUD feature. Conversion from T-Splines to Clayoo is a separate command (`OM9-SUBD-043`). Emboss and Matrix Art likewise use separate domain catalogs. Resolve IDs through the helper rather than guessing a filename from a menu label.
