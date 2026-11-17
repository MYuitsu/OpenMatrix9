# Canonical representation transfer — integration in progress

The OCCT topology capture can now transfer a freshly rebuilt display to an
existing canonical B-rep after matching the process-local native attestation.
The core transfer checks the mapping and adjacency, remaps face, edge and
vertex display indices to the canonical topology, and offers an independently
accounted display owner. Structural mapping checks are not native attestation.

`CARGO_TARGET_DIR=L:\om9-target cargo check --workspace -j 2` passed on the
current dirty workspace on 2026-11-14. This is compilation evidence only. No
tests, command runs, benchmarks or runtime checks were performed under the
current plan's implementation-only direction.

The asynchronous history path now locates retained proof for root OCCT B-reps,
rebuilds missing displays on its worker under reservations, and prepares the
scene before its existing guarded publication. `cargo check -p om9-app -j 2`
passed after this integration. This is compile evidence only, not evidence that
the rebuild or its cancellation, quota, picking or publication works at runtime.
The history candidate now also owns rollback records for newly rebuilt entries;
abandoning it removes its exact entries, while successful publication retains
them. The entry and rollback metadata have separate managed reservations. This
extension passed `cargo check --workspace -j 2` and has not been run.

The OCCT representation store now keys a known profile by linear and angular
deflection plus an adapter version. A display without known settings uses its
own display identity instead of colliding with another unspecified profile.
History rebuilds record their explicit profile; not all producers yet carry
known profile metadata. `cargo check --workspace -j 2` passed after the change;
profile behavior is not runtime verified.

## Builder and registered history handoff in progress

The producer registry now carries transient native candidates alongside
canonical outputs for OCCT ring and sweep producers. Synchronous Builder
previews retain the candidate for drawing; Apply estimates the scene from that
candidate and stages its display through transaction-owned cleanup. Registered
history recomputation similarly carries candidates into scene preparation and
keeps its recompute cache private until publication. Callers without a display
handoff explicitly refuse a B-rep candidate. `cargo check --workspace -j 2`
passed. These synchronous producer allocations are not admitted before the
native operation, and preview lifetime accounting, cancellation, geometry and
UI behavior have not been verified. This does not close the Builder or C4 work.
The application also compiles with `cargo check -p om9-app --no-default-features
-j 2` using the same external target; this is a feature-configuration check,
not native or runtime evidence.

## Opaque native representation in progress

The two ownership descriptions below describe earlier intermediate states and
are superseded by the source-only document variant recorded at the end.

Opaque native import now stages an independently reserved representation tied
to the exact payload allocation and original display, with a scene pin and
transaction rollback. It is not eligible for live-source eviction: no native
rebuild disposition has been certified. Orphan cleanup only retires an entry
after the source and display readers are gone. The document still owns the
same display allocation, so its charge overlaps conservatively and this is not
source-only document ownership. `cargo check --workspace -j 2` passed; no
runtime, error-path, pin-lifetime or allocation measurements were run.

Open, history block members, unattested legacy and native payloads still need
their own safe disposition and integration. Cache reclamation, canonical
NativeBrep, v4 migration, v5, I1 and affected consumers remain open. The
synchronous controller Undo/Redo path has not gained cold rebuild. No M09 gate
is claimed.

## Native source ownership in progress

`NativeBrepSource` now owns only the opaque payload and supplied CRC. Existing
tracked native source leases and Undo-memory accounting use that value, and an
adapter result can transfer its source and display separately without copying.
This does not certify the payload or prove it can regenerate a display. The
document's `NativeBrep` still owns its display, and no native cache, v4/v5
migration, or native source-only restoration is claimed. A subsequent
`CARGO_TARGET_DIR=L:\om9-target cargo check --workspace -j 2` passed; no runtime
checks were run.

The 3DM import scene preflight now reads the coordinator's representation store.
Editable conversion has an admitted source/meshing interface and carries its
prepared display into that store with a separate reservation before scene
preflight. Budget, overflow and cancellation errors do not silently become an
opaque fallback. A transaction-owned rollback list removes exact candidate
entries on abandoned preparation or failed commit and keeps them on successful
unscoped publication. Cache nodes and rollback metadata have reservations.
The workspace compile check passed after this change. Runtime behavior, cold
rebuild and general cache reclamation are not established; editable import is
not claimed complete.

## Native document source split and archive guard

`Geometry::NativeBrep` now retains the opaque payload and CRC only. The import
adapter separates the display into the reserved representation store, and scene
objects retain its pin. Opaque imports without a display or certified rebuild
are refused; this does not establish that embedded payload caches are stripped.
Native source topology and bounds consumers currently refuse the operation rather
than deriving exact answers from the display. Those capabilities require source
adapters before C4 can close.

The v4 encoder now refuses documents containing OCCT or native BReps, including
definition members and result snapshots, so it cannot label the new schema as
v4. Existing v4 BRep migration, v5, source-only Open, complete Undo/Redo cold
rebuild, general eviction, wireframe/isocurve and remaining consumers are still
open. The external-target workspace and `om9-app --no-default-features` compile
checks passed; no tests or runtime validation were run. The native exchange adapter also compiled in the existing
`L:/om9-target/p1-native-exchange` Release tree with MSBuild `/m:2`; it was not
run or installed into the active app layout. No milestone or release gate is
claimed.

## Staged representation preference

When a transaction stages a second OCCT display profile for a source that
already has a preferred representation, insertion now preserves the existing
preference. Removing a preferred profile selects another retained profile if
one exists. This
avoids losing an existing preference on rollback. A missing source still uses
its staged entry for private candidate-scene construction, so this is not a
general transaction-private cache or a complete eviction policy. The workspace
and core compile checks passed against `L:/om9-target` with `-j 2`; no runtime
or tests were run. Cold restore, attestation and cache reclamation remain open.

## Explicit v4 migration candidate

The archive crate now has a separate typed v4 decoder that retains old OCCT
and native displays alongside their exact transferred sources, including
definition members and result snapshots. It applies the existing archive
structural preflight and chunk validation, admits the split's collection
overlap, checks legacy display integrity and hydrates resources. This is a
private candidate API, not an Open path or v5 codec: the caller must keep and
charge the returned displays, handle topology attestation and build a scene
before publication. Opaque native payloads still have no proven canonical mesh
stripping disposition. The initial decoder work was not connected to Open;
the subsequent controller handoff below supersedes that limitation.

`CARGO_TARGET_DIR=L:/om9-target cargo check --workspace -j 2` and a subsequent
`cargo check -p om9-archive -j 2` passed. No test,
fixture, benchmark or runtime Open was run; compatibility and memory peaks
remain unverified. The v4 save guard remains in force.

## V4 Open candidate handoff

The controller Open path now reads the typed v4 candidate, joins the private
candidate coordinator to the existing account before scene capture, and
transfers the old displays into separately charged representation owners.
It checks each display source belongs to the candidate world, plans the scene
with those representations, and keeps the prior controller published until
the replacement admission and scene preparation complete. Normal non-BRep v4
input continues through the ordinary private-account path. Workspace and app
without default features compile checks passed with the external target and `-j 2`; no Open, cancellation,
memory-pressure or compatibility test was run. The exact accounting under
real workloads and native topology attestation remain unverified. At this
checkpoint Open was synchronous; the later UI worker section supersedes that
for the picker path. Cold source-only restore and v5 are not implemented, and v4 BRep
saving remains refused. A legacy native display without vertices or faces is
explicitly refused because no verified source-only rebuild is available; the
original archive is not changed.

## Source-dead representation reclamation

OCCT representations now carry a weak source-lifetime receipt. On a managed
memory refusal, admission can retire an unpinned OCCT or native entry only if
its source has no strong owners; it releases the entry after dropping the cache
lock and retries admission. The blind API that discarded all unpinned OCCT
displays was removed. Source-receipt allocation overhead is included in the
separately charged import, history, Builder and v4 handoff paths.

That source-dead path did not evict a live or history-owned source; the later
attested live-source eviction section supersedes that limitation for scoped
current-session proofs. It does not complete the requested cache policy. Some older producer paths still need
their ownership accounting reconciled. The workspace compile check passed; no
runtime, pressure or pin-lifetime tests were run.

## V4 UI Open worker

The native-picker Open path now sends v4 file decode, retained-display handoff,
and candidate scene preparation to a worker. The UI keeps the old world and
scene published, accepts Escape as cancellation, and checks the file token,
scene publication, and selection revisions before transferring the prepared
candidate. Worker-held reservations survive cancellation until the worker
actually exits; its returned candidate is discarded if no receiver remains.
The controller's direct/batch Open API remains synchronous. Cancellation is
checked at admission and preparation boundaries; an individual blocking file
read or native call is not made interruptible by this change.

Both default and no-default-features app compile checks passed with the external
target and `-j 2`. No runtime Open, cancellation, stale-result, compatibility,
or memory-pressure tests were run. V5, source-only cache misses, block-member
attestation and the remaining C2–C4 work are still open.

## History rebuild interior wires

For OCCT root sources rebuilt by the history worker from retained topology
evidence, the worker now prepares the same source-derived interior wire buffers
used by remeshing and attaches them to its private candidate scene. The buffer,
display, snapshot metadata and prior shared-buffer owner retain separate
reservations through publication or refusal. Validation scratch for materializing
the rebuilt representation is admitted separately. Multiple root occurrences
in the rebuild job each receive a prepared buffer.

This only covers roots in the existing attested cache-miss job; it does not add
proofs for legacy or block members, rebuild native payloads, or establish
wireframe parity for other scene creation paths. The workspace compile check
passed using the external target and `-j 2`; no runtime, cancellation, geometry,
memory-pressure or command tests were run.

## Shared-source definition history rebuild

The history candidate can now locate an existing in-process topology proof for
a definition member, or another root, when the candidate uses the exact same
immutable source and topology allocations as that proof. The worker rechecks
the proof's account and original session binding before rebuilding. After scene
capture, it prepares source-derived wires for matching nested occurrences using
their root stamps and complete member paths. Unused definitions without proof
do not by themselves force a rebuild; a displayed cache miss without a proof
still refuses scene preparation.

This does not attest independently decoded legacy sources or arbitrary equal
text. Wire buffers are currently prepared per occurrence rather than shared; the
general cache, source-only Open, v5 and native rebuild remain open. The app and
workspace compile checks passed with the external target and `-j 2`; no runtime,
geometry, cancellation, or pressure tests were run.

## Reserved primitive transfer

The reserved primitive insertion path now splits an exclusive admitted kernel
result into a source reservation and an independently charged display/store
owner before staging the transaction. The transaction accepts the source's one
weak cache receipt only when the store confirms that receipt and its account
charge. Sphere and the optional-topology primitive path use this transfer;
failed insertion prunes a source-dead entry after display readers leave.

This addresses an integration mismatch introduced by the source/display split:
the former combined reservation no longer matched the source-only transaction,
and the cache's weak receipt defeated the old exclusivity check. The workspace
compile check passed with `L:/om9-target` and `-j 2`. No primitive command,
failure-path, memory-pressure or runtime tests were run. Other producer paths
and pre-admission of native allocations remain open.

## Attested live-source eviction

The in-process topology-proof cache can now retire an unpinned, separately
charged OCCT representation used only outside the current document while retaining its source-only proof. Current-document
entities, definition members and result snapshots remain protected even if an
unused definition has no scene pin. It drops
the display owner outside cache locks and keeps the proof until the source no
longer has another strong owner. A history preparation guard prevents the proof
cache from evicting entries between candidate planning and scene pinning.
Synchronous Undo/Redo callers retry through the history worker if scene capture
reports this cache miss; the UI already uses the worker directly.

This applies only where a current-session proof and an exclusively owned,
accounted display are available. Opaque native sources, independently decoded
legacy sources, unaccounted producer displays, durable proof and per-entry
recency remain open. Workspace and no-default-features app compile checks passed
using `L:/om9-target` and `-j 2`; no runtime, concurrency, memory-pressure,
Undo/Redo or geometry tests were run. Native allocation bounds are not proved.


## History representation recency

The proof cache records successful proof lookups and insertions, then selects the least recently used eligible history source for representation reclamation. Multiple proofs for one source use their most recent access; live current-document sources, pinned displays, and sources without retained rebuild evidence remain excluded. The counter resets ordering only on integer wrap. This is compile-only evidence: no runtime, eviction-order, concurrency, or memory-pressure checks were run. The workspace compile passed with L:/om9-target and -j 2. Durable topology binding, source-only Open, v5 archive disposition, native source handling, other producers, and complete runtime validation remain open.

## Occurrence wire extraction and source queries

ExtractWireframe now inspects the published OCCT B-rep occurrence buffers for
block members as well as roots. When those buffers contain interior wires, the
command extracts source-derived edges and default face isocurves with the
occurrence placement. Matching uses root, leaf and composed placement; if
coincident matches disagree on the displayed profile the command refuses the
ambiguous result. Opaque native members still use the existing edge-only path;
full native disposition and occurrence identity remain open. Trim's temporary
partition distance queries now use canonical source without registering their
temporary display in the representation store.

Workspace compilation passed with L:/om9-target and -j 2. No command, geometry,
block, ambiguity, runtime or memory-pressure checks were run. This does not
establish full wireframe parity or completion of I1.

## V5 container and versioned Open in progress

The current container writes version 5 for documents without OCCT or native
B-reps, using the existing bounded serializer and atomic save verification.
Application Open and the inspection command dispatch between v4 migration
candidates, which retain legacy displays, and current v5 metadata. The
explicit v4-only migration API remains available; old schema versions are not
silently accepted. The document-only decoder accepts v4 without B-reps and
refuses v4 B-reps instead of dropping their displays; display-aware callers use
the candidate API. Legacy v1-v3 migration now reports its actual output
version. B-rep save remains explicitly refused: durable topology attestation,
source-only Open reconstruction, and native opaque-payload disposition are not
implemented. This is not complete C3 or evidence of v4/v5 roundtrips.

The workspace compiled with L:/om9-target and -j 2. No save, Open, migration,
failure, geometry, memory-pressure or runtime tests were run in this batch.

## Exact occurrence wire binding and opaque refusal

The previous root/leaf/placement matching is superseded: ExtractWireframe now
follows the complete instance path to the exact published OCCT occurrence and
checks its buffers before choosing source wires. Path storage is admitted and
retained during extraction. NativeBrep ExtractWireframe now explicitly refuses
unverified face/trim binding instead of returning an edge-only result; the
existing source remains untouched because the single output transaction has
not started. This is a partial I1 integration, not proof of parity or native
face support. Workspace compilation passed with L:/om9-target and -j 2; no
command, geometry, memory-pressure, or runtime tests were run.

## Exclusive producer display accounting

The ordinary B-rep candidate registration path now reserves an already-built,
exclusive, unowned candidate and uses the existing split-owner handoff. Its
display and store metadata retain a separate cache charge after registration;
source-dead, unpinned entries are pruned before the next registration. The
candidate source's temporary reservation is released on return and ordinary
transaction staging continues to account the source. Already-owned or shared
candidates still use the prior path and need explicit handoff; this change does
not pre-admit native construction, establish a topology proof, or make an
unattested display rebuildable. Workspace compilation passed with
L:/om9-target and -j 2. No producer, rollback, concurrency, memory-pressure,
or runtime tests were run; C1/C2 remain incomplete.

## Boolean worker reservation handoff

The Boolean worker's non-preview results now split each exclusive candidate
from the worker's existing kernel reservation rather than reserving its complete
already-admitted output again. The source guards remain live through the
transaction; each display receives its own owner, and a failed transaction
prunes source-dead unpinned entries. The reserved handoff accepts both Kernel
and CpuCache reservations from the same account. Preview acceptance and other
shared or owned producers still require their own handoffs; native allocations,
topology attestation, rollback and cancellation are not proved by compilation.
Workspace compilation passed with L:/om9-target and -j 2 after correcting a
closure type error. No Boolean, memory-pressure, cancellation or runtime tests
were run.

## Boolean preview split owners

An admitted Boolean Two Objects worker preview now splits its existing output
reservation into independently owned source, topology and display allocations
before becoming the active preview. Display/store metadata is included in the
display owner. Acceptance shares that display owner while the preview remains
available on failure; source-only document geometry does not own the display.
An empty preview retains its original reservation, and the synchronous preview
path remains on its prior accounting. Workspace compilation passed with
L:/om9-target and -j 2. No preview, acceptance, cancellation, memory-pressure,
or runtime checks were run. Other shared producers, source-only reconstruction,
and durable topology binding remain open.

## Empty legacy native display rebuild

Private v4 Open candidates now attempt to rebuild an empty NativeBrep display
from its retained 3DM payload using the selected OCCT bridge, on the UI worker
or the existing synchronous/batch path. Rust-owned snapshot and mesh buffers
are admitted separately; the resulting mesh is bound to the original source
and handed to the candidate coordinator before scene publication. A failed or
canceled worker does not publish the candidate. The payload is not rewritten,
and this does not authorize eviction or assert that an opaque payload is free
of embedded caches. Native parser/mesher allocations remain outside Rust
admission. V5 sources with no legacy display still need integration, as do
durable OCCT topology binding and v5 BRep save. Workspace and no-default app
compile checks passed with L:/om9-target and -j 2 after a borrow correction;
no Open, geometry, cancellation, memory-pressure or runtime tests were run.

## Source-only native Open candidate

Private Open preparation now discovers NativeBrep sources that have no legacy
representation, including entity, definition-member and result snapshots, and
attempts the same admitted Rust-side display reconstruction before candidate
scene construction. This also covers source-only v5 input; the stored opaque
payload is not changed. Failure refuses the candidate, and native internal
parser/mesher allocations remain outside Rust admission. This does not permit
v5 NativeBrep saves or cache eviction, and it does not establish persistent
OCCT topology binding. Workspace compilation passed with L:/om9-target and
-j 2; no Open, geometry, malformed-input, cancellation, memory-pressure or
runtime tests were run.

## Synchronous Boolean preview ownership

Synchronous Boolean Two Objects previews now reserve their completed result
and use the same split source/topology/display owner transfer as worker
previews. Replacement and acceptance check whether the preview is already
accounted before adding its usage as an external hold. The synchronous native
operation still allocates before this reservation, and the transfer has not
been exercised at runtime. Workspace and no-default-features app compile
checks passed using L:/om9-target and -j 2; no command, failure, preview,
memory-pressure, or runtime tests were run.

## Fillet preview ownership

Fillet Edge previews now use the shared split source/topology/display owner
handoff after their synchronous native result is produced. Commit and preview
replacement retain the accounted display without adding its bytes again to
external holds. Solid pending usage likewise excludes already-reserved Boolean
and Fillet preview bytes while retaining structural point/node counts. The
native operation is still not pre-admitted. Workspace and no-default-features
app compile checks passed with L:/om9-target and -j 2; no Fillet, Boolean,
preview replacement, rollback, memory-pressure or runtime tests were run.

## Native BRep minimum-distance source handoff

Minimum object distance now converts each selected NativeBrep payload to an
OCCT source using the existing admitted adapter, holds both converted sources
under a kernel reservation for the query, and passes them to the distance
adapter without creating a document or viewport representation. Other operand
types use the existing path. The adapter API requires the caller to bind each
source to the matching payload; the selected-object caller performs that
conversion directly. Native parser and query allocations are not covered by
the Rust reservation. Workspace compilation passed with L:/om9-target and
-j 2; no distance, geometry, memory-pressure or runtime checks were run. This
is one affected consumer, not completion of native consumer migration or C4.


## Representation owner account check

The ordinary candidate-registration path now verifies that existing source, topology and display reservation owners belong to the receiving coordinator account. The reserved split path also requires an unowned, exclusive candidate. Unowned shared candidates still use the transitional path, so producer accounting is incomplete. Workspace compilation passed with L:/om9-target and -j 2; no cross-account, command, rollback, concurrency or runtime tests were run. Process-local topology proofs remain unsuitable for serialization and source-only OCCT Open remains incomplete.


## Native source-only history candidate

The history worker now discovers missing native representations in candidate entities and definitions, reconstructs displays from retained payloads using the same helper as Open, and stages independently charged representations in the prepared history rollback batch. Synchronous Undo/Redo retries via the worker on a native representation miss. Sources and opaque payloads are unchanged; native displays are not eligible for eviction. Candidate freshness, cancellation, rollback, native allocation behavior, and geometric results remain runtime-unverified. Workspace compilation passed with L:/om9-target and -j 2; no Undo/Redo, geometry, memory-pressure, or runtime tests were run.


## Primitive topology-proof coverage

General primitive creation, placed torus, ellipsoid without focus-point outputs, closed regular pyramid and closed cone/cylinder now use the existing optional in-process topology capture and admitted source/display handoff. If proof capture or admission is refused before publication, the existing primitive fallback remains available. This extends potential history-only cache eviction and rebuild only when proof adoption succeeds; open shells, star pyramids, ellipsoids with focus outputs, other producers, and persistent proof remain open. Workspace compilation passed with L:/om9-target and -j 2. No primitive, topology, eviction, Undo/Redo, memory-pressure or runtime tests were run.


## Candidate registration collision handling

Ordinary BRep candidate registration now reuses only an exact existing source/display/profile and owner binding, and refuses a conflicting entry for the same key. Redundant or rejected representation owners are dropped after the store lock is released. This protects current readers and preview retries, but does not address every cache insertion path, shared unowned accounting, or persistent topology proof. Workspace compilation passed with L:/om9-target and -j 2; no collision, concurrency, preview, rollback or runtime tests were run.


## Pending representation account and lock handoff

Private transaction and history representation batches now require each OCCT or native display to carry an allocated reservation from the batch account. Binding validation happens before the representation-store lock, and duplicate candidates retain their owner until after that lock is released. These checks do not prove complete producer accounting, durable topology binding or runtime correctness. Workspace compilation passed with L:/om9-target and -j 2; no account, collision, cancellation, rollback, concurrency or runtime tests were run.


## Direct and batch Open preparation worker

Direct and batch Open now use the same worker-backed candidate preparation and publication path as UI Open. Their synchronous APIs wait for the worker result; batch Open continues to retain the saved document name and accept archive paths without an .om9 extension. Workspace compilation passed with L:/om9-target and -j 2. No Open, cancellation, rollback, naming, memory-pressure or runtime tests were run. Native parser and mesher allocations remain outside Rust admission, and this change does not establish durable OCCT topology binding or permit v5 BRep saves.


## Native representation collision guard

The native representation store now rejects a duplicate source identity instead of silently replacing its retained display. The existing private transaction and migration callers already check for duplicates before insertion. UI Open also checks its operation token before validating the path, preserving the prior phase-check order. Workspace compilation passed with L:/om9-target and -j 2; no collision, Open, concurrency or runtime tests were run. Opaque native entries remain ineligible for live-source eviction: reconstruction of some payloads does not establish that every native payload can reproduce its retained display. The public insertion API still consumes a rejected candidate, so callers holding the store lock must manage its owner lifetime appropriately.


## Retained representation history wires

History worker preparation now creates source-derived interior wire buffers for root and nested OCCT occurrences whose representations are retained, as well as those rebuilt after eviction. The candidate is materialized from its source-bound representation and is staged before scene publication. Direct synchronous Undo and Redo now wait on the same worker path as UI requests, including for cache hits; this also makes the worker's physical-memory sampling requirement apply to those calls. Workspace and om9-app no-default-features compile checks passed with L:/om9-target and -j 2. No history, geometry, occurrence, cancellation, memory-pressure or runtime tests were run. Initial Open and other scene creation paths still need wire integration, and this change does not establish durable topology attestation.


## Open candidate source wires

With OCCT enabled, the Open worker now prepares source-derived interior wires for each root or nested OCCT occurrence in the private initial scene, before publication. Wire buffers retain separate admitted owners, and a temporary scene snapshot reservation covers the replacement overlap. A preparation error or cancellation rejects the candidate; some legacy sources that cannot supply these wires may therefore fail Open rather than publish an incomplete wire scene. Workspace and om9-app no-default-features compile checks passed with L:/om9-target and -j 2. No Open, legacy compatibility, placement, geometry, memory-pressure, cancellation or runtime tests were run. Other scene creation and update paths, native face wires, and durable topology attestation remain open.


## Edit candidate representation and wires

Full edit-scene capture now uses the coordinator's representation store instead of a newly empty store. Incremental and full edit candidates prepare missing source-derived OCCT interior wires before committing the transaction; occurrences already carrying wires retain their existing buffers. This work currently runs synchronously in the edit preparation path and may refuse an edit if a source cannot produce wires. Workspace compilation passed with L:/om9-target and -j 2. No command, scene, placement, memory-pressure, rollback or runtime tests were run. Other publication paths, native face wires, durable topology binding and general producer accounting remain open.


## Explicit interior-wire preparation receipt

Prepared OCCT scene buffers now record whether source interior-wire preparation completed, separately from the number of resulting segments. The receipt remains attached to the immutable buffer allocation and survives scene sharing; unprepared edge-only buffers do not acquire it. Existing prefix and index checks still run before the receipt is accepted. This distinguishes a successfully prepared empty result from missing work during scene updates and ExtractWireframe checks. Workspace compilation passed with L:/om9-target and -j 2; no zero-wire, extraction, scene-sharing, geometry or runtime tests were run. This receipt is not topology attestation and does not resolve native face wires or durable source binding.


## Trim and Tube exclusive registration

Both affected Trim paths now register the completed candidate once and share the resulting canonical source, instead of first registering a cloned unowned candidate. Tube moves its Boolean body into registration instead of cloning it. When their completed candidates are otherwise exclusive, the existing registration path can split and retain a display owner. Workspace compilation passed with L:/om9-target and -j 2; no Trim, Tube, geometry, memory-pressure, rollback or runtime tests were run. Kernel production is still not pre-admitted by this change. The ordinary registration helper's residual source reservation is not attached to its returned canonical source; document validation currently rejects transient source leases, so source accounting between registration and document staging still requires a coordinated handoff design.


## Primitive fallback source handoff

The proof-free fallback in optional-topology primitive creation now uses the admitted Rust snapshot callback, splits its display reservation, and retains the remaining source reservation until the existing single-insert transaction handoff registers the candidate document. This covers the fallback for callers of that primitive helper; it does not change the ordinary shared or unreserved producer paths. Native allocations inside the adapter are not bounded by the Rust callback. Workspace compilation passed with L:/om9-target and -j 2; no primitive, fallback, memory-pressure, rollback or runtime tests were run.


## Tube source reservation handoff

Tube now releases its temporary Boolean inputs, reserves the completed exclusive result, and keeps its source reservation through the existing single-insert transaction handoff. This covers the result after native production; the Boolean and primitive input construction remain outside that reservation. Workspace compilation passed with L:/om9-target and -j 2; no Tube, geometry, allocation-pressure, rollback or runtime tests were run. The Boolean worker's multi-output guards remain held through its transaction but are not yet atomically transferred like the single-insert case.


## Boolean worker batch source handoff

Transaction preparation now accepts a nonempty ordered batch of exclusively owned BRep insertion reservations alongside delete mutations. It checks each source, account, reservation and display receipt, then transfers the combined source charges to staged documents under the existing account locks; refusal leaves the guards charged. The Boolean worker uses this path for its one or many result bodies. Workspace compilation passed with L:/om9-target and -j 2; no Boolean, multi-output, budget, rollback, concurrency or runtime tests were run. Native production and general shared or unreserved producers remain outside this handoff; this is not proof of complete C1 accounting.


## Shell worker source handoff

The reserved transaction handoff now also accepts BRep geometry replacements and tracks CPU-cache and kernel reservation classes independently during atomic transfer. Shell's face-verification worker splits the already allocated output from its kernel reservation, registers its display separately, and transfers the remaining source charge when staging the replacement. Workspace compilation passed with L:/om9-target and -j 2; no Shell, mixed-class, cancellation, rollback, memory-pressure or runtime tests were run. Extract Surface and other worker outputs still need their own handoffs, and native allocations are not fully bounded by the Rust-side reservation.


## Extract Surface worker source handoff

Extract Surface now splits the already allocated worker reservation for every output face and replacement body, registers their displays separately, and passes ordered source guards into the batch transaction handoff. The guard order follows the numeric path's source-first mutation rotation; deletion-only groups carry no guard. Temporary guard and mutation metadata is reserved and actual vector capacities are reconciled. Workspace compilation passed with L:/om9-target and -j 2; no Extract Surface, multi-selection, guard-order, cancellation, rollback, memory-pressure or runtime tests were run. Native kernel allocations and other producer paths remain open.


## Leased preview document staging

Registration of a preview-owned BRep now returns an owner-free canonical handle for document staging while the stored representation retains the original source and display leases. The handle shares the exact canonical allocations and identity; it does not put the transient lease in the document, which document validation rejects. This addresses the registration boundary used by preview commits but conservatively double-charges the retained source while both the representation lease and staged document account for it. Workspace compilation passed with L:/om9-target and -j 2; no preview, commit, budget, rollback or runtime tests were run. A full atomic lease transfer and general shared-candidate accounting remain open.


## Reserved source guard binding

Reserved BRep registration now marks the remaining source guard with its process-local canonical source identity. Batch transaction preparation requires these bindings and rejects a guard paired with a different source even when sizes match; bound guards cannot be split or merged before handoff. The binding clears on successful transfer, while failed preparation leaves the guard charged. The older single-insert entry point still accepts an unbound guard for its existing direct-reservation use. Workspace compilation passed with L:/om9-target and -j 2; no swapped-guard, cancellation, batch, rollback, concurrency or runtime tests were run. The binding is an accounting receipt, not persistent topology attestation.


## OCCT store duplicate protection

The OCCT representation store now refuses duplicate source/profile keys in both ordinary and pending insertion rather than replacing the existing display. Existing coordinator paths already perform their own collision checks; this adds the invariant at the store boundary. Workspace compilation passed with L:/om9-target and -j 2; no duplicate, concurrent-reader, rollback or runtime tests were run. Callers holding the store lock must continue to manage rejected owner lifetimes appropriately; this does not implement persistent identity, eviction or source accounting for unowned shared candidates.


## Source-wire display profile binding

Open, history and edit wire preparation now carry the stored representation's certified linear and angular profile into the replacement scene. When the profile is unspecified they use the scene's unknown marker instead of inventing an angular setting of 0.3. This avoids treating an unknown or differently configured retained display as a matching remesh request. Workspace compilation passed with L:/om9-target and -j 2; no profile, remesh, Open, Undo/Redo or runtime tests were run. This records an existing in-process profile and is not durable topology attestation.


## Edit-scene local wire reuse

Full edit-scene preparation now reuses an independently owned prior BRep wire buffer when the occurrence path, canonical source identity and pinned display allocation match. The candidate keeps its own placement and scene metadata; mismatched or internally shared prior buffers use the normal source-wire preparation path. This avoids regenerating those local wires solely because an edit rebuilds the scene. Workspace compilation passed with L:/om9-target and -j 2; no transform, instance, allocation, cache-owner, rollback or runtime tests were run. Initial candidate capture still builds its ordinary display buffers before this reuse, so this does not establish that all placement-only work avoids mesh-buffer construction.

The reuse path additionally requires an unambiguous matching leaf and validates the prior wire receipt and display-buffer prefix before reuse. It preserves the prior output deflection and refuses replacement of a shared registered candidate buffer. A subsequent workspace compile check passed with the same target and job limit; the above runtime limitations remain.


## Edit capture reuses local wires directly

Full edit-scene capture now selects an eligible previous BRep wire buffer before calling the ordinary buffer constructor. It checks occurrence and leaf uniqueness, canonical source identity, exact display allocation, independent buffer ownership and the prior wire receipt; placement and other candidate metadata are rebuilt. This supersedes the post-capture reuse implementation. Planning still conservatively budgets ordinary buffers, and reused buffer usage is checked against candidate limits, so tight-budget behavior may remain overly conservative. Workspace and om9-app no-default-features compile checks passed with L:/om9-target and -j 2; no transform, instance, memory-pressure, rollback, concurrency or runtime tests were run.

Capture accounting now treats reused bytes as already charged to their retained owner while continuing to enforce per-scene point and buffer limits. The API requires the caller to keep the prior scene and its buffer owners charged in the same account through handoff. The planner still reserves ordinary-buffer bounds, so it can refuse an otherwise feasible reuse under a tight budget. A subsequent workspace compile check passed; account and runtime behavior remain unverified.


## Root wire reuse admission

Edit capture now checks that the prior scene's memory reservation and representation store belong to the receiving coordinator before considering reuse; otherwise it builds ordinary buffers. Preflight and capture share the exact source/display/occurrence eligibility check for root BReps, so preflight counts their existing points but does not reserve a second ordinary buffer. Nested occurrences continue to use conservative ordinary-buffer planning. Workspace compilation passed with L:/om9-target and -j 2; no cross-account, tight-budget, instance, rollback, concurrency or runtime tests were run. The account check relies on the scene's existing owner-binding invariant.


## Nested wire reuse admission

The same edit preflight now tracks complete instance paths with borrowed stack nodes and applies the capture eligibility check to nested BRep occurrences. Reused occurrences contribute their existing point counts; an ordinary buffer is still planned for each distinct leaf cache key that has a non-reused occurrence. Workspace compilation passed with L:/om9-target and -j 2; no nested-instance, mixed reuse, budget, rollback, concurrency or runtime tests were run. The same-account and scene owner-binding limitations above remain.


## History scene wire reuse

With OCCT enabled, Undo/Redo preparation now retains an independently accounted snapshot of the current scene on its worker. Candidate capture uses the same account-checked occurrence reuse and preflight as edit capture; only occurrences still missing source wires are prepared afterward, before publication. The history proof-cache guard stays alive through that preparation, and the worker cancellation flag is checked by the shared missing-wire path. Workspace and om9-app no-default-features compile checks passed with L:/om9-target and -j 2. No Undo/Redo, instance, cache-eviction, cancellation, budget, rollback or runtime tests were run. The additional snapshot metadata is reserved and may itself cause budget refusal.

History preparation now skips the optional snapshot when the current scene has no independently owned prepared BRep wires. If snapshot admission or construction returns a budget error, it drops that private attempt and continues without reuse; other errors still propagate. Candidate preparation can still refuse independently if rebuilding its scene or wires exceeds budget. A subsequent workspace compile check passed; fallback and runtime behavior remain unverified.


## Failed transaction representation cleanup

After failed ordinary or reserved transaction preparation, and after failed or unpublished application scene commits, the coordinator now prunes source-dead, unpinned OCCT and, where applicable, native representation entries. The existing pruning functions remove entries under the store lock and release their owners afterward; live sources or display readers remain ineligible. Workspace compilation passed with L:/om9-target and -j 2; no failed-command, preview-reader, cancellation, concurrency, memory-pressure or runtime tests were run. This is orphan cleanup, not eviction of a live history source or a complete producer handoff.


## ExtractWireframe source-wire completion

Synchronous, worker and multi-occurrence OCCT ExtractWireframe paths now request source edges and default interior face wires even if the current scene lacks a wire-preparation receipt. Their existing candidate-scene commit prepares missing scene wires before publication; failure leaves the prior document and scene in place. Native opaque face/trim extraction remains explicitly unsupported. Workspace compilation passed with L:/om9-target and -j 2; no menu, CMD, worker, instance, trimmed geometry, cancellation, rollback or runtime tests were run. Other scene paths and native binding remain open, so this is not a complete I1 claim.


## Wire owner no longer retains source candidate

Open, history and edit wire-buffer owners no longer retain the temporary BRep candidate after scene replacement. The scene's display pin keeps its display lease, and the wire owner reserves a conservative topology charge so a display pin can outlive the document without relying on the document's topology accounting. That charge can overlap the document's or an existing topology owner's charge and remains while either the wire buffer or display lease retains the owner. Workspace compilation passed with L:/om9-target and -j 2; no source-lifetime, buffer-only reader, deletion, budget, rollback or runtime tests were run. Unowned shared display candidates and exact de-duplication of topology charges remain open.

Wire-buffer and display-topology reservations are now split into independent owners. A buffer-only clone therefore retains its wire charge without retaining the display pin's topology charge; a surviving display pin retains its topology charge separately. The topology charge may still overlap document or other topology ownership. A subsequent workspace compile check passed; the lifetime, accounting and runtime limitations above remain unverified.

## Shared candidate display admission

Registration now refuses a nonexclusive BRep candidate unless its display already has a same-account reservation covering its allocated display bytes. Exclusive unowned candidates still use the existing reserved adoption path; properly owned preview clones retain their independent display lease. This prevents registering a shared ownerless display whose other clones could outlive a newly attached reservation. It can expose producers that still share unowned candidates as command failures; those producers need an ownership handoff rather than bypassing this check. Workspace compilation passed with L:/om9-target and -j 2. No command, preview, accounting, failure-path or runtime tests were run. Source handoff, pre-admission, topology charge de-duplication, native disposition and the full plan remain open.

The two explicit clone registrations found in app code are Boolean Two and Fillet previews; their current creation paths attach split preview owners before cloning. Other producers can still create implicit sharing and need further audit. Shared registration now uses the same independently reserved validation scratch as reserved exclusive registration, including a budget recheck before allocation. This does not solve the source lease handoff or pre-admit native work. Workspace compilation passed with L:/om9-target and -j 2; neither preview nor budget behavior was run.

## Solid insertion source handoff

The common solid-insertion helper used by Pipe variants and text solids now keeps an exclusive candidate's source reservation through the existing reserved transaction staging path, instead of dropping it when registration returns. Text-solid construction releases its temporary glyph vector before this handoff so an unchanged first glyph is no longer retained by that vector. The native shape and initial candidate still exist before this reservation; other registration paths and implicit sharing remain to be addressed. Workspace compilation passed with L:/om9-target and -j 2. No Pipe, text, low-memory, rollback or runtime tests were run.

## Sweep history source handoff

Sweep 4 and both Sweep History submission paths now retain a reserved BRep source through transaction staging. Reserved mutation admission accepts an accompanying History mutation only when its published-result list is empty; the recipe and its normal accounting remain part of the transaction. ProfileEndCap's BRep insertion also uses the reserved solid-insertion helper. Native candidate construction is still prior to reservation, other producers remain, and history replay is not covered by this change. Workspace and om9-app no-default-features compile checks passed with L:/om9-target and -j 2. No sweep, history-recording, ProfileEndCap, budget, rollback or runtime tests were run.

## Solid replacement source handoff

Rib, synchronous Fillet Edge, and both synchronous Shell replacement paths now hold the candidate source reservation through the existing reserved mutation staging path. Failed publication prunes only source-dead, unpinned representations. Native result construction still precedes reservation; multi-object Cap and other producers remain to be converted. Workspace compilation passed with L:/om9-target and -j 2. No command, selection, rollback, memory-pressure or runtime tests were run.

## Multi-object Cap source handoff

Cap now keeps one reserved source guard per candidate through the batch transaction and separately reserves storage for the guard vector. Failure drops the private batch before orphan pruning. Candidate construction still happens before each guard is acquired, and this change does not establish pre-admission for native allocations or all temporary vectors. Workspace compilation passed with L:/om9-target and -j 2. No multi-object Cap, allocation, rollback, memory-pressure or runtime tests were run; other producers remain open.

## Sweep replay source handoff

ReplayHistory now retains a reservation for rebuilt Sweep BReps through the replacement transaction; its curve result continues on the ordinary path. Native result creation still precedes reservation. The separate registered-builder recompute path creates a draft and published-result geometry snapshots that share candidate sources before transaction staging; it is not covered by this handoff and requires a shared-source transfer design. Workspace compilation passed with L:/om9-target and -j 2. No replay, builder recompute, budget, rollback or runtime tests were run.

## Builder recompute candidate reservations

Registered-builder recompute now reserves each generated BRep's source and display before transaction staging, and keeps those reservations while draft and published-result snapshots share the source. Once the staged document is registered, it splits each guard so the pending display has its own lease; the source portion can then be released. This is a conservative overlap, not an atomic no-double-charge transfer, and may reject a feasible operation under tight limits. Generation and earlier draft construction still precede these reservations; allocation and validation admission remain incomplete. Workspace compilation passed with L:/om9-target and -j 2. No recompute, budget, rollback, cancellation or runtime tests were run.

## Pending display validation admission

Pending OCCT representations used by import, builder operations and history rebuild now reserve topology-validation scratch before insertion. The pending batch verifies that the supplied coordinator shares both its account and representation store, and retains the caller's validation limits. This covers the scratch allocated by this validation step; it does not pre-admit upstream native construction or solve shared-source transfer. Workspace compilation and om9-app no-default-features compilation passed with L:/om9-target and -j 2. No import, history, builder, low-memory, rollback or runtime tests were run.

## Legacy display validation admission

The private v4 migration coordinator now also reserves OCCT topology-validation scratch before adopting each legacy display. The account peer used by history rebuild is constructed with the same reservation registry and representation store, matching the pending-batch coordinator check above. This does not certify v4 topology or make opaque native payloads canonical-only. Workspace compilation passed with L:/om9-target and -j 2; no legacy Open, history worker, low-memory, rollback or runtime tests were run.

## Placed primitive source handoff

The Pyramid and open Truncated Cone paths now pass their candidate and existing placement through the reserved single-solid insertion path. The source reservation remains live through transaction staging. The original native construction and shape inspection still precede reservation; Ellipsoid with focus outputs and other mixed or batch producers require separate handoffs. Workspace compilation passed with L:/om9-target and -j 2. No command, placement, low-memory, rollback or runtime tests were run.

## Extrude batch source handoff

Extrude now keeps one source reservation per candidate through batch transaction staging and separately reserves the guard vector. Candidate validation occurs in the reserved registration path; the accumulated per-operation limits still account for output storage and counts. Native construction and temporary mutation/output vectors still precede or fall outside this handoff. Workspace compilation passed with L:/om9-target and -j 2. No multi-object Extrude, selection, low-memory, rollback or runtime tests were run.

## Ellipsoid and focus handoff

The Ellipsoid path that also creates focus points now retains its BRep source guard through the mixed insertion transaction. Reserved transaction admission permits point inserts alongside guarded BReps; point storage remains part of ordinary request accounting. Native ellipsoid construction remains before reservation. Workspace compilation passed with L:/om9-target and -j 2. No Ellipsoid, focus, mixed-batch, memory-pressure, rollback or runtime tests were run.

## Boolean and Create Solid source handoff

Synchronous Boolean group and selected-object results now keep their source guard through transaction staging. Create Solid and Boolean Split likewise keep one guard per generated output and reserve the guard vector. The kernel can create all split pieces before individual guards are acquired, and the shapes, native work and other temporary vectors are not pre-admitted by this change. Workspace compilation passed with L:/om9-target and -j 2. No Boolean, Create Solid, selection, batch, budget, rollback or runtime tests were run.

## Slab source handoff

The internal Slab adapter now retains its BRep source guard through staging for its supported curve forms, preserving the output placement and selection. The circular-cut path releases its temporary Boolean outcome before handing off the cloned body. Native construction and temporary input solids still precede or fall outside the reservation. The interactive Slab command remains unavailable. Workspace compilation passed with L:/om9-target and -j 2. No Slab, placement, selection, budget, rollback or runtime tests were run.

## Boss and single-face extraction handoff

Boss replacement and synchronous single-face Extract Surface now retain source reservations through transaction staging. When extraction also replaces the remainder, guards are ordered to match its replacement and insertion mutations. Both native outputs are currently built before these reservations, including overlap between the extracted face and remainder; multi-face extraction is not covered by this change. Workspace compilation passed with L:/om9-target and -j 2. No Boss, extraction, copy/delete, placement, memory-pressure, rollback or runtime tests were run.

## Multi-face extraction handoff

Synchronous multi-face Extract Surface now keeps a guard for each extracted face and replacement remainder through batch staging, ordered with its mutations; deleted sources need no guard. The guard vector and known selection scratch are included in admission. Native face and remainder construction still precedes its individual reservation, and not all temporary structures are pre-admitted. Workspace compilation passed with L:/om9-target and -j 2. No multi-face, multi-object, copy/delete, selection, low-memory, rollback or runtime tests were run.

## Multiple Pipe source handoff

Both multiple-Pipe paths, including station-driven output, now retain a source reservation for each candidate through batch staging and reserve the guard vector. Existing aggregate point and node limits remain counted without charging candidate bytes twice through the local held estimate. Shape construction still precedes each reservation, and temporary structures are not fully pre-admitted. Workspace compilation passed with L:/om9-target and -j 2. No multiple-Pipe, station, output-count, memory-pressure, rollback or runtime tests were run.

## Trim source handoff

The two OCCT surface Trim paths now retain the replacement source guard through staging. Temporary planning entities are released before the guarded transaction, and failed preparations prune source-dead unpinned displays. Source generation and temporary partition/planning structures still precede or fall outside this handoff. Workspace and om9-app no-default-features compile checks passed with L:/om9-target and -j 2. No Trim, iterative editing, selection, budget, rollback or runtime tests were run. The v5 BRep save guard remains: process-local topology proofs and the unbounded untrusted shape-set reader do not provide durable binding.

## Split isocurve source handoff

The OCCT Split isocurve path now reserves each generated face and carries its source guard through the delete-and-insert transaction. Its guard vector is independently reserved. Source construction and the split partition still precede or fall outside the individual guards. Trim planning produces separate kernel partitions and curve fractions; its temporary planning entities are dropped before its guarded transfer. Workspace and om9-app no-default-features compile checks passed with L:/om9-target and -j 2. No Split, Trim, selection, placement, budget, rollback or runtime tests were run.

## Single-face extraction overlap

Single-face Extract Surface now reserves its extracted candidate before building a replacement remainder, so that candidate remains charged during their overlap. The earlier statement that both outputs were built before reservation no longer applies. Initial native extraction and remainder construction still precede their own reservations. Workspace compilation passed with L:/om9-target and -j 2. No extraction, low-memory, rollback or runtime tests were run. General Split can mix curves and BReps and still needs an admitted dynamically sized guard batch.

## Mixed Split source handoff

General Split now retains one source guard for each BRep output through its transaction, including batches that also insert curves. Guard-vector growth reserves the new allocation while retaining the old vector's charge; curve inserts remain in ordinary request accounting. Native partition and face construction, and other temporary vectors, still precede or fall outside the guard. Workspace and om9-app no-default-features compile checks passed with L:/om9-target and -j 2. No mixed Split, curve, surface, budget, rollback or runtime tests were run.

## Explode source handoff

Explode now retains guards for newly generated BRep faces through its transaction. Reserved batch admission matches guards in mutation order by their source identities; other geometry, including existing BReps copied from block definitions, stays in ordinary request accounting. Group replacements remain part of that transaction. Native conversions, face extraction and temporary piece vectors still precede or fall outside individual reservations. Workspace and om9-app no-default-features compile checks passed with L:/om9-target and -j 2. No Explode, block, group, mixed-geometry, budget, rollback or runtime tests were run.

## Join source handoff

Two-input and multi-input OCCT BRep Join now retain their generated result's source reservation through the delete-and-insert transaction. Non-BRep Join retains the ordinary path. Native transformations, conversions and sewing still precede the result reservation. Workspace and om9-app no-default-features compile checks passed with L:/om9-target and -j 2. No Join, selection, placement, budget, rollback or runtime tests were run.

## Mirror source handoff

Both interactive and axis Mirror routes now use the same guarded transaction for generated BRep copies or replacements. Placement mutations remain under ordinary transaction accounting, while source guards follow generated BReps in mutation order. Native transformations and temporary geometry still precede individual reservations. Workspace and om9-app no-default-features compile checks passed with L:/om9-target and -j 2. No Mirror, copy/replacement, mixed-selection, placement, budget, rollback or runtime tests were run.

## Materialization validation admission

Coordinator BRep materialization now admits topology-validation scratch before constructing the transient source/display pair. App call sites pass their current scene held estimate to the new held-aware form. This does not pre-admit later native operations or all caller temporaries, and the default API cannot infer caller-held memory. Gem source handoff remains open across its loader, editor, placement and definition paths. Workspace compilation passed with L:/om9-target and -j 2. No measurement, Join, Mirror, Trim, memory-pressure or runtime tests were run.

## Direct Gem source handoff

Gem loader, reload, non-duplicate positioner scaling and properties edits now retain a BRep source guard through transaction staging. Their placement, attribute, label and extension mutations remain ordinarily accounted. The duplicate-to-definition route and Gem cluster, pave and multi-edit consumers still use the general path; native conversion precedes the new guard. Workspace and om9-app no-default-features compile checks passed with L:/om9-target and -j 2. No Gem, metadata, scaling, selection, budget, rollback or runtime tests were run.

## Gem definition source handoff

Gem positioner duplication now carries a generated BRep source guard into its definition-insertion transaction. Reserved admission scans definition members and instances in order and transfers a matching source guard while retaining ordinary accounting for other members and mutations. Native conversion still precedes reservation; Gem cluster, pave and multi-edit paths remain open. Workspace and om9-app no-default-features compile checks passed with L:/om9-target and -j 2. No definition, duplication, instance, undo, budget, rollback or runtime tests were run.

## Gem multi-edit and cluster handoff

Gem multi-edit and cluster layout now retain source guards for generated BReps through their batch transactions, including mixed mesh/BRep and metadata mutations. Guard-vector growth is reserved. Gem conversion and other temporary structures still precede or fall outside these reservations; pavé and other Gem consumers remain open. Workspace and om9-app no-default-features compile checks passed with L:/om9-target and -j 2. No multi-edit, cluster, mixed-geometry, budget, rollback or runtime tests were run.

## Wire topology owner reuse

Open, history and edit wire preparation now avoid reserving a second topology allocation when the candidate already carries an independently allocated topology owner from the same account that covers its topology. The display pin retains that existing owner; the wire-buffer owner remains separate. Candidates without that exact coverage keep the conservative topology reservation. This does not deduplicate document-owned topology or establish durable topology binding. Workspace compilation passed with L:/om9-target and -j 2. No lifetime, deletion, shared-owner, budget, Open, Undo/Redo or runtime tests were run.

## Source-bound viewport wire replacement

The Open, history and edit wire-replacement entry point now requires the candidate to share the exact canonical source and topology allocations and validation fingerprints with the target occurrence. Equal source text alone is insufficient for this path. This is an in-process ownership check, not durable topology attestation, and it does not yet share newly prepared wires across occurrences. Workspace compilation passed with L:/om9-target and -j 2. No Open, Undo/Redo, instance, cache sharing or runtime tests were run.

## Shared occurrence wire buffers

When unpublished Open, history or edit scenes contain occurrences already sharing an ordinary buffer, source-wire preparation can now replace that buffer once for the entire group. Sharing requires every occurrence to have the exact same canonical source/topology allocations and display pin, and none may already own a registered override. Each occurrence retains the shared buffer and display leases while the scene charges the allocation once. Open skips occurrences already prepared by this group replacement. Other cases retain the prior individual path, and reuse across later scene captures remains conservative. Workspace and om9-app no-default-features compilation passed with L:/om9-target and -j 2. No Open, Undo/Redo, instance, remesh, accounting, cancellation or runtime tests were run; memory savings and behavior are unverified.

## Shared wires across scene capture

Edit and history capture can now retain an already shared, independently owned wire buffer for matching occurrences in a new scene. Capture counts that allocation once by buffer identity; each occurrence keeps its display and buffer lease. An occurrence without an exact previous-path match still follows the ordinary path. Preflight remains conservative and may count the same shared points more than once, potentially refusing a feasible capture under tight point limits. Workspace compilation passed with L:/om9-target and -j 2. No scene capture, instance, undo, memory-pressure or runtime tests were run; actual memory savings are unverified.

## Shared-wire preflight accounting

Edit and history scene preflight now counts each reused wire buffer's points once by its live allocation identity, matching the capture path. It reserves a bounded identity set sized from the previous scene before planning; ordinary non-reused buffers remain independently planned. The external edit capture now requests the corresponding preflight scratch estimate. Workspace compilation passed with L:/om9-target and -j 2. No tight-limit, instance, mixed reuse, Open, Undo/Redo, cancellation or runtime tests were run.

Capture now also uses a bounded identity set for reused wire buffers instead of scanning all already-captured objects to count each allocation once. Its memory is included in the prior-scene planning bound; matching an occurrence against the previous scene remains a separate lookup. Workspace compilation passed with L:/om9-target and -j 2. No runtime performance, memory-pressure or behavior tests were run.

## V5 BRep read guard

The v5 reader and writer now share the same temporary refusal for OCCT and native BRep sources in entities, definition members and result snapshots. A v5 document alone does not attest a durable native topology binding or a safe opaque-payload disposition. The separate v4 migration path remains available and retains legacy displays. Workspace compilation passed with L:/om9-target and -j 2; no archive, migration or runtime tests were run. This guard does not implement BRep persistence: durable binding, native payload disposition and source-only restoration remain open.

## Indexed previous-scene reuse

Planning and capturing a scene with reusable BRep wires now build a temporary sorted index of prior object positions by root and leaf. A lookup searches the corresponding group before checking full occurrence paths and the existing source, display and receipt conditions; duplicate matching paths remain ineligible. The bounded index is included in reuse planning scratch and does not copy geometry or viewport buffers. Workspace compilation passed with L:/om9-target and -j 2 after correcting an initial compile error. No runtime, instance, allocation-pressure or performance tests were run; actual speed and memory impact are unmeasured. Durable topology binding and v5 BRep persistence remain open.

## Native Open without rebuild adapter

The no-OCCT build now refuses an Open candidate when any native BRep in entities, definition members or result snapshots lacks its retained display or has an empty display. It continues to accept retained nonempty displays for this check, and observes cancellation while scanning. The default OCCT-enabled rebuild path is unchanged. om9-app compilation without default features passed with L:/om9-target and -j 2; no Open, cancellation, malformed-display or runtime tests were run. Existing native payloads remain opaque: the inventory fixture exercises mesh destruction on a specific object, but does not prove universal preservation of arbitrary payloads or permit v5 migration.

## Shared-source history rebuild jobs

History preparation now queues at most one missing OCCT representation rebuild per process-local canonical source identity, across document entities and definition members. The existing proof requirement remains; sources without suitable evidence cannot be rebuilt by this change. A bounded temporary identity set is admitted before allocation and released after planning. Workspace compilation passed with L:/om9-target and -j 2. No shared-source, Undo/Redo, accounting, cancellation or runtime checks were run. Durable source binding and native opaque-payload disposition remain open.

The corresponding native history queue now also uses a bounded identity set rather than repeatedly scanning queued sources. OCCT and native sets have non-overlapping lifetimes, so their shared planning reservation uses the larger bound and is reduced after both sets are dropped. Workspace compilation passed with L:/om9-target and -j 2; native shared-source behavior, memory accounting and performance remain untested at runtime.

## Pick display source binding

Scene-object pick-display access now checks the supplied entity's identity, generation, geometry revision and source receipt against the captured occurrence. Ordinary OCCT displays also must share the exact canonical topology allocation; registered overrides can retain separately allocated candidate-order topology and rely on their caller's binding checks. Native remesh pins continue to use the native payload receipt. An initial version incorrectly required allocation equality for remesh overrides; it was corrected after inspecting the separately rebound candidate path. Workspace compilation passed with L:/om9-target and -j 2; no pick, snap, remesh, stale-document or runtime tests were run. This in-process check is not durable topology attestation.

## Source wire input binding

The shared source-wire preparation entry now refuses a source string different from the BRep candidate's own source before extracting face isocurves. This prevents combining one source's interior wires with another candidate's edge and triangle buffers; equality alone does not attest topology IDs. Workspace compilation passed with L:/om9-target and -j 2. No wire, source mismatch, remesh, Open or history runtime checks were run. Persistent descriptors, their build identity and untrusted native parser bounds remain open for v5.

## Proof cache release outside lock

Topology proof reclamation, replacement and document-retain cleanup now detach retired proof owners and cache nodes while holding the cache lock, then drop their reservations after releasing it. Retired linked nodes are dropped iteratively. The failed node-accounting path likewise releases the lock before dropping the node. Workspace compilation passed with L:/om9-target and -j 2. No memory-pressure, concurrent reclamation, lock-order or runtime checks were run; this does not establish durable BRep topology or complete eviction.

## Shared native sources during Open

The OCCT-enabled Open preparation now counts missing native displays by distinct source allocation identity, rather than counting every entity or definition member that shares that source. A bounded, separately reserved temporary set is released before display reconstruction. This reduces duplicate placeholder reservation while retaining the opaque payload and existing failure/cancellation behavior. Workspace compilation passed with L:/om9-target and -j 2; no shared-source, admission, Open or runtime checks were run. The source payload is not stripped or certified for v5.

## Native scene replacement source lease

Native root display replacement now checks that the tracked source lease and the occurrence hold the same live payload allocation and CRC. The lease keeps that allocation alive for the comparison; independently allocated byte-identical payloads no longer pass as the same source. This also avoids comparing the complete opaque payload in the scene replacement path. Workspace compilation passed with L:/om9-target and -j 2; no native remesh, stale-source, payload-size or runtime checks were run. This process-local binding is not persistent payload certification.

## Native Open vector reservation

Open preparation no longer reserves the native display vector a second time when its existing archive-accounted capacity already holds the missing entries. When growth is needed, it still reserves the full new allocation alongside the existing one to cover reallocation overlap. Workspace compilation passed with L:/om9-target and -j 2; no tight-budget, vector-growth, Open or runtime checks were run. V5 native payload certification and persistent OCCT binding remain open.

## Legacy display source lookup

Adopting v4 displays now indexes candidate document source identities once per source type instead of scanning every geometry for each representation. The OCCT and native temporary sets have separate lifetimes and share one admitted scratch bound; all displays still require an exact live source identity before the private store is published. Workspace compilation passed with L:/om9-target and -j 2. No large-file, stale-source, admission, cancellation or runtime checks were run; the index is not topology attestation and v5 BRep persistence remains open.

## Native Open display lookup

Open's native display preparation now builds the missing-source set by inserting document identities and removing retained display identities, avoiding a retained-display scan for each source. The no-OCCT branch uses an admitted map of retained display availability while preserving the prior first-entry behavior and refusing missing or empty displays. Neither branch reserves this lookup scratch for a document with no native source. Workspace and om9-app no-default-features compilation passed with L:/om9-target and -j 2. No large-file, cancellation, memory-pressure, Open or runtime checks were run; no speedup is claimed without measurement.

## Incremental scene source ownership

Incremental metadata scene updates now require the same OCCT or native canonical source identity before reusing an occurrence's buffers and display pin. Byte-equal native payloads in a new allocation no longer inherit an old display pin and a newly assigned receipt. An existing older scene test expects this independent-allocation reuse and must be reconciled when the user-authorized testing phase begins. Workspace compilation passed with L:/om9-target and -j 2; no incremental update, native pin, stale-source or runtime checks were run. This does not implement durable BRep persistence.

## Topology mapping duplicate detection scratch

The source topology matcher now detects duplicate mapped candidate indices with an admitted byte marker array instead of allocating and sorting a second u32 array. Descriptor uniqueness, provenance and oriented incidence checks remain in place. Workspace compilation passed with L:/om9-target and -j 2; no duplicate, topology, memory-limit or runtime checks were run. Persistent descriptor provenance and BRep v5 restoration remain open.

## Representation lookup during reclamation

The OCCT representation store now uses its ordered source/profile keys to locate entries for a particular source when checking or reclaiming an unpinned display and when selecting a replacement preferred profile. The minimum range key depends on the documented profile variant order; no cache ownership or eviction eligibility rules changed. Workspace compilation passed with L:/om9-target and -j 2. No multi-profile, eviction, memory-pressure, concurrency or performance runtime checks were run.

## OCCT representation source receipt

OCCT representation matching and both preferred/profile lookup paths now require the source identity and the live source allocation receipt to match the supplied canonical source. A mismatch returns a cache miss rather than handing out that display. The single-accounted-receipt lookup also uses the source-key range. Workspace compilation passed with L:/om9-target and -j 2. No profile, stale-source, source-lifetime or runtime checks were run. This is an in-process ownership check, not a durable topology binding.

## Remesh profile cache release outside lock

The remesh representation cache now detaches exclusive entries under its lock and drops them after unlocking, both when replacing a full cache entry and during memory reclamation. Reclamation uses a fixed-size local array bounded by the existing cache slot count, retaining oldest-first selection and the exclusive-use check. Workspace compilation passed with L:/om9-target and -j 2. No concurrent reader, profile reuse, reclamation, accounting or runtime checks were run; this does not complete general cache eviction.

The exclusivity check also requires the display reservation owner and the buffer reservation owner to have no separate strong owners. A retained owner can otherwise keep its reservation alive after the mesh or buffer becomes exclusive, so the cache must not report those bytes as released. This is conservative and can defer eviction. Workspace compilation passed with L:/om9-target and -j 2; no separate-owner, reclamation-accounting or runtime checks were run.

## Source-dead representation cleanup

OCCT and native stores may now remove an entry once its canonical source has no strong owner, even if a display reader still holds an independent pin. The pin continues to retain its display and allocation owner; this only removes an unreachable cache entry and does not evict a representation for a live source. Memory-pressure admission still retries after removal and does not claim the pinned display was freed. Workspace compilation passed with L:/om9-target and -j 2. No pinned-reader, source-lifetime, reservation or runtime checks were run.

## Replacing one shared wire occurrence

An unpublished scene can now replace one occurrence whose registered wire buffer is shared, retaining the old buffer's charge while other occurrences still use it. Reinstalling a cached root representation also counts its buffer only if no other occurrence already uses it; existing registered-byte consistency is checked. This removes the prior unconditional shared-remesh refusal. Workspace compilation passed with L:/om9-target and -j 2. No shared-instance, repeated-profile, accounting, rollback or runtime checks were run; actual savings and remesh behavior remain unverified.

Both replacement paths now also refuse inconsistent per-occurrence registered-byte values for the same old buffer before adjusting shared accounting. Separately, v4 display adoption no longer reserves a zero-element BTree scratch estimate for documents with no legacy display entries. Workspace compilation passed with L:/om9-target and -j 2; no shared-buffer or tight-budget runtime checks were run.

## History rebuild display profile

When a history-only OCCT representation is reclaimed, its proof cache now retains the source's preferred display profile alongside the pinned rebuild evidence. Undo/Redo uses that profile when it is certified for the current adapter and has valid parameters; otherwise it rebuilds with the existing default and records the profile actually used. The profile is process-local and is cleared with its proof; it is not persisted in the document or an attestation. Workspace compilation passed with L:/om9-target and -j 2. No profile, multi-profile eviction, Undo/Redo, memory-pressure or runtime checks were run; durable topology and v5 BRep persistence remain open.

For a source with multiple profiles, the first recorded preference is now retained through that eviction cycle instead of being overwritten by the store's fallback preference after each removal. When that source is published again and the store has a representation, the old preference is cleared for a subsequent cycle. If the store is busy during that cleanup, the prior value is conservatively retained. Workspace compilation passed with L:/om9-target and -j 2; multi-profile, cancellation and runtime behavior remain untested.

History representation reclamation now selects nonpreferred profiles before the preferred profile, and keeps the preferred one while any other profile remains. This avoids changing the store's preferred profile merely because an alternate profile is still pinned. It can conservatively reclaim less memory when alternates are pinned. Workspace compilation passed with L:/om9-target and -j 2; no multi-profile, pinning, history, memory-pressure or runtime checks were run.

When a source is published as current again and its representation is present, its rebuild-only proof pin is now cleared along with the prior profile. The proof remains an optional cache entry; if later reclaimed, a live representation cannot be evicted without obtaining fresh rebuild evidence. If the store is busy the pin remains. Workspace compilation passed with L:/om9-target and -j 2; no proof lifetime, history, eviction or runtime checks were run. Native face wires still require a source-bound conversion and are not inferred from triangle edges.

## Canonical source ownership checks

Canonical BRep shared-payload checks now require the same source and topology Arc allocations in addition to process-local identity and fingerprints. Document accounting therefore cannot treat separately allocated payloads as shared on those values alone. The helper that strips allocation owners for coordinated document staging is now restricted to om9-core, where its existing callers perform the handoff. Workspace compilation passed with L:/om9-target and -j 2; no shared-source, accounting, staging or runtime checks were run. The shape-set native reader remains unsuitable for untrusted files until its internal allocations are bounded.

## Result snapshot source accounting

Document validation and storage accounting now count shared OCCT source/topology and native payload allocations once across entities, definitions and result snapshots. The indexed history inventory includes result sources in its scratch bounds and in cross-document/Undo deduplication; admission upper bounds remain duplicate-inclusive. Result metadata and nonshared geometry remain accounted separately. Workspace compilation passed with L:/om9-target and -j 2. No result-snapshot, multi-document, Undo, shared-source, limit or runtime checks were run; peak-memory savings remain unmeasured.
