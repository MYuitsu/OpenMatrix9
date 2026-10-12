# Core Workspace Implementation Plan

> Use superpowers:executing-plans to implement inline, task by task.

**Goal:** Implement all 01-core functions; this plan begins with the required native four-view workspace while the full scope stays in `docs/core-requirements.json`.

**Architecture:** Rust slot descriptors and command routing; native C++ viewport lifecycle/cameras/grid; shared command input.

**Tech Stack:** Rust, C++23, Qt6, Coin and matching FreeCAD SDK.

**Spec:** `docs/superpowers/specs/2026-10-05-core-design.md`.

- [x] Inventory every 01-core spec and preserve its requirements.
- [ ] Add failing Rust descriptor tests and native four-view runtime test.
- [ ] Implement `rust/src/core_views.rs` descriptor/command mapping and ABI.
- [ ] Implement `Gui/CoreWorkspace.h/.cpp` view lifecycle, layout, camera/grid management.
- [ ] Connect workbench lifecycle, restore/synchronize/center/grid commands and shared CMD.
- [ ] Connect per-view C-plane projection and F4/F7 keys.
- [ ] Build matching SDK; validate at three DPI scales, document/view closure, resizes and regressions.
- [ ] Update inventory with exact coverage and remaining requirements; proceed to the next dependency group without declaring full core completion.

Global constraints: retain original command names; no PDFs; no unrelated edits/commits; disabled placeholders are not implemented features. No broad refactor. Preserve native document task/edit permissions.

Review focus: view pointers destroyed during document closure; multiple document MDI isolation; camera/grid nodes changing model bounds or selection; DPI projection and active-view identity; reactivation creating extra views; CMD and mouse C-plane agreement; perspective synchronization isolation; persistence without adding model geometry.
