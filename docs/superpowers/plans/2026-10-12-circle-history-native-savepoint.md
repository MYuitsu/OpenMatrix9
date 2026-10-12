# Circle History native savepoint implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: use superpowers:executing-plans inline, with TDD and actual native verification for each deliverable.

**Goal:** Implement the approved native property replay savepoint, preserving caller history, protected metadata/source state and document lifetime during Circle replay and rejection cleanup.

**Architecture:** FreeCAD owns its property journal, checked rollback, object lifetime barrier and staged merge. Rust retains independent request/geometry/result validation; OM9 keeps existing native solvers. Circle uses the scope from before reading replay inputs through verified success or rollback/error cleanup.

**Tech stack:** Existing FreeCAD App/Transaction/Property APIs, C++ native bridge, Rust validation and isolated native FCMacro tests.

**Spec:** [Approved native savepoint design](../specs/2026-10-12-circle-history-native-savepoint-design.md).

## Global constraints

- User approved the design on 2026-10-12: “Đồng ý thiết kế native savepoint”. Continued inline implementation is authorized; no further design permission is pending.
- Keep all original RCORE-01 requirements and Task2/3 open until their complete acceptance audit passes. This plan covers the savepoint contract, not the whole chapter.
- Preserve every original transaction state: no booking/journal, unmaterialized booking, active caller journal; both booking modes and UndoMode0/1.
- The private journal never replaces the active caller pointer or enters visible Undo/Redo before acceptance. Preserve native first-write semantics and detached-object ownership.
- Observer mutation fails before data changes and latches rejection, even when Python swallows its exception. Native setters/restore may proceed without granting emitted callbacks that permission.
- Keep scope lifetime through capture/copy/merge/restore/destruction and paired error cleanup. Refuse close/remove/create before deletion queueing/allocation.
- Reuse `H:/FreeCAD-src/build/relWithDebInfo`; no root reconfiguration, full FreeCAD build, full runtime clone or replacement of binaries in a live user host.
- A necessary FreeCADApp change is authorized because journal isolation and lifetime state are private native fields. Explain that concrete reason before its isolated App build.
- Build OM9 only through `build/build-om9-plugin.ps1` with its existing cache and an explicit matching App import override. Fingerprint loaded App/Gui/plugin/host.
- Preserve all failed fixtures, source snapshots, models and logs. Do not reset/stage/commit unrelated concurrent changes.

## Review focus

1. Caller already changed the same property, or owns a newly created/deleted object: accepted merge must preserve first-write originals and unique native ownership.
2. An observer catches its own rejected mutation, or acts during Property::Copy/Paste: replay still rejects and restores under live protection.
3. Owner close/removal is requested during recompute: reject before queueing/freeing; native recompute must return normally.
4. Empty booking materialization or staged merge allocation fails: preserve exact booking/description/Redo and pending state.
5. UndoMode0, missing/dynamic properties, nested native setter notifications and cleanup failure: preserve supported semantics and report an unverified rollback explicitly.

## Task 1: Freeze current failing native contract

**Files:** existing `tests/rhino_core_circle_history_atomic_semantic_smoke.FCMacro`, `tests/rhino_core_circle_history_close_callback_smoke.FCMacro`; private `H:/FreeCAD-src/build` evidence.

- [ ] Verify the unchanged semantic432 fixture on the current matched Runtime112; retain actual host verdict, callbacks, exact journal/metadata/source observations and source hashes.
- [ ] Match the preserved owner-close witness to the current native App/fixture. A killed/prefix host is RED evidence, never a completed pass.
- [ ] Freeze relevant native source, App/Gui SDK, import-library, host, plugin, fixture and runner fingerprints before native implementation/build.

## Task 2: Implement strongly staged property-only journal merging

**Files:** `H:/FreeCAD-src/src/App/Transactions.h`, `Transactions.cpp`; native test/probe and FCMacro fixtures.

**Interfaces:** add `Transaction::PropertyMerge` as a staged property-only merge owner, `preparePropertyMerge(const Transaction&, size_t maxRecords, size_t maxBytes)` and `commitPropertyMerge(PropertyMerge&) noexcept`. Preparation never alters the destination; commit transfers preallocated entries without callbacks or allocation. The stage retains only newly copied transaction-property records; unchanged caller records are borrowed, never duplicated owners.

- [ ] Add RED native tests for same-property first-write preservation, caller New/Del ownership, preparation failure and Copy callbacks. Verify executable native Undo/Redo and source/child values.
- [ ] Implement per-record staging through TransactionFactory and native PropData copies. Preserve caller status/name/entries; refuse unsupported replay object/rename/move operations. Do not use a temporary Transaction whose destructor could delete caller-owned New objects.
- [ ] Check record traversal and copied-property memory using checked arithmetic before transfer; reuse request/publication limits and retain honest limits of native `getMemSize` accounting. Do not claim a complete RCORE aggregate allocator proof.
- [ ] Watch the tests pass on an isolated matching App; retain failure evidence and verify destination state on every exception path.

## Task 3: Implement the Document scope and barriers

**Files:** `H:/FreeCAD-src/src/App/Document.h`, `Document.cpp`, `private/DocumentP.h`, `Application.cpp`, `TransactionalObject.cpp`, `DocumentObject.cpp`, and property entry points found by before-mutation audit.

**Interfaces:** `App::Document::PropertyReplayTransaction` noncopyable scope with `verify()`, `commit()`, `rollback()` returning a completion verdict and explicit checked error-state completion. Document owns/knows the active scope without changing caller journal ownership.

- [ ] Add RED callbacks covering Document and Application close/remove/create/restore, dynamic property mutation, commit/abort/clearUndo/Undo/Redo, lock/booking and document-unit changes. Include swallowed observer errors and before-change/copy/restore phases.
- [ ] Route allowed replay property writes to the private journal before `_checkTransaction` can materialize a caller booking. Handle dynamic property add/remove; refuse moves/renames/object edits outside the approved scope.
- [ ] Reuse observer-depth discipline; latch attempted prohibited operations. Apply close/lifetime protection at Application and Document entry points, including before pendingRemove queueing and insertion allocation.
- [ ] Roll back through Document's native rollback/defining flags plus `applyChecked`; retain lifetime protection while restoring/freeing private values. Capture errors without certifying failed rollback.
- [ ] Prepare merge, verify, then materialize an existing booking only after acceptance. Protect/check open callbacks; failed materialization restores the original booked descriptor, pending state and native Redo. For no booking/journal, do not invent a user Undo operation.
- [ ] Pass exact state, source and executable history tests in all three caller states, both booking modes and UndoMode0/1.

## Task 4: Select the matching native SDK and integrate Circle

**Files:** `H:/FreeCAD-src/build/prepare-isolated-native-app.py`, `build/build-om9-plugin.ps1`, standalone SDK adapter; OM9 `Gui/CircleHistory.cpp` and `Gui/CircleHistory.h` as required.

- [x] Add a validated App import override to the standalone builder if missing. Build only required FreeCADApp objects into an isolated native output; preserve shared SDK/host and prior App19.
- [x] Start the scope before reading Circle replay metadata/sources, including refreshPlacements. Keep recipe-v2 and legacy solving, independent Rust source/contact/result checks and current native kernel validation.
- [x] Verify publication after all setters, then commit the private changes. On failure, checked rollback precedes protected Shape/result clearing and explicit error-state completion; the destructor must not restore cleared invalid output.
- [x] Run unchanged semantic432 and owner-close fixtures on matching App/plugin. Require complete result and actual normal host exit; no timeout/prefix substitute. Runtime131/App125:432/432 and4/4, clean normal exits; modes109/109 and Curve33/33 also pass. Evidence: `docs/validation/2026-10-12-rhino-core-circle-native-savepoint.json`.

## Task 5: Verify positive behavior and regressions

**Files:** existing Circle/Curve macros plus new native savepoint-specific fixtures in `tests/`.

- [ ] Positive replay for all three caller states, regular/preserving-Redo bookings, UndoMode0/1, same-property prior edits, object ownership, commit/abort and actual Undo/Redo.
- [ ] Source/metadata/output/unit, before-change, Copy/Paste/restore and merge callbacks; exact rejection-time state, paired invalid output and no silent overwrite.
- [ ] Matching Circle result, all-mode/legacy/cold/source/spatial/contact cohorts and affected Curve/publication regressions. Independent cold hosts must inspect actual FCStd bytes and native geometry.
- [ ] Preserve raw offset D3 discrepancy as RED; do not change oracle/solver to mask it. No unrelated broad retests without changed contracts or unresolved evidence.

## Task 6: Record evidence without narrowing the original goal

**Files:** Spec v1 RCORE-01/Circle spec/checklists, FEATURES JSON/YAML, MANIFEST, `docs/openmatrix9-progress.json`, README and validation report.

- [ ] Record exact source/build/loaded module fingerprints, every actual host exit and failure/timeout, native state observations and measurable resource bounds.
- [ ] Update synchronized views and catalog evidence only for the verified scope; run source audit and manifest/catalog consistency verification.
- [ ] Retain all open original adapters1–7/C01–C12/T01–T11, consumers, generic input, conversion/maps/deviation, families, sharing/budgets, persistence and lifecycle obligations. Savepoint success alone cannot complete RCORE-01.

## Persistence inverse checkpoint (2026-10-12)

- App154 prepares bounded inline native persistence buffers, seals the prepared merge source, retires all private Property instances and verifies before caller materialization/transfer. The inverse uses FreeCAD Save/Restore, not JSON compensation; its buffers contain no Property destructor.
- The unchanged final recovery disposal matrix becomes204/204 across32 callback cases on matching Runtime156/App154. Registered original retirement204/204, native inverse/dynamic-property/budget/sealed-source tests40/40, semantic432 and named regression/cold cohorts pass. This is partial evidence, not complete type/cache/allocator coverage.
- A new regular discarded-Redo matrix is **RED12/28** across4 callbacks. Attempts are blocked and replay does not report accepted, but prior Redo is physically disposed after caller transfer; exact booking/history and paired cleanup fail. Preserve original Redo ownership through rejection and all caller states; do not weaken or defer away the required callback witness.
- Complete unsupported/nonpersistent/opaque property inverse coverage, dynamic links and native identity/status/cache behavior, Save/Restore/factory failure paths, aggregate preflight and global participants. Private inverse progress does not certify these requirements.

## Retirement acceptance checkpoint (2026-10-12, earlier evidence)

- App147 prepares an independent native factory/Paste recovery journal, retires original Copy objects and verifies the observer latch before caller merge. Original retirement RED128/204 becomes204/204; a registered native property subtype also passes204/204. These cover32 cases each, all caller states and actual UndoMode0/1.
- The final recovery-journal disposal matrix remains **RED128/204**:32 attempts are blocked and no replay reports accepted, but rollback/paired cleanup do not complete. Returning an explicit unverified KernelFailure is honest failure reporting, not savepoint completion.
- The next implementation must retain a usable inverse through every callback-bearing disposal and preserve/revert the exact caller stage on rejection. A callback-free restoration payload, reversible staging or another mechanism must pass the complete late-disposal matrix; do not retire a caller state or weaken the witness. Include discarded regular Redo ownership, native status/cache flags and global participants in acceptance.
- Current matching App147/Runtime149 semantic432,close4,modes109,Curve33,booking80,capture34 and scope744 pass. Named cold/legacy/spatial verification is recorded separately. Full native lifecycle/resource coverage and all original RCORE01 obligations remain open.

Self-review: each approved design section maps to Tasks1–6; Review Focus cases are included in native gates. Circle integration, strong capture, named booking policy and original property retirement have native evidence. Final recovery disposal, other callbacks, participants, status/cache, budgets and all startup paths remain unverified. Tasks2/3 and the full chapter remain in progress.

Ruling: the human approval and persistent instruction to complete RCORE-01 authorize inline execution of this approved design; do not repeat the already answered permission request.


Native value snapshot checkpoint — 2026-10-12T10:44:33+07:00

App191/Runtime193 giữ identity native cho Link/XLink, subname/shadow/partial flags và cây biểu thức/validator; dùng lại setters/Paste của FreeCAD. Frozen journal giữ thao tác rename và refresh các entry mới/canceled. Prepared phase và cleanup guard chặn callback trong merge unwind; commit retry bị từ chối trước null journal.
Đã kiểm chứng **2607/2607 assertion qua21 host**, gồm Link128, XLink112, expression60, rename32, merge-unwind32, retry12, native journal51 và hồi quy/cold reload.
**Còn RED:** detached-object disposal16/32 và Rust121pass/1fail (raw native offset D3). Python property identity/orphan Save, frozen-source preparation và toàn bộ **Task2/3/full RCORE-01 còn mở**.

- Task2/3 remain in_progress. Reproduce native PropertyPythonObject/Proxy inverse identity and orphan Save failure, implement protected native retention, then fix frozen-source preparation and detached object retirement while continuing all original RCORE01 requirements.
- Evidence: H:/FreeCAD-src/Mod/OpenMatrix9/docs/validation/2026-10-12-rhino-core-native-value-snapshots.json


Checked native retirement verdict — 2026-10-12T12:07:51+07:00

App227/Runtime229 dùng lại transaction, native Property và Python exception của FreeCAD. Finalizer chạy trước verdict của commit/cleanup; callback bị chặn không còn báo thành công. Giữ inverse nếu retirement trước đó đã bị từ chối.
**3145/3145 assertion qua28 host** trên runtime tương ứng, gồm Python identity/private inverse72/201, finalizer6, frozen source180, native journal54, verdict50, cleanup14 và inverse retention12.
**RCORE-01/Task2/3 vẫn đang triển khai.** Test gốc Python final release 5/8 và detached object 16/32 còn RED; Rust121 đạt/1 lỗi OffsetBezier D3. Các checkpoint phía dưới là bằng chứng lịch sử.

Task2/3 remain in_progress. Evidence: H:/FreeCAD-src/Mod/OpenMatrix9/docs/validation/2026-10-12-rhino-core-native-retirement-verdict.json
Next: Resolve native final-disposal recovery without weakening the original identity witnesses; investigate raw native OffsetBezier D3, aggregate admission/global participants, then continue every original RCORE01 adapter and C01-C12/T01-T11.


Approved acceptance / offset D3 — 2026-10-12T13:22:01+07:00

Theo thiết kế người dùng duyệt, identity/Redo được giữ khi từ chối trước acceptance; destructor chạy sau acceptance dưới barrier và báo disposal diagnostic riêng. Circle History giữ publication hợp lệ khi logger ném lỗi.
**App247/Runtime254/kernel240:3313/3313 kiểm tra qua28 host**; boundary162, logger24. OCCT D3 helper/direct curve/adaptor RED4/12→GREEN12/12; đủ19 họ/183 query,36 D3 được sửa và45 offset khớp đạo hàm độc lập. **Rust toàn bộ1037 đạt/0 lỗi.**
**RCORE-01/Task2/3 vẫn đang triển khai:** aggregate budgets, global participants, lifecycle và toàn bộ hợp đồng gốc còn mở. Witness destructor-veto cũ giữ nguyên, đã được thay tiêu chí bởi thiết kế duyệt; các checkpoint dưới là lịch sử.

Evidence: H:/FreeCAD-src/Mod/OpenMatrix9/docs/validation/2026-10-12-rhino-core-acceptance-offset-d3.json
Next: Add failing constructor/history/name aggregate-budget tests and bounded native preflight; audit global/source participants and normal-close finalizer lifetime; continue all original RCORE01 services/adapters/C01-C12/T01-T11.


Snapshot/history disposal — 2026-10-12T14:02:42+07:00

FreeCAD replay kiểm tổng history/property records và dung lượng snapshot/tên trước khi sao chép; dùng lại bảng transaction descriptions với borrowed view đồng bộ. Hủy Undo/Redo dùng FlagToggler, chặn reentrant mutation và pop record trước destructor; nhánh visible Redo của failed preserving group cũng được bảo vệ.
**App273/plugin275/kernel240:3372/3372 kiểm tra hành vi qua34 host;33/34 host sạch log.** Một host close-object có startup Unhandled std::exception trước fixture, nên runtime gate chưa đạt toàn bộ. Snapshot38, normal-close finalizer16 và empty-group Redo5 đạt; các lượt diagnostic278/280 không cộng vào tổng. Rust toàn bộ lần gần nhất1037/0, không chạy lại cho thay đổi native này.
**RCORE-01 vẫn đang triển khai:** startup deadlock, peak/opaque allocation, lifecycle và phần còn thiếu của hợp đồng gốc; các checklist lịch sử cần đối chiếu theo evidence, không tự coi mọi fixture đã đạt là chưa làm.

Evidence: H:/FreeCAD-src/Mod/OpenMatrix9/docs/validation/2026-10-12-rhino-core-snapshot-history-disposal.json
Next: Capture the remaining startup deadlock at its native origin; reconcile original RCORE01 accepted slices, then reproduce remaining object/member lifecycle and aggregate preparation budgets using existing FreeCAD and Rust mechanisms.


Notification lock — 2026-10-12T14:20:19+07:00

Đã sửa startup resource-deadlock tại FreeCAD NotificationArea: dựng nội dung dưới mutex rồi thả khóa trước khi mở popup; cảnh báo Qt có thể vào logger bình thường. Popup thật vẫn giữ thông báo và warning, không tắt thông báo để né lỗi.
**App273/plugin275/GUI287/kernel240:3375/3375 kiểm tra qua35 host sạch log, tất cả exit0.** Fixture popup RED2/3→GREEN3/3, stack native xác nhận nguyên nhân. Giữ các lỗi startup274/278/284 và các crash cũ làm evidence lịch sử. Rust toàn bộ lần gần nhất1037/0, không chạy lại cho thay đổi GUI native này.
**RCORE-01 vẫn đang triển khai:** còn các phần chưa có evidence đầy đủ về conversion, consumer/result, ownership/preservation, aggregate bounds và lifecycle/cache; T01-T11 đã có accepted slices, không mở lại toàn bộ chỉ vì checkbox lịch sử.

Evidence: H:/FreeCAD-src/Mod/OpenMatrix9/docs/validation/2026-10-12-rhino-core-notification-lock.json
Next: Continue remaining native derived-object/member lifecycle witnesses and aggregate preparation budgets, then mesh/special Edit GeometryResult consumers using existing FreeCAD ownership and prepared discrete snapshots.
