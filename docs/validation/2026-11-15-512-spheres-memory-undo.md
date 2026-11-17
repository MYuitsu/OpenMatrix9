# 512-sphere Light/High: RAM and retained Undo

The user requested scaling the current sphere/Boolean workload to 512 spheres and reporting RAM plus the Undo steps still retained. This is a workload measurement, not a CAD, display-quality or retention-policy optimization.

## Fixed workload and measurement boundaries

Use the current optimized separated viewport/source, batch Boolean request and prepared selection-frame projection. The disposable native-window audit creates 512 radius-5 spheres as 256 independent intersecting pairs, then BooleanUnion DeleteInput=Yes. Expected result: one entity containing 256 closed solids, zero naked edges, volume `72000*pi` mm³ and bounds `[-155,-125,-5,160,125,5]`. Compare Light/High source SHA and keep all native analytic/topology oracles. Source tolerance and both display profiles remain unchanged.

Plan: one completed run per profile, Light then High on a frozen release executable. These are individual samples, not a repeated statistical comparison. Loaded Undo preferences, the RAM ceiling and other structural guards remain enabled; the test object-cap override is qualified below. A diagnostic-only deadline may extend the old 600-second timeout up to 14,400 seconds; it does not accelerate or relax the geometry operation.

The opt-in resource audit reuses the existing Windows process-memory API and O(1) journal counters. Snapshots occur at startup, each 64 created spheres, before/after request, after Boolean and the settled performance endpoint. Windows working set is resident process RAM; private bytes are committed private memory. OS lifetime peaks include creation and computation. They are not VRAM, managed quota estimates, or whole-system RAM. System capacity/availability, current effective limits and actual Undo policy are also recorded.

Undo/Redo counts are read directly from the live coordinator. Cached Undo-byte estimates are labeled with their pending-inventory flag; full deduplicated history inventory is attempted once after timed rendering and native geometry verification. The raw key `undo_exact_inventory` refers to this full managed-storage inventory: it still uses conservative storage estimates and is not an OS RAM measurement. It does not clear or enlarge history. Performance-end peak RAM is reported separately from later verification/inventory. The number of retained steps does not mean every one of those steps was replayed.

FPS counts distinct encoded Perspective frames during steady pointer navigation before/after the worker. All creation/fitting/request/commit gaps remain in active freeze accounting; idle/settling and final validation are separate. Client RDP video FPS and physical pointer acceptance are not measured.

## Preparation

Twelve focused integration tests passed (`boolean_publication`: 5, `viewport_detail`: 6, `selection_projection`: 1). Python benchmark syntax and the preparation public-source audit passed (16,079 files, 594 icon bindings, 593 SVGs, no errors). Release build passed in 5m36s. Both native 8-sphere resource smoke runs passed: 8 Undo before / 9 after Boolean, Redo 0, identical native source SHA and successful exact history inventory. The qualified batch is finished: Light completed and High was refused before Boolean commit; outcomes are reported below. No production CAD, source-accuracy, memory-admission or Undo-retention code was changed.

Initial default-limit batch artifacts: `build/standalone-validation/ui-artifacts/512-spheres-memory-20261115/` (refused outcome below). Completed resource-smoke artifacts: `build/standalone-validation/ui-artifacts/resources-smoke-20261115/`. These contain binary/source receipts and native JSONL; a running batch must not be treated as completed evidence before its end/oracle checks.

## Default object-cap refusal and explicit stress retry

The default-limit Light attempt stopped on Sphere 447 with 446 objects, 446 retained Undo, Redo 0 and native error `budget exceeded: objects`. Peak resident RAM at refusal was 419.0 MiB; private commit was 657.6 MiB. This is not a RAM-capacity exhaustion or a completed 512/Boolean run. A diagnostic radius retry through the same live window returned the exact error, leaving document revision and Undo unchanged; Cancel allowed the disposable failed window to close normally.

The retained-version storage path adds `objects` while points/nodes use per-document maxima (`transaction.rs::add_retained_document_usage`, `validation_limits`). For this one-edit-per-Sphere fixture, versions through 446 account for 99,681 object slots; candidate version 447 reaches 100,128, above the default 100,000. The Undo memory policy at refusal remains minimum 1 step / 1024 MiB and its cached estimate is about 142.4 MiB.

A second batch explicitly uses the already supported process-only environment override `OM9_MAX_OBJECTS=200000` for both Light and High. It changes neither the product default, RAM ceiling, Undo policy, other structural guards nor CAD accuracy. The qualified batch is finished: Light completed and High was refused before Boolean commit; outcomes are reported below. Artifacts: `build/standalone-validation/ui-artifacts/512-spheres-memory-200k-20261115/`. The failed default batch and `failure-diagnosis.json` remain preserved separately.

## Completed release results

Two qualified 512-sphere attempts finished on frozen release SHA-256 `f0eb4e8ca42f6efe4aa3d3baabe66a27bd6a0b371afcdeee4c6f141e7ca49733`: **Light completed; High was refused**. The production analyzer correctly returned failure for High, so no all-passing `summary.json` was produced. `outcomes.json` preserves the passing Light analysis separately from the failed High partial metrics. Twelve focused tests, release build and two separate 8-sphere resource smoke runs passed. No compiler/test workload overlapped formal timing.

Both attempts used the explicit process-only object cap 200000 after the initial default cap stopped at Sphere 447. Default product limits remain unchanged; no point, RAM or Undo cap was raised. This is not default-limit 512 acceptance.

| Profile / endpoint | Resident before → endpoint, MiB | Peak resident, MiB | Private endpoint / peak commit, MiB | Undo before → endpoint | Redo |
|---|---:|---:|---:|---:|---:|
| light / performance_end | 467.7 → 600.5 | 616.4 | 1120.7 / 1140.7 | 512 → 513 | 0 |
| high / failure | 650.0 → 710.5 | 943.6 | 1197.8 / 1453.1 | 512 → 512 | 0 |

Light endpoint is the settled viewport before geometry oracles; High endpoint is the failure, with all 512 input objects still present. Their endpoint/peak numbers must not be advertised as a successful final-result RAM comparison. Light post-verification resident/peak were 619.0/624.7 MiB, outside the timed endpoint. Both retain minimum 1 step / 1024 MiB Undo policy, with Undo enabled. Light retains all 512 Sphere steps plus 1 Union; High retains the 512 Sphere steps and adds no failed-Union step.

Light full managed-history inventory after verification: `{'bytes': 195993573}` (about 186.91 MiB, conservative storage estimate). High cached history estimate at refusal: 186.02 MiB, inventory pending; no full High result/history inventory is claimed. These figures are not physical RAM or VRAM. Retained counts do not prove every step was replayed.

| Profile | Steady FPS before / after | Largest active callback gap, s | Gaps >100 ms | Creation commands, s | Boolean wait / refusal, s |
|---|---:|---:|---:|---:|---:|
| light | 30.008 / N/A | 133.725 | 889 | 1565.633 | 1580.896 |
| high | 29.995 / N/A | 20.384 | 925 | 1573.759 | 3076.037 |

Light request call 1057.983 ms; selection rebuild 53.846 ms; whole request transition 1828.896 ms; successful final UI poll 70392.950 ms. High request call 1046.366 ms; no successful receive/commit measurement exists. High's smaller maximum gap cannot establish smoother successful Boolean handling because it never reached final publication. Background wall wait is distinct from a continuous UI freeze.

Light post-result steady FPS is N/A: the 32.019-second post-commit callback gap exhausted the eight-second navigation window. High result FPS is N/A because there is no committed result. Callback FPS is never substituted. The two before-Boolean FPS values do not prove uninterrupted 30 FPS through the operation or RDP client-presented video FPS.

Light passed the analytic bounds/volume, 256 closed solids, zero naked edges, full-resolution restoration and zero-drop checks. Native result source SHA: `ae813c1a93bb232bccb9bb0057249423c6c70b74edd55d8d344b92bf90e80f19`; measured volume 226194.671062352369 mm³; input/output display triangles 264192/224254. High input display triangles 2564096; High output source SHA, topology and display counts do not exist, so no 512-sphere cross-profile result-source equality is claimed.

### High refusal diagnosis

The owned native CMD transcript showed `I/O: Boolean union: invalid creation input: BRep snapshot points`. The snapshot point inserter (`brep_snapshot.hxx:201`) rejects at its effective point ceiling; `brep.rs::snapshot_budget` forwards the remaining point limit. The captured group Union folds inputs sequentially and subtracts held intermediate storage before constructing each next snapshot (`editing.rs:1269-1295`). The old intermediate is kept admitted until the new body is accepted. Thus dense intermediate snapshots can exhaust the configured 2000000-point budget even when physical RAM is available. Exact iteration and effective remaining count were not logged; no claim is made that the final output alone exceeds two million points or that exact Boolean mathematics is invalid.

At refusal, 512 objects, Undo 512, Redo 0 and about 42.1 GiB available physical RAM were recorded. Worker completion was `ok=false`; no commit/end/result oracle occurred. The point guard was preserved. The owned test window closed normally after Escape canceled the pending command/Close; its PID exited. Diagnostic UI actions occurred after timed failure and are excluded from timings. See the raw folder's `high-failure-diagnosis.json` for observed error and static-code evidence.

### Limits and next action

One sample per profile; one successful and one refused outcome. Do not claim a fully passing Light/High 512 comparison, general geometry/all-overlap acceptance, replay of every retained Undo step, client video FPS or full migration/persistence. Exact CAD tolerances, RAM ceiling and loaded history policy remain unchanged; only the explicitly qualified test object cap differs from the product default.

Next inspect `crates/om9-kernel/src/editing.rs::boolean_union_captured_group_impl` and `brep_snapshot.hxx`: profile repeated intermediate snapshot/tessellation and point accounting before choosing a source-only intermediate fold or final-only viewport build. Separately profile foreground retained-version admission/inventory behind the approximately 20-second Sphere creation gaps. No such optimization was implemented in this measurement task. Extend the future post-commit navigation window from the first usable result frame before claiming steady result FPS.

Reproduce with process-only `OM9_MAX_OBJECTS=200000`, then `python tools/benchmark_sphere_viewport.py --pairs 256 --repeats 1 --profiles light high --resources --timeout-seconds 14400 --present-mode auto-no-vsync --output <new-directory>`. The point limit remains unchanged.

Artifacts: `build/standalone-validation/ui-artifacts/512-spheres-memory-200k-20261115/` (one pass, one refusal); `build/standalone-validation/ui-artifacts/512-spheres-memory-20261115/` (default object-cap refusal); `build/standalone-validation/ui-artifacts/resources-smoke-20261115/` (2/2 resource smoke). [Machine-readable evidence](2026-11-15-512-spheres-memory-undo.json).

## Final verification

Public-source audit passed: 16,081 source files, 594 icon bindings, 593 SVGs, zero errors. The frozen executable and all 20 recorded runtime/analyzer source hashes matched. Scoped Git whitespace checks passed. Milestone owner records, README and the live ledger carry the mixed outcome; repository and installed Solid technical references match. No production performance/accuracy/retention behavior was changed, and the failed High run remains failed.

Artifact pruning was applied: zero runs removed; the newest ten per root and 202 older cited runs were retained. All three batch folders cited here remain available.
