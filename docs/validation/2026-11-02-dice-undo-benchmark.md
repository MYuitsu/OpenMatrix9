# Dice benchmark with and without Undo

2026-11-02, Windows x64, `L:/om9-target/debug/om9.exe`, default unoptimized dev profile. Identical MCP fixture and timing boundaries: startup through successful `.om9` save, one 20 mm box, then 21 spherical cuts with `Boolean2Objects`, Difference, DeleteInput=Yes. The normal reference run is documented in `2026-11-02-dice-21-cavities-timing.md`. Each variant was run once, sequentially; separate executables were rebuilt after adding the benchmark switch. This is not a controlled repeated or release-build study.

| Face in execution order | Undo enabled (s) | Benchmark Undo disabled (s) |
|---|---:|---:|
| 1 | 0.688 | 0.581 |
| 6 | 55.226 | 14.050 |
| 2 | 54.106 | 8.570 |
| 5 | 273.433 | 29.939 |
| 3 | 274.347 | 23.532 |
| 4 | 551.804 | 40.057 |
| Save | 2.497 | 2.594 |
| **Start through save** | **1212.506** | **120.182** |

The second run took 1092.324 s less (90.09% less elapsed time, 10.09× speedup). The benchmark opt-in sets `benchmark_no_undo: true` only at `om9_new`, before editing; `undo=0` and the mode flag were checked after every one of 21 cuts and in the save receipt. Default sessions still record Undo. The benchmark changes the transaction retention/validation path as well as storing no Undo; this is not an isolated CPU profile and does not assign the entire difference to one function.

Both runs saved one BRep and have identical native BRep source SHA-256 `22ce14cab9bf89e8e5b5a45b806ce1196f8d44b2b16bec5645a0ba4f3783a23f`, 31 displayed faces and identical display-mesh volume 7852.34279186927 mm³; the ideal volume is approximately 7851.559747 mm³. The display mesh has approximate bounds up to 20.0803 mm on x and 20.0703 mm on y, so these are tessellation measurements, not exact BRep tolerance certification. The benchmark file was reopened by the four-view offscreen renderer. Neither topology manifoldness nor STL printing was independently checked.

Focused checks: `om9-core` transaction_scopes and atomic_transactions: 25 passed; `om9-app` mcp_stdio: 3 passed, including default recording and benchmark Box/Sphere/Boolean; solid_workflows immediate Boolean: 1 passed (113 filtered); public source audit: 0 errors. Raw timing receipt: `2026-11-02-dice-no-undo-timing.jsonl`. Fixture script: `tools/dice_21_no_undo_benchmark.py`; it refuses to overwrite its output. Result `.om9` remains under `build/dice-21-no-undo-2026-11-02`.

Next: profile retained owner validation and scene update separately, then optimize with Undo enabled while preserving exact validation, stale guards, rollback, budgets and normal Undo/Redo. This opt-in mode is for experiments, not an interactive product default.
