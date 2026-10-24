# Selection write through SDK v1 (OM9-L3-01-07-002) — 2026-10-21

A plugin command can now replace or extend the selection through the Controller session.
The selection set itself, its revision and the scene-root validation belong to the M02
selection owner; this slice exports them and keeps the rule that selection is not document
state.

## Changed

| Layer | Change |
|---|---|
| Public SDK | `Host::set_selection(ids)` (replace, empty clears), `Host::add_to_selection(ids)`; two new `HostRequest` methods and capability entries. `Host::selected_objects` now returns the command's staged selection. |
| Host | `Staged` keeps an optional staged selection: each ID must be a canonical, non-deleted object whose effective permissions are selectable, the count is bounded by `max_outputs`, and the staged set is read back by `selected_objects`. `PluginRun` returns the staged selection. |
| App | `publish_plugin_request` now takes the staged selection and validates it against the scene it is about to publish (`root == id && selectable && has_pick_geometry`, the same rule `select_objects` uses) **before** reserving/committing, so a bad ID refuses the whole command instead of committing and then failing. A command that changes no document state applies its selection through the same validation. |
| Example plugin | `SelectionDemo` selects the first two objects, can extend with `{"add":true}` and can request an unknown canonical root with `{"invalid":true}`. |
| Tests | Host tests for staged/read-back selection and every refusal; a Controller unit test proving pre-publication validation; a native DLL test for the set/add/refused flows. |

## Semantics fixed by this slice

- Selection is not document state: a selection-only command produces no transaction and no
  Undo entry, and a refused command leaves the previous selection unchanged.
- Every ID must be a canonical, non-nil, non-deleted object with selectable effective
  permissions (layer chain included) in the host; the Controller then checks it against the
  scene root the command will publish (`selectable` and pick geometry) before committing.
- `set_selection` replaces the whole set (an empty list clears it); `add_to_selection` keeps
  the entries already present. A selection deleted by the same command is pruned before
  publication.
- `selected_objects` reads the staged selection, so a plugin can implement "select what I
  created" in the same command.
- Subobject refs, pick queries, window/crossing selection and drag transactions remain open.

## Measured batch

Windows x64, rustc 1.98.1, `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`,
`om9-app --no-default-features` (standalone Controller/document/scene/archive path; no
OCCT adapter and no FreeCAD/Rhino host).

| Batch | Result | What it establishes |
|---|---|---|
| `cargo test -p om9-sdk -p om9-plugin-host -j 2` | 45 passed, 8 suites | Staged selection, read-back, and the id/missing/permission/deleted refusals, on top of every existing host contract. |
| `cargo test -p om9-core -j 2` | 403 passed, 47 suites | No core change; document, layer and transaction contracts are unchanged. |
| `cargo test -p om9-app --no-default-features --lib session::plugins -j 2` | 5 passed, 135 filtered | Pre-publication selection validation refuses an unknown root without committing or changing the selection. |
| `cargo test -p om9-app --no-default-features --test plugin_native -j 2 -- --ignored` | 9 passed | A real `.om9p` DLL: `SelectionDemo` selects two roots, `add` keeps them, and an unknown canonical root refuses the command while the document, Undo length and selection stay unchanged. |
| `python tools/validate_plugin_sdk.py --target-dir L:/om9-target --output build/standalone-validation/ui-artifacts/20261021-plugin-sdk-selection` | PASS, 10 commands | Package/inspect, `CreateProfile`, cold `--inspect` hash equality, cancel/unknown-command preserving the `.om9` bytes, SDK ZIP built here and its example compiled offline outside the checkout. |
| `cargo check -p om9-app -p om9-plugin-host -j 2` (default features, incl. `occt-brep`) | exit 0 | The product-feature build still compiles. |
| `python tools/public_source_audit.py` | 8793 source files, 511 icon bindings, 510 SVG, 0 errors | No private/commercial source or binary was introduced. |
| `rustfmt --check` on the edited Rust files; `git diff --check` | PASS | Formatting and tracked whitespace are clean. |

The [JSON receipt](2026-10-21-m02-selection-sdk.json) records commands, exit codes, source
hashes and artifact fingerprints; `observed.json` remains under the ignored
`build/standalone-validation/ui-artifacts/20261021-plugin-sdk-selection/` run directory.

## Not proved here

- `OM9-L3-01-07-002` asks for add/remove/toggle. Replace and add are exposed; remove and
  toggle are not, and subobject refs, pick queries and window/crossing selection remain at
  their owners.
- The selection is validated against the owner's published scene rule; it is not a
  document change and does not participate in Undo/Redo.
- macOS host and plugin execution are unverified; interactive UI acceptance (rubber band,
  hover/pick) was not performed. No manual or M09 gate was promoted.

## Next exact reference and action

Next document: [`SDK-INTEGRATION-PLAN.md`](../../OM9_Milestones/SDK-INTEGRATION-PLAN.md).
Next action: complete the selection L3 (remove/toggle) or take the next plan-named owner
slice — `OM9-L2-01-12` file lifecycle (host file service with atomic save/open and no
document swap inside a command).
