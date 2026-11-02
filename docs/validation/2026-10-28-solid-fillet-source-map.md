# Fillet Edge: bounded mapping and next implementation

The recovered `modMillC_Defaults.bas` procedure `Proc_198_35_1A2F840`
associates `SolidFilletEdgeFilletEdge` near `01A35594` with external
`_FilletEdge` near `01A3557C` and its previous-edge alternate near
`01A35567`. The same-address Ghidra procedure is `FUN_01a2f840`.
Registration proves identity and the existence of the alternate binding,
not the external fillet implementation, default radius or history policy.
No recovered code or artwork is reproduced here.

The normalized M03 reference describes selecting edges, a NextRadius option,
Enter, then editing fillet handles and Enter; joined open or closed
polysurfaces are allowed. Current standalone accepts a preselected BRep
and numeric edge index/radius, immediately commits one constant-radius
OCCT solid-only operation. The kernel can accept multiple indices but the
command exposes only one. Viewport edge selection, open polysurfaces, handle
preview/edit, previous-edge sets, chain options, alternate binding and
actual desktop/human UI are not established. Implement and test these
incrementally without claiming the numeric path satisfies the full command.

The later checkpoint adds staged viewport edge picking and constant-radius
preview for one closed BRep; the numeric path remains available. It does not
resolve the original default, editable handles, variable radii, open shells or
the other features listed above.

The following checkpoint adds tested distinct constant radii on independent
edges. Variable radius along a single tangent contour and handle editing
remain open.

A later checkpoint supports a tested internal edge of a joined open box shell,
while rejecting naked boundary edges. General open-shell topology remains
unverified.

The subsequent checkpoint exposes one sampled handle per picked edge and
edits that edge's constant radius via CurrentRadius. This is not the full
documented handle drag or variable-radius workflow.

The following checkpoint exposes exact oriented endpoint handles and linear
endpoint-radius interpolation for tested independent edges; multi-edge tangent
contours and handle dragging remain open.

The latest kernel checkpoint supports matching vertex radii on a fully
selected open tangent contour, tested on two straight segments. It does not
resolve shared-handle editing, curved or closed contours, or handle dragging.

The subsequent shared-vertex checkpoint synchronizes endpoint radii for a
tested two-edge tangent contour, using native topology and contour identity.
General curves, handle dragging and actual desktop picking remain open.

The latest kernel checkpoint supplies interior radius constraints to OCCT on
fully selected open contours. It does not yet provide the documented AddHandle
interaction or viewport projection.

The subsequent application checkpoint connects AddHandle and viewport edge
projection for a tested translated straight-edge fixture. Default radius,
general curved/closed edges, dragging and the other handle options remain open.

The later RemoveHandle checkpoint supports removal of added controls in the
preview interaction. General curved/closed edges and desktop remain open.

The later CopyHandle checkpoint handles source selection and copies to tested
new or existing destinations. Dragging, general curves and desktop remain open.
