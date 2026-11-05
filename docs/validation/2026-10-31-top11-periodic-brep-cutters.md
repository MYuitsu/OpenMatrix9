# Top11 Split/Trim: repeated periodic NURBS and BRep — 2026-10-31

Status: partial automated evidence; Top11 and physical M03-UI remain open.

The OCCT periodic curve constructor expects a different pole/knot encoding from OM9's certified repeated basis. The prior curve/BRep intersection failed with `BSpline curve: # Poles and degree mismatch`; the general measurement/surface-partition adapter failed the same way when the periodic curve was a cutter. Both Rust adapters now certify the repeated basis and send its exact extended nonperiodic representation and active interval to OCCT. Other periodic encodings are explicitly refused; no native C++ was added.

A kernel test verifies four curve/box intersections against the actual box boundary in both orientations and with/without a shifted seam, plus a trimmed interval and invalid repeated-pole rejection. Another kernel test splits a 200 mm² planar face into two positive-area faces using a closed periodic rational NURBS cutter, verifies preserved total area, and rejects an invalid basis. App controller tests use CMD and toolbar for Split of a translated periodic curve by a similarly placed box, with both orientations and shifted seams: five pieces, sampled source membership, total length, .om9 roundtrip and Undo/Redo. Trim tests remove a picked curve interval against the box in those variants and check Undo/Redo; a further test trims a planar surface inside the periodic boundary, checks area and Undo/Redo. Tests exercise controller invocation, not physical pointer/display acceptance.

Windows x64, CARGO_TARGET_DIR=L:\om9-target and -j 2: complete om9-kernel suite with 3dm-brep 156 passed, 1 ignored; selected app Top11/options/BRep/snapshot 195 passed; no-default Top11/options 110 passed. No-default has existing unrelated warnings. These are selected OCCT adapter tests, not openNURBS measurement batches. 3DM interoperability, general periodic encodings, other tolerance and topology cases, aggregate native allocation and physical M03-UI remain open.

Source SHA-256:

- `crates/om9-kernel/src/intersect.rs`: `531e35148f6bc88bffb9d06efa6ef1b173aec52789fce4506448cfb4f88e0d40`
- `crates/om9-kernel/src/measurement.rs`: `b11c52d5672f4aad26c6f2b4343711bedfb18307e0553e00baadf6615a298417`
- `crates/om9-kernel/tests/ellipsoid.rs`: `e38def3946b4406ad3bd994e422d83b0126673fe35582e51f49c0346b1e3e1b8`
- `crates/om9-kernel/tests/l2_01_10.rs`: `9c8fbaefbfffd993f13793279ce234a6b6e1636714f6c42c785946c4ee534853`
- `crates/om9-app/tests/top11_brep.rs`: `b6a625c5bf6df6795d43fed5e4ec01972e062056cd77cfe03c1a8a0989f2da01`
