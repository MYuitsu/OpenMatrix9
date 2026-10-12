# Curve native booking — disposition, 2026-10-12

This resolves the transaction-identity finding from the fresh read-only
[Curve publication review](2026-10-11-curve-publication-review.md). That original
review is retained as historical evidence; the reviewer did not execute these
hosts or review a second iteration. Root traced and exercised the native APIs.

The original bridge discarded the returned booking ID and allowed a metadata
observer to commit the journal before the output check. Real native commit
callbacks reproduced this: RED12/30 across six commands, actual host exit0,
no fixture exception. Shape was rejected after early commit, but the output
survived and the original executable Redo had already been discarded. A separate
Line/abort witness was RED3/5, actual exit0: callback mutation survived outside
the retired journal, with an untyped native failure. A full exploratory run
timed out and is excluded from assertions; its owned PID was terminated by
the host runner and the terminal receipt is retained.

The existing host's `Document::lockTransaction` is reusable here:
`Application::closeActiveTransaction` refuses commit/abort of a locked
participant, and `Document::openTransaction` refuses replacement bookings.
Materializing or writing the already booked transaction remains allowed.
The bridge stores the ID returned by `openTransactionPreservingRedo`, owns one
balanced lock, and checks the same live booking and lock at each barrier. It
unlocks before final close, passes that exact ID to Application commit/abort,
requires the commit verdict, and never aborts a replacement booking. A missing
or replaced document is resolved before any access; this does not certify
every callback boundary in the surrounding callers.

Runtime56 commit/abort/commit-reopen/abort-reopen fixtures: GREEN120/120,
24 actual callbacks, actual host exit0, no fixture exception. Every changed
output is rejected with the actual command, complete document/Undo state is
restored and prior native Redo executes. Current total1372/1372 in24 actual
host exits0, including the unchanged callback/layer/replacement cohorts and
662 regression checks. Full Rust882/0/0; standalone plugin build0. SDK unchanged.

Rust continues to own session and independent geometry policy. This C++ code is
the required FreeCAD transaction adapter, not a portable policy fallback. No
FreeCAD root build or complete host runtime copy was made.

The named booking finding is resolved. Full global commit notifications,
arbitrary native unlock/close callbacks, document lifetime across every setter,
aggregate rollback across all families, foci common GeometryResult, History/
Rebuild, sharing/budgets/conversion/maps and all original RCORE-01 requirements
remain required. Offscreen native and FCStd tests do not certify mouse/render.
