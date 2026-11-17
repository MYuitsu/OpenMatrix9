# Sphere translation and source bounds — 2026-11-06

The product `Sphere 5` command followed by a free Gumball move exposed an
under-bound: the old general OCCT transform converted the analytic sphere to
spline geometry before `AddOptimal`, producing minimum X 2.0434244897291616
instead of 2 for a translation of [7, -2, 3] mm. The source and object
placement themselves were unchanged. The adapter now applies a pure
translation as an OCCT location so the source surface remains analytic.
General affine transforms continue through the existing GTransform path.

On the pinned OCCT 8.0.1 SDK and linked runtime, with
`CARGO_TARGET_DIR=L:\om9-target` and `-j 2`:

- `cargo test -p om9-app --test sphere_gumball_source -j 2`: one test passed.
  It runs the command, translates by [7, -2, 3], and checks source bytes,
  topology IDs, shared display, analytic bounds (1e-6 mm), and scene placement
  through two Undo and two Redo operations. The earlier run failed at the
  minimum-X assertion above. An earlier compile attempt used a wrong fixture
  field name and was corrected.
- `cargo test -p om9-app --test measure brep_bounding_box_uses_source_and_world_affine_not_display_mesh -j 2`:
  one passed for the existing two-profile affine-sphere fixture.
- `cargo test -p om9-kernel --features 3dm-brep --test l2_01_10 -j 2`:
  75 passed. Running the latter without `3dm-brep` first failed to link the
  3DM bridge symbols; the corrected feature configuration passed.

The adapter source SHA-256 at this run was
`f8ae24890b7cc2ddffd3f733a5b0f72781bda6a3f5854638bb86987aa87f271b`.
The test operates through the production controller and CPU scene, not desktop
pointer/GPU interaction. It proves neither bounds for arbitrary placements and
shapes nor any universal tolerance guarantee. The display is still retained by
document and Undo; the fixture intentionally holds another display reference,
so it does not test eviction, remesh, memory peak or source-only ownership.
The existing open OM9 session and drawing were not touched.

## Controller picking, snap and export follow-up

The same production-command fixture additionally checks that CPU face picking
at the original location stops hitting the sphere after the move, that the
face ID at the moved location is preserved, and that pre-move and pre-Undo
pick receipts are rejected after the respective scene changes. A surface snap
at the moved location is refined to the radius-5 canonical surface within
1e-6 mm; the pre-Undo snap is rejected after Undo. At export profile 0.05 mm /
0.3 rad, STL bytes differ after translation and return byte-for-byte to their
respective original/moved values after Undo/Redo. Command:
`cargo test -p om9-app --test sphere_gumball_source -j 2`: one passed.
These are CPU/controller checks for one sphere and one location. The byte
comparison is not an independent geometric-error oracle, and no remesh,
cache eviction, desktop pointer event or GPU image was exercised.
