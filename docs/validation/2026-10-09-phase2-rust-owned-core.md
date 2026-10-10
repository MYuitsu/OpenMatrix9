# Phase2 Rust-owned core — 2026-10-09

Status: **accepted_scoped** after all six implementation tasks, matching application gates and one independent whole-branch review. Full openNURBS/Phase3–5 remain open.

Portable curve basis/validation/graph, owned CV/Rebuild sessions and options, snap rows/tiles/candidate cache/budgets/ranking now live in safe Rust modules with compiler-enforced `forbid(unsafe_code)`. Small C ABI modules copy borrowed numeric input and return caller-owned buffers. Native adapters retain FreeCAD/Qt/OCCT/openNURBS/Coin inspection, projection, construction, preview and GUI transactions. Native dependencies and the FFI boundary retain their safety obligations.

PointsOn now uses native Qt widgets backed by Rust; production `ModelingCurveEditor.py` and its packaging/import path are removed. Repeated PointsOn closes the previous editor and drops its session immediately. Curve publication, whole-wire sampling and Rebuild no longer depend on Python Part wrappers. Required workbench hooks, API entry points and application test/tools remain Python.

Verification on runtime `H:/FreeCAD-src/build/om9-phase2-rust-sdk`: 112 Rust tests, 58 Python tool tests, 50 native suites, 19 original host batches plus three ownership macros, two review-fix macros and HiDPI 2. Actual Rhino 5 checks three rational/periodic/placed fixtures (125 checks), then FreeCAD rereads their real SaveAs outputs (55 checks). The combined host evidence contains 30 reports/507 checks and verifies source/module/executable/macro/fixture hashes. Historical shorthand “57 reread checks” was inaccurate; report-array counts are used here.

- [Machine summary](modeling-phase-2-rust/summary.json)
- [Requirements](modeling-phase-2-rust/requirements.json)
- [Actual application evidence](modeling-phase-2-rust/application-evidence.json)
- [Ownership map](modeling-phase-2-rust/ownership-map.json)
- [Packaging guard](modeling-phase-2-rust/packaging.json)

The worker default remains 60% of logical CPUs, bounded by tasks/RAM. Warm snap probes use 500 queries on 10,000 remote CAD objects and million-point Mesh/cloud fixtures. Their raw measurements and hardware are in the machine summary; these do not establish cold rebuild latency. Literal-zero heavy-access counters remain uninstrumented, supplemented by source exclusions and scoped heavy probes.

Additional clipboard diagnosis uses20 repeated rollback probes/40 checks with direct Windows CF_UNICODETEXT readback. Native text restores correctly; cached Qt text can append terminator NULs after OleFlushClipboard. The production clipboard bridge is byte-identical to the accepted baseline. Failed/shared-clipboard attempts remain archived and are excluded from positive evidence.

Clippy exits 0 with warnings; repository-wide rustfmt still reports inherited formatting debt. New Rust modules/tests are formatted. Final review findings and their resolution/deferred minors are recorded in [final review](modeling-phase-2-rust/final-review.md) and [resolution](modeling-phase-2-rust/review-resolution.json).

The accepted pre-migration source/runtime snapshot is retained at `H:/FreeCAD-src/build/phase2-rust-source-backup-20261009`; historical `om9-phase2-sdk` remains intact. Migration source is isolated in `build/om9-dev`; no commit, push or primary product integration is included. The broken Git pointer is preserved, with immutable snapshots/hashes and the plan ledger providing recovery.

For interactive use after acceptance, run `H:/FreeCAD-src/build/launch-om9-phase2-rust.cmd`, then choose OpenMatrix9 workbench. It uses a separate persistent profile.

Full openNURBS and Phase3–5 remain open, including seven implementation packages. Prepared migration gates remaining: zero. Total future test-batch count is unknown.

## Decisions and deferred minors

Ruling: Use immutable snapshots and this ledger instead of Git task helpers/commits — existing om9-dev Git pointer is broken and user scope retains isolated source — cost if wrong: no commit-based recovery; snapshots/hash reports provide recovery.
Task2: Ruling: Add caller-owned session-copy ABI alongside original plan exports — UI/kernel must consume the actual Rust-owned draft rather than validate and reread parallel native state — cost if wrong: ABI layout/capacity errors; explicit contract test and matching host header cover it.
Task2: Ruling: Retain temporary typed Python compatibility wrappers solely for existing Rebuild callers until Task5 — allows independently testable native factory transition without half-compiling host — cost if wrong: migration could silently retain Python; Task5 removal and ownership gate explicitly require deleting wrappers.
Task5: Ruling: Add owned Rebuild-option copy and early input-count/Join-token Rust ABI — kernel/native transaction must consume Rust decision, and budgets/duplicate rejection precede topology conversion — cost if wrong: capacity/layout or source-order mismatch; Rust and native ownership tests cover it.
Final: minor (deferred): restore visible object-local CV coordinate explanation in native dialog; coordinates/placement remain correct, headings are ambiguous.
Final: Ruling: Arbitrary pointer validity/buffer overlap/allocation failure and complete native dependency memory safety remain caller/native obligations — approved FFI contract and safe-core compiler guards do not certify these — cost if wrong: caller UB or allocation/native failure can still abort.
Final: Ruling: Native exception coverage uses staged Rust failure tests plus audited native abort guards, not exhaustive live OCCT/Coin fault injection — preserves approved scoped proof — cost if wrong: an unusual untested native failure path may remain.
Final: Ruling: Heavy-array exclusion remains source inspection and scoped performance proof with uninstrumented zero counters — no independent instrumentation was added — cost if wrong: unexpected heavy access may escape this evidence.
Final: Ruling: Performance acceptance covers500 warm queries on the recorded24-thread Windows machine — no cold-cache or other-hardware guarantee — cost if wrong: cold/other-machine responsiveness may differ.
Final: Ruling: Full openNURBS/Phase3–5 and seven packages remain open — this is current Phase2 architectural migration — cost if wrong: later geometry/workflows still need implementation.
Final: Ruling: Platform/SDK acceptance covers the recorded Windows FreeCAD/Rhino5 runtime; reviewer independently inspects evidence while root replays actual applications — no cross-platform or reviewer-run replay claim — cost if wrong: portability/other-SDK defects remain untested.
Final: Ruling: Preserve Git/primary product and inherited formatting/Clippy debt under approved isolated-workspace boundary — no repair/integration/push or unrelated style sweep — cost if wrong: snapshot-based recovery and warning/format debt persist.
Final: Ruling: Verify controlled text rollback directly through Windows CF_UNICODETEXT instead of exact cached Qt text readback — probe showed correct native UTF-16 data while Qt readback appended two terminator NULs; testing-native protocol is the intended clipboard contract — cost if wrong: Qt-specific readback/cache quirks remain outside that proof. Experimental bridge changes and diagnostic Qt-class override were reverted; ThreeDmClipboard.py byte-identical to accepted baseline0750ddeb846a268d890c5282fbbae9fcd7fbcc16c472d956a7af9d39d17fc086. Native20 rollback probes40 checks GREEN; normal clipboard/failure19 checks GREEN. Failed attempts retained and excluded from positive evidence.
