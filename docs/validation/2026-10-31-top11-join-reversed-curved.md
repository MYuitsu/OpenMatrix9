# Top11 Join: reversed and curved surfaces — 2026-10-31

Status: partial automated evidence; not full Top11 or M03-UI acceptance.

The surface conversion no longer rejects the reversed flag. OM9's reverse_surface reverses the poles/knots for U or V and records parity; surface evaluation and normal use those stored parameters. The adapter passes the actual basis unchanged rather than applying another face reversal. Kernel fixtures convert U- and V-reversed planes and compare OCCT display-triangle orientation with the Rust surface normal. The Join fixtures exercise both reversed axes via CMD and toolbar, checking two faces, one shell, six naked edges, area and Undo/Redo. This verifies those concrete representations, not arbitrary imported reversed metadata or final orientation across every sewing configuration.

A separate fixture joins two adjacent rational curved surfaces through CMD and toolbar; it checks shell topology and six sampled points against Rust evaluation using OCCT ray intersections. A tolerance fixture joins planar patches with a 0.0005 mm gap under the template tolerance and refuses a 0.002 mm gap atomically. For the accepted gap, OCCT changes the trimmed boundary: measured area is 12.0015 mm² versus 12 mm² before sewing; underlying parameter bounds remained [0,1] in this fixture. Bounds alone do not prove underlying surface identity. The seam adjustment and general tolerance geometry remain open for closer comparison against the contract.

Windows x64, CARGO_TARGET_DIR=L:\om9-target, -j 2: 78 OCCT kernel tests with 3dm-brep, 48 core curve_ops/top11 tests, 185 app Top11/options/BRep/snapshot tests and 107 no-default tests passed. OCCT uses the project's pinned 8.0.1 SDK; no-default reports existing unrelated warnings. Physical desktop and signed M03-UI acceptance remain open; a prior desktop launch returned access denied. No openNURBS measurement batch was performed.

Source SHA-256: measurement.rs `5b130dd49decef6b79216932ea974f6818b8cdb049138670680f1d4ee8b79210`; kernel tests/l2_02_05.rs `3efeaf7c997ca3b73b567e23f995a75c158d650df90514094b11cdf6ea9de06d`; app tests/top11_brep.rs `877730dc2afce8db1a745dd848aa564deeb2967b733ce7c4ac2f8e1bd50a5c9a`.

Update: [surface guard evidence](2026-10-31-top11-join-surface-guard.md) checks native surface identity, location and serialized representation before/after sewing for the tolerated gap sample; trim behavior remains open.
