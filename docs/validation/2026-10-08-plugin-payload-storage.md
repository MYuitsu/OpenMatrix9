# Plugin-owned 3DM payload storage

OM9 stores the immutable source 3DM archive and editable Hatch loop bytes in
FCStd without a FreeCAD PropertyFileIncluded core patch. The FreeCAD source tree
at 21d36cfa1eb110a1d0667050ff31706298805bbd equals the original baseline
0205a6b3b5eb9546760a2aa39391354504f20194. FreeCADBase/FreeCADApp were freshly
built from this tree and deployed to the isolated validation runtime.

## Storage contract

ThreeDmStorage uses 64 KiB binary chunks encoded in PropertyStringList, SHA256
and storage schema version 1. Archive and Hatch limits remain 512 MiB and
32 MiB. Original source archive bytes are immutable; a preview is not their
replacement. Existing native manifest, namespace, UUID and Hatch hash/schema
validation remain in force.

Converter paths are transient caches, independently owned per document/object
and payload. Every cache reuse verifies its hash. Changed bytes are validated
before an atomic staged replacement. Edits reuse one live cache path; deletion
and document closure remove caches. Missing caches can be regenerated from FCStd.
Invalid chunks, schema, digest or oversized payloads are rejected.

Legacy FCStd migration reads and validates every included payload before changing
any object. It adds chunks, digest, version and a separate CachePath. It never
removes, reassigns or changes the type of the old file-owning property: stock
FreeCAD legacy copies can share that file. After migration that property becomes
Transient, so saving/copying retains its metadata without embedding obsolete
bytes. The plugin reads authoritative chunks and ignores the legacy path.
Undo/Redo synchronizes this persistence flag with chunk presence. Migration
failure aborts additive changes and leaves all original included files untouched.
When Undo is disabled, migration temporarily enables transactions for rollback
and restores the user's mode afterward.

GUI restoration queues migration until restoration/current transactions finish;
recompute also checks for legacy payloads. Headless callers can explicitly use
ThreeDmStorage.migrate_document(document) before editing old payloads. Export
still validates its staging file before replacing the destination.

## Validation and limits

[Machine-readable evidence](../plugin-storage-checks-2026-10-08.json) records the
fresh stock-core hashes and individual FreeCAD reports. The storage suite uses
independent binary fixtures and an actual stock-core copy with a shared legacy
FileIncluded pathname. It checks copy/delete, cross-document copy, Undo/Redo,
FCStd reopen after removing caches/source, corruption/size rejection and an
injected mid-migration failure. A separate native Hatch FCStd fixture checks
decoded UUID/loop exchange and editing after migration.

Hatch edit/block/UI, recursive namespace/shared copy, source signature, native
queued menu actions, both caller-owned regression rings and serial/parallel
import are checked on the same runtime. The block regression exposed a missing
wire_density in generated instance/geometry manifests; those records now take
the value from native attributes. Strict retained-field comparisons remain.

This report supersedes the earlier Hatch loop report's core-patch dependency.
Earlier combined-source reports remain historical evidence for their binaries.
The native module still requires a matching FreeCAD SDK/ABI. Arbitrary installer
binaries and other operating systems are not certified by this run. No new Rhino
5 application run is claimed here; prior Rhino evidence remains scoped to its
fixtures. Full openNURBS exchange still has 7 open packages; future test batch
count is not yet enumerated.
