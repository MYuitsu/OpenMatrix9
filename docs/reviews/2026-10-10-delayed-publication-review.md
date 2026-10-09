# Delayed native Rebuild publication independent review

Date: 2026-10-10. Baseline/HEAD: `3f5fa93d4e6cdfe2281acce786c317167b0972de`. The implementation and fixture are uncommitted. This review covers Task 2 and its native acceptance evidence in `docs/superpowers/plans/2026-10-10-rhino-core-delayed-publication.md`, against RCORE-01.T08. Task 1's earlier gate review remains in `2026-10-10-delayed-gate-review.md`.

**Status: final for the authored Rebuild T08 publication slice.** A blocking teardown defect was identified in the first native implementation, repaired during review, and validated on the rebuilt module. The final delayed fixture and unchanged spline/source-lifecycle controls record 245 passes across three fresh processes with exit 0. The earlier 144-check run is superseded for final acceptance. Aggregate P0 remains open.

## Scope and method

Read the complete native adapter/header/module registration, portable Rust gate and gate tests, and delayed native fixture. Compared the existing synchronous publication behavior from `git show 3f5fa93:Gui/CoreRebuild.cpp`. Inspected `CoreObjects`, `CurveGeometry`, local FreeCAD document/GUI shutdown and Part shape construction, installed Qt 6.11.2 headers, and bundled Coin child-reference behavior. No production or test files were edited, no build/test process was launched, and no additional reviewers were dispatched by this reviewer. This report is the only authored checkout file.

The review distinguishes ordinary GUI-thread queued delivery from worker execution: the existing solver stays synchronous, and only publication is queued. C++ is used for the necessary FreeCAD/Part/Qt/Coin lifetime adapter; safe Rust owns bounded monotonic claim/cancel/retire policy. No Rust object retains host addresses.

## Strengths

- The result carries owned generated Part shapes, copied document name and `ObjectSnapshot` values, commit/options values, and a separate value-only lease. It carries no document, document-object, source Python-wrapper or view-provider pointer. The existing fitter constructs a new BSplineCurve and `toShape()` result, rather than carrying the selected source wrapper into delivery.
- Publication resolves the document name at delivery, requires the current active/editable document/dialog, validates every copied session/generation/revision/Undo-epoch/UID/unit/ancestor/world-placement guard, and then claims the Rust ticket before adding Coin nodes or opening a transaction. Geometry restoration cannot erase an earlier revision; same-name recreation cannot satisfy the old session.
- Options, cancellation and the next calculation invalidate acceptance. An old/bogus ticket does not consume the latest ticket; a rejected old preview does not clear the newer preview or its deviation label. PointCount, Degree and DeleteInput all connect to invalidation.
- Result destruction releases the vector of owned Python/native-shape references under the GIL and retires its slot once. Construction failure before the shared result exists retires the already-issued ticket explicitly; failures after construction use the result destructor. The bounded gate therefore limits outstanding envelopes to sixteen.
- Preview attachment retains each Coin root; cancellation removes attached children and unreferences roots without using a dead viewer. Commit still resolves the source immediately before copying appearance and uses the existing one-transaction create/remove/recompute path. Both preview and immediate commit pass through the shared validation/claim boundary.
- The deterministic fixture controls the same real generated output owners and production `publish()` path. It compares held/delivered ticket and exact serialized output BRep, owner-count deltas, full document object/property/Undo/Redo inventories, persisted source-shape bytes, placement/group values and actual Coin searches over all 3D view roots. It does not regenerate on release or normalize the source bytes.

## Issues

### Important — resolved

**P1: queued result destructor outlived controller State.** Pre-repair `Gui/CoreRebuild.cpp:82` retained `State& owner`; `:244` queued a functor owning that result with the controller as its QObject context. The controller is parented to `qApp`. Its implicit derived destructor destroyed `unique_ptr<State>` before Qt's QObject destructor removed pending posted events and destroyed their functors. Discarding an undelivered queued result at application teardown could therefore access freed State in the result destructor when updating counters and retiring the gate.

This conclusion follows the inspected C++ destruction order and Qt's implementation: the installed `qtimer.h` creates a callable object; [Qt 6.11.2 `QTimer::singleShotImpl`](https://github.com/qt/qtbase/blob/v6.11.2/src/corelib/kernel/qtimer.cpp) implements a zero-duration shot with a queued meta-call; [Qt 6.11.2 QObject private destruction](https://github.com/qt/qtbase/blob/v6.11.2/src/corelib/kernel/qobject.cpp) removes pending events. The [QObject destructor contract](https://doc.qt.io/qt-6/qobject.html#dtor.QObject) also states that pending posted events are removed.

The repaired implementation (`Gui/CoreRebuild.cpp:71-124`) gives results a shared lease containing only gate/counter values, so retirement never dereferences State. A dedicated child delivery receiver owns queued callbacks. `shutdownDelivery()` cancels the command, nulls/deletes that receiver and drains held fixture results while the interpreter is live; `aboutToQuit` calls it, and the explicit controller destructor repeats it before State destruction. There is no Result/State ownership cycle. Calculation and fixture requeue reject the shut-down receiver. The new fixture queues a real native result and calls this exact production cleanup helper, then settles and checks that no publication callback ran and that owned reference/slot retirement is balanced.

The local host shutdown ordering supports the GIL lifetime: `src/Gui/Application.cpp:2653` creates the stack GUI QApplication before the GUI application; its shutdown guard closes documents before returning from `runApplication()`. QApplication/controller destruction occurs before `src/Main/MainGui.cpp:373` calls `App::Application::destruct()`, which finalizes Python. The final repaired run confirms the same production cleanup helper discards a real queued native envelope without publication: last delivered ticket stays 35, exactly one envelope and shape-reference lease retire, all counters return to zero live ownership, and further requeue rejects. Fresh-process exit 0 exercises the later idempotent application/controller cleanup as well. The source-level dangling-State defect is resolved; no global memory-safety claim follows from this targeted run.

### Critical / additional important / minor

No other blocking defect identified in the inspected scope. No style-only findings raised.

## Exact reviewed fingerprints

SHA256 values were measured from disk. Relative paths are under `H:/FreeCAD-src/Mod/OpenMatrix9`. Repaired source/fixture/gate hashes were measured again after all final immutable runs and remain unchanged.

| File | SHA256 |
| --- | --- |
| Repaired `Gui/CoreRebuild.cpp` | `05d0f861bd0205e857aebce7d514d4a7a852349ff43db5e77e0a46428a1f6166` |
| `Gui/CoreRebuild.h` | `8c633e2cd70cf6f4e6903f6042100c8551b8bd6edd3786d53f057eeb71838325` |
| `Gui/AppOpenMatrix9Gui.cpp` | `62b090f738be9d3b1ed5d820aba8de299f8cf2d37cdbbd19504bb34a62e07282` |
| `rust/src/result_publication.rs` | `b18db2a4b4bb988a287e2df7095490af8e41b09c99397a0025920e2a4ebf951a` |
| `rust/tests/result_publication.rs` | `8d360fead1970e77d5b9f7a38724aeb0b45719f8f98a6f7bb7d3b6f94bca3540` |
| `rust/src/lib.rs` | `efade0d2356022b39d1fd953782bd26e718035d16318be129a57bbf421972288` |
| Repaired `tests/rhino_core_delayed_publication_smoke.FCMacro` | `29d2f8cc618fe1e2e3f38b0d3cc400cbdf64af68963812ad1b408c01a2ff7e0e` |
| Unchanged `tests/curve_spline_smoke.FCMacro` | `b74c0e86b4133aee06854e3f88206e87632dea2b94cf68c55344e8d1326b6c66` |
| Unchanged `tests/core_object_lifecycle_smoke.FCMacro` | `3ea583a03bbd049452ddde14e92da1c699d607e4b4063f785f08c4314004e721` |
| Final `docs/reviews/2026-10-10-delayed-gate-review.md` | `e782f8e5d758e1112566cd543e51a2cf5bc96cf2442d257272717700a419454d` |
| Actual final `H:/FreeCAD-src/build/reusable-history-runtime/bin/OpenMatrix9Gui.pyd` | `e97cad9efd76181853d7ac70af874833f5dadfc47b7063fab809f99e0aecc680` |
| Implementation plan | `30bb03bc564b1e9e5496496843dd5583ab13fd58625536409e78af54c2b07cb5` |
| `Gui/CoreObjects.h` | `3800b6922ad57be6bd4bb2a93a7ba8c034f8ee16a610637bf955a80c496feb03` |
| `Gui/CoreObjects.cpp` | `fedbdfba5593f89d5b4bf2f6148aabd58112e4f2c7bb5cc43e105d3c8acaa98e` |
| `Gui/CurveGeometry.h` | `97b1a31db127615db2796f39eeba90b264a534d24c57720e2cd54caabff2c431` |
| `Gui/CurveGeometry.cpp` | `959cdbb209891cf6860f408ecd69865307742e7cf741b4539e499d7e50a38efc` |
| `H:/FreeCAD-src/src/Gui/Application.cpp` | `9958af29630b00dc8853607a448b7bcc86f5938d0a93f954f4f494a38dbe5629` |
| `H:/FreeCAD-src/src/App/Application.cpp` | `cb936b32ae6eda8772677d2d236aef608fbe435224bca530134f39970c6554af` |
| `H:/FreeCAD-src/src/Main/MainGui.cpp` | `2f133f6c027426352f83dcd075b58423595e4f27214201a3706bce45b8453546` |
| Installed Qt 6.11.2 `QtCore/qtimer.h` | `ec61137676f42e6fd9891ca274d30f35c8e85488b8e38f352a5ecac7c111817b` |
| Installed Qt `QtCore/qobjectdefs_impl.h` | `ed5344aa62679c7695089bf2b41cfe6f5509f6a1acbfd784343b1acb5bb8c768` |
| Installed Qt `QtCore/6.11.2/QtCore/private/qobject_p.h` | `8f5d0b6ce5b4b24610fbfb787cc89c8261eeefc9844d248b83073b5e97466bea` |

## Inspected pre-repair and portable evidence

- Native behavioral RED `build/merge_replay-1/b45e8568555e4c2f8044489430fbefab` solved and held a real result, edited the source, then incorrectly previewed that same late output. The saved initial adapter is `H:/FreeCAD-src/build/p0-delayed-publication-red-CoreRebuild.cpp`, SHA256 `2cc3e338056cbb97073ab3dffa5ffc8d86919cf15de1f53cb155c421987c115b`. RED result/process/module-manifest SHA256 respectively: `51048e28a6c14f71b2f898e6e09194ae201285f14467d7ae3a0193f98d39d601`, `8453c0cbe21466b5e19fcc0d0fa59aa00077986830c0a63d59ac018c4fb01af6`, `30f2484b44dbaddfb0cd15da38c16c607a28a19d8e6f4523b3712d39f8c689a2`.
- The pre-teardown-repair full run `build/merge_replay-1/7b115bb38e704fa4be3825c975c22eb4` records 144 passes, zero failures, process 50896 exit 0, sixteen case records, and balanced 32 envelope/32 native-shape-reference leases with zero previews/held owners/retire failures. Its result/process/module-manifest SHA256 respectively: `fe085641130d35395cfab7a6ee38be8230e6d582566c83333c48d84c8395ab37`, `06d7b7d85797f27ecf60c5e63ebe29912bccd4ff1f9e1b0723e8423b0697c0b3`, `3b0c4a59019c1a0a2ee1fbb8152824b9f022bb7595540250e7cc386ec9bced95`. Manifest macro SHA256 `fe14e02e5ca1662196670ca50ccff0c1d054086a50a7287eadf65877a19a4b64`; measured module SHA256 `c73f6f12a721c3e238a5c0ebf7476e7d6d17bc659f3f89b16e41ca679ac77f38`. This run does not validate the later teardown repair.
- Persisted Rust test output independently sums to 548 passes and zero failures. `H:/FreeCAD-src/build/p0-delayed-rust-results.json` records test exit 0 and Clippy exit 0. JSON/test-log/Clippy-log SHA256 respectively: `a38c60799b3f77124698c51b0943d4b3439d33a69d29e2693de4da85824a0c49`, `2cd09d896da4eaf96bbf2318fa508eba341335940268ffd7e58cdf519c900164`, `a356bd67871a2aaee04b4da013be8cd55ebea2d2e85531d8ef5553e7d33aedc8`. Clippy success is an exit-status statement; its log contains existing warnings and is not warning-free.

## Final native evidence

All three manifests identify the final module hash above, and their macro hashes match independently measured current files. The delayed result itself records the exact selected native module path. All three process records name the single shared host `H:/FreeCAD-src/build/relWithDebInfo/bin/FreeCAD.exe`, completed waits and exit 0. The reviewer inspected existing immutable artifacts instead of launching duplicate runs.

| Run under `build/merge_replay-1/` | Checks | Process | `results.json` SHA256 | `process.json` SHA256 | `merge-native-module.json` SHA256 |
| --- | --- | --- | --- | --- | --- |
| `e0e0c06454aa41e5a9bf5594268afc24` — delayed publication | 173 / 173 | 42320, exit 0 | `1aeb4b2a4bf9d1ed99d143c406a6c8be76d7ca94752a3050fc3fd1f6e238d40a` | `95ba36c1c9d577c8c1a4a26166b0ac7b98b1d87027b1ec025d060e90ed1f6871` | `3c41a86b52f3e598a69c13c46ffb9981d1ea852bc7b9bd3336e6bd0caa038cac` |
| `1bc3ce3c87aa4ed6af70ff4a6119b4d5` — spline/commit control | 41 / 41 | 9876, exit 0 | `83720ad0edc6b21e524345b75142ee9ae0f02104531f0c2350e742d82c0df796` | `dcdc8e05b48c56cbb9e2038b91433693935e08dfea1b57068b67cce2fe916c3f` | `41790ec3b35eafe326c7f2a85397d6f98e86a99641d1ccf56a00f17260dd9287` |
| `054d5589d4c944dfb4e6401c5c652195` — source-lifecycle control | 31 / 31 | 74368, exit 0 | `b828aef43f62c3e22397ea596c203e9f69c163aede6f7b13d67cc1cc0b53d560` | `001a958e69213470f6b7e3f0f8cc1af83eb6694d0b8410068e168ee8e9542211` | `cc7734b3358cd5af0a81482cffc121b44305df565600a3527f914205e7182908` |

The delayed run contains twenty named case groups: unchanged, source edit, edit/restore, same-name source replacement, Undo epoch, unit context, PointCount, Degree, DeleteInput, cancel, parent placement, newer overtaking older, document switch, missing closed document, close/same-name recreation, fresh session, default queued delivery, capacity/recovery and shutdown discard. The final state records 36 created/released envelopes and 36 created/released owned shape-reference leases, zero live/held envelopes, zero preview nodes and zero retire failures.

Independent JSON comparison confirms sixteen direct held/delivered cases preserve the exact ticket/BRep and do not increment shape creation on release. The newer/older pair preserves its respective held tickets and exact BReps; their output bytes are distinct, and late older rejection preserves the four attached newer preview nodes. Cases with a before/after delivery inventory retain exact object/property/Undo/Redo/shape/placement/group values. The missing-document case verifies absence rather than attempting to read a destroyed document. The default queued case initially has one live envelope and no attachment, then automatically publishes/releases it without a fixture release action.

The final delayed directory has 32 raw `.brep` files, each 675 bytes with SHA256 `015b0abc2a03bf58f9dc7f23a01a7d71714e65510a720b9446b581ef6c2e02e6`. Fifteen on-disk held/delivered pairs match exactly. Their prefixes are `cancel`, `capacity-recovered`, `close-reopen`, `degree-option`, `delete-input-option`, `edit-restore`, `fresh-session`, `options`, `parent-placement`, `source-edit`, `source-replace`, `switch`, `unchanged`, `undo-epoch` and `unit-context`. The additional files are `close-missing-held.brep` and `older-held.brep`; their delivered values are bound by the exact comparisons inside the hashed result JSON. No raw byte normalization was used.

The unchanged spline control covers ordinary queued preview, cancellation, immediate commit with source preservation, output degree/count/endpoints, Undo/Redo, atomic batch DeleteInput, periodic output and world-placement behavior. The unchanged source-lifecycle control checks shared snapshot identity/revision/session/parent/context guards and real command stale-input rejection. Together these controls preserve the existing publication/commit behavior beyond the new deterministic hook.

## Recommendations

Retain the blocking finding and repair history, final immutable artifacts and their fingerprints in the validation report/package. Keep prior partial and instrumentation-failure runs excluded and explain the inventory/active-view instrumentation corrections. Record acceptance for the authored Rebuild RCORE-01.T08 fixture only, and keep remaining P0 and background-worker coverage open.

## Declined to judge

- Actual background-worker cancellation/concurrency: solver execution remains synchronous by design; no worker was introduced.
- Global memory safety, ASAN/Valgrind, every OCCT/cache allocation or all native/FFI lifetimes: the counters establish owned Python/native-shape reference leases and ticket retirement for this path only.
- All command/representation families, durable subelement remapping, remaining RCORE-01/P0 requirements: this is the authored Rebuild T08 publication slice.
- Rhino numerical/command parity beyond the existing fitter and its unchanged native controls: delayed publication does not change solver semantics.
- Final package/ledger completeness: packaging and progress-ledger work belong to the executor; this review independently validates the source and supplied run artifacts.

## Assessment

**Ready to merge: yes for the reviewed authored Rebuild publication change.** The material teardown ownership defect is repaired, and the final source/module/macro fingerprints match the inspected successful artifacts: 548 Rust tests, Clippy exit 0 and 245 host checks across three processes with exit 0. The authored Rebuild RCORE-01.T08 delayed-publication fixture has adequate acceptance evidence in this scope; this review does not close aggregate P0 or certify background workers, every native allocation, or global memory safety.
