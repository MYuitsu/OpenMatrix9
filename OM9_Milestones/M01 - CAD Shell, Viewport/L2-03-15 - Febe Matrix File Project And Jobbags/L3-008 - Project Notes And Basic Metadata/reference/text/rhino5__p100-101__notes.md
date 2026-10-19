---
source_pdf: h:\OpenMatrix9\manuals\rhino5.pdf
source_key: rhino5
page_range: 100-101
section_title: Notes
section_id: rhino5-071-notes
---
# Notes

Source: rhino5.pdf, pages 100-101

<!-- section rhino5-071-notes | Notes | rhino5 pages 100-101 -->

<!-- page 100 -->

Organization and Annotation
### Dots
The Dot command places a text dot.
Dots are always parallel to the view. Dots are displayed in the layer color. Dot size is constant on the screen. As
you zoom in and out, the dot displays the same size.
### Hidden line removal
The Make2D command creates curves from the selected objects as silhouettes relative to the active view. The
silhouette curves are projected flat and then placed on the world x,y-plane.
The command options create the 2-D drawing from the current view, current construction plane, create a four-
view layout using US or European projection angles, set layers for the hidden lines, and display tangent edges.
### Notes
The Notes command provides a means of storing text information in your model file. You can type information
directly into the Notes text box. If you leave the Notes box displayed when you close the model file, it will
display the next time the file is opened.

<!-- page 101 -->

## Render
In addition to shaded previews, Rhino provides full-color rendering with lights, transparency, shadows,
textures, and bump mapping.
Objects will render white until you add render color, highlight, texture, transparency, and bumps. These
attributes are controlled through the Properties panel, Material page.
The process needed to render scenes consists of four basic steps:
●
Add lighting
●
Assign materials
●
Set up the environment and ground plane
●
Render
Although the steps do not have to be done in this order, using this method seems to make setting up a scene
more efficient. To improve quality, repeat these steps until the image looks right to you.
### Lights
In every Rhino rendering there are light sources that Rhino uses to calculate how the objects are to be
illuminated. If you do not add any light sources to your scene, the default light is used. The default light is a
directional light with parallel rays that acts as though you have a lamp shining over your left shoulder.
