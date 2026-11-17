# B-rep in-memory source/topology integrity — 2026-11-06

`BrepSource` now retains the existing SHA-256 integrity digest over the source digest and topology/ID arrays. Source validation and the borrowed canonical record use that retained digest instead of recomputing and silently accepting a changed association. Native construction and an explicitly rebound candidate establish a new digest; v4 deserialization reconstructs it from the v4 fields. This is an in-memory consistency check, not proof that v4 UUIDs identify particular OCCT subshapes, and it does not authorize display eviction.

On Windows, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- A focused source-owner regression first failed when a face UUID was changed to another structurally valid UUID; afterward source validation and canonical-record validation both refused it. The full core B-rep group passed 23 tests, including source/display sharing, Undo accounting and v4 JSON byte compatibility.
- Kernel topology capture: 9 passed, 3 ignored. Archive project codec and source admission: 9 passed. The application source-consumer batch passed DupBorder 4, Measure 8 and Sphere FitPoints 3 tests. The Sphere reservation/history pressure test passed. Public-source audit and `git diff --check` passed; existing warnings remain.
- The Measure regression is an existing eight-test application batch, not a newly planned openNURBS matrix. One test uses one generated 3DM box with deliberately shifted display and checks source bounds and placement; it also rejects an invalid payload and missing tolerance. It is not independent Rhino interoperability or a native memory measurement. See the openNURBS roadmap ledger for the unchanged queue and packages.

The digest adds inline owner state and is not serialized into v4. No general native topology binding, source-only document/Undo owner, display eviction, native peak bound or desktop acceptance is claimed. No open OM9 session was changed.
