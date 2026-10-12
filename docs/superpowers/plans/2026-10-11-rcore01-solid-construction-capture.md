# Solid construction capture implementation plan

> **For agentic workers:** Use superpowers:executing-plans for the sequential test/native integration task below.

**Goal:** Preserve Rust-owned accepted Box/Sphere construction state as independent input evidence for the required shared GeometryResult integration.

**Architecture:** Reuse the existing Session and capability policy. Rust serializes the accepted state and a generation-bound copy/query ABI returns only owned UTF-8 bytes. The FreeCAD adapter stores those bytes and the actual unit-context snapshot with the Shape inside its existing checked publication transaction. Existing Part solvers and supported modes remain unchanged.

**Scope retained:** Full RCORE-01 remains required: every representation, enabled consumer, eight errors, adapter1–7, C01–C12 and T01–T11. This prerequisite does not substitute construction metadata for GeometryResult, conversion deviation/maps or live source validity. Selected FitPoints provenance/guards, full source snapshots, actual Solid preview leases and other open contracts remain required.

**Native exception:** C++ is needed only to call the value ABI on the GUI thread and write/read native FreeCAD properties atomically. Pure capture and state decisions remain Rust. Python macros are host tests only. Reuse the single SDK/plugin cache and isolated App19/camera-guard test host; native hosts run serially.

- [x] Add Rust tests for owned accepted Box/Sphere state, point/constraint/frame fidelity, rejection before completion, exact u64 generation, cancellation/Undo isolation and output-buffer canaries. Observe failure before implementation.
- [x] Implement Session construction serialization using the existing bounded JSON writer; expose an exception-contained, generation-bound query/copy ABI. Do not retain host pointers or change the solver.
- [x] Add an actual native command fixture for persisted construction/units data, standalone/cold lifetime and rollback on metadata tampering. Run current Runtime33 RED before native integration.
- [x] Capture before callbacks, persist the frozen construction/units properties in the same transaction, and reject their mutation/removal before commit. Reuse existing checked publication gates.
- [x] Build a fresh standalone runtime, run full Rust and relevant serial Solid native regression suites, request independent review, audit/synchronize evidence and manifest. Leave full RCORE-01 in_progress.

## Verified prerequisite checkpoint — 2026-10-11T17:44:24+07:00

Runtime35 native757/8 includes cold21 and final capture67. Full Rust816 runs at identical Runtime34 Rust inputs. Review findings about reduced points and missing metadata were repaired after actual failing witnesses. [Evidence](../../validation/2026-10-11-rhino-core-solid-construction.md). Full RCORE-01 remains in_progress; full Solid GeometryResult is next. Public audit/document verification must pass before this checkpoint is reported.
