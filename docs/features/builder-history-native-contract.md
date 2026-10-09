# Durable Builder History foundation — OM9-HISTORY-001 / OM9-GEM-027

This native API is a reusable OpenMatrix9 foundation. It does not enable original
setting or cutter commands, and does not claim to implement their parameter solvers.

## Source evidence

Verified using installed PyMuPDF against the local reference manuals:

- `matrix8_book_1.pdf`, PDF pages 49–51 (printed 39–41): parent/child History,
  independent child edits break links, Undo can restore them; page 51 explains
  retained gem settings/cutter memories, including deleted outputs and multiple
  settings/cutters per gem. It identifies specific excluded builders.
- `matrix_8_manual_book2.pdf`, PDF page 235 (printed 461), heading Match
  Attributes until Save Styles: select targets then source; available style sheets
  are selected for recreation. Pages 235–236 (printed 461–462) discuss same-shape
  compatibility and retained builder values.
- The same book, PDF page 273 (printed 499), Gem Cutters & Match Attributes:
  cutter cuts do not follow moved gems; stored cutter settings can be recreated.

Consequently live affine mapping below is an explicit OpenMatrix9 recipe choice,
not a recovered promise that all cutters follow gems. Original per-builder
defaults, styles UI, interactive handles, arbitrary parameter solving and .mss
interchange remain outside this foundation.

## Native interface

`OpenMatrix9Gui.createBuilderRecord(gem, featureId, parametersJson, seedObjects,
scaleToGem=False)` returns a hidden `OpenMatrix9Gui::BuilderRecord`. Each seed is
copied; the input objects remain unchanged. Newly created native
`OpenMatrix9Gui::BuilderOutput` names appear in `record.OutputNames`. Callers may
remove temporary seed objects inside their own workflow once recording succeeds.
Seeds must be separate, same-document native Part geometry. Grouped seed geometry
is mapped through its global frame before copying; existing consumers are untouched.

`restoreBuilderOutputs(record)` explicitly recreates missing output slots and
returns all outputs. Deleting an output never triggers implicit resurrection.
An occupied name belonging to an unrelated object is rejected. Multiple
records on one gem remain independent, including duplicate feature IDs; a record
survives deletion of every output.

`matchBuilderAttributes(sourceGem, targetGems)` copies every supported durable
record, retaining each raw parameters string. Source and every target must carry
the exact same explicit, nonempty `App::PropertyString OM9GemShape` tag. Native
shape/role/document checks and recipe validation happen before document writes;
one transaction applies all results and aborts on error. Selecting a subset of
styles is a future command/UI concern; this programmatic foundation matches all
available registered records. It rejects source=target; same-gem restoration uses
`restoreBuilderOutputs`.

Rust checks aggregate replay counts before the native transaction: at most 4,096
new records and 16,384 new outputs, using checked multiplication. Each target's
recipe is validated with the source record's initial dimensions and target's
current dimensions before writes.

All mutating APIs require valid native wrappers, objects still contained by their
documents, and the active editable GUI document with FreeCAD Control permission,
no active edit task and no closing document. `featureId` requires the exact
syntactic form `OM9-<1–16 uppercase domain letters>-<001–999>`. This stores feature
identity; it does not claim the corresponding original command solver exists.
Generic builder names and malformed identities are rejected before writes.

Persistent record/output origin UUIDs identify slot ownership without reverse
links. Existing detached slots owned by the same record are returned unchanged
by restoration; missing slots become new outputs. If global Record is off, new
restored outputs are permanently unrecorded and have no ParentRecord link.

## Versioned recipe and geometry

```json
{"schema":1,"evaluator":"om9.affine-template","version":1,
 "settings":{"builder":"bezel","height":1.7},
 "scale":[1,1,1],"offset":[0,0,0]}
```

Only this registered evaluator/version is supported. Unknown versions or
evaluators fail, with no eval, Python Proxy, expression execution or guessed
builder semantics. Every JSON field in `settings` is preserved byte-for-byte in
the full `Parameters` string; these settings are metadata, not solved geometry.
Optional `scale` defaults to `[1,1,1]`, `offset` to `[0,0,0]`; both are explicit
affine recipe controls. Unknown top-level evaluator fields fail. Duplicate keys,
malformed JSON, nonfinite numbers, excessive size/depth/nodes and invalid numeric
bounds fail in Rust.

Native seed geometry is copied into the source gem's initial global frame,
stored as independent direct children of a persisted template compound. Compound
child order is the stable slot index. Outputs are rebuilt from these copies,
not from whichever output objects happen to survive. The local affine transform
scales about the gem frame origin, then applies offset in millimetres. With
`ScaleToGem=True`, each scale coefficient also multiplies the current/initial
gem-local bounding-box dimension ratio. The result maps into the current global
gem frame. Initial dimensions and frame are persisted explicitly.

Current source gem placement includes parent App::Part frames. Hidden frame
dependencies and native document notifications schedule changes without creating
container cycles. The schema requires positive, finite 3D gem dimensions. This
affine template capability therefore accepts a native 3D gem representation,
not a planar gem outline as a substitute.

## Persistence and History integration

Normal dependency graph: gem → record → output. The record contains a SourceGem
link, templates, raw parameters and an output **name** list, never links back to
outputs. Outputs contain ParentRecord and immutable slot index. The native type
namespace imports OpenMatrix9Gui during FCStd restoration without selecting the
workbench. No Python proxy or runtime memory registry is needed to replay it.

History integration uses `builderHistoryRecorded`, `builderHistoryParents` and
`detachBuilderHistory`. Under the RCORE-09 policy revision (2026-10-09), global
Update suspension preserves existing links and geometry; re-enabling Update
schedules them through the existing Rust History graph, independently of Record.
Record controls new relationships and does not freeze existing outputs.
When Record is off, newly created outputs stay unrecorded permanently. Independent
output Shape/Placement/parent/slot edits detach only that output. Lock refuses
independent edits; Clear Object History detaches the selected output or suspends
the selected record without deleting its durable recipe. Explicit detach preserves
current geometry. Native document properties/transactions support Undo/Redo.

Grouped outputs map evaluated world geometry back into their own native local
frame while preserving Placement. Reparenting schedules remapping; moving an
output-only ancestor follows independent-edit Lock/detach policy. A common gem
and output ancestor follows the recipe without applying its transform twice.
Hidden output frame dependencies persist and refresh after restoration and Undo.

## Ownership and validation

Safe Rust owns JSON schema/version/evaluator registration, finite numeric
validation, affine dimension mapping and explicit shape compatibility. It retains
owned strings and numbers only. FFI pointers are borrowed for one call and never
retained; allocation stays on its owning side. C++ is the required native adapter
for FreeCAD document objects/transactions/persistence, signals and OCCT geometry.
Python is used only for host integration fixtures.

Rust fixture: `rust/tests/builder_history.rs`. Native fixture:
`tests/builder_history_smoke.FCMacro`, with cold restore selected by
`OM9_BUILDER_RESTORE_FILE`. Native fixtures assert independently known centroid
and volume, output deletion/restoration, compatibility all-or-error, multiple
records, global controls, detach and Undo/Redo, and FCStd persistence.
`tests/builder_history_groups_smoke.FCMacro` covers native group coordinates,
shared ancestors, Lock/detach/Undo/Redo, rotated reparenting, saved groups and
aggregate Match rejection without document writes.

Matching SDK acceptance passed 43 storage/policy checks, 10 grouped output/budget
checks and 4 cold-restore checks; the installed SDK module also passed cold restore.
See the [validation record](../validation/2026-10-09-reusable-builder-cage.md) for
the exact runtime checksum, reports and accepted scope.
