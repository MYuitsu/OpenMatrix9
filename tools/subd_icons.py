"""Authored SubD/Clayoo SVG catalog; preserve reference colors and operation cues."""
import argparse
import configparser
import hashlib
import html
import json
from pathlib import Path
import re
from curve_icons import WHITE, path, arrow, points
from builder_icons import BUILDER_ICONS, svg as shared_svg, write_asset as shared_write_asset

RED='#FF0505'
EDGE='#FF6A68'
BLUSH='#FFB5A2'
CYAN='#05B3FF'
PINK='#FF60AF'
GREEN='#05FF05'
AXIS_GREEN='#059A4F'
STYLE='minimal-subd-v1-reviewed'
EDGE_RING_DOC='https://help.autodesk.com/cloudhelp/2024/ENU/Maya-Modeling/files/GUID-2ADFD0C1-05C5-42EC-90F8-3035A6B21D77.htm'
SHARED={'ClayooCreationRing','ClayooCreationSignetRing'}

def face(d,color=WHITE,fill=BLUSH,opacity=.2,width=2.2):
    return path(d,color,width=width).replace('fill="none"',f'fill="{fill}" fill-opacity="{opacity}"')

def ellipse(x,y,rx,ry,color=WHITE,width=2.2):
    return f'<ellipse cx="{x}" cy="{y}" rx="{rx}" ry="{ry}" fill="none" stroke="{color}" stroke-width="{width}"/>'

def cube():
    return face('M4 9 L18 3 L28 9 V23 L14 29 L4 23 Z')+path('M4 9 L14 15 L28 9 M14 15 V29',EDGE,width=1.8)

def patch(grid=False):
    return face('M4 10 Q16 3 28 9 L28 26 Q16 20 4 28 Z')+path('M4 28 Q16 20 28 26',EDGE,width=2.2)+(path('M4 19 Q16 12 28 18 M16 7 V24',EDGE,width=1.5) if grid else '')

def sphere(grid=True):
    return ellipse(16,16,12,12)+face('M4 16 A12 12 0 0 0 28 16',EDGE,width=1.7)+(ellipse(16,16,5,12,EDGE,width=1.5)+ellipse(16,16,12,4,EDGE,width=1.5) if grid else '')

def cylinder():
    return face('M5 8 V24 C5 31 27 31 27 24 V8')+ellipse(16,8,11,5)+path('M5 8 C5 15 27 15 27 8 M16 13 V29',EDGE,width=1.6)

def quad_grid():
    return face('M4 5 H28 V29 H4 Z',WHITE,BLUSH,.08,1.7)+path('M12 5 V29 M20 5 V29 M4 13 H28 M4 21 H28',WHITE,width=1.5)

def selected(x,y,color=RED,opacity=.35):
    return face(f'M{x} {y} H{x+8} V{y+8} H{x} Z',color,color,opacity,1.8)

def brackets():
    return path('M3 9 V3 H9 M23 3 H29 V9 M29 23 V29 H23 M9 29 H3 V23',EDGE,width=1.8)

def pencil():
    return face('M18 13 L25 4 L29 7 L22 16 L17 18 Z',WHITE,WHITE,.2,1.8)+path('M24 5 L28 8',EDGE,width=1.4)

def brush():
    return face('M20 14 L26 3 L29 6 L23 17 Z',WHITE,WHITE,.15,1.8)+face('M20 14 L23 17 L21 20 L18 17 Z',WHITE,BLUSH,.2,1.5)+face('M18 17 Q13 17 15 24 Q21 26 21 20 Z',EDGE,BLUSH,.4,1.8)

def sweep(rails):
    art=face('M4 28 C3 10 15 2 27 6 L29 18 C17 13 12 20 12 29 Z')
    art+=path('M4 28 C3 10 15 2 27 6',EDGE,width=2.6)
    if rails==2: art+=path('M12 29 C12 20 17 13 29 18',EDGE,width=2.6)
    art+=path('M5 19 Q15 8 28 12',WHITE,width=1.6)
    number=path('M26 5 L28 3 V11',RED,width=1.8) if rails==1 else path('M24 5 Q26 1 29 3 Q32 5 27 8 L24 11 H30',RED,width=1.8)
    return '<g transform="translate(0 3) scale(.8)">'+art+'</g>'+number

def conversion(kind):
    # Large separate source and smooth result; source topology remains visible.
    d='M3 3 H15 V13 H3 Z'
    art=face(d,GREEN if kind=='tspline' else WHITE,BLUSH,.15,1.7)
    if kind=='mesh': art+=path('M3 3 L15 13 M15 3 L3 13',WHITE,width=1.5)
    elif kind=='tspline': art+=path('M9 3 V13 M3 8 H15 M12 8 V13',GREEN,width=1.5)
    elif kind=='surface': art+=path('M3 8 Q9 4 15 8',WHITE,width=1.5)
    art+=face('M14 23 Q20 16 28 20 L29 29 Q21 25 14 30 Z')+path('M21 20 V27',EDGE,width=1.4)
    return art+arrow('M20 5 Q28 5 27 15','M24 12 L27 15 L30 12')

def objects(selected_all=False):
    c=RED if selected_all else WHITE
    return face('M4 4 H13 V13 H4 Z',c,BLUSH,.15,1.8)+ellipse(23,9,5,5,c,width=1.8)+face('M11 19 L23 19 L27 28 H6 Z',c,BLUSH,.15,1.8)

SUBD_ICONS={
 'ClayooEditItem':('Edit SubD','Enable editing of SubD objects and show the HUD',1,
    ellipse(13,18,10,10,EDGE)+path('M7 18 H14 M11 14 V22',CYAN,width=1.5)+pencil()),
 'ClayooEditDivide':('Divide faces','Divide faces along U, V or both directions',6,
    patch()+path('M4 19 Q16 12 28 18 M16 7 V24',RED,width=2.4)),
 'ClayooEditSplitSides':('Split sides','Insert parallel edges on both sides of selected edges',9,
    patch()+path('M16 7 V24',WHITE,width=1.4)+path('M11 8 V25 M21 7 V24',RED,width=2.2)),
 'ClayooEditInset':('Inset','Inset selected faces by a specified distance',8,
    cube()+face('M17 16 L25 13 V21 L17 25 Z',RED,BLUSH,.35,2.2)),
 'ClayooEditKnife':('Knife','Subdivide a surface using a line or axis',11,
    sphere(False)+path('M5 25 L24 6',RED,width=2.2)+face('M22 10 L26 3 L29 5 L25 12 Z',WHITE,WHITE,.2,1.8)),
 'ClayooEditOffset':('Offset faces','Copy selected faces at a distance from their originals',13,
    face('M3 16 L12 12 V26 L3 30 Z',WHITE,BLUSH,.1,1.7)
    +face('M21 5 L29 2 V17 L21 20 Z',EDGE,BLUSH,.35)
    +arrow('M12 21 L21 12','M17 12 H21 V16')),
 'ClayooEditExtrude':('Extrude','Extrude selected geometry',10,
    face('M3 10 H12 V28 H3 Z',WHITE,BLUSH,.12,1.7)
    +face('M12 10 L25 4 L29 7 V22 L12 28 Z')
    +face('M25 4 L29 7 V22 L25 19 Z',EDGE,BLUSH,.45)
    +path('M12 10 L25 4 M12 28 L25 19',EDGE,width=1.5)),
 'ClayooEditBridge':('Bridge','Connect faces with a transition between SubD objects',12,
    face('M3 4 H10 V13 H3 Z',WHITE,BLUSH,.1,1.6)+face('M23 20 H29 V29 H23 Z',WHITE,BLUSH,.1,1.6)
    +face('M3 13 C3 28 23 6 23 20 L29 20 C29 1 10 25 10 13 Z',EDGE,BLUSH,.3)),
 'ClayooCreationAppendFace':('Append face','Draw new faces to close a surface',14,
    face('M3 4 H13 V28 H3 Z')+face('M13 15 L27 8 V24 L13 28 Z',EDGE,BLUSH,.3)
    +points((13,15),(27,8),(27,24),color=WHITE,radius=1.9)+path('M23 4 H29 M26 2 V8',EDGE,width=1.8)),
 'ClayooCreationPipe':('Pipe','Create SubD surfaces around curves and blend intersecting branches',15,
    path('M5 27 C9 24 11 15 14 15 C18 14 21 22 28 20 M14 15 C14 10 18 6 25 4',BLUSH,width=7)
    +path('M3 25 Q11 20 10 15 Q9 7 23 2 M6 29 Q17 19 18 16 Q22 24 29 23',EDGE,width=1.8)),
 'ClayooCreationClayooBezel':('SubD bezel','Create a bezel around an existing gem',16,
    cylinder()+face('M8 6 L12 2 H21 L25 6 L17 12 Z',WHITE,WHITE,.08,1.7)
    +path('M8 6 H25 M12 2 L17 12 L21 2',WHITE,width=1.4)),
 'ClayooCreationRing':BUILDER_ICONS['ClayooCreationRing'],
 'ClayooCreationSignetRing':BUILDER_ICONS['ClayooCreationSignetRing'],
 'ClayooEditCrease':('Crease','Add sharp edges to a SubD surface',20,
    cube()+path('M4 9 L14 15 L28 9 M14 15 V29',CYAN,width=3)),
 'ClayooEditFlipNormals':('Flip normals','Reverse the normal direction of a SubD surface',41,
    face('M3 23 L22 16 L29 23 L10 30 Z')
    +arrow('M10 22 V4','M7 7 L10 4 L13 7',EDGE)
    +arrow('M23 3 V17','M20 14 L23 17 L26 14',WHITE)),
 'ClayooEditProjectToPlane':('Project to plane','Project selected geometry onto a plane',19,
    ellipse(15,7,5,5)+face('M3 24 L20 18 L29 24 L12 30 Z')
    +ellipse(16,25,5,2,RED,width=1.8)+arrow('M15 14 V22','M12 19 L15 22 L18 19')),
 'ClayooEditCollapse':('Collapse','Join selected vertices at the center of the selection',22,
    face('M4 4 H28 V28 H4 Z',WHITE,BLUSH,.1,1.6)
    +points((5,5),(27,5),(5,27),(27,27),radius=1.9)
    +arrow('M7 7 L13 13','M9 13 H13 V9',EDGE)+arrow('M25 25 L19 19','M23 19 H19 V23',EDGE)
    +path('M25 7 L19 13 M7 25 L13 19',EDGE,width=1.8)+points((16,16),color=RED,radius=2.1)),
 'ClayooEditMatch':('Match','Match SubD edges to another surface or target curve',23,
    face('M3 7 L12 3 V27 L3 29 Z',WHITE,BLUSH,.1,1.6)
    +face('M20 3 Q28 9 29 25 L20 29 Z')
    +path('M12 3 V27 M20 3 V29',EDGE,width=2.2)
    +arrow('M4 16 H17','M14 13 L17 16 L14 19')),
 'ClayooEditWeld':('Weld','Connect SubD surfaces to each other or themselves',25,
    face('M3 7 L16 3 L29 7 V26 L16 29 L3 26 Z')
    +path('M16 3 V29',EDGE,width=2.4)+points((16,9),(16,16),(16,23),color=RED,radius=1.9)),
 'ClayooEditSymmetry':('Symmetry','Mirror a SubD surface along an axis',24,
    path('M16 2 V30',WHITE,dashed=True,width=1.4)
    +face('M4 6 Q12 6 12 17 L4 27 Z')+face('M28 6 Q20 6 20 17 L28 27 Z',EDGE,BLUSH,.3)),
 'ClayooPrimitivesBox':('SubD box','Create a box with dimensions and surface divisions',26,
    cube()+path('M4 16 L14 22 L28 16 M9 7 L20 12 V26',EDGE,width=1.4)),
 'ClayooPrimitivesSphere':('SubD sphere','Create a sphere or quad sphere with surface divisions',27,sphere()),
 'ClayooPrimitivesCylinder':('SubD cylinder','Create a cylinder with surface segments',28,cylinder()),
 'ClayooPrimitivesCone':('SubD cone','Cone primitive with surface divisions',29,
    face('M4 26 L16 3 L28 26 C28 31 4 31 4 26 Z')+ellipse(16,26,12,3,EDGE,width=1.8)+path('M16 3 V29',EDGE,width=1.4)),
 'ClayooPrimitivesTorus':('SubD torus','Torus primitive reference',None,
    ellipse(16,16,13,9)+ellipse(16,14,6,3,EDGE)+path('M3 16 Q16 26 29 16',EDGE,width=1.8)),
 'ClayooPrimitivesPlane':('SubD plane','Create a plane with dimensions and surface segments',30,
    face('M3 15 L18 5 L29 17 L14 28 Z')+path('M8 12 L20 24 M13 8 L25 20 M7 20 L22 10',EDGE,width=1.4)),
 'ClayooCreationSweep1Rail':('Sweep 1 rail','Create a SubD surface from one rail and profiles',31,sweep(1)),
 'ClayooCreationSweep2Rails':('Sweep 2 rails','Create a SubD surface from two rails and profiles',32,sweep(2)),
 'ClayooCreationLoft':('Loft','Create a SubD surface through two or more curves',33,
    face('M4 28 Q10 7 14 4 L27 10 Q23 14 28 27 Z')
    +path('M4 28 Q16 17 28 27 M7 18 Q16 9 26 17 M14 4 Q23 2 27 10',EDGE,width=2.2)),
 'ClayooEditWrap':('Wrap','Wrap a SubD object around a target object',34,
    sphere(False)+face('M3 12 Q16 3 29 12 L28 22 Q16 13 4 22 Z',EDGE,BLUSH,.3)
    +path('M16 8 V17',WHITE,width=1.5)),
 'ClayooCreationRevolve':('Revolve','Create a SubD surface by revolving a curve around an axis',35,
    path('M16 2 V30',WHITE,dashed=True,width=1.4)
    +face('M7 4 Q14 12 9 23 Q16 30 23 23 Q18 12 25 4 Z')
    +path('M25 4 Q18 12 23 23',EDGE,width=2.2)
    +arrow('M4 17 Q16 24 28 17','M25 15 L28 17 L25 20',EDGE)),
 'ClayooCreationFromCurve':('From curve','Create a SubD surface from an input curve',36,
    patch()+path('M4 10 Q16 3 28 9 L28 26 Q16 20 4 28 Z',EDGE,width=2.8)
    +path('M4 19 Q16 12 28 18 M16 7 V24',WHITE,width=1.5)),
 'ClayooCreationFrom2Curves':('From 2 curves','Create a SubD surface or solid from two open curves',37,
    face('M4 4 Q11 16 4 28 L28 28 Q21 16 28 4 Z')
    +path('M4 4 Q11 16 4 28 M28 4 Q21 16 28 28',EDGE,width=2.8)
    +path('M5 12 H27 M5 21 H27',WHITE,width=1.4)),
 'ClayooEditFill':('Fill','Create a face to close a surface opening',39,
    face('M5 8 V25 C5 31 27 31 27 25 V8')+ellipse(16,8,11,5,EDGE)
    +face('M5 8 C5 2 27 2 27 8 C27 14 5 14 5 8 Z',EDGE,BLUSH,.45)
    +path('M12 8 H20 M16 4 V12',WHITE,width=1.8)),
 'ClayooEditShell':('Shell','Hollow a solid SubD object while removing selected faces',40,
    face('M4 9 V24 C4 32 28 32 28 24 V9')+ellipse(16,9,12,6,EDGE)
    +ellipse(16,9,7,3,WHITE,width=2.2)+path('M9 9 V22 Q16 27 23 22 V9',EDGE,width=1.5)),
 'ClayooEditExtract':('Extract faces','Duplicate or remove selected faces from a SubD model',38,
    face('M3 14 L13 9 L21 13 V26 L11 30 L3 26 Z')
    +face('M14 4 L24 2.5 L29 8 L19 11 Z',EDGE,BLUSH,.4)
    +path('M3 14 L11 19 L21 13 M11 19 V30',EDGE,width=1.4)
    +arrow('M18 19 V12','M15 15 L18 12 L21 15')),
 'ClayooCreationFromSurface':('From surface','Create a SubD surface from a surface or polysurface',42,conversion('surface')),
 'ClayooCreationFromTSplines':('From T-Splines','Create a SubD surface from a T-Splines object',43,conversion('tspline')),
 'ClayooCreationFromMesh':('From mesh','Create a SubD surface from a polygon mesh',45,conversion('mesh')),
 'ClayooCreationToNurbs':('To NURBS','Convert a SubD object to a NURBS surface',44,
    ellipse(16,16,13,13,EDGE)+face('M7 14 Q16 4 24 11 L25 23 Q16 17 8 25 Z')
    +path('M8 19 Q16 10 25 16 M16 9 L17 21',WHITE,width=1.5)),
 'ClayooSelectionSelectionSets':('Selection sets','Save selection groups of vertices, edges, faces or objects',46,
    face('M4 6 H22 V28 H4 Z',EDGE,BLUSH,.1,1.7)
    +path('M8 20 L15 11 L20 21 Z',WHITE,width=1.5)+points((8,20),(15,11),(20,21),color=RED,radius=1.9)
    +face('M23 3 H29 V16 L26 13 L23 16 Z',WHITE,WHITE,.15,1.7)),
 'ClayooSelectionSelectRing':('Select ring','Select an edge ring across adjacent faces',48,
    face('M4 8 H28 V24 H4 Z',WHITE,BLUSH,.07,1.5)+path('M4 16 H28',WHITE,width=1.5)
    +path('M4 8 V24 M12 8 V24 M20 8 V24 M28 8 V24',RED,width=2.4)+brackets()),
 'ClayooSelectionSelectLoop':('Select loop','Select a continuous loop of connected edges',47,
    face('M4 8 H28 V24 H4 Z',WHITE,BLUSH,.07,1.5)+path('M12 8 V24 M20 8 V24',WHITE,width=1.5)
    +path('M4 16 H28',RED,width=2.8)+brackets()),
 'ClayooSelectUV':('Select U / V','Select contiguous faces in U and V directions',49,
    quad_grid()+selected(12,5)+selected(12,13)+selected(12,21)+selected(4,13)+selected(20,13)),
 'ClayooSelectionSelectAll':('Select all','Select geometry according to the active selection mode',52,objects(True)+brackets()),
 'ClayooSelectionSelectNone':('Select none','Deselect SubD geometry',53,objects(False)
    +ellipse(27,27,3,3,WHITE,width=1.4)),
 'ClayooSelectionPaintSelection':('Paint selection','Add geometry by clicking and dragging across it',51,
    '<g transform="translate(0 5) scale(.8)">'+quad_grid()+selected(4,13)+selected(12,13)+selected(12,21)+'</g>'+brush()),
 'ClayooSelectionGrowSelection':('Grow selection','Grow the selection by one in each direction',55,
    quad_grid()+face('M4 5 H28 V29 H4 Z',EDGE,BLUSH,.15,1.7)+selected(12,13)
    +arrow('M16 13 V6','M13 9 L16 6 L19 9')+arrow('M20 17 H27','M24 14 L27 17 L24 20')),
 'ClayooSelectionShrink':('Shrink selection','Shrink the selection by one in each direction',56,
    quad_grid()+face('M4 5 H28 V29 H4 Z',EDGE,BLUSH,.15,1.7)+selected(12,13)
    +arrow('M16 5 V12','M13 9 L16 12 L19 9')+arrow('M28 17 H21','M24 14 L21 17 L24 20')),
 'ClayooSelectionInvertSelection':('Invert selection','Invert the current selection within the active mode',54,
    face('M4 9 H26 V29 H4 Z',WHITE,BLUSH,.08,1.5)
    +face('M4 9 H15 V19 H4 Z',RED,RED,.3,1.6)+face('M15 19 H26 V29 H15 Z',RED,RED,.3,1.6)
    +path('M15 9 V29 M4 19 H26',WHITE,width=1.4)
    +arrow('M5 5 Q16 0 28 5','M25 2 L28 5 L25 8')),
 'ClayooSelectionSelectCreased':('Select creased','Select creased edges',57,
    cube()+path('M14 15 L28 9 M14 15 V29',CYAN,width=3)+brackets()),
 'ClayooSelectionSelectNaked':('Select naked edges','Select naked edges, extending contiguous selection when present',58,
    patch(True)+path('M4 10 Q16 3 28 9 L28 26 Q16 20 4 28 Z',PINK,width=2.8)+brackets()),
 'ClayooSelectionSelectCoplanar':('Select coplanar','Select geometry sharing a plane',61,
    cube()+face('M4 9 L18 3 L28 9 L14 15 Z',RED,RED,.28,1.8)
    +path('M11 6 L21 12 M9 12 L22 6',WHITE,width=1.4)+brackets()),
 'ClayooEditSubdivisionLevel':('Subdivision level','Control how accurately the SubD mesh displays an object',62,
    sphere()+path('M7 7 Q16 12 25 7 M7 25 Q16 20 25 25 M8 7 Q13 16 8 25 M24 7 Q19 16 24 25',EDGE,width=1.4)),
 'ClayooCreationClayooLibrary':('SubD library','Library of pre-built SubD designs',60,
    face('M3 10 H12 V24 H3 Z')+face('M13 5 Q19 1 23 5 V20 H13 Z')
    +face('M23 12 L29 21 L20 26 Z')+path('M3 29 H29',EDGE,width=2.2)),
 'ClayooSelectionSetPlane':('Set plane','Set-plane reference: oriented plane with axes',None,
    face('M3 21 L15 8 L29 18 L17 30 Z')
    +arrow('M16 20 L26 13','M22 13 H26 V17',RED)
    +arrow('M16 20 L7 13','M7 17 V13 H11',AXIS_GREEN)
    +arrow('M16 20 V3','M13 6 L16 3 L19 6')),
 'ClayooSelectionResetPlane':('Reset plane','Reset-plane reference: restore a plane and axes',None,
    face('M3 23 L18 16 L29 23 L14 30 Z')
    +arrow('M14 24 L24 19','M20 19 H24 V23',RED)
    +arrow('M14 24 L6 19','M6 23 V19 H10',AXIS_GREEN)
    +arrow('M6 13 C3 1 26 1 27 12','M23 9 L27 12 L29 8')),
 'ClayooCreateIsocurves':('Create isocurves','Isoparametric-curve creation reference on a SubD surface',None,
    patch()+path('M4 17 Q16 10 28 16 M10 8 V26 M21 7 V24',EDGE,width=2.2)),
}

NOTES={
 'ClayooEditItem':'White edit pencil and familiar red circular cue; cyan manipulation cross.',
 'ClayooEditDivide':'Crossing U/V subdivision lines; distinct from parallel split-side lines.',
 'ClayooEditSplitSides':'Two separated new edges flanking the central selected edge.',
 'ClayooEditInset':'Inner boundary on a cube face; does not suggest offsetting the whole object.',
 'ClayooEditKnife':'Slanted subdivision line and separate knife blade.',
 'ClayooEditOffset':'Detached copied face with a clear gap and transfer direction; no joining side walls.',
 'ClayooEditExtrude':'Extended face remains joined to the source by visible side walls.',
 'ClayooEditBridge':'Broad transition spans separate boundary faces.',
 'ClayooCreationAppendFace':'New face outlined with visible corner picks; Fill instead uses a completed cap.',
 'ClayooCreationPipe':'Broad curved branches meeting at a blended junction.',
 'ClayooCreationClayooBezel':'Faceted white gem enclosed by a blush bezel; no plain cylinder substitution.',
 'ClayooCreationRing':'Reuse approved Builder white/blush ring SVG without repainting.',
 'ClayooCreationSignetRing':'Reuse approved Builder white/blush signet SVG without repainting.',
 'ClayooEditCrease':'Cyan sharp edge on a white box; crease edit has no selection brackets.',
 'ClayooEditFlipNormals':'Opposite arrows on the face distinguish reversed normal orientation.',
 'ClayooEditProjectToPlane':'Object above a plane, downward arrow and projected planar outline.',
 'ClayooEditCollapse':'Corner vertices converge toward one red central vertex.',
 'ClayooEditMatch':'Separate boundaries and a directed alignment cue; Weld uses a connected seam.',
 'ClayooEditWeld':'Joined patch with common red seam and merged vertex stations.',
 'ClayooEditSymmetry':'Mirrored forms separated by a clear axial line.',
 'ClayooPrimitivesBox':'White/blush box with sparse surface divisions.',
 'ClayooPrimitivesSphere':'Round sphere silhouette with a few curved quad directions.',
 'ClayooPrimitivesCylinder':'Separate elliptical cap and straight sides with a subdivision seam.',
 'ClayooPrimitivesCone':'Single apex and elliptical base; no cylinder-like upper cap.',
 'ClayooPrimitivesTorus':'Torus silhouette and distinct central opening; no dedicated local spec.',
 'ClayooPrimitivesPlane':'Flat quad plane and sparse subdivisions.',
 'ClayooCreationSweep1Rail':'One highlighted rail plus a large 1 cue.',
 'ClayooCreationSweep2Rails':'Two highlighted rails plus a large 2 cue.',
 'ClayooCreationLoft':'Multiple section curves across the resulting surface.',
 'ClayooEditWrap':'Broad red/blush patch visibly bends over a round target.',
 'ClayooCreationRevolve':'Profile, dashed axis and large curved turn arrow.',
 'ClayooCreationFromCurve':'Single input boundary emphasized around a created patch.',
 'ClayooCreationFrom2Curves':'Two separate input curves highlighted on opposite sides of the result.',
 'ClayooEditFill':'Closed face across an opening, cap silhouette and plus cue.',
 'ClayooEditShell':'Visible inner opening and inner wall distinguish hollow shell from Fill.',
 'ClayooEditExtract':'Detached original face and remaining volume silhouette; distinct from parallel Offset copy.',
 'ClayooCreationFromSurface':'Separate source surface and resulting quad patch with transfer arrow.',
 'ClayooCreationFromTSplines':'Preserve green source T-junction grid; result remains white/blush.',
 'ClayooCreationFromMesh':'Triangulated white source, separated smooth result.',
 'ClayooCreationToNurbs':'Retain circular red family cue around a clean parametric patch.',
 'ClayooSelectionSelectionSets':'Selected triangle vertices plus bookmark denote saving a group.',
 'ClayooSelectionSelectRing':'Parallel disconnected edge stations across faces; general ring convention clarified by Autodesk, Matrix-specific details not established.',
 'ClayooSelectionSelectLoop':'One connected selected edge chain, not a row of disconnected parallel edges.',
 'ClayooSelectUV':'Cross of selected face strips; combined key maps to local U and V specs 049/050.',
 'ClayooSelectionSelectAll':'Three geometry types with red selection outlines and selection brackets.',
 'ClayooSelectionSelectNone':'Same geometry types stay white, with an empty indicator; not delete/X symbolism.',
 'ClayooSelectionPaintSelection':'Selected face trail and a broad brush head distinguish paint selection from Edit pencil.',
 'ClayooSelectionGrowSelection':'Selected center and outward arrows to surrounding cells.',
 'ClayooSelectionShrink':'Selected center and inward arrows from surrounding cells.',
 'ClayooSelectionInvertSelection':'Complementary selected/unselected cell pattern and turn arrow.',
 'ClayooSelectionSelectCreased':'Cyan selected edges plus red selection brackets; distinct from adding a crease.',
 'ClayooSelectionSelectNaked':'Pink outer boundary on an open patch; preserves original special edge color.',
 'ClayooSelectionSelectCoplanar':'Only the common top plane selected on a box with other planes visible.',
 'ClayooEditSubdivisionLevel':'Denser quad directions on a sphere, not arrows suggesting geometric scale.',
 'ClayooCreationClayooLibrary':'Different prebuilt forms on a shelf; no saved-selection bookmark.',
 'ClayooSelectionSetPlane':'Oriented work-plane axes retained; exact selection procedure and plane role unverified.',
 'ClayooSelectionResetPlane':'Plane axes and restore arrow retained; reset target frame unverified.',
 'ClayooCreateIsocurves':'Parametric curves visible on a patch; exact creation options/output unverified.',
}

def svg(key):
    label, meaning, spec, geometry = SUBD_ICONS[key]
    if key in SHARED: return shared_svg(key)
    return (f'<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32">\n'
            f'<title>{html.escape(label)}</title>\n<desc>{html.escape(meaning)}. '
            f'OpenMatrix9 {STYLE}; authored vector geometry.</desc>\n{geometry}\n</svg>\n')


def write_asset(output_root, key, tooltip=None):
    if key in SHARED: return shared_write_asset(output_root,key,tooltip)
    relative = f'icons/subd-minimal/{key}.svg'
    target = Path(output_root) / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    raw = svg(key).encode('utf-8')
    target.write_bytes(raw)
    label, meaning, spec, _ = SUBD_ICONS[key]
    metadata = {'image': relative, 'tooltip': tooltip or key,
                'source': 'OpenMatrix9-authored-svg', 'mapping': 'authored-command-symbol', 'status': 'resolved'}
    record = {'symbol': key, 'label': label, 'meaning': meaning, 'style': STYLE,
              'mapping': metadata['mapping'], 'sha256': hashlib.sha256(raw).hexdigest(),
              'palette': [c for c in (RED, EDGE, BLUSH, CYAN, PINK, GREEN, AXIS_GREEN, WHITE) if c in raw.decode()],
              'spec_id': f'OM9-SUBD-{spec:03}' if spec else None,
              'related_spec_ids': ['OM9-SUBD-049','OM9-SUBD-050'] if key=='ClayooSelectUV' else [],
              'semantic_source': 'local SubD spec' if spec else 'reference cue; no exact local spec',
              'semantic_status': 'local-spec-reviewed' if spec else 'reference-cue-only',
              'source_url': EDGE_RING_DOC if key=='ClayooSelectionSelectRing' else None,
              'reference_image': f'icons/rgb-plus5/ButtonIcons/{key}_1.png',
              'reference_type': 'selected Matrix90 RGB+5 derivative',
              'design_note': NOTES[key]}

    return metadata, {k:v for k,v in record.items() if k not in ('reference_image','reference_type','reference_images','palette_reference','reference_palette')}


def subd_keys(resources):
    menu = configparser.ConfigParser(interpolation=None)
    menu.optionxform = str
    menu.read(Path(resources) / 'menu/MainMenu.ini', encoding='utf-8')
    groups = [s for s in menu.values() if s.get('Name') in ('SubD','Clayoo')]
    if len(groups) != 1:
        raise ValueError('Expected one SubD/Clayoo group')
    group = groups[0]
    keys = [v for k,v in group.items() if re.fullmatch(r'Icon\d+', k)]
    if len(keys) != int(group['IconCount']) or set(keys) != set(SUBD_ICONS):
        raise ValueError('SubD menu and authored catalog disagree')
    return keys


def contact_sheet(keys):
    columns, cell_w, cell_h = 8, 160, 132
    height = 64 + ((len(keys)+columns-1)//columns)*cell_h
    parts = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{columns*cell_w}" height="{height}">',
             '<rect width="100%" height="100%" fill="#333333"/>',
             '<text x="16" y="28" font-family="Arial,sans-serif" font-size="20" fill="white">OpenMatrix9 — SubD / minimal SVG</text>',
             '<text x="16" y="48" font-family="Arial,sans-serif" font-size="12" fill="#cccccc">58 icons · white / blush SubD · red actions · cyan crease · pink naked edges · green T-Splines · 64px and 24px samples</text>']
    for i,key in enumerate(keys):
        x,y = (i%columns)*cell_w,64+(i//columns)*cell_h
        label,_,_,geometry = SUBD_ICONS[key]
        parts += [f'<g transform="translate({x} {y})">',
                  '<rect x="4" y="4" width="152" height="124" rx="4" fill="#696969"/>',
                  f'<text x="12" y="20" font-family="Arial,sans-serif" font-size="11" fill="#dddddd">{i+1:02}</text>',
                  f'<g transform="translate(24 24) scale(2)">{geometry}</g>',
                  f'<g transform="translate(120 45) scale(.75)">{geometry}</g>']
        lines=[]
        for word in label.split():
            if lines and len(lines[-1])+len(word)+1 <= 24:
                lines[-1] += ' '+word
            else:
                lines.append(word)
        parts += [f'<text x="80" y="{98+n*12}" text-anchor="middle" font-family="Arial,sans-serif" font-size="10.5" fill="white">{html.escape(line)}</text>' for n,line in enumerate(lines)]
        parts.append('</g>')
    return '\n'.join(parts+['</svg>\n'])


def describe_style(report, count):
    report['subd_icon_style'] = {'name': STYLE, 'count': count,
        'source': 'OpenMatrix9-authored-svg', 'generator': 'tools/subd_icons.py',
        'geometry': 'authored geometric primitives; no tracing or embedded bitmaps',
        'palette_reference': 'selected RGB+5: white #FFFFFF / blush #FFB5A2; edge #FF6A68; red #FF0505; crease #05B3FF; naked #FF60AF; T-Splines #05FF05; axes #059A4F; two Builder shared icons retain their palette',
        'priority': 'authored SubD symbols precede legacy image sources'}
    prefix='authored SubD SVG first; '
    if not report.get('asset_policy','').startswith(prefix):
        report['asset_policy']=prefix+report.get('asset_policy','')


def install(project_root):
    root=Path(project_root)
    resources=root/'Resources'
    keys=subd_keys(resources)
    ini_path=resources/'menu/icons.ini'
    original=ini_path.read_text(encoding='utf-8')
    ini=configparser.ConfigParser(interpolation=None)
    ini.read_string(original)
    manifest_path=resources/'menu/source-manifest.json'
    report=json.loads(manifest_path.read_text(encoding='utf-8'))
    for key in keys:
        if not ini.has_section(key):
            raise ValueError('Missing binding: '+key)
    for key in keys:
        metadata,record=write_asset(resources,key,ini[key].get('tooltip'))
        if ini[key].get('feature_id'):
            metadata['feature_id']=ini[key]['feature_id']
        section=f'[{key}]\n'+''.join(f'{k} = {v}\n' for k,v in metadata.items())+'\n'
        original,n=re.subn(rf'^\[{re.escape(key)}\]\n.*?(?=^\[|\Z)',lambda _:section,original,flags=re.M|re.S)
        if n != 1:
            raise ValueError('Non-unique binding: '+key)
        for field in ('shifted_icons','original_buttons','named_crops','missing','auxiliary_missing'):
            report.get(field,{}).pop(key,None)
        report.setdefault('authored_symbols',{})[key]=record
    describe_style(report,len(keys))
    ini_path.write_text(original.rstrip()+'\n',encoding='utf-8')
    manifest_path.write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    gallery=root/'docs/images/subd-icons-minimal.svg'
    gallery.parent.mkdir(parents=True,exist_ok=True)
    gallery.write_text(contact_sheet(keys),encoding='utf-8')
    return {'count':len(keys),'style':STYLE,'preview':str(gallery)}


if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--project-root',type=Path,default=Path(__file__).resolve().parents[1])
    args=parser.parse_args()
    print(json.dumps(install(args.project_root)))
