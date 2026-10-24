# OM9-L2-01-03 object attributes and query through SDK v1 — 2026-10-21

The second implementation slice of
[SDK-INTEGRATION-PLAN](../../OM9_Milestones/SDK-INTEGRATION-PLAN.md) is delivered inside
its existing owner, `OM9-L2-01-03` (Be Document Scene Entities). The document already
owned typed entities, attributes, revisions and the attribute/label mutations; this
slice adds the public adapter over those services and proves it from a plugin built
outside the checkout.

## Changed

| Layer | Change |
|---|---|
| Public DTOs | `ObjectInfo` gains `generation`, `geometry_revision`, `attributes_revision`; new `ObjectAttributes`, `AttributesPatch`, `ObjectQuery`, `ObjectPage`. |
| Public methods | `Host::query_objects`, `Host::object_attributes`, `Host::set_object_attributes`, `Host::rename_object`; four new `HostRequest` methods and capability entries. |
| Host dispatch | `crates/om9-plugin-host/src/staged.rs` stages one attribute record, one label and the query view, evaluates staged records with the document owner's permission rule and emits `Mutation::Attributes` / `Mutation::Label`. No mutable `Document` crosses the ABI. |
| Core service | `Document::permissions_for_attributes` extracted from `permissions_for_entity` so a staged record is evaluated by the same layer-chain rule as a published entity; `permissions_for_entity` now delegates. Behaviour unchanged (full core suite re-run). |
| Conflict guard | One command may stage only one of `Placement`, `Attributes` or `Label` per object. A second kind is refused with the `conflict` code instead of an opaque `conflicting source writes` failure at commit. |
| Example plugin | `AttributesDemo` edits one object's colour/lock and renames a second object in one transaction, then reads both records and the query back. |
| Tests | Host-level attribute staging/read-back and query tests, the refusal cases, the full core suite and an ignored native test against the real `.om9p`. |

## Semantics fixed by this slice

- `query_objects` returns objects ordered by object ID, includes this command's staged
  additions and attribute edits, supports `layer`, exact `geometry_kind`
  (`ObjectInfo.geometry_kind` values such as `"Point"`), case-sensitive
  `name_contains`, `only_selected` and `include_hidden`, and returns `total` with the
  page. A page is at most 1024 objects; `limit: 0` selects the host default of 256 and
  a larger request is a `budget` refusal, never a silent truncation.
- `object_attributes` reports stored flags and the effective flags after the layer chain,
  plus `generation`, `geometry_revision` and `attributes_revision`. A command's own
  staged edit is visible in later reads in the same invocation and the staged
  `attributes_revision` is the revision the commit will publish.
- `set_object_attributes` accepts `color` (`#rrggbb`/`#rrggbbaa`, empty clears to the
  layer colour), `visible`, `hidden`, `selectable` and `locked`. The object's layer must
  be editable; object-level lock/hidden stay editable so a plugin can unlock or unhide.
- `rename_object` applies the document label grammar (non-empty, trimmed, no control
  characters, within the name budget) and becomes a `Mutation::Label`.
- Mixing a transform, an attribute edit and a rename on the same object in one command
  is refused with `conflict`. That mirrors the application's typed property-edit model
  and the document transaction's one-source-write-per-object rule. A layer created by
  the same command cannot receive newly added objects; the transaction engine validates
  an insert against the captured world, so that flow needs two commands until that owner
  extends insert admission.
- Layer create/update/current/assignment remain unexposed. `ObjectInfo`/`ObjectAttributes`
  read layers, and object attribute edits are gated by the layer permission rule.

## Measured batch

Windows x64, rustc 1.98.1, `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`,
`om9-app --no-default-features` (standalone Controller/document/scene/archive path; no
OCCT adapter and no FreeCAD/Rhino host).

| Batch | Result | What it establishes |
|---|---|---|
| `cargo test -p om9-sdk -p om9-plugin-host -j 2` | 35 passed, 8 suites | SDK DTO/ABI contract and host dispatch, including attribute staging/read-back, query filters/paging, the refusal codes and the one-edit-per-object rule. |
| `cargo test -p om9-core -j 2` | 399 passed, 47 suites | The `permissions_for_attributes` extraction is behaviour-preserving across the document, layer, transaction and scene contracts. |
| `cargo test -p om9-app --no-default-features --lib session::plugins -j 2` | 4 passed, 135 filtered | The Controller publication/Undo boundary used by plugin commands still holds. |
| `cargo test -p om9-app --no-default-features --test plugin_native -j 2 -- --ignored` | 5 passed | A real `.om9p` DLL built from the example: `AttributesDemo` sets `color=#ff8800` and `locked=true` on the first object, renames the second to `SDK renamed`, returns `total=3`, bumps `attributes_revision` by one, keeps `effective_editable=false` for the locked object, and the change is exactly one Undo unit that restores the prior document on Undo and returns on Redo. |
| `python tools/validate_plugin_sdk.py --target-dir L:/om9-target --output build/standalone-validation/ui-artifacts/20261021-plugin-sdk-attributes` | PASS, 10 commands | Package/inspect, `CreateProfile` creating three objects, cold `--inspect` hash equality, cancel/unknown-command preserving the `.om9` bytes, and the SDK ZIP building its example offline outside the checkout. The native-tests step now runs all five native checks. |
| `cargo check -p om9-app -p om9-plugin-host -j 2` (default features, incl. `occt-brep`) | exit 0 | The product-feature build still compiles. |
| `python tools/public_source_audit.py` | see the JSON receipt | No private/commercial source or binary was introduced; only this record's Markdown/JSON increased the file count. |
| `rustfmt --check` on the edited Rust files; `git diff --check` | PASS | Formatting and tracked whitespace are clean. |

The [JSON receipt](2026-10-21-m01-03-attributes-sdk.json) records the commands, exit
codes, source hashes and artifact fingerprints; `observed.json` remains under the ignored
`build/standalone-validation/ui-artifacts/20261021-plugin-sdk-attributes/` run directory.

## Not proved here

- Only `OM9-L3-01-03-002/003` receive a scoped public adapter. Layer records/membership
  write, groups, blocks/instances, jewellery extension records, change notification,
  snapshot/restore and import provenance remain open.
- A layer created in the same command cannot receive new objects; layer
  create/update/current/assignment are not exposed yet.
- Geometry editing remains read-only in the SDK; only attributes and labels are written.
- macOS host and plugin execution are unverified; the SDK ZIP and package target support
  are structural only on this host.
- Interactive UI acceptance was not performed; this slice is a headless SDK/Controller
  and CLI proof.

## Next exact reference and action

Next document: [`sdk/API-INVENTORY.md`](../../sdk/API-INVENTORY.md) and
[`SDK-INTEGRATION-PLAN.md`](../../OM9_Milestones/SDK-INTEGRATION-PLAN.md). Next action:
expose the layer service (`LayerAction` create/update/set-current/assign) through the same
`Staged` dispatch, deciding first whether the transaction owner extends insert admission
so one command can create a layer and publish objects into it, or whether the documented
two-command flow stays.
