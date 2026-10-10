# Single independent whole-branch review

Reviewer: fresh-context gpt-6-astra, `/root/phase2_rust_final_review`; read-only, no subagents or independent application replay. Base: immutable accepted source snapshot. Reviewed pre-fix module491d90856c0fd88d840829462e4457cae8e2faebb4eb6f737d1f9293721d6f55 /manifest43f351a48f542ec0af28ae5841b0f34ed9e65dec2086d1021de86e68fbdeffe1.

Verdict: **not ready for acceptance before fixes**. Two Important findings, no Critical findings, one Minor. Root resolves findings with the authorized single RED→GREEN fix pass and fresh complete matching verification; there is no second review.

## Strengths

Portable basis/session/snap state genuinely Rust-owned; safe modules forbid unsafe and FFI copies input using monotonic handles/caller-owned output. Native snap reconstruction has aborting build guard; Rust rejects failed staged builds/stale publication and incomplete clears picked. Cache keys include object/mode/residual budget; deterministic ties/depth/logical-pixel projection retained. Native transactions follow validation/kernel construction, and Rebuild checks dependencies/witnesses. Reviewer independently verified every manifest source/runtime hash, all27 report hashes/456 passing entries and suite-log hashes; actual Rhino125 and saved reread55 passing entries.

## Important

1. `Gui/ModelingCurveEditorDialog.cpp:64`, commit at32: PointsOn context cancellation only polled every100ms, with synchronous connections only for Undo/Redo. Workbench switch followed by immediate OK can commit; document switch-away-and-back can retain old session. No dangling-pointer defect claimed. Fix native synchronous document/workbench notifications, active-context guard before commit, immediate no-settle draft→context→commit tests and document-close/replacement coverage.
2. `Gui/ModelingCurveEditorDialog.cpp:56`, request at34: approved First/Last active-domain controls missing. Knot interval remapping[0,1]→[2,3] fails because hidden active domain remains[0,1]. Old Python omission does not satisfy approved design. Fix numeric First/Last initialized from owned Rust draft and submitted through Rust; test valid domain edit, invalid rejection, Undo.

## Minor

P3: same dialog at56 removes the old editor's explanation that CV coordinates are in the object-local frame. X/Y/Z alone ambiguous for placed objects; geometry remains correct. Restore visible guidance as later polish.

## Five review-focus classes

Malformed FFI scalar/dimension/null/alignment/count/product/capacity/handle checks inspected without additional blockers. Copied stale witnesses and dropped handles protect inspected deletion/Undo/geometry paths, but context cancellation needs finding1. Native RAII abort plus Rust build/generation checks prevent partial clean publication. Rational/periodic data and single native world transform preserved, supported by matching Rhino/Link evidence; domain edit needs finding2. Mode/residual budget keys, cumulative limits, exhaustive-mode completion, tie/depth/HiDPI/incomplete/manual fallback inspected without additional blockers.

## Declined to judge (verbatim scope)

- Arbitrary-address validity, caller buffer overlap, allocator failure, and complete memory safety of FreeCAD/Qt/OCCT/openNURBS: explicitly retained native/FFI obligations; checked lengths/alignment cannot establish those properties.
- Exhaustive native fault injection: reviewed Rust failure tests and native abort control flow; this read-only review did not inject live OCCT/Coin failures.
- Instrumented absence of Mesh/cloud heavy-array reads: counters are literal zeros. Source exclusions and bounded performance evidence support only their documented scope.
- Cold-index latency and performance on other hardware: recorded measurements concern500 warm queries on the stated machine.
- Full openNURBS coverage, Phase3–5, and seven future packages: explicitly outside this migration.
- Other platforms/SDK combinations and independent fresh application replay: reviewed the recorded Windows runtime evidence; the review instructions prohibited running applications/tests.
- Git repair, integration, commit/push, primary-product replacement, and inherited formatting/Clippy cleanup: excluded by the ledger and review boundary.

The two Important findings require correction and complete matching fresh verification before acceptance. Post-fix outcome is recorded in review-resolution.json and the plan ledger.
