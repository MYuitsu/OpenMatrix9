"""Authored cutter symbols with reference colors and unchanged shared command assets."""
import argparse
import configparser
import hashlib
import html
import json
from pathlib import Path
import re
from curve_icons import WHITE, path, arrow, points
from settings_icons import SETTINGS_ICONS, svg as setting_svg, write_asset as setting_write_asset
from tools_icons import TOOLS_ICONS, svg as tool_svg, write_asset as tool_write_asset

ORANGE = '#F2793A'
DARK = '#C05D05'
LIGHT = '#FFC88E'
CREAM = '#FFFFCF'
QUAD = '#FF6605'
STYLE = 'minimal-cutters-v1-reviewed'
SHARED = {'BuilderBezelCutter','BuilderBooleanBuilder'}


def face(d,color=ORANGE,fill=DARK,opacity=.35,width=2.6):
    return path(d,color,width=width).replace('fill="none"',f'fill="{fill}" fill-opacity="{opacity}"')


def ellipse(x,y,rx,ry,color=WHITE,width=2):
    return f'<ellipse cx="{x}" cy="{y}" rx="{rx}" ry="{ry}" fill="none" stroke="{color}" stroke-width="{width}"/>'


def cutter():
    return (face('M12 3 H20 V12 L26 16 L20 19 V29 H12 V19 L6 16 L12 12 Z')
        +path('M6 16 Q16 23 26 16',CREAM,width=2)
        +path('M15 5 V11 M15 22 V27',LIGHT,width=1.8))


def channel_body():
    return face('M4 3 H19 L28 11 V28 H12 L4 20 Z')+path('M4 3 L12 11 V28 M12 11 H28',LIGHT,width=1.6)


CUTTERS_ICONS = {
 'BuilderGemCutter':('Gem Cutter','Generate gem cutters for one or multiple stones','OM9-CUTTER-001',cutter()),
 'BuilderGemCutterLibrary':('Gem Cutter Library','Choose cutter styles for gem seats or azures','OM9-CUTTER-002',
     face('M4 5 L27 2 V27 L4 30 Z')+path('M4 5 V30 M7 6 V28',LIGHT,width=1.7)
     +'<g transform="translate(9 5) scale(.6)">'+cutter()+'</g>'),
 'BuilderChannelBuilder':('Channel Builder','Generate a continuous channel cutter for a line of gems','OM9-CUTTER-003',
     channel_body()+path('M12 5 Q23 15 21 27',CREAM,width=4.4)),
 'BuilderMicroProngCutter':('Micro Prong Cutter','Create cutters for a micro-prong setting along a line of gems','OM9-CUTTER-004',
     face('M4 12 L13 3 L28 10 L20 29 L5 23 Z')
     +path('M6 12 L12 15 L19 8 M7 20 L13 23 L25 14',CREAM,width=3.2)
     +path('M5 23 L13 27 L20 29',LIGHT,width=1.7)),
 'BuilderBrightCutChannel':('Bright Cut Channel','Create a bright-cut setting cutter for a line of gems','OM9-CUTTER-005',
     channel_body()+path('M11 5 L19 10 L15 14 L23 19 L20 24 L26 27',CREAM,width=3.6)),
 'BuilderBrightCutter':('Bright Cutter','Create a bright cutter for a single gem','OM9-CUTTER-006',
     face('M5 11 L16 4 L27 11 V25 L16 29 L5 24 Z')
     +face('M7 11 L16 6 L25 11 L21 18 L9 15 Z',CREAM,CREAM,.48,1.7)
     +path('M5 24 L16 29 L27 25',LIGHT,width=1.6)),
 'BuilderBezelCutter':SETTINGS_ICONS['BuilderBezelCutter'],
 'OthersCuttoFingerRail':('Cut to Finger Rail','Remove excess material where an object intersects the finger rail','OM9-CUTTER-008',
     ellipse(16,19,12,10,WHITE,width=2.1)
     +face('M5 4 H27 V17 Q16 8 5 17 Z')
     +path('M5 17 Q16 8 27 17',CREAM,width=2.1)
     +path('M13 22 H19 M16 19 V25',LIGHT,width=1.5)),
 'BuilderPlaneandCubeCutters':('Plane and Cube Cutters','Choose planes for splits or cubes for Boolean cuts around F4','OM9-CUTTER-009',
     face('M6 10 L18 4 L27 10 V23 L15 29 L6 23 Z')
     +path('M6 10 L15 16 L27 10 M15 16 V29',LIGHT,width=1.7)
     +face('M3 17 L16 10 L29 17 L16 24 Z',CREAM,CREAM,.1,1.8)),
 'BuilderQuadFlip':('Quad Flip Routines','Split and mirror a selected quadrant into all four quadrants around F4','OM9-CUTTER-011',
     path('M16 3 V29 M3 16 H29',WHITE,width=1.5)
     +face('M4 4 H11 Q11 10 4 11 Z',QUAD,LIGHT,.9,2)
     +face('M28 4 H21 Q21 10 28 11 Z',QUAD,QUAD,.2,2)
     +face('M4 28 H11 Q11 22 4 21 Z',QUAD,QUAD,.2,2)
     +face('M28 28 H21 Q21 22 28 21 Z',QUAD,QUAD,.2,2)),
 'BuilderBooleanBuilder':TOOLS_ICONS['BuilderBooleanBuilder'],
}

NOTES = {
 'BuilderGemCutter':'Stepped orange cutter stem and a broad cream seat collar retain the single cutter silhouette.',
 'BuilderGemCutterLibrary':'Orange library book with a small stepped cutter distinguishes selecting cutter styles.',
 'BuilderChannelBuilder':'One continuous cream groove on an orange cutter volume.',
 'BuilderMicroProngCutter':'Separated transverse cream cuts across a tilted orange volume; different from a continuous channel.',
 'BuilderBrightCutChannel':'Repeated angular cream cut faces on a channel-length orange volume; not a generic straight groove.',
 'BuilderBrightCutter':'One large angular bright-cut face on an orange volume, distinct from the repeated channel cutter.',
 'OthersCuttoFingerRail':'Orange material conforms to the upper arc of a white finger rail, retaining the circular rail reference cue.',
 'BuilderPlaneandCubeCutters':'A separate cream cutting plane intersects the orange cube context; no specific plane choice implied.',
 'BuilderQuadFlip':'Four mirrored quadrant lobes around a white F4 cross; pale source quadrant and orange mirrored results.',
}


def svg(key):
    if key=='BuilderBezelCutter': return setting_svg(key)
    if key=='BuilderBooleanBuilder': return tool_svg(key)
    label, meaning, spec, geometry = CUTTERS_ICONS[key]
    return (f'<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32">\n'
            f'<title>{html.escape(label)}</title>\n<desc>{html.escape(meaning)}. '
            f'OpenMatrix9 {STYLE}; authored vector geometry.</desc>\n{geometry}\n</svg>\n')


def write_asset(output_root, key, tooltip=None):
    if key=='BuilderBezelCutter': return setting_write_asset(output_root,key,tooltip)
    if key=='BuilderBooleanBuilder': return tool_write_asset(output_root,key,tooltip)
    relative = f'icons/cutters-minimal/{key}.svg'
    target = Path(output_root) / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    raw = svg(key).encode('utf-8')
    target.write_bytes(raw)
    label, meaning, spec, _ = CUTTERS_ICONS[key]
    metadata = {'image': relative, 'tooltip': tooltip or key,
                'source': 'OpenMatrix9-authored-svg', 'mapping': 'authored-command-symbol', 'status': 'resolved'}
    record = {'symbol': key, 'label': label, 'meaning': meaning, 'style': STYLE,
              'mapping': metadata['mapping'], 'sha256': hashlib.sha256(raw).hexdigest(),
              'palette': [c for c in (ORANGE, DARK, LIGHT, CREAM, QUAD, WHITE) if c in raw.decode()],
              'spec_id': spec,
              'semantic_source': 'local spec matched by command meaning' if spec else 'reference-cue-only',
              'reference_image': f'icons/rgb-plus5/ButtonIcons/{key}_1.png',
              'reference_type': 'selected Matrix90 RGB+5 derivative',
              'design_note': NOTES[key]}

    return metadata, {k:v for k,v in record.items() if k not in ('reference_image','reference_type','reference_images','palette_reference','reference_palette')}


def cutters_keys(resources):
    menu = configparser.ConfigParser(interpolation=None)
    menu.optionxform = str
    menu.read(Path(resources) / 'menu/MainMenu.ini', encoding='utf-8')
    groups = [s for s in menu.values() if s.get('Name') == 'Cutters']
    if len(groups) != 1:
        raise ValueError('Expected one Cutters group')
    group = groups[0]
    keys = [v for k,v in group.items() if re.fullmatch(r'Icon\d+', k)]
    if len(keys) != int(group['IconCount']) or set(keys) != set(CUTTERS_ICONS):
        raise ValueError('Cutters menu and authored catalog disagree')
    return keys


def contact_sheet(keys):
    columns, cell_w, cell_h = 8, 160, 132
    height = 64 + ((len(keys)+columns-1)//columns)*cell_h
    parts = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{columns*cell_w}" height="{height}">',
             '<rect width="100%" height="100%" fill="#333333"/>',
             '<text x="16" y="28" font-family="Arial,sans-serif" font-size="20" fill="white">OpenMatrix9 — Cutters / minimal SVG</text>',
             '<text x="16" y="48" font-family="Arial,sans-serif" font-size="12" fill="#cccccc">11 icons · orange cutters · cream cut faces · shared Scallop and Boolean · 64px and 24px samples</text>']
    for i,key in enumerate(keys):
        x,y = (i%columns)*cell_w,64+(i//columns)*cell_h
        label,_,_,geometry = CUTTERS_ICONS[key]
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
    report['cutters_icon_style'] = {'name': STYLE, 'count': count,
        'source': 'OpenMatrix9-authored-svg', 'generator': 'tools/cutters_icons.py',
        'geometry': 'authored geometric primitives; no tracing or embedded bitmaps',
        'palette_reference': 'selected RGB+5: orange #F2793A / dark #C05D05 / light #FFC88E, cream #FFFFCF, Quad Flip #FF6605, white #FFFFFF; shared Scallop/Boolean palettes unchanged',
        'priority': 'authored Cutters symbols precede legacy image sources'}
    prefix='authored Cutters SVG first; '
    if not report.get('asset_policy','').startswith(prefix):
        report['asset_policy']=prefix+report.get('asset_policy','')


def install(project_root):
    root=Path(project_root)
    resources=root/'Resources'
    keys=cutters_keys(resources)
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
    gallery=root/'docs/images/cutters-icons-minimal.svg'
    gallery.parent.mkdir(parents=True,exist_ok=True)
    gallery.write_text(contact_sheet(keys),encoding='utf-8')
    return {'count':len(keys),'style':STYLE,'preview':str(gallery)}


if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--project-root',type=Path,default=Path(__file__).resolve().parents[1])
    args=parser.parse_args()
    print(json.dumps(install(args.project_root)))
