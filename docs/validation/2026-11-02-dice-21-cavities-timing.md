# 21-cavity dice: end-to-end MCP timing

2026-11-02, Windows, `L:\om9-target\debug\om9.exe`, default unoptimized dev build with the fingerprint buffering change described in `2026-11-02-dice-fingerprint-performance.md`. The clock starts before MCP process startup; it stops immediately after `om9_save` returns. Fixture: 20 mm box, 21 radius-1.5 mm spherical cutters centered on its faces, Boolean2Objects Difference and DeleteInput=Yes after each cutter. Opposite face counts sum to seven. Script validates one document object after every operation.

| Face | Cavities | Seconds |
|---|---:|---:|
| 1 | 1 | 0.688 |
| 6 | 6 | 55.226 |
| 2 | 2 | 54.106 |
| 5 | 5 | 273.433 |
| 3 | 3 | 274.347 |
| 4 | 4 | 551.804 |

All 21 operations returned success; the saved state reports one B-Rep object. The final cavity took 159.848 s. `om9_save` took 2.497 s; start to save returned was **1212.506 s (20 min 12.506 s)**. Startup and fixture setup are included; the later JSON report and offscreen render are excluded. See `2026-11-02-dice-21-cavities-timing.jsonl` for the complete receipt. The saved `build/dice-21-timed-2026-11-02/dice.om9` reopened for a four-view offscreen render; the visible faces show recessed pips. The output project and image are build artifacts, not the sole record of the measurement.

This is one debug/headless/MCP run, not a release or physical desktop benchmark; face durations depend on execution order and retained history. The run proves successful command completion and one saved body, not independent topology, volume or 3D-print readiness. No STL was requested or tested.
