# Rectangular box viewport display — 2026-10-30

The user supplied a viewport image with green shaded box faces, green edges and
face midlines when unselected, and magenta edges and face midlines visible
through the box when selected. This slice changes GPU presentation only; it
does not create model curves or change geometry, selection, picking or Undo.

For a display mesh consisting of six rectangular faces and twelve triangles,
the GPU viewport derives two midpoint guides on each face. It checks the face
identifiers, four vertices, rectangular geometry, and finite nonzero extent;
triangle diagonals are not used as guides. Unselected boxes draw the guides and
edges in the object's existing colour, while selected boxes retain their face
colour and draw magenta edges and guides in a separate on-top overlay. The
overlay follows the selected object's preview transform and is invalidated with
selection and scene state.

The subsequent user-directed global line-colour rule makes unselected edges and
guides green irrespective of the stored object colour; see
`docs/validation/2026-10-30-object-line-colours.md`.

Validation on the default OCCT-enabled target with `CARGO_TARGET_DIR=L:/om9-target`
and `-j 2`:

- `cargo test -p om9-app --lib box_guide_tests`: one real OCCT box passed,
  confirming 12 axis-aligned face midlines and rejecting an incomplete mesh.
- `cargo test -p om9-render --test antialiasing`: two GPU tests passed,
  including depth-occluded normal lines versus the on-top overlay.
- `cargo test -p om9-app --test gpu_navigation_publication
  rectangular_brep_selection`: one GPU viewport test passed for unselected,
  selected, then unselected publication and unchanged document.
- `cargo test -p om9-app --test gpu_navigation_publication`: all four GPU
  publication tests passed, including the new box test.

Limits: recognition currently covers the measured six-face, twelve-triangle
box representation; other rectangular tessellations need a separate fixture.
The on-top pass is not isolated from unrelated occluders, so selected box lines
can also show through another object in front. The GPU checks do not compare a
desktop screenshot with the supplied image and do not establish native-window
visual approval. Matrix's original implementation was not checked.
