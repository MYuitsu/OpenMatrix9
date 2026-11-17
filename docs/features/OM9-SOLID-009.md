# OM9-SOLID-009 — Extrude Curve Straight

Standalone OM9 extrudes a selected supported closed planar curve along the active construction-plane normal by the entered height and publishes a B-rep in one transaction. The source curve remains; the output is placed on the current layer. The menu dispatcher and `ExtrudeCrv` route to the same operation. `ExtrudeAll` is a separate command that batches selected curves into one transaction.

As of 2026-11-12, the source curve's placement is carried to the output, with the working direction transformed into local coordinates. Reopened rectangle fixtures cover non-uniform scale, in-plane rotation and shear; two translated sources cover ExtrudeAll, world bounds and Undo/Redo. [Validation and limits](../validation/2026-11-12-extrude-curve-placement.md).

A reopened reflected circle with non-uniform scale also passed the specified YZ construction-plane direction, analytic volume and Undo/Redo fixture.

This is partial fixture evidence. General curves, arbitrary construction planes and affine transforms, imported input, native allocation peak, actual menu clicks and the M03 human UI gate are not certified by this batch.
