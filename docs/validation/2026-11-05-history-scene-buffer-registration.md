# Undo/Redo persistent scene buffer registration — 2026-11-05

Undo and Redo previously held the scene construction allowance through publication but released all of it afterward. The history scene callback now admits buffer-owner overhead and splits the existing allowance into a persistent buffer owner plus the remaining scene/staged-metadata reservation before changing history. It returns the same ordered reserved-scene bundle as ordinary full-scene edits. The Controller publishes the buffer owner with the historical scene, then releases the remaining transient allowance. A failure before acceptance leaves the previous scene and both history stacks intact.

## Verification

Default features, OCCT 8.0.1 runtime, E: debug target, `-j 2`, serial:

- A new viewport-owner fixture holds an old snapshot while Undo and Redo build different full scenes. Without asking for a new viewport snapshot, it verifies the account contains the old worker charge plus exactly the current scene buffers and lease overhead. Dropping the old snapshot removes only its charge. All seven presentation owner tests pass.
- Six scene pressure tests and six New/Open reservation tests pass. The first runs exposed former zero/one-byte expectations after Undo: the tests now compare failure and retry with their previously captured live-buffer baseline. They still require refusal to leave the document, scene and Redo unchanged. Successful Open/New to an empty document still requires old buffers to be released.
- Forty-four STL cancellation/admission, 11 document-contract, 20 file-lifecycle and nine ordinary interactive-memory tests pass; six large interactive fixtures were ignored. Final targeted library build 24.93 s; integration build 24.82 s.
- Twenty-seven offscreen GPU/egui viewport/Gumball tests pass without skips in 51.05 s; ten hover tests pass in 3.71 s. No physical desktop latency or actual display presentation was measured.

## Limits

This transfers render-buffer ownership for Undo/Redo; it does not implement all scene metadata or lazy picking-cache ownership. Snapshot/Master replacement paths still need their own continuous admission and transfer. History scenes are rebuilt into new allocations rather than reusing an earlier scene cache; identical content in independent allocations is correctly charged separately. The pending-versus-allocated physical-memory model, native peaks, real OS exhaustion and current-source release/desktop measurements remain open. R0 and the overall plan are incomplete.
