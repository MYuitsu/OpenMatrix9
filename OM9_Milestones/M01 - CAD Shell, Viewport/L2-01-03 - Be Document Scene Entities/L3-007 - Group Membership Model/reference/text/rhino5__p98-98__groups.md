---
source_pdf: h:\OpenMatrix9\manuals\rhino5.pdf
source_key: rhino5
page_range: 98-98
section_title: Groups
section_id: rhino5-063-groups
---
# Groups

Source: rhino5.pdf, pages 98-98

<!-- section rhino5-063-groups | Groups | rhino5 pages 98-98 -->

<!-- page 98 -->

Organization and Annotation
### Groups
A group is a collection of objects that select as one for moving, copying, rotating, or other transforms and
applying properties such as object color. Grouping objects assigns a group name to each object that is displayed
as a part of its properties. Objects with the same group name belong to the same group.
●
Group groups objects for selection. A group can contain one or more sub-groups.
●
Ungroup destroys the group.
●
SetGroupName changes the name assigned by default. Naming different groups to the same name
combines those groups into one.
●
AddToGroup and RemoveFromGroup add and remove objects from groups.
●
SelGroup selects groups by name.
### Blocks
A block is another way of associating objects together to form a single object. The Block command creates a
block definition in the current model. The Insert command places instances of this block definition in your
model. You can scale, copy, rotate, array, and otherwise transform block instances in the model. If you edit the
block definition, all instances of the block are changed to this new definition. Blocks can streamline modeling,
reduce model size, and promote standardization of parts and details.
Multiple instances of a block can be located, scaled, and rotated into a model with the Insert command. Block
definitions are created with the Block or Insert command. Materials and other object properties on block
instances are determined by the component objects.
Exploding a block instance places the block geometry using the instance location, scale, and rotation. To
redefine a block, use the BlockEdit command, to edit the geometry.
The BlockManager command displays a dialog box that lists all the block definitions in the model. Use the
Block Manager dialog box to view block properties, export a block definition to a file, delete a block definition
and all its instances, update a block definition from a file, find out what blocks are nested in other blocks, and
count the number of block instances in the model.
### Worksessions
The Worksession command lets many users work on a large project by managing many files. Each user can
edit a different file in the project and at the same time see the related portions of the project. By refreshing as
needed, each user can see the current version of the related files in the projects. Only one user can have a file
open for editing, but many users can see it.
Rhino worksessions let you attach external files to your current work environment. Attached geometry cannot
be edited (Move, Scale), but it can be used for input to creation commands (Copy, ExtrudeCrv).
### Dimensions
You can dimension objects in your model, with your choice of font, units display, decimal precision, text and
arrow size, and text alignment. After dimensions are placed, you can select all dimensions, edit dimension text,
turn control points on to move dimension elements, and delete dimensions. You can place horizontal, vertical,
aligned, rotated, radial, diameter, and angle dimensions, text blocks, leaders, and create a 2-D hidden line
drawing.
Dimensions are not associative. Changing your geometry will not update the dimension unless the dimension
was drawn with history enabled. Changing the dimension will not update your geometry.
The Dim command places horizontal and vertical dimensions depending on the direction you pick the points.
