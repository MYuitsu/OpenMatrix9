# Dice navigation investigation — 2026-11-02

The user video shows 5–6 FPS during navigation with a selected B-Rep dice in the debug desktop build. The viewport overlay uses egui's frame interval; it does not independently measure presented frames or identify a bottleneck. The saved dice contains 23,368 display vertices and 41,847 triangles.

On the existing Windows debug build, a temporary controller probe loaded the saved dice, selected its object and called `gumball_pivot` 100 times. The mean was 22.91 ms per call. This is a microbenchmark, not a measurement of overall viewport FPS, and the concurrent release compilation may have affected it. The probe executable ran one passing test; its temporary test source was removed. Cargo's encompassing test command separately failed to link the running debug application's PDB (LNK1318).

The current source caches the calculated selection centre by document ID, scene epoch and selected IDs. The axis alignment and relocated origin are still resolved on each call. `cargo check -p om9-app --lib -j 2` passed with `CARGO_TARGET_DIR=L:/om9-target`; the public source audit reported zero errors. The release build and subsequent library test failed with disk-full errors on L:, so the cache has not been benchmarked, regression-tested or exercised in the live application. The running binary remains unchanged; 30 FPS is not established.

Next: free space on L:, run selection, Gumball, document replacement, undo and navigation regression tests; repeat the same dice pivot probe; build a current release with `-j 2`; compare selected and unselected orbit, pan and zoom in the visible window using interaction/present timing. Investigate remaining costs if the 30 FPS target is not met.
