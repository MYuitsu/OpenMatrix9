# Source/display separation: controlled coupled-document reconstruction

The user requested a runnable unseparated control versus the current implementation on identical 64/128-sphere Boolean scenes, retaining exact CAD data. This record separates that question from Light/High quality, UI-worker publication and Remote Desktop VSync changes.

## Counterfactual definition

The repository HEAD's former `Geometry::Brep(Brep)` stores an exact `Arc<String>` source and an `Arc<BrepDisplay>` together. It already uses a tessellated GPU mesh. Rebuilding a mesh every camera frame would invent a slower baseline and is not done here.

Structural reference: `dc2d4dfd665e14d1a68aa9c30f1fd59daca6db02`, `crates/om9-core/src/brep.rs` and `geometry.rs`. That historical checkout is not the executable under test.

The opt-in control reconstructs that document coupling on the current runtime:

- Each registered fresh BRep document object retains the same admitted display mesh and its lease, alongside exact source/topology. Document/history clones retain that display.
- Geometry validation checks the combined source/display using the existing `Brep::validate_admitted`; serialization/fingerprinting uses the existing complete-pair `Brep::serialize` boundary. The separated arm validates/serializes its canonical source independently.
- Scene planning, rendering and pick-display pinning read the coupled mesh from its document owner. The separated arm resolves its independent representation cache. Both retain normal GPU-buffer/image reuse; neither remeshes on camera motion.
- Source-only kernel APIs, the exact Boolean worker, worker-prepared wire buffers, guarded atomic publication, modern memory-admission policies, source identities and existing failure checks are kept in both arms. Display allocations remain charged once through their retained leases; the coupled attachment metadata is separately admitted. No precision or validation guard is disabled.
- This is a scoped architectural ablation, **not** the original historical executable or a rollback of every migration change. It measures document/display lifetime, validation and fingerprint coupling with other improvements held constant. It does not measure old archive behavior; the diagnostic is throwaway-only and introduces no supported file format.

`OM9_SPHERE_BOOLEAN_AUDIT_ARCHITECTURE=coupled|separated` is read only by the opt-in empty-document native audit. The normal product remains separated. The control requires fresh admitted OCCT BRep candidates; it explicitly refuses the unsupported shared-candidate registration path. It is not a general alternate CAD backend or a persisted preference.

## Matched workload and measurements

Both arms use the same executable, OCCT build, quality profile, 30-Hz CAD clock, RDP presentation mode, viewport size, MSAA/motion policy, creation order, authored positions and BooleanUnion DeleteInput=Yes. Light and High are compared **within the same profile**. The two scenes contain 32/64 disjoint intersecting pairs of radius-5 spheres, pair-center distance 5 mm and lattice spacing 20 x 16 mm. The result is one BRep compound with 32/64 closed solids.

Synthetic drag phases use the same gesture-relative path and speed before and after Boolean. Drag pauses while the worker publishes, as in the existing harness. FPS counts distinct encoded Perspective frame IDs rather than UI callbacks. Steady windows trim the first 1 s and final 250 ms; all creation/request/publication gaps remain in active-freeze accounting, with idle settling reported separately.

Every run must prove its actual path through per-input/result `coupled_display` flags and positive combined-validation counters for the coupled arm. Paired source bytes, input/result triangle counts, native volume/bounds/closedness/solid-count/naked-edge oracles and final full-size four-sample targets must match. The source SHA records exact native source bytes independently of the diagnostic serialization boundary.

Planned frozen-release batch: two alternating A/B repetitions for each profile at both 64 and 128 spheres, eight runs per scene size, sixteen runs total. Debug pilots are path checks, not accepted release performance measurements. Current test/build/results receipts will be recorded below after completion.

## Verified preparation

Nineteen focused integration tests passed across `coupled_viewport_control`, `boolean_publication`, `viewport_detail`, `navigation_deadlines`, `display_modes` and `display_persistence`. The new test exercises both architectures and both profiles, identical exact Union source bytes/display counts/bounds, volume/closedness/solids, source/display path witnesses, independent-cache-free coupled scene construction, source-derived wires, Surface Snap refined to an analytic source point, atomic publication and Undo/Redo. The new control test was rerun after adding complete-pair serialization and passed again. The ordinary product serialization branch is unchanged.

All four small native-window debug pilots passed exact-source SHA and mesh-count equality per profile, actual path/counter checks, source oracles, final targets and zero-drop checks. Receipt: `build/standalone-validation/ui-artifacts/coupled-display-pilot-20261115/`. A release build overlapped the pilot, so its FPS/duration values are excluded from performance conclusions. Public-source audit: 16,071 files, 594 bindings, 593 SVGs, zero errors. Full library-unit compilation remains affected by the previously documented canonical-migration errors and is not counted as passing.

The final default-feature release build completed with cargo build -p om9-app --bin om9 --release --target-dir L:/om9-target -j 2. The six focused integration targets were rerun on the final code: 19/19 passed, including the added source-snap checks. Formal measurements start only after all build/test processes finish.

## Frozen release results

All **16/16** runs passed on executable SHA-256 `cdf3d39173c748395f72fd7f3c385b8178ded8a4a25f6ef9d8affc86c74278df` with matching runtime-source receipts at both scene sizes. Build/tests completed before all formal runs. Two alternating pairs were run per profile at each size. Geometry source bytes were identical across all eight runs at each size; triangle counts matched within each profile.

Reproduce each size with `python tools/benchmark_sphere_viewport.py --architecture-comparison --pairs 32 --repeats 2 --present-mode auto-no-vsync --output <new-directory>`, using --pairs 64 instead for the 128-sphere scene. Normal commands, source oracles and zero-drop/final-target assertions run in Rust/native windows; Python orchestrates and analyzes.

| Spheres | Profile | Architecture | FPS before / after Boolean | Encoded p95, ms | Steady max, ms |
|---:|---|---|---:|---:|---:|
| 64 | high | coupled | 29.997 / 29.923 | 36 | 70 |
| 64 | high | separated | 29.996 / 30.004 | 36 | 37 |
| 64 | light | coupled | 30.004 / 30.004 | 36 | 38 |
| 64 | light | separated | 30.002 / 30.004 | 36 | 37 |
| 128 | high | coupled | 30.005 / 29.999 | 36 | 37 |
| 128 | high | separated | 30.002 / 30.006 | 36 | 38 |
| 128 | light | coupled | 30.002 / 29.995 | 36 | 37 |
| 128 | light | separated | 30.004 / 30.001 | 36 | 43 |

| Spheres | Profile | Coupled → separated UI final poll, ms | Reduction | Creation total, s | Worst active gap, ms | >100 ms gaps per run |
|---:|---|---:|---:|---:|---:|---|
| 64 | high | 216.669 → 53.514 | 75.3% | 7.271 → 3.707 | 256.864 → 182.873 | [46, 45] → [1, 2] |
| 64 | light | 41.305 → 24.584 | 40.5% | 2.792 → 2.471 | 88.129 → 86.112 | [0, 0] → [0, 0] |
| 128 | high | 486.004 → 151.792 | 68.8% | 27.008 → 12.989 | 621.254 → 612.447 | [110, 110] → [61, 61] |
| 128 | light | 125.533 → 91.826 | 26.9% | 11.805 → 10.279 | 437.603 → 453.508 | [60, 57] → [48, 47] |

Final polls, creation totals and FPS are medians of two runs. P95/steady maxima and largest active gaps are the worst across both runs/phases. Final poll is the complete UI receive/validate/prepare/publish call, not isolated native kernel time. Active pauses include all Sphere creation, scene fitting, request and commit transitions; final idle waits remain separate.

| Spheres | Profile | Total Boolean wall wait coupled → separated, s | Moving result GPU p50 coupled → separated, ms |
|---:|---|---:|---:|
| 64 | high | 57.359 → 52.980 | 1.176 → 1.663 |
| 64 | light | 24.572 → 24.814 | 0.578 → 0.498 |
| 128 | high | 198.372 → 205.811 | 1.149 → 1.801 |
| 128 | light | 91.738 → 92.180 | 0.893 → 0.618 |

## Interpretation and evidence limits

The measured benefit is edit/publication responsiveness. High final UI calls fell by 75.3% at 64 spheres and 68.8% at 128; Light fell by 40.5% and 26.9%. High foreground creation totals fell by 49.0% and 51.9%. Both architectures remain near the configured 30 encoded-FPS cap, without a demonstrated material steady-FPS gain. At 128 spheres, the largest active pauses remain about 621 to 612 ms High and 438 to 454 ms Light: no clear improvement in the worst request-time pause. Total Boolean wall wait is not consistently reduced.

This experiment attributes differences to the reconstructed document/display coupling boundary with tessellation, worker publication and RDP policy held fixed. It does not attribute the previous VSync-related 10.6 → 30 FPS gain to source/display separation. The steady camera path reuses cached GPU buffers in both arms; the 30-Hz cap limits any maximum-FPS inference. Evaluate the measured edit/publication pauses separately from navigation FPS and total worker wall wait. The native Boolean mathematics are unchanged.

The control is not the entire historical runtime: modern source-only kernel APIs, admission/accounting, worker orchestration and cache infrastructure remain common. Its complete-pair serialization is used only by the throwaway benchmark and is not a supported save/reopen format. Results concern the specified disjoint-pair sphere fixtures; all-overlapping/general scenes, other native geometry families, pressure/persistence coverage and RDP client displayed frames remain outside this A/B evidence. Synthetic pointer timing is gesture-relative; OS event delivery and exact sampled camera poses need not be bit-identical. No geometry precision or validation guard was relaxed.

Artifacts:

- `build/standalone-validation/ui-artifacts/source-display-ab-64-20261115/`: 8/8 passed, raw JSONL, binary/source receipt and summaries.
- `build/standalone-validation/ui-artifacts/source-display-ab-128-20261115/`: 8/8 passed, raw JSONL, binary/source receipt and summaries.

[Machine-readable receipts, source witnesses, per-run summaries and aggregates](2026-11-15-coupled-source-display-ab.json).

Final public-source audit after recording results: 16,072 files, 594 bindings, 593 SVGs, zero errors. Validation-artifact pruning retained the newest ten plus cited evidence and removed no cited runs.
