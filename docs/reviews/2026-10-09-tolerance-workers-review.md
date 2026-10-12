# Effective-tolerance and worker-policy increments: final read-only review

## Final followup result

**No remaining actionable defect found in the reviewed increments.** The prior surface-kernel-floor finding is resolved by `Gui/EditGeometry.cpp:88–91`: the adapter reads `sewing.Tolerance()` immediately after construction and rejects a changed cutoff before adding faces or calling `Perform()`. The saved red run reproduces the old silent relaxation; current green runs exercise refusal without document/Undo mutation. The withdrawn `Standard_Failure` suspicion remains withdrawn.

This followup extends the original scope below to `rust/src/worker_budget.rs`, its five tests, `Gui/ThreeDmImportJobs.{h,cpp}`, both affected CMake files, and the extended worker macro. Only this report was changed. No build, GUI launch, or test execution was performed by this reviewer; saved parent-produced results were independently read and their module/macro hashes checked.

### Final evidence checked

All run folders below are under `H:/FreeCAD-src/Mod/OpenMatrix9/build/merge_replay-1/`. Each accepted `results.json` reports `ok=true` with no failed checks. For all eight accepted runs, `merge-native-module.json` matches both the current fixture bytes and the current `H:/FreeCAD-src/build/reusable-history-runtime/bin/OpenMatrix9Gui.pyd` SHA-256 `ea12455c1a6d4065dc02d29c5923a8148818b51e051b989bcc75037028f40e1d`.

| Evidence | Run | Checked result |
|---|---|---|
| Tolerance final replay | `dde13a0e4de2408f88a4e0da1aef7713` | 29 passed: curve/surface document rules, explicit surface cutoff, legacy surface default/rejection, kernel-floor refusal, precision preference changes, context change, numerical floor, Undo/Redo. |
| Tolerance cold reload | `4e7d7c12fc30466c8b3c4fa4a4974243` | 4 passed: matching module and preserved report/context/joined shell. |
| Worker runtime | `891ce9994810495686b5cc27a40ded01` | 39 passed, including actual process partitions, serial equivalence, invalid overrides before mutation, failure cleanup and exited processes. |
| Edit commands | `6f89e136f98249c8af312619954c2d70` | 115 passed. |
| Edit options | `23daded1d0b64807ac6055f427677a9d` | 57 passed. |
| History | `8adfbb8717774dbc8c639512b3ab1ae1` | 83 passed. |
| Surface | `f78d25d73905410da9ec7c905f7d9a86` | 97 passed. |
| Units | `0b0d0c7afd0a418f92b3f0d358b88c24` | 20 passed. |
| Surface-floor RED | `9661eee942ac44fc93804ae47f2ab2ad` | `ok=false`; failed check is specifically “surface kernel cannot silently raise the requested cutoff.” Correctly excluded from accepted evidence. |
| Worker setup failure | `53373d3c5aae4df7a551cc8abf5a52fd` | `ok=false` because the helper was absent from the SDK application directory. Correctly excluded; parent then staged the helper at the actual lookup location and reran. |

The worker result records 24 logical CPUs, default budget 14, source size 15,489 bytes, and successful above-four exercise. Override 1 creates no request files because it uses serial host conversion; override 4 creates four nonempty partitions per group. Override 64 and absent override each create 11 geometry partitions and eight definition partitions, exactly capped by independent roots. Each source identity is assigned once. The two groups are processed sequentially; these are not 19 simultaneous workers.

The parent separately reports a complete Rust-suite pass and successful standalone native geometry linking. Those two claims were not independently re-executed or traced to their final logs in this review. The worker implementation record reports five focused Rust tests passing. The accepted native results above provide actual ABI/runtime evidence in addition to source inspection.

### Final worker-policy and linkage assessment

- `default_worker_count` uses `(N/5)*3 + ((N%5)*3)/5`, then at least one. Its intermediate values remain within `u32`; the public native path accepts signed Qt counts and clamps unknown/nonpositive counts to one. This is the exact `floor(0.60*N)` policy in RCORE-10 lines 157–159, with no fixed four-worker cap.
- `import_worker_count` validates an explicit ceiling in `1..=i32::MAX` before the legacy serial fallbacks; nonnumeric/overflowed Qt parsing supplies the rejection sentinel. Successful counts are capped by default, ceiling, and independent tasks before conversion. Empty archives deliberately select the existing serial converter. The `<8 roots` and `>16 MiB` fallbacks are preserved and documented.
- The new worker ABI takes only signed/unsigned fixed-width integer scalars and a native Boolean, returns `u32`, allocates nothing, and retains no pointer or native object. Its arithmetic and branches contain no panic-capable allocation/indexing path. C++ casts only a successful count already bounded by signed native-int range. No new ownership or unwind problem was identified.
- `Gui/ThreeDmImportJobs.cpp` changes count selection and configuration rejection only. Existing process creation, grouping, ordered result validation, timeout, and RAII cleanup remain intact. The final macro now locates the helper using `QCoreApplication::applicationDirPath()`, matching production lookup rather than assuming the isolated module directory equals the application directory.
- `Gui/CMakeLists.txt:115–118` publicly attaches `OpenMatrix9RustLib` and Windows Rust system libraries to `OM9ThreeDm`; its imported Rust target already depends on the Cargo custom target. `tests/native/CMakeLists.txt:180–183` does the equivalent with `OM9NativeRust`. Both CMake entry points including `OpenNURBS.cmake` were checked. Static consumers now receive the Rust dependency transitively; no unresolved dependency defect was found. The new tolerance sewing symbol is already supplied by the `TKTopAlgo` dependency of `OM9ThreeDm`.
- The original preview/commit, context ownership, report buffer, and supported direct HistoryJoin assessments below remain applicable. Native exceptions are contained by this installed SDK's `std::exception` inheritance; the earlier contrary suspicion is not a finding.

### Scope that remains open outside these increments

This does **not** complete all P0, RCORE-02, or RCORE-10. The worker-count policy is not measured RAM budgeting, a global scheduler, cancellation, native job-generation ownership, or late-result rejection. Those explicit contracts remain required. The `<8`/`>16 MiB` fallback is an import heuristic, not RAM certification. Tolerance coverage here is curve/surface Join, not a shared policy migrated to every solver or History family. The precision fixture changes 1 and 8 display decimals and verifies Join; it does not close the spec's Boolean precision contract. The large-coordinate Join refusal does not substitute for the spec's Refit fixture. Broader contracts and every numbered fixture remain governed by the full-P0 audit and acceptance matrix.

### Reviewed source snapshot

| File | SHA-256 |
|---|---|
| `rust/src/tolerances.rs` | `f3fa758c1aa511d5de0701fa7dbbed351c4f7255d08b9565e566fc2d4399f9b1` |
| `rust/src/worker_budget.rs` | `ca52b3994b700ccda0ae2dfe34107e6e80ea20a668739d40e8693325c71d20a2` |
| `Gui/EditGeometry.cpp` | `fa9767a9d6de99156fbf8891e93c5f140c8eba7b72450824840fb170baab9452` |
| `Gui/ThreeDmImportJobs.cpp` | `ee47e8f7bcc47ca71e9bec3edfb2f074ed6f076b1290afae21f13e9232f32f0f` |
| `Gui/CMakeLists.txt` | `349c43f82fdad01e3f69e55fecac04a19fc5bd1443a9fa307f509f92c0cda20c` |
| `tests/native/CMakeLists.txt` | `376ed54b2876ebc927be1cb4bd493c9d72ee0301ef9e91bdde3bac440637aa0d` |
| Tolerance macro | `b7e0dade6aeab5944260a6e807a4433ce65c265f9f88c603c2467d30abf0f47a` |
| Worker macro | `17db586a200a94ae9ec6eb7d68071ac4df13739a82619084061e323236fa2791` |

## Earlier review record — superseded status

The following preserves the original review trail. Its one finding and its Join-native verification gaps were resolved by the final followup above; they are not current open items. It predates the guard and final macro additions.

Reviewed 2026-10-09 in `H:/FreeCAD-src/Mod/OpenMatrix9`, while the parent continued implementation. Scope: `rust/src/tolerances.rs`, `rust/tests/tolerances.rs`, `Gui/CoreTolerance.{h,cpp}`, `Gui/EditController.{h,cpp}`, `Gui/EditGeometry.{h,cpp}`, `tests/rhino_core_tolerance_smoke.FCMacro`, the effective-tolerance plan, and direct `buildEdit` callers. No production edits, build, GUI run, or test execution by this reviewer. Parent reported the old native reproduction failing and four Rust tests passing; those are attributed evidence, not runs independently verified here.

### Original actionable finding — now resolved

| Priority | Location | Problem | Required correction / verification |
|---|---|---|---|
| P1 | `Gui/EditGeometry.cpp:88` | Surface Join passes the resolved cutoff into `BRepBuilderAPI_Sewing` but never checks the cutoff accepted by that backend. The resolver permits values down to `1e-9` mm. OCCT's upstream `Init` clamps working tolerance to at least `Precision::Confusion()`, and the installed 8.0.1 headers define that constant as `1e-7`. A request below this floor can therefore produce geometry under a relaxed backend tolerance while preview/provenance say the original value was used. | Immediately inspect `sewing.Tolerance()` after construction and fail clearly if the backend cannot honor the resolved cutoff; do not silently report the smaller request as effective. Exercise an explicit `1e-9` surface request and check rejection without output/Undo changes. Verify ordinary document `2*tau` and explicit surface overrides still work. |

Evidence: current native constructor is at line 88, with face additions immediately following; Rust range/floor logic is at `rust/src/tolerances.rs:73–89`. The installed `.pixi/envs/default/Library/include/opencascade/Precision.hxx:165` returns `1.e-7`; installed `BRepBuilderAPI_Sewing.hxx` exposes `Tolerance()`. Official [OCCT source](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/master/src/ModelingAlgorithms/TKTopAlgo/BRepBuilderAPI/BRepBuilderAPI_Sewing.cxx) shows `Init` selecting the greater of the requested tolerance and `Precision::Confusion()`. That fetched source is upstream master, not a successfully retrieved 8.0.1 tag; querying the actual constructed native object is the decisive version-specific check and makes the contract robust to backend version changes.

The parent has already received this finding. At the last read, `rhino_core_tolerance_smoke.FCMacro:92–95` contained a new kernel-floor fixture, but the corresponding native guard was not yet present. This report describes that observed snapshot, not later edits.

### Original reviewed contracts with no additional defect established

- **Policy and precedence:** explicit override wins; explicit document context follows; legacy curve/surface defaults remain `1e-7`/`1e-6` mm. Only document-derived surface tolerance receives the factor of two. Relative and angular tolerance are validated as part of context but are not used in this rule. Invalid document context remains an error even when an override is present, consistent with fail-closed context ownership.
- **Numerical policy:** all inputs and multiplication results are checked; finite nonnegative world-coordinate bounds are required. The documented `32*epsilon*max(1 mm, bound)` floor rejects insufficient resolution. The report identifies this as the OM9 numerical floor, not a certified kernel error estimate. The surface backend floor above is a separate constraint.
- **ABI and lifetime:** the C layout contains a 64-bit revision, four 32-bit fields, and three doubles; C++ checks 48-byte size. Native snapshots own both context and report strings. The resolver does not retain incoming string pointers. Rust validates null/encoding/context and leaves output untouched on failure. Report generation supports size query, refuses a short buffer without writing, and documents that its input must be an unmodified successful result. The latest Rust test revision includes report-size/short-buffer coverage.
- **Preview/commit:** the controller captures context at session start, resolves and revalidates it for preview, and resolves again before commit. `commitEdit` rechecks current context before transaction. Both builds receive the same resolved effective value when state is unchanged. Existing input verification and owned geometry snapshots remain in use. The latest controller status displays effective Join cutoff and source, which helps explain stale spinbox values after a typed override or a surface document rule.
- **Transaction/provenance:** output `JoinTolerance`, `OM9ToleranceReport`, and explicit `OM9UnitContext` provenance are written inside the same existing edit transaction. Exceptions abort that transaction. Legacy documents intentionally have no explicit context payload; their report identifies the legacy source.
- **Legacy direct callers:** repository searches found the controller, internal build recursion, and `HistoryFeature.cpp` callers. Supported HistoryJoin creation is curve-only and supplies its explicit saved tolerance. Its path still reaches the unchanged curve implementation; this surface-sewing rewrite does not establish a regression in that supported caller. Applying the shared context resolver to History is separate scope, not evidence that all-P0 tolerance work is complete.
- **Sewing topology:** the surface branch still rejects results that are not one shell, lose faces, or fail the common validity check. Existing area/count smoke assertions help detect representation damage; they do not prove every trim/hole/continuity contract.
- **Exception handling:** an initial suspicion about uncaught `Standard_Failure` was withdrawn after inspecting this actual SDK. `.pixi/envs/default/Library/include/opencascade/Standard_Failure.hxx:28` derives it from `std::exception`; existing catches cover that inheritance. No change should be made based on the withdrawn suspicion.

### Original pending verification — superseded by final evidence above

| Contract | Current evidence in inspected changes | Remaining evidence |
|---|---|---|
| Rust resolution/ABI | Four focused tests exist; parent reports all four passing. Latest source adds report buffer checks. | Confirm the latest source revision was the one executed after any concurrent edits. |
| Curve document cutoff and override | Native smoke covers `.005` pass at `.01`, `.015` reject, explicit `.02` recovery, legacy reject, failed commit no mutation, Undo/Redo. | Run against the newly built native module; the parent's old-module reproduction proves the prior defect, not the fix. |
| Surface document cutoff | Smoke covers `.015` pass and `.03` reject at document `.01`, output shell/face count/area, report `.01` requested and `.02` effective. | Run new binary and cold restore. |
| Surface backend floor | A new native `1e-9` override rejection fixture exists. | Add the native guard and run it against the installed OCCT; assert clear refusal rather than output under a relaxed cutoff. |
| Explicit surface override and legacy surface threshold | Rust verifies policy; native surface branch is new. | Add/run bounded native fixtures proving overrides are not doubled and old-document `1e-6` behavior is preserved. |
| Display precision invariance | Resolver takes no precision input; Rust test name states precision independence. | The test does not actually change the application's display precision. Add/run the plan's native precision-setting invariant and restore the preference afterward. |
| Context change and large coordinates | Smoke covers changed document context rejecting commit and `1e12` coordinates rejecting `.001`. | Execute and check unchanged source geometry/Undo for each rejected operation. |
| Save/reopen | Smoke persists surface report/context and checks cold restore shape/area/report. | Run both sessions against the new binary and preserve module/source identities in results. |

The detailed plan remains an incremental Join dependency. Passing these tests would not close the broader P0 operation-wide tolerance registry, generic topology/ownership, preview-generation, or all solver contracts described in `p0-full-kernel-audit.md`.
