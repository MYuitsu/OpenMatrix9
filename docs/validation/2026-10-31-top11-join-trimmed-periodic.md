# Top11 Join: trimmed certified periodic NURBS — 2026-10-31

Status: partial automated evidence, not complete Top11 or M03-UI acceptance.

Certification now distinguishes the full repeated periodic basis from its active interval. Certified repeated-pole/weight and knot bases can be evaluated and displayed when trimmed; the full-domain certification API still requires the entire support. Evaluation applies the periodic seam continuity limit at actual support boundaries, not at arbitrary trim endpoints. Join extracts the active interval exactly and marks the edited output nonperiodic before midpoint seam adjustment.

The core fixture checks three trimmed intervals, both orientations and each adjusted end at 201 samples, with coordinate error under 1e-10 before adjustment. It checks ordinary trim endpoint D2 and rejects D2 at the true quadratic periodic seam; full-domain certification still rejects trimmed input. The app fixture includes a trimmed periodic rational curve in both orientations through CMD/toolbar, checks midpoint, 101 sampled displacements <= 0.000401 for a 0.0008 gap, .om9 roundtrip and Undo/Redo. Controller selection exercises scene admission; it is not physical viewport acceptance.

Windows x64, CARGO_TARGET_DIR=L:\om9-target, -j 2: 203 core unit tests, 48 core curve_ops/top11 integration tests, the om9-scene test suite, 165 app Top11/options/BRep/snapshot tests and 94 no-default Top11/options tests passed. No-default reports existing unrelated warnings.

Other periodic representations, discontinuous boundaries, extreme precision, aggregate memory, broader 3DM interoperability and physical M03-UI acceptance remain open. No native or openNURBS measurement was performed.

Source SHA-256: geometry.rs `cf817b1f0d704b752fd4b83e09e1fe68e41227823ff4b99d35e0fc09be3c5493`; evaluation.rs `5921a0ab40ad9cfd9239661aa11273eefeb14c66878e4497e3b669ed2e9d96d8`; curve_ops.rs `7f865036dcc97b4f9d6b74750ba44720084e9b886f51e421f749c8de800a5777`; scene.rs `2e3b7a54e32ec4f9238c20a200e114b099fa60ea72e2b47924c15fc1c7970ae4`; app tests/top11.rs `eecc3837b2070aee7fa65724d502bec5f14e0d6604bffa238b837b9ec15df9f8`.

Update: [discontinuous-start evidence](2026-10-31-top11-join-discontinuous-start.md) covers an active lower boundary at a full-multiplicity knot; the upper-boundary case remains unsupported.
