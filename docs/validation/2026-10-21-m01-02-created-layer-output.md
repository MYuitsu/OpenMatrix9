# Output into a layer created by the same command — 2026-10-21

This closes the one gap the layer SDK slice recorded: a plugin command can now create a
layer, make it current, add objects into it, assign existing objects and read the whole
staged result back in one atomic transaction with one Undo unit. The work belongs to the
transaction owner (`OM9-L2-01-02`) and is proven through the public SDK at
`OM9-L2-01-03`.

## Changed

| Layer | Change |
|---|---|
| Transaction admission | `Mutation::Insert` validates the output layer against a lookup that resolves layers proposed earlier in the same request (`Document::layer_permissions_with`), so the proposed record and its captured parent chain decide editability instead of failing with `missing layer`. |
| Read/write guard | `crates/om9-core/src/layers.rs` gains `check_layer_reads_with` (the existing `check_layer_reads` delegates). The permission-read overlap check excludes only layer IDs this request creates, because they have no captured permission that could be stale. A layer whose flags change in the same request is still refused. |
| SDK / host | The host no longer refuses a `current_layer` that this command created; `ObjectAdd` uses the staged current layer and evaluates it with the same merged layer chain. |
| Example plugin | `LayerDemo` now creates `SDK Gems`, makes it current, adds one point into it, assigns an existing object, and returns the staged records. |
| Tests | Four transaction-level tests (create+insert one transaction and one Undo; created locked layer refused; created child under a locked captured parent refused; insert into a flag-changing layer still refused) plus the host and native SDK tests. |

## Semantics fixed by this slice

- Output into a layer created by the same request is admitted when the proposed layer
  record is editable along its parent chain. A locked proposed layer, or a proposed layer
  whose existing parent is locked, refuses the insert with `permission: output layer` and
  publishes nothing.
- The anti-bypass rule is unchanged for captured layers: an insert into a layer whose flags
  change in the same request is refused with `invalid layer read/write conflict within
  transaction`, because the permission would be read from state the request rewrites.
  Only layers this request creates are excluded from that overlap, and they are excluded
  because they have no captured state to read at all.
- The SDK now advertises the same behaviour: `layer_create(..., make_current: true)`
  followed by `add_point`/`add_line`/... in one command publishes the layer, the object and
  the current-layer choice as one transaction and one Undo unit.
- No new public ABI, DTO or capability name was added; this is an admission capability of
  an already-exported flow.

## Measured batch

Windows x64, rustc 1.98.1, `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`,
`om9-app --no-default-features` (standalone Controller/document/scene/archive path; no
OCCT adapter and no FreeCAD/Rhino host).

| Batch | Result | What it establishes |
|---|---|---|
| `cargo test -p om9-core -j 2` | 403 passed, 47 suites | The transaction engine change plus four new admission tests; no existing layer/document/Undo contract regressed. |
| `cargo test -p om9-sdk -p om9-plugin-host -j 2` | 38 passed, 8 suites | The readable create-then-add flow, the refusals, and the staged reads around it. |
| `cargo test -p om9-app --no-default-features --lib session::plugins -j 2` | 4 passed, 135 filtered | The Controller publication/Undo boundary still holds. |
| `cargo test -p om9-app --no-default-features --test plugin_native -j 2 -- --ignored` | 6 passed | A real `.om9p` DLL: `LayerDemo` creates the layer, adds a new point into it and assigns an existing object in one command; the document ends with four objects, the new object is on the created layer, and one Undo restores the pre-command document. |
| `python tools/validate_plugin_sdk.py --target-dir L:/om9-target --output build/standalone-validation/ui-artifacts/20261021-plugin-sdk-created-layer` | PASS, 10 commands | Package/inspect, `CreateProfile`, cold `--inspect` hash equality, cancel/unknown-command preserving the `.om9` bytes, SDK ZIP built here and its example compiled offline outside the checkout. |
| `cargo check -p om9-app -p om9-plugin-host -j 2` (default features, incl. `occt-brep`) | exit 0 | The product-feature build still compiles. |
| `python tools/public_source_audit.py` | 8787 source files, 511 icon bindings, 510 SVG, 0 errors | No private/commercial source or binary was introduced. |
| `rustfmt --check` on the edited Rust files; `git diff --check` | PASS | Formatting and tracked whitespace are clean. |

The [JSON receipt](2026-10-21-m01-02-created-layer-output.json) records commands, exit
codes, source hashes and artifact fingerprints; `observed.json` remains under the ignored
`build/standalone-validation/ui-artifacts/20261021-plugin-sdk-created-layer/` run directory.

## Not proved here

- Only the insert path into a proposed layer is admitted. `Mutation::Publish` (prepared
  operation results) and cross-document `Move` keep their existing base-layer validation.
- Layer delete/reparent/preset-slot editing, groups, blocks/instances, change notification,
  snapshot/restore and import provenance remain open.
- macOS host and plugin execution are unverified; interactive UI acceptance was not
  performed. No manual or M09 gate was promoted.

## Next exact reference and action

Next document: [`SDK-INTEGRATION-PLAN.md`](../../OM9_Milestones/SDK-INTEGRATION-PLAN.md).
Next action: take the next plan-named owner slice — `OM9-L2-01-12` file lifecycle or
`OM9-L3-01-08-004/005/006` typed builder parameters — with the same evidence path.
