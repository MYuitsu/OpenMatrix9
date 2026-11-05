# Persistent buffer lease for full candidate scenes — 2026-11-05

The common full candidate-scene path previously released its whole construction reservation after publication. Its new scene buffers were not registered again until viewport or hover preparation. It now includes the buffer-lease allocation in pre-commit admission, splits the existing construction allowance without changing the account total and stores the buffer owner with the prepared scene. The normal commit and plugin paths publish that owner with the scene. A validation/commit failure drops scene data before the buffer owner and remaining construction reservation. No fallible buffer registration is added after a successful commit.

Full publication updates detached STL's scene high-water estimate using only bytes not covered by the new buffer owner. Indexed source identity, topology and command geometry behavior are unchanged. Empty scenes need no buffer owner.

## Verification

E: debug target, default features, OCCT 8.0.1 runtime, Cargo `-j 2`, serial tests:

- Six incremental/full scene pressure tests passed. The full candidate test checks actual capacity after an intentionally undersized construction allowance, refusal with document/scene/Redo intact, and the exact persistent buffer plus owner byte charge after successful retry.
- Five plugin tests passed. Invalid staged selection now compares the remaining charge with the live scene baseline, proving the refused candidate adds no retained buffer reservation.
- Forty-four STL cancellation/admission tests passed. The interleaved scene-growth test now measures registered buffer growth plus unregistered high-water growth, rather than expecting all growth in the unregistered counter. It still asserts over 720,000 bytes of total growth, export headroom reduction, cancellation preserving the old destination, and a fresh-capture reset after buffer registration.
- Eighteen clipboard, 11 document-contract, 20 file-lifecycle and nine ordinary interactive RAM tests passed. Three native clipboard and six large RAM tests were ignored in this run.
- The 152-test Top11 suite and 27 serial offscreen GPU/egui Gumball/viewport tests passed, with no GPU skips; GPU tests took 50.93 s. These do not prove actual desktop mouse or display latency.

Initial runs found the obsolete plugin zero-reservation assertion and the scene-growth assertion against the wrong counter. The test corrections preserve their intended no-leak and total-budget checks; no product tolerance or structural limit was loosened.

## Remaining work

This covers callers of the common candidate-scene helper, including plugin staged selection. Undo/Redo scene construction and snapshot/Master replacements still use other publication routes and need persistent-owner transfer. Incremental additions without a previously registered owner and lazy picking caches also remain separate gaps. Current admission still conflates already allocated leases with pending physical reservations and can refuse conservatively. Capacity excess is reconciled after allocation; native peaks, real OS exhaustion, large representative files and release/desktop behavior remain unverified. R0 and the complete plan are not closed.
