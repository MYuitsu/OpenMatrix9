# Boat Hull - Loft and Sweep

- Source PDF: h:\OpenMatrix9\manuals\rhino5.pdf (Rhinoceros 5 User's Guide)
- Page range: 215-240
- Section title: Boat Hull - Loft and Sweep
- Section id: rhino5-082-boat-hull-loft-and-sweep
- Kind: tutorial
- Images (../images/): rhino5_p215_img01.png, rhino5_p215_img02.png, rhino5_p216_img01.png, rhino5_p216_img02.png, rhino5_p216_img03.png, rhino5_p217_img01.png, rhino5_p217_img02.png, rhino5_p218_img01.png, rhino5_p219_img01.png, rhino5_p219_img02.png, rhino5_p220_img01.png, rhino5_p220_img02.png, rhino5_p221_img01.png, rhino5_p221_img02.png, rhino5_p221_img03.png, rhino5_p222_img01.png, rhino5_p222_img02.png, rhino5_p223_img01.png, rhino5_p223_img02.png, rhino5_p224_img01.png, rhino5_p225_img01.png, rhino5_p226_img01.png, rhino5_p226_img02.png, rhino5_p227_img01.png, rhino5_p227_img02.png, rhino5_p227_img03.png, rhino5_p227_img04.png, rhino5_p228_img01.png, rhino5_p228_img02.png, rhino5_p228_img03.png, rhino5_p229_img01.png, rhino5_p229_img02.png, rhino5_p230_img01.png, rhino5_p230_img02.png, rhino5_p231_img01.png, rhino5_p231_img02.png, rhino5_p232_img01.png, rhino5_p232_img02.png, rhino5_p233_img01.png, rhino5_p233_img02.png, rhino5_p233_img03.png, rhino5_p234_img01.png, rhino5_p234_img02.png, rhino5_p235_img01.png, rhino5_p236_img01.png, rhino5_p236_img02.png, rhino5_p237_img01.png, rhino5_p237_img02.png, rhino5_p238_img01.png, rhino5_p238_img02.png

---
<!-- section rhino5-082-boat-hull-loft-and-sweep | Boat Hull - Loft and Sweep | rhino5 pages 215-240 -->

<!-- page 215 -->

## Boat Hull - Loft and Sweep
This tutorial demonstrates classic boat hull lofting techniques using typical plan and profile curves. The classic
hull shape is based on a design from an old Boat Builder’s Handbook magazine. Many designs similar to this are
available over the Internet.
You will learn how to:
●
Create 3-D curves from a 2-D lines drawing.
●
Rebuild and simplify the curves.
●
Use analytical techniques to ensure fairness.
●
Loft surfaces from the curves.
Rhino is used by marine designers in many segments of the industry. For more tutorials and information about
marine design, see the Rhino website at www.rhino3d.com.
### Note:
The images in this tutorial use a display setting to change the color of the back of the
surfaces.
Front face (1) , backface (2). Yellow arrows indicate surface normal direction, and
green color indicates surface backface.
This lets you see which way the normal direction of the object faces. Search the Rhino
Help for Backface settings.

<!-- page 216 -->

Boat Hull - Loft and Sweep
Marine Terms Used in this Tutorial
Sheer
The fore-and-aft curvature from the bow to the stern of a ship’s deck as shown in side elevation.
Chine
The intersection of the bottom and the sides of a flat or v-bottomed boat.
Transom
The planking forming the stern of a square-ended boat.
Fair
The meaning of “fair” is much debated in the marine industry. No one can define it, but they know when they
see it. Although fairing a surface is traditionally associated with hull surfaces, all visible surfaces on any object
can benefit from this process. In Rhino, the first cue for fairness in a surface is the spacing of the surface display
isocurves.
There are other characteristics of fair curves and surfaces. Although a curve or surface may be fair without
exhibiting all of the characteristics, they tend to have these characteristics. If you keep these in mind while
modeling, you will end up with a better final product.
The guidelines for creating a fair surface include:
●
Use the fewest possible control points to get the curve shape.
●
Use the fewest possible curves to get the surface shape.
### Lay out the hull curves
The hull lines were created by tracing the original plans using a background bitmap. The first step is to check
the lines for fairness before creating surfaces from them.
The designer’s lines are illustrated. The sheer and chine have been extended at the forward and aft ends to
accommodate the lofting process.

<!-- page 217 -->

Boat Hull - Loft and Sweep
Start the Model
4
Open the tutorial model file Victory.3dm.
Download the tutorial models.
The lines are laid out on the Plan layer and the Profile layer.
### Check the curves for fairness
Select each of the designer’s curve pairs in plan and profile and use the CurvatureGraph command to
determine if the curves are “fair.” In this case, the file has the original curves that were traced from the
background bitmap. They are not "fair." In other words, the curves do not smoothly transition from one end of
the sheer to the other. If any curve is not fair, adjust points to make it fair. Start with the sheer (the curve at
the top of the hull shape). It has the biggest impact on the appearance of the vessel.
Check curvature
1. Select the curves you want to check.
2. Use the CurvatureGraph command to display its curvature graph.
The illustration shows the curvature graph applied to the two-dimensional sheer in profile.
The curvature graph should be continuous and exhibit the characteristics desired for the curve. When the
curve is concave downward, the graph will be above the curve. Conversely, concave upward curves will have
their graphs below them. The point of inflection (where the curve is neither concave upward nor downward) is
indicated where the graph crosses the curve.
Fix the curvature
Before doing any point editing to make the curves fair, rebuild the curves to remove excess control points.
Select each curve and use the Rebuildcommand to reduce the number of points and set the degree. Do not
use more points than you absolutely need.
Use the CurvatureGraph command to check the curves again for fairness. If the curvature graph is still not
satisfactory, move the control points until you have a smooth graph. Proceed with the rest of the curves in the
model to be certain they are fair before beginning to surface the model.
Rebuild the curves
1. Select the sheer curve.
2. Start the Rebuild command.

<!-- page 218 -->

Boat Hull - Loft and Sweep
3. In the Rebuild Curve dialog box, change the Point count to 6 and the Degree to 5.
### Create the 3-D curves
So far, you have been working with two-dimensional curves. In order to loft the surfaces, these planar curves
will be used to create to three-dimensional curves and the planar curves can be discarded.
With the 3D Lines layer current, select the profile and plan view representations of each curve. Use the
Crv2View command to create the three-dimensional curve that combines the x-, y-, and z-coordinates of the
two-dimensional curves. The two-dimensional curves must be planar for this command to work.

<!-- page 219 -->

Boat Hull - Loft and Sweep
Create the three-dimensional curves
1. Set the 3D Lines layer current.
2. Select the plan and profile representations of the sheer curve.
3. Start the Crv2View command.
The three-dimensional representation of that curve will be created.
4. When you are satisfied that the proper curve was created, delete or Hide the two-dimensional
representations.

<!-- page 220 -->

Boat Hull - Loft and Sweep
5. Repeat the Crv2View command for the chine curve.
### Revise the curves
For the loft process to work on the bottom panel, it cannot come to a point. The lofted shape must be
rectangular. This is why the curves are extended beyond the centerline. The curves can be lofted into a
rectangular surface that can then be trimmed back. The curves in the Victory model are already extended for
you except for the bottom centerline curve.
Copy the centerline curve
We are going to use a copy of the centerline to create a new extended curve for lofting the hull bottom.
1. Copy the centerline using the InPlace option.
2. Hide the centerline.
Shorten the centerline
1. Select the centerline.
2. Start the SubCrv command.

<!-- page 221 -->

Boat Hull - Loft and Sweep
3. At the Start of curve prompt, using the End object snap, click the aft end of the centerline.
4. At the End of curve... prompt, using the Midpoint object snap, click the middle of the curve.
Extend the centerline
1. Start the Extend command, and at the Select boundary objects... prompt, press Enter for
dynamic extend.
2. At the Select curve to extend... prompt, set the Type to Smooth and select the centerline near
the forward end.

<!-- page 222 -->

Boat Hull - Loft and Sweep
3. Draw the curve so it aligns nicely with the chine and sheer curves in the plan view as illustrated.
This creates a new bottom curve to use for the surface loft.

<!-- page 223 -->

Boat Hull - Loft and Sweep
4. Turn on the control points (F10) to check the curve.
### Loft the hull surfaces
Now that you have created a set of edge curves for the side and bottom, create lofted surfaces from these
curves. Start by lofting the bottom surface. Once you have finished it, use its upper edge as the curve from
which to loft the side panel.
To loft the bottom panel, select the two edges (chine and centerline) and use the Loft command. In this case,
be sure to select the new centerline you created in the previous step.
Loft chine and centerline
1. Select the chine and centerline.
2. Start the Loft command.
3. In the Loft Options dialog box, under Cross-section curve options, select Rebuild with..., and
set the control point count to 15, click OK.

<!-- page 224 -->

Boat Hull - Loft and Sweep
Loft the side and bottom
1. Select the surface edge and the sheer curve.
Tip: Hold Ctrl + Shift to select the upper edge of the loft surface.
2. Repeat the Loft for the side panel.

<!-- page 225 -->

Boat Hull - Loft and Sweep
3. In the Loft Options dialog box, under Cross-section curve options, select Rebuild with..., and
set the control point count to 15, click OK.
### Trim the bow and bottom
When you have successfully created both the side and bottom surfaces, construct a buttock one-half inch off
the centerline and trim both surfaces to this buttock. To do this, in the Top viewport, draw a line longer than
the hull and one-half inch to the right of centerline.

<!-- page 226 -->

Boat Hull - Loft and Sweep
Draw a trim line
1. In the Top viewport, draw a Line along the centerline (x-axis) that is longer than the hull.
2. In the Top viewport, offset the line 1/2 inch toward the hull surfaces.
This creates a curve that will be used in the next step. We want a small gap down the center of the
boat between the two halves for a keel.

<!-- page 227 -->

Boat Hull - Loft and Sweep
Trim the side and bottom to the trim line
4
Using the offset curve, Trim the bottom (1) and side (2) as illustrated.
### Build the transom
Like all surfaces in this tutorial, the transom will be built with a surface larger than the finished surface and
then trimmed to the hull.
To make sure there is enough surface area to trim, Extend the transom centerline by a foot or two both above
the sheer and below the centerline. Trim the hull surfaces with the transom centerline.

<!-- page 228 -->

Boat Hull - Loft and Sweep
Extend the centerline
1. With the Profile layer on, start the Extend command.
2. At the Select boundary objects or enter extension length. Press Enter for dynamic
extend prompt, press Enter.
3. At the Select curve to extend… prompt, in the Front viewport, set the Type to Natural, and
select near the top of the transom centerline.
4. At the End of extension prompt, select a point above the current top of the transom centerline.
5. At the next Select curve to extend… prompt, select near the bottom of the transom centerline.
6. At the End of extension prompt, select a point below the current bottom of the transom centerline,
press Enter.

<!-- page 229 -->

Boat Hull - Loft and Sweep
Trim and
Join the hull surfaces
1. Select the transom centerline.
2. Start the Trim command.
3. Set UseApparentIntersections=Yes.
4. In the Front viewport, at the Select object to trim… prompt, select the hull side and bottom
surfaces aft of the transom centerline.
5. Join the hull bottom and side.
Mirror the hull surfaces
In the Right or Top viewport, Mirror the two hull surfaces about the centerline. Use the EdgeSrf
command to create surfaces between the two hull halves.
1. Select the two hull surfaces.
2. Start the Mirror command.
3. In the Top viewport, at the Start of mirror plane… prompt, type 0, press Enter.

<!-- page 230 -->

Boat Hull - Loft and Sweep
4. At the End of mirror plane prompt, with Ortho on, drag the mirror plane along the x-axis, and
click.

<!-- page 231 -->

Boat Hull - Loft and Sweep
Create the keel surface
1. Start the EdgeSrf command.
2. At the Select 2, 3, or 4 curves prompt, select the two inner edges of the hull bottom along the
keel.
3. Repeat the EdgeSrf command.
4. At the Select 2, 3, or 4 curves prompt, select the two inner edges of the hull sides along the keel
at the bow.
Tip: The order you select the surface edges determines what direction the keel sur-
face faces.
Extrude the surface
To create the transom surface, Extrude the transom centerline.

<!-- page 232 -->

Boat Hull - Loft and Sweep
1. In the Front viewport, select the extended transom centerline.
2. Start the ExtrudeCrv command.
3. At the Extrusion distance prompt, set the command-line option BothSides=Yes.
4. In the Perspective, Top, or Right viewport, drag the extension beyond the hull surface.
Trim the transom
Trim the transom surface with the hull and a line from the hull edges.

<!-- page 233 -->

Boat Hull - Loft and Sweep
1. Draw a line between the two aft hull top edges.
2. Start the Trim command.
3. At the Select cutting objects prompt, select all of the hull surfaces, including the keel surface and
the line at the top of the hull, press Enter.
4. At the Select object to trim… prompt, select the transom surface outside of the hull lines and
surfaces, press Enter.
The transom is now complete.

<!-- page 234 -->

Boat Hull - Loft and Sweep
Check for errors
1. Join all of the surfaces.
2. Use the ShowEdges command to check that the join was successful.
Display the naked edges. Naked edges are surface edges that are not joined to other surfaces. In this
case, the only naked edges should be the ones you expect around the outside of the hull surfaces –
not those between the surfaces.
When you have your surfaces built and joined, and have no unjoined edges, look at the surface with
the curvature analysis tools.
### Add the deck
The last step is to create the deck surface. In the profile lines, two curves describe the silhouette of the deck
curve. You will use this curve to create the deck.
Use the Project command to project the vertical line to the side of the hull. This line will act as a marker for the
end of the curve. In the Front viewport, draw a curve from the end of the deck centerline curve to the end of
the projected curve on one side of the hull. Use Planar mode to keep the curve planar. Place the first three
control points using Ortho to keep them lined up at the center.

<!-- page 235 -->

Boat Hull - Loft and Sweep
Project the vertical deck edge to the hull
1. Select the hull and the vertical line.
2. In the Front viewport, use the Project command to project the curve to the hull.
The curve will project to both sides of the hull, so you can draw your cross-section curve on either
side.

<!-- page 236 -->

Boat Hull - Loft and Sweep
Draw the cross-section curve
1. Click the Planar pane in the status bar to turn on Planar mode.
2. In the Right viewport, use the Curve command to draw a control point curve from the top end of the
deck centerline curve to the top of the curve projected to the hull.
Use Ortho to place the first three control points in a straight line.
Use the End object snap to place the last point at the top of the projected curve on the hull.
3. Use the CurvatureGraph command to check the curve.

<!-- page 237 -->

Boat Hull - Loft and Sweep
Create the deck surface
1. Use the Sweep2 command to create the deck surface.
2. At the Select rail curves prompts, select the centerline curve and the hull edge.
3. At the Select cross section curves… prompt, select the cross-section curve you created from the
deck centerline curve to the projected curve on the hull, press Enter.

<!-- page 238 -->

Boat Hull - Loft and Sweep
Mirror the deck
1. Use the Mirror command to copy the deck surface to the other side.
At the Start of mirror plane… prompt, in the Top viewport, type 0, press Enter.
2. At the End of mirror plane… prompt, in the Top viewport, drag the mirror plane with Ortho on.
Fill in the missing surface
4
Use the EdgeSrf command to create the small triangular surface at the tip of the bow.

<!-- page 239 -->

Boat Hull - Loft and Sweep
Join the parts
4
Select all of the surfaces and use the Join command to create a single polysurface.

<!-- page 240 -->
