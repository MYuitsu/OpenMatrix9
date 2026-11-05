# Multi-cutter Difference and MCP dice with Undo

2026-11-02, Windows x64, local OCCT 8.0.1, unoptimized debug app SHA-256 `93320282d380e2d4a603591957a34e29d683cd3fb66892effa048876d69b6ada`. Both fixtures run with normal Undo, using `tools/dice_multicut_benchmark.py` and the same binary. Clock: before MCP process creation through successful `.om9` save; report, mesh-volume evaluation, Undo and Redo occur after that clock.

| Workflow | Startup to save | Boolean commits | Undo depth after save |
| --- | ---: | ---: | ---: |
| 21 sequential Boolean2Objects cuts | 24.780 s | 21 | 43 |
| BooleanDifference: one target, 21 cutters | 8.874 s | 1, 3.122 s for the final command | 23 |

The runs are single, sequential debug samples; batch is approximately 2.79x faster for this fixture (64.2% less total elapsed time). They deliberately have different transaction granularity, not interchangeable Undo histories. The 21 spheres and box are individual creation transactions in both cases. Fixtures and timing boundaries are identical otherwise. Timing receipts: `2026-11-02-dice-multicut-direct.jsonl` and `2026-11-02-dice-multicut-direct-serial.jsonl`.

The final native BRep source SHA-256 in both cases was `22ce14cab9bf89e8e5b5a45b806ce1196f8d44b2b16bec5645a0ba4f3783a23f`; both display snapshots contain 31 faces. Approximate display mesh volumes were 7852.342631 and 7852.342792 mm³, respectively; the ideal exact volume is 7851.559747 mm³. A separate kernel test for exactly these 21 cutters requires the direct multi-tool branch, one closed solid and native measured volume within 0.0001 mm³ of the analytic result. This is not independent STL/manifold or physical desktop certification.

For batch, MCP Undo once restored the original target and all 21 original cutter object IDs (22 objects; undo depth 22); Redo restored the same result ID and BRep digest (one object; undo depth 23). For serial, one Undo restored the preceding body and last cutter (two objects; undo depth 42); Redo restored the same result ID and digest. Saved batch project: `build/dice-multicut-direct-2026-11-02/dice.om9` (build artifact; the validation record and raw timings are retained outside build).

Implementation: Difference first builds the existing target union, then screens at most 128 cutters with OCCT tolerance-aware optimal bounds. When every pair of bounding boxes is disjoint, the OCCT cut accepts all tools at once and emits a single snapshot. Overlapping, touching, unbounded, or more than 128 cutters fall back to the existing cutter-union path. The existing DeleteInput, source-stamp guards and atomic transaction/Undo remain. Inputs are validated and retained budgets accounted in Rust, OCCT shape validity/solid checks and output validation are retained; this is a narrow OCCT adapter exception, not a native document owner. Intersection keeps its prior path.

An initial attempt used ordinary loose BRep bounds; the dice-specific regression failed because these boxes overlapped despite disjoint spheres. Timings from that fallback are not evidence of a direct multi-cut. Using tolerance-aware optimal bounds made that test pass and the MCP timing above exercises the same arrangement. No commercial or decompiled code was copied.

Validation: `cargo test -j 2 -p om9-kernel --features 3dm-brep --test l2_01_10` 72 passed; focused app `difference_multiple_tools_commits_once_and_undo_restores_inputs` passed for disjoint and overlapping cutters; `mcp_stdio` 3 passed; normal debug app build passed; public source audit zero errors. No release benchmark, repeated p50/p95, general model performance comparison, human UI validation or independent print readiness assessment is claimed.
