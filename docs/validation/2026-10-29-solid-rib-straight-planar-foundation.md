# Rib straight planar foundation — 2026-10-29

The recovered Matrix menu entry `SolidRib` dispatches `! _Rib` (`docs/analysis/matrix-command-layers.json`); that registration establishes routing, not the original algorithm or defaults. The [Rhino Rib reference](https://docs.mcneel.com/rhino/mac/help/en-us/commands/rib.htm) describes planar curve selection, boundary selection, InCurvePlane/NormalToCurvePlane offset and Straight/Tapered modes. Local M03 L2 excludes specialized mechanical Rib, but the explicit user goal covers all recovered Solid commands; this is a limited development foundation rather than a claim that the L2 acceptance is met.

A Rust adapter builds a one-sided mitered strip from an open polyline, orients it toward the supplied plane normal and uses the existing capability-scoped planar Boss operation to join its walls to a bounded planar BRep. This one-sided construction is an OM9 interpretation, not verified Matrix behavior. It does not create a documented complete Rib session.

The test uses a four-unit straight line, two-unit offsets on either side, both source directions and a 10-by-10 planar face at height five. Result: one open joined shell, five faces, eight naked edges and area 152 mm². Zero distance and a profile outside the boundary are refused; the boundary remains unchanged. With `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`, `cargo test -j 2 -p om9-kernel --features 3dm-brep --test l2_01_10` passed 61 tests.

Multiple curves, exact offset semantics, NormalToCurvePlane, tapered mode, curved boundaries, menu/CMD, Undo, preview, desktop and original Matrix behavior remain unimplemented or unverified. Rib remains visible and disabled.
