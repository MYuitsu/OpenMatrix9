# Delete the selection

Key **Del**.

![Before and after: the circle goes with its constraints, and the degrees of freedom go up.](img/sketch-delete/)

## How to delete

Select lines, arcs, points in the sketch — or one constraint in the list of constraints, a text, a note — and press
**Del**. The deletion is one step of undo: **Ctrl+Z** brings everything back at once.

## What goes with them

The constraints and dimensions that held on the deleted geometry: a constraint missing one of its sides cannot work.
Points where other entities meet stay — only what belongs to no one any more goes. The origin and the axes of the
sketch are not deleted.

So after a deletion there are **usually more degrees of freedom**: you removed what held the shape. How many there
are now shows in the properties of the sketch on the right. If the sketch is no longer defined, define it before
building on it further.

## If a body already stands on the sketch

An operation that took the deleted contour — an extrusion, a revolve — turns **red** in the timeline after you leave
the sketch, with the reason "The sketch profile was not found"; the body stays as it was. Restore the contour in the
sketch and open the operation again to pick it — or **Ctrl+Z**.

## A common mistake

Deleting an "extra" line in a defined sketch and finding the shape moved. It is not the deletion: the constraints of
the neighbouring entities held on that line. The properties of the sketch show how many degrees of freedom were
freed.
