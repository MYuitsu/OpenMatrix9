# Top11 Join: trim starting at a discontinuous knot — 2026-10-31

Status: partial automated evidence; not complete Top11 or M03-UI acceptance.

At an internal full-multiplicity knot, evaluation selects the right-hand piece. When the active NURBS interval begins at such a knot, Join now discards the inactive left-hand piece rather than attempting an ordinary shared-pole split; this preserves the active shape before midpoint adjustment. An active interval ending at such a discontinuity remains unsupported for adjustment, since its endpoint uses the right-hand value while its interior approaches the left-hand value.

A quadratic rational fixture with a large jump checks both directions and both edited endpoints at 101 samples, and verifies that the upper boundary at the jump is refused. The app fixture covers the start-boundary case in both orientations through CMD and toolbar, checking midpoint, sampled displacement <= 0.000401 for a 0.0008 gap, .om9 roundtrip and Undo/Redo. The fixture does not prove arbitrary discontinuities or numerical scales.

Windows x64, CARGO_TARGET_DIR=L:\om9-target, -j 2: seven core restriction unit tests, 48 core curve_ops/top11 integration tests, 165 app Top11/options/BRep/snapshot tests and 94 no-default Top11/options tests passed. No-default reports existing unrelated warnings.

Upper discontinuous boundaries, other periodic forms, extreme precision, aggregate memory, wider geometry, broader 3DM and physical M03-UI acceptance remain open. No native or openNURBS measurement was performed.

Source SHA-256: curve_ops.rs `bc2c9929b9b27a28f35751e222332ecea9bb01ceab45ddf70fd27341ca1bef76`; app tests/top11.rs `49ba9bf9ec70b5030d21e8b6e0d8a78ef5d28637f588ac83e043e6773d391bef`.
