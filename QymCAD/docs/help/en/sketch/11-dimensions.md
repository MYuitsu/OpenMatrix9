# Dimensions

![The same rectangle before and after dimensions: labels appear, and the fixing glyph in the corner.](img/sketch-dimensions/)

## How to place one

- **Dimension** (key **D**): a click on a line gives its length, or pick two points. Where you pull the label with the
  pointer decides the dimension — along the points, horizontal or vertical.
- **Angular dimension**: two lines (the angle between them) or three points — A, the vertex, C. The angle is 1 to
  179°: the opening between the lines.
- **Radius or diameter**: a click on a circle or an arc.

A placed dimension opens its field: type a number or a formula — `50`, `w/2` — and press **Enter**. A double-click on
the label opens the field again.

## Where the label stands

Any label — of a length, a radius, an angle — can be taken with the mouse and led to where it reads well; the geometry
does not move. A length's text runs along its dimension line; led past an arrow it stands on a shelf, the dimension
line running on to it. A radius's or a diameter's text stands on a shelf past the bend of the leader; lower it inside
the circle and it lies along the dimension line itself, lead it out past the circle and it goes back onto the shelf.
An angle's label takes its arc along: farther from the vertex, a bigger arc. Lead the label past a side
of the angle and the arc runs on to it, and a short side reaches the arc with a thin extension line. **Ctrl+Z** undoes
a drag in one step.

## A dimension is a constraint, not a label

A dimension **holds** the geometry: change the number and the shape changes. That is why dimensions remove degrees of
freedom, and why they are not added "for looks".

## A formula and a name

The field takes an expression: `40/2`, `w*2`, `len+5`. Names come from the document's parameters
(**ƒx Parameters**). A dimension can get a name of its own in the **driver:** field — it then becomes a parameter
itself, and other dimensions and parts can refer to it in a formula.

## What a label says

By default a label is the value alone. The **Sketch** section of the settings has two switches: **Show the name of a dimension** — a
named dimension reads `w = 110`; **Show the formula of a dimension** — a dimension set by a formula reads
`2*w+10 = 110`; both together give `w = 2*w+10 = 110`. The Smaller and Larger buttons there change the size of the labels.

The text stands beside the dimension line and never crosses it — on the side away from the geometry: a dimension
above the part is written above its line, one below it below. The **Text of dimensions** choice there sets
**Along the dimension line** (a vertical dimension's text stands upright, read from the bottom up) or **Level**. When it does not fit between the arrows it is carried
past an arrow onto a shelf; a radius or a diameter has its text on a shelf after the bend of the leader.

## Reference dimensions

If a dimension would be redundant (the same is already set by other constraints), the program makes it a
**reference**: it shows the measured value but holds nothing. If a dimension **conflicts** with the existing ones, the
conflicting constraints turn red in the list of constraints on the right; the dimension has a ruler button there,
**Make it a reference** — or remove another constraint of the conflicting set.

## See also

- [Parameters and formulas](general/05-parameters) — how to use names across the part.
