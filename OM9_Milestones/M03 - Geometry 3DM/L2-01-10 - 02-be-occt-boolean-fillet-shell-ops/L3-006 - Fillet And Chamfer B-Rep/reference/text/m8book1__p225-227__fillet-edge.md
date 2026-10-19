# Fillet Edge ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 225-227
- Section title: Fillet Edge ￼
- Section id: m8book1-246-fillet-edge
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p225_img01.png, m8book1_p225_img02.png, m8book1_p225_img03.png, m8book1_p225_img04.png, m8book1_p226_img01.png, m8book1_p227_img01.png

---
<!-- section m8book1-246-fillet-edge | Fillet Edge ￼ | m8book1 pages 225-227 -->

<!-- page 225 -->

## Extract Surface
Outputlayer = Current: Current places the
results (the extracted surface) on the current layer.
‘ExtractSrf’
Use this command to separate a component
surface from a polysurface, creating a single
surface and an open polysurface.
With the
“OutputLayer=Current”
option, the extracted surface
will appear on the current
Layer color (Metal 01).
Where is this Command:
Access this command from Solid>Extract
Surface
Outputlayer = Input: Places the results (the
extracted surface) on the same layer as the input
curve.
Or, right-click the command to activate the
Copy=Yes option.
Steps:
## Fillet Edge
‘FilletEdge’
Use this tool to fillet (add tangent curvature to)
edges in a polysurface.
• Select Extract Surface from the Solid menu.
• Select surfaces or polysurfaces to extract.
• Press Enter when done.
• The extracted surface remains in the same place
until it is deleted or moved by you.
Where is this Command:
Access this command from Solid>Fillet Edge
Right-Click Command: Fillet Edge - Previous
Edge
Steps:
Using Extract Surface to extract the surface on the
shank end.
• Select Fillet Edge from the Solid menu.
• Select edges to Fillet. (Choose a ‘NextRadius’
amount before selecting the edge or use the
default amount shown.)
• Press Enter when done.
• Select Fillet handle to edit.
• Press Enter when done.
Note: Moving a handle at the end of the edge will
cause the fillet to extend beyond the surface. This
will have to be trimmed by other means.
Note: The polysurface may be open or closed, but
component surfaces must be joined.
Note:
• The surfaces are separated from the polysurfaces.
The remainder of each polysurface will stay joined.
• The extracted surface is also removed from any
groups to which the original surface may belong.
• To remove a surface from the selection, press the
‘Ctrl’ key while clicking.
• When extracting a surface from a polysurface this
command can save time. It only lets you extract
the selected surfaces from the polysurface
instead of using Explode and having to join
everything together again.
### Command Line Options:
Fillet before (left) and after (right).
Copy=Yes/No: Specifies whether or not the
objects (the surfaces being extracted) are copied.
A plus sign appears at the cursor when copy
mode is on.

<!-- page 226 -->

curves is less than this value, the chain selection
will ignore the gap and will select the next
segment.
Radius amount
is shown on
the handle
when Shows
Radius=Yes.
AngleTolerance: When Continuity is set to
Tangency, if the angle between two edges or
curves is less than this value, the chain selection
will consider the criteria for continuity met and will
select the next segment.
Undo: Undo last segment selection.
### Command Line Options:
Next: Select next segment.
ShowRadius: Controls the display of the current
radius in the viewport
All: Select all segments.
NextRadius: Specifies the radius for the next
handle.
### Options Continued:
### Chain Edges Options:
PreviousEdgeSelection: In cases where the
command is canceled or ended prematurely, re-
selects the previously selected edges. Supports
multiple sets of previously selected edges for up
to 20 previous edge sets.
AutoChain Yes/No: Selecting a curve or a
surface edge automatically selects all curve
segments connected with the level of continuity
set by the Chain Continuity option.
### Radius/Distance options
Chain Edges Steps:
Radius and Distance options appear on the
Command Line when you drag a handle grip.
FromCurve: Select a curve. The radius of the
curve at the picked location will be used.
• Select Chain Edges option in the Command Line.
• Select first chain segment. (If Auto Chain is
enabled the chain of edges will auto select.)
• Select next chain segment. (If Auto Chain is set to
‘No’.)
• Press Enter when done.
FromTwoPoints: Pick two points to show the
radius distance.
### Handle Options
AddHandle: Adds a handle along the edges.
Add Handle Steps:
ChainContinuity: Controls the level of continuity
(Position, Tangency or Curvature) required
between segments to be selected with the Auto
Chain option. With Position the curves are position
continuous (G0) and touch at the ends. Tangency;
the curves touch at the ends and go the same
direction a the point where they touch (G1).
Curvature continuity measures position, direction
and radius of curvature at the ends. They go the
same direction and have the same radius (G2).
• Enter a new Radius amount in the Command Line
by clicking on ‘CurrentRadius’.
• Select new fillet handle location. Drag your cursor
to the desired location. Left-click to set the
location. Repeat as many times as desired.
• Press Enter when done.
Direction: ‘Forward’ selects curves in the positive
curve direction. ‘Backwards’ selects curves in the
negative curve direction. ‘Both’ selects curves in
both the positive and negative curve direction.
CopyHandle: Adds a new handle using the
distance from the selected handle.
GapTolerance: If the gap between two edges or

<!-- page 227 -->

Copy Handle Steps:
TrimAndJoin: Trims and joins the resulting
surface to the input surfaces.
## Text
• Select Fillet handle to copy.
• Select new handle location or an existing handle.
Drag your cursor to the desired location. Left-
click to set the location. Repeat as many times as
desired.
• Press Enter when done.
‘TextObject’
Draws text-shaped curves, surfaces or
polysurfaces based on True Type fonts.
Where is this Command:
Access this command from Solid>Text
RemoveHandle: Visible only when at least one
handle has been added. Select the Fillet handle
to remove. Click on the handle(s) to remove and
press enter when done.
Steps:
SetAll: Sets the distance or radius for all handles.
LinkHandles Yes/No: Editing a single handle
updates all handles.
• Select Text from the Solid menu.
• Specify the options in the text object window.
Enter the text to create, select the font, select if
curve, surfaces or solids, etc.
• Pick a location in the viewport to place the Text.
Note:
• Only added handles can be removed.
• The default handles at the ends of each open
edge segment cannot be moved or deleted. This
is the minimum information the command needs
in order to work.
•  The handle at the end of a single closed edge
can be moved but not deleted.
### RailType options
DistFromEdge: The distance from the edge
curves determines the intersection,
Rolling Ball: The radius of a rolling ball
determines the intersection (default).
DistBetweenRails: The distance between the
edge rails determines the intersection.
### Handle Options Continued:
SelectEdges: Allows selecting more edges after
the selection has been closed.
### Options:
Text to Create: Type the desired text  in this
window. Right click in the edit area to cut, copy
and paste text.
Preview Yes/No: ‘Yes’ displays a dynamic
preview. You can change the options at this
point and the preview will update. ‘No’ means no
preview will display. There is no opportunity to
change the options.
Font: Selects the font and style.
