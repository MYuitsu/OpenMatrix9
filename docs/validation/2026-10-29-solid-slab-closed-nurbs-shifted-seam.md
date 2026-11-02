# Slab closed NURBS shifted seam — 2026-10-29

The hidden session and closed NURBS foundation accept a shifted seam on a full-domain, nonperiodic closed NURBS. They evaluate the same closed basis at its original seam for side selection and extrusion, without modifying the source entity. A shifted seam on a nonclosed basis and a shifted seam on a partial basis domain are refused. Periodic records remain unsupported.

A rational ellipse with seam 0.17 exercises preview, side/height input, BRep volume against an independent perimeter formula and Undo in forward and reversed directions. A deliberately open basis with a seam is refused without document or Undo mutation. With `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`, `cargo test -j 2 -p om9-app --test solid solid_slab_` passed 30 tests and `cargo test -j 2 -p om9-app --test solid_difference_ui viewport_slab_development_` passed 13 tests.

The shifted-seam fixture uses controller input, not viewport or desktop interaction. Offset fit and topology are sampled, other closed bases and original Matrix semantics remain unverified, and public Slab remains disabled.
