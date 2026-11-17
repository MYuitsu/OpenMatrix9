# ExtractIsocurve pending surface admission

Starting ExtractIsocurve previously cloned the selected NURBS surface into the
pending session without a dedicated managed-memory reservation. This matters
more now that a trimmed surface with metadata above 16 KiB is admitted.

The command now reserves a conservative allowance alongside the live scene,
copies its vectors using fallible reservations, and retains the accounting
owner while the pending surface exists. Copies of the pending session share
both the immutable surface and the reservation. Cancellation, successful
completion and final-owner drop release it; errors that keep the session also
keep its reservation. A separate fixed-size `Arc` allocation and actual OS or
native allocation failure are not covered by the simulated managed-pressure
fixture.

For a 28×28 trimmed surface after `.om9` reopen, managed pressure at command
start rejects without opening a pending session, changing the document or
losing Redo. After pressure is removed, cancellation releases the reservation;
a retry extracts the two expected portions around the hole, releases the
reservation and supports Undo/Redo. A unit test confirms a cloned pending owner
keeps the charge until its last owner drops. All 26 isocurve integration tests,
two command-option tests and two no-OCCT trimmed-isocurve tests passed. Desktop,
physical out-of-memory and all surface types remain unverified.
