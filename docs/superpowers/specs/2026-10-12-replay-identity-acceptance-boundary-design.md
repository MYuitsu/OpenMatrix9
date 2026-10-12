# Replay identity and the irreversible acceptance boundary

Status: approved by the user on 2026-10-12: acceptance before disposal,
with existing FreeCAD transaction semantics and separate disposal diagnostics.

The user wants identity-preserving rejection/rollback, reuse of FreeCAD's
transaction code, and completion of RCORE-01. This proposal preserves that
goal for every rejection before acceptance. It explicitly identifies a
contradiction in the current native-savepoint design rather than claiming
the remaining tests have passed.

## Observed contradiction

The unchanged Python fixture releases the only remaining strong reference
to an original value. Its `__del__` does not resurrect the object. The test
then requires that same object to remain alive for rollback. The native
fixture deletes a detached `DocumentObject`, runs its actual C++ destructor,
then requires its original identity and executable Redo ownership to return.

On App227/Runtime229 the original witnesses remain Python5/8 and native16/32.
Mutation barriers work, and acceptance is rejected, but ownership and lifetime
have already ended. Neither Property Copy/Paste, XML restoration nor a factory
can restore the identity of an actually destroyed object. Reconstructing a
replacement or retargeting its wrapper would not preserve native identity.

The existing implementation now checks finalizer attempts before reporting
success, but cannot satisfy both physical destruction and exact rollback.
This is not an issue that can be fixed by another retained reference: retaining
the Python value prevents its destructor from running in the first place.

## Recommended boundary

Use FreeCAD's existing transaction ownership and disposal semantics. Separate
reversible acceptance checks from irreversible destruction:

1. During preparation, retain original native objects, original Python values,
   caller first writes, booking and Redo. Check source/owner/generation/locks,
   native snapshots, Copy/Paste/Save/Restore, open notifications and all callbacks
   that can veto acceptance. Rejection restores the original identities.
2. Publish the prepared native journal only after those checks pass. This is the
   explicit acceptance point. Transfer through the existing FreeCAD transaction
   ownership APIs, with no new geometry or transaction implementation.
3. Dispose of obsolete private references and discarded Redo after acceptance,
   using the existing native destructors. Keep owner/source mutation barriers
   active throughout disposal. A destructor's attempted mutation is blocked and
   reported as a disposal diagnostic; it cannot veto or roll back the lifetime
   transition that already happened.
4. Expose acceptance and disposal diagnostics separately so neither the caller
   nor progress reporting claims a rollback of a destroyed object. Every enabled
   publication/History consumer must use the same boundary.

This changes the previously approved requirement that every finalizer attempt,
including an actual object's destructor, must veto acceptance. It does not
silently defer callbacks, manually call `__del__`, reconstruct objects at old
addresses or weaken mutation guards. The destruction point and its diagnostics
become explicit and testable. Successful plain replay retains no invented Undo;
regular and preserving caller transactions keep FreeCAD's normal semantics.

## Alternative: strict rejection before any unsafe destruction

If every destructor must remain a veto, reject a replay before publication when
it requires an unproven irreversible disposal. Keep the originals alive and
report a typed unsupported-lifetime failure. This preserves identity but limits
which Python/custom native objects and prior Redo states are supported. A type
name alone cannot certify arbitrary destructors or opaque Python graphs.

This alternative cannot be reported as full support of those states. Its tests
must require early rejection and intact original identities, not require the
destructor to run while also requiring its object to survive.

## Acceptance evidence required after approval

- Preserve the old contradictory witnesses and their historical failures.
- Test exact native/Python identity, booking, first writes and executable Redo
  for all reversible veto paths and both actual Undo modes.
- Test actual last-reference/destructor disposal once, its mutation barrier,
  explicit acceptance point, diagnostics and absence of false rollback claims.
- Test regular/preserving caller semantics, error cleanup, repeated completion,
  document lifecycle and all enabled publication/History consumers.
- Build against matched native SDK/import/plugin and record loaded DLL hashes.
- Keep full RCORE-01, aggregate budgets, global participants and remaining
  geometry/registry/worker/ownership requirements open until independently proven.

Offset D3 is independent of this design decision and can be corrected and
validated using the existing OCCT helper while identity review is pending.
