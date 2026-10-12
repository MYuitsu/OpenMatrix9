# RCORE-01 native Surface capability integration — 2026-10-11

**Scope:** Shared Rust Surface operation/type/options policy consumed by native declaration, capture and construction for existing Sweep1/Sweep2/Loft solvers; partial RCORE-01 integration. The chapter and construction plan remain in progress.

Safe Rust owns the current request decision. The native adapter supplies initialized
scalar options and a bounded borrowed slash array through the 72-byte ABI; Rust
owns its private copy and retains no host pointer. Controller declaration can
prompt without input; concrete capture checks the selected representation before
copy. Construction checks real options and current native rail closedness before
profile fitting or output construction. Existing source guards and native validity
checks remain necessary. C++ is required for FreeCAD/Qt/OCCT/Python access only.

Binding RED observed the absent method: one failed assertion, no fixture exception,
actual host exit0 on the previous plugin. This is binding evidence, not a reproduced
full command defect. Rust policy/ABI functional RED-GREEN is retained in the
[source receipt](2026-10-10-rhino-core-native-surface-policy-source.json); unchanged
source fingerprints retain the full 691-test Rust run with actual exit0.

Matching native GREEN: binding29 + input12 + Surface97/options180/advanced108/
constraints85/History65 + Curve/Rebuild41 + native Edit capability20 = **637 checks,
nine actual host exits0**. All runs used plugin SHA256 `a74e38f1cb9ee3845b3ec7f4722bd500932cb4604fa7373d318c05f9c95830bb` and the shared
FreeCAD SDK/host. Standalone incremental build confirmation returned actual exit0;
no FreeCAD root rebuild was performed. Sweep/Loft, supported options, constraints,
History, source/Undo/Redo and valid selected Edge2 behavior remained green.

The diagnostic query tests the native binding. The real command and History
fixtures provide the separate solver-path regression evidence. Neither establishes
complete Rhino parity, all enabled construction families or full result contracts.

Remaining work:
- Shared construction policy and bounded native references for Curve, Solid, Rebuild and cage consumers remain open.
- Surface seam/reverse/chain metadata, computed Link getters, selected-topology allocation and constraint-support admission still need full coverage.
- Native GeometryResult/conversion/deviation/parameter/topology maps and original chapter ownership, cache, worker and preservation obligations remain open.
- Full construction-plan regression and its final source review remain pending; these nine runs do not replace the broader native Edit/prepared-result coverage.
