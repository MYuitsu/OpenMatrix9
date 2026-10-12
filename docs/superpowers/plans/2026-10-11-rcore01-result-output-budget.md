# RCORE-01 Result Output Budget Implementation Plan

> **For agentic workers:** Use superpowers:executing-plans inline for this existing serializer repair. Steps use checkbox syntax.

**Goal:** Enforce the common GeometryResult node/depth bounds on generated output as well as the input request, without raising limits or dropping geometry.

**Architecture:** Reuse `prepared_result::encode_owned` for common result serialization. Safe Rust validates the owned tree before writing it; the existing bounded writer still enforces the byte limit. C++ and existing native solvers remain unchanged.

**Tech Stack:** Rust, existing Json tree/parser/writer, standalone OpenMatrix9Gui and shared FreeCAD SDK.

**Spec:** `OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/01-object-model-kernel.md` — bounded allocation/traversal and complete owned GeometryResult. This task does not close the rest of the chapter.

## Global Constraints

- Keep 16MiB result bytes, 1million JSON nodes and parser depth64; no limit increases.
- A resource overflow returns InvalidBasis2 before output publication or buffer copy.
- Preserve full snapshots, input identities, units and conversion uncertainty.
- Build only the standalone plugin using `build/build-om9-plugin.ps1` and the existing SDK/cache.
- Preserve Runtime41 frozen producer inputs and native receipts as historical proof after further source changes.

## Review Focus

- A request fitting exactly the node limit can exceed it after adding result metadata.
- Wrapping a valid deeply nested snapshot in output_snapshots adds an ancestor.
- Invalid output must leave a caller's initialized output buffer unchanged.
- Legal output at the boundary must retain all geometry and round-trip through the common parser.
- Unknown retained snapshot fields still consume output budget; they must not be silently discarded.

### Task 1: Bounded owned output

**Files:** `rust/src/prepared_result.rs`, unit tests in that module.

**Interfaces:** Keep `assemble(&str)->Result<String,u32>` and `encode_owned(&Json)->Result<String,u32>`. No FFI signature changes.

- [x] Write tests with an input tree independently confirmed within the existing parser limits: near-node-cap output growth, depth64 input becoming depth65 output, legal depth boundary, and untouched FFI output buffer on rejection.
- [x] Run the focused tests and preserve actual RED output.
- [x] Validate the tree's depth and total nodes before serializing; route assemble through encode_owned.
- [x] Run focused and full Rust tests, then standalone plugin build and matching native regressions.
- [x] Record exact source/binary/log evidence and synchronize the spec/README/ledger without marking the chapter complete.

No staging or commit is performed while this task's Git metadata is read-only.

Scoped task evidence — 2026-10-11T20:03:10+07:00:RED1/4→GREEN4/4;full Rust833/0/0;standalone build0;matching892/892 checks/18 actual exits0. This closes the stated serializer task only. Full chapter, pre-clone allocation and unknown final overhead remain required.
