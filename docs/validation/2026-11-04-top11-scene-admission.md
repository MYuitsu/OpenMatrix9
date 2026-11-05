# Top11 candidate-scene admission — 2026-11-04

Top11 previously constructed a complete replacement scene before adding its measured bytes to the prepared transaction's held-usage check. That left scene construction invisible to competing shared-account reservations until after allocation. For changed documents, Top11 now uses the existing candidate-scene commit path: register the prepared document, reserve planner/validation scratch and estimated scene construction before construction, reconcile actual scene capacity, commit and publish while the reservation remains held, then release it. An unchanged candidate retains its previous no-op path. This does not change geometry or intentionally change a command's successful result.

The first reuse attempt exposed an existing structural-accounting mismatch: `duplicate_budget_failure_keeps_prompt_for_retry` returned `Budget("objects")` for a valid retry because the shared helper subtracted the already-retained candidate document's object count before validating that same document again. The helper now restores the per-document object ceiling for scene validation, matching the old Top11 path; byte reservations remain charged. That existing test failed before this correction and passed after it.

## Tests

Default features, OCCT 8.0.1 runtime, E: debug target, `-j 2`, serial execution:

- `cargo test -p om9-app --test top11 -j 2 -- --test-threads=1`: 152 passed, including duplicate failure/retry and scene-budget refusal preserving document, scene and Undo. Build 21.56 s; tests 10.36 s.
- `cargo test -p om9-app --lib full_candidate_scene_reconciles_actual_capacity_before_commit -j 2 -- --test-threads=1`: one passed. It injects physical-memory pressure after construction, checks shared reservation visibility, preserves document/scene/Redo on refusal, then succeeds on retry. This tests the common helper, not an injected pressure point through Top11 itself.
- `cargo test -p om9-app --test top11_brep --test layer_visible_locked --test gesture_regressions -j 2 -- --test-threads=1`: 86 B-rep Top11, three layer and 11 gesture tests passed.
- Follow-up after the earlier GPU run was interrupted: `cargo test -p om9-app --lib ui::gumball_gpu_tests -j 2 -- --test-threads=1` passed all 27 offscreen GPU/egui fixtures without skips in 52.99 s. `cargo test -p om9-app --lib ui::top11::tests -j 2 -- --test-threads=1` passed five tests, including grip hit detection, selecting and deleting grips, and Undo. These fixtures simulate input and do not prove actual desktop mouse behavior or display presentation.

## Limits

The estimate is not a measured hard bound: allocator excess can be allocated before the actual-capacity reconciliation. The published scene is not yet a persistent registered owner; other scenes, caches, native OCCT temporaries and driver allocations remain incomplete. Top11 kernel/mutation preparation can allocate before this scene phase. No real OS exhaustion, representative large-file desktop interaction, release performance or process-wide RAM cap was established. The unchanged-candidate branch is not covered by the new scene reservation.
