# Shared runtime object snapshots

Scope: RCORE-01/04/06 P0, continuing the user's selected foundation work after
the main merge. This is an OpenMatrix9 implementation of the written lifecycle
contract, not a recovered Rhino API or a new command.

## Ownership and lifecycle

Safe Rust owns document-session IDs, object generations, geometry revisions and
Undo/Redo epochs. IDs are nonzero and counters never wrap; exhaustion or a poisoned
registry rejects validation. The native bridge sends integer values only. C++ owns
FreeCAD signal connections and resolves the live object/placement on the GUI thread.
Capture adds no properties, document objects, transactions or Undo entries.

A snapshot includes the native object ID, diagnostic document UUID/name,
subelement name at the captured revision, explicit units context, world placement
and bounded placement-owner stamps. Deletion/recreation and Undo resurrection
cannot revive a token. Parent placement changes are tracked even when the final
matrix is restored to equal values. Closing a document drops its registry state.
Reopening the same FCStd preserves provenance but creates a new runtime session.

Native App property changes invalidate the source. Built-in Visibility, labels
and the derived `_GroupTouched` notification are excluded: display/label changes
do not change geometric identity. Actual Group membership still invalidates the
owner. This is conservative for other native properties and may require reselecting
after a non-geometric App property edit. It is separate from command permissions.

## Consumers

- Edit captures a guard with each existing native input, including special Explode
  adapters, and checks it before the existing geometry signature and transaction.
  Reference consumers using the same EditInput capture retain the guard.
- Rebuild checks the same guard before sampling saved curves and immediately
  before its transaction, preserving existing world-placement/shape checks.
- Interactive Sweep1/Sweep2/Loft capture once when accepting a selection. Every
  Chain Edges member retains its own guard; Reverse/Seam/Automatic/options never
  refresh the token. Source changes require Cancel and reselect, even if equal
  geometry is later restored. Option-only corrections remain supported.
- Persistent Surface History reconstructs from its existing durable references;
  its inputs deliberately have no runtime selection stamps. Tokens are never
  written into FCStd/History metadata and cannot be used across processes.

`objectSnapshot(document, object, optional_subelement)` exposes an owned native
capsule plus diagnostic scalar fields for acceptance checks. `validateObjectSnapshot`
validates that capsule without mutation. The capsule retains no document pointer.
Subelement existence/type is resolved by the consumer before accepting selection;
the diagnostic API itself does not implement a general subelement resolver.

## Boundaries and verification

C++ is required for FreeCAD document/placement access, signals and the existing
Python extension boundary. Python additions are host acceptance fixtures only.
The fixed C ABI uses five `u64` fields and never retains a pointer passed by the
caller. Unwind builds catch Rust panics at the boundary; the existing release
profile uses `panic=abort`, so panic/OOM recovery is not promised. This does not
certify native libraries or the whole application as memory-safe.

The [validation record](../validation/2026-10-09-rhino-core-object-foundations.md)
contains the failing baseline, native assertions, full Rust checks and limits.
This increment does not complete durable History revision mapping, all-family
selection/permissions, operation/backend capability routing, effective tolerance
resolution or async worker cancellation. Existing geometry restrictions and
disabled commands remain unchanged.
