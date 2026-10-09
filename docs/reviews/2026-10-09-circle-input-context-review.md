# Circle accepted-input context review

Independent review on 2026-10-09 in `H:/FreeCAD-src/Mod/OpenMatrix9`, baseline `349203d`. Scope: Circle context/checkpoint changes in `rust/src/curve.rs`, `rust/src/curve_ffi.rs`, `rust/tests/circle_input_context.rs`, the two intentional `circle_advanced.rs` adaptations, and compatibility with `Gui/CurveCircle.{cpp,h}`, `Gui/CircleHistoryAdapter.{cpp,h}`, `Gui/RustBridge.h`, `Gui/CurveController.cpp`, and `tests/circle_input_context_smoke.FCMacro`. Read-only except this report and the disposition update in `2026-10-09-extended-routing-review.md`; no production/test source edits, Git changes, builds, UI or native runs by this reviewer.

## Initial freeze findings

### P2 — Undo restores selected native path after the adapter discarded it

`curve.rs` now restores every accepted option from a Rust checkpoint. `Gui/CurveCircle.cpp:87` only prunes `path` from current mode/count; it cannot reconstruct discarded reference identity when Undo restores an earlier option state.

Source-backed native reproduction:

1. Start Circle and select `AroundCurve`.
2. Select native path A: Rust sets `path_selected = true`; C++ stores `path = A`; reference mode becomes 2.
3. Enter `Center`. Mode changes are accepted while the construction-point vector is empty. Rust resets Circle mode/options. `circleNativeResult` clears C++ `path` because the construction is no longer AroundCurve.
4. Enter `Undo`. Rust restores AroundCurve with `path_selected = true` and reference mode 2. The discarded native `path` remains empty.
5. `OnCurve=0.5` throws `Reselect the path` although accepted-input Undo claims to have restored its selection context.

The old geometry-oriented Undo did not restore this earlier mode/selection state; this incompatibility is introduced by the broader accepted-input checkpoint contract. Restore the native resource context alongside the corresponding Rust checkpoint, retaining identity validation and bounded ownership.

### P2 — Undo restores Radius/Orientation values without their native sources

`Gui/CircleHistoryAdapter.cpp:108` erases role -1 Direction or role 4 Radius sources after an accepted Orientation/Radius option. Replacement native probes also erase the prior role at line 90. The new Rust Undo restores the previous normal/radius/previous point, but source-count/config pruning cannot recover erased source records.

Reproduction for Direction: accept a Circle center, `Orientation`, and native `Direction=DirectionA.Vertex1`; enter `Orientation` again; then `Undo`. The old normal/oriented state is restored, but DirectionA has been removed from `sources`. Commit with History enabled, then move DirectionA: its dependency is absent from the recipe, so History cannot replay that accepted native direction. The Radius equivalent uses a ThreePoint chord, accepted native `Point=RadiusA.Vertex1` in the Radius phase, another Radius option, then Undo. Verification can also miss a changed/deleted restored source because it is no longer in the native ledger.

The parent accepted both findings and added opaque Rust context identifiers plus bounded native resource snapshots. Source re-review and recorded native evidence are below; the first SDK build was explicitly superseded and is not final validation.

## Rust source checks at the first freeze

- `CircleInputCheckpoint` owns construction points, `Circle` options, frame, unit scale, accepted previous point and pending option. It contains no session/history field and no native object pointer. The snapshot graph is not recursive.
- `VecDeque<Arc<CircleInputCheckpoint>>` holds at most 1,024 checkpoints. Each snapshot owns at most 1,024 fit points. Preview/session clones share immutable checkpoint allocations through `Arc`; they do not clone nested histories or form reference cycles. Popping/evicting releases owned vectors when the last reference drops. Worst-case point storage is bounded at approximately 24 MiB, plus bounded options/strings and container overhead.
- Typed input, actual native/mouse point, native reference and fit batch wrappers create one checkpoint on accepted active input. Inner input recursion and typed-point delegation bypass the outer wrappers, avoiding duplicate checkpoints. Errors restore checkpoint fields and do not append history.
- Radius probes and orientation direction points update `circle_previous` without contaminating solver construction points. Native path selection is an accepted option but does not invent a point base; actual selected center/constraint points and fit batches update the base.
- Undo restores the accepted previous point and captured frame/unit/options/pending value. It removes one whole fit batch, and options are distinct accepted input checkpoints. The two advanced-test adaptations match that approved RCORE-04 contract instead of weakening geometry assertions.
- A ready Tangent blank Enter requests native solve without adding a no-op checkpoint. Other failed/incomplete inputs preserve accepted state. Commit/cancel/start clear input checkpoints; successful commit retains solver points/options needed by History snapshotting.
- `circle_finish` computes/validates its plan/output before final session assignments; ordinary failing paths do not partially commit active/output/plan state. Native tangent solution validation similarly returns before final mutations on failure.
- Existing native pruning handles removal of construction points and whole fit batches, but does not prove restoration of identity after accepted options discard resources; that is the finding above.

## Executed evidence

Existing, already built binaries were run independently without rebuilding:

- `circle_input_context-0abb3b9b1609c3c7.exe`: **15/15 pass**.
- `circle_advanced-cf7e1d099cd5084e.exe`: **11/11 pass**.
- `circle_history-09c448885f5155c0.exe`: **16/16 pass**.

The new regression reproduces the original relative Radius-probe center analytically and covers typed/native/mouse direction bases, Undo of Radius values/options/probes, pending option recursion/error atomicity, frame/unit restoration, selected path state, whole batch Undo, preview independence, checkpoint budget and ready-Tangent confirmation.

These Rust suites cannot establish native source-map restoration. Parent-reported full Rust 419 tests at the first freeze, and 421 tests plus Clippy with no errors at the repaired freeze, are not independent executions by this reviewer. All broader non-Circle session/context/identity and live constraint/locale/Solid-policy gaps remain open.

## Exact first-freeze source hashes

SHA-256, independently read before native repairs:

| File | SHA-256 |
|---|---|
| `rust/src/curve.rs` | `349d18c6e8062f05e8973c7006d7dcd136dbfe94e63889e49a1703d56e538df6` |
| `rust/tests/circle_input_context.rs` | `893d61285228443b0e68d2445a1371b089baddf628713133f3ece9f25e791354` |
| `rust/tests/circle_advanced.rs` | `6964ba2476c8481a9e82953ec4192afd600ba42bd8a3d0c3dddb7ae48b0603c9` |
| `Gui/CurveCircle.cpp` | `161056255a76d19aed938a4e223743825181886378406db064b009cdaccdd86e` |
| `Gui/CircleHistoryAdapter.cpp` | `7ff8dc57d74396adb6db203a57ed8380450ebd1f2b22bf86ceae0c006f129a25` |

## Repair followup

### Native restoration repair: source reviewed, matching restoration checks passed

No further defect established in the frozen repair. Rust owns an opaque current context key and checkpoint keys. Accepted input advances a checked counter; Undo restores the prior current key without rewinding the counter, so later branches cannot reuse an old identity. Errors consume no key. The retention API exposes current plus at most 1,024 checkpoint keys, with null/short-buffer queries leaving caller memory unchanged. Rust stores no native pointer.

The native map uses those retained keys as the authoritative eviction policy, bounded at 1,025 contexts. Contexts own reference/path/source metadata and immutable shared native shape snapshots. The weak geometry cache shares copies between snapshots and removes expired entries. Captured native geometry has no retained Python reference; live Python/native wrappers are reconstructed only on restore. Reference guards include document/object identity, revision/generation/epoch, subelement, units and placement stamps. Restored references, path and source vector are staged before swapping the live globals. Broader existing source-incarnation limitations remain outside this increment.

Input/pick adapters capture the old key before accepted mutation, capture the intermediate path-selection key for composed `Curve=...@...` input, and capture the complete source batch before an optional finish input. Result handling restores a retained context when Undo selects its key, then prunes current resources and captures a new accepted context. Commit verifies the restored guards. Start/cancel release native contexts; session restart is paired with native cleanup, preventing reused per-session keys from naming old resources. Existing geometry solvers are reused.

Independent executions of existing rebuilt binaries after key-policy changes: `circle_input_context-0abb3b9b1609c3c7.exe` **16/16 pass**, and `openmatrix9_rust-908b2f3029ddc2ce.exe context_key_tests --nocapture` **1/1 pass**. These cover branch-key uniqueness and atomic FFI short-buffer behavior. Scoped final `git diff 349203d --check` passed.

The reviewer directly read both native results and their manifests; the parent executed the native fixtures:

| Replay | Result | Fixture SHA-256 | Native module SHA-256 |
|---|---|---|---|
| `build/merge_replay-1/08da657878734ec59bcdcc4d0ba38c89/results.json` | RED: 13 checks, 7 failed; parent reports exit 1 | `d8ed7eab52e932b8fdb7a04be9bdb987d7d9175033ef975c55b8d754054f43bc` | `505603dbaee7dab676c096e09a686ee313ecf2c8ebce8d15f82ee2b25f44d648` |
| `build/merge_replay-1/5fa23d65ac834714ab692fdff2fa4501/results.json` | All original 13 restoration checks passed; expanded 16-check replay has only the 2 F4 failures below; parent reports exit 1 | `0858f9ceeae318b03b4285432f67148255779236f64e33643609e2f1823cd71d` | `b0605e49b982a889f1d41daf5e1ac1878d6479381c46ccda8bd2f2a1a0462377` |
| `build/merge_replay-1/989d485f5dda4c34ab85c8e5305fe6e8/results.json` | Preliminary GREEN: all 16 checks passed; superseded by the fresh rebuild after the test-module move | `0858f9ceeae318b03b4285432f67148255779236f64e33643609e2f1823cd71d` | `2e127f9c85a3abf4bdb235834ad47e2fb20f81b0776e22f1dd4f6de798df4360` |
| `build/merge_replay-1/97245d1aa9a546c69c6a212d761b9b06/results.json` | Final matching GREEN: all 16 checks passed, `ok: true`; parent reports rebuild 96086, exit 0 | `0858f9ceeae318b03b4285432f67148255779236f64e33643609e2f1823cd71d` | `71a762ab77445d52ef113d3a578c65a7c79c177d2e31cdcd50901b2739078e45` |

The original RED directly establishes missing selected-path restoration, missing Direction/Radius source restoration and subsequent association, and failure to reject a restored stale path. The second run establishes their matching repair, including intermediate composed-input context and cancel/restart cleanup.

### P2 — F4 bypasses stale-reference validation: repaired, matching native replay passed

This inherited shortcut calls `om9_curve_point` directly, skipping the native typed/pick adapters. Reproduction: Circle Tangent; accept native curve A; select Point; mutate A; press F4 with no radius. Adding the origin is not solve-ready, so later result handling need not validate references. Rust therefore accepts an origin/previous-base and checkpoint while A is stale. The expanded native replay above records exactly two failed checks: `F4 rejects stale tangent source before accepting origin` and `Rejected F4 adds no accepted-input checkpoint`. Its ordinary F4 Circle check passed.

Frozen `Gui/CurveController.cpp:367` now captures Circle's current native context and verifies references inside the existing F4 exception handler, after updating the input frame but before `om9_curve_point` and `CoreSnaps::acceptedPoint`. A stale-reference exception therefore cannot accept the origin, advance phase/base, or append accepted-input history. Capture preserves the initial/current context for later Undo. This order addresses the concrete bypass without changing non-Circle shortcuts. No further source defect established in this repair. The reviewer directly read the parent's final matching rebuilt 16-check GREEN replay `97245d1aa9a546c69c6a212d761b9b06`, including both F4 regression checks and ordinary F4 Circle acceptance. That run supersedes the preliminary `989d485f5dda4c34ab85c8e5305fe6e8` proof after the final test-only FFI module move and fresh rebuild. This is bounded fixture evidence, not exhaustive runtime parity.

## Exact repaired-freeze source hashes

SHA-256, independently read after the native and F4 source repairs. The final `curve_ffi.rs` hash includes moving its unchanged test module to EOF for Clippy; its exported implementations are unchanged and were reread:

| File | SHA-256 |
|---|---|
| `rust/src/curve.rs` | `ea9d982ab9766d2f50cddb3847207ca540457bf1ea3c4d94bfd6fd4b19ad1b06` |
| `rust/src/curve_ffi.rs` | `2c8dc2eeecb47db6bcacad51964424c29def2a8ca9059bcd324f2318b4612a75` |
| `rust/tests/circle_input_context.rs` | `213e47653b79be573991e74a2806c6d36f7a2b72840b0a484a190e5496a7098e` |
| `rust/tests/circle_advanced.rs` | `6964ba2476c8481a9e82953ec4192afd600ba42bd8a3d0c3dddb7ae48b0603c9` |
| `Gui/CurveCircle.cpp` | `528f634c7e8b10403d8e0972f506c60f100273517d1e437023f0e1755f5daff2` |
| `Gui/CurveCircle.h` | `24674ce7e41c8ea182195ff73faed8d2c04d32673278f8e901149a0d7b704edc` |
| `Gui/CircleHistoryAdapter.cpp` | `91f4a6092ad259d131b11a22602246038f344f81d172d69f29530a6237a445a8` |
| `Gui/CircleHistoryAdapter.h` | `11492d3b84d95710fb5d39fc3f958605ac10e78ccbb8670751ef33dfd9b95736` |
| `Gui/RustBridge.h` | `6d25d0a6fe651ab736d6eac614aa4126e34284e65c78fa91f0d8d9c0d14a96cc` |
| `Gui/CurveController.cpp` | `8f694e21abcd1774dcc7e1486a90b196cdb6492b4916ca93ca7e781cb57b0174` |
| `tests/circle_input_context_smoke.FCMacro` | `0858f9ceeae318b03b4285432f67148255779236f64e33643609e2f1823cd71d` |

All broader non-Circle accepted-input context, common-session reduction, live constraints, locale, complete Solid-unit policy, and remaining P0 gaps stay open. No all-P0 completion or exhaustive Rhino runtime parity claim is made.
