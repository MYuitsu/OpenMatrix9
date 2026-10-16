# Smart Flow Rigid ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix_8_book_2.pdf
- Page range: 18-18
- Section title: Smart Flow Rigid ￼
- Section id: m8book2-014-smart-flow-rigid

---

<!-- section m8book2-014-smart-flow-rigid | Smart Flow Rigid ￼ | m8book2 pages 18-18 -->

<!-- page 18 -->

### Related Commands:
## Smart Flow Rigid
These are a list of tools that can assist you when
working with Smart Flow.
‘GvSmartFlowRigid’
This toggles a Flow object to a Rigid state. Smart
Flow Rigid finds the center of an object and flows
the object to the Flow surface based on that
point without distorting the object. Use when the
objects on the Base surface should not distort
on the Target surface. By default, gems (block
instances) flow to the Target surface in a rigid
state.
Where is this Command:
Access this command from Transform>Smart
Flow Rigid
Steps:
• Select Smart Flow Rigid from the Transform menu.
• Select objects on the Base surface (flat surface) to
change to a Rigid state.
Direction ‘Dir’: When using Smart Flow, objects
‘flow’ to the Destination surface based on the
direction of the Base and Destination surfaces. If
the relationship between UV and Surface Normal
don’t match between the Base and Destination
surface objects will flow to the  Destination
surface with unpredictable and often undesirable
results. Use the Direction command to align the
Destination surface to the Base surface. See the
Utilities section of the manual for more information
on how to use the Direction command.
Before Smart Flow Rigid the bezels on the Flow
surface are distorted and break away from the
girdles of the gems.
Detect Smart Flow: This shade mode will
display Surface Normal direction as well as UV
direction on surfaces. Surface Normals that point
down (a Backface) will display red otherwise
they display by their layer color. The U direction
is indicated by red lines on the surface and the V
direction by blue.
In this example the UV and directions are aligned
between the Base and Destination surfaces, but
the Destination’s Surface Normal is flipped. Objects
added to the top of the base surface will appear on
the under side of the destination surface.
After Smart Flow Rigid the bezels on the Flow
surface are no longer distorted and follow the Base
surface bezels.
