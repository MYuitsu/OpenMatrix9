# Boolean request: batched exact-source lease admission

The user requested diagnosis and optimization of the largest pauses when sending Boolean, followed by 64/128-sphere Light/High retests retaining exact CAD data. The preceding [architecture experiment](2026-11-15-coupled-source-display-ab.md) left approximately 600 ms High / 450 ms Light request-time active gaps at 128 spheres.

## Cause and scoped change

`UnionJob::start` previously called `capture_brep_source` once per operand. Each call validates retained document/history owners and the live reservation/lease inventory. The core already provides `capture_brep_sources_into`, which checks one immutable root-source list and admits every lease atomically with a single grouped inventory. The earlier [core-only scaling probe](2026-11-05-brep-source-capture-scaling.md) motivated this API but did not measure the actual Boolean UI path.

The Boolean caller now uses that existing batch API, preserving selected input order, exact source/topology owners and per-input `SourceStamp`/placement checks. It retains source-type checks, target-set validation, rigid/shared-affine rules, the worker permit, cancellation, epoch/authority/command/selection/layer/publication guards, memory admission and guarded atomic publication. No native geometry algorithm, Boolean tolerance, tessellation parameter, wire-buffer preparation or RDP/render policy changed. It applies to the shared request constructor used by Union, binary operations, Split and Two Objects preview; the large-scene measurements below target Union.

The temporary captured-source vector is explicitly included in metadata admission. Its capacity remains conservatively reserved through handoff, as the former selected-ID capacity already was. If the existing batch API's source-count limit is below an otherwise valid command input count, the caller keeps the former per-object path rather than introducing a new command limit. No quota validation is disabled or replaced with an assumed constant.

`OM9_BOOLEAN_REQUEST_BASELINE=1` restores per-object source capture on the same executable. `OM9_BOOLEAN_REQUEST_PROFILE=1` emits cumulative request phase timings and accumulated serial-capture time to stderr. These are diagnostic flags, not product preferences. The opt-in native audit records a `boolean_request_return` marker and the job's actual `(batched, operand_count)` witness; the analyzer checks the selected path and keeps exact request-call duration separate from callback gaps, complete Boolean wall wait and final UI receive/commit.

## Preparation and diagnostic profiling

Twenty focused integration tests passed across `boolean_request_capture`, `boolean_publication`, `viewport_detail`, `coupled_viewport_control`, `navigation_deadlines`, `display_modes` and `display_persistence`; the request-only diagnostic is separately ignored by the normal suite. The new regression verifies requested capture order, exact source identity/bytes/account binding, rejection of invalid input and quota pressure without partial output, unchanged document/Undo/Redo and recovery after releasing pressure. Existing tests verify normal requests actually use the batch path, native source measures, Surface Snap, source wires, cancellation/stale guards, managed refusal, binary operations and Undo/Redo.

The explicitly invoked debug controller probe creates 128 real spheres, times only request setup and cancels before completing Boolean. It is not a GPU/FPS or completed-Boolean test. It overlaps the release build, so its absolute times are diagnostic, not release performance evidence. In its High serial sample, source capture took 11,643.944 ms of a 11,747.871 ms constructor call; selection/placement took 0.171 ms and initial admission completed at 101.562 ms. This identifies repeated source-lease admission as the dominant measured request phase.

Probe receipt/logs: `build/standalone-validation/ui-artifacts/boolean-request-probe-20261115/`. After the High serial sample, the optional debug probe was stopped during expensive Light fixture construction; its batch half was not measured. This is an interrupted diagnostic, not a passing completed Boolean test. Its owned Python/test processes were verified stopped before formal release timing; no build/test/probe load overlapped the formal runs.

## Frozen-release workload

Completed native-window batch: 64 and 128 spheres, Light and High, serial and batched capture on one release executable, two alternating A/B pairs per profile/size (16 complete runs). Both arms use the normal separated architecture, unchanged quality/motion/worker publication, AutoNoVsync and a 30-Hz CAD clock. They use the existing disjoint intersecting R5 sphere pairs and BooleanUnion DeleteInput=Yes, with gesture-relative synthetic pointer navigation before/after the worker. Drag pauses during guarded publication.

Acceptance requires normal completion, zero diagnostic drops, expected actual capture mode/count, matched exact result source SHA and display density per profile, native volume/bounds/closedness/solid-count/naked-edge oracles and final full-size 4x targets. Steady FPS counts distinct encoded Perspective frames; it is not native callback rate or RDP client video FPS. All active creation/request/commit gaps remain in freeze accounting, with final settling/idle reported separately. Full library-unit compilation and broader canonical-source/persistence migration remain outside these scoped checks.

## Frozen release results

All **16/16** native-window runs passed on release executable SHA-256 `9288dd4e8a1b09c5c082620e4ca2685faa4d33773cc8e8bb430ce6395241c0f1`, with matching frozen source receipts at both scene sizes. Each quality/size has two alternating A/B pairs. The kernel, exact-source/tolerance, display density, motion rendering, publication and RDP policy are unchanged between each pair.

Reproduce each scene with `python tools/benchmark_sphere_viewport.py --request-comparison --pairs 32 --repeats 2 --present-mode auto-no-vsync --output <new-directory>`, using 64 pairs for 128 spheres. Normal commands, source oracles and capture-path/zero-drop/final-target assertions execute in native Rust windows; Python orchestrates and analyzes.

| Spheres | Profile | Send request serial → batch, ms | Reduction | Whole request-transition gap, ms | Worst active gap, ms | >100 ms gaps per run |
|---:|---|---:|---:|---:|---:|---|
| 64 | high | 47.473 → 2.501 | 94.7% | 179.065 → 138.046 | 182.303 → 138.702 | [1, 1] → [2, 2] |
| 64 | light | 46.234 → 3.361 | 92.7% | 86.044 → 46.331 | 86.599 → 86.507 | [0, 0] → [0, 0] |
| 128 | high | 365.816 → 8.880 | 97.6% | 622.559 → 265.095 | 626.482 → 266.090 | [62, 65] → [63, 62] |
| 128 | light | 364.958 → 8.683 | 97.6% | 443.780 → 85.226 | 447.335 → 210.227 | [47, 46] → [47, 45] |

Request and transition values are medians of two samples. Worst active gaps are the maximum across both complete runs, including all creation, fitting, request and commit pauses. The whole request transition includes selection and UI work after synchronous request return; it must not be reported as source-capture duration. Earlier reports labeled these full gaps broadly as request setup; the new return marker resolves the distinction. Final settling/idle waits remain separate.

| Spheres | Profile | Capture | FPS before / after Boolean | Encoded p95, ms | Steady max, ms | Final UI poll, ms | Total Boolean wait, s |
|---:|---|---|---:|---:|---:|---:|---:|
| 64 | high | serial | 30.000 / 29.995 | 36 | 37 | 56.087 | 55.255 |
| 64 | high | batch | 30.002 / 30.001 | 36 | 41 | 54.893 | 54.025 |
| 64 | light | serial | 30.004 / 29.995 | 36 | 38 | 24.759 | 25.072 |
| 64 | light | batch | 29.998 / 29.995 | 36 | 37 | 23.830 | 25.108 |
| 128 | high | serial | 30.000 / 30.017 | 36 | 37 | 155.084 | 199.225 |
| 128 | high | batch | 29.991 / 30.001 | 36 | 37 | 149.389 | 198.819 |
| 128 | light | serial | 29.998 / 29.993 | 36 | 38 | 88.601 | 92.365 |
| 128 | light | batch | 29.998 / 29.997 | 36 | 38 | 88.288 | 91.450 |

FPS/final polls/Boolean waits are descriptive medians; encoded p95 and steady maxima are worst across both runs and motion phases. Total Boolean wait includes background native computation and is not a continuous UI freeze. Final poll is complete UI receive/validate/prepare/publish. This change targets request preparation, not native Boolean computation or presentation pacing.

## Exact-source evidence and remaining pauses

All eight runs per scene produced identical canonical native result source bytes across capture paths and quality profiles, while display triangles matched within each profile. Every result was closed, had zero naked edges and the expected solid count, analytic volume and bounds. Both modes restored all four targets to 649 × 410 with four samples. There were no dropped diagnostic records. Full source witnesses and receipts are retained in the companion JSON.

| Spheres | Native solids | Native volume, mm³ | High triangles input / result | Light triangles input / result |
|---:|---:|---:|---:|---:|
| 64 | 32 | 28274.333882766 | 320512 / 264448 | 33024 / 28032 |
| 128 | 64 | 56548.667765580 | 641024 / 528896 | 66048 / 56064 |

Batch admission removes the repeated retained-owner inventory without bypassing leases, quotas or stale-source guards. A successful request now returns much sooner; associated foreground UI redraw/selection, creation/scene fitting, worker admission polling and final publication remain measurable. The request-transition native timing witnesses show why the remaining callback gap must be evaluated independently from the small request call. Per-point multi-selection frame projection in `ui.rs::selection_frame` is a candidate for the remaining transition cost; it is a code-inspection lead, not an isolated timing result or an optimization claimed here.

At 128 spheres, optimized high runs retain [63, 62] active gaps over 100 ms; [59, 58] of those per run are attributed to sequential Sphere creation (stage 1). Counts need not fall when an improved request-transition gap still exceeds 100 ms.

At 128 spheres, optimized light runs retain [47, 45] active gaps over 100 ms; [45, 43] of those per run are attributed to sequential Sphere creation (stage 1). Counts need not fall when an improved request-transition gap still exceeds 100 ms.

The scoped integration tests cover the shared Boolean entry points, but the 64/128 native workload is Union over independent intersecting sphere pairs. It does not prove all-overlapping/general scenes, arbitrary native families, local-console behavior, full canonical/persistence migration or RDP client displayed FPS. The steady 30-Hz cap prevents a maximum-throughput FPS inference. No CAD business rule changed. The optional stopped debug probe is not counted among the twenty passing tests or sixteen completed release runs; existing full library-unit migration failures remain outside this focused validation.

Artifacts:

- `build/standalone-validation/ui-artifacts/boolean-request-64-20261115-ab/`: 8/8 passed, raw JSONL, frozen binary/source receipt and summary.
- `build/standalone-validation/ui-artifacts/boolean-request-128-20261115-ab/`: 8/8 passed, raw JSONL, frozen binary/source receipt and summary.

Focused validation command: `cargo test -p om9-app --target-dir L:/om9-target -j 2 --test boolean_request_capture --test boolean_publication --test viewport_detail --test coupled_viewport_control --test navigation_deadlines --test display_modes --test display_persistence` (20 passed, 1 ignored request-only diagnostic). Release build: `cargo build -p om9-app --bin om9 --release --target-dir L:/om9-target -j 2` passed. The benchmark Python module compiled successfully; public-source audit results are recorded in the companion JSON.

[Machine-readable results, source witnesses and per-run measurements](2026-11-15-boolean-request-source-batch.json).
