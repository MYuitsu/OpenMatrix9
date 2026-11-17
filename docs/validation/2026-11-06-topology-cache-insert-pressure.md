# Topology cache insertion under pressure — 2026-11-06

The existing experimental topology-proof cache held its own mutex while requesting memory for a new entry. If the shared coordinator needed to reclaim that same cache, its nonblocking callback could not acquire the mutex; admission was refused even though an idle proof could be retired. A fixture with document-validation scratch reserved reproduced `Budget("bytes")` while the idle proof remained owned. After moving admission outside the cache lock, the fixture passes: the idle proof is released and the new proof is retained. The slots are checked again under the lock because another insertion or reclamation may have occurred in between.

Windows worktree, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -p om9-app --lib topology_cache_ -j 2`: 3 passed. The cases cover idle entry reclamation, refusal while a proof is pinned followed by a successful retry, and two concurrent insertions of one proof without duplicate cache ownership. The pressure cases preserved the document; the idle case also preserved the published scene.
- `cargo test -p om9-app --lib topology_owner_tests -j 2`: 32 passed, 2 ignored. The existing group includes remesh, stale publication, Undo, reservation lifetime and STL source capture fixtures.
- `python tools/public_source_audit.py` and `git diff --check`: passed. Existing app unreachable-pattern warning remains.

These tests use the in-process experimental proof and remesh path; normal Sphere creation still does not create this proof or request a remesh. They do not establish native peak bounds, a production cache, document/Undo source-only ownership, v4 topology mapping, or desktop behavior. No open OM9 session was changed.
