# Gumball final review — 2026-10-11

Exactly one fresh-context reviewer (`gpt-6-astra`) reviewed the scoped range
`e4b7b979c69baf57fab6feccdec1a1397b259c31..d04e254741927ff55e73659b5097ef60e215a79d`.
The reviewer made no edits, builds or native test runs. The author re-graded
all three findings as Important by their effect on users. No Critical or
Minor findings were reported. Verdict: **With fixes**.

1. **P1: Mesh/Points synchronous same-field replacement could commit wrong
   geometry.** `Gui/GumballGeometry.cpp` allowed active-writer notifications
   and remembered actual output without comparing to staged discrete values.
   Required proof: one-shot real document observers during Scale, original
   geometry and unchanged Undo count after rejection.
2. **P2: constrained axis motion discarded drag direction.**
   `rust/src/gumball.rs` used positive entered distance directly after every
   motion. Required proof: pointer crosses the press point after Enter,
   positive/negative signed input and positive/negative extrusion.
3. **P2: unsupported BRep components silently used owner axes.**
   `Gui/GumballGeometry.cpp` returned Placement directions for `Vertex1`.
   Required proof: Object alignment reports `UnsupportedFrame` and preserves
   an existing rotated CPlane frame without model or Undo mutation.

Additional assessments: Python `Document.UndoMode` is always enabled on this
SDK (getter returns 1, setter is empty); disabled-Undo was ruled out rather
than imposing an unnecessary capability gate. The documented loss of an old
Redo queue on callback-triggered abort is a real, nonblocking limitation for
this partial slice; model/Undo rollback does not prove history-preserving
failure atomicity. Preview cancellation and the scoped build helpers yielded
no further actionable findings.

Declined to judge, with author rulings:

- Smart Targets, Clayoo/SubD and independent Move/Rotate/Scale commands are
  outside the user's selected three-spec scope. Cost: separate workflows
  remain unavailable under this deliverable.
- Parametric/Link Scale, external-document Links, Mesh-component/light/camera
  frames, persistent pivot cache, new associative History and exhaustive
  snap/IME/source-product parity remain explicitly deferred, with visible
  rejection and partial-support status. Cost: these capability gaps remain.
- Unrelated dirty code and other-chat runner changes are not attributed to
  Gumball. Integration tests cover the selected host; unrelated ownership
  requires separate review. Cost: that code has no acceptance claim here.

Single author fix pass: RED proved all three classes on the reviewed plugin
in `build/shared-host-tests/gumball-review-red-20261011/results.json`.
Two independent Rust regression tests also failed before implementation.
GREEN: `gumball-review-green-20261011` passed 9/9 native checks, actual exit0.
The stronger mixed Mesh/Points batch checks also pass. Focused Rust passed
36/36 tests across 3 suites, including the two RED direction regressions.
Final combined `gumball-post-review-final-20261011` passed 432/432 checks across
16 native processes, all actual exits0, with unchanged selected plugin SHA256
`3107FFC7FF0087DED9902C066C4B026F67090291EAAB0F1C590772C19FB5CD29`. Full Rust passed 808 tests across 115 suites.
The native suite covers existing ownership/lifecycle/DPI, warm/cold persistence
and Curve/CoreMouse/layers/History/3DM regressions. All Important findings
are addressed in the single author fix pass; no second review was dispatched.
