"""Authored Transform icons; functional cues reviewed against each reference."""
import argparse
import configparser
import hashlib
import html
import json
from pathlib import Path
import re
from curve_icons import WHITE, path, arrow, points

ORANGE='#FF5B05'
LIGHT='#FF9566'
RED='#B1050F'
STYLE='minimal-transform-v2-reviewed'

def face(d,color=ORANGE,opacity=.28,width=2.2):
    return path(d,color,width=width).replace('fill="none"',f'fill="{color}" fill-opacity="{opacity}"')

def rect(x,y,w,h,color=ORANGE):
    return face(f'M{x} {y} H{x+w} V{y+h} H{x} Z',color)

def prism(x=7,y=12,w=12,h=12,d=6,color=ORANGE):
    # A rectangular front and two projected faces, proportional at every size.
    return (face(f'M{x} {y} H{x+w} V{y+h} H{x} Z',color)
        +face(f'M{x} {y} L{x+d} {y-d} H{x+w+d} L{x+w} {y} Z',LIGHT if color==ORANGE else color,.18,1.7)
        +face(f'M{x+w} {y} L{x+w+d} {y-d} V{y+h-d} L{x+w} {y+h} Z',color,.18,1.7))

def small(x,y,color=ORANGE):
    return prism(x,y,5,5,2,color)

def diamond(d,color=ORANGE):
    return face(d,color)

def plane():
    return face('M3 22 L20 16 L29 23 L12 29 Z',WHITE,.1,1.6)

def standing():
    return face('M3 4 H12 V28 H3 Z',WHITE,.08,1.6)

def curved():
    return face('M18 6 Q23 3 28 7 L28 27 Q23 23 18 26 Z',WHITE,.08,1.6)

def patch():
    return face('M3 18 Q14 10 28 15 L28 28 Q15 23 3 29 Z',WHITE,.08,1.6)

def spine():
    return path('M16 3 V29',WHITE,dashed=True,width=1.4)

def capsule(x,y,color=ORANGE):
    return face(f'M{x} {y+3} Q{x} {y} {x+3} {y} H{x+6} Q{x+9} {y} {x+9} {y+3} V{y+5} Q{x+9} {y+8} {x+6} {y+8} H{x+3} Q{x} {y+8} {x} {y+5} Z',color)

def cage(warp=False):
    if warp:
        return (face('M3 10 L9 4 L25 8 L29 14 L24 27 L8 29 L3 23 Z',ORANGE,.06,1.8)
            +path('M3 10 L18 14 L29 14 M18 14 L14 29 M9 4 L18 14 M3 23 L14 29 L24 27',ORANGE,width=1.5))
    return (face('M3 10 L10 3 H28 V21 L21 29 H3 Z',ORANGE,.06,1.8)
        +path('M3 10 H21 L28 3 M21 10 V29 M3 20 H21 L28 13 M12 10 V29',ORANGE,width=1.5))

# Indexed by verified spec ID, not menu order. Zero is Matrix Tools Smart Pattern.
DESIGNS={
 1:(prism(3,11,17,17,8)+prism(8,15,8,8,4,WHITE),
    'Use a white source cube inside an orange three-dimensional envelope; remove tiny detached diamond'),
 2:(prism(3,19,4.5,10,2.7,WHITE)+prism(15,19,9,10,5.4)
    +path('M3 29 H29.4',WHITE,dashed=True,width=1.4),
    'Uniform factor2 in both footprint directions: width4.5 to9, depth2.7 to5.4; front height10 stays unchanged. White source and orange result share the same front baseline'),
 4:(prism(3,14,23,10,3)+rect(10,14,9,10,WHITE)+arrow('M7 6 H27','M23 3 L27 6 L23 9'),
    'Use an elongated prism with a white unchanged center and one directional scale cue'),
 3:(prism(3,13,23,15,3)+prism(5,18,8,8,3,WHITE)+arrow('M4 6 H25','M21 3 L25 6 L21 9'),
    'Use visibly unequal width, depth and height; white source remains a cube'),
 5:(diamond('M3 23 L13 18 L21 23 L11 28 Z')
    +path('M6 24 L15 22 M19 13 L26 5',WHITE,width=1.8)
    +points((6,24),(15,22),(19,13),(26,5),radius=1.9)
    +arrow('M6 14 Q8 5 15 5','M12 2 L15 5 L12 8'),
    'Show an object plane and both two-point reference/target segments; the points define orientation'),
 6:(diamond('M3 23 L13 18 L21 23 L11 28 Z')
    +path('M6 24 L15 22 L12 27 Z M20 5 L28 9 L20 13 Z',WHITE,width=1.7)
    +points((6,24),(15,22),(12,27),(20,5),(28,9),(20,13),radius=1.9)
    +arrow('M5 15 Q7 5 14 5','M11 2 L14 5 L11 8'),
    'Show two three-point triangles on an object context; retain three-point rotation cue'),
 8:(path('M5 4 V27',WHITE,width=2.2)+rect(7,22,20,5)
    +arrow('M10 5 Q27 6 25 19','M22 15 L25 19 L29 16'),
    'Restore perpendicular axis/object and a large quarter-turn arc, avoiding an unspecified cube orbit'),
 7:(rect(3,4,5,24,WHITE)+face('M13 28 Q12 11 26 5 L29 10 Q19 14 19 28 Z')
    +path('M15 21 L20 20 M17 14 L23 15',LIGHT,width=1.4),
    'Straight source bar beside a segmented bent bar, preserving bend rather than a plain curve'),
 9:(''.join(small(x,y,WHITE if n==0 else ORANGE) for n,(x,y) in enumerate(((3,14),(13,5),(23,14),(13,24))))+points((16,17),radius=1.9),
    'Use actual tiny cubes around a center; distinguish source from copied objects'),
 10:(standing()+prism(16,8,8,7,3,WHITE)+rect(17,24,10,4)
    +arrow('M22 17 V22','M19 19 L22 22 L25 19'),
    'Retain vertical construction-plane scaffold, lifted solid and flattened orange result'),
 11:(standing()+curved()+rect(5,9,5,5)+rect(5,20,5,5)
    +face('M20 9 Q23 6 26 9 V14 Q23 11 20 14 Z')
    +face('M20 19 Q23 16 26 19 V24 Q23 21 20 24 Z')
    +arrow('M12 17 H17','M14 14 L17 17 L14 20'),
    'Show two flat source motifs becoming curved motifs between base and destination surfaces'),
 12:(standing()+curved()+capsule(3,9)+capsule(18,9)
    +arrow('M12 22 H17','M14 19 L17 22 L14 25'),
    'Use the same undistorted capsule on both surfaces to make rigid placement explicit'),
 14:(standing()+curved()+face('M20 9 Q23 6 26 9 V14 Q23 11 20 14 Z')
    +face('M20 19 Q23 16 26 19 V24 Q23 21 20 24 Z')
    +face('M3 19 H12 V28 H3 Z',RED,.7,1.8)+path('M5 21 L10 26 M5 26 L10 21',WHITE,width=1.8),
    'Keep mapped geometry visible and show the original red break/update-off cue separately'),
 15:(face('M5 12 C5 1 27 1 27 12 Z')+path('M10 10 Q16 4 22 10',LIGHT)
    +rect(3,24,26,5,WHITE)+path('M7 26 Q11 20 15 26 M18 26 Q22 20 26 26',ORANGE,width=1.8)
    +arrow('M16 14 V22','M13 19 L16 22 L19 19'),
    'Use curved Flow surface above flat Base surface with curves pulled down, following command meaning'),
 0:(face('M3 3 H29 V29 H3 Z',ORANGE,.1,1.8)
    +''.join(face(f'M{x} {y-3} L{x+3} {y} L{x} {y+3} L{x-3} {y} Z',WHITE,.12,1.5) for x,y in ((9,9),(23,9),(16,16),(9,23),(23,23)))
    +path('M3 16 H8 M24 16 H29 M16 3 V8 M16 24 V29',ORANGE,width=1.8),
    'Use a repeating five-motif panel with a center motif; distinguish pattern from a simple four-cell array'),
 13:(plane()+prism(10,14,9,8,4)+arrow('M20 11 V3','M17 6 L20 3 L23 6'),
    'Seat a solid on a perspective surface and show its local normal direction'),
 16:(path('M3 27 Q9 12 28 15',WHITE,width=2)+prism(12,7,8,9,3)
    +path('M16 17 L16 22 H21',LIGHT,width=1.6),
    'Use solid upright to the local curve tangent with a right-angle cue'),
 18:(path('M16 3 V29',WHITE,width=1.6)
    +face('M12 6 Q4 10 4 27 H12 Z')+face('M20 6 Q28 10 28 27 H20 Z',LIGHT)
    +path('M12 14 H20',WHITE,dashed=True,width=1.4),
    'Keep the mirrored bent halves and central mirror plane; matching curvature signals symmetry'),
 19:(prism(5,11,15,16,7)+points((5,11),(12,4),(27,4),(20,11),(5,27),(20,27),(27,20),radius=1.9),
    'Use a filled prism with distinct grip points on actual corners instead of a hollow wire cube'),
 20:(cage(True)+face('M7 15 Q12 11 19 17 L17 25 Q12 20 7 23 Z',WHITE,.2,1.8)
    +points((29,14),(8,29),radius=1.9),
    'Warp cage edges and captured object together; align all cage lines with the deformed outline'),
 21:(path('M5 28 C5 10 11 6 29 10',WHITE,width=1.7)
    +small(3,24,WHITE)+small(8,12)+small(23,11),
    'Use source and copied solids sitting on one visible path; remove flat pixel squares'),
 24:(small(3,23,WHITE)+small(12,15)+small(21,7)
    +path('M4 29 L28 8',WHITE,dashed=True,width=1.4),
    'Three identical cubes in a straight diagonal sequence, closer to the familiar linear array cue'),
 23:(face('M3 7 Q16 2 29 7 V28 Q16 23 3 28 Z',WHITE,.05,1.5)
    +small(5,11,WHITE)+small(21,11)+small(5,24)+small(21,24),
    'Show four separated solids in surface rows/columns; broad spacing avoids merging at menu size'),
 26:(standing()+path('M14 28 Q15 12 28 7',WHITE,width=1.7)
    +small(12,24,WHITE)+small(18,15)+small(24,7),
    'Keep surface boundary and curved copy path together so this differs from array along a curve alone'),
 25:(small(3,12,WHITE)+small(18,12)+small(3,25)+small(18,25)
    +path('M10 8 H26 M8 17 V22',WHITE,dashed=True,width=1.4),
    'Replace skinny diamonds with four proportional cubes; regular rows/columns remain visible'),
 27:(standing()+path('M5 25 V8',WHITE,width=1.6)
    +face('M14 27 Q15 13 27 5 L30 9 Q20 17 20 27 Z')
    +path('M15 23 L20 23 M17 16 L23 17 M22 10 L27 12',LIGHT,width=1.4),
    'Straight source spine and band following curved target; show actual deformation rather than a transfer arrow'),
 28:(standing()+path('M4 12 H11 M7 5 V26 M4 21 H11',WHITE,width=1.4)
    +face('M15 27 Q15 12 27 4 L30 9 Q22 17 23 27 Z')
    +path('M16 21 L23 23 M19 13 L27 15',LIGHT,width=1.5),
    'Source surface grid and warped target grid explicitly distinguish surface mapping from curve mapping'),
 29:(plane()+face('M6 14 A10 10 0 0 1 26 14 A10 10 0 0 1 6 14 Z')
    +path('M6 14 Q16 21 26 14 M16 4 Q9 13 16 24 Q23 13 16 4',LIGHT,width=1.5),
    'Restore a large reference sphere seated on surface; avoid a floating arc that resembles rotate'),
 30:(points((5,7),(9,17),(4,27),radius=2)+path('M23 4 V29',WHITE,width=1.4)
    +points((23,7),(23,17),(23,27),color=ORANGE,radius=2)
    +arrow('M12 17 H19','M16 14 L19 17 L16 20'),
    'Scattered source points align to a datum with separated orange target points'),
 31:(face('M3 4 H29 V29 H3 Z',ORANGE,.1,1.8)+rect(3,20,9,9,WHITE)
    +path('M12 20 H26 V8 M12 20 L26 8',ORANGE,width=1.6)
    +arrow('M13 17 L25 5','M20 5 H25 V10'),
    'Retain planar square and reference corner; enlarge unequal two-axis rectangle instead of only a diagonal arrow'),
 32:(prism(3,15,23,10,3,WHITE)+rect(3,15,7,10)+path('M10 12 V28',LIGHT,dashed=True,width=1.4)
    +arrow('M4 5 H16','M12 2 L16 5 L12 8'),
    'Orange selected end of a white elongated solid; contrast selection with whole-object scaling'),
 33:(standing()+face('M17 5 L28 27 H13 Z')+path('M20 7 V26',LIGHT,width=1.4),
    'Use tapered solid silhouette against straight reference; preserve original taper-on-plane cue'),
 34:(standing()+face('M9 28 L16 5 H29 L22 28 Z')
    +path('M9 28 H22 M16 5 H29',LIGHT,width=1.6),
    'Skewed slab against vertical source frame; remove generic arrow that crowds the corner'),
 36:(standing()+face('M19 4 Q10 10 23 15 Q31 21 19 28 H27 Q14 20 20 15 Q27 9 27 4 Z')
    +path('M19 4 H27 M18 10 L25 12 M20 18 L27 21 M19 28 H27',LIGHT,width=1.5),
    'Twisted column with rotated cross-section stations, replacing misleading hourglass silhouette'),
 35:(face('M3 3 H29 V29 H3 Z',WHITE,.06,1.6)
    +path('M4 12 Q21 2 25 14 Q29 28 14 27 Q1 26 7 13 Q13 4 21 12 Q28 20 17 22 Q10 23 11 16 Q12 11 17 14',ORANGE,width=2.3)
    +path('M16 6 A10 10 0 0 1 26 16 M6 16 A10 10 0 0 0 16 26',LIGHT,dashed=True,width=1.4),
    'Show planar warp within boundary and two reference-circle arcs; distinguish Maelstrom from a free spiral curve'),
 37:(face('M4 8 L10 4 L19 17 L13 21 Z')+plane()+rect(13,25,13,3)
    +arrow('M23 6 Q29 10 24 19','M21 15 L24 19 L28 16'),
    'Tilted object is placed down on a selected flat face; floor is an actual plane, not only a line'),
 38:(standing()+rect(5,7,5,7)+plane()+face('M14 23 L21 20 L26 24 L19 27 Z')
    +arrow('M15 9 V18','M12 15 L15 18 L18 15'),
    'Two explicit perpendicular construction planes carrying the same object before and after remapping'),
 39:(face('M3 8 Q15 3 28 8 V27 Q15 22 3 27 Z',WHITE,.08,1.6)
    +points((15,19),color=ORANGE,radius=2.5)
    +arrow('M15 19 V3','M12 6 L15 3 L18 6')
    +arrow('M15 19 H27','M24 16 L27 19 L24 22')
    +arrow('M15 19 L5 27','M5 23 V27 H9'),
    'Selected control point and three separate local directions on a surface; reduce patch-arrow overlap'),
 40:(standing()+path('M3 28 H29',WHITE,width=1.6)
    +face('M8 6 C26 6 26 22 28 28 H8 Z')
    +path('M12 7 V28 M17 10 V28 M22 16 V28',LIGHT,width=1.4)
    +arrow('M9 4 H23','M19 1 L23 4 L19 7'),
    'Show a graded deformation of a point-rich surface rather than another curve-edit icon'),
 41:(cage()+prism(8,17,9,9,3,WHITE)
    +points((3,10),(10,3),(28,3),(21,29),radius=1.9),
    'Regular box cage enclosing an undeformed object with corner controls; distinguish from cage edit by shape'),
}

COMMANDS = {'TransformScaleScale3D': ('Scale 3D', 'Uniform scale on three axes', 1), 'TransformScaleScale2D': ('Scale 2D', 'Uniform scale in two directions', 2), 'TransformScaleScale1D': ('Scale 1D', 'Scale along one axis', 4), 'TransformScaleNonUniformScale': ('Non-uniform scale', 'Different lengths along three axes', 3), 'TransformOrient2Points': ('Orient: 2 points', 'Orient using two reference and target points', 5), 'TransformOrient3Points': ('Orient: 3 points', 'Orient using three reference and target points', 6), 'TransformRotate3D': ('Rotate 3D', 'Rotate around a specified spatial axis', 8), 'TransformBend': ('Bend', 'Bend an object along a spine arc', 7), 'TransformArrayPolar': ('Polar array', 'Copy objects around a center', 9), 'TransformProjecttoCPlane': ('Project to CPlane', 'Flatten objects onto the construction plane', 10), 'gvSmartFlow': ('Smart Flow', 'History-enabled morph between base and destination surfaces', 11), 'gvSmartFlowRigid': ('Smart Flow: rigid', 'Flow without distorting each object', 12), 'gvSmartFlowBreak': ('Break Smart Flow', 'Disable flow updates while retaining geometry', 14), 'gvSmartFlowPullDownCurves': ('Pull down curves', 'Pull curves from Flow surface to Base surface', 15), 'TransformSmartPattern': ('Smart Pattern', 'Arrange a pattern over a design surface', 0), 'TransformOrientOnSurface': ('Orient on surface', 'Orient objects using a surface normal', 13), 'TransformOrientPerpendiculartoCurve': ('Orient perpendicular', 'Orient an object perpendicular to a curve', 16), 'TransformSymmetry': ('Symmetry', 'Mirror with tangent continuity', 18), 'TransformSolidPtOn': ('Solid points on', 'Show solid edge editing grips', 19), 'TransformCageEditingCageEdit': ('Cage edit', 'Deform an object with cage controls', 20), 'TransformArrayAlongCurve': ('Array along curve', 'Copy objects at intervals along a curve', 21), 'TransformArrayArrayLinear': ('Linear array', 'Copy objects in one direction', 24), 'TransformArrayAlongSurface': ('Array on surface', 'Copy objects in surface rows and columns', 23), 'TransformArrayAlongCurveonSurface': ('Array curve on surface', 'Copy and rotate objects along a surface curve', 26), 'TransformArrayRectangular': ('Rectangular array', 'Copy in columns, rows and levels', 25), 'TransformFlowAlongCurve': ('Flow along curve', 'Map objects from a base curve to a target curve', 27), 'TransformFlowAlongSurface': ('Flow along surface', 'Map objects from a base surface to a target surface', 28), 'TransformSplop': ('Splop', 'Wrap an object onto a surface using a reference sphere', 29), 'TransformSetPoints': ('Set points', 'Align or set positions of control points', 30), 'TransformScaleScaleByPlane': ('Scale by plane', 'Different scales in two plane directions', 31), 'TransformStretch': ('Stretch', 'Scale a selected object region in one direction', 32), 'TransformTaper': ('Taper', 'Deform toward or away from an axis', 33), 'TransformShear': ('Shear', 'Skew parallel to an axis', 34), 'TransformTwist': ('Twist', 'Rotate sections along an axis', 36), 'TransformMaelstrom': ('Maelstrom', 'Twist a region between two reference circles', 35), 'BuilderMakeDownFacing': ('Make down-facing', 'Place the selected face flat at the bottom', 37), 'TransformOrientRemaptoCPlane': ('Remap to CPlane', 'Reorient to a different construction plane', 38), 'TransformMoveUVN': ('Move UVN', 'Move surface points along U, V and normal', 39), 'TransformSoftMove': ('Soft move', 'Move nearby control points with a soft falloff', 40), 'TransformCageEditingCreateCage': ('Create cage', 'Create a box cage for later deformation', 41)}

TRANSFORM_ICONS = {k: (*v, DESIGNS[v[2]][0]) for k,v in COMMANDS.items()}

def svg(key):
    label, meaning, spec, geometry = TRANSFORM_ICONS[key]
    return (f'<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32">\n'
            f'<title>{html.escape(label)}</title>\n<desc>{html.escape(meaning)}. '
            f'OpenMatrix9 {STYLE}; authored vector geometry.</desc>\n{geometry}\n</svg>\n')


def write_asset(output_root, key, tooltip=None):
    relative = f'icons/transform-minimal/{key}.svg'
    target = Path(output_root) / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    raw = svg(key).encode('utf-8')
    target.write_bytes(raw)
    label, meaning, spec, _ = TRANSFORM_ICONS[key]
    metadata = {'image': relative, 'tooltip': tooltip or key,
                'source': 'OpenMatrix9-authored-svg', 'mapping': 'authored-command-symbol', 'status': 'resolved'}
    record = {'symbol': key, 'label': label, 'meaning': meaning, 'style': STYLE,
              'mapping': metadata['mapping'], 'sha256': hashlib.sha256(raw).hexdigest(),
              'palette': [c for c in (ORANGE, LIGHT, WHITE, RED) if c in raw.decode()],
              'spec_id': f'OM9-TRANSFORM-{spec:03}' if spec else 'OM9-MATRIXTOOLS-001',
              'semantic_source': 'local Transform spec' if spec else 'local Matrix Tools Smart Pattern spec',
              'design_note': DESIGNS[spec][1]}
    return metadata, {k:v for k,v in record.items() if k not in ('reference_image','reference_type','reference_images','palette_reference','reference_palette')}


def transform_keys(resources):
    menu = configparser.ConfigParser(interpolation=None)
    menu.optionxform = str
    menu.read(Path(resources) / 'menu/MainMenu.ini', encoding='utf-8')
    groups = [s for s in menu.values() if s.get('Name') == 'Transform']
    if len(groups) != 1:
        raise ValueError('Expected one Transform group')
    group = groups[0]
    keys = [v for k,v in group.items() if re.fullmatch(r'Icon\d+', k)]
    if len(keys) != int(group['IconCount']) or set(keys) != set(TRANSFORM_ICONS):
        raise ValueError('Transform menu and authored catalog disagree')
    return keys


def contact_sheet(keys):
    columns, cell_w, cell_h = 8, 160, 120
    height = 64 + ((len(keys)+columns-1)//columns)*cell_h
    parts = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{columns*cell_w}" height="{height}">',
             '<rect width="100%" height="100%" fill="#333333"/>',
             '<text x="16" y="28" font-family="Arial,sans-serif" font-size="20" fill="white">OpenMatrix9 — Transform / minimal SVG</text>',
             '<text x="16" y="48" font-family="Arial,sans-serif" font-size="12" fill="#cccccc">40 icons · orange-red objects · peach highlights · white actions · 64px and 24px samples</text>']
    for i,key in enumerate(keys):
        x,y = (i%columns)*cell_w,64+(i//columns)*cell_h
        label,_,_,geometry = TRANSFORM_ICONS[key]
        parts += [f'<g transform="translate({x} {y})">',
                  '<rect x="4" y="4" width="152" height="112" rx="4" fill="#696969"/>',
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
    report['transform_icon_style'] = {'name': STYLE, 'count': count,
        'source': 'OpenMatrix9-authored-svg', 'generator': 'tools/transform_icons.py',
        'geometry': 'authored geometric primitives; no tracing or embedded bitmaps',
        'palette_reference': 'orange-red #FF5B05 objects; peach #FF9566 highlights; white source/actions',
        'priority': 'authored Transform symbols precede legacy image sources'}
    prefix='authored Transform SVG first; '
    if not report.get('asset_policy','').startswith(prefix):
        report['asset_policy']=prefix+report.get('asset_policy','')


def install(project_root):
    root=Path(project_root)
    resources=root/'Resources'
    keys=transform_keys(resources)
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
    ini_path.write_text(original,encoding='utf-8')
    manifest_path.write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    gallery=root/'docs/images/transform-icons-minimal.svg'
    gallery.parent.mkdir(parents=True,exist_ok=True)
    gallery.write_text(contact_sheet(keys),encoding='utf-8')
    return {'count':len(keys),'style':STYLE,'preview':str(gallery)}


if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--project-root',type=Path,default=Path(__file__).resolve().parents[1])
    args=parser.parse_args()
    print(json.dumps(install(args.project_root)))
