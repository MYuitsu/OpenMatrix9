"""Authored Setting icons; shared SubD Bezel retains its approved vector and palette."""
import argparse
import configparser
import hashlib
import html
import json
from pathlib import Path
import re
from curve_icons import WHITE, path, arrow, points
from subd_icons import SUBD_ICONS, svg as subd_svg, write_asset as subd_write_asset

PINK = '#C655A3'
DARK = '#A73283'
LIGHT = '#F6C3E6'
BEAD = '#FF67D0'
ORANGE = '#F3782B'
ORANGE_LIGHT = '#FF9C5C'
GOLD = '#FFD705'
STYLE = 'minimal-settings-v1-reviewed'
SHARED = {'ClayooCreationClayooBezel'}


def face(d, color=PINK, fill=DARK, opacity=.3, width=2.6):
    return path(d,color,width=width).replace('fill="none"',f'fill="{fill}" fill-opacity="{opacity}"')


def ellipse(x,y,rx,ry,color=PINK,width=2.4):
    return f'<ellipse cx="{x}" cy="{y}" rx="{rx}" ry="{ry}" fill="none" stroke="{color}" stroke-width="{width}"/>'


def gem(x=16,y=10,r=9):
    return face(f'M{x-r} {y} L{x-r*.55} {y-r*.55} H{x+r*.55} L{x+r} {y} L{x} {y+r*.8} Z',WHITE,LIGHT,.15,1.7)+path(
        f'M{x-r} {y} H{x+r} M{x-r*.55} {y-r*.55} L{x} {y+r*.8} L{x+r*.55} {y-r*.55}',WHITE,width=1.4)


def clock(x=26,y=26):
    return ellipse(x,y,4,4,GOLD,width=1.8)+path(f'M{x} {y-2} V{y} H{x+2}',GOLD,width=1.4)


def patch():
    return face('M3 13 Q14 6 27 10 L29 27 Q17 23 5 29 Z',PINK,DARK,.18,1.8)


def prong(x=16,y=15,height=18,width=6):
    a,b=x-width/2,x+width/2
    return (face(f'M{a} {y-height/2} Q{x} {y-height/2-3} {b} {y-height/2} V{y+height/2} Q{x} {y+height/2+2} {a} {y+height/2} Z')
            +path(f'M{a} {y-height/2} Q{x} {y-height/2+2} {b} {y-height/2}',LIGHT,width=1.5))


def bead(x,y,r=4):
    return (f'<circle cx="{x}" cy="{y}" r="{r}" fill="{PINK}" fill-opacity=".35" stroke="{BEAD}" stroke-width="2.2"/>'
            +path(f'M{x-r*.4} {y-r*.25} Q{x-r*.2} {y-r*.6} {x+r*.25} {y-r*.55}',LIGHT,width=1.5))


SETTINGS_ICONS = {
 'BuilderHead': ('Head Builder','Create a basket or half-bezel head around an existing stone','OM9-SETTING-001',
     gem(16,10,9)+ellipse(16,24,7,4)
     +path('M5 7 L10 25 M27 7 L22 25 M16 16 V28 M5 7 V4 M27 7 V4',PINK,width=2.8)
     +path('M8 18 Q16 22 24 18',LIGHT,width=1.7)),
 'BuilderBezel': ('Bezel Builder','Create a bezel around an existing gem','OM9-SETTING-002',
     face('M4 11 V24 Q14 31 25 24 V11 Z')+ellipse(14.5,11,10.5,6,PINK)
     +gem(14.5,9,7)+path('M4 15 Q14.5 22 25 15',LIGHT,width=1.6)+clock()),
 'ClayooCreationClayooBezel': SUBD_ICONS['ClayooCreationClayooBezel'],
 'BuilderPullToRail': ('Pull Object to Rail','Pull a head or other object to meet the finger rail','OM9-SETTING-005',
     ellipse(15,24,12,5,WHITE,width=1.8)+gem(15,8,8)
     +path('M6 7 L9 23 M24 7 L21 23 M9 23 Q15 27 21 23',PINK,width=2.8)
     +arrow('M28 10 V21','M25 18 L28 21 L30 18')),
 'BuilderBezelCutter': ('Scallop Bezel','Create scalloped bezel designs with shaped cutters','OM9-SETTING-006',
     face('M4 10 Q8 19 12 10 Q16 19 20 10 Q24 19 28 10 L24 25 Q16 31 8 25 Z',ORANGE,ORANGE,.16,2.6)
     +path('M4 10 V5 M12 10 V4 M20 10 V4 M28 10 V5 M8 25 Q16 21 24 25',ORANGE_LIGHT,width=2)
     +path('M7 9 L9 15 M15 9 L17 15 M23 9 L25 15',WHITE,width=1.5)),
 'BuilderProngAdder': ('Prong Builder','Add prongs to a line of gems','OM9-SETTING-007',
     gem(9,17,5.5)+gem(23,17,5.5)
     +prong(4.5,15,12,3)+prong(16,15,12,3)+prong(27.5,15,12,3)),
 'BuilderProngEditor': ('Prong Editor','Edit an individual prong using control handles','OM9-SETTING-008',
     prong(14,16,20,9)+points((14,4),(14,28),color=WHITE)
     +arrow('M22 14 H29','M26 11 L29 14 L26 17')
     +face('M24 22 H28 V26 H24 Z',WHITE,WHITE,.3,1.5)),
 'BuilderProngOnSurface': ('Prong on Surface','Place individual prongs on a selected surface','OM9-SETTING-009',
     patch()+prong(16,14,16,7)),
 'BuilderBeadOnSurface': ('Bead on Surface','Place beads or objects on a selected surface','OM9-SETTING-010',
     patch()+bead(16,16,7)),
 'BuilderMetalPiece': ('Metal from Gems','Create a history-enabled metal piece around a line or loop of gems','OM9-SETTING-011',
     face('M3 7 H29 V23 L23 29 H3 Z')+path('M3 23 H23 L29 17 M23 23 V29',LIGHT,width=1.6)
     +gem(9,13,5)+gem(22,13,5)+clock()),
 'BuilderBeadOnCrv': ('Bead on Curve / Milgrain','Set beads along a curve to create milgrain','OM9-SETTING-012',
     path('M3 27 Q13 13 29 5',WHITE,width=1.8)+bead(6,24,3.5)+bead(13,17,3.5)+bead(21,11,3.5)+bead(28,6,2.5)),
 'BuilderChannelBorderCurve': ('Channel Border Curve','Create an offset border curve around a group of gem curves','OM9-SETTING-013',
     path('M7 5 Q16 1 25 5 L27 25 Q16 31 5 25 Z',PINK,width=2.6)
     +ellipse(16,10,5,3,WHITE,width=1.5)+ellipse(16,21,5,3,WHITE,width=1.5)
     +points((7,5),(27,25),color=LIGHT,radius=1.9)),
}

NOTES = {
 'BuilderHead':'Faceted white stone inside separate pink basket posts and gallery rail; distinct from continuous bezel walls.',
 'BuilderBezel':'Continuous pink wall and closed upper rim surrounding a white stone, with gold History cue.',
 'BuilderPullToRail':'Pink head posts meet the white finger rail; downward arrow depicts reaching the rail, not curve projection.',
 'BuilderBezelCutter':'Orange scalloped openings and light-orange cutter stations retain this command’s separate cutter palette.',
 'BuilderProngAdder':'Multiple pink prongs around a line of two white stones, rather than one edited prong.',
 'BuilderProngEditor':'One large pink prong with white grips, a dimension arrow and scale handle.',
 'BuilderProngOnSurface':'Tall pink post rises from a bowed pink surface.',
 'BuilderBeadOnSurface':'Round bright-pink bead sits on the same surface context; distinct from a tall prong.',
 'BuilderMetalPiece':'White gems enclosed by a thick pink metal body with a gold History cue.',
 'BuilderBeadOnCrv':'Separated pink spherical beads follow a white curve; no faceted gem substitution.',
 'BuilderChannelBorderCurve':'Closed pink offset border surrounds two separate white gem-profile curves.',
}


def svg(key):
    if key in SHARED: return subd_svg(key)
    label, meaning, spec, geometry = SETTINGS_ICONS[key]
    return (f'<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32">\n'
            f'<title>{html.escape(label)}</title>\n<desc>{html.escape(meaning)}. '
            f'OpenMatrix9 {STYLE}; authored vector geometry.</desc>\n{geometry}\n</svg>\n')


def write_asset(output_root, key, tooltip=None):
    if key in SHARED: return subd_write_asset(output_root,key,tooltip)
    relative = f'icons/settings-minimal/{key}.svg'
    target = Path(output_root) / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    raw = svg(key).encode('utf-8')
    target.write_bytes(raw)
    label, meaning, spec, _ = SETTINGS_ICONS[key]
    metadata = {'image': relative, 'tooltip': tooltip or key,
                'source': 'OpenMatrix9-authored-svg', 'mapping': 'authored-command-symbol', 'status': 'resolved'}
    record = {'symbol': key, 'label': label, 'meaning': meaning, 'style': STYLE,
              'mapping': metadata['mapping'], 'sha256': hashlib.sha256(raw).hexdigest(),
              'palette': [c for c in (PINK, DARK, LIGHT, BEAD, ORANGE, ORANGE_LIGHT, GOLD, WHITE) if c in raw.decode()],
              'spec_id': spec,
              'semantic_source': 'local spec matched by command meaning' if spec else 'reference-cue-only',
              'reference_image': f'icons/rgb-plus5/ButtonIcons/{key}_1.png',
              'reference_type': 'selected Matrix90 RGB+5 derivative',
              'design_note': NOTES[key]}

    return metadata, {k:v for k,v in record.items() if k not in ('reference_image','reference_type','reference_images','palette_reference','reference_palette')}


def settings_keys(resources):
    menu = configparser.ConfigParser(interpolation=None)
    menu.optionxform = str
    menu.read(Path(resources) / 'menu/MainMenu.ini', encoding='utf-8')
    groups = [s for s in menu.values() if s.get('Name') == 'Settings']
    if len(groups) != 1:
        raise ValueError('Expected one Settings group')
    group = groups[0]
    keys = [v for k,v in group.items() if re.fullmatch(r'Icon\d+', k)]
    if len(keys) != int(group['IconCount']) or set(keys) != set(SETTINGS_ICONS):
        raise ValueError('Settings menu and authored catalog disagree')
    return keys


def contact_sheet(keys):
    columns, cell_w, cell_h = 8, 160, 132
    height = 64 + ((len(keys)+columns-1)//columns)*cell_h
    parts = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{columns*cell_w}" height="{height}">',
             '<rect width="100%" height="100%" fill="#333333"/>',
             '<text x="16" y="28" font-family="Arial,sans-serif" font-size="20" fill="white">OpenMatrix9 — Settings / minimal SVG</text>',
             '<text x="16" y="48" font-family="Arial,sans-serif" font-size="12" fill="#cccccc">12 icons · pink settings · orange scallop cutter · gold History · shared SubD bezel · 64px and 24px samples</text>']
    for i,key in enumerate(keys):
        x,y = (i%columns)*cell_w,64+(i//columns)*cell_h
        label,_,_,geometry = SETTINGS_ICONS[key]
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
    report['settings_icon_style'] = {'name': STYLE, 'count': count,
        'source': 'OpenMatrix9-authored-svg', 'generator': 'tools/settings_icons.py',
        'geometry': 'authored geometric primitives; no tracing or embedded bitmaps',
        'palette_reference': 'selected RGB+5: pink #C655A3 / dark #A73283 / light #F6C3E6, beads #FF67D0, cutter #F3782B / #FF9C5C, History #FFD705, white #FFFFFF; shared SubD bezel unchanged',
        'priority': 'authored Settings symbols precede legacy image sources'}
    prefix='authored Settings SVG first; '
    if not report.get('asset_policy','').startswith(prefix):
        report['asset_policy']=prefix+report.get('asset_policy','')


def install(project_root):
    root=Path(project_root)
    resources=root/'Resources'
    keys=settings_keys(resources)
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
    gallery=root/'docs/images/settings-icons-minimal.svg'
    gallery.parent.mkdir(parents=True,exist_ok=True)
    gallery.write_text(contact_sheet(keys),encoding='utf-8')
    return {'count':len(keys),'style':STYLE,'preview':str(gallery)}


if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--project-root',type=Path,default=Path(__file__).resolve().parents[1])
    args=parser.parse_args()
    print(json.dumps(install(args.project_root)))
