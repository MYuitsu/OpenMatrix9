"""Authored, deterministic Solid menu SVGs; no bitmap tracing.

Run python tools/solid_icons.py. Only the Solid menu bindings are replaced.
"""
import argparse
import configparser
import hashlib
import html
import json
from pathlib import Path
import re

from curve_icons import WHITE, path, arrow, points

# Flat palette sampled from the selected original Solid reference icons.
PURPLE = '#A668D1'
HIGHLIGHT = '#FFD166'

STYLE = 'minimal-solid-v3-warm-highlight'


def face(d, color=PURPLE, opacity=.18, width=2.2):
    return path(d, color, width=width).replace('fill="none"',
        f'fill="{color}" fill-opacity="{opacity}"')


def ellipse(cx, cy, rx, ry, color=PURPLE, width=2.2):
    return (f'<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" '
            f'fill="none" stroke="{color}" stroke-width="{width}"/>')


def cube():
    return (face('M4 11 L19 5 L28 11 L13 17 Z')
            + face('M4 11 L13 17 V28 L4 22 Z')
            + face('M13 17 L28 11 V22 L13 28 Z'))


def plate():
    return (face('M3 15 L22 7 L29 14 L10 22 Z')
            + path('M3 15 V22 L10 29 L29 21 V14 M10 22 V29', PURPLE, width=1.8))


def cylinder():
    return (face('M6 9 V24 C6 30 26 30 26 24 V9')
            + ellipse(16, 9, 10, 4))


def hole(x, y, color=HIGHLIGHT):
    return ellipse(x, y, 3.2, 2.4, color)


# Spec IDs are explicit: their ordering differs from the menu.
SOLID_ICONS = {
    'SolidUnion': ('Boolean union', 'Combine solid objects into one volume', 3,
        face('M3 10 H14 V5 H27 V22 H17 V27 H3 Z', HIGHLIGHT)
        + path('M3 10 L6 6 H14 M27 5 L30 2 V19 L27 22', PURPLE, width=1.6)
        + path('M9 18 H19 M14 13 V23', WHITE, width=1.8)),
    'SolidDifference': ('Boolean difference', 'Remove the cutter volume from a solid', 1,
        face('M4 6 H27 V12 H16 V23 H27 V28 H4 Z', PURPLE)
        + path('M27 12 H16 V23 H27', HIGHLIGHT)
        + path('M21 17 H29', WHITE, width=1.8)),
    'SolidIntersection': ('Boolean intersection', 'Keep only the common solid volume', 2,
        path('M3 4 H20 V21 H3 Z M12 11 H29 V28 H12 Z', PURPLE, dashed=True, width=1.6)
        + face('M12 11 H20 V21 H12 Z', HIGHLIGHT, .35, 2.6)),
    'SolidBooleanTwoObjects': ('Boolean: 2 objects', 'Choose among Boolean results of two objects', 4,
        face('M3 5 H18 V20 H3 Z') + face('M13 12 H28 V27 H13 Z')
        + path('M13 12 H18 V20 H13 Z', HIGHLIGHT)
        + arrow('M22 5 H28 V10', 'M25 7 L28 10 L31 7')
        + arrow('M10 27 H4 V22', 'M1 25 L4 22 L7 25')),
    'SolidCapPlanarHoles': ('Cap planar holes', 'Close a planar opening to create a solid', 5,
        cylinder() + face('M6 9 C6 3 26 3 26 9 C26 15 6 15 6 9 Z', HIGHLIGHT, .3)
        + arrow('M16 1 V7', 'M13 4 L16 7 L19 4')),
    'SolidExtractSurface': ('Extract surface', 'Separate a component face from a polysurface', 6,
        face('M3 19 L13 24 V30 L3 25 Z')
        + face('M13 24 L26 18 V24 L13 30 Z')
        + face('M4 8 L18 2 L27 8 L13 14 Z', HIGHLIGHT, .25)
        + arrow('M16 21 V15', 'M13 18 L16 15 L19 18')),
    'SolidFilletEdgeFilletEdge': ('Fillet edge', 'Round an edge with tangent curvature', 7,
        face('M4 12 L18 5 L28 10 V24 L14 30 L4 24 Z')
        + path('M4 12 L11 16 Q15 18 15 23 V29', HIGHLIGHT)
        + path('M15 23 L28 16 M18 5 Q22 7 22 12 L11 16', PURPLE, width=1.6)),
    'SolidText': ('Solid text', 'Text-shaped curves, surfaces or polysurfaces', 8,
        face('M7 26 L15 5 H21 L29 26 H23 L21 20 H14 L12 26 Z M16 15 H20 L18 9 Z', HIGHLIGHT)
        + path('M7 26 L3 23 L11 2 H17 L21 5 M3 23 H7 M11 2 L15 5', PURPLE, width=1.6)),
    'SolidExtrudePlanarCurveStraight': ('Extrude straight', 'Extrude a closed planar curve into a solid', 9,
        face('M4 14 L20 8 L28 14 L12 20 Z')
        + face('M4 14 V26 L12 30 L28 24 V14 L12 20 V30')
        + path('M4 26 L12 30 L28 24', HIGHLIGHT)
        + arrow('M16 19 V3', 'M12 7 L16 3 L20 7')),
    'SolidGVDAllExtrude': ('Extrude all', 'Extrude curves or surfaces straight, tapered or along a path', 10,
        face('M4 22 L13 18 L20 22 L11 27 Z')
        + face('M4 22 L11 10 L20 6 L28 10 L20 22 L11 27 Z')
        + path('M11 10 L20 14 L28 10 M20 14 L11 27', HIGHLIGHT)
        + arrow('M6 17 Q3 8 10 4', 'M6 4 H10 V8')),
    'SolidPipe': ('Pipe along curve', 'Round profile swept around a curve', 11,
        face('M6 24 C7 11 19 23 23 7 L29 9 C25 30 11 16 12 26 Z')
        + ellipse(9, 25, 3, 2, HIGHLIGHT)
        + path('M9 24 C11 15 22 25 26 8', HIGHLIGHT)),
    'SolidBoxCornertoCornerHeight': ('Box', 'Solid rectangular box', 12, cube()),
    'SolidSphereCenterRadius': ('Sphere', 'Solid sphere with circular silhouette', 14,
        ellipse(16, 16, 12, 12) + ellipse(16, 16, 12, 4, PURPLE, 1.6)
        + path('M16 4 C8 8 8 24 16 28 C24 24 24 8 16 4', PURPLE, width=1.6)),
    'SolidEllipsoidFromCenter': ('Ellipsoid', 'Solid stretched ellipsoid', 13,
        ellipse(16, 16, 14, 9) + ellipse(16, 16, 14, 3, PURPLE, 1.6)
        + path('M16 7 C10 9 10 23 16 25 C22 23 22 9 16 7', PURPLE, width=1.6)),
    'SolidTorus': ('Torus', 'Closed ring with a central opening', 15,
        face('M2 16 C2 2 30 2 30 16 C30 30 2 30 2 16 Z M10 15 C10 9 22 9 22 15 C22 21 10 21 10 15 Z')
        + path('M3 13 C5 27 27 27 29 13 M10 15 C12 18 20 18 22 15', PURPLE, width=1.6)),
    'SolidCylinder': ('Cylinder', 'Closed cylindrical solid', 16, cylinder()),
    'SolidTube': ('Tube', 'Cylinder with a concentric through-hole', 17,
        cylinder() + ellipse(16, 9, 5, 2.5, HIGHLIGHT)
        + path('M11 9 V23 M21 9 V23', PURPLE, dashed=True, width=1.4)),
    'SolidPyramid': ('Pyramid', 'Solid with polygon base and one apex', 18,
        face('M16 3 L3 23 L15 29 L29 23 Z')
        + path('M3 23 L15 19 L29 23 M16 3 L15 29', PURPLE, width=1.6)),
    'SolidCone': ('Cone', 'Circular base tapering to a point', 19,
        face('M16 3 L4 25 C4 31 28 31 28 25 Z')
        + path('M4 25 C4 19 28 19 28 25', PURPLE, width=1.6)),
    'SolidTruncatedCone': ('Truncated cone', 'Cone ending in a flat circular top', 20,
        face('M10 6 L4 25 C4 31 28 31 28 25 L22 6')
        + ellipse(16, 6, 6, 2.5, HIGHLIGHT)
        + path('M4 25 C4 19 28 19 28 25', PURPLE, width=1.6)),
    'SolidBoss': ('Boss to boundary', 'Extrude a closed profile toward and join a boundary surface', 21,
        face('M3 8 L23 3 L29 9 L9 14 Z')
        + face('M10 11 V25 C10 30 22 30 22 25 V8', HIGHLIGHT, .2)
        + ellipse(16, 25, 6, 2.5, HIGHLIGHT)
        + arrow('M16 22 V9', 'M13 12 L16 9 L19 12')),
    'SolidRib': ('Rib to boundary', 'Thicken an open curve and extrude it toward a boundary', 22,
        plate() + face('M9 21 L9 6 L14 4 L25 13 L25 17 L14 10 L14 23 Z', HIGHLIGHT, .25)
        + path('M9 6 L14 10 M14 4 V10', HIGHLIGHT, width=1.6)),
    'SolidSlab': ('Slab from curve', 'Offset, extrude and cap a curve to form a slab', 24,
        face('M3 11 Q16 3 28 11 L28 19 Q17 11 3 19 Z', HIGHLIGHT, .15)
        + face('M3 19 V26 Q17 18 28 26 V19')
        + path('M3 15 Q16 7 28 15', WHITE, dashed=True, width=1.6)),
    'SolidSolidEditToolsHolesMakeHole': ('Make hole', 'Project a closed curve into an object to cut a hole', 23,
        plate() + hole(16, 15)
        + arrow('M16 2 V11', 'M13 8 L16 11 L19 8')),
    'SolidSolidEditToolsHolesArrayHole': ('Array holes', 'Copy holes in rows and columns', 25,
        face('M3 6 H29 V27 H3 Z')
        + ''.join(hole(x, y) for x, y in ((9,12),(23,12),(9,21),(23,21)))),
    'SolidSolidEditToolsHolesArrayHolePolar': ('Polar array holes', 'Copy holes around a central location', 26,
        ellipse(16,16,14,14)
        + ''.join(hole(x,y) for x,y in ((16,7),(25,16),(16,25),(7,16)))
        + points((16,16),radius=1.9)),
    'SolidSolidEditToolsHolesMoveHole': ('Move hole', 'Relocate a hole on one planar surface', 27,
        face('M3 6 H29 V27 H3 Z') + ellipse(9,20,3.2,2.4,WHITE)
        + hole(23,12) + arrow('M12 18 L19 14', 'M15 14 H19 V18')),
    'SolidSolidEditToolsFacesMoveFace': ('Move face', 'Move a face of a polysurface', 28,
        face('M3 13 L12 19 V29 L3 23 Z')
        + face('M3 13 L15 7 L23 12 L12 19 Z')
        + face('M20 19 L30 14 V24 L20 29 Z', HIGHLIGHT,.25)
        + arrow('M13 23 H18', 'M15 20 L18 23 L15 26')),
    'SolidShell': ('Shell', 'Hollow a closed solid by removing faces and retaining wall thickness', 29,
        face('M3 11 L19 4 L29 11 L13 18 Z M8 11 L19 7 L24 11 L13 15 Z', HIGHLIGHT,.15)
        + face('M3 11 V23 L13 29 L29 22 V11 L13 18 V29')
        + path('M8 11 V20 L13 23 L24 18 V11', PURPLE, width=1.6)),
    'SolidPtOn': ('Solid points on', 'Show solid editing grips', None,
        cube() + points((4,11),(19,5),(28,11),(13,17),(4,22),(13,28),(28,22),radius=1.9)),
}


def svg(key):
    label, meaning, spec, geometry = SOLID_ICONS[key]
    return (f'<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32">\n'
            f'<title>{html.escape(label)}</title>\n<desc>{html.escape(meaning)}. '
            f'OpenMatrix9 {STYLE}; authored vector geometry.</desc>\n{geometry}\n</svg>\n')


def write_asset(output_root, key, tooltip=None):
    relative = f'icons/solid-minimal/{key}.svg'
    target = Path(output_root) / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    raw = svg(key).encode('utf-8')
    target.write_bytes(raw)
    label, meaning, spec, _ = SOLID_ICONS[key]
    metadata = {'image': relative, 'tooltip': tooltip or key,
                'source': 'OpenMatrix9-authored-svg', 'mapping': 'authored-command-symbol', 'status': 'resolved'}
    record = {'symbol': key, 'label': label, 'meaning': meaning, 'style': STYLE,
              'mapping': metadata['mapping'], 'sha256': hashlib.sha256(raw).hexdigest(),
              'palette': [c for c in (PURPLE, HIGHLIGHT, WHITE) if c in raw.decode()],
              'spec_id': f'OM9-SOLID-{spec:03}' if spec else None,
              'semantic_source': 'local Solid spec' if spec else 'local Rhino foundation control-point notes; SolidPtOn editing grips'}
    return metadata, {k:v for k,v in record.items() if k not in ('reference_image','reference_type','reference_images','palette_reference','reference_palette')}


def solid_keys(resources):
    menu = configparser.ConfigParser(interpolation=None)
    menu.optionxform = str
    menu.read(Path(resources) / 'menu/MainMenu.ini', encoding='utf-8')
    groups = [s for s in menu.values() if s.get('Name') == 'Solid']
    if len(groups) != 1:
        raise ValueError('Expected one Solid group')
    group = groups[0]
    keys = [v for k,v in group.items() if re.fullmatch(r'Icon\d+', k)]
    if len(keys) != int(group['IconCount']) or set(keys) != set(SOLID_ICONS):
        raise ValueError('Solid menu and authored catalog disagree')
    return keys


def contact_sheet(keys):
    columns, cell_w, cell_h = 6, 160, 120
    height = 64 + ((len(keys)+columns-1)//columns)*cell_h
    parts = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{columns*cell_w}" height="{height}">',
             '<rect width="100%" height="100%" fill="#333333"/>',
             '<text x="16" y="28" font-family="Arial,sans-serif" font-size="20" fill="white">OpenMatrix9 — Solid / minimal SVG</text>',
             '<text x="16" y="48" font-family="Arial,sans-serif" font-size="12" fill="#cccccc">30 icons · purple solids · warm-yellow result / selected feature · white actions · 64px and 24px samples</text>']
    for i,key in enumerate(keys):
        x,y = (i%columns)*cell_w,64+(i//columns)*cell_h
        label,_,_,geometry = SOLID_ICONS[key]
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
    report['solid_icon_style'] = {'name': STYLE, 'count': count,
        'source': 'OpenMatrix9-authored-svg', 'generator': 'tools/solid_icons.py',
        'geometry': 'authored geometric primitives; no tracing or embedded bitmaps',
        'palette_reference': 'purple #A668D1 solids; warm-yellow #FFD166 result/selected feature; white actions/grips',
        'priority': 'authored Solid symbols precede legacy image sources'}
    prefix='authored Solid SVG first; '
    if not report.get('asset_policy','').startswith(prefix):
        report['asset_policy']=prefix+report.get('asset_policy','')


def install(project_root):
    root=Path(project_root)
    resources=root/'Resources'
    keys=solid_keys(resources)
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
    gallery=root/'docs/images/solid-icons-minimal.svg'
    gallery.parent.mkdir(parents=True,exist_ok=True)
    gallery.write_text(contact_sheet(keys),encoding='utf-8')
    return {'count':len(keys),'style':STYLE,'preview':str(gallery)}


if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--project-root',type=Path,default=Path(__file__).resolve().parents[1])
    args=parser.parse_args()
    print(json.dumps(install(args.project_root)))
