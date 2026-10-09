# OM9-CURVE-004 — Rectangle

Verified supported scope: sharp corner-to-corner, Center, 3Point and Vertical
rectangles. Original command name and icon key remain `Rectangle` and
`CurveRectangleCornertoCorner`.

## Source evidence

Matrix 8 Book 1, printed pp.127–128 / PDF pp.137–138, was read on 2026-10-09
from `OpenMatrix9_Codex_Spec_v1/matrix8_book_1.pdf`. Rectangle begins on PDF137;
its Rounded options continue on PDF138 before Circle. The source describes
two opposite corners, a numeric length followed by a corner, Shift square,
3Point from an edge and opposite side/width, Vertical, Center and Rounded.
Rounded Arc and Conic have different decomposition; neither is implemented
by this slice. Exact Matrix defaults for numeric Center dimensions are not
established by those pages.

## OpenMatrix9 decisions

- Start in Corners mode, no fixed Length or Width, sharp corners. Choose
  `3Point`, `Vertical`, `Center` or `Corners` before the first point.
- Inputs are world points from mouse/snaps; CMD `x,y[,z]` and `rX,Y[,Z]` use
  the construction frame. The frame is latched at the first point. Modes and
  dimensions are reset on a new command. No preselection is required.
- Units are mm, with explicit `mm`, `cm`, `in` suffixes. Finite coordinates
  and output corners must stay within ±1e9 mm. Each side is at least 1e-7 mm.
- Corners uses CPlane X/Y components of the diagonal, in a plane parallel to
  the latched CPlane through the first corner. Off-plane second-point height
  is projected out. Center reflects the corner around the picked center;
  entered Length/Width are full side dimensions.
- `Length=N` / `L=N` is positive. `Width=N` / `W=N` is signed and nonzero.
  Bare `Length` or `Width` requests its value. In Corners/Center, after the
  first pick a bare number fixes Length; another fixes Width and completes
  the frame. Edge modes still require a second pick for edge direction;
  after that pick a bare number sets Width.
  Length followed by an opposite-corner pick uses the pick's X sign/Y width.
  Explicit Width sets the side direction by its sign.
- 3Point uses the first two picks as an edge; the opposite side is the third
  pick's component perpendicular to that edge, allowing arbitrary slant.
  Numeric width uses CPlane normal × edge, with CPlane Y as the fallback
  when the edge is parallel to the normal.
- Vertical projects the first edge onto the latched CPlane, then uses its
  normal as the width direction. After two picks, mouse rays intersect the
  vertical session plane; change camera or type width if the ray is parallel.
- Shift makes both sides equal, independently of global Ortho. With a fixed
  Length, it uses that Length; otherwise corner mode uses the larger diagonal
  component and edge modes use the edge length. A pre-entered Width also
  obeys Shift during the final mouse pick and matching preview.

## Native output and lifecycle

Rust owns mode, dimensions, geometry validation, constraints and session state.
C++ adapts the resulting five closed polygon coordinates through the typed
Part API. One root `Part::Feature` contains one valid closed wire with exactly
four straight edges and `OM9FeatureId=OM9-CURVE-004`. No face/solid is implied.
Menu, Curve sidebar and CMD share the handler; the F6 CurveLayout entry uses
the same command.

The Coin preview displays the exact pending outline and picked-point markers
without document objects or transactions. Success creates one object in one
transaction and enters command history only after commit. Invalid coordinates,
degenerate geometry and unsupported options keep the input phase editable.
Native adapter/kernel failures abort the transaction and end the session with
an error; restart Rectangle to retry such a failure.
Undo removes the most recent picked point and clears dimension constraints;
Esc/Cancel, workbench deactivation and document changes clear the preview and
session. Idle prompts return to `Command:` after commit/cancel.

Output is a geometric snapshot with no linked source dependencies; Matrix
History recomputation and active-layer inheritance are unsupported. FreeCAD
Undo/Redo and FCStd save/reload preserve the snapshot and exact feature ID.
Output uses the shared Curve visibility/color policy.

## Validation

- `rust/tests/rectangle_session.rs`: geometry, slant, signed dimensions,
  rotated construction frame, units, square, equivalent preview/click,
  recoverable invalid input, overflow rejection, Undo and Cancel.
- `rust/tests/curve_command_permissions.rs`: document-write permission.
- `tests/rectangle_smoke.FCMacro`: native menu/CMD/sidebar, four-edge wire,
  mouse preview/square/Vertical, errors, Undo/Redo, lifecycle, persistence.

Runtime evidence and build details: `docs/validation/2026-10-09-rectangle.md`.
Native results are recorded there after execution; source/sample presence
does not establish runtime validation.

## Remaining work

Rounded Arc/Conic, the Rounded right-click action, radius picking, active-layer
integration, source-edit History, and exhaustive Matrix/Rhino compatibility.
Unsupported option input reports an error and does not silently create a sharp
rectangle. The next feature in this Curve sequence is OM9-CURVE-005 Circle.
