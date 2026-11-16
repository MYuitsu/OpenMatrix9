# Plan A/B scope against manuals and recovered command bindings

Date: 2026-11-13. Derived behavioral review, not recovered source or a full parity certification. No runtime behavior changed and no geometry tests were rerun for this review. The authored summary is stored in docs/validation because the public-source audit excludes the analysis directory; no raw recovered listing is included.

## Sources and confidence

Read the normalized Matrix 8 Book 1 sections under the M03 milestone contracts: Duplicate Edge/Border p165, Extract Isocurve p154, Extract Wire Frame p173, Boolean Union p223 and Boolean Two Objects p224. These pages contain adjacent columns; unrelated Tween/Offset/Soft Edit/Cap text is not automatically an option of the selected command. In particular the DupEdge page's Number/MatchMethod paragraphs are insufficient evidence for adding those options to DupEdge. Current exact command-option completeness needs separate verification.

Recovered Matrix menu defaults are in logical input `code/vb6-lite`, container `Matrix90/modMillC_Defaults.bas`, enclosing procedure `Proc_198_34_1A28600` (header line 18508, VA 0x01A28600). Its disassembly binds the normal Curve entries to the Rhino commands DupEdge (26764), DupBorder (26833), ExtractIsocurve (27385) and ExtractWireframe (27661). Solid bindings separately name BooleanUnion (37984) and Boolean2Objects (38191). The exact Ghidra definition is `FUN_01a28600`, line 458258 of logical `code/ghidra/Matrix90.exe.c`; ordinal records 0x154 and 0x161 occur at 461452 and 461829. Ghidra string-copy argument recovery is incomplete, so macro text relies on the native disassembly, not inferred pseudocode arguments. Mapping EXACT/A; command-binding confidence High; geometry-kernel behavior not established by this registration code. No Attribute VB_Name header was found in the selected defaults export; retain its container identity without inventing that attribute.

`clsMInterface.cls` explicitly has Attribute VB_Name `clsMInterface`; signature `Public Function DupBorder(strID)` is VA 0x01567D20 (line 11162). Ghidra definition `FUN_01567d20` at 62938 calls through the same late-bound wrapper observed in native disassembly at 11228–11242. Native code passes the DupBorder command string; surrounding calls use COM offsets 0x48/0x54/0x5c and an error branch whose full semantics remain unresolved. Mapping EXACT/A; delegation High; no conclusion about native Rhino border computation or Undo storage follows.

The recovered .NET `Matrix/Commands/ExtractWireframeCommand.cs`, RunCommand token 0x06000F89, RVA 0x000BDE84, is the separate gvtsExtractWireframe command. It requests T-Splines objects and obtains their display curves. It is not evidence that the ordinary Curve command is restricted to T-Splines or has the same implementation.

Recovered RhinoCommon `Geometry/Brep.cs:1596` exposes GetWireframe(density) through a native call; `DocObjects/ObjectAttributes.cs:774` exposes WireDensity. This supports distinguishing B-rep wire curves and density from merely duplicating topology edges; it does not reveal the native wire solver. Density 1 agrees with the currently implemented boundary/knot/midpoint meaning. The recovered RhinoCommon comment for N>=2 says N+1 interior wires, whereas the installed openNURBS header says N-1; do not extrapolate density-1 evidence to higher densities or silently resolve this version/documentation conflict.

## Comparison with the implemented goal

| Area | Original behavior supported by the selected sources | Current scope and assessment |
|---|---|---|
| DupEdge | Duplicate selected edges of open or closed surfaces to curves; Enter completes. A separate Duplicate Mesh Edge registration exists. | Source-derived edge output and refusal of unverified identity are appropriate implementation choices. Proof, legacy identity recovery and supported-placement restrictions are OM9 limitations, not original command requirements. Generic mesh parity must not be inferred from the unrelated mesh command or a generated command summary. |
| DupBorder | Duplicate open boundaries of surfaces/polysurfaces, hatch or mesh. | Omitting internal seams and refusing a closed solid with no naked border matches the intended boundary meaning. Current surface/B-rep-only acceptance leaves mesh/hatch support missing; it is a partial delivery. Exact joining/polyline-output behavior needs a separate bounded check. |
| ExtractWireframe | Curves corresponding to displayed surface/polysurface isoparametric wires and mesh edges. | Mesh face-edge extraction and standalone untrimmed surface default wires address real contract requirements. In `wireframe.rs:1085`, B-rep/NativeBrep paths still obtain topology edges only; interior face isocurves and non-default density remain missing. Supporting only standalone untrimmed isocurves is a valid first slice, not full parity. |
| ExtractIsocurve | Curves along the source surface's isoparametric directions. | Exact source extraction, placement and trim-aware clipping are consistent with this goal. The specific clipping defaults/options are not established by p154 alone. Planar/cylindrical trim certificates, topology ceilings and point-free conservative refusal remain temporary OM9 scope limits. |
| Boolean | Union/Difference/Intersection on surfaces/polysurfaces; manual warns of conditions causing failure. Boolean Two Objects cycles five results for two objects and accepts on Enter. Mesh booleans are directed to a separate builder. | The five-mode workflow is appropriate. A closed-solid-only path is narrower than the surface/polysurface wording and needs an explicit scope declaration. Temporary certified-box normalization is a kernel workaround, not original business behavior. Rejection of uncertain results protects data but cannot count as supporting a previously promised valid input. |
| Memory and history | These selected manual sections and command wrappers do not establish memory representation or automatic Undo deletion. | Consent-based pruning/retry is an explicit user-approved OM9 feature. It must not be called original Matrix behavior or source-only eviction. History-only display eviction B1–B3 remains a separately deferred architecture change. |

## Consequences for the plan

Keep the separation of CAD correctness and memory optimization. Retain completed deliveries with their exact tested scope. Classify each remaining limit as an original semantic restriction, an OM9 safety/implementation restriction, or unverified coverage; do not merge those categories.

The most direct documented parity gap after the latest work is B-rep/polysurface wireframe isocurves and their relationship to displayed density, not an unlimited expansion of source-only restoration. DupBorder mesh/hatch and unsupported surface/trim routes are other explicit gaps to prioritize by actual user models. Numeric ceilings such as 707 or 1024 are implementation budgets; the selected source evidence does not make them Matrix requirements. Full-model coverage and native peak measurements should remain separately scoped rather than blocking every completed command slice.

The plan's current queue and feature documentation still contain older boundary-only surface and proof/placement restrictions. Reconcile those entries with the newer evidence before treating them as future work. This review does not modify command semantics, approve new implementations, close Plan A/B, or certify every manual option.

## Input fingerprints

SHA-256, logical paths resolved through the local registry; originals preserved:

- `code/vb6-lite/Matrix90/modMillC_Defaults.bas`: 101a5a8edb1caf9bca44db4cb6677ab269e556e91117f38d95f3f99fe83b55bd
- `code/vb6-lite/Matrix90/clsMInterface.cls`: 8e027f3d5c125170cb1b1516651723df923792697a6fa9f01295b56182e90e86
- `code/ghidra/Matrix90.exe.c`: dcbee0f15ca62c18c19273403165af3ccdbb7064acc5988090148f46f0c1e2cb
- `code/decompiled-ui/Matrix/Commands/ExtractWireframeCommand.cs`: 1cad6924f6db2de04aac6c0ed1b075c8a1a802934ece61a9f2fcff703f74af42
- `rhino5/dnspy/RhinoCommon/RhinoCommon/Geometry/Brep.cs`: f7557d2b44d8d9ab096384d134b18ecd28c4c588be1eb7a4527f616f2a8d22f3
- `rhino5/dnspy/RhinoCommon/RhinoCommon/DocObjects/ObjectAttributes.cs`: cdd18f1c47726c8778ab4650eb47b80a1e3f259e5bf321357a8720dee8764062
