# B-rep source world bounds — 2026-11-06

The `BoundingBox` command previously used display points for OCCT B-reps.
It now reads the source in OCCT, applies the entity affine transform and uses
`BRepBndLib::AddOptimal` with triangulation disabled. `CenterObject` uses the
same world bounds, so an existing placement is considered. In builds without
OCCT, B-rep bounds are refused instead of presenting display bounds as exact.

With OCCT 8.0.1, `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`,
`cargo test --locked -p om9-app --test measure --test utilities` passed 8 + 6
tests. An independently calculated affine sphere fixture compares the bounds
at display deflections 0.4 and 0.01 mm, and a translated sphere fixture checks
centering and Undo. `cargo check --locked -p om9-app --no-default-features`
passed with existing warnings.

Source bounds validation now checks the source and topology rather than
traversing display geometry. A further fixture permits zero display points in
the test budget: full B-rep validation refuses the displayed sphere, while
source bounds still match its radius. This tests the validation path only;
the document and Undo still retain the display allocation.

`NativeBrep` bounds now convert its retained 3DM payload using the document
tolerance and compute the result from that source. A 4 × 5 × 6 mm box fixture
retains its expected translated bounds even after the display mesh is shifted;
invalid payload and a missing document tolerance are refused. The interactive
path reserves and accounts the Rust-owned converted-source buffer and releases
the reservation after use. This fixture is not a broad 3DM conversion or
geometric deviation certification.

These are fixture-specific numerical results, not a universal tolerance
certificate. OCCT's internal memory and synchronous UI latency are not bounded
or measured here. Other geometry still uses its previous portable bounds path;
those bounds must not be interpreted as proven exact.
Complex topology, extreme coordinates, and the desktop remain unverified.
The public compatibility wrapper retains its default `Limits` and does not own
a session reservation; interactive commands pass session limits and an account.
P1, v5 and display controls remain open.
