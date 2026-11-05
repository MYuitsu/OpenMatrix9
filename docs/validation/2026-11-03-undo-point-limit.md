# Undo history and document point limit

P0 partial: retained-document accounting now keeps the largest individual document point count instead of summing points across immutable Undo/Redo versions. Distinct retained document bytes remain additive. During transaction preparation, the candidate document's points are likewise not added a second time to the saved document's points. Each document and candidate still passes its structural validation.

The reproducer previously failed with `budget exceeded: points` on move 10 for a three-point mesh under a 30-point limit. After the change, 16 moves and 16 Undo/Redo steps pass and the test verifies history bytes still increase. Separate tests verify a 20-point document can be moved/undone/redone at the same limit, a 31-point document is refused, and inserting a 31st point is refused without changing the transaction snapshot.

Windows, `CARGO_TARGET_DIR=L:/om9-target`, Cargo `-j 2`: the atomic transaction, Undo-memory, transaction-scope, resource and layer transaction suites passed; four retained-owner unit tests passed. The 27 STL integration tests also passed with the same point-accounting behavior before the final equivalent atomic-error-path cleanup. These tests are fixtures and estimated accounting, not a real >256 MiB allocation or desktop measurement.

This corrects point aggregation in the tested retained-owner and candidate paths; it does not establish that every import, scope, scene or multi-document structural limit is independent. Other structural counters, shared subpayloads, session-wide owner reservations and actual kernel/export/cache peaks remain to audit. The existing 256 MiB aggregate byte ceiling is still present; the 60% RAM budget and other plan gates are not complete. The live OM9 session was untouched.
