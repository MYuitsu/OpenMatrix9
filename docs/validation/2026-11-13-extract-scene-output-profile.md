# Extract Surface output profile at the scene boundary

2026-11-13. Scene objects now retain a transitional BRep output deflection captured during scene construction. Extract Surface obtains this profile from its validated scene object rather than reading a document display in its synchronous single/multi-face paths or its source-only background capture. The witness continues to use the currently published pick display. The output profile is copied with scene snapshots and metadata-only scene updates, and remeshing a witness does not change it. The extra inline scene metadata is covered by existing `SceneObject` size accounting. At present scene creation still seeds this profile from the document's display; a future source-only scene must supply it through representation policy.

With `CARGO_TARGET_DIR=L:\om9-target`, OCCT 8.0.1 runtime and `-j 2`:

- Scene lib: 48 passed. The BRep fixture exercises valid/stale stamp and preserved metadata after scene representation replacement.
- App `--lib extract -- --test-threads=2`: 38 passed, 3 ignored (two large native workloads and one timing probe). A worker fixture and synchronous sphere fixture use a scene remeshed to 1.0 while retaining the original 0.1 output profile, and check source-derived area; other fixtures cover placement, multiple faces, Undo/Redo, cancellation and stale/failure paths. The worker fixture also checks a render snapshot retains the profile.
- Workspace check passed with two existing warnings; app no-default-features check passed with 21 warnings. Desktop interaction and real memory pressure were not tested.

This removes direct document-display profile reads from these Extract Surface consumers; it does not detach Document or Undo from display, define the final representation policy, or establish cold restore or v5. The remaining solid and other consumers still require migration.

SHA-256: `scene.rs` `cb082e109dfa5c60ac0c4ba72e93d2b8a8f0752e36ae6521fca7a1e985ee5f30`; `face_verification.rs` `af0151f2630762e112576f07a68c09985b4e43afa1a933fffc14c1aa39803bad`; `solid.rs` `ffca961656f51353e2e0df0b8e84551b4b46a85389d34aaa4e6c9ac3e3c28113`; `session.rs` `6bdedf3e39ef275cf85bdcd7125c24fc3f07d63a74cd5fd3665ceb7cc4634c93`.
