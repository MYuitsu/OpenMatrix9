# Top11 Trim: multiple multi-face BRep targets

The multi-object Trim session retains original world-space edges separately for each selected open multi-face BRep. When a surface region is removed, its own original edges are protected during unification; other selected surface partitions and curve intersections are recomputed before the transaction. Original edges are reused when a later curve edit causes repartition. At the time of this record solid BRep targets were refused in this branch.

The controller fixture selects two perpendicular BReps, each with two sewn coplanar faces, and a line. It trims both BReps and the line, checks that each BRep retains two authored faces, verifies areas including 30 and 100 mm² on the first BRep, and performs four Undo/Redo operations. A simulated egui fixture performs four picks across Looking Down and Through Finger and verifies both face counts, areas and Undo. This is not physical desktop or GPU acceptance.

Windows MSVC, pinned OCCT 8.0.1 runtime and `CARGO_TARGET_DIR=L:\om9-target`: Top11/options/BRep suites passed 97 tests and, after consolidating the original-edge preparation helper, both multi-face controller tests passed again. No-default Top11/options passed 65 tests; the simulated multi-BRep viewport fixture passed; public source audit reported zero errors.

SHA-256: `session/top11.rs` 5c8164ca175f232cbe49f024900196cc7fdcf359acca1153a873aca5afbf53ac; `ui/chrome.rs` 7df9563040676a918be665d985f31cc3276e8c3048142d4a9211a65863b0c5f3; `tests/top11_brep.rs` 54b39ea4863246ee80b5de874371c6d983dd6a42c26263b7fce414db310637e8.

The original-edge classification remains bounded pairwise OCCT geometry comparison, with no general proof for curved/periodic, self-overlapping or affine cases and no performance benchmark. The fixture uses identity placement and planar faces. Limited solid target support added after this record is described in `2026-10-30-top11-trim-solid-boundary.md`; .om9/3DM round trips and physical desktop acceptance remain open.
