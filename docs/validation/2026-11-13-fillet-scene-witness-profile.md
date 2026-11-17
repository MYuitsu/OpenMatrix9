# Fillet scene witness profile

2026-11-13. This is a scoped consumer correction on the path to source/display separation, not completion of C0–C5.

The Fillet controller obtains its edge witness from the currently published scene. It now carries that display's deflection to native witness verification for endpoint lookup, projection, shared-endpoint classification, preview and direct fillet. The fillet output tessellation deflection remains a separate argument. The handle for a closed edge is also positioned from the currently published scene display. Previously these paths used the document B-rep's display or its profile, which can differ after an attested scene remesh. This correction does not make the document or Undo source-only.

The regression creates an attested radius-2, height-5 cylinder with document deflection 0.1, publishes scene remeshes at 0.04 and 1.0, and uses a top circular edge from each scene. In the 1.0 case, source endpoint verification using the old document profile explicitly fails while verification using the scene profile succeeds. Both direct numeric FilletEdge and the picked-edge controller path succeed for both profiles; the picked path also checks the closed-edge handle position against the published scene segment. Each result's native volume matches an independently invoked source-edge fillet, and Undo/Redo restore the before/after documents. The fixture is symmetric, so volume is not a complete independent spatial oracle; the selected scene segment is checked against the native source by the command.

## Validation

Default features, OCCT SDK runtime on PATH, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -p om9-app -j 2 --lib fillet_uses_published_witness_profile_after_remesh -- --nocapture`: 1 passed (four profile/invocation combinations).
- `cargo test -p om9-app -j 2 --lib fillet_ -- --nocapture`: 12 passed, including the fixture above and existing reordered-edge, recovery and command regressions.
- `cargo test -p om9-app -j 2 --test solid fillet -- --nocapture`: 1 passed.
- `cargo check -p om9-app -j 2 --no-default-features`: passed, 21 warnings. Public source audit: 15,990 source files, 594 bindings, 593 SVGs, no errors. `git diff --check` passed with CRLF notices. Artifact pruning removed zero runs and retained 169 cited older runs.

Physical desktop/GPU, non-circular edges under mismatched profiles, shared-endpoint and point-projection profile changes, native peak and physical OOM have not been independently established by this fixture. Source-only canonical ownership/history, eviction, v5, ordinary scene and remaining background paths remain open. Next: continue C0 consumer disposition and C1 representation ownership with native topology proof and accounting, then cold restore and codec migration as specified in the source/display plan.

SHA-256: `editing.rs` `48372cd49406b631125fe92cf4eefada3d98ddce401aad258feb769b6344f4cf`; `solid.rs` `7def6debe39ee13ad33cc2a11f4b4b2f8e97143ea4ac4db30a90f880fad28649`; `solid_boolean.rs` `16c40eb5689f5e198c401a9b644f47e8dc0db7874f211548ea1280e1ee322a1c`; `session.rs` `687b464197b5e22a886554fcd0460689b71bc69d77ef99743f54423a2fa09ac5`.
