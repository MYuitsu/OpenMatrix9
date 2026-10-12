# OM9 main integration and native rebuild — 2026-10-12

The user requested commit/push to main and rebuilding all OM9 components, and selected **all current OM9 changes, with FreeCAD core kept separate**. Snapshot `041e62c` preserves the complete development tree; integration retains main input `c2475df` and its existing Gumball, canonical Layers, exchange, Phase2/3 and launcher work.

Command Options retain Rust schemas/reducers, per-step prompts, single-character mnemonics, clickable names, toggles, sub-options, numeric entry, CMD history/focus and shared CMD–Builder proposals. Default Box remains two opposite base corners then Height; the four methods stay optional and explicit 3Point remains two edge endpoints, Width and Height.

Integration restores native Rust Rebuild session/witness ownership and native sampling/deviation while keeping new admission/result validation and CMD bindings. Canonical transactions finish before Surface/Edit commit, with owned rollback and Redo preservation. 3DM keeps canonical receive target/transaction checks, immutable owned preflight, original typed admission errors and Python binding causes. Prepared preview dictionaries are distinguished from canonical layer packets. The native regression found and fixed a generic Shape/Python `discretize` call in Rebuild preview.

## Verification

- Complete Rust: **1218 passed, 0 failed**, 168 result suites, exit0.
- Selected native cohort: **20 processes / 1066 passing checks**, one plugin SHA and all host exits0. Includes Box/Sphere/CMD–Builder, Rebuild ownership, canonical layer guards/receive, prepared CAD diagnostics, Surface/Edit publication and Gumball multi-selection.
- Standalone `OpenMatrix9Gui` build exit0, including Rust release core, openNURBS, 3DM import worker and packaged scripts/resources. Shared FreeCAD root was not reconfigured or rebuilt.
- Public source audit and diff whitespace check against current main pass. Spec manifest byte/hash entries match their files.
- Independent scoped integration review found the ownership/diagnostic gaps above; corrected source and native02 regressions provide acceptance evidence.

Plugin SHA256: `247afa60deee6b11fc2a7e12209a1540716e84936e24ecb29ca8a284585ad328`.

Runtime: `C:/Users/nguye/AppData/Local/Temp/om9-main-native-runtime-20261012`. Raw receipts and build/Rust logs: `C:/Users/nguye/AppData/Local/Temp/om9-main-integration-20261012`. [Machine-readable evidence](2026-10-12-main-command-integration.json) records source and report hashes.

## Host and scope

Reuse `H:/FreeCAD-src/build/relWithDebInfo` and the checkout's plugin cache. Matching isolated App299 import/DLL and camera-close Gui override are hash-bound by the native launcher; this is a native development SDK proof. FreeCAD core source is not included in these OM9 commits or pushed. This source depends on the matching custom App APIs; stock FreeCAD/CI builds have not been verified by this task. Existing release receipts retain their original revision/host scope.

The full catalog, remaining RCORE-01 work and full original-host parity remain partial. Only the listed cohort is newly certified. Binary fixture inputs and user models remain outside source. The isolated synthetic layer fixture location can be selected with `OM9_LAYER_FIXTURES`.
