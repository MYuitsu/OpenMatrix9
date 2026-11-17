# DupBorder: placement of standalone NURBS surfaces

The standalone `Geometry::Surface` branch generated four local isocurves and committed them with identity placement. Thus a 4 × 3 mm surface translated to (10, 20, 30) produced border curves at the origin. The new regression first failed on the expected world boundary from (10, 20, 30) to (14, 20, 30).

The branch now commits the original surface placement on each local NURBS boundary curve using the existing placed-geometry transaction. The source surface is unchanged; all four outputs remain one Undo/Redo transaction. Reopened planar surface fixtures exercise translation and a nonuniform-scale/shear matrix through CMD and menu dispatcher. Four boundary endpoint pairs match independent world-space transformed corners, outputs retain their geometry and placement across a further save/reopen, and Undo/Redo restores entity sets. An output-object limit fixture refuses without changing the coordinator snapshot or preexisting Redo. The B-rep branch was not changed.

`cargo test -p om9-app --test dup_border --test wireframe --test curve_menu --target-dir L:/om9-target -j 2 -- --test-threads=1`: DupBorder 11 passed; wireframe 44 passed, 3 ignored; curve menu 6 passed. `cargo check -p om9-app --no-default-features --target-dir L:/om9-target -j 2` passed with existing warnings. The user approved recording this scope in feature guidance.

The fixtures cover planar standalone surfaces, not all NURBS surfaces, periodic/trimmed geometry, arbitrary placements, downstream commands or desktop clicks. The production code retains the placement for these output entities; callers must honor it when consuming curves.
