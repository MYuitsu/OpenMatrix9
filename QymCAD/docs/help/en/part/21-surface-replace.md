# Replace face

Hands a surface back to the body: the picked faces are removed, the surface takes their place, and the
body is whole again.

![The same round trip: the face is taken off as a copy, reshaped on its own and handed back.](img/part-face-copy/)

## Why

This is the far end of the bridge into the design layer. The loop is: **take a face out** ("Copy
face"), reshape the surface on its own, **hand it back**. The node lives in the timeline, not as a
layer on top of the model, so anything can be built below it: fillets, holes, a shell.

## How

1. Press **Replace face**.
2. Click the body faces you are replacing.
3. Click the surface — it is a separate sheet body, hard to confuse with anything.
4. **Enter** to apply, **Esc** to cancel.

## Worth knowing

- **One face can be replaced by a sheet standing off it.** The neighbouring faces reach the sheet: the top of a
  block replaced by an [offset](part/27-offset-surface) sheet lifted 5 makes the block 5 taller, a sheet sunk 3 makes
  it 3 lower. The sheet must cover the face whole.
- **Several faces — only along their edges.** The surface stands on the edges of the faces replaced. A common
  mistake: the top of a shelled part is a RING as wide as the wall, while the patch is stretched across the whole
  opening. The ring and the lid are bounded differently, the inner loop is left unpaired — and the node turns red.
  Build the patch on the same edges that bound the face being replaced.
- **Capping an open box is a different job.** A shelled part is already closed: the top is a recess in
  the material, not a hole in the surface. A lid there adds material — that is an extrude or a thicken,
  not a face replacement.
- **The surface must close the opening.** If a gap is left after the faces are removed, the node turns
  red and says how many edges are left unpaired, and the part stays as it was.
- **Faces are stored as a description.** Say "all faces of this feature" and the node keeps replacing
  exactly those after the base changes, not yesterday's numbers.
- **Both inputs are absorbed.** One part is left on screen, not a part plus a surface over it.
- **The faces are gone after an edit of the base** — the node turns red with the reason; pick the faces
  again by a double click on the row.
