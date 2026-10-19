# RCORE-10 retained exchange continuation

The full RCORE-00 objective stays active. Existing owned-current-geometry export
and the supplied Matrix9 interface are foundations, not the requested end state.
The previous turn made verified implementation progress. This continuation adds
the resource/schema substrate while preparing actual retained import/merge.

## Current resource/schema contract

Safe Rust `Document.resources` owns typed ResourceIds and immutable embedded byte
allocations. Serialized descriptors contain kind, name, exact byte length and
SHA-256; they are incomplete until matching payloads are hydrated. Runtime model
validation rejects an unhydrated descriptor. Clones share immutable bytes; names,
map storage, input/output copies and retained snapshots are admitted together.
Current retention accounting conservatively charges resources per distinct
document allocation; exact cross-snapshot resource allocation deduplication and
the global concurrent reservation pool are still required.

Schema 2 keeps the 16-byte header and 72-byte table entry. Chunk kind 1 is the
typed document manifest with the nil chunk UUID. Chunk kind 2 is an embedded
resource with its non-nil ResourceId and exact payload/hash. Zero-byte resources
are distinct from missing resources. Entries tile the payload area without
overlaps, gaps or trailing bytes; duplicate chunk IDs, unsupported flags/kinds,
noncanonical typed UUID map keys, absent references and unreferenced payloads
reject. Hydration budgets include the serialized input, table bookkeeping,
typed document and prospective resource bytes before resource allocation.

Schema 1 requires the explicit `Archive::migrate_from_v1` or
`Archive::upgrade_v1_in_place` path. A migration preserves durable IDs and adds
empty resources because no source provenance existed in that schema. The
developer CLI `om9 --migrate-v1 path.om9` uses `upgrade_v1_restored_in_place` to admit restoration before publishing; default inspect/open
does not silently upgrade. Runtime restoration still issues a fresh owner
session and separately gates History replay. Windows save uses the same held
VerifiedFile handle for final fingerprint and atomic rename. Missing-module diagnostics use borrowed module matching and admit both message copies plus traversal/session/input storage before clone or growth; overflow/budget failure preserves schema 1.

ResourceInsert/ResourceDelete participate in the existing global coordinator,
use the initial batch document revision, preserve payloads/IDs through one
Undo/Redo and retire ResourceIds independently of Undo/savepoint rollback.
Schema 3 now adds owned retained-source bindings and tombstones. Explicit
schema-1/schema-2 migration targets schema 3 and establishes empty provenance;
schema-2 resources retain their durable IDs, payloads, hashes and kinds. The
coordinator reconciles direct edits and solver publication into sticky source
dispositions; structural import bundles are atomic and use fresh namespace/host
IDs. Native-to-host admission and preservation merge remain to be implemented.

## Native source investigation (official pinned checkout)

Inspected openNURBS `eb92af3ba1806b0a34a99aba0d3bda83e3d46083`:

- `ONX_Model::Read(archive,0,0,log)` and incremental read APIs cover registered
  native objects/tables. Unregistered object class return code 3 may skip a
  record; generic successful Read is insufficient for complete preservation.
  Use a strict low-level completeness scan and reject skipped/normalized/failed
  records or missing table entries before accepting a merge.
- `ON_UnknownUserData` and `m_userdata_table` retain opaque bytes. Writer can
  continue after anonymous-user-table write failure, and ordinary CopyUserData
  skips records with nonpositive copycount. A true Write result or plain native
  Duplicate is therefore insufficient. Prove object ownership and cold payload
  correspondence with actual fixtures.
- `Manifest`, `OriginalToModelMap`, `ModelToOriginalMap`, `ON_ManifestMapItem`,
  `AddModelComponentForExperts` and `RemoveModelComponent` supply component
  operations. Automatic name/ID resolution and ignored reference-update failure
  must not silently change the admitted plan.
- Explicitly map instance-definition UUIDs/member lists, layer parent UUIDs,
  History object IDs, hatch pattern indices and annotation style IDs. Attributes
  can reset unresolved material/layer/group/linetype references and return false.
  Unknown plugin UUID/index relations have no generic correct remapping method.
- Use argument-bearing `MillimetersPerUnit(ON_DBL_QNAN)`/`MetersPerUnit` APIs;
  deprecated no-argument meter conversion is not correct for standard units, and
  enum-only custom-unit conversion is not a unit resolver. Original model/page,
  tolerance and definition units need independent fixtures.
- Real native inventory can supply supported Point/Line/full Circle/NURBS/
  Polycurve/Surface/Mesh/Cloud/Instance fields. Curve conversion return code 2
  does not establish identity parameter correspondence. Extra mesh/cloud fields
  and native CAD masters require explicit retained-only/edit restrictions.

These are verified source observations, not native retention/merge runtime
acceptance. They do not certify C++ memory safety or actual Rhino 5 opening.

## Implemented source inventory and parameter contract

The separate `--inventory-source` operation emits schema-2 owned JSON from a
strict native completeness scan. Registered source tables, native-local
UUID/index references, original model/page/definition units, tolerances,
userdata descriptors and exact geometry projections have actual native fixture
receipts. Unknown classes/tables, damaged CRCs, SDK recovery warnings,
incomplete endmarks, missing/cyclic dependencies and aggregate recursive curve
point budget overflow reject. Decoded SDK indices are explicit; immutable
source bytes remain authoritative. Unrepresented attributes and transitive
definition/unit dependencies remain host-admission requirements. This DTO does
not establish `editable_exact`, `host_ready`, native import or cold preservation.

Owned circles use native `domain` plus optional affine `angle_domain`; absence
preserves the existing radians contract. Native full circles keep phase and
parameter correspondence, proved with independent ON_ArcCurve D0–D3 values.
Exact full angular span is required for the current Circle export; near-full
and partial arcs remain explicit unsupported/retained-only source records.
Fractional evaluation composes angular maps before derivative powers. Scene
samples and picks retain reproducing native parameters; too-narrow f64 maps
emit a display/binding precision diagnostic without altering durable geometry.

## Remaining implementation sequence

The next implementation uses a document-owned retained source table and fresh
import namespace table. A source links immutable resource bytes/hash/version to
resolved original model/page units and tolerances. Namespace-local native keys
never become host IDs. An atomic import bundle carries resource, source,
namespace, entities, layers and definitions; missing/cyclic dependencies reject
before public mutation. Bound records keep a streaming host fingerprint and
Unchanged/Edited/Deleted disposition; retained-only records expose no fabricated
host geometry. A common candidate reconciliation handles direct edits and
PreparedResult::apply. Deleted keys remain tombstones through cold save, Undo
restores the original live binding, and unsupported cross-document transfers
reject before changing either owner. Source/namespace IDs remain retired after
Undo or caller rollback. This establishes merge inputs, not a merged writer.

1. Implemented structurally: document-owned SourceId, fresh ImportNamespaceId, source unit context,
   namespace-qualified native record keys, source-to-host bindings and explicit
   dispositions/tombstones. Extend all edit/delete/result/cross-document paths
   atomically; cross-document retained transfer remains gated until complete.
2. Strict native inventory DTO and completeness/projection receipts implemented;
   native-to-host admission and opaque payload preservation remain. Allocate
   a fresh host namespace for every import, including identical source bytes;
   prepare all tables/definitions/members/relations before one global commit.
3. Implement actual native current-state preservation merge: verify owned source
   hashes, map all admitted references, retain untouched masters/tables/payloads,
   overlay edited A/deleted B/new C, convert lengths once into chosen output
   units and refuse unsupported opaque reference/length remaps explicitly.
4. Verify cold native types, current geometry, absent tombstones, shared/nested
   graph, tables, units and userdata against independent sources before source
   revalidation/cancel and held-file publication. Add actual Rhino 5 open/edit
   evidence separately when an authorized accessible runtime surface exists.
5. Add structural OM9 clipboard, OS transport/focus and separately gated Rhino
   directions; finish archive geometry/material/annotation/page/camera resources,
   external-missing/relink policy, migrations and complete RCORE-00 M01–M12.

Mandatory named fixtures remain RCORE-10.T01–T12 at their full scope. Current
resource tests support only parts of T04/T08/T09; they do not replace T03/T05
namespace/tombstone/merge, T06/T07 clipboard or T10 target-application tests.
The total future native batch count is unknown. No 607-feature or 148-capability
completion status is promoted from these foundation slices.

## Next native-to-host admission implementation

Before the native job, capture the complete target epoch/runtime owner/context/
revision and make a held, hash-verified immutable source snapshot. Bind the
native receipt to the exact embedded source bytes; file length/endmark alone
does not establish this correspondence. Admit the typed DTO, held source,
native reservation, output/conversion maps, candidate clone, scene and Undo
storage together before allocation or launch.

Resolved mm/inch/custom projections already declare their canonical-mm basis;
do not scale them again. None/Unset currently emits null geometry, so a trusted
explicit factor needs actual native re-projection and a chosen-factor/basis
receipt while keeping original enum/name/tolerances. Model, page and used
definition units require separate policy. The existing JSON cannot be scaled
into geometry that it does not contain.

Map SDK component types through a closed numeric mapping. TextStyle maps to
the pinned SDK type 7, represented by `Other { native_type: 7 }`; never guess
from the native class label. Resolve UUID-only/index-only relations to each
component's complete canonical NativeRecordKey, including its actual decoded
index. Layer/definition projection arrays overlap the component inventory and
must not create duplicate source records. Current Core relations represent
source-component dependencies only. System/unset references and repeated
attachment/user-table identities require explicit retained diagnostics and
typed provenance extensions before full merge coverage can be claimed.

Build only used definition closure, with local members and shared/nested
instances. Unsupported masters, attributes and unit dependencies propagate to
dependent instances; unrelated valid definitions remain independent. A
per-field default/omission policy must decide whether a geometry projection can
become an admitted host master. Source completeness and geometric projection
alone do not make a record editable. Native sample-binding precision limits
must propagate to scene/editor admission.

Revalidate the original target capture before preparing one atomic import.
`Mutation::Import.expected_revision` alone covers only synchronous prepare;
the later PreparedTransaction epoch guard does not cover the earlier native
job interval. Actual fixtures must cover source mm/inch/custom/resolved-unitless,
duplicate imports, local members, current edit/delete/new, cancel/stale/Undo/
close and cold embedded-source restoration. Preservation merge remains a
separate writer with independently verified remaps and opaque-data policy.

## Actual source import implementation contract (current continuation)

`--inventory-source` keeps schema 2. A new `--project-source source.3dm
resolution.json` operation emits schema 3, status `source_projected`, policy
`om9-source-projection-v1`, an actual archive SHA-256 and chosen model/page/
definition unit receipts. Trusted schema-1 resolution input has optional model/
page factors and canonical native-definition UUID factors. Model/page unknown
units require explicit resolution; unresolved definition entries stay null and
retain dependent closure. Resolved local definition geometry and nested
placements use `S_parent * native_transform * inverse(S_definition)`. Numeric
SDK component kinds and per-record default/nondefault preservation fields are
explicit. No producer field grants host readiness.

Rust snapshots the original regular file into a private operation directory on
the supplied staging volume, hashes/admit bytes before copying, holds read-only
share-read handles while the native reader runs, and checks the returned hash
against the exact immutable Resource payload. It parses strict bounded typed
DTOs before constructing canonical key maps, fresh host/layer/definition IDs
and all source relations. Every import creates fresh SourceId/ImportNamespaceId;
native UUIDs stay provenance. Preserve mode admits supported geometry/attributes
with representable defaults and retains unsupported data; explicit geometry
mode reports each nonrepresented semantic omission while retaining original
bytes. Unsupported used-definition closure remains retained, with local
members kept out of top level and no partially fabricated definition.

ImportCapture binds coordinator incarnation, epoch, target runtime owner/session/
context/revision and caller scope lifecycle before work. Plain and scoped
captures are separate explicit APIs. Equal-counter coordinators cannot exchange
prepared transactions. Finalization revalidates the target, reserves retained
receipt/auxiliary storage with the candidate/Undo peak and prepares one global
Mutation::Import. Native failure, cancellation, stale target and bad unit/
dependency/budget data leave public state and executable Redo intact. Scene
preparation runs on the private candidate before publication and preserves exact
native parameter/subset binding or explicit display diagnostics.

Native producer and Rust consumer are implemented independently, then actual
native fixture imports test both unit modes, repeated identical source bytes,
shared/nested structure, nondefault/unsupported propagation, edit/delete/new,
scoped/cancel/stale/close paths and cold embedded source restoration. This
imports current native data; preservation export remains the next separate
operation and all complete chapter/foundation gates remain open.

## Current-state preservation overlay (2026-10-14)

Persist the actual Preserve/Geometry policy on every new import namespace.
Missing legacy policy remains unknown; it does not authorize restoring native
records. Bound geometry keeps independent streaming baselines for geometry,
placement, attributes and label as well as the complete original entity hash.
The importer installs these fields without losing resolved native relations.

Compile a bounded overlay from an immutable, operation-owned export capture:
unchanged Preserve records retain native owners, changed fields produce an
explicit patch, missing bound records delete, and unbound current objects,
layers, definitions and members become additions. Geometry-mode records are
regenerated from current host data or omitted; retained-only source records
never reappear. Revision-only changes that restore all original host fields
retain the native record. Missing edited-field baselines reject. Native
dependencies left unchanged cannot point to deleted records. Compiler refusal,
cancellation and stale ownership leave document state, Undo and destination
untouched.

This logical overlay precedes output UUID/index/unit policy and does not itself
certify native writing. Rust owns those maps and physical unit transforms. The
selected openNURBS adapter performs archive serialization and independent cold
inspection. Original geometry keys have no native index; object-table ordinals
are not identities. Indexed tables require explicit source-to-output maps.

Do not use a full ONX_Model copy/write as a preservation implementation: the
pinned SDK skips some read failures, omits copycount-zero userdata on copies,
regenerates RDK data and legacy text-style UUIDs, and can continue after user
table write failures. Investigate validated original table/chunk spans for
untouched primary legacy dim/text/user tables and public serializers for the
changed tables. Generic Read/Write3dmGoo does not establish nested CRC or v50
chunk safety. Verify actual A/edit, B/delete, C/new output plus unchanged native
wire data before extending the runtime capability. Full RCORE-10.T01–T12 and
RCORE-00 M01–M12 remain separate unfinished acceptance gates.

The Rust output intent now maps every namespace-local source key to one output
identity or explicit omission, and every live host root/member/layer/definition
to an output identity. Preserve primary keys/indices remain original, including
retired indexed slots when computing append indices. Secondary and Geometry
output identities are fresh and avoid source/host UUIDs. Multiple Preserve
namespaces require explicit primary selection; BTree UUID order is not import
chronology. Source geometry/history keys have no native index. Unknown SDK
component kinds and invented geometry ordinals reject.

Primary host-bound definitions retain their separately resolved local unit
factors. New/regenerated definitions use canonical millimeters. Rust supplies
`inverse(S_parent) * current_placement * S_child` for instances and
`inverse(S_parent) * current_placement` for other geometry. Independent physical
point tests include inches, local definition units, reflection and nonuniform
scale. This unit/identity intent does not admit opaque UUID/index/length remaps,
choose secondary document metadata disposition or certify SDK conversion.
Those are remaining writer/verification requirements. Field baselines loaded
from a document are not native provenance proof: the independent cold verifier
must compare actual output with the complete captured current state regardless
of patch flags, as well as original wire bytes for retained native records.

## Original-wire preparation and dependency caches (2026-10-14)

Schema4 read-only `--prepare-retained-source source.3dm` now reports the exact
source hash, original model/page and definition unit declarations, complete
record/start/table/end spans and original attachment metadata. The selected
wire codec admits archive version50. The native strict scanner/reference map
and original native owners read the same held FILE; a different path cannot
contribute another source's units or identities. Rust stages exact immutable
Resource bytes, holds a read-only share-read File, admits the process with the
document/Undo/external-input peak, and verifies all spans and catalog keys before
accepting typed preparation. The publication guard has DELETE access for its
atomic rename; preparation uses a separate read-only owner so the CRT reader can
share it. Source staging is cleaned on success, refusal and cancellation.

Decoded source keys, serialized component indices, archive reference indices
and SDK destination indices remain distinct. The authored sparse Layer is wire7,
reference1 and decoded/destination1; Group is wire/reference5 and decoded/
destination0; Light has no decoded source key index, wire/destination7 and
reference0. V5 definition records have no serialized index and use UUID
references. Geometry/history indices are absent. The current OutputPlan remains
a decoded logical intent; it must not be serialized directly as the wire map.
Allocate actual output table/reference identities from admitted original spans
and final record ordering. Deleting an earlier Layer can shift reference slots
and therefore requires explicit closed remapping or refusal for opaque records.

Attachment DTOs distinguish known native data from unknown payloads. Copy count
zero does not authorize omission; original legacy V5x DimStyle attachments may
survive on the wire owner while the converted decoded DimStyle loses them.
Conservative preservation flags add admission gates for opaque native fields,
compound owners, extra chunks and userdata. A false flag grants no blanket
permission to remap identities, indices or lengths. Secondary source metadata
and unresolved retained-only definition contexts still need explicit policy.

The bounded Rust dependency graph marks every affected definition ancestor and
Iref for cache refresh after member geometry/placement changes or deletion.
It crosses retained-only ancestors while leaving unrelated definitions clean.
An Iref's own geometry/placement patch or regeneration also refreshes its cache
when its child definition remains clean. Raw replay is eligible only for Retain
with needs_cache_refresh=false. The native writer must recalculate actual bounds
from current members and resolved local units; Iref.GetBBox's cached value is
not an independent oracle. Selected class-data serialization must retain
original userdata bytes for cache-only changes, followed by a cold geometry/
bounds oracle independent of the action flags.

Measured preparation inputs: three Rust tests, thirteen authored files, ten
source scenarios and thirty-two owned native jobs, including nine preparations.
All final native preparation/compatibility/infrastructure batches total317
executions/123 expected rejections. See the dated retained input and
infrastructure receipts. These counts certify the stated slices; full merge,
native cache serialization, Rhino5 and all RCORE-00 gates remain open.
