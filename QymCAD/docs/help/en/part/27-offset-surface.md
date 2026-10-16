# Offset surface

Builds a **sheet at a distance** from faces of a body: the same shape, moved along the normal. The body stays put,
the sheet stands beside it as a surface of the part.

## How

1. Pick the faces — or press **Offset surface** with nothing picked and click faces of a body or a sheet.
2. Type the **Distance** in the field at the geometry: positive goes out of the body, negative goes in, zero is a copy
   in place. The field takes a formula.
3. The preview shows the sheet before it is applied. **Enter** applies, **Esc** cancels.

A double click on the timeline row opens the command again — with the same faces and distance.

## Why

- To set a **surface off by a thickness**: a skin along the outline, a gap for a coating.
- To give [Replace face](part/21-surface-replace) another shape: the top of a block replaced by a sheet lifted 5 makes
  the block 5 taller.
- To get a sheet for [Thicken](part/15-thicken), [Trim](part/24-trim) or [Stitch](part/23-stitch).

## Worth knowing

- **The sheet follows the body.** Change the part higher up the timeline and the sheet rebuilds at the same distance
  from the same faces.
- **Faces are stored as a description**, as for [Copy face](part/20-face-copy): "all faces of this feature" stay those
  after the base changes.
- **A sheet has no volume.** The source body does not disappear: the sheet stands beside it.

## If it does not work

- **The node is red: a face turns inside out or vanishes** — the distance inward is larger than the radius of a curved
  face (a cylinder of radius 10 cannot move in by 12). Take a smaller distance.
- **A click took no face** — an edge or empty space was clicked; click the face itself.
