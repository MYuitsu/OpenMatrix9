# Rhino core delayed Rebuild publication implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans. Continue the authorized native execution and all-P0 scope.

**Goal:** Reject a real Rebuild solver result delivered after its source/session/options become stale, and release its owned native outputs exactly once.

**Architecture:** Reuse the existing Rust fitter, native Part shape factory, ObjectSnapshot guards and Coin preview. Safe Rust owns a bounded, monotonic request gate; C++ retains only owned result shapes and copied guard values across queued delivery. Resolve the document by name and validate session/source/context at the actual publication boundary before Coin attachment or a transaction. Preview delivery is queued on the GUI thread; the solver remains synchronous. An environment-gated fixture holds the same production result/delivery path for deterministic late completion.

**Tech Stack:** Rust2024, C++/Qt6/FreeCAD/OCCT/Coin, native FreeCAD macro fixture.

**Spec:** `OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/01-object-model-kernel.md`, RCORE-01.T08 and ownership/preview clauses. This is an authored Rebuild result-publication proof; no all-worker/thread-safety certificate.

## Global constraints

- Reuse existing commands and solvers. Rust owns portable policy/state; C++ is the host/kernel/lifetime adapter.
- Never retain a Document/DocumentObject/view-provider pointer in the deferred result.
- Source snapshots include session, generation, revision, Undo epoch, UID, units, ancestors and world placement.
- Preview adds no document object or Undo. Commit remains one existing native transaction.
- Invalid or canceled delivery releases its owned result and attaches no preview.
- Request IDs never wrap/reuse. At most16 undelivered envelopes; each Rebuild already permits at most16 sources.
- Existing Line/Polyline/InterpCrv/Rebuild geometry and supported options stay unchanged.

## Review focus

- Source edit then exact geometry restoration must remain stale: revision guards, not geometric equality alone.
- Same-name document/source recreation must reject the old result without dereferencing its old document.
- Options/newer solve must invalidate an older result without clearing the newer preview on late delivery.
- Cancellation/document switch/close must clear displayed Coin nodes; queued native owners retire once on delivery.
- Failure, duplicate claim/retire and request exhaustion must fail closed; capacity must recover after retirement.

### Task1: Portable bounded request gate

**Files:** create `rust/src/result_publication.rs`, `rust/tests/result_publication.rs`; modify `rust/src/lib.rs`.
**Interfaces:** fixed-width `Gate { next:u64,current:u64,slots:[u64;16] }`; init next1. Rust `issue`, `invalidate`, `claim(ticket,context_valid)`, `retire(ticket)`, `pending`; borrowed synchronous C ABI `om9_result_*`. No allocation or host addresses retained.

- [x] Write tests for monotonic/replay/cancel/newer-result/invalid-context/capacity/exhaustion and ABI layout/null handling; observe RED before implementation.
- [x] Implement the safe gate and small synchronous ABI, then run Rust suite and Clippy.

### Task2: Guard the actual native result-publication boundary

**Files:** modify `Gui/CoreRebuild.cpp`, `Gui/CoreRebuild.h`, `Gui/AppOpenMatrix9Gui.cpp`; create `tests/rhino_core_delayed_publication_smoke.FCMacro`.
**Interfaces:** `CoreRebuild::publish(shared_ptr<Result>)` resolves current document by copied name, verifies all copied ObjectSnapshot values, then claims Rust ticket exactly once. The existing preview/commit body consumes these same native outputs. QObject-context queued callback is GUI-thread-only. A test API only when `OM9_TEST_NATIVE_DIR` is set controls hold/release FIFO and returns actual envelope/shape/preview counters.

- [x] Add a host RED for a real solved result pending publication; source-edit and close before late delivery must reject. Record exact baseline/actual output owners; never regenerate on release.
- [x] Move the existing publication body behind the shared gate. Queue previews; immediate commit uses the same validation. Invalidate on options/cancel/start; document close cancellation uses existing signals. Owned envelope destructor releases shapes under GIL and retires its ticket once.
- [x] Exercise unchanged positive preview; edit, edit+restore, replace, Undo, parent placement, unit context, options/newer result, cancel, switch and close+same-name recreation. Assert inventories/Undo/source bytes, actual Coin count and envelope/native-shape release counts. A late older result must preserve newer preview.
- [x] Build the standalone SDK plugin and run the new fixture plus unchanged curve_spline and source-lifecycle controls on the exact module. Re-run Rust and applicable native checks.

### Task3: Review and acceptance evidence

**Files:** validation report/evidence, independent review, README/spec/feature/live ledger/acceptance matrix and package manifest.

- [x] Obtain independent review of Rust policy, native deferred ownership, document-close/cancel boundary and actual fresh-process artifacts.
- [x] Record immutable source/module/macro/result/process/raw geometry hashes, RED and all excluded attempts, exact scope and limits. Update only earned T08 coverage; keep all-P0 active.
- [x] Independently verify every evidence fingerprint and package manifest, diff-check, and save a local checkpoint. Continue T10 and remaining P0 requirements.
