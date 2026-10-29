# Proposed reusable rules — awaiting user consent

No new business rule has been written to repository or installed skills by this batch.
The prior Rectangle authorization concerns its own signed-CPlane/input regressions.

Before → after:

- SetPt moved placement origin → SetPt edits the selected canonical point/control coordinates,
  using explicit axes and World/CPlane while preserving placement. Solid control points require
  an actual solid-grip service; curve/surface grips are insufficient.
- ArrayCrv kept orientation and only accepted count → Count/Spacing mm and explicit
  NoRotation/Freeform/Roadlike; read-only preview and commit share arc-length station parameters.
- Measure accepted curves/circles/points only → canonical BRep/isocurve/integral readouts,
  pointwise curvature and object modes; labels distinguish exact/native and numerical scope.
- Annotation catalog rows were disabled/unowned → one model-space owner with typed persistent
  semantics, style, edits, preview, atomic Undo and tested menu/CMD bindings.

Proposed text for feature-port/UI/build-validation references:

> Measure canonical geometry after occurrence placement; never substitute a display mesh or
> bounding box for exact edge/solid measures. Label spline/ellipse radius as local curvature at
> an explicit station. At knots, compare exact one-sided geometric jets and refuse incompatible
> limits. Object distance/angle must state operand restrictions and keep the document unchanged.
> Unknown gem weights stay unknown; group placed occurrences without adding definitions to totals.
>
> SetPt edits point/control coordinates, not placement. ArrayCrv uses the shared arc-length
> parameter for both station position and tangent; expose spacing and orientation explicitly.
> Preview is read-only and uses the same plan as commit; validate axes, placement, orientation,
> invalid/retry, Cancel and one Undo with independent numeric oracles.
>
> Model-space annotation stores typed witness/style/text semantics separately from disposable
> display strokes. Preserve catalog ids/icons and shared menu/CMD dispatch; test edits, Unicode,
> atomic Undo/Redo and native archive persistence. Do not infer Rhino annotation/style/layout
> compatibility or human/GPU acceptance from those tests.
>
> Before declaring a backend missing, search and exercise its current safe Rust owner. A
> closest-point/pullback routine does not by itself prove a whole curve is attached to a surface
> or that a menu command correctly selects, places and orients objects. Undefined facing behavior
> remains blocked until resolved.

If approved, sync the relevant repository skill references and installed copies, then validate
their links and record the exact approved scope. No unrelated defaults or business rules change.
