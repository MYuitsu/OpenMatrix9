"""Authored Tools SVGs with per-command reference colors and semantic mapping."""
import argparse
import configparser
import hashlib
import html
import json
from pathlib import Path
import re
from curve_icons import WHITE, path, arrow, points

BLUE='#4260FF'
LIGHT='#9ABFFF'
PINK='#FF0CA1'
MAGENTA='#FF05FF'
PROFILE='#FFDE2A'
GOLD='#FFD705'
STYLE='minimal-tools-v1-reviewed'

def face(d,color=BLUE,fill=LIGHT,opacity=.22,width=2.4):
    return path(d,color,width=width).replace('fill="none"',f'fill="{fill}" fill-opacity="{opacity}"')

def ellipse(x,y,rx,ry,color=BLUE,width=2.4):
    return f'<ellipse cx="{x}" cy="{y}" rx="{rx}" ry="{ry}" fill="none" stroke="{color}" stroke-width="{width}"/>'

def ring():
    return ellipse(16,16,11,13,BLUE,width=3)+path('M8 7 Q2 16 8 25',LIGHT,width=1.8)

def clock(x=26,y=26):
    return ellipse(x,y,4,4,GOLD,width=1.8)+path(f'M{x} {y-2} V{y} H{x+2}',GOLD,width=1.4)

def patch():
    return face('M4 10 Q16 3 28 9 L28 26 Q16 20 4 28 Z')

def cube():
    return face('M4 9 L18 3 L28 9 V23 L14 29 L4 23 Z')+path('M4 9 L14 15 L28 9 M14 15 V29',WHITE,width=1.7)

def target(x=16,y=16,r=9):
    return path(f'M{x-r} {y} H{x+r} M{x} {y-r} V{y+r}',PINK,width=3)+points((x,y),color=PINK,radius=1.9)

def custom_rail():
    return ellipse(16,20,12,8,WHITE,width=1.5)+path('M4 20 C7 13 13 15 16 10 C18 6 21 5 23 8 L28 19 C29 28 4 31 4 20 Z',BLUE,width=2.8)

def profile():
    return face('M4 26 V19 Q16 -1 28 19 V26 Z',BLUE,BLUE,.32)+path('M4 26 V19 Q16 -1 28 19 V26 Z',PROFILE,width=2.4)

def book(letter):
    d='M10 8 H16 Q22 8 20 13 Q24 19 18 23 H10 Z M10 15 H18' if letter=='B' else 'M22 9 Q13 4 10 11 Q8 15 17 17 Q26 21 19 25 Q13 27 8 23'
    return face('M4 4 L25 2 V28 L4 30 Z')+path('M4 4 V30 M7 5 V28',LIGHT,width=1.5)+path(d,WHITE,width=2.2)

TOOLS_ICONS={
 'BuilderRingRail':('Ring rail','Place a ring rail using a finger size, region or custom dimensions',1,ring()),
 'BuilderProfilePlacer':('Profile Placer','Place closed section curves on a rail for sweep operations',2,
    path('M3 27 Q15 15 29 7',WHITE,width=1.8)
    +ellipse(8,23,4,6)+ellipse(16,16,4,6)+ellipse(25,9,4,6)),
 'BuilderOutsideRingRail':('Outside ring rail','Add an outside rail for a two-rail sweep',3,
    face('M10 3 H23 L29 10 V23 L23 29 H10 L3 23 V10 Z',BLUE,LIGHT,.05,2.8)
    +ellipse(16,16,8,10,WHITE,width=2.2)),
 'BuilderCustomRail':('Custom rail','Create a custom rail from a ring rail and planar shape curve',4,custom_rail()),
 'BuilderFourProfile':('Four rail profile','Place profiles on four rails for a four-rail sweep',5,
    path('M3 29 Q7 16 7 3 M10 29 Q12 16 13 3 M22 29 Q20 16 19 3 M29 29 Q25 16 25 3',BLUE,width=2.2)
    +ellipse(16,16,12,5,WHITE,width=2.2)),
 'BuilderMetalWeights':('Metal weights','Calculate design weight using assigned material or available metal types',6,
    path('M16 3 V27 M9 29 H23 M4 9 H28 M6 9 L3 20 H11 L6 9 M26 9 L21 20 H29 L26 9',BLUE,width=2.4)
    +path('M4 20 Q7 25 10 20 M22 20 Q25 25 28 20',LIGHT,width=2.2)
    +points((16,9),color=WHITE,radius=1.9)),
 'BuilderResize':('Ring Resizer','Create different sizes of the completed ring design',7,
    '<g transform="translate(4 4) scale(.75)">'+ring()+'</g>'
    +path('M2 9 V2 H9 M23 2 H30 V9 M30 23 V30 H23 M9 30 H2 V23',WHITE,width=1.6)
    +arrow('M8 16 H2','M5 13 L2 16 L5 19')+arrow('M24 16 H30','M27 13 L30 16 L27 19')),
 'BuilderSurfacePullback':('Surface Pullback','Create recessed areas from a closed curve on a jewelry surface',8,
    face('M4 7 Q16 0 28 7 L27 27 Q16 22 5 29 Z')
    +face('M9 10 Q16 6 23 10 L22 19 L16 24 L10 19 Z',WHITE,LIGHT,.1,2.2)
    +path('M12 13 Q16 10 20 13 L19 18 L16 20 L13 18 Z',LIGHT,width=1.6)),
 'BuilderBermarkLibrary':('Bermark library','Browse prebuilt jewelry designs',9,book('B')),
 'BuilderStullerFindingsLibrary':('Stuller findings','Browse findings and load a selected item into the viewport',10,book('S')),
 'gvSmartTarget':('Smart Target','Create a blend point and a directional target handle',11,target()),
 'gvSmartTargetAdd':('Add Smart Target','Add another target direction to an existing blend point',13,
    target(17,13,8)+path('M17 13 L7 25',PINK,width=2.2)
    +face('M2 21 H11 V30 H2 Z',PINK,PINK,.1,1.4)+path('M4 25.5 H9 M6.5 23 V28',WHITE,width=1.5)),
 'gvSmartTargetOnCrvEnd':('Target at curve end','Add a Smart Target at the end of an existing curve',14,
    path('M3 26 C3 12 13 7 21 15',WHITE,width=2.2)+target(21,15,8)),
 'gvSmartBlend':('Smart Blend','Blend a curve between two blend points',12,
    path('M6 23 C6 0 26 0 26 23',WHITE,width=2.6)+target(6,23,3)+target(26,23,3)),
 'gvSmartMSR':('Smart MSR','Select and manipulate Smart Targets without breaking history',15,
    target(14,14,7)+path('M3 10 V3 H10 M21 3 H28 V10 M28 20 V28 H21 M10 28 H3 V21',WHITE,width=1.6)
    +face('M17 16 L28 22 L23 24 L20 29 Z',PINK,PINK,.22,1.8)),
 'gvJoinHistory':('Join History','Create a child curve corresponding to joined source curves',17,
    path('M3 8 Q8 2 14 8 M19 8 Q25 14 29 7',WHITE,width=1.8)
    +path('M3 23 Q8 17 16 23 Q24 29 29 22',BLUE,width=3)
    +path('M10 8 V15 M23 9 V16',LIGHT,dashed=True,width=1.4)+clock(26,6)),
 'BuilderImageTrace':('Image Trace','Create modeling curves from an input image',16,
    face('M3 3 H29 V29 H3 Z',MAGENTA,LIGHT,.06,1.5)
    +path('M5 24 L10 14 L15 20 L22 8 L27 23 Z',WHITE,width=3.2)
    +path('M5 24 L10 14 L15 20 L22 8 L27 23 Z',MAGENTA,width=1.8)
    +points((10,14),(22,8),color=MAGENTA,radius=1.9)),
 'BuilderObjectChecker':('Object Checker','Report whether an object is valid',18,
    '<g transform="translate(0 0) scale(.75)">'+cube()+'</g>'
    +path('M13 23 L19 29 L29 14',BLUE,width=4)+path('M14 23 L19 28 L29 14',LIGHT,width=1.6)),
 'BuilderMeshRepair':('Mesh Repair','Create a single watertight mesh for 3D printing',20,
    face('M3 12 L15 4 L27 12 L25 29 H4 Z',BLUE,LIGHT,.1,1.8)
    +path('M3 12 L25 29 M27 12 L4 29 M15 4 L15 20',BLUE,width=1.5)
    +face('M18 3 V8 L23 10 L26 6 L24 2 Q30 3 29 9 Q28 16 22 15 L10 28 Q5 29 4 25 Q4 22 7 21 L18 10 Q14 7 18 3 Z',WHITE,WHITE,.3,1.8)),
 'Builder3dPrinting':('3D printing','Open tools for adding model supports before 3D printing',19,
    ellipse(16,10,8,7,WHITE)+path('M10 15 L8 26 M16 17 V26 M22 15 L24 26 M3 29 H29',BLUE,width=2.6)
    +path('M8 26 L16 20 L24 26',LIGHT,width=1.8)),
 'BuilderProfile':('Profile','Make a closed planar curve an editable profile with control handles',32,
    profile()+points((4,26),(16,9),(28,26),color=WHITE,radius=1.9)),
 'BuilderProfileMerge':('Profile Merge','Create a new profile from overlapping profiles while retaining originals',34,
    face('M3 27 V19 Q10 2 16 14 Q22 2 29 19 V27 Z',BLUE,BLUE,.3)
    +path('M3 27 V19 Q10 2 16 14 Q22 2 29 19 V27 Z',PROFILE,width=2.4)
    +path('M16 14 Q19 20 19 27 M16 14 Q13 20 13 27',WHITE,width=1.4)),
 'BuilderProfileEndCap':('Profile End Cap','Create a blended cap from a profile',33,
    face('M4 12 H28 V28 H4 Z',BLUE,LIGHT,.2)
    +face('M4 12 Q16 -4 28 12 Z',BLUE,LIGHT,.4)
    +path('M4 12 H28',WHITE,width=2.2)+path('M16 4 V12',LIGHT,width=1.6)),
 'BuilderCustomRailAdvanced':('Custom rail advanced','Create a custom rail with editable planar-curve history',35,
    custom_rail()+points((16,10),(23,8),color=WHITE,radius=1.9)+clock()),
 'BuilderObjectOnCrv':('Object on curve','Place objects along a curve with spacing, taper and orientation controls',36,
    path('M3 27 Q17 26 29 5',WHITE,width=2.2)
    +ellipse(8,25,4,4)+ellipse(18,18,4,4)+ellipse(26,9,4,4)),
 'BuilderSurfaceInset':('Surface Inset','Create an inset surface from a surface and curve',38,
    cube()+face('M17 17 L25 13 V21 L17 25 Z',WHITE,LIGHT,.12,2)
    +path('M19 18 L23 16 V20 L19 22 Z',BLUE,width=1.4)),
 'BuilderMeshReducer':('Mesh Reducer','Reduce mesh polygon count',37,
    face('M3 4 H15 V17 H3 Z',BLUE,LIGHT,.1,1.6)
    +path('M3 4 L15 17 M15 4 L3 17 M9 4 V17 M3 10 H15',WHITE,width=1.4)
    +face('M19 18 H29 V29 H19 Z',BLUE,LIGHT,.18,1.8)+path('M19 18 L29 29',WHITE,width=1.6)
    +arrow('M20 5 Q28 5 26 14','M23 11 L26 14 L29 11')),
 'BuilderMesh':('Mesh Mapper','Map a mesh onto a surface or between curves',39,
    face('M3 3 H14 V13 H3 Z',WHITE,LIGHT,.1,1.5)+path('M3 3 L14 13 M14 3 L3 13',WHITE,width=1.4)
    +face('M11 23 Q20 15 29 20 L29 29 Q20 24 11 30 Z',BLUE,LIGHT,.15,1.7)
    +path('M11 23 L29 29 M29 20 L11 30 M20 19 V27',WHITE,width=1.4)
    +arrow('M20 5 Q29 5 27 15','M24 12 L27 15 L30 12')),
 'BuilderCurveTransformTool':('Curve Transform','Bend a curve like wire with hold and bend controls',40,
    path('M3 26 H28',WHITE,dashed=True,width=1.6)
    +path('M3 26 C19 26 14 8 28 6',BLUE,width=3)
    +points((3,26),color=WHITE,radius=1.9)+points((28,6),color=LIGHT,radius=1.9)
    +arrow('M27 22 V13','M24 16 L27 13 L30 16')),
 'BuilderCenterLine':('Center line','Calculate a center line from a surface and two curves on it',42,
    face('M4 27 Q9 7 19 4 L29 10 Q20 12 16 29 Z',BLUE,LIGHT,.07,1.4)
    +path('M4 27 Q9 7 19 4 M16 29 Q20 12 29 10',WHITE,width=2)
    +path('M10 28 Q15 10 24 7',BLUE,width=3)),
 'BuilderBooleanBuilder':('Boolean Builder','Use objects and cutters for cut, join or unjoin operations',41,
    face('M3 5 H19 V22 H3 Z',BLUE,LIGHT,.2)+ellipse(22,21,8,8,BLUE,width=2.6)
    +path('M19 13 V22 H14',WHITE,width=2)
    +path('M6 27 H12 M9 24 V30 M19 27 H25',WHITE,width=1.8)),
}

NOTES={
 'BuilderRingRail':'Single blue rail with light highlight; distinct from a solid ring builder.',
 'BuilderProfilePlacer':'Several blue section curves placed along one white rail.',
 'BuilderOutsideRingRail':'Blue outside rail around a separate white ring reference.',
 'BuilderCustomRail':'Deformed custom rail with white ring reference; no invented generic transform arrows.',
 'BuilderFourProfile':'Four separate blue rails and one common white profile.',
 'BuilderMetalWeights':'Blue scale balance; assigned materials determine calculated weight, not a fixed metal.',
 'BuilderResize':'Blue ring reference with white dimension brackets and outward resize arrows.',
 'BuilderSurfacePullback':'Closed shield-shaped inset on a bowed jewelry surface; not curve projection.',
 'BuilderBermarkLibrary':'Authored blue book with geometric B identifying the design library; no copied logo.',
 'BuilderStullerFindingsLibrary':'Authored blue book with geometric S identifying the findings library; no copied logo.',
 'gvSmartTarget':'Retain the pink cross; center represents blend point and arms direction handles.',
 'gvSmartTargetAdd':'Additional pink direction and a separate white plus badge.',
 'gvSmartTargetOnCrvEnd':'White curve ends at the pink target; curve does not pass through the target.',
 'gvSmartBlend':'White blended curve between two pink target stations.',
 'gvSmartMSR':'Pink target, white selection brackets and cursor preserve selection-filter meaning.',
 'gvJoinHistory':'Separate white source segments and a single blue child curve, with gold History marker.',
 'BuilderImageTrace':'Framed input-image outline and magenta vector points/curve; retain its unique magenta accent.',
 'BuilderObjectChecker':'Blue check next to a checked object; validation action rather than a guarantee of valid input.',
 'BuilderMeshRepair':'White repair tool across a blue triangular mesh.',
 'Builder3dPrinting':'White model above blue support branches and bed; depicts support preparation.',
 'BuilderProfile':'Yellow-bordered blue section with white control handles.',
 'BuilderProfileMerge':'Yellow result boundary around two overlapping section shapes; internal white source edges remain.',
 'BuilderProfileEndCap':'Blue blended dome above the white profile boundary.',
 'BuilderCustomRailAdvanced':'Same rail family plus white editable points and gold History cue.',
 'BuilderObjectOnCrv':'Repeated blue objects follow a white curved path; not just a single-object transfer arrow.',
 'BuilderSurfaceInset':'White inner boundary and inset region on a blue surface-bearing volume.',
 'BuilderMeshReducer':'Dense mesh source and sparse mesh result, kept separate by transfer direction.',
 'BuilderMesh':'Different source/result contexts show mapping a mesh to a bowed surface; not generic mesh creation.',
 'BuilderCurveTransformTool':'Held endpoint and bent result compared with a white straight reference.',
 'BuilderCenterLine':'Two separated white curves on a faint surface, blue line between them.',
 'BuilderBooleanBuilder':'Overlapping object/cutter contexts and plus/minus operation cues represent the selectable Boolean builder modes.',
}

def svg(key):
    label, meaning, spec, geometry = TOOLS_ICONS[key]
    return (f'<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32">\n'
            f'<title>{html.escape(label)}</title>\n<desc>{html.escape(meaning)}. '
            f'OpenMatrix9 {STYLE}; authored vector geometry.</desc>\n{geometry}\n</svg>\n')


def write_asset(output_root, key, tooltip=None):
    relative = f'icons/tools-minimal/{key}.svg'
    target = Path(output_root) / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    raw = svg(key).encode('utf-8')
    target.write_bytes(raw)
    label, meaning, spec, _ = TOOLS_ICONS[key]
    metadata = {'image': relative, 'tooltip': tooltip or key,
                'source': 'OpenMatrix9-authored-svg', 'mapping': 'authored-command-symbol', 'status': 'resolved'}
    record = {'symbol': key, 'label': label, 'meaning': meaning, 'style': STYLE,
              'mapping': metadata['mapping'], 'sha256': hashlib.sha256(raw).hexdigest(),
              'palette': [c for c in (BLUE, LIGHT, PINK, MAGENTA, PROFILE, GOLD, WHITE) if c in raw.decode()],
              'spec_id': f'OM9-TOOLS-{spec:03}',
              'semantic_source': 'local spec matched by command meaning',
              'reference_image': f'icons/rgb-plus5/ButtonIcons/{key}_1.png',
              'reference_type': 'selected Matrix90 RGB+5 derivative',
              'design_note': NOTES[key]}

    return metadata, {k:v for k,v in record.items() if k not in ('reference_image','reference_type','reference_images','palette_reference','reference_palette')}


def tools_keys(resources):
    menu = configparser.ConfigParser(interpolation=None)
    menu.optionxform = str
    menu.read(Path(resources) / 'menu/MainMenu.ini', encoding='utf-8')
    groups = [s for s in menu.values() if s.get('Name') == 'Tools']
    if len(groups) != 1:
        raise ValueError('Expected one Tools group')
    group = groups[0]
    keys = [v for k,v in group.items() if re.fullmatch(r'Icon\d+', k)]
    if len(keys) != int(group['IconCount']) or set(keys) != set(TOOLS_ICONS):
        raise ValueError('Tools menu and authored catalog disagree')
    return keys


def contact_sheet(keys):
    columns, cell_w, cell_h = 8, 160, 132
    height = 64 + ((len(keys)+columns-1)//columns)*cell_h
    parts = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{columns*cell_w}" height="{height}">',
             '<rect width="100%" height="100%" fill="#333333"/>',
             '<text x="16" y="28" font-family="Arial,sans-serif" font-size="20" fill="white">OpenMatrix9 — Tools / minimal SVG</text>',
             '<text x="16" y="48" font-family="Arial,sans-serif" font-size="12" fill="#cccccc">31 icons · blue tools · pink Smart Targets · magenta Trace · yellow Profile · gold History · 64px and 24px samples</text>']
    for i,key in enumerate(keys):
        x,y = (i%columns)*cell_w,64+(i//columns)*cell_h
        label,_,_,geometry = TOOLS_ICONS[key]
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
    report['tools_icon_style'] = {'name': STYLE, 'count': count,
        'source': 'OpenMatrix9-authored-svg', 'generator': 'tools/tools_icons.py',
        'geometry': 'authored geometric primitives; no tracing or embedded bitmaps',
        'palette_reference': 'selected RGB+5: blue #4260FF, light #9ABFFF, Smart Target pink #FF0CA1, Image Trace magenta #FF05FF, Profile yellow #FFDE2A, History gold #FFD705, white #FFFFFF',
        'priority': 'authored Tools symbols precede legacy image sources'}
    prefix='authored Tools SVG first; '
    if not report.get('asset_policy','').startswith(prefix):
        report['asset_policy']=prefix+report.get('asset_policy','')


def install(project_root):
    root=Path(project_root)
    resources=root/'Resources'
    keys=tools_keys(resources)
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
    gallery=root/'docs/images/tools-icons-minimal.svg'
    gallery.parent.mkdir(parents=True,exist_ok=True)
    gallery.write_text(contact_sheet(keys),encoding='utf-8')
    return {'count':len(keys),'style':STYLE,'preview':str(gallery)}


if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--project-root',type=Path,default=Path(__file__).resolve().parents[1])
    args=parser.parse_args()
    print(json.dumps(install(args.project_root)))
