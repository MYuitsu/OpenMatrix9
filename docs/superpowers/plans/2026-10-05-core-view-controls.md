# Core View Controls Implementation Plan

> Use superpowers:executing-plans inline. Full 130-spec goal remains active.

**Goal:** complete remaining basic View interactions, then continue the next dependency groups in the full core inventory.

**Architecture:** Rust owns drag state, validation, scale math, crosshair toggle and original command mapping. C++ owns native camera operations, Qt rubber-band/crosshair painting and document/view lifecycle.

**Spec:** `docs/superpowers/specs/2026-10-05-core-design.md`, OM9-VIEW-003/004/008/009 in local 01-core.

1. RED tests: Perspective-source sync; ordered/reversed window drag; tiny rectangle; dynamic incremental scaling; invalid input/cancel.
2. Implement Rust controls and ABI, preserving public names SynchronizeViews, Zoom_Dynamic, Zoom_Window, Crosshairs.
3. Native controller routes menu/CMD and mouse to that state. Window drag calls native boxZoom with physical pixels; Dynamic and Ctrl+left drag scale the camera. Esc restores pre-tool camera; successful history waits until release. Task restrictions, document/workbench changes and deleted views cancel safely.
4. Crosshairs paint white tracking lines in viewport widgets without intercepting mouse events or creating model geometry. State comes from Rust; hide outside workbench, retain toggle when returning.
5. Native tests: changed camera scale/center, rectangle direction/DPI, cursor tracking, toggle, cancellation, no object/Undo mutations, view deletion, multiple documents and permissions. Run prior core and feature regressions.
6. Update exact requirement evidence and continue File/Info/remaining core groups; no full core completion claim.

Host decisions: 3 logical-pixel minimum rectangle; Dynamic factor is 2^(vertical delta / viewport height), clamped per move; Perspective scale is the apparent height at its focal plane. No additional product dependencies or reference PDFs.
