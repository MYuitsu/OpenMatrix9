# Ellipse

Key **E**. Three clicks: the first is the centre, the second the end of the major semi-axis (its length and
direction, so the ellipse can lie at an angle), the third the minor semi-axis: its length is the distance from the
click to the major axis. Until the third click the ellipse is only a picture; after it the ellipse enters the
sketch and opens the width and height fields - Enter takes them, Esc leaves it as drawn.

![An ellipse with 26 and 14 mm semi-axes.](img/sketch-ellipse.png)

## When you need it

An ellipse shows up where a round section is cut at an angle: a slanted branch pipe, a hole in a
slanted wall, a decorative cut-out.

## What defines it

A centre and the ends of two semi-axes, major and minor. The direction of the major axis sets the tilt,
so a tilted ellipse stays an ellipse instead of turning into a spline.

## Where it is used

Oval windows and hatches, eccentrics, transitions between round and flat. An ellipse is an exact
curve, not an approximation made of arcs: a surface extruded from it stays smooth, and a machine
cuts it without steps.

## Do not confuse it with a slot

A [slot](sketch/07-slot) is two semicircles and two straight lines; its width is constant along its
whole length. An ellipse narrows towards its ends. A screw guide needs a slot; a shape needs an
ellipse.
