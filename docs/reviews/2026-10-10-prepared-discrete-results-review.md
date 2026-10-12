# Prepared discrete results review — 2026-10-10

One fresh read-only reviewer inspected the original chapter/plan, exact MeshPy
ownership and delay-loading, bounded native copies, frozen fields, Rust channel/
index validation, ON_Plane parity, float64 retention, report binding/persistence
and ABI/schema integration. No actionable regression was reported. The reviewer
did not modify source, build a plugin or run a host.

After that review the controller traced the existing export signature path and
found independent validation of archive/native mesh did not establish their
correspondence. Real host RED reproduced different geometry, reversed winding
and missing native location accepted, published and destroying old Redo:12/81
checks failed. Rust RED also accepted unrelated vertices. The controller added
a bounded oriented triangle multiset check at declared float32 host precision.
Matching quads, original float64/unused vertices, cyclic rotations, reindexing
and a once-applied location remain accepted; mismatches reject before mutation.
This additional correction was controller-reviewed and tested, without another
fresh review or any claim that the earlier reviewer covered this later change.

Controller source inspection also found segment ownership/index/byte copying
needed aggregate bounds. A real16384-segment owner reproduced4/65 failed checks
before the Rust index policy/native owner budget correction. Current-size_t
segment count arithmetic avoids wrap before the owner limit. Exact SDK copy
semantics preserve private kernel/segments and remap segment owner references.

Final matching evidence:675 Rust tests,3790 host checks,26 actual exits0.
Bounds are logical, FFI storage/lifetime remain caller obligations and native
master restoration requires the matching SDK. Native mesh reader exception
categories and CAD writer budget exceptions still need uniform typed mapping.
Full original RCORE-01 remains in progress; upstream maps/certificates/attributes,
other consumers and every original remaining item stay explicit.
