"""Authored Builder SVGs from command meaning and the selected red references."""
import argparse
import configparser
import hashlib
import html
import json
from pathlib import Path
import re
from curve_icons import WHITE, path, arrow

RED='#D92D32'
LIGHT='#FF6065'
DARK='#990505'
ROPE='#C82623'
TEXT='#FF7168'
BLUSH='#F2BEA9'
STYLE='minimal-builder-v1-reviewed'
REVO_MANUAL='https://s3.us-east-2.amazonaws.com/gvwebsite/files/support/revo/revo540cx_manual.pdf'

def face(d,color=RED,opacity=.22,width=2.4):
    return path(d,color,width=width).replace('fill="none"',f'fill="{color}" fill-opacity="{opacity}"')

def ellipse(x,y,rx,ry,color=RED,width=2.4):
    return f'<ellipse cx="{x}" cy="{y}" rx="{rx}" ry="{ry}" fill="none" stroke="{color}" stroke-width="{width}"/>'

def gem(x,y,r=2.3):
    return face(f'M{x} {y-r} L{x+r} {y} L{x} {y+r} L{x-r} {y} Z',WHITE,.4,1.4)

def band():
    return face('M5 12 C5 5 27 5 27 12 V23 C27 31 5 31 5 23 Z')+ellipse(16,12,11,5,LIGHT)+path('M5 12 C5 20 27 20 27 12',DARK,width=1.7)

def cutter(x=16):
    return face(f'M{x-5} 3 H{x+5} V11 H{x-5} Z',RED,.3)+path(f'M{x-2} 3 V11 M{x+2} 3 V11',WHITE,width=1.6)+face(f'M{x-2} 11 H{x+2} V19 L{x} 23 L{x-2} 19 Z',WHITE,.15,1.8)

BUILDER_ICONS={
 'BuilderEternity':('Eternity band','Create a channel-set eternity ring','OM9-BUILDER-001',
    ellipse(16,16,11,13)+ellipse(16,16,7,9,LIGHT,width=1.8)
    +gem(16,4)+gem(8,8)+gem(5,16)+gem(8,24)+gem(16,28)
    +path('M22 7 Q28 16 22 25',WHITE,width=1.6)),
 'BuilderSignet':('Signet ring','Create a signet ring with an editable top and shank','OM9-BUILDER-002',
    face('M5 10 L9 4 H23 L27 10 L23 14 H9 Z',LIGHT,.28)
    +face('M5 10 C2 32 30 32 27 10 L23 14 C24 27 8 27 9 14 Z')
    +path('M9 4 H23 M9 14 H23',WHITE,width=1.7)),
 'ClayooCreationSignetRing':('Clayoo signet','Create a Clayoo signet ring','OM9-SUBD-018',
    face('M5 12 Q4 5 10 4 H23 Q29 5 28 12 L24 16 H9 Z',BLUSH,.45)
    +face('M5 12 C2 33 31 33 28 12 L24 16 C24 27 9 27 9 16 Z',WHITE,.05)
    +path('M5 12 L9 16 H24 L28 12 M9 16 C9 27 24 27 24 16',RED,width=1.8)
    +path('M12 5 L10 13 M22 5 L24 13',BLUSH,width=1.5)),
 'ClayooCreationRing':('Clayoo ring','Create a simple band from a Clayoo surface','OM9-SUBD-017',
    '<g transform="rotate(-24 16 16)">'+ellipse(16,16,10,13,WHITE,width=3.2)
    +ellipse(16,16,7,10,BLUSH,width=2)
    +path('M6 16 Q6 29 16 29 Q26 29 26 16',RED,width=1.8)
    +path('M6 12 L9 13 M23 13 L26 12 M10 27 L12 24 M20 24 L22 27',BLUSH,width=1.5)+'</g>'),
 'TextOnCurve':('Text on curve','Create 2D or 3D text along a curve','OM9-MATRIXTOOLS-004',
    path('M3 25 Q16 13 29 25',WHITE,width=2.2)
    +'<g transform="rotate(-18 9 16)">'+path('M5 18 L9 7 L13 18 M6.5 14 H11.5',TEXT,width=2.4)+'</g>'
    +'<g transform="rotate(18 23 16)">'+path('M19 18 V7 H24 Q29 7 25 12 Q30 16 25 18 Z M19 12 H25',TEXT,width=2.2)+'</g>'),
 'BuilderRaised':('Raised band','Place raised or sunken lettering or closed planar designs on a band','OM9-BUILDER-004',
    band()+path('M11 25 L16 16 L21 25 M13 22 H19',WHITE,width=2.6)
    +path('M11 28 L16 19',LIGHT,width=1.4)),
 'BuilderAwardRingBuilder':('Award ring top','Create an award ring top in conjunction with a signet ring','OM9-BUILDER-005',
    face('M8 3 H24 L29 8 V24 L24 29 H8 L3 24 V8 Z')
    +face('M10 8 H22 L24 10 V22 L22 24 H10 L8 22 V10 Z',WHITE,.08,2.2)
    +path('M8 3 L10 8 M24 3 L22 8 M29 8 L24 10 M29 24 L24 22 M24 29 L22 24 M8 29 L10 24 M3 24 L8 22 M3 8 L8 10',LIGHT,width=1.6)),
 'Builder_Rope':('Rope','Create a braided rope around an input curve','OM9-MATRIXTOOLS-003',
    path('M7 28 C1 20 11 3 27 5',ROPE,width=4)
    +path('M4 25 Q13 25 10 18 Q7 12 18 12 Q25 13 25 4',LIGHT,width=2.8)
    +path('M5 19 Q13 18 16 9 M11 8 Q16 10 21 6',RED,width=2.6)),
 'BuilderJump':('Jump ring','Sweep a jump ring using rail and section profiles','OM9-BUILDER-008',
    path('M17 15 C22 7 13 0 8 6 C0 14 5 27 13 22 L17 18',RED,width=3)
    +path('M15 16 C10 24 17 33 24 26 C32 18 28 5 20 10 L16 14',LIGHT,width=3)
    +path('M7 9 Q4 13 6 18 M25 14 Q28 18 25 23',RED,width=1.5)),
 'BuilderFreeFormKnot':('Freeform knot','Create custom freeform knot shapes','OM9-BUILDER-007',
    path('M19 10 C10 -1 1 9 7 17 L22 27 C30 32 32 17 24 16 L10 15 C0 14 8 31 16 25 L24 9 C29 1 16 2 16 8',RED,width=3)
    +path('M9 18 L14 21 M19 9 L22 12 M18 22 L20 18',LIGHT,width=2.2)),
 'BuilderKnot':('Celtic knot','Create a classic Celtic knot with a grid-based layout','OM9-BUILDER-006',
    path('M14 8 C5 0 0 9 8 16 L16 24 C24 32 32 23 24 16 L16 8',RED,width=3)
    +path('M18 8 C27 0 32 9 24 16 M22 18 L16 24 C8 32 0 23 8 16 L16 8',LIGHT,width=3)
    +path('M8 16 L12 20 M16 8 L20 12 M20 20 L24 16',RED,width=3)),
 'BuilderPatternBuilder':('Pattern Builder','Repeat patterns created from two input objects','OM9-BUILDER-009',
    ''.join(face(f'M{x} {y-2.5} L{x+2.5} {y} L{x} {y+2.5} L{x-2.5} {y} Z',RED,.2,1.8)
            +ellipse(x+7,y,2.2,2.2,LIGHT,width=1.8) for x,y in [(5,6),(5,16),(5,26),(21,6),(21,16),(21,26)])),
 'BuilderNautilusBuilder':('Nautilus','Create a nautilus or spiral from a provided circle','OM9-BUILDER-010',
    path('M29 27 V18 C29 0 3 -1 3 17 C3 32 25 33 25 18 C25 6 9 5 9 17 C9 26 21 27 21 18 C21 12 14 12 14 17 Q14 21 17 20',RED,width=3.2)
    +path('M27 13 C22 5 9 4 6 14',LIGHT,width=1.8)),
 'BuilderMillWork':('MillWork','Milling builder reference: vertical cutter and cutting path',None,
    cutter()+path('M4 28 H10 L14 25 H25 L28 28',RED,width=2.6)
    +path('M6 25 H11 M23 25 H27',LIGHT,width=1.6)),
 'BuilderMillArea':('Mill area','Milling area reference: rectangular fixture outline',None,
    face('M4 5 H28 V27 H4 Z',RED,.08,2.8)
    +face('M8 5 H24 V23 H8 Z',WHITE,.3,2.2)),
 'BuilderMillWorkC':('MillWork C','Milling builder C reference: vertical and horizontal cutters with C marker',None,
    '<g transform="translate(-4 -1) scale(.85)">'+cutter()+'</g>'
    +face('M28 20 V27 H22 V20 Z',RED,.3)+path('M28 22 H22 M28 25 H22',WHITE,width=1.5)
    +face('M22 22 H18 L14 24 L18 26 H22 Z',WHITE,.15,1.6)
    +path('M26 6 Q20 3 20 9 Q20 15 26 12',LIGHT,width=2.2)
    +path('M4 29 H12 L16 27 H26',RED,width=2.2)),
}

NOTES={
 'BuilderEternity':'Large channel band with separated white gem stations around the perimeter.',
 'BuilderSignet':'Broad flat top and tapered shoulders with a visible finger opening; no gemstone added.',
 'ClayooCreationSignetRing':'Rounded blush top, white shank and red inner edges preserve the separate Clayoo reference palette.',
 'ClayooCreationRing':'Simple white/blush oval band, red rim and a few subdivision stations; distinct from the gem-set eternity band.',
 'TextOnCurve':'Two geometric letters follow the white bowed baseline.',
 'BuilderRaised':'White raised lettering on the red band, including a separate depth edge.',
 'BuilderAwardRingBuilder':'Top view of the framed award ring top; retain red frame and white inset.',
 'Builder_Rope':'Curved red rope with broad interwoven strand highlights; not a chain of links.',
 'BuilderJump':'Two oval jump-ring links preserve the reference family cue; does not assert automatic chain generation.',
 'BuilderFreeFormKnot':'Irregular continuous knot silhouette distinguishes custom freeform geometry.',
 'BuilderKnot':'Regular square Celtic interlace distinguishes the classic grid-based builder.',
 'BuilderPatternBuilder':'Two distinct repeating input shapes, six stations; no unsupported claim that it creates a lattice.',
 'BuilderNautilusBuilder':'Large red spiral with a separated terminal and center turns.',
 'BuilderMillWork':'Keep vertical cutter and cutting-path cue; exact Matrix9 command mapping unverified.',
 'BuilderMillArea':'Keep white rectangular area and red fixture rim; dimensions and exact command mapping unverified.',
 'BuilderMillWorkC':'Keep paired cutter orientations and C marker visible; C meaning and exact Matrix9 command mapping unverified.',
}

def svg(key):
    label, meaning, spec, geometry = BUILDER_ICONS[key]
    return (f'<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32">\n'
            f'<title>{html.escape(label)}</title>\n<desc>{html.escape(meaning)}. '
            f'OpenMatrix9 {STYLE}; authored vector geometry.</desc>\n{geometry}\n</svg>\n')


def write_asset(output_root, key, tooltip=None):
    relative = f'icons/builder-minimal/{key}.svg'
    target = Path(output_root) / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    raw = svg(key).encode('utf-8')
    target.write_bytes(raw)
    label, meaning, spec, _ = BUILDER_ICONS[key]
    metadata = {'image': relative, 'tooltip': tooltip or key,
                'source': 'OpenMatrix9-authored-svg', 'mapping': 'authored-command-symbol', 'status': 'resolved'}
    record = {'symbol': key, 'label': label, 'meaning': meaning, 'style': STYLE,
              'mapping': metadata['mapping'], 'sha256': hashlib.sha256(raw).hexdigest(),
              'palette': [c for c in (RED, LIGHT, DARK, ROPE, TEXT, BLUSH, WHITE) if c in raw.decode()],
              'spec_id': spec,
              'semantic_source': 'local spec matched by command meaning' if spec else 'reference symbol; general milling context from Gemvision Revo manual',
              'semantic_status': 'local-spec-reviewed' if spec else 'reference-cue-only',
              'source_url': None if spec else REVO_MANUAL,
              'reference_image': f'icons/rgb-plus5/ButtonIcons/{key}_1.png',
              'reference_type': 'selected Matrix90 RGB+5 derivative',
              'design_note': NOTES[key]}

    return metadata, {k:v for k,v in record.items() if k not in ('reference_image','reference_type','reference_images','palette_reference','reference_palette')}


def builder_keys(resources):
    menu = configparser.ConfigParser(interpolation=None)
    menu.optionxform = str
    menu.read(Path(resources) / 'menu/MainMenu.ini', encoding='utf-8')
    groups = [s for s in menu.values() if s.get('Name') == 'Builder']
    if len(groups) != 1:
        raise ValueError('Expected one Builder group')
    group = groups[0]
    keys = [v for k,v in group.items() if re.fullmatch(r'Icon\d+', k)]
    if len(keys) != int(group['IconCount']) or set(keys) != set(BUILDER_ICONS):
        raise ValueError('Builder menu and authored catalog disagree')
    return keys


def contact_sheet(keys):
    columns, cell_w, cell_h = 8, 160, 132
    height = 64 + ((len(keys)+columns-1)//columns)*cell_h
    parts = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{columns*cell_w}" height="{height}">',
             '<rect width="100%" height="100%" fill="#333333"/>',
             '<text x="16" y="28" font-family="Arial,sans-serif" font-size="20" fill="white">OpenMatrix9 — Builder / minimal SVG</text>',
             '<text x="16" y="48" font-family="Arial,sans-serif" font-size="12" fill="#cccccc">16 icons · red jewelry / patterns · white gems / tools · blush Clayoo bands · 64px and 24px samples</text>']
    for i,key in enumerate(keys):
        x,y = (i%columns)*cell_w,64+(i//columns)*cell_h
        label,_,_,geometry = BUILDER_ICONS[key]
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
    report['builder_icon_style'] = {'name': STYLE, 'count': count,
        'source': 'OpenMatrix9-authored-svg', 'generator': 'tools/builder_icons.py',
        'geometry': 'authored geometric primitives; no tracing or embedded bitmaps',
        'palette_reference': 'selected RGB+5: red #D92D32; light #FF6065; dark #990505; rope #C82623; text #FF7168; blush #F2BEA9; white #FFFFFF',
        'priority': 'authored Builder symbols precede legacy image sources'}
    prefix='authored Builder SVG first; '
    if not report.get('asset_policy','').startswith(prefix):
        report['asset_policy']=prefix+report.get('asset_policy','')


def install(project_root):
    root=Path(project_root)
    resources=root/'Resources'
    keys=builder_keys(resources)
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
    gallery=root/'docs/images/builder-icons-minimal.svg'
    gallery.parent.mkdir(parents=True,exist_ok=True)
    gallery.write_text(contact_sheet(keys),encoding='utf-8')
    return {'count':len(keys),'style':STYLE,'preview':str(gallery)}


if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--project-root',type=Path,default=Path(__file__).resolve().parents[1])
    args=parser.parse_args()
    print(json.dumps(install(args.project_root)))
