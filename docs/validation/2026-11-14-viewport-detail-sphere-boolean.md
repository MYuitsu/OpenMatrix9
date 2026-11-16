# Viewport detail: two spheres and Boolean Union

Follow-up: [Light pointer-navigation optimization and matched release measurements](2026-11-14-light-navigation-smoothness.md). It distinguishes actual navigation-event replay from this original direct-camera timing and records reduced moving GPU cost without claiming higher steady FPS under the 30 Hz cap.

The user approved a manual viewport parameter, Light by default, and requested implementation and measurement of Sphere → Sphere → Boolean Two Objects Union. This authorizes the scoped tests below despite the earlier compile-only core-migration sequence. It does not close the full source/display migration.

## Implementation

DISPLAY offers Light (linear 0.1 mm, angular 0.5 rad) and High detail (0.01 mm, 0.3 rad). Both use a canonical BRep for computation and a separate representation for GPU display. These values are display parameters, not CAD/export tolerances. There is no automatic hardware classification.

Sphere creates its requested profile directly, including its admitted fallback constructor. The existing C++ OCCT primitive adapter accepts angular deflection through a new profile entry point; the old ABI delegates with 0.3 rad. Rust owns preference, validation, state, memory admission, switching and publication. Native OCCT internal allocations remain outside the Rust callback's accounting.

Boolean workers take a display request from the session rather than an operand's viewport representation. The output source is captured with in-process topology attestation before preview publication; the committed output retains evidence with its insertion history. The capture currently performs an additional output tessellation to obtain attestation/angular profile; it is not a source-only Boolean native adapter. No new geometric Boolean tolerance is introduced.

Profile switching uses admitted remesh workers serially and prepares all root BRep representations against the same original scene. Only the final combined scene is published. Failure, cancellation, stale scene and quota refusal retain the old scene and effective preference; command dispatch is busy during switching. This path requires attested root BReps. Other/opaque native geometry and complete legacy migration remain outside the tested slice. Cache keys on producer displays may remain conservatively unspecified; batch switching rebuilds candidates and does not promise a warm-cache shortcut.

`DisplayPreferences.viewport_detail` is additive to schema version 1 and missing fields default to Light. Existing unsupported BRep scenes are not labeled as newly rebuilt Light scenes. The existing BRep v5 save refusal remains; ordinary preference/preset persistence is covered, not BRep save/reopen completion.

## Semantic verification

`cargo test -p om9-app --target-dir L:/om9-target -j 2 --test viewport_detail --test display_persistence --test display_modes -- --nocapture`: **9 passed, zero failed**. The four new profile tests cover:

- Sphere and Boolean menu/CMD dispatch, same input source text, closed Union, one solid, zero naked edges, analytic volume `1125π/4 = 883.572933822... mm³`, bounds `[-5,-5,-5,10,5,5]`, Undo/Redo and profile switching without document changes.
- Two-root atomic switching and cancellation while a worker is active; a subsequent request can wait for the canceled proof worker to release its permit.
- Rejection of an intervening scene mutation and managed budget refusal without publication.
- Surface Snap refined to an independent analytic sphere point in both profiles; old display JSON without the new field.

| Representation | High triangles | Light triangles |
|---|---:|---:|
| Two radius-5 spheres | 10,016 | 1,032 |
| Union result | 8,264 | 876 |

The full `--lib` test target does not compile: 319 errors were reported in tests affected by the existing canonical-model migration. Those tests were not repaired or counted as passes. A default-feature release build succeeded; a debug native-window smoke run completed with zero dropped records, correct geometry and a normal diagnostic end.

## Reproduction and timing

Build: `rtk proxy cargo build -p om9-app --bin om9 --release --target-dir L:/om9-target -j 2`.

Run: `rtk proxy python tools/benchmark_sphere_viewport.py --output build/standalone-validation/ui-artifacts/viewport-detail-20261114-ab`.

The Python tool orchestrates Windows processes and analyzes JSONL. Rust performs CAD, command replay and timing. The opt-in `OM9_SPHERE_BOOLEAN_AUDIT=high|light` starts only on an empty document with `OM9_INTERACTION_LOG`; each run warms for five seconds, creates the first sphere, measures five seconds, creates the second, measures five seconds, performs Union preview/commit and measures the result for five seconds. Camera speed is time based. There are six Shaded runs in High/Light, Light/High, High/Light order and one Wireframe run per profile. Menu and CMD dispatcher origins alternate; this is not a physical menu click test.

Frame records originate in the native raw-input hook, not per-pane/per-layout calls. Command markers use the application's monotonic clock. The analysis includes long intervals across stage boundaries so slow commands cannot vanish from results, and checks geometry, completion, GPU configuration and lost records. It also counts unique encoded Perspective frames. Neither callback cadence nor encoded frames establish displayed Remote Desktop FPS or input-to-photon latency. The previous approximately 10.7 callback/s long-camera audit is not a comparable baseline.

## Paired release results

All **8/8 native-window runs** completed, checked the analytic Union volume/bounds/closedness, configured the GPU and reported zero diagnostic drops. Binary SHA-256: `3c5dfd970903ad0b788e6c46976389a30cdf887836a1e1267f52a73e8dc8c66b`, default features, optimized release, NVIDIA RTX 3060/Vulkan/AutoVsync. Raw logs, per-run summary and binary receipt: `build/standalone-validation/ui-artifacts/viewport-detail-20261114-ab/`. This is one machine/session and a small authored fixture, not general performance coverage.

The following are medians of the three Shaded runs per profile, except the maximum callback interval, which is the worst observed in all three after the first sphere command begins (includes transitions and commit):

| Metric | High detail | Light |
|---|---:|---:|
| Encoded Perspective frames/s, two spheres | 29.984 | 29.992 |
| Encoded Perspective frames/s, result | 29.984 | 30.167 |
| Raw callback cadence/s, result | 32.253 | 32.232 |
| Result callback p95 | 31.787 ms | 31.756 ms |
| First sphere command | 45.623 ms | 26.585 ms |
| Second sphere command | 43.996 ms | 24.635 ms |
| Boolean request to preview observed ready | 247.005 ms | 122.624 ms |
| Commit on application thread | 65.455 ms | 61.519 ms |
| Worst callback interval after first command | 71.749 ms | 68.007 ms |
| Callback intervals >100 ms after first command | 0 | 0 |

Each Wireframe profile also completed with the same geometry and triangle counts; preview latency was 247.143 ms High and 122.434 ms Light. The three Shaded preview samples were High 247.005/246.501/247.767 ms and Light 122.525/122.632/122.624 ms. Light reduced the observed preview wait by about 50%; steady rendering was approximately 30 encoded frames/s in both modes. No meaningful steady-FPS improvement is established under this cap; no >100 ms stall was observed in the measured command-to-result portion. The largest intervals are callback gaps, not proven display freezes. GPU-present time and RDP client display remain unmeasured.

An actual Windows UI follow-up opened the updated debug build, observed Light by default, clicked the DISPLAY dropdown, selected High detail, then selected Light again on an empty document. The controls fit the existing row and both effective labels updated. Two-object atomic replacement, cancellation and precision evidence come from integration tests; the physical dropdown check itself used an empty document. The updated debug window is left open in Light mode.

Final checks: the three focused integration targets passed again (9 tests); `cargo check -p om9-app --no-default-features --lib --target-dir L:/om9-target -j 2` passed with warnings (compile-only fallback, not native runtime evidence). `tools/public_source_audit.py` reported 16,061 source files, 594 icon bindings, 593 SVGs and zero errors. `git diff --check` on the changed implementation paths passed. Artifact pruning removed no runs and retained the newest ten plus cited evidence. The tested Solid viewport rule was synchronized to the repository and installed Solid guidance under the standing Solid-skill update authorization.
