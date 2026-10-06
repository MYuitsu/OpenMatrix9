"""Authored Surface SVGs from per-command meaning and sampled green references."""
import argparse
import configparser
import hashlib
import html
import json
from pathlib import Path
import re
from curve_icons import WHITE, path, arrow, points

GREEN='#5FCB62'
DARK='#0A6C0A'
GOLD='#FFD705'
STYLE='minimal-surface-v1-reviewed'

def face(d,color=GREEN,opacity=.3,width=2.2):
    return path(d,color,width=width).replace('fill="none"',f'fill="{color}" fill-opacity="{opacity}"')

def ellipse(x,y,rx,ry,color=WHITE):
    return f'<ellipse cx="{x}" cy="{y}" rx="{rx}" ry="{ry}" fill="none" stroke="{color}" stroke-width="2.2"/>'

def patch(grid=False):
    return face('M4 11 Q15 3 27 9 L28 26 Q15 21 5 28 Z')+(path('M5 19 Q16 11 27 17 M16 7 L17 25',DARK,width=1.4) if grid else '')

def clock(x=26,y=26):
    return ellipse(x,y,4,4,GOLD)+path(f'M{x} {y-2} V{y} H{x+2}',GOLD,width=1.4)

def number(n):
    d={1:'M25 5 L28 3 V12 M25 12 H30',2:'M24 5 Q25 1 29 3 Q32 5 27 8 L24 12 H30',4:'M28 3 L23 9 H30 M28 3 V13'}[n]
    return path(d,WHITE,width=1.8)

def sweep(rails=1):
    art=face('M4 27 C4 9 14 2 27 6 L29 19 C17 14 12 20 12 29 Z')
    art+=path('M5 20 Q14 10 28 13 M11 9 Q17 5 28 10',DARK,width=1.4)
    art+=path('M4 27 C4 9 14 2 27 6',WHITE,width=2.2)
    if rails==2:
        art+=path('M12 29 C12 20 17 14 29 19',WHITE,width=2.2)
    return '<g transform="translate(0 3) scale(.8)">'+art+'</g>'+number(rails)

def corner(kind='fillet',variable=False):
    # Perpendicular input faces; result is a rounded band or a flat bevel.
    art=face('M3 3 H29 V9 H14 L9 14 V29 H3 Z',GREEN,.13,1.8)
    if kind=='chamfer':
        band='M9 18 L18 9 H29 L9 25 Z' if variable else 'M9 18 L18 9 H27 L9 27 Z'
        art+=face(band,GREEN,.45)
        art+=path('M9 18 L18 9',WHITE,width=1.8)
    elif kind=='blend':
        art+=face('M9 27 C10 23 6 14 14 10 C20 6 24 10 29 9 L29 19 C20 19 15 13 15 29 Z',GREEN,.45)
        art+=path('M9 27 C10 23 6 14 14 10 C20 6 24 10 29 9',WHITE,width=1.8)
    else:
        band='M9 27 Q9 9 27 9 V19 Q17 17 15 27 Z' if variable else 'M9 27 Q9 9 27 9 V15 Q15 15 15 27 Z'
        art+=face(band,GREEN,.45)
        art+=path('M9 27 Q9 9 27 9',WHITE,width=1.8)
    if variable:
        art+=points((12,22),radius=1.9)+points((23,11),radius=2.6)
    return art

def offset(variable=False):
    lower='M3 24 Q15 17 27 22 L29 29 Q15 25 5 30 Z'
    upper='M3 8 Q15 1 27 6 L29 13 Q15 9 5 14 Z' if not variable else 'M3 15 Q16 2 27 4 L29 11 Q16 8 5 21 Z'
    return face(lower,WHITE,.06,1.6)+face(upper)+arrow('M16 23 V13','M13 16 L16 13 L19 16')

SURFACE_ICONS={
 'SurfaceSweepSweep1Rail':('Sweep 1 rail','Profiles swept along a single rail',1,sweep(1)),
 'SurfaceSweepSweep2Rails':('Sweep 2 rails','Profiles swept between two rails',3,sweep(2)),
 'SurfaceSweepProfileSweep':('Profile Sweep','History-enabled surface between two closed profiles; no rail required',5,
    face('M3 9 C3 15 11 24 14 26 L28 23 C27 17 18 7 17 7 Z')
    +ellipse(10,8,7,4)+ellipse(21,24,7,4)+clock(26,6)),
 'SurfacePlanarCurves':('Planar curves','Planar surface bounded by coplanar curves, with an interior opening',13,
    face('M4 5 H26 V15 H29 V28 H4 Z M10 17 A4 4 0 1 0 18 17 A4 4 0 1 0 10 17 Z')
    +path('M4 5 H26 V15 H29 V28 H4 Z',WHITE,width=1.7)),
 'SurfacePlaneCornertoCorner':('Plane from corners','Rectangular plane between opposite corners',15,
    face('M4 5 H28 V28 H4 Z')+path('M4 16 H28 M16 5 V28',DARK,width=1.6)
    +points((4,5),(28,28),radius=1.9)),
 'OthersSurfaceRebuild':('Rebuild surface','Change surface control point counts in U and V',6,
    patch(True)+points((5,11),(16,7),(27,9),(6,27),(17,25),(28,26),radius=1.9)
    +arrow('M5 5 Q15 0 24 4','M20 2 L24 4 L20 6')),
 'SurfaceBlendSurface':('Blend surface','Smooth connecting surface between separate surface edges',7,
    face('M3 3 H11 V12 H3 Z',WHITE,.08,1.6)+face('M22 21 H29 V29 H22 Z',WHITE,.08,1.6)
    +face('M3 12 C4 25 23 9 22 21 L29 21 C28 3 11 23 11 12 Z')),
 'SurfaceVariableFilletBlendChamferVariableBlendSurfaces':('Variable blend','Blend between surfaces with varying radius values',8,corner('blend',True)),
 'SurfaceLoft':('Loft','Surface stretched between a sequence of section curves',9,
    face('M4 27 Q9 7 15 4 L27 11 Q23 14 27 27 Z')
    +path('M4 27 Q15 16 27 27 M7 18 Q17 7 25 18 M15 4 Q23 3 27 11',WHITE,width=2.2)),
 'SurfaceCurveNetwork':('Curve network','Surface from crossing curves in both directions',10,
    patch()+path('M5 17 Q16 9 28 15 M5 23 Q16 15 28 21 M11 8 L12 26 M22 7 L23 25',WHITE,width=1.8)),
 'SurfaceSurfaceEditToolsShrinkTrimmedSurface':('Shrink trimmed surface','Contract underlying surface domain toward trim boundaries while retaining trims',12,
    path('M3 9 V3 H9 M23 3 H29 V9 M29 23 V29 H23 M9 29 H3 V23',WHITE,width=1.6)
    +face('M9 9 H23 V23 H9 Z')+path('M12 16 H20 M16 12 V20',DARK,width=1.4)
    +arrow('M2 16 H7','M4 13 L7 16 L4 19')+arrow('M30 16 H25','M28 13 L25 16 L28 19')),
 'SurfacePatch':('Patch','Fit a surface through boundary curves and point objects',11,
    patch(True)+points((5,11),(27,9),(6,27),(28,26),(13,14),(21,19),radius=1.9)),
 'SurfaceEdgeCurves':('Edge curves','Surface bounded by two to four edge curves',14,
    patch()+path('M4 11 Q15 3 27 9 L28 26 Q15 21 5 28 Z',WHITE,width=2.2)
    +path('M16 7 L17 25',DARK,width=1.4)),
 'SurfaceSurfaceEditToolsUntrim':('Untrim','Remove a trimming boundary and restore the underlying surface',17,
    face('M3 3 H29 V29 H3 Z')+path('M10 10 H22 V22 H10 Z',WHITE,dashed=True,width=1.4)
    +arrow('M16 16 V5','M13 8 L16 5 L19 8')),
 'SurfaceGVDExtrudeAll':('Extrude all','Extrude curves or surfaces straight, tapered or along a path',16,
    face('M4 27 L10 8 L22 5 L28 23 L16 29 Z')
    +path('M4 27 L16 29 L28 23 M10 8 L16 12 L22 5 M16 12 V29',WHITE,width=1.7)
    +arrow('M4 20 V4','M1.5 7 L4 4 L6.5 7')),
 'SurfaceRevolve':('Revolve','Revolve a profile around a straight axis',18,
    path('M16 2 V29',WHITE,dashed=True,width=1.4)
    +face('M9 5 Q15 13 7 24 Q16 31 25 24 Q17 13 23 5 Z')
    +path('M23 5 Q17 13 25 24',WHITE,width=1.8)
    +arrow('M4 16 Q16 22 28 16','M25 14 L28 16 L25 19')),
 'SurfaceRailRevolve':('Rail revolve','Revolve a profile using a perpendicular path curve',19,
    face('M5 26 Q15 15 16 5 Q20 18 28 26 Z')
    +ellipse(16,26,12,3)+path('M16 5 Q20 18 28 26',WHITE,width=1.8)),
 'BuilderFourProfileSweep':('Sweep 4 rails','History-enabled sweep on four rails with attached profiles',21,
    '<g transform="translate(0 3) scale(.8)">'+patch()
    +path('M4 11 L5 28 M12 7 L13 25 M20 7 L21 24 M27 9 L28 26',WHITE,width=1.7)
    +path('M5 18 Q16 10 28 16',DARK,width=1.4)+'</g>'+number(4)),
 'SurfaceSweepSweepMultiRail':('Sweep Multi','History-enabled sweep with editable profile points',32,
    patch()+path('M4 11 L5 28',WHITE,width=1.8)
    +points((9,10),(17,8),(24,11),radius=1.9)+clock(26,26)),
 'SurfaceTweenSurfaces':('Tween surfaces','Intermediate surface between two input surfaces',27,
    face('M3 4 Q16 0 28 5 L29 9 Q16 5 4 8 Z',WHITE,.04,1.5)
    +face('M3 26 Q16 22 28 27 L29 31 Q16 27 4 30 Z',WHITE,.04,1.5)
    +face('M3 15 Q16 11 28 16 L29 20 Q16 16 4 19 Z')),
 'SurfaceFilletSurfaces':('Fillet surfaces','Rounded tangent transition between two surfaces',20,corner()),
 'SurfaceVariableFilletBlendChamferVariableFilletSurfaces':('Variable fillet','Rounded surface transition with varying radii',22,corner('fillet',True)),
 'SurfaceChamferSurfaces':('Chamfer surfaces','Flat bevel surface between two surfaces',23,corner('chamfer')),
 'SurfaceVariableFilletBlendChamferVariableChamferSurfaces':('Variable chamfer','Bevel between surfaces with varying distances',24,corner('chamfer',True)),
 'SurfaceOffsetSurface':('Offset surface','Surface displaced by the same normal distance',25,offset()),
 'SurfaceVariableOffsetSurface':('Variable offset','Deform the surface while offsetting',26,offset(True)+points((3,15),(27,4),radius=1.9)),
 'SurfaceSurfaceEditToolsMerge':('Merge surfaces','Combine untrimmed surfaces into one new surface',29,
    patch()+path('M15 8 L16 25',WHITE,dashed=True,width=1.4)
    +arrow('M3 18 H12','M9 15 L12 18 L9 21')+arrow('M30 18 H21','M24 15 L21 18 L24 21')),
 'SurfaceSurfaceEditToolsMatch':('Match surfaces','Adjust a surface edge to match continuity of a neighbor',28,
    face('M3 8 L13 5 V28 L3 26 Z',WHITE,.06,1.6)
    +face('M18 5 Q28 9 29 23 L18 28 Z')
    +path('M13 5 V28 M18 5 V28',GREEN,width=2.2)
    +arrow('M27 16 H15','M18 13 L15 16 L18 19')),
 'SurfaceDrape':('Drape','Fit a NURBS surface over underlying objects',30,
    face('M8 25 L15 12 L25 25 Z',WHITE,.05,1.6)
    +face('M3 26 C5 22 7 7 16 7 S26 23 29 26 L25 30 C20 23 12 23 7 30 Z')
    +path('M16 7 V25 M5 20 Q16 14 27 20',DARK,width=1.4)),
 'SurfaceHeightfieldfromImage':('Heightfield from image','Create height geometry from image grayscale values',31,
    face('M2 3 H14 V12 H2 Z',WHITE,.08,1.6)+path('M3 10 L6 7 L9 10 L12 6',WHITE,width=1.4)
    +face('M5 24 L15 13 L21 21 L25 16 L29 24 L20 29 L5 29 Z')
    +path('M15 13 L14 29 M25 16 L23 28 M5 24 L20 29 L29 24 M11 19 L17 23 L24 20',DARK,width=1.4)),
 'SurfaceSurfaceEditToolsSplitatIsocurve':('Split at isocurve','Split a surface using an isoparametric curve',34,
    face('M3 5 H14 V28 H3 Z')+face('M19 5 H29 V28 H19 Z')
    +path('M14 5 V28 M19 5 V28',WHITE,width=2.2)+path('M3 16 H14 M19 16 H29',DARK,width=1.4)),
 'SurfaceExtendSurface':('Extend surface','Lengthen a surface by moving its edge',33,
    face('M3 8 H17 V28 H3 Z')+face('M17 8 H29 V28 H17 Z',GREEN,.14)
    +path('M17 8 V28',WHITE,dashed=True,width=1.4)+arrow('M19 4 H29','M26 1.5 L29 4 L26 7')),
 'SurfaceExtrudeCurveNormalToSurface':('Extrude normal','Extrude a curve along a surface normal or tangent',35,
    face('M3 24 L22 18 L29 25 L10 30 Z',WHITE,.06,1.6)
    +face('M10 25 V8 Q16 3 22 8 V22 Q16 17 10 25 Z')
    +path('M10 25 Q16 17 22 22',WHITE,width=1.8)
    +arrow('M16 17 V3','M13 6 L16 3 L19 6')),
 'SurfaceUnrollDevelopableSrf':('Unroll developable','Unroll a singly curved developable surface into a flat pattern',36,
    face('M3 5 Q8 0 13 5 V18 Q8 13 3 18 Z',WHITE,.06,1.6)
    +path('M8 2 V15',WHITE,width=1.4)+face('M18 18 H29 V29 H18 Z')
    +arrow('M16 7 Q26 7 24 15','M21 12 L24 15 L27 12')),
 'SurfaceSmash':('Smash','Approximate flattening of a surface curved in two directions',None,
    face('M3 11 Q8 0 13 6 L16 14 Q9 9 4 18 Z',WHITE,.06,1.6)
    +path('M4 12 Q9 4 14 10 M8 7 L10 15',WHITE,width=1.4)
    +face('M18 20 L28 17 L30 28 L20 30 Z')
    +arrow('M21 6 V15','M18 12 L21 15 L24 12')),
 'SurfaceSurfaceEditToolsSoftEdit':('Soft edit surface','Move a surface region with smooth distance-based falloff',37,
    face('M3 27 C7 27 9 8 16 8 S25 27 29 27 L24 30 Q16 22 8 30 Z')
    +path('M16 8 V26 M7 20 Q16 13 25 20',DARK,width=1.4)
    +points((16,8),radius=1.9)+arrow('M16 21 V11','M13 14 L16 11 L19 14')),
 'SurfacePlaneCuttingPlane':('Cutting plane','Create planar surfaces through objects at selected locations',39,
    path('M4 8 L18 3 L28 9 V24 L14 29 L4 23 Z M4 8 L14 14 L28 9 M14 14 V29',WHITE,width=1.6)
    +face('M3 19 L22 10 L30 16 L11 26 Z')),
 'SurfaceSurfaceEditToolsAdjustClosedSurfaceSeam':('Adjust surface seam','Move the seam on a closed unjoined surface',38,
    face('M5 8 V25 C5 30 27 30 27 25 V8')+ellipse(16,8,11,4,GREEN)
    +path('M16 11 V28',WHITE,dashed=True,width=1.5)
    +arrow('M7 17 Q16 22 25 17','M22 15 L25 17 L22 20')),
 'SurfaceSurfaceEditToolsSetSurfaceTangent':('Set surface tangent','Set the tangent direction of an untrimmed edge',40,
    patch()+path('M4 11 L5 28',WHITE,width=2.2)
    +points((5,19),radius=1.9)+arrow('M5 19 L22 7','M17 7 H22 V12')),
 'SurfaceSurfaceEditToolsRefittoTolerance':('Refit to tolerance','Reduce control points while fitting the surface within tolerance',None,
    path('M4 3 V10 M4 3 H10 M4 6 H9 M16 3 V10 M22 3 H29 M25.5 3 V10',WHITE,width=1.8)
    +face('M3 21 Q15 12 27 18 L29 28 Q15 22 5 30 Z')
    +path('M3 15 Q15 6 27 12',WHITE,dashed=True,width=1.4)),
}


def svg(key):
    label, meaning, spec, geometry = SURFACE_ICONS[key]
    return (f'<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32">\n'
            f'<title>{html.escape(label)}</title>\n<desc>{html.escape(meaning)}. '
            f'OpenMatrix9 {STYLE}; authored vector geometry.</desc>\n{geometry}\n</svg>\n')


def write_asset(output_root, key, tooltip=None):
    relative = f'icons/surface-minimal/{key}.svg'
    target = Path(output_root) / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    raw = svg(key).encode('utf-8')
    target.write_bytes(raw)
    label, meaning, spec, _ = SURFACE_ICONS[key]
    metadata = {'image': relative, 'tooltip': tooltip or key,
                'source': 'OpenMatrix9-authored-svg', 'mapping': 'authored-command-symbol', 'status': 'resolved'}
    record = {'symbol': key, 'label': label, 'meaning': meaning, 'style': STYLE,
              'mapping': metadata['mapping'], 'sha256': hashlib.sha256(raw).hexdigest(),
              'palette': [c for c in (GREEN, DARK, WHITE, GOLD) if c in raw.decode()],
              'spec_id': f'OM9-SURFACE-{spec:03}' if spec else None,
              'semantic_source': 'local Surface spec; Shrink clarified by official Rhino docs' if spec==12 else ('local Surface spec' if spec else 'official Rhino command documentation'),
              'source_url': ('https://docs.mcneel.com/rhino/8/help/en-us/commands/shrinktrimmedsrf.htm' if spec==12 else
                             'https://docs.mcneel.com/rhino/9/help/en-us/commands/smash.htm' if key=='SurfaceSmash' else
                             'https://docs.mcneel.com/rhino/8/help/en-us/commands/fitsrf.htm' if key=='SurfaceSurfaceEditToolsRefittoTolerance' else None)}
    return metadata, {k:v for k,v in record.items() if k not in ('reference_image','reference_type','reference_images','palette_reference','reference_palette')}


def surface_keys(resources):
    menu = configparser.ConfigParser(interpolation=None)
    menu.optionxform = str
    menu.read(Path(resources) / 'menu/MainMenu.ini', encoding='utf-8')
    groups = [s for s in menu.values() if s.get('Name') == 'Surface']
    if len(groups) != 1:
        raise ValueError('Expected one Surface group')
    group = groups[0]
    keys = [v for k,v in group.items() if re.fullmatch(r'Icon\d+', k)]
    if len(keys) != int(group['IconCount']) or set(keys) != set(SURFACE_ICONS):
        raise ValueError('Surface menu and authored catalog disagree')
    return keys


def contact_sheet(keys):
    columns, cell_w, cell_h = 8, 160, 120
    height = 64 + ((len(keys)+columns-1)//columns)*cell_h
    parts = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{columns*cell_w}" height="{height}">',
             '<rect width="100%" height="100%" fill="#333333"/>',
             '<text x="16" y="28" font-family="Arial,sans-serif" font-size="20" fill="white">OpenMatrix9 — Surface / minimal SVG</text>',
             '<text x="16" y="48" font-family="Arial,sans-serif" font-size="12" fill="#cccccc">40 icons · green surfaces · white construction / actions · gold History · 64px and 24px samples</text>']
    for i,key in enumerate(keys):
        x,y = (i%columns)*cell_w,64+(i//columns)*cell_h
        label,_,_,geometry = SURFACE_ICONS[key]
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
    report['surface_icon_style'] = {'name': STYLE, 'count': count,
        'source': 'OpenMatrix9-authored-svg', 'generator': 'tools/surface_icons.py',
        'geometry': 'authored geometric primitives; no tracing or embedded bitmaps',
        'palette_reference': 'sampled green #5FCB62 surfaces; dark green #0A6C0A structure; white construction; gold #FFD705 History',
        'priority': 'authored Surface symbols precede legacy image sources'}
    prefix='authored Surface SVG first; '
    if not report.get('asset_policy','').startswith(prefix):
        report['asset_policy']=prefix+report.get('asset_policy','')


def install(project_root):
    root=Path(project_root)
    resources=root/'Resources'
    keys=surface_keys(resources)
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
    gallery=root/'docs/images/surface-icons-minimal.svg'
    gallery.parent.mkdir(parents=True,exist_ok=True)
    gallery.write_text(contact_sheet(keys),encoding='utf-8')
    return {'count':len(keys),'style':STYLE,'preview':str(gallery)}


if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--project-root',type=Path,default=Path(__file__).resolve().parents[1])
    args=parser.parse_args()
    print(json.dumps(install(args.project_root)))
