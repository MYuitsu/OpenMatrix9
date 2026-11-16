# ExtractIsocurve on a public external Rhino 5 B-rep

## Source-ray bridge regression

Batch `external-v5-isocurve-ray-bridge-regression`, package 3: one execution on the existing public V5 fixture passed after the source-ray bridge gained optional face-witness checking. The fixture imports the file as NativeBrep, saves/reopens it, uses a simulated viewport ray on one face, checks five curve samples and source-boundary endpoints, then verifies Undo/Redo. This is one existing fixture, not a new independent model, physical desktop interaction or a proof of the full curve. The adapter and fixture are the ones described below; the test ran in the existing `L:/om9-target-surface` directory with `-j 2`.

One prepared batch remains: `modeling-public-rhino5-generated-exchange` (four fixtures, seven objects), awaiting independent Rhino 5. Packages 2–8 remain open; the total number of future batches is unknown. Next: continue the current Plan A product gaps without repeating unchanged fixtures merely for count.

Batch `external-v5-native-isocurve`, package 3: two executions of one public fixture passed; the second added explicit boundary and nondegeneracy checks. The [openNURBS V5 example](https://github.com/mcneel/opennurbs/blob/eb92af3ba1806b0a34a99aba0d3bda83e3d46083/example_files/V5/v5_my_brep.3dm) was imported as NativeBrep via the admitted exchange host, saved and reopened as OM9. The simulated viewport ray selected a face from +X, after which the controller extracted the default isocurve and verified Undo/Redo with full document snapshots.

For the expected face, the [independently published example source](https://github.com/mcneel/opennurbs/blob/eb92af3ba1806b0a34a99aba0d3bda83e3d46083/example_brep/example_brep.cpp) defines a quadrilateral in x=10. Five sampled output points lay in that quadrilateral, both endpoints lay on its boundary, and the output was nondegenerate. This establishes one selected face and sampled output, not full curve equivalence, physical desktop interaction, all external files, memory peak or physical OOM.

The test fixture is held outside the checkout at `L:/om9-target/opennurbs-v5-my-brep.3dm`, SHA-256 `59a632b9a04c5278053bd4f28d3580bfdc9de160aa9138f45ffe0d2f5b0bcc4c`, and is passed through `OM9_EXTERNAL_3DM_FIXTURE`; the test does not distribute the file. The exchange executable had SHA-256 `8bdd11bcb529b9e55b7eeed5e4e80f5741cb2a4d1e4e1c0bd3040347d4817a6c`.

One prepared batch remains: `modeling-public-rhino5-generated-exchange` (four fixtures, seven objects); packages 2–8 remain open and the total future batch count is unknown. Next continue Plan A and validate other external geometry and physical desktop when available.
