# Design review: M01 backend trio (OM9-L2-01-08, OM9-L2-01-02, OM9-L2-01-03)

Reviewed: `IMPLEMENTATION-CHECKLIST.md`, `OM9-L2-01-0{8,2,3}/{crosswalk.json, api.md, oracles.json}`,
`.agents/tasks/m01-l2-01-02-03-backend/{stub-integration-note.md, m00-decisions.md}`. Cited source was
re-read in the working tree (no builds or tests run, per the step instructions).

**Verdict: CHANGES_REQUESTED** (1 HIGH, 9 MEDIUM, 13 NIT).

The audit itself is strong. All 31 L3s (7 + 11 + 13, matching the `OM9_Milestones` directories) are
classified, and each one has file:line evidence. I spot-checked about 60 citations and they hold.
The design keeps one engine (`session::Controller`), plans no loop edits to forbidden UI files, and
decides who owns the Line-save bug. What blocks approval is mostly plan mechanics: the plan never
produces the one L4 artifact the UI L3s are blocked on, and several error labels, index rules and
budget rules are not pinned down.

## Criteria summary

| # | Criterion | Result |
|---|---|---|
| 1 | 31 L3s classified with file:line evidence | PASS (08: 0/6/1, 02: 3/7/1, 03: 2/8/3; evidence verified) |
| 2 | One command engine, no rewrite of owners | PARTIAL: dispatch is single, but a second catalog parse stays in `ui.rs:351` (M5) |
| 3 | Stable-ID early slice for L2-01-03 | PARTIAL: ADR-03-1/2 good; identity semantics of snapshot restore unspecified (M6) |
| 4 | Stub note names functions, replaced stubs, CommandInputUnavailable test | PASS on content; trigger/ordering gaps (M9) |
| 5 | No planned edits to concurrency-forbidden UI files | PASS for the loops; orchestrator-only edits are gated (M9) |
| 6 | Line-save bug ownership decided | PASS (codec owner L2-01-12; fix = `float_roundtrip` in 8.1 with RED and golden-byte guard) |
| 7 | Concrete RED per gap | PARTIAL: test-only/already-green items have no defined RED procedure (M1) |

## Findings

### HIGH

**H1. The L4-05 artifacts that unblock the UI L3s are never produced, and 02 runs before an artifact it depends on.**
Where: checklist §0 ("DONE_WITH_EVIDENCE L3s only get `<L4-05-id>.regression.log`"), 2.8 (only 006/007/008), 3.10 (only 001/006).
`OM9_Milestones/.../L2-01-01/L3-00{1..8}/L4-03/l4.json` and `L2-03-07/L3-00{1..8}/L4-03/l4.json` declare
`depends_on_task_ids: ["OM9-L4-01-02-001-05", "OM9-L4-01-03-001-05", ...]`.
`OM9-L4-01-02-001-05` (outputs `implementation/evidence/M01/OM9-L3-01-02-001/OM9-L4-01-02-001-05.regression.log`
plus `.evidence.json` per its runbook) is never written, because L3-01-02-001 is PARTIAL. So even with every
test green, L3-01-01-002/-003 stay BLOCKED_DEPENDENCY. That defeats the user's request "Làm cho xong 2 cái kia để
không bị block nữa". Separately, `L2-01-02/L3-002/L4-03` depends on `OM9-L4-01-03-001-05` and
`OM9-L4-01-02-001-05`. The checklist writes the first only in 3.10 (after loop 02), and item 2.1 bundles
L3-001 and L3-002 with no L4-05 checkpoint between them.
Fix (concrete):
- §0: replace the rule with "Every L3 whose targeted tests are GREEN gets
  `implementation/evidence/M01/<L3-id>/<L4-05-id>.regression.log` and `<L4-05-id>.evidence.json`
  (command, exit code, counts, SHA-256 of the logs). An L3 that keeps a NEEDS_USER_M00_DECISION
  sub-item records it in evidence.json as `open_sub_items` and still gets its regression log."
- New item **2.0** before 2.1: rerun `cargo test -p om9-core --test atomic_transactions --test model_admission`,
  `cargo test -p om9-archive --test container_admission` and `cargo test -p om9-app --test cold_reopen` into
  `OM9-L3-01-03-001/OM9-L4-01-03-001-05.regression.log` (L3-01-03-001 is DONE_WITH_EVIDENCE today).
- Split 2.1 into 2.1a (L3-001 registry → GREEN → write `OM9-L4-01-02-001-05.regression.log` from
  `cargo test -p om9-app --lib session::registry --test command_registry --test command_sessions --test file_widgets`)
  and 2.1b (L3-002 descriptor).
- 2.8 and 3.10: write L4-05 logs for **all** L3s of the module, not only the DONE ones.

### MEDIUM

**M1. No RED procedure for gaps whose test may pass on first run.**
Where: 2.4 (L3-01-02-009, "code may already pass parts"), 3.1 (L3-01-03-003/-004/-005 are PARTIAL(test-only);
`model.rs:322-327` and `:337-341` already refuse), 8.6 ("Tests only"). §0 requires saving failing output to
`<L4-03-id>.red.log`, which cannot exist for these.
Fix: add to §0: "For a PARTIAL(test-only) gap, or when the new test passes on its first run, save the
first-run output as `<L4-03-id>.red.log` with the header `PRE_EXISTING_GREEN: behavior already implemented at <file:line>; test added as proof`.
Also add a mutation check: temporarily invert the guarded condition, confirm the test fails, and save that as
`<L4-03-id>.mutation.log`. Then restore the code." For 2.4 the real RED is
`assert!(!registry.by_id(UNDO)?.repeatable && registry.by_id(LINE)?.repeatable)` together with
`repeat` being set from `descriptor.repeatable`. Make that the named RED.

**M2. `UnitError` → `om9_core::Error` labels are unspecified, but the oracles assert exact text.**
Where: 08 api.md §3 (`every other variant -> Invalid(<static label>)`), `OptionDomain::check` doc, 02 api.md §5 rule 4.
Oracles expect `Invalid("option unit/dimension")` (08-004 bare-number, 08-006 missing-unit), while
`request.rs:384` today returns `Invalid("option dimension/type")` for a wrong-dimension Quantity, and 02-005
expects `Unsupported("unit")` for `length=2.5yd` even though rule 4 says every malformed value becomes `Invalid(spec.invalid_value)`.
Fix: add this table to 08 api.md §3 and reference it from 02 §5:

| UnitError | Error |
|---|---|
| UnsupportedUnit | `Unsupported("unit")` |
| MissingUnit, WrongDimension | `Invalid("option unit/dimension")` (also replaces `"option dimension/type"` for the Number-domain arm of `check`; the existing `source_binding` test only asserts `is_err`) |
| NonFinite, OutOfRange, NotAnInteger | `Invalid("numeric option domain")` |
| Malformed | `Invalid("numeric text")` |

02 §5 rule 4 becomes: "`UnsupportedUnit` propagates as `Unsupported("unit")`. Every other parse/check failure
maps to `Invalid(spec.invalid_value)`."

**M3. Registry name index: same-descriptor duplicates and `DuplicateAlias` semantics are undefined.**
Where: 02 api.md §3 (`BTreeMap<String, Vec<CommandId>>`, "match on name, command and aliases").
Catalog `OM9-CURVE-002` has `name: "Line"` and `command: "Line"` (verified in `assets/features.json`). Indexing both
pushes the same ID twice under `"line"`. A literal reading of "≥ 2 implemented match → Ambiguous", or a build-time
alias check, then breaks the primary oracle `resolve_name("LINE")`.
Fix: use `BTreeMap<String /* ASCII-lowercased */, BTreeSet<CommandId>>`. `DuplicateAlias` fires only when one key maps
to two **different implemented** IDs (`first != second`). `Ambiguous` counts distinct implemented IDs. Add the
unit test `resolve_name("line")` on the real catalog == `OM9-CURVE-002` (not Ambiguous).

**M4. The Line `BothSides` phase contradicts the approved workspace contract.**
Where: 02 api.md §5 ("BothSides (B, Boolean, AnyStep) ... current behavior kept") vs
`skills/openmatrix9-workspace-contract/references/command-and-curve.md:38`
(`| Line | B, B=Y, B=N | BothSides before the first point |`). The design cites that row for the initials but
silently departs from its phase.
Fix: choose one and write it down. (a) `OptionPhase::BeforeFirstPoint` per the approved row, plus a negative oracle
`dispatch(LINE); point; submit("B") → Err(Invalid("option after first point"))` with `both_sides` unchanged. Existing
tests only set BothSides before the first point (`command_sessions.rs:81`, `layer_controller.rs:41`). (b) Keep
`AnyStep` and add "BothSides accepted after the first point (deviation from the approved row)" to the end-of-task
business-rule list in §0. Recommended: (a).

**M5. A second, parallel catalog owner stays.**
Where: `ui.rs:351` parses `include_str!("../assets/features.json")` into `Vec<Feature>` for menus. The new
`CommandRegistry::matrix9()` parses the same asset for dispatch. Neither the checklist nor the stub note retires the
UI copy, so names/IDs have two owners (criterion 2; L4 boundary "không nhân đôi backend trong menu/panel").
Fix: add stub-note §3 step 1b: "replace `MatrixApp.features` construction with
`CommandRegistry::matrix9()?.descriptors()` (filtered to catalog entries, same order as the asset)". Until then,
add a non-UI parity test in `tests/command_registry.rs`: an independent parse of the asset gives the same
`(id, name, command, module)` list as `registry.descriptors().filter(kind == Catalog or catalog-backed builtin)`.

**M6. Snapshot restore: identity and diagnostics semantics are unspecified (stable-ID contract).**
Where: 03 api.md `restore_snapshot` (candidate = `Self::new_with_held(..)`), `DocumentSnapshot::restore` →
`Archive::restore(bytes, limits, &{})`. Verified: `new_with_held` builds a new `TransactionCoordinator`, so the
`IdentityLedger` (`transaction.rs:529`) is rebuilt only from the restored world. IDs allocated after capture (Point B
in the oracle) leave the ledger. `restore.rs:153-184` renews the owner session and, with an empty replay set, marks
recipes `Unresolved` and returns diagnostics. `fingerprint(document)` (`file_lifecycle.rs:148`) includes
`owner_session`, so the kept `saved` hash makes the restored document dirty. None of this is stated.
Fix: add ADR-03-7: "restore_snapshot has File > Open identity semantics. The ledger is rebuilt from the restored world,
and every pre-restore SourceStamp/SelectionState/CommandContext is Stale because `owner_session` changes. Restore
diagnostics go to `file.diagnostics`, and the document is dirty after restore." Add these oracles:
`stamp captured before restore → document().resolve(&stamp) == Err(Stale("source stamp"))`;
`is_dirty() == true` after restore; a recipe in the snapshot comes back `Unresolved` and `file.diagnostics.len() == 1`.

**M7. The change-batch ring is not charged to the Controller budget.**
Where: 03 api.md `notify.rs` (`MAX_CHANGE_BATCHES = 64`, each `ChangeBatch.changes: Vec<Change>` sized to the diff).
02 §7 charges command events in `auxiliary_usage` (`session/layers.rs:194`), but 03 does not. Every live owner in this
codebase is charged (`scene_cache_usage`, `auxiliary_usage`, `retained_usage_with_held_usage` in `files.rs:356-369`).
An Undo of a large import keeps up to 64 × N `Change`s uncharged.
Fix: "`auxiliary_usage` adds `Usage::array::<ChangeBatch>(ring.capacity())` plus
`Usage::array::<Change>(b.changes.capacity())` per batch. Before pushing a batch, oldest batches are dropped until
`coordinator.retained_usage_with_held_usage(aux)` admits it. A single batch that alone exceeds the budget is replaced
by `ChangeBatch { changes: vec![Change::Context], .. }`, which tells the observer to rescan." Add the test
`tiny Limits → batch truncated to [Context]; commit still published`.

**M8. The 08-006 `params-roundtrip` oracle can be read as a new `Value` wire format (an M00 archive decision).**
Where: 08 oracles.json L3-006 `"recipe_parameters": {"Length": {"mm": 1.234}, "Angle": {"deg": 45}}`. api.md keeps
`Value::Quantity { value, dimension }`, but the fixture shows `LengthMm`/`AngleDeg` wire objects as parameter values.
Fix: rewrite the fixture as parameter-name → `Value` JSON:
`{"len": {"Quantity": {"value": 1.234, "dimension": "Length"}}, "ang": {"Quantity": {"value": 45.0, "dimension": "Angle"}}}`.
Add to api.md §4: "`Value`'s wire form is unchanged; `{"mm":..}`/`{"deg":..}` are the standalone `LengthMm`/`AngleDeg`
forms only and never appear in `.om9` metadata."

**M9. Stub note: the trigger is not observable and the RED ordering is inverted.**
Where: stub-integration-note.md header ("Applies AFTER (1) the concurrent UI workflow ... has finished") and §3 (the
frame test is step 3 but "RED before step 1"). Checklist 3.10 says "hand the stub note to the orchestrator (do not
apply it)", but no observable condition or owner step makes the swap happen. The swap is what actually removes the
L3-002/003 stub.
Fix: (a) Define the trigger: "UI workflow done = `.agents/tasks/m01-l3-003-panels/` contains its final verification/closeout
file and `git status --porcelain standalone/crates/om9-app/src/ui.rs src/workbench_regions.rs` shows no edits in
progress for 10 min, or the orchestrator receives that workflow's success message." (b) Reorder §3: add the frame
test first, run it, and save `implementation/evidence/M01/OM9-L3-01-01-002/stub-swap.red.log`. Then apply steps 1-2
and save `stub-swap.target-test.log`. (c) Name the step that applies it: a finalize item "F1 apply
stub-integration-note.md §3" in the checklist.

### NIT

- **N1.** Line drift: `auxiliary_usage` is `session/layers.rs:194` (cited :191); the features.json parse is `ui.rs:351`
  (cited :274); `refresh_region_plan` is `ui.rs:711` (cited :623-637); "duplicate display names (Boolean, ...)" is
  wrong because only `boolean builder` repeats. Fix the citations.
- **N2.** The 03 crosswalk L3-003 gap says refusal "by Scene capture". The oracles and 3.1 test validate/encode/prepare
  only. Drop Scene capture: a `World` cannot hold an invalid document, so it is unreachable.
- **N3.** Adding tolerance/groups/extensions to `new_layer_field` (`json_admission.rs:50-55`) reuses the error
  "legacy schema has no preset/Hidden fields" (`:202`). Specify `Invalid("legacy schema has no tolerance/group/extension fields")`
  for the new keys.
- **N4.** Under M00 option C, `DocumentSnapshot::capture` (via `container::encode`) also refuses documents with
  tolerance/groups/extensions. State it in 03 §4.
- **N5.** `Mutation::Extension` on an imported entity flips its retained record to `Edited`, because
  `retained.rs:493` hashes the whole entity, including `attributes_revision`. State this and add a test in
  `document_graph.rs`. Also state whether extensions on `Definition.members` are validated: recommend "validated,
  but not mutable in M01".
- **N6.** The 08 api.md `OptionDomain::validate` sentence about `min_inclusive` is garbled. Rewrite it as
  "`min <= max`; if `!min_inclusive` then `min < max`".
- **N7.** `Unit::parse` "µm": state that both U+00B5 and U+03BC are accepted.
- **N8.** Notify: "emit a batch only when `receipt.published` and the diff is non-empty" (`commit_with_guard` returns
  `published: false` for empty entries, `transaction.rs:1962`). Also state whether `ObserverId` is one counter
  shared by command and document observers.
- **N9.** Refusing Line/Point dispatch when `scene == None` (02 §4) changes behavior: today the session starts and
  `accept_world_point` fails with `Missing("command scene")`. Add it to the §0 business-rule list.
- **N10.** Items 2.7/3.9 have no evidence directory. Use `implementation/evidence/M01/OM9-L3-01-01-002/`.
- **N11.** With bare Numbers refused, `source_binding.rs:173-174` (`Number(NaN)`, `Number(0.)`) now fail for "missing
  unit", not for their original reasons. Add `Quantity{Length}` NaN and exclusive-min cases to that test.
- **N12.** Pruning a group member on Delete/Move: say whether `Group.revision` is bumped (this drives `GroupModified`).
- **N13.** The geometry-only exchange packet (`om9-exchange/src/lib.rs:168-178`, `export_document.rs:86-95`) drops
  groups/extensions silently. State that this is accepted M01 behavior (3DM not approved).

## Verified assumptions

- `serde_json = "=1.0.150"` is at `standalone/Cargo.toml:13`, and every crate uses `serde_json.workspace = true`.
  A feature flag does not change `Cargo.lock`.
- The Line-save failure path: `atomic_file.rs:74-75` compares `Archive::decode(&bytes) != *document` →
  `Invalid("archive encoding verification")`. The Line domain uses `hypot` (`session.rs:353-361`), so a non-exact
  float parse fails equality. The root cause (no `float_roundtrip`) is consistent with that, but the probe numbers
  were not re-run here.
- Session code citations hold: `session.rs:15-16, 30-38, 181-192, 206-226, 240-249, 262-264, 287-299, 318-330, 404, 409-417`;
  `files.rs:124-131, 375, 415, 426`; `replace_*` do `*self = candidate` (so carrying observers requires an explicit move).
- `CommandInputHost`/`ViewportSceneHost` (`workbench_regions.rs:554-571`), `compute_regions_for` checks the command
  host first (`:574-583`), and `RegionError::CommandInputUnavailable` displays "command input is not ready".
  `AttachedCmdFrame` (`ui.rs:83-89`) is constant `true` and is passed at `ui.rs:716`.
- The frame test is feasible: `MatrixApp::new(Document)`, `show`, `run_ui` pattern (`ui.rs:2163-2177`), and the public
  `controller/dock/status/region_plan/region_error` fields. `ui.rs` handles `scene() == None`. `session::hosts` can reach
  the private `scene` field and `refresh_scene`, because `hosts` is a descendant of `session`.
- `lib.rs` declares `pub mod workbench_regions`/`dock_layout`. `DockLayout::matrix_default`, `WindowSize{width,height}` are public.
- `features.json`: 607 entries, all IDs match `^OM9-[A-Z0-9]+(-[A-Z0-9]+)+$`, no `OM9-EDIT-*` IDs, `command` is null for 152
  entries, there are no Undo/Redo entries, File names are `New/Open/Save/Save As`, and duplicate commands include `dim`,
  `extrudecrv`, `gvrpsupportline`.
- Undo/Redo texts are `Missing("Undo entry")`/`Missing("Redo entry")` (`transaction.rs:2086, 2105`). The private `option()`
  is at `request.rs:368-399`, and the only `OptionDomain::Number` construction sites are `request.rs` and `source_binding.rs:165`.
- `Tolerance::validate` gives `Invalid("effective tolerance")` (`history.rs:21-30`), and `Mutation::Context` bumps
  `context_revision` (`transaction.rs:1883-1891`).
- `AllocatedIdentity`/`IdentityLedger` (`transaction.rs:477-583`) can take a `Group` variant. `Geometry` is externally
  tagged, so `{"Brep":{}}` yields a serde "unknown variant" message that `container.rs:264-272` can map.
- The legacy guard root is `Location::Document` only for legacy files (`json_admission.rs:272-276`). Schema-4 files start at `Unknown`.
- Retained baselines are unchanged for entities without extensions (`skip_serializing_if`). `EntityBaseline` hashes only
  geometry/placement/attributes/label.
- The DONE_WITH_EVIDENCE test citations exist with matching names (atomic_transactions, resource_transactions,
  transaction_scopes, history_replay, layer_transactions, ui_events, model_admission, container_admission, cold_reopen,
  layer_colors, world_picking, startup_layers).

## Unverified or wrong assumptions

- Wrong: the plan does not unblock `OM9-L4-01-02-001-05` (H1).
- Wrong: "Line BothSides AnyStep follows the approved row" (M4).
- Wrong citation: `ui.rs:274` for the catalog (it is `:351`), and `layers.rs:191` (it is `:194`) (N1).
- Unverified (no run allowed): the probe counts (15461/199999) and that `float_roundtrip` leaves every golden byte test
  unchanged. 8.1's guard covers this.
- Unverified: `Archive::decode` refuses a dangling entity layer with `Missing("entity layer")` (oracle 03-005). Decode
  calls `validate_manifest` first, and the path to `validate_entity` was not traced.
- Unverified: no transitive dependency enables `serde_json/arbitrary_precision` (features unify).
- Assumption accepted: the D4 stub swap needs no M00 decision (no external code reuse; no visible change in normal use).
- `OpenMatrix9_Codex_Spec_v2/` is absent (`Test-Path` false), as the design states.
