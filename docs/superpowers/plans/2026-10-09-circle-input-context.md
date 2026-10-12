# Circle accepted-input context repair

Authorized scope: the inherited Radius/Orientation relative-base gap reported in
`docs/reviews/2026-10-09-extended-routing-review.md`, using existing Circle solvers,
native reference kinds and geometry. Rust owns session state and opaque context
identity/retention. A required FreeCAD/OCCT adapter retains private geometry copies
and source identity metadata that cannot be stored in Rust; this native ownership
exception is bounded below. No replacement solver is added.

RCORE-03 requires relative vectors to use the previous accepted world point,
recoverable errors to preserve input state, and session Undo to restore the frame
checkpoint. RCORE-04 requires one checkpoint per accepted input and none for hover.
These are existing OM9 contracts, not a new claim of complete Rhino runtime parity.

Keep Circle's previous accepted point separate from construction points. Store
nonrecursive snapshots of points, Circle options, input frame and unit scale,
previous accepted point and pending option. Retain at most 1024 immutable `Arc` snapshots, discarding
the oldest before accepting a newer checkpoint. Session/preview clones share these
owned snapshots; Undo takes ownership or copies only the one restored snapshot.
The bounded point payload is at most 24 MiB (1024 snapshots x 1024 points x 24 bytes),
plus fixed options/short tangent constraints and at most 1024 pending option names.
Preview history copying is at most 8 KiB of shared references on 64-bit hosts,
plus the current geometry, without recursively copying checkpoint contents.
Selection-only native path events have a
checkpoint but do not create a point base. A nonempty fit batch is one input and
its last point becomes the base.

Centralize accepted-input handling around public text, point, native-reference
and fit-batch entry points. Internal text recursion and point dispatch use inner
methods so one input produces one checkpoint. Failed input restores its snapshot;
Undo pops a snapshot, including pending Radius/Orientation and numeric options.
Cancel/restart/commit discard temporary checkpoints. Other Curve families keep
their current behavior. Preview operates on copies and cannot change live context.
Enter on a ready Tangent session requests the existing solve without accepting
input; it does not add a checkpoint.

The monotonic context counter does not rewind on Undo. A later branch cannot reuse
a discarded identity. The short-buffer FFI leaves output unchanged. At most 1025
native contexts survive (1024 recent Undo checkpoints plus current). Their geometry
is immutable private OCCT data shared across matching reference captures; snapshots
retain no Python objects. Restoration creates live wrappers, restores the source
ledger before pruning and verifies guards before new input or commit. Native
composite path/center and fit/completion input captures its intermediate context.
F4 captures/verifies sources before accepting the origin, as text and mouse do.
Aggregate BRep/source-ledger RAM is not certified by the context-count limit.

1. Add independent analytic regressions; run them RED before production changes.
2. Add bounded snapshots and accepted-point tracking in `rust/src/curve.rs`.
3. Run the new regressions, all Circle/Curve suites and the full Rust suite.
4. Report any intentional conflict with older point-only Undo fixtures to the
   parent before changing those fixtures. Parent owns native replay and final audit.
5. Run native path, Direction/Radius association and F4 stale-source regressions
   RED/GREEN; verify Cancel/restart releases old captures and ordinary F4 still works.
