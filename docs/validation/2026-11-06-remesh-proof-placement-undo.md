# Remesh proof after placement and Undo — 2026-11-06

The optional topology-proof cache discarded a proof after every geometry
revision, including a placement-only Gumball/numeric move. The corresponding
remesh could therefore not be requested after moving an attested sphere even
though its source and display were unchanged. The test failed at that point
before the change.

Cache retention now checks the same document/session/object generation and a
cheap matching display fingerprint. This is only a filter: before the proof
can be reused, `rebind_unchanged` still checks account and session and fully
validates the current B-rep, source hash, topology and display against the
attestation. A geometry revision change alone no longer precludes that
validation. A foreign source/display drops out of the cache; an independently
held old proof also fails full revalidation. Active workers still block
rebinding, and receipt publication still checks its original scene/document.

With OCCT 8.0.1, `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`:

- `cargo test -p om9-app --lib topology_owner_tests -j 2`: 34 passed, two
  ignored (3DM exchange and release timing), including the new radius-5
  fixture. It remeshes after translation, Undo and Redo; a receipt captured
  before the edit remains stale even after rebind. A radius-6 replacement
  evicts the cached proof and rejects an independently held old proof.
- After adding simulated validation-admission refusal checks, the new test
  was rerun alone and passed. Refusal keeps document and scene unchanged and
  preserves Redo after Undo; retry then succeeds.
- `cargo test -p om9-kernel --features 3dm-brep --lib brep::topology_capture::tests -j 2`:
  nine passed, three ignored (two timing, one cross-process probe).

This remains a test-only attested primitive and experimental remesh route. It
does not preserve a proof after Undo removes the whole object, make a source
owner part of production Sphere/history, evict document display, or bound
native OCCT peak. The fixture uses a numeric move through the product
placement transaction, not a desktop pointer gesture or GPU presentation.
