"""Authored minimal Curve symbols. Geometry is constructed, not bitmap-traced.

Run `python tools/curve_icons.py` to install the 58 SVGs and update their menu
bindings. Only Curve bindings are changed; no reference images are removed.
The generator needs Python's standard library only.
"""
import argparse
import configparser
import hashlib
import html
import json
from pathlib import Path
import re

YELLOW = '#FFFF05'
WHITE = '#FFFFFF'
GOLD = '#FFD705'
CYAN = '#59CBE8'
STYLE = 'minimal-curve-v4-cyan'


def path(d, color=YELLOW, dashed=False, width=2.6):
    # 32-unit designs are displayed at 24px: keep secondary strokes visible.
    width = max(width, 1.4)
    dash = ' stroke-dasharray="2 3"' if dashed else ''
    return (f'<path d="{d}" fill="none" stroke="{color}" stroke-width="{width}"'
            f' stroke-linecap="round" stroke-linejoin="round"{dash}/>')


def points(*xy, radius=2, color=WHITE):
    radius = max(1.9, radius * 1.15)
    return ''.join(f'<circle cx="{x}" cy="{y}" r="{radius}" fill="{color}"/>'
                   for x, y in xy)


def ring(x, y, radius, color=YELLOW):
    return f'<circle cx="{x}" cy="{y}" r="{radius}" fill="none" stroke="{color}" stroke-width="2.6"/>'


def arrow(d, head, color=WHITE):
    return path(d, color, width=1.8) + path(head, color, width=1.8)


def surface(mesh=False, grid=False):
    boundary = 'M4 14 Q12 7 26 11 L28 25 Q16 21 6 28 Z'
    lines = ('M4 14 L17 23 L26 11 M6 28 L26 11' if mesh else
             ('M5 20 Q16 14 27 18 M15 10 L17 24' if grid else ''))
    return (face(boundary) + (path(lines, CYAN, width=1.4) if lines else ''))


def face(d, fill_opacity=0.16):
    return path(d, CYAN, width=1.6).replace('fill="none"',
        f'fill="{CYAN}" fill-opacity="{fill_opacity}"')


def box():
    return path('M6 11 L20 5 L27 12 L13 18 Z M6 11 V24 L13 29 L27 23 V12 M13 18 V29', CYAN, width=1.6)


def pencil():
    return path('M19 12 L25 6 L28 9 L22 15 L18 16 Z M24 7 L27 10', WHITE, width=1.25)


def cross(x, y, color=WHITE):
    return path(f'M{x-3} {y} H{x+3} M{x} {y-3} V{y+3}', color, width=1.25)


# Each entry carries its own semantic design; shared primitives ensure consistent
# stroke weights, margins and colors, rather than a generic icon for every tool.
CURVE_ICONS = {
    'CurvePolylinePolyline': ('Polyline', 'Connected straight segments with three vertices',
        path('M5 25 L13 7 L27 20') + points((5, 25), (13, 7), (27, 20))),
    'CurveLineSingleLine': ('Line', 'A straight segment defined by two endpoints',
        path('M7 25 L25 7') + points((7, 25), (25, 7))),
    'CurveFreeFormInterpolatePoints': ('Interpolate points', 'Smooth curve passing through picked points',
        path('M5 24 C5 15 13 22 16 16 S26 15 27 7') + points((5, 24), (16, 16), (27, 7))),
    'CurveRectangleCornertoCorner': ('Rectangle', 'Rectangle defined by opposite corners',
        path('M6 7 H26 V25 H6 Z') + points((6, 25), (26, 7))),
    'CurveCircleCenterRadius': ('Circle', 'Circle with center and radius',
        ring(16, 16, 10) + path('M16 16 L23 9', WHITE, width=1) + points((16, 16), (23, 9), radius=1.7)),
    'CurveEllipseFromCenter': ('Ellipse', 'Ellipse with center and two principal axes',
        '<ellipse cx="16" cy="16" rx="12" ry="8" fill="none" stroke="#FFFF05" stroke-width="2.6"/>'
        + path('M16 8 V16 H28', WHITE, width=1) + points((16, 16), (16, 8), (28, 16), radius=1.5)),
    'CurveArcCenterStartAngle': ('Arc: center / angle', 'Arc with two radii meeting at its center',
        path('M6 7 A19 19 0 0 1 25 26') + path('M6 7 V26 H25', WHITE, width=1)
        + points((6, 7), (6, 26), (25, 26), radius=1.7)),
    'CurveArcStartEndDirection': ('Arc: end / tangent', 'Arc between endpoints with a starting tangent',
        path('M6 25 Q6 6 26 6') + arrow('M6 25 V9', 'M3 13 L6 9 L9 13') + points((6, 25), (26, 6))),
    'OthersCurveRebuild': ('Rebuild curve', 'Reconstructed smooth curve with a regular control polygon',
        path('M4 25 C8 5 24 30 28 13') + points((4, 25), (10, 17), (21, 22), (28, 13), radius=1.6)
        + arrow('M8 10 C12 2 23 2 26 8', 'M22 8 H26 V4')),
    'CurveCurveEditToolsRefitToTolerance': ('Refit to tolerance', 'Refitted curve inside a pair of tolerance bounds',
        path('M4 24 C12 24 18 8 28 8')
        + path('M4 20 C12 20 18 4 28 4 M4 28 C12 28 18 12 28 12', WHITE, dashed=True, width=1)),
    'CurveBlendCurvesBlendCurves': ('Blend curves', 'Smooth transition joining two curve ends',
        path('M4 25 Q9 25 10 20 M22 12 Q23 7 28 7', WHITE)
        + path('M10 20 C12 11 20 21 22 12') + points((10, 20), (22, 12), radius=1.6)),
    'CurveBlendCurvesBlendCrv': ('Blend with handles', 'Smooth transition with editable tangent handles',
        path('M4 25 Q9 25 10 20 M22 12 Q23 7 28 7', WHITE)
        + path('M10 20 C10 8 22 24 22 12') + path('M10 20 V8 M22 12 V24', WHITE, width=1)
        + points((10, 8), (22, 24), radius=1.6)),
    'CurveCurveFrom2Views': ('Curve from 2 views', 'Two perpendicular projections combine into a spatial curve',
        path('M3 13 V4 H12 M20 4 H29 V13', WHITE, width=1.4)
        + path('M5 11 Q8 4 11 9 M22 6 Q29 8 23 11', WHITE, width=1.8)
        + path('M6 27 C12 14 20 31 27 17') + points((6, 27), (27, 17), radius=1.5)),
    'BuilderCrv2ViewHistory': ('2-view history', 'Two-view curve with a history clock',
        path('M3 13 V4 H12 M20 4 H29 V13', WHITE, width=1.4)
        + path('M5 11 Q8 4 11 9 M22 6 Q29 8 23 11', WHITE, width=1.8)
        + path('M5 27 C10 14 17 24 19 14')
        + ring(23, 23, 6, GOLD) + path('M23 19 V23 L26 25', WHITE, width=1.5)),
    'CurveCurveFromObjectsPullback': ('Pull curve to surface', 'Curve pulled to the nearest point on a curved surface',
        surface() + path('M7 4 Q15 1 25 5', WHITE) + path('M7 24 Q17 18 27 22')
        + arrow('M22 7 L14 20', 'M13 15 L14 20 L19 18')),
    'CurveCurveFromObjectsProject': ('Project curve', 'Parallel projection rays from a curve onto a surface',
        surface() + path('M5 4 Q16 1 25 4', WHITE) + path('M7 24 Q17 18 27 22')
        + arrow('M9 7 V21', 'M6.5 18 L9 21 L11.5 18')
        + arrow('M23 7 V20', 'M20.5 17 L23 20 L25.5 17')),
    'CurveCurveFromObjectsIntersection': ('Surface intersection', 'Curve where two surfaces cross',
        surface() + face('M14 4 L23 7 L18 28 L10 24 Z')
        + path('M12 22 Q15 16 21 13')),
    'CurveOffsetCurve': ('Offset curve', 'Parallel curve at a chosen distance',
        path('M4 10 Q16 3 28 10', WHITE) + path('M4 25 Q16 18 28 25')
        + arrow('M16 9 V20', 'M12.5 16.5 L16 20 L19.5 16.5')),
    'CurveOffsetOffsetCrvOnSrf': ('Offset on surface', 'Two parallel curves constrained to a surface',
        face('M3 9 L23 4 L29 26 L9 30 Z')
        + path('M6 15 Q16 9 25 12') + path('M9 25 Q19 19 28 22')
        + arrow('M16 14 L18 21', 'M14 18.5 L18 21 L20 17')),
    'CurveCurveFromObjectsExtractIsocurve': ('Extract isocurve', 'One parameter curve highlighted on a surface',
        surface(grid=True) + path('M5 20 Q16 14 27 18') + points((5, 20), (27, 18), radius=1.5)),
    'CurveFilletCurves': ('Fillet curves', 'Rounded connection between two separate curves',
        path('M5 27 V18 M14 9 H27', WHITE) + path('M5 18 Q5 9 14 9')
        + points((5, 18), (14, 9), radius=1.5)),
    'CurveFilletCorners': ('Fillet corners', 'Closed outline with rounded corners',
        path('M12 6 H21 M27 12 V20 M21 26 H12 M6 20 V12', WHITE)
        + path('M6 12 Q6 6 12 6 M21 6 Q27 6 27 12 M27 20 Q27 26 21 26 M12 26 Q6 26 6 20')),
    'CurveChamferCurves': ('Chamfer curves', 'Straight beveled connection replacing a sharp corner',
        path('M5 27 V17 M15 7 H27', WHITE) + path('M5 17 L15 7')
        + points((5, 17), (15, 7), radius=1.5)),
    'CurveCurveFromObjectsCreateUVCurves': ('Create UV curves', 'Unwrap a surface curve onto a flat parameter grid',
        face('M3 7 Q8 1 13 7 V17 Q8 11 3 17 Z')
        + path('M4 11 Q8 6 12 11') + path('M19 17 H29 V29 H19 Z', WHITE, width=1.8)
        + path('M21 25 Q24 19 27 25') + arrow('M17 9 L24 14', 'M19 14 H24 V9')),
    'CurveCurveFromObjectsApplyUVCurves': ('Apply UV curves', 'Map a flat parameter curve back onto a surface',
        path('M19 17 H29 V29 H19 Z', WHITE, width=1.8) + path('M21 25 Q24 19 27 25')
        + face('M3 7 Q8 1 13 7 V17 Q8 11 3 17 Z')
        + path('M4 11 Q8 6 12 11') + arrow('M24 14 L17 9', 'M17 14 V9 H22')),
    'CurveFreeFormThroughPoints': ('Curve through points', 'Smooth curve through an existing point set',
        path('M4 24 C6 13 12 24 16 15 S25 18 28 6')
        + points((4, 24), (9, 19), (16, 15), (22, 13), (28, 6), radius=1.7)),
    'CurveExtendCurveExtendCurve': ('Extend curve', 'Existing curve with an extended continuation',
        path('M4 26 Q14 26 18 15', WHITE) + path('M18 15 Q20 9 27 5')
        + arrow('M22 9 L27 5', 'M22 5 H27 V10') + points((18, 15), radius=1.5)),
    'CurvePolygonCenterRadius': ('Polygon', 'Regular polygon defined by center and radius',
        path('M16 4 L27 10 V22 L16 28 L5 22 V10 Z') + path('M16 16 L27 10', WHITE, width=1)
        + points((16, 16), (27, 10), radius=1.5)),
    'CurveSpiral': ('Spiral', 'A planar curve winding outward from its center',
        path('M16 16 C12 13 11 19 16 20 C24 22 25 9 16 8 C3 6 2 27 17 28 C28 29 31 17 27 8')
        + points((16, 16), radius=1.5)),
    'CurveHelix': ('Helix', 'Three-dimensional turns around a vertical axis',
        path('M16 3 V29', WHITE, dashed=True, width=1)
        + path('M7 7 C7 2 25 2 25 7 S7 12 7 16 S25 20 25 25 S7 30 7 25')
        + path('M25 7 C25 11 7 11 7 16 M7 16 C7 20 25 20 25 25', WHITE, dashed=True, width=1)),
    'CurvePointObjectSinglePoint': ('Point', 'A single point located by a small crosshair',
        path('M16 5 V10 M16 22 V27 M5 16 H10 M22 16 H27', WHITE, width=1)
        + points((16, 16), radius=3)),
    'CurvePointObjectMarkCurveStart': ('Mark curve start', 'First endpoint marked on a curve',
        path('M5 25 C8 5 22 26 27 7') + points((5, 25), radius=2.8)
        + path('M5 25 V12 L11 15 L5 18', WHITE, width=1.25)),
    'CurvePointObjectMarkCurveEnd': ('Mark curve end', 'Last endpoint marked on a curve',
        path('M5 25 C8 5 22 26 27 7') + points((27, 7), radius=2.8)
        + path('M27 7 V20 L21 17 L27 14', WHITE, width=1.25)),
    'CurveCurveEditToolsAdjustClosedCurveSeam': ('Adjust closed seam', 'Move the seam point along a closed curve',
        ring(16, 17, 10) + points((26, 17), radius=2.5)
        + arrow('M25 10 Q21 4 13 4', 'M17 2 L13 4 L17 6')),
    'CurveFreeFormContinueInterpCurve': ('Continue interpolate', 'Extend an interpolated curve through further points',
        path('M4 25 Q7 9 16 16', WHITE) + path('M16 16 Q24 23 28 7')
        + points((4, 25), (16, 16), (25, 17), (28, 7), radius=1.7)),
    'OthersCurveDivideCurve': ('Divide curve', 'Equal divisions marked along a curve',
        path('M4 18 H28') + path('M6 13 V23 M16 13 V23 M26 13 V23', WHITE, width=1.8)
        + points((6, 18), (16, 18), (26, 18), radius=1.7)),
    'CurveCurveEditToolsCurveBoolean': ('Curve boolean', 'Overlapping regions used to trim, split and join curves',
        ring(12, 16, 8, WHITE) + ring(21, 16, 8, WHITE)
        + path('M16.5 9.4 A8 8 0 0 1 16.5 22.6 A8 8 0 0 1 16.5 9.4')),
    'CurveTweenCurves': ('Tween curves', 'Intermediate curve between two source curves',
        path('M4 10 Q14 3 28 10 M4 27 Q14 20 28 27', WHITE, width=1.5)
        + path('M4 18 Q14 11 28 18') + arrow('M16 7 V12', 'M14 10 L16 12 L18 10')
        + arrow('M16 24 V19', 'M14 21 L16 19 L18 21')),
    'CurveCurveFromObjectsDuplicateEdge': ('Duplicate edge', 'A single edge copied from a solid',
        path('M5 17 L19 11 L27 17 L13 23 Z M5 17 V25 L13 29 L27 23 V17 M13 23 V29', CYAN, width=1.4)
        + path('M5 17 L19 11') + path('M5 6 L19 2')
        + arrow('M12 12 V6', 'M9.5 8.5 L12 6 L14.5 8.5')),
    'CurveCurveFromObjectsDuplicateBorder': ('Duplicate border', 'Closed boundary copied from a surface',
        face('M3 19 Q10 15 18 18 L19 29 Q11 25 4 29 Z', fill_opacity=0.55)
        + path('M3 19 Q10 15 18 18 L19 29 Q11 25 4 29 Z')
        + path('M12 5 Q19 1 27 4 L28 15 Q20 11 13 15 Z')
        + arrow('M23 25 V17', 'M20 20 L23 17 L26 20')),
    'CurveFreeFormControlPoints': ('Control-point curve', 'Smooth curve controlled by an off-curve control polygon',
        path('M5 25 C5 5 27 27 27 7') + path('M5 25 V5 L27 27 V7', WHITE, dashed=True, width=1)
        + '<g fill="#FFFFFF"><rect x="3" y="3" width="4" height="4"/><rect x="25" y="25" width="4" height="4"/></g>'
        + points((5, 25), (27, 7), radius=1.5)),
    'CurveFreeFormContinueCurve': ('Continue control curve', 'Extend a control-point curve with a new control handle',
        path('M4 25 Q9 8 17 17', WHITE) + path('M17 17 Q27 28 27 7')
        + path('M17 17 L27 28 V7', WHITE, dashed=True, width=1)
        + '<rect x="25" y="26" width="4" height="4" fill="#FFFFFF"/>'
        + points((17, 17), (27, 7), radius=1.5)),
    'CurveGVExtractIsocurve': ('Extract angle isocurve', 'A surface curve extracted at a specified angle',
        surface() + path('M7 25 L24 12') + path('M7 25 H24', WHITE, width=1.8)
        + path('M18 25 A11 11 0 0 0 15.7 18.3', WHITE, width=1.8)),
    'CurveCrossSectionProfiles': ('Cross-section profiles', 'A cross-section curve through several parallel profiles',
        path('M4 26 Q7 4 10 26 M13 26 Q16 4 19 26 M22 26 Q25 4 28 26', WHITE, width=1.8)
        + path('M7 15 H25') + points((7, 15), (16, 15), (25, 15), radius=1.5)),
    'CurveFreeFormSketch': ('Sketch', 'Freehand curve drawn with a pencil',
        path('M4 25 Q7 16 11 22 T19 21 Q23 20 22 16') + pencil()),
    'CurveFreeFormSketchonSurface': ('Sketch on surface', 'Freehand curve constrained to a curved surface',
        surface() + path('M5 24 Q9 17 13 23 T22 19') + pencil()),
    'CurveFreeFormSketchonPolygonMesh': ('Sketch on mesh', 'Freehand curve on a triangulated mesh',
        face('M3 12 L14 7 L27 15 L24 28 L5 28 Z')
        + path('M3 12 L24 28 M14 7 L5 28 M14 7 L24 28', CYAN, width=1.6)
        + path('M5 24 Q9 17 13 23 T22 19') + pencil()),
    'BuilderPolylineOnSurface': ('Polyline on surface', 'Straight segments with picked vertices on a surface',
        surface() + path('M6 24 L14 13 L26 21') + points((6, 24), (14, 13), (26, 21), radius=1.7)),
    'CurveFreeFormInterpolateonSurface': ('Interpolate on surface', 'Smooth interpolated curve through points on a surface',
        surface() + path('M6 24 C8 14 12 21 16 17 S23 24 26 14')
        + points((6, 24), (16, 17), (26, 14), radius=1.7)),
    'CurveConvertCurveToLines': ('Convert curve to lines', 'Smooth curve approximated by connected straight segments',
        path('M4 25 C6 7 24 29 28 5', WHITE, dashed=True, width=1)
        + path('M4 25 L10 16 L19 19 L25 15 L28 5')
        + points((4, 25), (10, 16), (19, 19), (25, 15), (28, 5), radius=1.4)),
    'CurveCurveFromObjectsSection': ('Section curve', 'Profile where a cutting plane intersects a solid',
        box() + path('M7 17 L14 22 L26 17 L19 12 Z')),
    'CurveCurveEditToolsMatch': ('Match curve ends', 'Align endpoints and tangent directions of two curves',
        path('M4 25 Q8 16 16 16 M16 16 Q24 16 28 7')
        + path('M10 16 H22', WHITE, dashed=True, width=1) + points((16, 16), radius=2.5)
        + arrow('M10 9 L15 13', 'M11 13 H15 V9')),
    'CurveCurveFromObjectsSilhouette': ('Silhouette', 'Outer contour of a solid from a viewing direction',
        box() + path('M6 11 L20 5 L27 12 V23 L13 29 L6 24 Z')),
    'CurveCurveFromObjectsExtractWireframe': ('Extract wireframe', 'All visible edge curves extracted from a solid',
        path('M6 11 L20 5 L27 12 L13 18 Z M6 11 V24 L13 29 L27 23 V12 M13 18 V29')
        + path('M20 5 V17 L6 24 M20 17 L27 23', YELLOW, dashed=True, width=1)),
    'CurveCurveEditToolsSoftEdit': ('Soft edit curve', 'Local smooth deformation with a falloff region',
        path('M4 24 Q16 19 28 24', WHITE, dashed=True, width=1)
        + path('M4 24 C10 24 10 11 16 11 S22 24 28 24')
        + arrow('M16 20 V12', 'M13 15 L16 12 L19 15')
        + points((16, 11), radius=1.8)),
    'CurveOffsetOffsetCrvNormalToSurface': ('Offset along normal', 'Curve displaced perpendicular to a surface',
        face('M3 23 L23 18 L29 27 L9 30 Z')
        + path('M7 25 Q16 19 26 23', WHITE) + path('M5 7 Q14 1 24 5')
        + arrow('M16 20 V6', 'M12.5 10 L16 6 L19.5 10')),
    'CurveBlendCurvesArcBlend': ('Arc blend', 'Two circular arcs connecting curve ends with adjustable bulge',
        path('M3 25 H6 M26 5 V2', WHITE)
        + path('M6 25 V15 H26 V5', WHITE, dashed=True, width=1.4)
        + path('M6 25 A10 10 0 0 0 16 15 A10 10 0 0 1 26 5')
        + points((6, 25), (26, 5), radius=1.7)),
    'CurveCurveFromObjectsIntersectTwoSets': ('Intersect two sets', 'Intersection points between two sets of curves',
        path('M5 5 V27 M14 5 V27') + path('M3 11 H29 M3 21 H29', WHITE, width=1.5)
        + points((5, 11), (5, 21), (14, 11), (14, 21), radius=2)),
}


def svg(key):
    label, meaning, geometry = CURVE_ICONS[key]
    return (f'<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32">\n'
            f'  <title>{html.escape(label)}</title>\n'
            f'  <desc>{html.escape(meaning)}. OpenMatrix9 {STYLE}; authored vector geometry.</desc>\n'
            f'  {geometry}\n</svg>\n')


def write_asset(output_root, key, tooltip=None):
    relative = f'icons/curve-minimal/{key}.svg'
    target = Path(output_root) / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    content = svg(key).encode('utf-8')
    target.write_bytes(content)
    metadata = {'image': relative, 'tooltip': tooltip or key,
                'source': 'OpenMatrix9-authored-svg',
                'mapping': 'authored-command-symbol', 'status': 'resolved'}
    record = {'symbol': key, 'label': CURVE_ICONS[key][0],
              'meaning': CURVE_ICONS[key][1], 'style': STYLE,
              'mapping': metadata['mapping'], 'sha256': hashlib.sha256(content).hexdigest(),
              'palette': [YELLOW, WHITE] + ([GOLD] if GOLD in svg(key) else [])
                         + ([CYAN] if CYAN in svg(key) else [])}
    return metadata, {k:v for k,v in record.items() if k not in ('reference_image','reference_type','reference_images','palette_reference','reference_palette')}


def curve_keys(resources):
    menu = configparser.ConfigParser(interpolation=None)
    menu.optionxform = str
    menu.read(Path(resources) / 'menu/MainMenu.ini', encoding='utf-8')
    groups = [s for s in menu.values() if s.get('Name') == 'Curve']
    if len(groups) != 1:
        raise ValueError('Expected exactly one Curve menu')
    group = groups[0]
    keys = [value for name, value in group.items() if re.fullmatch(r'Icon\d+', name)]
    if len(keys) != int(group['IconCount']) or set(keys) != set(CURVE_ICONS):
        raise ValueError('Curve menu and authored icon catalog disagree')
    return keys


def contact_sheet(keys):
    """Show enlarged vectors beside actual 24px menu icons on a neutral gray."""
    columns, cell_w, cell_h = 8, 144, 120
    height = 64 + ((len(keys) + columns - 1) // columns) * cell_h
    parts = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{columns*cell_w}" height="{height}" viewBox="0 0 {columns*cell_w} {height}">',
             f'<rect width="100%" height="100%" fill="#333333"/>',
             '<text x="16" y="28" font-family="Arial,sans-serif" font-size="20" fill="white">OpenMatrix9 — Curve / minimal SVG</text>',
             '<text x="16" y="48" font-family="Arial,sans-serif" font-size="12" fill="#cccccc">58 icons · yellow curves · cyan surfaces #59CBE8 · white points/arrows · 64px and 24px samples</text>']
    for index, key in enumerate(keys):
        x, y = (index % columns) * cell_w, 64 + (index // columns) * cell_h
        geometry = CURVE_ICONS[key][2]
        parts.extend([f'<g transform="translate({x} {y})">',
                      '<rect x="4" y="4" width="136" height="112" rx="4" fill="#696969"/>',
                      f'<text x="12" y="20" font-family="Arial,sans-serif" font-size="11" fill="#dddddd">{index+1:02}</text>',
                      f'<g transform="translate(22 24) scale(2)">{geometry}</g>',
                      f'<g transform="translate(104 45) scale(0.75)">{geometry}</g>'])
        label = CURVE_ICONS[key][0]
        lines = []
        for word in label.split():
            if lines and len(lines[-1]) + len(word) + 1 <= 21:
                lines[-1] += ' ' + word
            else:
                lines.append(word)
        for n, line in enumerate(lines):
            parts.append(f'<text x="72" y="{98+n*12}" text-anchor="middle" font-family="Arial,sans-serif" font-size="10.5" fill="white">{html.escape(line)}</text>')
        parts.append('</g>')
    parts.append('</svg>\n')
    return '\n'.join(parts)


def describe_style(report, count):
    report['curve_icon_style'] = {
        'name': STYLE, 'count': count, 'source': 'OpenMatrix9-authored-svg',
        'generator': 'tools/curve_icons.py', 'geometry': 'authored geometric primitives; no tracing or embedded bitmaps',
        'palette_reference': 'selected yellow curves/white markers/history gold; user-selected cyan #59CBE8 surface/object support',
        'priority': 'authored Curve symbols precede shifted/original art; other groups retain existing policy'}
    policy_prefix = 'authored Curve SVG first; '
    if not report.get('asset_policy', '').startswith(policy_prefix):
        report['asset_policy'] = policy_prefix + report.get('asset_policy', '')
    note_prefix = 'Curve symbols use authored geometry and precede legacy image sources. '
    if not report.get('mapping_note', '').startswith(note_prefix):
        report['mapping_note'] = note_prefix + report.get('mapping_note', '')


def install(project_root):
    root = Path(project_root)
    resources = root / 'Resources'
    keys = curve_keys(resources)
    ini_path = resources / 'menu/icons.ini'
    original = ini_path.read_text(encoding='utf-8')
    ini = configparser.ConfigParser(interpolation=None)
    ini.read_string(original)
    manifest_path = resources / 'menu/source-manifest.json'
    report = json.loads(manifest_path.read_text(encoding='utf-8'))
    # Validate every binding before writing any product files.
    for key in keys:
        if not ini.has_section(key):
            raise ValueError(f'Missing menu binding: {key}')
    for key in keys:
        metadata, record = write_asset(resources, key, ini[key].get('tooltip'))
        if ini[key].get('feature_id'):
            metadata['feature_id'] = ini[key]['feature_id']
        section = f'[{key}]\n' + ''.join(f'{name} = {value}\n' for name, value in metadata.items()) + '\n'
        pattern = rf'^\[{re.escape(key)}\]\n.*?(?=^\[|\Z)'
        original, replacements = re.subn(pattern, lambda _: section, original,
                                        flags=re.MULTILINE | re.DOTALL)
        if replacements != 1:
            raise ValueError(f'Expected one binding for {key}, got {replacements}')
        for field in ('shifted_icons', 'original_buttons', 'named_crops', 'missing', 'auxiliary_missing'):
            report.get(field, {}).pop(key, None)
        report.setdefault('authored_symbols', {})[key] = record
    describe_style(report, len(keys))
    ini_path.write_text(original, encoding='utf-8')
    manifest_path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    gallery = root / 'docs/images/curve-icons-minimal.svg'
    gallery.parent.mkdir(parents=True, exist_ok=True)
    gallery.write_text(contact_sheet(keys), encoding='utf-8')
    return {'count': len(keys), 'style': STYLE, 'preview': str(gallery)}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--project-root', type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    print(json.dumps(install(args.project_root)))
