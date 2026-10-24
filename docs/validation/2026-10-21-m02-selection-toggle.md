# Selection remove and toggle through SDK v1 — 2026-10-21

This completes the add/remove/toggle scope of `OM9-L3-01-07-002` (Object Selection Set Core)
for the plugin boundary: the set can now be replaced, extended, reduced and toggled by
persistent object IDs, with the Controller's scene-root validation before commit.

## Changed

| Layer | Change |
|---|---|
| Public SDK | `Host::remove_from_selection(ids)` and `Host::toggle_selection(ids)`, plus `SelectionRemove`/`SelectionToggle` requests and capability entries. |
| Host | One staging routine with four modes. Joining the set requires a canonical id, a live object and selectable effective permissions; leaving it never needs selectability, so a stale or newly locked entry can still be dropped. Removing an unselected id is an accepted no-op. |
| App | Unchanged from the previous slice: the staged set is validated against the scene that will be published before commit, and applied after the command succeeds. |
| Example plugin | `SelectionDemo` accepts `{"remove":true}` and `{"toggle":true}` in addition to set/add/invalid. |
| Tests | Host test for toggle on/off, remove and the no-op removal; refusal additions for toggling a locked root and a malformed remove id; native DLL test extended to toggle and remove. |

## Semantics fixed by this slice

- `toggle_selection` removes a selected id and joins an unselected one; joining follows the
  add rule, leaving does not.
- `remove_from_selection` is idempotent for unselected ids and only requires a canonical
  object UUID.
- Selection remains outside document state: no transaction, no Undo entry, and a refused
  command leaves the previous selection untouched.

## Measured batch

Windows x64, rustc 1.98.1, `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`,
`om9-app --no-default-features` (standalone Controller/document/scene/archive path; no
OCCT adapter and no FreeCAD/Rhino host).

| Batch | Result | What it establishes |
|---|---|---|
| `cargo test -p om9-sdk -p om9-plugin-host -j 2` | 46 passed, 8 suites | Toggle/remove staging, the no-op removal, the locked-toggle refusal and every existing host contract. |
| `cargo test -p om9-core -j 2` | 403 passed, 47 suites | No core change. |
| `cargo test -p om9-app --no-default-features --lib session::plugins -j 2` | 5 passed, 135 filtered | Pre-publication validation still holds. |
| `cargo test -p om9-app --no-default-features --test plugin_native -j 2 -- --ignored` | 9 passed | A real `.om9p` DLL: select two roots, add keeps them, toggle drops one, remove drops the other, and the invalid case still refuses without touching document, Undo or selection. |
| `python tools/validate_plugin_sdk.py --target-dir L:/om9-target --output build/standalone-validation/ui-artifacts/20261021-plugin-sdk-selection-toggle` | PASS, 10 commands | Package/inspect, `CreateProfile`, cold `--inspect` hash equality, cancel/unknown-command preserving the `.om9` bytes, SDK ZIP built here and its example compiled offline outside the checkout. |
| `cargo check -p om9-app -p om9-plugin-host -j 2` (default features, incl. `occt-brep`) | exit 0 | The product-feature build still compiles. |
| `python tools/public_source_audit.py` | 8795 source files, 511 icon bindings, 510 SVG, 0 errors | No private/commercial source or binary was introduced. |
| `rustfmt --check` on the edited Rust files; `git diff --check` | PASS | Formatting and tracked whitespace are clean. |

The [JSON receipt](2026-10-21-m02-selection-toggle.json) records commands, exit codes, source
hashes and artifact fingerprints; `observed.json` remains under the ignored
`build/standalone-validation/ui-artifacts/20261021-plugin-sdk-selection-toggle/` run
directory.

## Not proved here

- `OM9-L3-01-07-002` is now covered on the SDK side for replace/add/remove/toggle by
  persistent IDs. Typed subobject/occurrence refs, pick queries, window/crossing selection,
  drag transactions and highlighting remain at their owners.
- Selection is still not document state and does not participate in Undo/Redo.
- macOS host and plugin execution are unverified; interactive UI acceptance was not
  performed. No manual or M09 gate was promoted.

## Next exact reference and action

Next document: [`SDK-INTEGRATION-PLAN.md`](../../OM9_Milestones/SDK-INTEGRATION-PLAN.md).
Next action: take the next plan-named owner slice — `OM9-L2-01-12` file lifecycle (host file
service with atomic save/open and no document swap inside a command) or the pick/query work
at M02 `01-07`.
