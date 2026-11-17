# Remesh display pin and reservation lifetime

The remeshed scene object now stores one `BrepDisplayPin` that contains both
the display Arc and the admitted owner of its reservation. Cloning the scene
object clones both together; borrowing the display does not expose its Arc.
The ordinary document display remains borrowed while its tracked document is
alive. A consumed B-rep can move its display out only when the display Arc is
unique; a shared display is refused.

In the new production remesh fixture, two detached scene-object clones keep
the remeshed display and its kernel charge after Undo. Dropping one keeps both;
dropping the last releases both. The existing buffer-only fixture confirms a
render-buffer clone does not retain the separate display charge.

Validation: app topology owner 49 passed/2 ignored, scene 34, core B-rep 23,
kernel B-rep 32 passed/3 ignored, picking 20, Sphere/Gumball including v4 2,
and GPU offscreen 29 passed. The 238 app integration targets compiled excluding
the unchanged `curve_shape_audit` fixture with missing symbols; all targets of
other workspace packages compiled. App no-default check, public-source audit
and diff check passed. Builds used `L:/om9-target` and `-j 2`.

The pin's caller is responsible for providing the correct admitted owner; the
type keeps that supplied owner and display together but cannot independently
prove the owner's reservation. At this test point identity used a weak Arc;
production receipts were subsequently changed to a value identity, as recorded
in [the follow-up](2026-11-10-display-identity-without-weak.md).
This change does not evict display from documents or Undo, attach proof/source
to history, bound native/process peak, or validate the running desktop.
`Brep::display()` and consuming-display APIs changed for external Rust callers.
M0/M1 remain open.
