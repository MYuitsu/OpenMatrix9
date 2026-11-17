# DupBorder: NativeBrep source boundary

2026-11-13. Previously NativeBrep was rejected. The command now converts its source to OCCT source with memory admission and extracts edges belonging to exactly one face. It does not infer the border from viewport triangles. Placement stays with output; mixed selection commits once.

- A reopened cylindrical shell, translated by 30 mm, produced two source circles at z=0 and z=5 and radius 2 mm on sampled points. Both CMD and menu dispatch passed, with Undo/Redo. Simulated exhausted managed memory refused and preserved document and Redo.
- A standalone plane and NativeBrep shell produced six boundaries in one transaction; malformed NativeBrep in a mixed selection refused without partial output and preserved Redo. This mixed fixture was not reopened.
- `cargo test -p om9-app --test dup_border -j 2`: 26 passed. Existing reopened NativeBrep ExtractWireframe regression: 1 passed. `cargo check -p om9-app --no-default-features -j 2` passed. Public source audit: 0 errors. Existing warnings are unrelated. No new target directory was created.
- The user approved recording this tested behavior. Desktop interaction, native allocation peak, genuine out-of-memory and other imported topology or transforms were not tested. Sampled points do not establish all-curve accuracy. Plan A remains open; B1–B3 deferred.
