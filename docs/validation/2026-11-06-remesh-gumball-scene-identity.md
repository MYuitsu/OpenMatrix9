# Gumball cache follows remeshed scene — 2026-11-06

The Gumball centre cache was keyed by document, epoch and selection, but a
remesh can publish different display buffers with the same document and epoch.
In a radius-5 Sphere fixture, the old cached mesh centre was approximately
[0.002446934, 0, 0], while the replacement coarse scene's displayed mesh
centre was approximately [0.122358709, 0, 0]. The regression failed with the
old key. The cache now also checks the scene publication UUID before reuse;
the same fixture passes and computes its expected centre from the newly
published scene buffers. This preserves the existing displayed-mesh pivot
semantics; it is not a claim that the pivot is the exact analytic centre.

With `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`:

- `cargo test -p om9-app --lib topology_remesh -j 2`: 24 passed, one ignored
  (3DM exchange); includes the red/green Sphere fixture, stale results,
  admission, worker cancellation, scene replacement and Undo tests.
- `cargo test -p om9-app --test gumball_interaction --test sphere_gumball_source -j 2`:
  seven plus one passed. The Sphere test exercises the real Sphere command,
  CPU picking, source-refined snap, STL and Gumball/Undo; it does not request
  the experimental remesh.

The remesh fixture uses the test-only attested primitive path. This fixes a
cache invalidation dependency needed by that path but does not make remesh a
production Sphere feature, release document/Undo display meshes, measure
native peak or verify desktop interaction or GPU images.
