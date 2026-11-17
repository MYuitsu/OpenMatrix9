# Native 512-sphere provisional Boolean comparison — 2026-11-15

Both profiles were attempted on the same recovery-audit release executable. **Light completed; High published its provisional result but the exact worker failed before document/scene commit.** This is not a passing two-profile final-result comparison. The pending native matrix was executed, with one open product failure.

| 512 spheres | Light | High detail |
|---|---:|---:|
| CPU provisional result ready | 2.911 s | 4.333 s |
| Published provisional viewport content | 3.830 s | 5.360 s |
| Exact source/document/scene commit | 783.798 s | Not committed; worker failed after 884.358 s |
| Encoded FPS before Boolean | 26.30 | 25.13 |
| Encoded FPS while waiting | 29.32 | 27.75, until failure |
| Encoded FPS after exact commit | 29.93 | Not available |
| Largest Boolean UI callback gap | 1658.2 ms | 858.1 ms, no successful final commit |
| Largest Boolean fresh-GPU gap | 2299 ms | 2640 ms, until failure |
| Resident RAM peak | 672.9 MiB | 1005.6 MiB |
| Resident RAM at performance_end / failure | 591.2 MiB | 760.0 MiB at failure, not settled result RAM |
| Undo / Redo after attempt | 513 / 0 | 512 / 0 |
| Document objects after attempt | 1 compound | Original 512 spheres |

The fixture consists of 256 independent intersecting radius-5 mm pairs, then BooleanUnion with DeleteInput=Yes. A successful result is one entity containing 256 closed solids. Both runs retain normal Sphere-command Undo and use the same prior **process-only object cap 200000**; configured point cap 2000000, CAD tolerance and loaded Undo policy are unchanged. Default-limit 512 acceptance is not claimed.

Preview time means the first audit observation of published Perspective content carrying the temporary mesh key. FPS counts fresh encoded Perspective frames over the complete fixed interior stage-window duration, including first/last frame holds. It is not measured RDP-client video FPS or latency. The initial strict analyzer rejected Light's 1411 ms trailing hold; the revised calculation measures that hold instead of excluding its time. All completed 64/128 traces were reanalyzed consistently. High's failure has a separate partial analyzer; successful-result checks were not bypassed to label it a pass.

Light passed native analytic volume/bounds, 256 closed solids and zero naked edges. Source SHA-256 `ae813c1a93bb232bccb9bb0057249423c6c70b74edd55d8d344b92bf90e80f19` matches the earlier Light source and independent kernel proof. Input/result/provisional triangles: 264192 / 224254 / 183296. High provisional triangles: 234496. High has **no verified final result source hash, volume, display density or post-commit FPS**. The original inputs and 512 Undo remain after failure; the live state was checked independently through OM9's bridge.

The High worker-complete marker reports `ok=false`, final poll 0.647 ms. The audit then reports that Union did not commit. The exact error text was displayed only in the application CMD and is absent from these markers; it was requested from the user. A reduced point budget during source-buffer preparation is an inspection candidate, not an established diagnosis. Kernel-only High capture previously passed with 1120000 points, but that does not prove the whole worker preparation/publication path. No point cap was raised to obtain a successful result.

## Creation remains slow

| Complete creation/navigation attempt | Light | High |
|---|---:|---:|
| Largest active UI callback gap | 7.514 s | 7.085 s |
| Largest fresh-GPU image hold | 903.034 s | 896.047 s |

Rapid scripted creation starts the next Sphere before asynchronous scene publication catches up. These long image holds occur during creation, while the individual commands progress. The near-30 Boolean waiting/result FPS does not establish smoothness throughout creation. High's smaller pre-failure UI gap cannot establish smoother successful publication because that publication never occurred.

## Validation and provenance

The preceding 64/128 Light/High native profiles and 16 focused integration checks passed. Only the opt-in audit and analyzer changed for this resumed task: synthetic orbit re-arms after native gesture cancellation, and FPS requires fresh frame identity/rendered geometry with measured endpoint holds. A release rebuild passed in 5m45s with `-j 2`, reusing `L:/om9-target`. Two 16-sphere native Light/High smoke profiles deliberately injected focus loss and verified recovered dragging, GPU content, preview-before-exact, precise geometry and 17 Undo. Boolean/kernel/renderer implementation source is unchanged relative to the earlier frozen 64/128 build. No build overlapped the formal 512 runs.

Three earlier pilots are excluded: minimized/inaccessible desktop, interrupted pointer replay and direct-camera revisions that encoded but did not publish stable preview content. The current Light and High attempts use the recovered pointer path. The full lib-test target still has the previously recorded canonical-BRep fixture compilation failures; no full-suite acceptance is claimed. Native dependencies are not certified memory-safe by this validation.

Artifacts remain under `build/standalone-validation/ui-artifacts/boolean-speculative-20261115/`: `native-512-recovered/light-summary.json`, `native-512-high-recovered/failure-summary.json`, both raw traces/binary receipts, GPU coverage, smoke receipts and build logs. Full structured data and prior 64/128 results: [results JSON](2026-11-15-speculative-boolean.json). Implementation details and historical attempts: [implementation report](2026-11-15-speculative-boolean.md).

Open: High exact-worker failure/detail, batch-creation holds, repeated statistics, all-overlap/general geometry, other Boolean preview families, complete Undo replay, full migration/persistence and manual/client-video acceptance. The next step is to capture the precise High CMD error and isolate its worker buffer/admission stage. No six-profile success or manual acceptance is claimed.
