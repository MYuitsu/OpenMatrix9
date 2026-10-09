# Independent periodic FCStd fixture review — 2026-10-10

Baseline: `83a806c`. Reviewed increment: `tests/rhino_core_periodic_persistence_smoke.FCMacro`, final SHA-256 `a072dc1ca13d83471d2ece28be630a0d65fa9d4b6721200b57df2eb16a703aa2`, and the fixture launcher's process-record addition. Result: no blocking fixture defect found for the authored `RCORE-01.T09` acceptance scope. The final matching warm and cold artifacts contain 204 and 102 passing assertions and completed process exit zero. The earlier 203/101 runs used a different macro hash and are superseded by the added inventory-preservation assertions. The first 204/102 pair is also superseded as acceptance evidence by the final pair with persisted process provenance.

The reviewer read source and persisted artifacts, compared hashes/raw files, parsed the macro without executing it, and independently expanded the oracle polynomials. Only this review document was written. No production/test source was edited; no build, GUI run or further delegation was performed. This is a fixture/reuse review, not a claim that production persistence behavior changed.

## Literal requirement and scope

The current specification at `OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/01-object-model-kernel.md:233` requires FCStd save/reload of a periodic curve and seam/hole face, comparing type, topology, basis, world bounds and deviation, with evidence beyond shaded images. The plan and earlier reuse audit correctly choose a true periodic curve plus two separate existing faces: a seam face and a hole face. The existing circle remains a rational-NURBS control. The periodic curve is authored directly in the FCStd candidate, so the fixture does not conflate FCStd persistence with a 3DM converter's periodic canonicalization.

This supports acceptance of the four authored T09 fixtures. It does not prove all-family persistence, full `RCORE-01.T08`, `T11`, T10 parameter mapping or all P0. The older kernel result's explicit limitations remain valid for that older fixture; the new fourth object closes its specific missing periodic-curve FCStd evidence.

The Python exception to Rust-first is appropriate here: the fixture exercises the actual FreeCAD Python host APIs and GUI lifecycle. No production solver, serializer, representation model or business decision was added in Python.

## Source and evidence matching

Both final run folders record the final macro hash and module hash in `merge-native-module.json`. The actual module file at `H:/FreeCAD-src/build/reusable-history-runtime/bin/OpenMatrix9Gui.pyd` independently hashes to the recorded `8b03c89907672d0579544a1fce934632c231ad5b70e23612fc09bd3c6b790598`.

Final warm run `3e28cafa09074a9bb4f432e6a100d88d` reports `ok: true` and 204 checks, all true. Final cold run `d350bdd11328495aaca38bf5967dbf65` reports `ok: true` and 102 checks, all true. Cold names the exact warm `PeriodicKernel.FCStd`, whose actual bytes hash to `4854be11ed9531f23c8ae9616a0cd58c2f27b45dcc616628a885876777006456`; its recorded warm baseline hash equals the actual warm results hash. The legacy baseline result/FCStd hashes also match their actual files. Both final periodic runs' full snapshot/inventory trees and every raw BRep equal the previously audited 204/102 pair, independently confirming that the provenance replay did not change the geometric observations.

The unchanged old macro control passed 142 warm and 79 cold assertions on the same recorded module. Ten helper functions are copied source-exactly from the old macro: `check`, `digest`, `shape_digest`, `xyz`, `curve_basis`, `surface_basis`, `topology`, `boundary_components`, `snapshot` and `close_numeric_tree`. Their source segments were independently compared. The old macro itself retains SHA-256 `5032a961fb9870a916e46faec6d7caf95c63d0a349d2a7ac341c4a22ee34e4d9`.

The launcher's change at `tests/run_menu_smoke.ps1:26–36` records the PID, launch/observation UTC, actual wait completion and native exit code before reading the assertion report. It retains the existing timeout and nonzero-exit rejection; on timeout it records `wait_completed: false` and a null exit rather than inventing completion. It retains hidden `Start-Process`, isolated per-run output/profile paths, the same executable/macro arguments and environment restoration. No production command is affected. The completed branch was runtime-exercised by all four final runs; this review did not exercise a real timeout.

The actual persisted `process.json` records show completed waits and exit zero for legacy warm/cold PIDs 2732/34232 and periodic warm/cold PIDs 21536/82080. They name `H:/FreeCAD-src/build/relWithDebInfo/bin/FreeCAD.exe` and the `merge_replay.FCMacro` wrapper; each paired module record names and hashes the selected actual fixture macro. These distinct native processes and the cold exact-file/baseline checks provide concrete fresh-process provenance. Success is established independently of the JSON assertion count.

## True periodic basis and world placement

The constructor at macro lines 298–300 supplies compact periodic degree-2 data: poles `(0,0,0)`, `(1,2,0)`, `(2,0,0)`; knots `[0,1,2,3]`; multiplicities `[1,1,1,1]`; weights `[1,1,1]`; periodic true. The pre-save predicate at lines 207–214 requires an actual `Part.BSplineCurve`, native `isPeriodic()`, the exact compact data, degree 2, nonrational status and full edge domain `[0,3]`. It therefore rejects a merely closed nonperiodic substitute.

All three persisted stages—before save, warm reopen and cold reopen—contain the same native basis, one valid closed Edge and one vertex, Forward orientation and full domain. They retain translation `(7,-5,2)` and identity rotation. Local support poles remain the authored poles; world support poles are `(7,-5,2)`, `(8,-3,2)`, `(9,-5,2)`. The independent world-locus and extrema checks would reject an omitted or twice-applied translation. This is the authored rigid placement case, not a general nested-instance transform certificate.

`periodic_snapshot` deep-copies native geometry and explicitly obtains world shape with `Part.getShape(..., transform=True)`. It resets the copied local shape placement to inspect local support. It does not treat an unknown descriptor transform as known, mutate the source placement, or route through display mesh.

## Independent oracle and numerical limits

For the three cyclic spans the reviewer independently expanded the formula into:

| Span | Local x | Local y |
| --- | --- | --- |
| 0 | `s + 1/2` | `-2s² + 2s + 1` |
| 1 | `3/2 + s - 3s²/2` | `(1-s)²` |
| 2 | `1 - 2s + 3s²/2` | `s²` |

All have z zero and `s ∈ [0,1]`. These give local x extrema `1/3` and `5/3`, y extrema zero and `3/2`, with the four authored witnesses exactly as stated in the reuse audit. Translation gives world bounds `[22/3,-5,2,26/3,-7/2,2]`.

Every stage stores 775 points and 775 residuals: three spans with 257 observations each, plus four exact extrema. The stored points equal the independently expanded polynomials exactly for this dyadic sample schedule. Every recorded residual is finite; the maximum is `8.881784197001252e-16 mm` at each stage, below the unchanged `1e-7 mm` predicate. The oracle queries distance from independently authored world points to the native curve, so no assumption about OCCT starting-span ordinal or a parameter map is needed.

This is a one-direction sampled point-to-locus observation. It is not a continuous maximum-deviation or bidirectional Hausdorff certificate. The report expressly sets `sampled_only: true` and keeps that qualification. Geometry-only optimal bounding boxes are compared separately. The actual bounds are `[7.333333233333333,-5.0000001,1.9999999,8.666666766666665,-3.4999999,2.0000001]`; the declared bbox allowance `1e-7 + 64*ulp(10)` is `1.0000011368683772e-7 mm`. The approximately `1e-7` native box padding is not substituted for the point-to-locus bound. Numerical persistence comparisons retain `1e-12` rather than being relaxed to make reload pass.

## Retained face topology and inspection claims

The hole face stays a valid Face with no solid, five unique edges/five uses and two loops. The outer loop has four Forward uses; the inner circle has one Reversed use. Its hole-center UV classifier rejects the center, center-to-face distance is R3, and a retained point belongs to the face. The 20×20 bounds, supporting surface and 3D edge bases persist.

The cylinder stays a valid open Face with no solid. It has three unique edge identities and four ordered uses. The seam identity is used twice with opposite orientations; the two other identities are free boundaries. Their connected components remain two R4 circular boundaries at z0 and z7. The converted supporting surface is U-closed and nonperiodic, which is recorded rather than misreported as analytic periodic preservation or solid status. The true-periodic requirement belongs to the separate authored curve.

`compare_snapshots` checks type, validity/closure, face/solid counts, curve/surface basis, domains/orientations, placement, world bounds and ordered face-loop/use topology after each reopen. Pcurve observations contain native class and domain, not a separate full numeric pcurve basis extraction. For these actual runs, the reviewer additionally compared all four raw exported object BReps byte-for-byte across before-save, warm and cold stages; all are identical, including the serialized pcurve data. That is concrete evidence for these fixtures, not a general durable subelement-identity guarantee.

Final inventories are exactly identical: four intended `Part::Feature` objects, exact object names/types/property-name lists and UndoCount zero. The new warm/cold assertions compare against before-save inventory. Each snapshot also compares inventory and exact source shape stream fingerprints before/after inspection. These support no read-created object/property/Undo and no CAD change during the inspected snapshot calls. They do not compare every property's value; shape/basis/placement are checked separately, while arbitrary material/visibility/expression preservation is outside the claim.

Warm guards are initially valid, all four old tokens reject after close/reopen with the same document name, and fresh tokens accept. Cold fresh tokens accept. This is the existing session/incarnation lifecycle control. It is not delayed-preview cancellation, native handle release instrumentation or full T08/T11.

## Preserved failures and fixture repair

The failed runs `343d9efcb3be472ba68c56c16a0635db`, `703103faa9834f1c9a7ef8d0b9aa0db2`, `504ae80f79a84b9c9180aef8fc6b3323` and `4e2fae8ea93a421ca330181a72263858` retain the failed source-copy predicate. The later raw captures in 504ae/4e2fae show a single serialized circle-root flag line changing `1101100 → 0101100` (the first/root Free flag), with all other BRep lines unchanged and source inventory unchanged. In 4e2fae, `copy_stages.after_new_document` already has the changed hash before any copy or assignment; later copy/assignment/recompute stages retain it.

Earlier 703103 raw before/after files are identical despite its recorded earlier baseline hash mismatch, so that folder alone is not the decisive raw one-bit diagnosis. The later stage-aware capture and final pre-target/before-copy files establish the phase. The final fixture retains the exact pre-target bytes and activation report, then establishes its copy baseline after target-document initialization. It does not normalize root flags or delete differing bytes. Final raw `source-before-*` and `source-after-*` files are exactly equal for all four source objects. This is a justified isolation of the copy operation; it does not certify that document initialization leaves every native topological flag unchanged or that T11 is complete.

Failure `550c5e3deaeb447abf4e56a58cfdc244` is a separate fixture lifetime error: reading `source.Name` after `App.closeDocument` raises `ReferenceError`. The current macro retains `source_name` before closure at line 310. The passing runs exercise that repair. These are retained fixture diagnostics, not a new production RED→GREEN behavior.

Two serialization fingerprints must remain distinct. Native `exportBrep(filename)` adds `DBRep_DrawableShape\n`, while `exportBrepToString` writes the shape-set stream (`src/Mod/Part/App/TopoShape.cpp:939–971`). Removing only that Draw wrapper header from the raw periodic exports reproduces the reported stream hashes. Source diagnostic `write_text` files use Windows CRLF; the predicate hashes the unmodified UTF-8 native string. The reviewer used actual raw bytes for before/after equality and records raw file hashes below, rather than conflating these formats or weakening the predicate.

## Fingerprints

Paths in the first table are relative to `H:/FreeCAD-src/Mod/OpenMatrix9`, except explicitly absolute references.

| Source/reference | SHA-256 |
| --- | --- |
| `tests/rhino_core_periodic_persistence_smoke.FCMacro` | `a072dc1ca13d83471d2ece28be630a0d65fa9d4b6721200b57df2eb16a703aa2` |
| `tests/rhino_core_kernel_contract_smoke.FCMacro` | `5032a961fb9870a916e46faec6d7caf95c63d0a349d2a7ac341c4a22ee34e4d9` |
| `tests/run_menu_smoke.ps1` | `3649ae126bd39306a384e77bd0f46d4a8a3f8f227d5cb3e82161a4baec6bb9bd` |
| `docs/superpowers/plans/2026-10-10-rhino-core-periodic-persistence.md` | `d9728215c27db77fa141eddf9b08d1284b6573f7882a51e1651b1606b5c57bb8` |
| `OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/01-object-model-kernel.md` | `54ce8187e013a456e41ebdec3af15a005df3de8156dccb51d94a06256738d143` |
| `H:/FreeCAD-src/build/p0-next-kernel-reuse.md` | `b18c860e0eb31ddf4fa034eec8a4e4be6a510d2ce5026ea901f6529eab0bc409` |
| `H:/FreeCAD-src/src/Mod/Part/App/TopoShape.cpp` | `2f7de58f556e3ce2cb0877ebdb1a83536d1e73be2d8a2b9243056083f0619e5e` |
| `H:/FreeCAD-src/build/reusable-history-runtime/bin/OpenMatrix9Gui.pyd` | `8b03c89907672d0579544a1fce934632c231ad5b70e23612fc09bd3c6b790598` |

Evidence root: `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1`. W = `3e28cafa09074a9bb4f432e6a100d88d`; C = `d350bdd11328495aaca38bf5967dbf65`; LW = `4605d00ec2444a74a2395a3df6143390`; LC = `594892c738464933817651e9fdf53aee`.

| Artifact | SHA-256 |
| --- | --- |
| `W/results.json` | `8558c576ed2ba12a78c00002123be7ee03e9ab298e4f3dcf223e5f4ecc379cc8` |
| `C/results.json` | `faeb790b61fbcd5ceebcfebd1a71b226445241f41e5ac99783f7eba6f221f2e8` |
| `W/process.json` | `c2c8994361dce5ddd7519b4297c603a839b8ef39119f66be98e152200c72e3a6` |
| `C/process.json` | `a5f4f97250a59b0165186900dc8a1d1813d909903704ae9a5231844e6f6b2d1f` |
| `W/merge-native-module.json` and `C/merge-native-module.json` | `228b7f5909f9dc458a1a74e5e15f604173b9b4d5495f0e7e72a956c384a07c7e` |
| `W/PeriodicKernel.FCStd` | `4854be11ed9531f23c8ae9616a0cd58c2f27b45dcc616628a885876777006456` |
| `LW/results.json` | `57188b51030e55b8b46b320c4fa792e0e40764905801057f30182c3f82d29d1e` |
| `LC/results.json` | `52aa443cfd048e8b4d92c7982ac65d9fc79f6b25acc91892353271ee26f41478` |
| `LW/process.json` | `fb8adf004af104310d91217e7d74b28bcd4d06feb0c93ccbea7ba83d26ff8088` |
| `LC/process.json` | `73667dcd1ca02d1274718b9f47e1020953371a0cb7f49258159857521cd12433` |
| `LW/merge-native-module.json` and `LC/merge-native-module.json` | `575394618a533053d89077539e79c42f51b2e89ee6520ce7a42744e4dd3022c5` |
| `LW/exact-kernel-candidate.FCStd` | `2beb85fde1eab7f7f1eea6acde93cb7134a19d153554bac28c27d9403330e2da` |
| `LW/exact-kernel-candidate.3dm` | `07b9ca7a83b1c8a1517ba01f389ba362a1efeef203fd0bc8549939b0141622f9` |
| `W/before-FCStd-save-periodic-local.brep` | `5e840aba26a49c2b91af84e44427a04039cd76e9ecde01aefc4955d321541684` |
| `W/before-FCStd-save-periodic-world.brep` | `fef06393b38a9d7c876a6fa5dec06bccfcd6a1c073d476a10a324f44ec87859b` |
| `W/before-FCStd-save-RhinoObject.brep` | `d4660ee3e32ac658f8b041c8873909effe5360c3f0113074af881218e164dba1` |
| `W/before-FCStd-save-RhinoObject001.brep` | `fd35c159347fd60457f471f90a720709f63097e76041ac574062520fda1952d0` |
| `W/before-FCStd-save-RhinoObject002.brep` | `cdfe5bc773abd76099bf33635bed64bd9ed541b5081556817971c9e81b631a5b` |
| `W/before-FCStd-save-PeriodicQ2.brep` | `377e2abd81c472ee02e32e63b165933e81727eb0301d0e29c1af38d6fdf4aa65` |
| `W/source-pre-target-RhinoObject.brep` | `70e72d6c33a69471bb717cafdb92eb6779c5bbb8c32dedca20fa61c90e3c8070` |
| `W/source-before-RhinoObject.brep` and `W/source-after-RhinoObject.brep` | `73586bf247aeadaff2135f91119856fa57ea6dfed9ef52d4014cebdd26bfde16` |

Each before-save periodic local/world raw export has the same respective hash as its warm and cold counterpart. Each of the four before-save raw object BReps likewise equals its corresponding warm/cold raw object artifact.

| Retained failed result | SHA-256 |
| --- | --- |
| `343d9efcb3be472ba68c56c16a0635db/results.json` | `9d630e2498b2550504a81eb406360c60bad65b69f80bb520c93a3c60e1b4e7a1` |
| `703103faa9834f1c9a7ef8d0b9aa0db2/results.json` | `89e9f54ae851bb94028f0f8ac19bfd0dfd3621f65afef4deb77c6e85d7d8ddd5` |
| `504ae80f79a84b9c9180aef8fc6b3323/results.json` | `798d97a877ad0607740d4336bbc00990a86cea5dd1c2671e3673d9d298b27337` |
| `4e2fae8ea93a421ca330181a72263858/results.json` | `5188195c8080b7e3418069dc5dc4be5b3dff5e00c559eb53fb814cb0ce18ebe7` |
| `550c5e3deaeb447abf4e56a58cfdc244/results.json` | `4c68ee174865b9264f9c5e412cdf05a68e8f475dd244cc468ac5c83bfbedc345` |
