# OM9-SURFACE-009 — Loft

Menu ID: `OM9_SurfaceLoft`. CMD alias: `Loft`.

## Source-derived contract

The supplied `OpenMatrix9_Codex_Spec_v1/specs/03-surface/om9-surface-009-loft.md`
defines a surface through ordered open or closed curves/edges, closed-curve seam
alignment, Enter, and a Loft options dialog. Citation: Matrix 8 Book 1,
printed p.183 / PDF p.193. The option cues include Normal (Default), Loose,
Tight, Straight Sections, Developable, Flip, Automatic, Natural and Point.

`TODO_EVIDENCE`: the original PDF and continuation pages have not been rechecked;
option cue names alone do not establish their complete algorithms/defaults.

## OpenMatrix9 implementation decisions

Shares the input, preview, transaction, error and persistence contract of
[Sweep 1](OM9-SURFACE-001.md), except no rails and at least two sections.
Native `Part.makeLoft` produces an uncapped surface/shell. Selection order is
preserved rather than spatially sorting sections. Edge subreferences are saved.

- Normal: native smooth loft (`ruled=false`), the initial host choice.
- Straight Sections: native ruled loft (`ruled=true`).
- Closed loft: explicit opt-in connecting last section to first; requires at
  least three sections. This is separate from closed cross-section curves.
- Reverse: independently flip each selected input. A normalized parameter seam
  fraction [0,1) is available for single-edge closed curves; the host inserts a
  NURBS knot and changes the periodic origin while preserving curve geometry.

Loose, Tight, Developable, Automatic/Natural/Point seam modes and dragged seam
markers are not claimed as implemented. Plain Loft creates a snapshot with
source references; editing source curves does not automatically recompute it.

## Validation

Rust tests check two-section minimum, consistent closure, preserved order and
Undo input. Native tests compare circle loft area with an analytic cylinder,
exercise Normal/Straight Sections/Closed loft, seam and Reverse edits, surface
edge inputs, command aliases, Cancel, invalid selection, document-close cleanup,
Undo/Redo and FCStd reload. See `docs/openmatrix9-progress.json` for results.
