# Top11 Duplicate: preserve prompt after rejected placement — 2026-10-31

Status: PARTIAL_CHECKS_PASSED. Reproduced before the fix: after one copy with UseLastDistance=Yes, selecting the base point as the next destination failed to normalize a zero direction and discarded the pending Duplicate command. The document was unchanged, but the user could not retry.

The controller now retains the pending Duplicate state on an error if no document revision was published. If publication already changed the revision, it does not restore the old prompt and invite an accidental repeat. CMD and toolbar fixture paths reject the zero direction without a document or Undo change, then accept a valid direction, place the previous distance and Undo the new copy. A separate constrained-budget fixture proves refusal leaves the prompt and document intact; Enter then ends the command and Undo removes the prior copy. A synthetic egui viewport fixture tests an invalid click followed by a valid click and Undo.

Windows x64, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`: app Top11/options/BRep/snapshot 151 passed; no-default app 83 passed; the default-feature viewport test passed. This is not proof of all error types, post-publication failure handling, native focus, GPU presentation or human M03-UI acceptance. Other Top11 requirements remain open. No openNURBS measurement batch was performed.

Source SHA-256: `crates/om9-app/src/session/top11.rs` ab0c416101fe1a1fbd6f21ffa0b2a99cdbe935abf9e387a7016b28a4950a1686; `crates/om9-app/src/ui/chrome.rs` 2a0a2ede5ac9f8bd9403b0f8227b0620f7aa0cd3e84f7a94d8aab168a2ddcbf6; `crates/om9-app/tests/top11.rs` 5388ac8251bbcf13739e1463eadfaf8d84ba766e89439c82e03f480780f9c909.
