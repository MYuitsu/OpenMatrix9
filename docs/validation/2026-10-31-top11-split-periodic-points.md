# Top11 Split Point: certified periodic NURBS — 2026-10-31

Status: partial automated evidence; full Top11 and physical M03-UI remain open.

Rust certifies repeated-pole/weight and shifted-knot periodic bases and materializes the active interval with both ends clamped before exact knot-insertion splitting; the resulting segments are nonperiodic without refitting. Split Point accepts this certified representation. Invalid periodic bases are rejected, and records carrying a seam remain explicitly unsupported by the shared split path: the current fraction evaluator does not apply the stored seam, so silently splitting such a curve would be incorrect.

The core fixture checks both parameter orientations and full or trimmed domains at 101 samples per piece, and checks the seam-spanning closed result by comparing all three constituent parameter intervals. Invalid bases and reseamed records are rejected. App tests exercise a rational full closed curve with two points and a trimmed open curve with one, each in both orientations through CMD and toolbar, with translation placement, total length, archive roundtrip and Undo/Redo. They do not establish general periodic representations, all knot multiplicities or exact visual input.

Windows x64, CARGO_TARGET_DIR=L:\om9-target and -j 2: 75 selected core integration tests, 191 app Top11/options/BRep/snapshot tests and 109 no-default Top11/options tests passed; the final added invalid-input assertions passed in a focused rerun. No-default reports existing unrelated warnings. No native or openNURBS measurement was performed. Physical desktop M03-UI and 3DM interoperability remain unverified.

Source SHA-256:

- `crates/om9-core/src/curve_ops.rs`: `d89e624c67c7b4e382cd5d70be18f3a7f3ca86aa26329fdd24e92a8395e73f2e`
- `crates/om9-core/tests/top11.rs`: `bc9a36b7704d803d223df6a9cab9448d1b61fca2a6ab24e88d182305e332588c`
- `crates/om9-app/src/session/top11.rs`: `0fd6451f75d6ec1821cd079de715d305442fee530007500ef644c15f902d7b5c`
- `crates/om9-app/tests/top11.rs`: `b06c25ae9931a50addc965a038adb139d53cfd4a7a793ac81e0a0f2c113aa9c9`
