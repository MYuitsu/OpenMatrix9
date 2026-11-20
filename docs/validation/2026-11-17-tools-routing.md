# Tools icon routing snapshot — 2026-11-17

The headless controller fixture resolves all 31 icons for each installed package and again after disabling it. This is routing evidence, not proof of the command behavior, desktop interaction, native DLL, or milestone completion. The package is built and installed by the host fixture; its commands run in-process over the add-on JSON boundary. The standalone Tools package does not contain Boolean Builder. The 124 observations are in [the JSON snapshot](2026-11-17-tools-route-snapshot.json).

Join History remains visible and disabled, including the F6 availability decision; typed aliases and direct add-on starts refuse without changing document or Undo. Profile has a disabled fallback because the shared feature ID implements Profile Placer. Its add-on route is unaffected. The owner and outstanding semantics for each icon remain in [the gap plan](../plans/2026-11-16-tools-menu-gap-plan.md).

| Icon | Matrix installed | Tools installed | Package disabled fallback |
| --- | --- | --- | --- |
| `BuilderRingRail` | `BuilderRingRail` (enabled) | `BuilderRingRail` (enabled) | `OM9-TOOLS-001` (disabled) |
| `BuilderProfilePlacer` | `BuilderProfilePlacer` (enabled) | `BuilderProfilePlacer` (enabled) | `OM9-TOOLS-002` (enabled) |
| `BuilderOutsideRingRail` | `BuilderOutsideRingRail` (enabled) | `BuilderOutsideRingRail` (enabled) | `OM9-TOOLS-003` (enabled) |
| `BuilderCustomRail` | `BuilderCustomRail` (enabled) | `BuilderCustomRail` (enabled) | `OM9-TOOLS-004` (enabled) |
| `BuilderFourProfile` | `BuilderFourProfile` (enabled) | `BuilderFourProfile` (enabled) | `OM9-TOOLS-032` (enabled) |
| `BuilderMetalWeights` | `BuilderMetalWeights` (enabled) | `BuilderMetalWeights` (enabled) | `OM9-TOOLS-006` (enabled) |
| `BuilderResize` | `BuilderResize` (enabled) | `BuilderResize` (enabled) | `OM9-TOOLS-007` (disabled) |
| `BuilderSurfacePullback` | `BuilderSurfacePullback` (enabled) | `BuilderSurfacePullback` (enabled) | `OM9-TOOLS-008` (disabled) |
| `BuilderBermarkLibrary` | `BuilderBermarkLibrary` (enabled) | `BuilderBermarkLibrary` (enabled) | `OM9-TOOLS-009` (disabled) |
| `BuilderStullerFindingsLibrary` | `BuilderStullerFindingsLibrary` (enabled) | `BuilderStullerFindingsLibrary` (enabled) | `OM9-TOOLS-010` (disabled) |
| `gvSmartTarget` | `gvSmartTarget` (enabled) | `gvSmartTarget` (enabled) | `OM9-TOOLS-011` (disabled) |
| `gvSmartTargetAdd` | `gvSmartTargetAdd` (enabled) | `gvSmartTargetAdd` (enabled) | `OM9-TOOLS-013` (disabled) |
| `gvSmartTargetOnCrvEnd` | `gvSmartTargetOnCrvEnd` (enabled) | `gvSmartTargetOnCrvEnd` (enabled) | `OM9-TOOLS-011` (disabled) |
| `gvSmartBlend` | `gvSmartBlend` (enabled) | `gvSmartBlend` (enabled) | `OM9-TOOLS-012` (disabled) |
| `gvSmartMSR` | `gvSmartMSR` (enabled) | `gvSmartMSR` (enabled) | `OM9-TOOLS-015` (disabled) |
| `gvJoinHistory` | `gvJoinHistory` (disabled) | `gvJoinHistory` (disabled) | `OM9-TOOLS-017` (disabled) |
| `BuilderImageTrace` | `BuilderImageTrace` (enabled) | `BuilderImageTrace` (enabled) | `OM9-TOOLS-016` (disabled) |
| `BuilderObjectChecker` | `BuilderObjectChecker` (enabled) | `BuilderObjectChecker` (enabled) | `OM9-TOOLS-018` (disabled) |
| `BuilderMeshRepair` | `BuilderMeshRepair` (enabled) | `BuilderMeshRepair` (enabled) | `OM9-TOOLS-020` (disabled) |
| `Builder3dPrinting` | `Builder3dPrinting` (enabled) | `Builder3dPrinting` (enabled) | `OM9-TOOLS-019` (disabled) |
| `BuilderProfile` | `BuilderProfile` (enabled) | `BuilderProfile` (enabled) | `OM9-TOOLS-002` (disabled) |
| `BuilderProfileMerge` | `BuilderProfileMerge` (enabled) | `BuilderProfileMerge` (enabled) | `OM9-TOOLS-034` (enabled) |
| `BuilderProfileEndCap` | `BuilderProfileEndCap` (enabled) | `BuilderProfileEndCap` (enabled) | `OM9-TOOLS-033` (enabled) |
| `BuilderCustomRailAdvanced` | `BuilderCustomRailAdvanced` (enabled) | `BuilderCustomRailAdvanced` (enabled) | `OM9-TOOLS-035` (enabled) |
| `BuilderObjectOnCrv` | `BuilderObjectOnCrv` (enabled) | `BuilderObjectOnCrv` (enabled) | none |
| `BuilderSurfaceInset` | `BuilderSurfaceInset` (enabled) | `BuilderSurfaceInset` (enabled) | `OM9-TOOLS-038` (disabled) |
| `BuilderMeshReducer` | `BuilderMeshReducer` (enabled) | `BuilderMeshReducer` (enabled) | `OM9-TOOLS-037` (disabled) |
| `BuilderMesh` | `BuilderMesh` (enabled) | `BuilderMesh` (enabled) | `OM9-TOOLS-020` (disabled) |
| `BuilderCurveTransformTool` | `BuilderCurveTransformTool` (enabled) | `BuilderCurveTransformTool` (enabled) | `OM9-TOOLS-040` (disabled) |
| `BuilderCenterLine` | `BuilderCenterLine` (enabled) | `BuilderCenterLine` (enabled) | `OM9-TOOLS-042` (disabled) |
| `BuilderBooleanBuilder` | `BuilderBooleanBuilder` (enabled) | `OM9-TOOLS-041` (disabled) | `OM9-TOOLS-041` (disabled) |

Default addon_session: 18 passed, 2 ignored; no-default addon_session: 16 passed, 1 ignored. Builds use `L:/om9-target` and `-j 2`. The snapshot establishes icon target decisions only; remaining aliases, exact command semantics, disabled-package fallback equivalence, actual OS input, and owners marked unresolved in the plan require further work.
