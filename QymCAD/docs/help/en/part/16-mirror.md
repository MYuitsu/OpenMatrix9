# Mirror a body

Key **M**. Select the body, click the mirror **plane, datum plane or face** in the viewport, press **Enter**. The
**With the original (union)** tick keeps the source half and merges the reflection with it into one body; without it
only the reflection stays.

![A plate with a hole and its reflection — one feature, not two parts.](img/part-mirror.png)

## Half the work instead of the whole

A symmetric part is drawn as one half and mirrored. The gain is not only time: a change to one half
carries over by itself, because the other half is not a copy but a consequence.

## What serves as the mirror

A base plane, a datum plane or a planar face of the part itself. A datum is handier: its position is
parametric, and the mirror can be moved without touching the body.

## A hint

If the part is symmetric about the origin, draw it so that the plane of symmetry coincides with a
base plane: then the mirror needs neither a datum nor dimensions.
