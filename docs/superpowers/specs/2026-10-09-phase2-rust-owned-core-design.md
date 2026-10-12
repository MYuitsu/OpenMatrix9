# Chuyển logic Phase2 hiện tại sang Rust

Status: design approved by the user on2026-10-09 (“Duyệt thiết kế”); plan approved and implementation accepted_scoped after matching application gates and one final review. User explicitly requests Rust first and chooses migration of current Phase2, not only future work. This is an architectural migration of ownership and FFI, preserving the accepted Phase2 feature contract.

## Outcome and boundary

Rust owns portable Phase2 logic and independent data. C++ remains the minimal FreeCAD/Qt/OCCT/openNURBS/Coin adapter. Python remains only required workbench hooks and existing application tests/tools; the production PointsOn editor no longer imports ModelingCurveEditor.py. Do not reimplement the native CAD kernel or claim its C++ dependencies become memory-safe.

Current baseline: Phase2 runtime native module56270dc28aa635445518a61f1b16db48a6dfb1fd899fb0b2d84d7ef1448e61dc; accepted24 host reports/437 checks, native49 suites, Rust88, Python58, Rhino5 3 fixtures/125 checks, actual SaveAs reread55 checks. Evidence at docs/validation/modeling-phase-2/summary.json. These are historical baseline proofs, not proofs for the migration binary.

## Component ownership

| Slice | Rust owner | Native adapter retained |
|---|---|---|
| Classification/eligibility | Kind/capability/preview decisions and allowed operations | Inspect actual native runtime types/property presence, resolve native Links, expose cheap shape/display witness/affine transform |
| Curve basis | Typed owned poles, weights, distinct knots, multiplicities, degree, periodicity, active domain; validation and editor request state | Read native owning curve/basis; construct/check native BSpline/edge; preserve placement/orientation; document identity/stale witness and transaction commit |
| PointsOn | Session/model, draft edits, validation, error state and lifecycle decisions | Qt numeric widgets, signals, native document/workbench lifecycle notifications; no Python editor model/UI dependency |
| Join/Rebuild | Selection/input budgets, graph valence/complete unique ordered traversal, rebuild options/session and spline fitting | OCCT topology/edge identities and ordered traversal snapshots, native shape builder, preview rendering, dependent-object/lifetime witness and atomic GUI transaction |
| Snap | Owned rows/tiles/candidate cache, budgets, mode eligibility, nearest-point ranking, invalidation generations and complete/incomplete state | Visible native objects, world bounds/transforms, Coin camera/viewport projection, bounded OCCT extraction and host signals |
| Worker | Existing Rust60%/task/RAM policy plus owned task snapshots | Required native geometry/serialization API and GUI document read/commit |

## Typed boundary and memory contract

- Pure modules expose checked Rust APIs with Vec/String/typed records; no document/widget/kernel pointers in the core. C++ must not be the second implementation of portable validation.
- FFI is small and separate from safe core. Transfer numeric buffers with explicit pointer+length, fixed-layout scalars, integer identity/generation tokens and status/error values. Reject lengths/caps/checked-product overflow before constructing slices or allocating copies. Caller guarantees pointer validity/alignment and lifetime for the call; Rust cannot validate arbitrary addresses.
- Rust copies borrowed input into owned data where sessions/cache/workers outlive the call. Cross-language results use caller-owned buffers or explicit same-side allocation/free; no shared mutable ownership. Handle deletion/document close invalidates generation; stale results cannot commit.
- No panic or native exception may unwind across ABI. Check existing panic=abort profile and avoid panic-producing core/FFI operations; test malformed fields/lengths/indexes and poisoned/stale handles. Native exceptions become incomplete/error and preserve document atomicity.
- GUI/native adapter reads or mutates FreeCAD documents on the permitted host thread. Workers receive independent immutable Rust data or a kernel copy proven safe for the operation; worker count policy stays60%, bounded by tasks/RAM. Rust does not own native Qt widget pointers.

## CV model and UI behavior

Keep current original PointsOn name/menu/CMD route, explicit owning single-edge selection, local CV coordinates, numeric degree/poles/positive weights/strictly ordered distinct knots/multiplicities/periodic/domain controls, stale-signature guard, Cancel/document/object/workbench close and one Undo transaction.

Rust validates finite values/ranges, typed integer fields, array shape/budgets, weights/knot constraints and compatible basis/domain information. OCCT performs native constructibility/validity checks before commit. Native identity/placement/orientation are captured as immutable witnesses, not live Rust pointers. Qt host presents Rust errors and updates the draft; invalid input stays open without transaction.

## Join/Rebuild behavior

Join preserves2..16 distinct open inputs,64-edge cap and one independent native nonbranching chain/cycle; branch/disconnected/closed-input rejection remains atomic. Rust receives native endpoint/edge identity and traversal snapshots and checks degree/coverage/uniqueness. Rebuild retains16-curve/256-pole options, default DeleteInput=true, complete whole-chain sampling, dependent-input protection, preview/cancel, identity/placement stale guards and one Undo/Redo. Rust owns portable decisions/fit; OCCT sampling/native topology witnesses stay in the adapter.

## Snap behavior

Preserve End/Mid/Point native CAD only,8 logical-pixel radius,64 objects/2048 candidates per object/8192 total before allocation, manual/typed fallback when incomplete, and nested/reflected Link/visibility semantics. Mesh/cloud/SubD preview arrays are never queried for default snapping.

Rust stores rows/tiles and candidate caches using numeric object tokens plus document/view/geometry generations. Native camera projection and bounded topology extraction feed copied numeric data. Query budget/ranking decisions move out of C++ containers. Rebuild uses staged Rust data published only after success; an aborted native camera/index rebuild leaves the new generation incomplete and cannot certify a partial cache. This also addresses the earlier deferred exception-path issue as part of ownership migration.

Source-level no-heavy-access checks remain necessary; literal zero counters and Python Proxy poisoning are not independent access instrumentation. Do not silently promote that old limitation into an instrumented proof.

## Migration and verification boundary

Preserve an immutable accepted-code/runtime/manifest snapshot before changing product files; retain pre-existing broken Git-link ruling and no primary product overwrite/commit/push. Build a separate migration runtime and bind all new evidence to its source/module/helper/fixture hashes. Skill edits do not modify or certify the accepted Phase2 binary.

Suggested implementation sequence: owned Rust basis/graph core and typed FFI; thin native curve/CV UI adapters removing production Python; Rust snap cache/query core and native extraction/projection adapter; packaged runtime and complete replay. This sequence is a design dependency description, not an approved implementation plan.

Acceptance requires meaningful RED→GREEN Rust semantic tests for every migrated core rule and malformed/stale input; FFI bounds/ownership/lifecycle tests; existing host19 batches plus HiDPI2; actual Rhino5 rational/periodic/placed Export Selected→edit/new/Rebuild/Join→currentV5→Open/SaveAs→FreeCAD reread; native49 and full Rust/Python regression suites. Validate source/runtime hashes and artifact packaging, unchanged Undo/Redo/Cancel, clipboard, worker60%, units and ring tolerances. Record hardware and500 warm queries on remote10k CAD and million-point Mesh/cloud with p95<=16ms; do not claim cold rebuild has warm latency.

Migration completes only after code is actually Rust-owned, production Python CV editor dependency is removed, new binary/application evidence is green, and scoped remaining native bridges are recorded. Full openNURBS/Phase3–5 remain open; 7 implementation packages are not closed by this refactor. No file-count completion percentage or blanket memory-safety claim.

## Risks and alternatives

Chosen: existing Rust static library + small C ABI + native adapters, retaining FreeCAD and CAD kernels. Alternative retaining C++ business logic conflicts with the requested migration; writing a new Rust CAD kernel/Qt binding stack adds unrelated scope and does not remove native dependencies automatically. Main risks are FFI ownership/lifetime, stale events/handles, projection precision/HiDPI, periodic/rational basis parity, packaging removal of Python UI, and performance after copying. Acceptance probes target each boundary.
