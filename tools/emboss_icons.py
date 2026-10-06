"""Authored Emboss vectors, preserving each reference icon's color family."""
import argparse
import configparser
import hashlib
import html
import json
from pathlib import Path
import re
from curve_icons import WHITE, path, arrow

PALE='#FFFFB2'
GOLD='#FFE057'
GREEN='#05B677'
GRAY='#DCDCDC'
STYLE='minimal-emboss-v1-reviewed'

def face(d,color=PALE,opacity=.18,width=2.2):
    return path(d,color,width=width).replace('fill="none"',f'fill="{color}" fill-opacity="{opacity}"')

def circle(x,y,r,color=WHITE,fill=None):
    return f'<circle cx="{x}" cy="{y}" r="{r}" fill="{fill or "none"}" stroke="{color}" stroke-width="2.2"/>'

def motif():
    # Raised quatrefoil, with a separate base line showing relief height.
    return face('M16 9 C20 5 24 10 20 14 C25 17 21 22 17 19 C14 24 9 20 12 16 C7 13 11 8 16 9 Z',GOLD,.3)

EMBOSS_ICONS={
 'EmbossMatrixArt':('Matrix Art','Create a flat-backed relief mesh from closed curves and image texture','OM9-ART-001',
    face('M4 21 L12 6 Q16 2 21 7 L14 23 Z',WHITE,.1)
    +face('M13 25 L21 10 Q25 6 28 12 L21 27 Z',GREEN,.28)
    +path('M13 25 V29 L21 29 L28 14 V12',GREEN,width=2.2)
    +path('M5 24 L11 24',WHITE,width=1.6)),
 'EmbossHeightfieldImage':('Heightfield image','Create a grayscale heightfield image from selected geometry','OM9-EMBOSS-013',
    circle(9,9,5,WHITE,GRAY)+path('M6 8 Q7 5 10 5',WHITE,width=1.4)
    +arrow('M17 8 H26 V14','M23 11 L26 14 L29 11')
    +face('M4 18 H28 V29 H4 Z',GOLD,.12)
    +path('M9 26 V23 M15 26 V20 M21 26 V22',GRAY,width=3)),
 'TextureBuilder':('Texture Builder','Create surface texture from a grayscale image','OM9-MATRIXTOOLS-002',
    face('M4 13 L19 4 L29 19 L14 28 Z')
    +path('M7 16 L10 7 L14 13 L17 4 M12 24 L16 13 L20 20 L23 11 M18 26 L22 22 L26 23',GOLD,width=2.6)
    +path('M4 13 L14 28 L29 19',WHITE,width=1.7)),
 'EmbossClayEmboss':('Emboss','Create reliefs from curves, images and geometry','OM9-EMBOSS-001',
    circle(16,16,13)+motif()+path('M10 23 Q16 27 23 22',PALE,width=1.8)),
 'EmbossSculpt':('Sculpt','Convert geometry to a sculpting mesh and deform it with brushes','OM9-EMBOSS-012',
    circle(14,18,10)+path('M5 19 Q14 12 22 20 M14 8 Q8 18 14 28',PALE,width=1.6)
    +face('M18 16 L26 4 L29 6 L21 18 Z',WHITE,.15,1.8)
    +face('M18 16 Q13 16 15 23 Q21 24 21 18 Z',GOLD,.6,2.2)),
 'EmbossDecimator':('Decimator','Reduce mesh density while retaining shape detail','OM9-EMBOSS-015',
    face('M3 6 L8 2 L13 6 V16 L8 20 L3 16 Z')
    +path('M3 6 L13 16 M13 6 L3 16 M3 11 H13 M8 2 V20',GOLD,width=1.4)
    +face('M19 16 L24 12 L29 16 V26 L24 30 L19 26 Z')
    +path('M19 16 L29 26 M29 16 L19 26',WHITE,width=1.6)
    +arrow('M16 4 Q26 4 26 9','M23 6 L26 9 L29 6')),
 'EmbossClayEmbossResources':('Emboss resources','Library of profiles, texture images and heightfield images','OM9-EMBOSS-014',
    face('M3 9 V5 H12 L15 9 H28 V27 H3 Z',PALE,.25)
    +path('M3 13 H28',GOLD,width=1.8)
    +path('M7 23 Q10 13 14 23 Q17 26 20 19 L24 23',WHITE,width=2.2)),
}

NOTES={
 'EmbossMatrixArt':'Keep paired white/green relief silhouettes and show a flat back; no generic paint palette.',
 'EmbossHeightfieldImage':'Gray geometry transfers to a framed gray height map; does not suggest terrain generation from an image.',
 'TextureBuilder':'Broad yellow ridges on a surface indicate applied texture; no bitmap tracing.',
 'EmbossClayEmboss':'White circular family cue with a raised yellow motif and offset base contour.',
 'EmbossSculpt':'White circular mesh and a large brush contacting the gold surface distinguish it from Emboss.',
 'EmbossDecimator':'Two matching mesh silhouettes, fewer internal edges on the result and a transfer arrow.',
 'EmbossClayEmbossResources':'Retain pale yellow folder cue and add a profile curve representing the resource library.',
}

def svg(key):
    label, meaning, spec, geometry = EMBOSS_ICONS[key]
    return (f'<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32">\n'
            f'<title>{html.escape(label)}</title>\n<desc>{html.escape(meaning)}. '
            f'OpenMatrix9 {STYLE}; authored vector geometry.</desc>\n{geometry}\n</svg>\n')


def write_asset(output_root, key, tooltip=None):
    relative = f'icons/emboss-minimal/{key}.svg'
    target = Path(output_root) / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    raw = svg(key).encode('utf-8')
    target.write_bytes(raw)
    label, meaning, spec, _ = EMBOSS_ICONS[key]
    metadata = {'image': relative, 'tooltip': tooltip or key,
                'source': 'OpenMatrix9-authored-svg', 'mapping': 'authored-command-symbol', 'status': 'resolved'}
    record = {'symbol': key, 'label': label, 'meaning': meaning, 'style': STYLE,
              'mapping': metadata['mapping'], 'sha256': hashlib.sha256(raw).hexdigest(),
              'palette': [c for c in (PALE, GOLD, GREEN, WHITE, GRAY) if c in raw.decode()],
              'spec_id': spec,
              'semantic_source': 'local spec matched by command meaning',
              'reference_image': f'icons/rgb-plus5/ButtonIcons/{key}_1.png',
              'reference_type': 'selected Matrix90 RGB+5 derivative',
              'design_note': NOTES[key]}

    return metadata, {k:v for k,v in record.items() if k not in ('reference_image','reference_type','reference_images','palette_reference','reference_palette')}


def emboss_keys(resources):
    menu = configparser.ConfigParser(interpolation=None)
    menu.optionxform = str
    menu.read(Path(resources) / 'menu/MainMenu.ini', encoding='utf-8')
    groups = [s for s in menu.values() if s.get('Name') == 'Emboss']
    if len(groups) != 1:
        raise ValueError('Expected one Emboss group')
    group = groups[0]
    keys = [v for k,v in group.items() if re.fullmatch(r'Icon\d+', k)]
    if len(keys) != int(group['IconCount']) or set(keys) != set(EMBOSS_ICONS):
        raise ValueError('Emboss menu and authored catalog disagree')
    return keys


def contact_sheet(keys):
    columns, cell_w, cell_h = 4, 180, 132
    height = 64 + ((len(keys)+columns-1)//columns)*cell_h
    parts = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{columns*cell_w}" height="{height}">',
             '<rect width="100%" height="100%" fill="#333333"/>',
             '<text x="16" y="28" font-family="Arial,sans-serif" font-size="20" fill="white">OpenMatrix9 — Emboss / minimal SVG</text>',
             '<text x="16" y="48" font-family="Arial,sans-serif" font-size="12" fill="#cccccc">7 icons · pale yellow / gold reliefs · white tools · green Matrix Art · 64px and 24px samples</text>']
    for i,key in enumerate(keys):
        x,y = (i%columns)*cell_w,64+(i//columns)*cell_h
        label,_,_,geometry = EMBOSS_ICONS[key]
        parts += [f'<g transform="translate({x} {y})">',
                  '<rect x="4" y="4" width="172" height="124" rx="4" fill="#696969"/>',
                  f'<text x="12" y="20" font-family="Arial,sans-serif" font-size="11" fill="#dddddd">{i+1:02}</text>',
                  f'<g transform="translate(24 24) scale(2)">{geometry}</g>',
                  f'<g transform="translate(140 45) scale(.75)">{geometry}</g>']
        lines=[]
        for word in label.split():
            if lines and len(lines[-1])+len(word)+1 <= 24:
                lines[-1] += ' '+word
            else:
                lines.append(word)
        parts += [f'<text x="90" y="{98+n*12}" text-anchor="middle" font-family="Arial,sans-serif" font-size="10.5" fill="white">{html.escape(line)}</text>' for n,line in enumerate(lines)]
        parts.append('</g>')
    return '\n'.join(parts+['</svg>\n'])


def describe_style(report, count):
    report['emboss_icon_style'] = {'name': STYLE, 'count': count,
        'source': 'OpenMatrix9-authored-svg', 'generator': 'tools/emboss_icons.py',
        'geometry': 'authored geometric primitives; no tracing or embedded bitmaps',
        'palette_reference': 'selected RGB+5: pale #FFFFB2, gold #FFE057, white #FFFFFF, gray #DCDCDC; green #05B677 only Matrix Art',
        'priority': 'authored Emboss symbols precede legacy image sources'}
    prefix='authored Emboss SVG first; '
    if not report.get('asset_policy','').startswith(prefix):
        report['asset_policy']=prefix+report.get('asset_policy','')


def install(project_root):
    root=Path(project_root)
    resources=root/'Resources'
    keys=emboss_keys(resources)
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
    gallery=root/'docs/images/emboss-icons-minimal.svg'
    gallery.parent.mkdir(parents=True,exist_ok=True)
    gallery.write_text(contact_sheet(keys),encoding='utf-8')
    return {'count':len(keys),'style':STYLE,'preview':str(gallery)}


if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--project-root',type=Path,default=Path(__file__).resolve().parents[1])
    args=parser.parse_args()
    print(json.dumps(install(args.project_root)))
