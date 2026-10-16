# Retained current-state writer ABI — schema 4

Status: implementation contract, not a native-write acceptance receipt. This
continues RCORE-00.M10; the full goal and M01–M12 remain open. Safe Rust owns
all business choices, UUIDs, indices, units, actions and publication. Selected
openNURBS C++ adapters own only strict SDK reading, serialization and validation.

## Request and private files

`om9-exchange-native --write-retained request.json private.3dm` reads a closed
JSON object with exactly these fields:

| Field | Meaning |
|---|---|
| `schema_version` | Integer 4 |
| `writer_policy` | `om9-retained-current-v1` |
| `metadata_policy` | `primary_archive` when a primary Preserve namespace exists; otherwise `new_mm` |
| `bounds_policy` | `cold_leaf_hull_v1` |
| `primary_namespace` | Primary namespace UUID or null |
| `model_mm_per_unit` | Rust-resolved positive finite output model factor; `new_mm` requires 1 |
| `document` | Correlation `id`, `name`, `page_unit`, plus the explicit `current_layer` policy below |
| `sources` | Complete namespace source catalog described below |
| `additions` | Complete current unbound hosts described below |
| `default_layer` | Native-only generated Layer addition or null; does not fabricate a host Layer |
| `table_order` | Component type number strings to ordered live output UUID arrays |

Each source file is a sibling `<namespace-uuid>.3dm` of `request.json`. Native
derives that filename from a strictly parsed canonical UUID, never accepts an
arbitrary source path in JSON. Rust materializes the immutable embedded source
resource with exclusive creation, syncs it, holds a deny-write/deny-delete read
handle across all native jobs, and hashes the actual held bytes. Every native
read again uses one held FILE for strict inventory, original class/attribute
owners, record/reference maps and final digest. Source external paths are not
required. Native validates its pinned V5/version50 boundary; no ONX_Model copy
or forgiving read/write becomes a retained preservation fallback.

`sources` entries have exactly `namespace`, `source_id`, `sha256` (lowercase hex),
`model_mm_per_unit`, `page_mm_per_unit`, `records`. Both factors are explicit
persisted Rust resolution contexts; None/Unset page declarations must not
silently lose their resolved context. Records include every original catalog key,
including deleted, omitted, retained-only and unbound orphan records. Each row:

- `source`: `native_component_type`, canonical `id`, nullable decoded `native_index`.
- `expected_wire`: original `id`, nullable `serialized_index`,
  `archive_reference_index`, `sdk_destination_index`, `table_type`, `record_type`,
  `offset`, `byte_length`, lowercase-hex `sha256`.
- `action`: `retain`, `delete`, `omit`, `regenerate`, `patch_entity`, `patch_layer`
  or `patch_definition`.
- `changed`: four booleans `geometry`, `placement`, `attributes`, `label`.
- `needs_cache_refresh`: independently derived Rust dependency-cache intent.
- `output`: null for delete/omit; otherwise the mapped output wire identity below.
- `definition_mm_per_unit`: resolved output-local factor for live definitions,
  null for other/deleted rows. Primary retained-only definitions keep their
  resolved original factor without becoming host-editable.
- `current`: null for retained-only rows; otherwise one tagged current payload.

Output identity is exactly `native_component_type`, `id`, `serialized_index`,
`archive_reference_index`, `sdk_destination_index`. These indices are distinct.
Layer wire indices append after the primary's actual serialized maximum, while
references/SDK slots are table ordinals. V5 definitions have no serialized index;
definition ordinal contexts follow all surviving source definitions. Geometry
has none of the three indices; object table ordinals are not component identities.

Additions/default Layer have exactly `output`, `needs_cache_refresh`, `current`.
The complete table order must agree with all live rows/additions exactly once.
Native cannot allocate replacement UUIDs, infer missing mappings, add defaults
or silently skip a row. Untouched nonselected primary tables and anonymous user
tables preserve their original complete table chunks and contexts.

## Current payloads

`current` is one tagged object:

- `Entity`: `id`, `label`, `geometry`, `placement`, `attributes`,
  `definition_member`. Geometry uses the existing Rust owned geometry schema,
  except `Instance` holds the mapped **native** definition UUID. All geometry
  length values are canonical millimeters. `placement` is the already decided
  final native frame: parent inverse scale for leaves, child/parent scale for
  instances, translation divided by the parent factor. Native applies it once
  and performs no further unit-policy scaling. Attributes are `layer_index`
  (mapped archive reference), `visible`, `selectable`, `locked`, `color`.
- `Layer`: `id`, `name`, mapped nullable `parent` UUID, `visible`, `selectable`,
  `locked`, `color`. Its serialized/ref/SDK indices come only from `output`.
- `Definition`: `id`, `label`, `mm_per_unit`, ordered mapped `members` UUIDs.
  Primary patch/cache preserves the original UnitSystem enum/custom name;
  supplied factor must match that original declaration or its explicit persisted
  unresolved-unit resolution. New/regenerated definitions initially require
  factor 1 and declare Millimeters. No implicit custom-unit declaration.

Unsupported native representation of independent Selectable, hidden/locked
combinations, channels, metadata or geometry must refuse explicitly. An unchanged
projection payload is not permission to replace a richer retained native master.

## Metadata and current layer

For `primary_archive`, original header/properties, model/page unit declarations,
tolerances and settings remain the primary archive's values. Document id/name/
page_unit are correlation fields, not claims of rewriting native metadata. An
unresolved original declaration stays unresolved; Rust's resolved numeric factor
is context and is not turned into a fabricated declared unit.

`document.current_layer` is exactly `policy`, nullable `id`, nullable
`archive_reference_index`:

- `preserve_primary`: both values null, requires primary_archive. Rust chooses
  this when the current OM9 host layer is None. Preserve original settings bytes.
- `explicit_output`: mapped live Layer UUID/reference. A mapped host current
  layer is preferred. For new_mm with host None, Rust chooses the generated
  native default, otherwise the first planned native Layer. This file policy
  does not change OM9 host current layer. Native never chooses the first layer.

Initial primary writer may accept explicit_output only when it equals the
original current Layer or a settings codec has been proven to update both V5
index and UUID without changing unrelated settings/userdata. Otherwise refuse.
Layer removal/reorder and opaque reference remapping remain closed gates until
settings/light/material/group/history/userdata dependencies have serializers.

## Preservation, caches and cold proof

Changes to UUID/index/frame/placement/geometry with opaque geometry or attribute
userdata require a proven relevant remap codec. A false flag is not certification.
Class DATA replacement preserves only the untouched raw surrounding chunks;
it does not certify arbitrary semantic remapping. Cache-only edits with unchanged
identity/xform must preserve raw userdata/attribute chunks exactly and admit any
length/offset/reference consequences. Copy-count-zero owner data must survive.

Dirty definition/instance caches include retained-only ancestors and root Irefs.
Cold geometry traversal ignores every old cached definition/Iref bbox and fully
composes instance matrices down to actual leaves. `cold_leaf_hull_v1` means a
conservative hull, not tight/exact CAD bounds: actual point/line/circle extrema,
positive-weight rational poles, mesh/cloud vertices and bounded native BRep
surface/vertex data. Unsupported/nonpositive/unbounded leaves refuse. Do not use
ON_InstanceRef.GetBBox, mesh/display tessellation or a source bbox as the oracle.

Writer output is exclusive/private, synced and never publishes a public path.
Fresh-process `--inspect-retained-current request.json private.3dm` uses strict
original-owner reading of the **output file**, checks current geometry/attributes/
graph/frames, original unchanged records/tables/chunks, missing/deleted/new rows,
and recomputes independent bounds before any Rust atomic publication. Embedded
original source stage files can serve as authenticated retained oracles, even
after the original external import files are unavailable.

Write acknowledgement fields: `schema_version` 4, `writer_policy`, `bounds_policy`,
`status` `written_retained_current`, `archive_version` 50, `objects`, `definitions`,
`layers`, nullable `primary_source_sha256`, `cold_validation_required` true,
`admission_required` true, `host_ready` false. A write acknowledgement alone does
not enable export capability. Cold acknowledgement uses status
`validated_retained_current`. Both envelopes add `request_sha256`,
`output_sha256`, `output_bytes`; the cold envelope adds numeric counters in
`checks` (`live_records`, `raw_records`, `raw_tables`, `opaque_chunks`, `bounds`)
and `actual` (`model_units`, `page_units`, `current_layer`, `objects`, `layers`,
`definitions`, `bounds`). Hashes cover the actual held request/output bytes.
The cold envelope also carries `raw_spans`: bounded actual output record and
top-level ranges, each with exactly `kind` (`record` or `top_level`), nullable
`native_component_type`, nullable `id`, `table_type`, nullable `record_type`,
`offset`, `byte_length`, lowercase `sha256`. All native component types are
included. Top-level type/id/record_type are null, start uses table_type 0, and
the exact EOF span is included. Rust streams the held output ranges to verify
these hashes and compares unchanged source records/tables against the admitted
original span hashes. Counters alone do not prove preservation.

Actual objects contain `id`, `native_component_type` 12, `native_kind`,
`native_class_uuid`, actual `wire` contexts, `label`, `is_definition_member`,
actual native `geometry` or null, `placement`, `attributes` (native inventory
with resolved layer UUID) and full actual `preservation` values. Unrepresented
native CAD remains null geometry with its actual class/kind; no source geometry
is substituted. Layers contain id/type5/kind/class/wire/name/parent/visible/
locked/color/preservation. Definitions contain id/type11/kind/class/wire/label/
members/units/linked/preservation. Units reuse the strict native-unit declaration
schema, preserving original None/Unset; definition units include enum/name/
nullable millimeters-per-unit. Actual current layer contains nullable id/archive
reference/SDK destination indices, read from output settings. Each bounds entry
contains id/type/`stored`/`computed` ([min,max] boxes) and `matched` boolean.
Rust still requires independent capture/frame/retained correspondence before
publication; native write/check acknowledgement alone is insufficient.

Rust inspection consumes `PrivateRetainedFile` and returns a sealed
`ValidatedRetainedFile` owning the same held output plus a fixed cold proof.
It reserves the complete current coordinator, prepared sources, wire plan,
staging, capture stamps, result and native-process buffers before launch. The
independent verifier receives only the remaining budget. Preparation, staging,
write, inspection and final caller cancellation stay joined; an unrepresentable
token union refuses before process/publication. The inspector executable hash
must equal the writer executable hash.

Publication requires exact acknowledgement of `RetainedExportLosses`: OM9
state/ownership is not encoded by this selected 3DM export, and the report counts
OM9 History recipes, results, resource containers and deliberately omitted native
records. Deleted records are current intent, not omissions. This does not claim
lossless OM9 persistence or full 3DM export capability. Publication is restricted
to the canonical staging parent, rechecks capture authority and source/request
hashes, then changes from the CRT-compatible read guard to a final
DELETE-capable `VerifiedFile` guard. That guard hashes the actual file again,
compares the sealed proof, and atomically renames the same handle. The complete
receipt is constructed before OS acceptance; no fallible receipt construction
is deferred until after replacement. Refusal leaves the public target intact.

## Implementation and validation sequence

1. Rust borrowed two-pass request encoder with complete mapped identities,
   cancellation and aggregate held-owner/packet allocation admission.
2. Strict native request validation, selected original-owner serializers and
   private current-state output. Refuse unproven operations without fallback.
3. Fresh output reader, authored stale-but-valid cache negatives, raw userdata/
   table/chunk comparisons, unit/instance correspondence and missing-source cases.
4. Rust owned staging/output handle, cold acknowledgement and atomic publication
   with same capture authority, cancellation, budget and failure guards.
5. Measure named batches; update public provenance/ledger with exact scoped
   receipts. No feature/M gate promotion from request encoding or compilation.
