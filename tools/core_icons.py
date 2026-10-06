"""Authored core/sidebar SVGs. Geometry is independent of reference bitmaps."""
import argparse
import configparser
import hashlib
import html
import json
from pathlib import Path
import re
from curve_icons import path, points, arrow, WHITE

BLUE = '#80A9E8'
LIGHT = '#D5E7FF'
DEEP = '#315EA8'
TAN = '#D8BF91'
PINK = '#EC45C4'
GOLD = '#FFD705'
RED = '#FF4747'
GREEN = '#25BC69'
STYLE = 'minimal-core-v1'


def face(d, color=BLUE, fill=None, opacity=.16, width=2):
    return path(d, color, width=width).replace('fill="none"',
        f'fill="{fill or color}" fill-opacity="{opacity}"')


def circle(x=16, y=16, r=10, color=BLUE, width=2.2):
    return f'<circle cx="{x}" cy="{y}" r="{r}" fill="none" stroke="{color}" stroke-width="{width}"/>'


def page(color=BLUE):
    return face('M6 3 H20 L27 10 V29 H6 Z',color,WHITE,.06)+path('M20 3 V10 H27',WHITE,width=1.6)


def save():
    return face('M4 3 H24 L29 8 V29 H4 Z')+face('M9 3 H22 V12 H9 Z',WHITE,WHITE,.15,1.6)+face('M9 20 H24 V29 H9 Z',WHITE,WHITE,.1,1.6)


def folder():
    return face('M3 9 H12 L16 13 H29 L25 27 H3 Z')+path('M3 16 H27',WHITE,width=1.5)


def cube(color=BLUE):
    return face('M16 4 L28 10 V23 L16 29 L4 23 V10 Z',color)+path('M4 10 L16 16 L28 10 M16 16 V29',WHITE,width=1.6)


def image():
    return face('M3 5 H29 V27 H3 Z')+path('M5 24 L12 16 L17 21 L24 12 L27 18',WHITE,width=1.6)+points((10,11),color=WHITE,radius=2)


def plus(color=WHITE):
    return path('M24 19 V29 M19 24 H29',color,width=2.2)


def pencil():
    return face('M17 22 L26 13 L29 16 L20 25 L16 26 Z',WHITE,WHITE,.12,1.8)+path('M24 15 L27 18',BLUE,width=1.5)


def check():
    return path('M8 17 L13 23 L26 8',WHITE,width=2.8)


def cross(color=WHITE):
    return path('M9 9 L23 23 M23 9 L9 23',color,width=2.7)


def corners(color=WHITE):
    return path('M3 10 V3 H10 M22 3 H29 V10 M29 22 V29 H22 M10 29 H3 V22',color,width=1.7)


def lens():
    return circle(13,13,8)+path('M19 19 L29 29',WHITE,width=2.6)


def ring(color=BLUE):
    return circle(16,18,10,color)+face('M10 7 L13 3 H19 L22 7 L16 13 Z',color, color,.2,1.5)


def mesh(color=TAN):
    return face('M4 7 L25 4 L28 26 L7 29 Z',color)+path('M4 7 L28 26 M25 4 L7 29 M6 18 L26 15',WHITE,width=1.4)


def patch(color=BLUE):
    return face('M4 8 Q15 3 28 8 L25 27 Q15 23 7 28 Z',color)+path('M6 18 Q16 13 26 18',LIGHT,width=1.4)


def grid(color=BLUE):
    return face('M4 4 H28 V28 H4 Z',color)+path('M12 4 V28 M20 4 V28 M4 12 H28 M4 20 H28',WHITE,width=1.4)


def badge(geometry, symbol, color=WHITE):
    return '<g transform="scale(.8)">'+geometry+'</g>'+path(symbol,color,width=2.1)


def dimension(a=(6,22), b=(26,22), color=WHITE):
    x,y=a;u,v=b
    dx,dy=u-x,v-y;length=(dx*dx+dy*dy)**.5;nx,ny=-dy/length*4,dx/length*4
    return path(f'M{x-nx} {y-ny} L{x+nx} {y+ny} M{u-nx} {v-ny} L{u+nx} {v+ny} M{x} {y} L{u} {v}',color,width=1.8)+points(a,b,color=BLUE,radius=2)


def letter(text,x=16,y=12,color=WHITE,size=10):
    return f'<text x="{x}" y="{y}" text-anchor="middle" font-family="sans-serif" font-size="{size}" font-weight="bold" fill="{color}">{html.escape(text)}</text>'


def gear():
    return face('M13 3 H19 L20 8 L24 10 L28 9 L30 15 L26 18 L25 23 L27 26 L21 29 L17 26 L12 25 L8 27 L5 21 L8 17 L7 12 L3 9 L7 4 L11 6 Z',WHITE,BLUE,.12,1.8)+circle(16,16,5,BLUE,2)


def clock():
    return circle(16,16,11)+path('M16 8 V16 L23 20',WHITE,width=2)


CORE_ICONS = {}


def add(key,label,meaning,geometry,spec=None,note='Functional geometry; reference color family preserved.'):
    CORE_ICONS[key]=(label,meaning,geometry,spec,note)


# File: no brand logos and no manufacturing-service links are embedded.
add('FileNew','New','Create a document',page(), 'OM9-FILE-002')
add('FileOpen','Open','Open a saved document',folder(),'OM9-FILE-003')
add('FileExportSelected','Export Selected','Export selected objects',page()+arrow('M14 19 H29','M25 15 L29 19 L25 23'),'OM9-FILE-001')
add('FileImport','Import','Import objects into the document',page()+arrow('M29 19 H14','M18 15 L14 19 L18 23'),'OM9-FILE-004')
add('FileSave','Save','Save the current document',save(),'OM9-FILE-005')
add('FileSaveAs','Save As','Save under a chosen filename',save()+pencil(),'OM9-FILE-006')
add('FileSaveSmall','Save Small','Save without cached render meshes',badge(save(),'M23 18 V29 M19 24 L23 29 L27 24'),'OM9-FILE-007')
add('FileSaveSmallAs','Save Small As','Save under a chosen filename without cached render meshes',badge(save(),'M20 22 L25 17 L29 21 L24 26 L19 27 Z'),'OM9-FILE-010')
add('FileUserLibrary','User Library','Open the user object library',folder()+circle(14,12,3,WHITE)+path('M8 22 Q8 16 14 16 Q20 16 20 22',WHITE,width=1.6),None,'Library/person reference cue; dedicated local command mapping absent.')
add('FileUserLibraryAdd','Add to User Library','Add an object to the user library',badge(folder(),'M24 18 V29 M19 24 H29'),None,'Library/add reference cue; dedicated local command mapping absent.')
add('FileStullerSubmit','Manufacturing Service','Access the external manufacturing service workflow',circle(13,16,10,WHITE)+path('M13 6 Q4 16 13 26 Q22 16 13 6 M3 16 H23',BLUE,width=1.6)+arrow('M21 9 H29 V3','M25 7 L29 3 L30 8'),'OM9-FILE-011','Neutral globe/outgoing cue replaces the proprietary S logo; no service connection implemented.')
add('FileNotes','Notes','Edit document notes',page()+path('M10 14 H22 M10 19 H22 M10 24 H18',WHITE,width=1.6),'OM9-FILE-008')
add('FilePrint','Print','Print document views',face('M3 12 H29 V25 H3 Z')+face('M8 3 H24 V13 H8 Z',WHITE,WHITE,.1,1.6)+face('M8 22 H24 V29 H8 Z',WHITE,WHITE,.1,1.6),'OM9-FILE-009')

# View: image import, capture, layout and zoom remain visibly different.
add('ViewBackgroundBitmapPictureFrame','Picture Frame','Place an image plane',image()+corners(),'OM9-VIEW-001')
four=face('M4 4 H28 V28 H4 Z')+path('M16 4 V28 M4 16 H28',WHITE,width=1.8)
add('ViewRestoreViewports','Restore Viewports','Restore the viewport layout',four,'OM9-VIEW-002')
add('OthersViewSynchronizeViews','Synchronize Views','Synchronize view settings',four+path('M7 9 H12 M20 9 H25 M7 23 H12 M20 23 H25',WHITE,width=2),'OM9-VIEW-003')
add('ViewZoomZoomDynamic','Dynamic Zoom','Interactively adjust magnification',lens()+path('M9 13 H17 M13 9 V17',WHITE,width=1.7),'OM9-VIEW-004')
add('ViewZoomZoomExtents','Zoom Extents','Fit object extents in the view',circle(16,16,7)+corners(),'OM9-VIEW-006')
add('ViewZoomZoomSelected','Zoom Selected','Fit selected objects in the view',lens()+face('M8 8 H18 V18 H8 Z',WHITE,WHITE,.1,1.4),'OM9-VIEW-007')
add('ViewZoomZoomWindow','Zoom Window','Magnify a chosen rectangular region',face('M3 3 H26 V23 H3 Z',WHITE,WHITE,.05,1.5)+lens(),'OM9-VIEW-008')
add('ViewManager','View Manager','Manage saved views',face('M3 5 H23 V23 H3 Z')+face('M9 11 H29 V29 H9 Z')+path('M9 17 H29 M16 17 V29 M23 17 V29',WHITE,width=1.5),None,'Stacked viewport reference cue; detailed local command mapping absent.')
add('ViewCaptureToFile','Capture to File','Save a viewport image',image()+badge('', 'M24 18 V29 M19 24 L24 29 L29 24'),'OM9-VIEW-005')
add('OthersSetCrosshairs','Crosshairs','Toggle crosshairs in the view',path('M16 3 V29 M3 16 H29',WHITE,width=2)+circle(16,16,3),'OM9-VIEW-009')
add('ViewZoomZoom1To1','Zoom 1:1','Set the calibrated actual-size view',lens()+letter('1:1',13,16,size=7),'OM9-VIEW-011')
add('ViewZoomZoom1To1Calibrate','Calibrate 1:1','Calibrate screen scale for actual-size viewing',face('M3 5 H29 V24 H3 Z')+letter('1:1',16,17,size=10)+path('M8 29 H24 M8 27 V30 M24 27 V30',WHITE,width=1.5),'OM9-VIEW-013')
add('ViewSetCameraCenterViewport','Center Viewport','Center the view using a target',circle(16,16,10)+path('M16 3 V11 M16 21 V29 M3 16 H11 M21 16 H29',WHITE,width=1.8)+points((16,16),color=RED),'OM9-VIEW-012')
add('OthersSystem6HD3View','Three-view Image','Load a three-view camera image for modeling',image()+path('M12 5 V27 M21 5 V27',WHITE,width=1.7),'OM9-VIEW-010')
add('ClayooCreationBlueprint','Blueprint','Prepare modeling reference images',image()+path('M6 9 H25 M8 7 V24 M23 7 V24',RED,width=1.5),'OM9-MATRIXTOOLS-007')

# Utilities: face/edge diagnostics, grouping and mesh operations.
add('AnalyzeDirection','Direction','Inspect object directions',patch()+arrow('M10 21 V8','M6 12 L10 8 L14 12')+arrow('M23 21 V8','M19 12 L23 8 L27 12'),'OM9-UTIL-001')
add('AnalyzeDiagnosticsSelectBadObjects','Select Bad Objects','Select invalid objects',badge(patch(),'M20 19 L29 28 M29 19 L20 28',WHITE)+corners(),'OM9-UTIL-003')
add('AnalyzeDiagnosticsCheck','Check Objects','Check object validity',check(),'OM9-UTIL-002')
add('AnalyzeEdgeToolsShowEdges','Show Edges','Inspect object boundaries',cube()+path('M4 10 L4 23 L16 29 L28 23',WHITE,width=3),'OM9-UTIL-004')
add('AnalyzeEdgeToolsSplitEdge','Split Edge','Divide an edge at a chosen point',patch()+path('M4 8 H13 M19 8 H28',WHITE,width=2.8)+points((16,8),color=WHITE),'OM9-UTIL-006')
add('AnalyzeEdgeToolsMergeEdge','Merge Edge','Merge adjacent edge segments',patch()+arrow('M6 4 H13','M10 2 L13 4 L10 6')+arrow('M26 4 H19','M22 2 L19 4 L22 6'),'OM9-UTIL-005')
add('AnalyzeEdgeToolsJoin2NakedEdges','Join Naked Edges','Join two matching open boundaries',face('M3 7 H12 V27 H3 Z')+face('M20 7 H29 V27 H20 Z')+path('M12 8 L20 8 M12 26 L20 26',WHITE,width=2),'OM9-UTIL-007')
add('AnalyzeEdgeToolsUnjoinEdge','Unjoin Edge','Separate joined boundaries',face('M3 7 H12 V27 H3 Z')+face('M20 7 H29 V27 H20 Z')+path('M14 11 L18 15 L14 19 L18 23',WHITE,width=1.6),'OM9-UTIL-009')
add('DimensionMake2DDrawing','Make 2D Drawing','Generate a flat drawing from spatial geometry',badge(cube(),'M22 19 V28 H30 M22 28 L29 21'),'OM9-UTIL-008')
add('AnalyzeBoundingBox','Bounding Box','Create a bounding volume',cube()+corners(),'OM9-UTIL-010')
add('AnalyzeCenterPoint','Center Point','Find the center of an object',patch()+path('M16 9 V25 M8 17 H25',WHITE,width=1.6)+points((16,17),color=WHITE),'OM9-UTIL-011')
grouped=face('M5 5 H17 V17 H5 Z',WHITE)+face('M15 15 H27 V27 H15 Z',WHITE)
add('EditGroupsGroup','Group','Group selected objects',grouped+corners(),'OM9-UTIL-012')
add('EditGroupsUnGroup','Ungroup','Release objects from their group',grouped+path('M5 26 L10 21 M22 10 L27 5',BLUE,width=2.5),'OM9-UTIL-015')
add('OthersExtractBadSurface','Extract Bad Surface','Separate invalid surfaces for repair',patch()+face('M18 3 H29 V13 H18 Z',WHITE)+cross(BLUE),'OM9-UTIL-013')
add('ViewShowZBuffer','Z Buffer','Display view-depth information',face('M3 5 H29 V27 H3 Z',WHITE)+face('M6 8 H12 V24 H6 Z',LIGHT,LIGHT,.6)+face('M13 8 H19 V24 H13 Z',BLUE,BLUE,.6)+face('M20 8 H26 V24 H20 Z',DEEP,DEEP,.6),'OM9-UTIL-014')
hole=mesh()+face('M12 12 L20 11 L21 20 L13 21 Z',WHITE,DEEP,.8,1.5)
add('MeshMeshRepairToolsFillHole','Fill Hole','Fill a selected mesh opening',hole+path('M16 13 V19 M13 16 H19',WHITE,width=1.4),'OM9-UTIL-017')
add('MeshMeshRepairToolsFillHoles','Fill Holes','Fill mesh openings',hole+points((7,10),(24,23),color=WHITE,radius=2.4),'OM9-UTIL-018')
add('MeshMeshRepairToolsAlignMeshVertices','Align Mesh Vertices','Align mesh vertices',mesh()+arrow('M3 16 H12','M9 13 L12 16 L9 19')+arrow('M29 16 H20','M23 13 L20 16 L23 19'),'OM9-UTIL-016')
add('MeshFromNURBSObject','Mesh from Surface','Generate a mesh from surface geometry',patch(WHITE)+path('M7 8 L25 27 M27 8 L7 27 M6 18 L26 18',TAN,width=1.8),'OM9-UTIL-019')
add('MeshMeshEditToolsCollapseReduceVertexCount','Reduce Mesh','Reduce mesh complexity',badge(mesh(),'M20 23 H29'),'OM9-UTIL-021')
add('MeshApplyMeshUVN','Apply Mesh UVN','Fit mesh geometry onto a target surface',patch(TAN)+path('M10 7 Q13 17 13 26 M20 6 Q23 17 22 26 M6 14 Q15 10 27 14 M6 21 Q16 17 26 21',WHITE,width=1.5),'OM9-UTIL-020')
add('MeshMeshEditToolsExtractRenderMesh','Extract Render Mesh','Extract the object display mesh',badge(cube(TAN),'M19 19 L28 19 L29 29 L20 29 Z M19 19 L29 29'),'OM9-UTIL-022')

# Measurement: differences are axis, reference arc and text/edit cues.
angle=path('M5 26 H29 M5 26 L24 5',WHITE,width=2)+path('M17 26 Q17 19 13 17',BLUE,width=2)
add('AnalyzeAngle','Angle','Measure an angle',angle+letter('A',8,12,size=9),'OM9-MEASURE-001')
add('AnalyzeDistance','Distance','Measure point-to-point distance',dimension()+letter('D',16,13),'OM9-MEASURE-002')
add('AnalyzeLength','Length','Measure curve length',path('M4 24 Q9 4 18 14 T28 8',BLUE,width=2.5)+letter('L',8,10),'OM9-MEASURE-003')
rad=path('M5 27 A11 11 0 0 1 27 27 M16 27 L24 9',WHITE,width=1.9)+points((16,27),color=BLUE)
add('AnalyzeRadius','Radius','Measure arc or circle radius',rad+letter('R',25,8,size=8),'OM9-MEASURE-005')
add('OthersMeasureDimHorizontal','Horizontal Dimension','Create a horizontal dimension',dimension()+path('M6 11 V26 M26 15 V26',BLUE,width=1.5),'OM9-MEASURE-004')
add('OthersMeasureDimVertical','Vertical Dimension','Create a vertical dimension',dimension((22,6),(22,26))+path('M10 6 H26 M14 26 H26',BLUE,width=1.5),'OM9-MEASURE-006')
add('DimensionAngleDimension','Angular Dimension','Create an angular annotation',angle+letter('30',19,17,size=7),'OM9-MEASURE-007')
add('DimensionRotatedDimension','Rotated Dimension','Create a dimension at a specified rotation',dimension((7,24),(25,7))+path('M3 29 H15 M3 29 V17',BLUE,width=1.5),'OM9-MEASURE-008')
add('DimensionAlignedDimension','Aligned Dimension','Annotate along two chosen points',dimension((7,24),(25,7))+path('M3 20 L11 28 M21 3 L29 11',BLUE,width=1.5),'OM9-MEASURE-009')
add('DimensionDiameterDimension','Diameter Dimension','Annotate a circle diameter',circle(16,17,10)+dimension((8,25),(24,9))+letter('Ø',6,7,size=8),'OM9-MEASURE-010')
add('DimensionRadialDimension','Radial Dimension','Annotate a circle radius',rad+letter('R',25,8,size=8)+path('M16 27 H29',BLUE,width=1.5),'OM9-MEASURE-011')
add('DimensionLeader','Leader','Create a leader annotation',arrow('M28 7 H18 L5 26','M5 21 V26 H10'),'OM9-MEASURE-012')
text=path('M5 6 H27 M16 6 V27 M11 27 H21 M5 6 V10 M27 6 V10',WHITE,width=2.8)
add('DimensionTextBlock','Text','Create a text annotation',text,'OM9-MEASURE-013')
add('OthersMeasureEditText','Edit Text','Edit annotation text',text+pencil(),'OM9-MEASURE-015')
add('OthersMeasureEditDim','Edit Dimension','Edit a dimension annotation',dimension()+pencil(),'OM9-MEASURE-014')
add('DimensionRecenterDimensionText','Recenter Dimension Text','Center annotation text between extension lines',dimension()+letter('T',16,13,size=9)+arrow('M4 6 H10','M7 3 L10 6 L7 9')+arrow('M28 6 H22','M25 3 L22 6 L25 9'),'OM9-MEASURE-017')
add('OthersMeasureDimOptions','Dimension Options','Configure dimension annotation settings',badge(dimension(),'M19 21 H29 M19 26 H29 M22 18 V24 M27 23 V29'),'OM9-MEASURE-016')

# Top bar and generic viewport actions.
add('TopIconDuplicate','Duplicate','Copy selected objects',face('M3 4 H18 V19 H3 Z')+face('M13 14 H28 V29 H13 Z',LIGHT),'OM9-TOP11-002')
curve=path('M4 26 C7 3 22 30 28 5',BLUE,width=2.5)
add('TopIconEditPointsOn','Edit Points','Show points on a curve',curve+points((4,26),(16,16),(28,5),color=WHITE),'OM9-TOP11-003')
add('TopIconControlPointsOn','Control Points','Show curve control handles',curve+path('M4 26 L7 5 L22 27 L28 5',WHITE,width=1.4)+face('M5 3 H9 V7 H5 Z',WHITE)+face('M20 25 H24 V29 H20 Z',WHITE),'OM9-TOP11-001')
add('TopIconMirror','Mirror','Reflect objects across a plane',path('M16 3 V29',WHITE,width=1.8)+face('M4 8 L11 4 V27 L4 23 Z')+face('M28 8 L21 4 V27 L28 23 Z',LIGHT),'OM9-TOP11-004')
move=arrow('M16 3 V29','M12 7 L16 3 L20 7 M12 25 L16 29 L20 25')+arrow('M3 16 H29','M7 12 L3 16 L7 20 M25 12 L29 16 L25 20')
add('TopIconMove','Move','Translate selected objects',move,'OM9-TOP11-006')
add('TopIconRotate','Rotate','Rotate selected objects',path('M8 8 A11 11 0 1 1 5 22',BLUE,width=2.5)+arrow('M8 8 L8 3','M8 3 L14 4 M8 3 L3 8'),'OM9-TOP11-007')
add('TopIconExplode','Explode','Separate joined components',face('M12 12 H20 V20 H12 Z')+path('M4 4 L9 9 M23 9 L28 4 M4 28 L9 23 M23 23 L28 28',WHITE,width=2.6),'OM9-TOP11-005')
add('TopIconJoin','Join','Join compatible objects',face('M3 10 H14 V22 H3 Z')+face('M18 10 H29 V22 H18 Z')+path('M10 16 H22',WHITE,width=3),'OM9-TOP11-008')
add('TopIconSplit','Split','Split objects with a cutter',face('M3 8 H13 V26 H3 Z')+face('M19 8 H29 V26 H19 Z')+path('M16 3 V29',WHITE,width=1.7),'OM9-TOP11-009')
add('TopIconTrim','Trim','Remove geometry beyond a cutting boundary',path('M3 13 H29',BLUE,width=2.5)+circle(8,25,3,WHITE,1.8)+circle(18,25,3,WHITE,1.8)+path('M10 22 L22 4 M16 22 L6 4',WHITE,width=1.8),'OM9-TOP11-010')
add('TopIconRingRail','Ring Rail','Create a ring construction rail',circle(16,16,11)+circle(16,16,7,RED),'OM9-TOP11-011')
for key,right in [('Undo',False),('Redo',True)]:
    geom=path('M7 13 Q21 6 27 23',WHITE,width=2.6)+path('M7 6 V13 H14',WHITE,width=2.6)
    if right:geom='<g transform="translate(32 0) scale(-1 1)">'+geom+'</g>'
    add(key,key,'Reverse an edit' if not right else 'Reapply an undone edit',geom,None,'Native edit operation.')
for key,name in [('ViewFront','F'),('ViewTop','T'),('ViewRight','R'),('ViewLeft','L'),('ViewRear','B'),('ViewBottom','D')]:
    add(key,key.replace('View','')+' View','Set an orthographic view',face('M4 5 H28 V26 H4 Z')+letter(name,16,21,size=15),None,'Native orthographic view; letter identifies view orientation.')
add('FitAll','Fit All','Fit all visible objects',cube()+corners())
add('SelectAll','Select All','Select all objects',corners()+face('M6 6 H14 V14 H6 Z')+circle(23,10,4)+face('M10 25 L16 17 L22 25 Z'))
add('SelectNone','Select None','Clear selection',corners()+path('M6 26 L26 6',WHITE,width=2.8))
add('Delete','Delete','Delete selected objects',face('M8 9 H24 L22 29 H10 Z')+path('M5 6 H27 M12 3 H20 M14 13 V24 M19 13 V24',WHITE,width=1.7))

# Snap markers retain a blue target and white construction.
for key,label,geom,spec in [
 ('End','End',path('M7 24 L24 7',WHITE)+points((7,24),color=BLUE,radius=3),'002'),
 ('Near','Near',path('M4 25 Q14 8 28 8',WHITE)+points((15,14),color=BLUE,radius=3),'006'),
 ('Point','Point',points((16,16),color=BLUE,radius=4),'007'),
 ('Midpoint','Midpoint',path('M3 16 H29',WHITE)+points((16,16),color=BLUE,radius=3),'003'),
 ('Center','Center',circle(16,16,11,WHITE)+points((16,16),color=BLUE,radius=3),'001'),
 ('Intersection','Intersection',path('M4 26 L28 6 M4 7 L28 25',WHITE)+points((16,16),color=BLUE,radius=3),'005'),
 ('PerpendicularTo','Perpendicular',path('M4 26 H28 M16 3 V26 M16 20 H22 V26',WHITE)+points((16,26),color=BLUE,radius=3),'010'),
 ('TangentTo','Tangent',circle(16,19,9,WHITE)+path('M3 10 H29',WHITE)+points((16,10),color=BLUE,radius=3),'008'),
 ('Quadrant','Quadrant',circle(16,16,10,WHITE)+points((6,16),(16,6),(26,16),(16,26),color=BLUE,radius=2.5),'004'),
 ('Knot','Knot',path('M4 24 C5 3 15 10 16 16 S27 30 28 7',WHITE)+points((16,16),color=BLUE,radius=3),None),
]:add('ToolsObjectSnap'+key,label+' Snap','Snap to '+label.lower()+' geometry',geom,'OM9-SNAP-'+spec if spec else None,'Reference snap cue; knot has no dedicated core spec.')
add('SnapBetween','Between Snap','Find a position between picked points',path('M4 24 L28 8',WHITE)+points((4,24),(28,8),color=BLUE,radius=2.5)+points((16,16),color=WHITE,radius=3),'OM9-SNAP-009')
add('SnapOnSurface','Surface Snap','Snap to a selected surface',patch(WHITE)+points((16,17),color=BLUE,radius=3),'OM9-SNAP-012')
add('SnapOnPolysurface','Polysurface Snap','Snap to a selected polysurface',cube(WHITE)+points((10,20),color=BLUE,radius=3),'OM9-SNAP-011')
add('GridSnapON','Grid Snap','Snap to construction grid points',grid()+points((12,20),color=BLUE,radius=3),'OM9-SNAP-013')
add('OrthoSnapON','Ortho','Constrain orthogonal directions',path('M5 4 V27 H28',WHITE,width=2.3)+path('M5 21 H11 V27',BLUE,width=1.6),'OM9-SNAP-014')
add('PlanarSnapON','Planar','Constrain subsequent points to a plane',patch(TAN)+points((16,17),color=BLUE,radius=3),'OM9-SNAP-015')
add('ProjectSnapON','Project Snap','Project snap positions onto the construction plane',patch()+path('M16 3 V20',WHITE,width=1.7)+points((16,20),color=WHITE,radius=3),'OM9-SNAP-016')

# Info/settings: repeated contexts use different action cues.
add('InfoSettingsDisplayProperties','Display Properties','Configure display properties',face('M3 5 H29 V23 H3 Z')+path('M16 23 V29 M10 29 H22',WHITE,width=1.8),'OM9-INFO-010')
add('RhinoOptions','Application Options','Open application options',gear(),'OM9-INFO-002')
add('GVObjectInfo','Object Info','Inspect object information',badge(cube(),'M25 19 V29 M25 15 V16'),'OM9-INFO-005')
add('ViewGridOptions','Grid Options','Configure construction grid',badge(grid(),'M20 22 H29 M20 27 H29 M23 19 V25 M27 24 V30'))
add('ObjectProperties','Object Properties','Edit selected object properties',badge(cube(),'M20 24 H29 M24 19 V29'),'OM9-INFO-001')
add('AllObjectInfo','All Object Info','Inspect information for all objects',page()+badge(cube(),'M25 19 V29 M25 15 V16'),'OM9-INFO-003')
add('CommandHistory','Command History','Display prior command entries',page()+path('M10 14 H22 M10 19 H22 M10 24 H19',WHITE,width=1.6)+path('M3 7 V26',BLUE,width=2),'OM9-INFO-004')
add('ProjectNotes','Project Notes','Edit project notes',page()+pencil(),'OM9-INFO-006')
add('SuperSelect','Super Select','Select objects using filtering criteria',cube()+corners(PINK),'OM9-INFO-007')
add('InfoSettingsViewportTabsToggle','Viewport Tabs','Show or hide viewport tabs',four+path('M4 29 H11 M14 29 H21 M24 29 H28',WHITE,width=2),'OM9-INFO-008')
add('InfoSettingsBoxEdit','Box Edit','Edit objects using a bounding box',cube()+points((4,10),(28,10),(16,29),color=WHITE,radius=2),'OM9-INFO-009')
add('InfoSettingsLibraries','Libraries','Access object libraries',face('M4 5 H11 V27 H4 Z')+face('M13 3 H20 V27 H13 Z')+face('M22 7 H29 V27 H22 Z',LIGHT),'OM9-INFO-011')
add('InfoSettingsSelectionFilter','Selection Filter','Configure selectable object types',face('M3 4 H29 L20 15 V27 L12 29 V15 Z'),'OM9-INFO-012')
add('InfoSettingsDesignReport','Design Report','Generate a design report',page()+face('M10 12 H21 V18 H10 Z',WHITE)+path('M10 23 H22 M10 26 H19',WHITE,width=1.4),'OM9-INFO-014')
gumball=arrow('M16 16 V3','M12 7 L16 3 L20 7',color=GOLD)+arrow('M16 16 L3 28','M3 23 V28 H8',color=RED)+arrow('M16 16 L29 28','M24 28 H29 V23',color='#05D7E5')
add('InfoSettingsGumballAlignment','Gumball Alignment','Set manipulator alignment',gumball+corners(),'OM9-INFO-021')
add('InfoSettingsRelocateGumball','Relocate Gumball','Move the manipulator origin',gumball+points((16,16),color=WHITE,radius=3),'OM9-INFO-022')
add('InfoSettingsGumballON','Gumball','Toggle the object manipulator',gumball+circle(16,16,9,BLUE,1.5),'OM9-INFO-020')
add('InfoSettingsSmartTargetsGumballON','Smart Targets','Enable smart target manipulator',gumball.replace(GOLD,PINK).replace(RED,PINK).replace('#05D7E5',PINK)+circle(16,16,9,PINK,1.5),'OM9-INFO-019')
add('RhinoSmartTrackON','Smart Track','Track temporary snap references',path('M5 26 H27 M5 26 V4 M5 26 L26 5',WHITE,width=1.7)+points((5,26),(26,5),color=BLUE,radius=2.5),'OM9-INFO-013')
add('RhinoHistoryON','History','Toggle dependency history',badge(clock(),'M19 20 L29 20 L29 29 L19 29 Z'),'OM9-INFO-015')
history=path('M5 7 H26 V26 H15',GOLD,width=2.8)+path('M15 22 L11 26 L15 30',GOLD,width=2.4)
add('GVHistoryUpdateON','History Update','Toggle dependency updates',history+face('M5 15 L12 20 L5 25 Z',WHITE),'OM9-INFO-018')
add('GVHistoryRecordON','History Record','Toggle recording object dependencies',history+points((8,21),color=RED,radius=3),'OM9-INFO-017')
add('InfoSettingsGVClearHistory','Clear History','Remove object dependency history',history+path('M3 17 L12 26 M12 17 L3 26',RED,width=2.5),'OM9-INFO-016')
add('GridON','Show Grid','Toggle the construction grid',grid()+path('M16 5 V16 H27',RED,width=1.8),'OM9-DISPLAY-004')
add('PreviewCutterON','Show Cutters','Toggle cutter preview',badge(grid(),'M23 16 V29 M19 20 H27',color='#F2793A'),'OM9-DISPLAY-003')
add('PreviewShadeON','Preview Shade','Toggle shaded preview',circle(16,16,11)+face('M16 5 A11 11 0 0 0 16 27 Z',BLUE,BLUE,.5),'OM9-DISPLAY-001')
add('ShadeSelectedOnlyON','Shade Selected','Shade only selected objects',circle(16,16,10)+corners(PINK),'OM9-DISPLAY-002')
add('GVGemView_1','Gem View','Toggle gem display mode',ring()+path('M16 18 H29',RED,width=2),'OM9-DISPLAY-005')
add('GVSurfaceView_1','Surface View','Toggle surface display mode',patch(GREEN)+path('M16 7 V26 M5 18 H27',RED,width=1.6),'OM9-DISPLAY-007')

# Twelve existing authored generic assets had no original PNG. Preserve their
# existing blue/gold color roles and supply command-specific geometry.
for key,label,meaning,geom in [
 ('LayerArrow','Layer Expand','Expand layer controls',face('M10 6 L23 16 L10 26 Z','#2678c9','#2678c9',.6)),
 ('LayerLock','Layer Lock','Lock the layer',face('M7 14 H25 V29 H7 Z','#f5b400')+path('M11 14 V8 A5 5 0 0 1 21 8 V14',WHITE,width=2)),
 ('LayerVisibility','Layer Visibility','Toggle layer visibility',face('M3 16 Q16 2 29 16 Q16 30 3 16 Z',WHITE)+circle(16,16,4,'#2678c9')),
 ('LayerHide','Hide Layers','Hide layer objects',face('M3 16 Q16 2 29 16 Q16 30 3 16 Z',WHITE)+path('M5 27 L27 5','#2678c9',width=2.5)),
 ('LayerShow','Show Layers','Show layer objects',face('M3 16 Q16 2 29 16 Q16 30 3 16 Z',WHITE)+circle(16,16,4,'#2678c9')+path('M16 1 V5 M16 27 V31',WHITE,width=1.6)),
 ('ProjectAdd','New Project','Add a project',folder().replace(BLUE,'#2678c9')+plus('#f5b400')),
 ('ProjectOut','Project Out','Save current workspace objects into a project slot',folder().replace(BLUE,'#2678c9')+arrow('M14 19 H29','M25 15 L29 19 L25 23',color='#f5b400')),
 ('ProjectIn','Project In','Load project objects into the workspace',folder().replace(BLUE,'#2678c9')+arrow('M29 19 H14','M18 15 L14 19 L18 23',color='#f5b400')),
 ('ProjectSave','Save Project','Save the project',save().replace(BLUE,'#2678c9').replace(LIGHT,'#f5b400')),
 ('ProjectDelete','Delete Project','Delete a project slot',folder().replace(BLUE,'#2678c9')+path('M19 19 L29 29 M29 19 L19 29','#e66352',width=2.3)),
 ('ProjectManager','Project Manager','Manage saved project slots',folder().replace(BLUE,'#2678c9')+path('M7 19 H11 M15 19 H19 M23 19 H27','#f5b400',width=3)),
 ('ViewIsometric','Isometric View','Set the isometric view',cube('#2678c9')),
]:add(key,label,meaning,geom,None,'Native UI / previously authored generic asset; retained existing blue/gold palette. No reference PNG exists.')

for key,label,geom in [
 ('Dial_Wireframe','Wireframe',ring(GREEN)+path('M9 9 L23 25 M23 9 L9 25','#AE5CE7',width=1.3)),
 ('Dial_Shaded','Shaded',ring(GREEN)+face('M8 12 Q16 8 24 12 L25 24 Q16 29 7 24 Z',GREEN,GREEN,.3)+path('M10 8 H22','#AE5CE7',width=2)),
 ('Dial_Working Shade','Working Shade',ring(GREEN)+path('M10 6 H22',PINK,width=3)+path('M9 17 Q16 22 23 17',LIGHT,width=2)),
 ('Dial_Working Render','Working Render',ring(TAN)+path('M10 6 H22',RED,width=3)+path('M8 12 L24 24 M24 12 L8 24',GOLD,width=1.5)),
 ('Dial_Ghosted','Ghosted',ring(GREEN)+circle(16,18,6,WHITE,1.3)+path('M10 6 H22',PINK,width=2)),
 ('Dial_Tech Shade','Technical Shade',ring(GREEN)+face('M8 12 Q16 8 24 12 L25 24 Q16 29 7 24 Z',WHITE,WHITE,.3)+path('M10 6 H22','#AE5CE7',width=2)),
]:add(key,label,'Set '+label.lower()+' display mode',geom,'OM9-DISPLAY-008','Independent ring primitives preserve green/purple or warm render accents; no bitmap tracing.')


def svg(key):
    label,meaning,geometry,_,_=CORE_ICONS[key]
    return f'<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32">\n<title>{html.escape(label)}</title>\n<desc>{html.escape(meaning)}; authored OpenMatrix9 vector.</desc>\n{geometry}\n</svg>\n'


def write_asset(resources,key,tooltip=None):
    label,meaning,_,spec,note=CORE_ICONS[key]
    relative=f'icons/core-minimal/{key}.svg'
    target=Path(resources)/relative;target.parent.mkdir(parents=True,exist_ok=True)
    raw=svg(key).encode('utf-8');target.write_bytes(raw)
    colors=sorted(set(re.findall(r'#[0-9A-Fa-f]{6}',raw.decode())))
    metadata={'image':relative,'tooltip':tooltip or label,'source':'OpenMatrix9-authored-svg',
        'mapping':'authored-command-symbol','status':'resolved'}
    record={'symbol':key,'label':label,'meaning':meaning,'style':STYLE,'spec_id':spec,
        'semantic_source':'local behavior specification' if spec else 'native operation or reference cue; see design note',
        'design_note':note,'palette':colors,'sha256':hashlib.sha256(raw).hexdigest(),
        'mapping':'authored-command-symbol'}
    return metadata,record


def describe_style(report,count):
    report['core_icon_style']={'name':STYLE,'count':count,'source':'OpenMatrix9-authored-svg',
        'generator':'tools/core_icons.py','geometry':'independent primitives; no traced/embedded bitmap data',
        'palette':'blue/white core actions; tan meshes; pink selection; green/purple display; warm render accents; existing blue/gold native layer/project controls',
        'priority':'authored core symbols precede legacy sources'}


def install(root):
    root=Path(root);resources=root/'Resources';target=resources/'menu/icons.ini'
    raw=target.read_text(encoding='utf-8');ini=configparser.ConfigParser(interpolation=None);ini.read_string(raw)
    manifest_path=resources/'menu/source-manifest.json';report=json.loads(manifest_path.read_text(encoding='utf-8'))
    if set(CORE_ICONS)-set(ini.sections()):raise ValueError('Missing core icon bindings')
    for key in CORE_ICONS:
        metadata,record=write_asset(resources,key,ini[key].get('tooltip'))
        if ini[key].get('feature_id'):metadata['feature_id']=ini[key]['feature_id']
        section=f'[{key}]\n'+''.join(f'{k} = {v}\n' for k,v in metadata.items())+'\n'
        raw,count=re.subn(rf'^\[{re.escape(key)}\]\n.*?(?=^\[|\Z)',lambda _:section,raw,flags=re.M|re.S)
        if count!=1:raise ValueError('Non-unique core binding')
        for f in ('shifted_icons','original_buttons','named_crops','missing','auxiliary_missing'):
            report.get(f,{}).pop(key,None)
        report.setdefault('authored_symbols',{})[key]=record
    describe_style(report,len(CORE_ICONS));target.write_text(raw.rstrip()+'\n',encoding='utf-8')
    manifest_path.write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    return {'core_icons':len(CORE_ICONS),'style':STYLE}


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--project-root',type=Path,default=Path(__file__).resolve().parents[1])
    args=parser.parse_args();print(json.dumps(install(args.project_root)))
