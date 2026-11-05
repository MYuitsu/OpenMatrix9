# Top11 Split Point: reseamed NURBS — 2026-10-31

Status: partial automated evidence; complete Top11 and physical M03-UI remain open. This supersedes the prior periodic Split Point record's reseamed-NURBS limitation for the fixtures below.

The previous fraction evaluator ignored the stored NURBS seam. Fraction evaluation now starts at the seam and follows curve orientation; native parameters remain native. Set Seam composes with an existing seam and orientation. Closest-point knot candidates and the native BRep intersection-fraction adapter use the same mapping. Closest-point refinement now considers both endpoint neighborhoods when the best coarse candidate is an endpoint, including points close to a closed seam.

Parameter trimming materializes one or two exact nonperiodic intervals across the original seam, without refitting, after validating underlying closure. Split Point uses this path for tested full-domain periodic rational curves and closed nonperiodic NURBS ellipses; invalid open records with a seam are refused. A seam-spanning result can be a polycurve with segment-uniform parameterization; the direct split_curve API still refuses seamed inputs because it cannot promise affine parameterization of both pieces. The operation preserves tested geometry, not a claim of unchanged parametrization for a polycurve output.

Core fixtures test both orientations, successive seam changes, 101 evaluation samples, projection around the seam, sampled bidirectional geometric coverage of split pieces, total length, and invalid open bases. Controller fixtures exercise CMD and toolbar, both orientations, output endpoints and Undo/Redo; periodic fixture includes translated placement and .om9 roundtrip. An OCCT box/ellipse intersection fixture tests four intersection fractions in both directions. The existing Slab test was updated to compare the normalized curve at the shifted fraction instead of assuming the original seam was ignored. These are automated checks, not physical viewport acceptance.

Windows x64; CARGO_TARGET_DIR=L:\om9-target; -j 2: complete om9-core suite 954 passed, 2 ignored; selected OCCT kernel suites 85 passed; app Top11/options/BRep/snapshot/solid 386 passed, 1 ignored; no-default Top11/options 110 passed. The initial Slab regression failed under its old seam-ignored oracle; after correction all listed suites passed. No-default emits existing unrelated warnings. Native code was not changed; the Rust adapter maps output from the selected OCCT API. No openNURBS measurement was performed.

Other periodic representations, extreme knot scales, seam continuity and closure tolerance, general projection guarantees, direct affine split, 3DM interoperability and physical M03-UI remain open. An imported open curve carrying a seam remains representable in the document but is refused by this split path.

Source SHA-256:

- `crates/om9-core/src/evaluation.rs`: `2e961987bcd92e4e50d7d40fb43a76041473ecf45a12cde154639cb1f80f316f`
- `crates/om9-core/src/curve_ops.rs`: `c2831a091e92b11d4d72d89cd2a58eea05326c20e148cf2ff65150295357df48`
- `crates/om9-core/src/curve_surface_ops.rs`: `ef45cd79935308ce54edf886c3ff9adfdc75ae25b00d9094b732b7fc72f6a9fb`
- `crates/om9-core/tests/top11.rs`: `82fac6a646492b52438c2186b29af463c164b08b1714f9a68f1e4ec37495838f`
- `crates/om9-kernel/src/intersect.rs`: `b5999c849be4625283ce57d6b173c64beda55bc920a3c911c4ffa1b3baba7289`
- `crates/om9-kernel/tests/ellipsoid.rs`: `43964c164182b3014fa60253dd59f02211f90d1bef09d6424263935817a8c118`
- `crates/om9-app/src/session/top11.rs`: `ec0c402446e45f073dfdbe0e77b4b4a4e342e1d8a47aa09f6241ced812d064a8`
- `crates/om9-app/tests/top11.rs`: `2e95d884efd637fa4ec6a35b21395c0ef195cd3369f30d1b15f6048ec2be185b`
- `crates/om9-app/tests/solid.rs`: `518d420277aa63b34b1b9c7cca245c2d4d77f898f8d84e58659936d0d14cd410`
