"""Authored Gems SVGs preserving reference color roles and verified command cues."""
import argparse
import configparser
import hashlib
import html
import json
import math
from pathlib import Path
import re
from curve_icons import WHITE, path, arrow, points

BLUE = '#7CB8F5'
DEEP = '#3E88C8'
LIGHT = '#CDE7FF'
CUSTOM = '#4987FF'
CUSTOM_LIGHT = '#90B0FF'
YELLOW = '#FFE81A'
BADGE = '#FFF305'
GOLD = '#FFE105'
MINT = '#74E2CB'
PINK = '#CD7586'
INK = '#352B05'
STYLE = 'minimal-gems-v1-reviewed'


def face(d, color=BLUE, fill=DEEP, opacity=.35, width=2.4):
    return path(d, color, width=width).replace('fill="none"', f'fill="{fill}" fill-opacity="{opacity}"')


def circle(x, y, r, color=BLUE, width=1.7, fill=DEEP):
    return (f'<circle cx="{x}" cy="{y}" r="{r}" fill="{fill}" fill-opacity=".32" '
            f'stroke="{color}" stroke-width="{width}"/>')


def top(x=16, y=16, r=11, color=BLUE):
    # Large octagonal gem with a table; small gems retain one broad facet cue.
    vertices = [(x+r*math.cos(math.pi/8+i*math.pi/4), y+r*math.sin(math.pi/8+i*math.pi/4)) for i in range(8)]
    d = 'M'+' L'.join(f'{a:.2f} {b:.2f}' for a,b in vertices)+' Z'
    result = face(d, color, color, .24, 2.4 if r>7 else 1.7)
    if r>7:
        table = f'M{x-r*.4} {y-r*.4} H{x+r*.4} V{y+r*.4} H{x-r*.4} Z'
        result += path(table, LIGHT, width=1.6)
        for sx,sy in [(-1,-1),(1,-1),(1,1),(-1,1)]:
            result += path(f'M{x+sx*r*.4} {y+sy*r*.4} L{x+sx*r*.707:.2f} {y+sy*r*.707:.2f}', LIGHT, width=1.4)
    elif r>=4:
        result += path(f'M{x-r*.48} {y} L{x} {y-r*.48} L{x+r*.48} {y}', LIGHT, width=1.4)
    return result


def diamond(color=BLUE):
    return (face('M3 11 L9 4 H23 L29 11 L16 28 Z', color, color, .3, 2.6)
            +path('M3 11 H29 M9 4 L11 11 L16 28 L21 11 L23 4 M11 11 L16 4 L21 11', LIGHT, width=1.7))


def baguette(x=16, y=16, w=18, h=23):
    return face(f'M{x-w/2} {y-h/2} H{x+w/2} L{x+w*.35} {y+h/2} H{x-w*.35} Z')+path(
        f'M{x-w*.25} {y-h*.28} H{x+w*.25} L{x+w*.18} {y+h*.28} H{x-w*.18} Z', LIGHT, width=1.7)


def rail():
    return path('M3 27 Q15 9 29 8', YELLOW, width=2.2)


def curve_gems():
    return rail()+top(7,23,4)+top(16,14,4)+top(26,10,4)


def patch():
    return face('M3 13 Q13 5 27 9 L29 27 Q17 22 5 29 Z', DEEP, BLUE, .12, 1.7)


def channel():
    return path('M3 8 Q16 4 29 10 M3 26 Q16 20 29 26', WHITE, width=2)


def pave():
    return top(10,8,4)+top(22,8,4)+top(5,18,3.5)+top(16,18,4.5)+top(27,18,3.5)+top(10,28,3)+top(22,28,3)


def document():
    return face('M5 3 H21 L27 9 V29 H5 Z', WHITE, DEEP, .12, 1.7)+path('M21 3 V9 H27', LIGHT, width=1.5)


def style_file():
    return document()+'<g transform="translate(3 4) scale(.55)">'+diamond()+'</g>'


def halo():
    result=top(16,16,6)
    for x,y in [(16,4),(25,7),(28,16),(25,25),(16,28),(7,25),(4,16),(7,7)]:
        result+=circle(x,y,2.3,BLUE,width=1.7)
    return result


GEMS_ICONS = {
 'BuilderGemLoader': ('Gem Loader', 'Load gems with selected shape, size and cut', 1, diamond()),
 'BuilderGemOnCrv': ('Gem on Curve', 'Set gems along a curve', 2, curve_gems()),
 'BuilderGemOnCrvAdvanced': ('Gem on Curve Advanced', 'Set and offset gems along a curve', 3,
     path('M3 28 Q15 15 29 15',YELLOW,width=2)+top(8,20,4)+top(20,7,5)
     +arrow('M27 14 V4','M24 7 L27 4 L30 7',BADGE)),
 'BuilderGemCountOnCrv': ('Gem Count on Curve', 'Place a specified number of gems along a curve', 4,
     path('M3 28 Q16 13 29 21',YELLOW,width=2)+top(7,25,4)+top(17,20,4)+top(26,23,3.8)
     +face('M3 3 H15 V14 H3 Z',BADGE,BADGE,1,1.4)
     +path('M7 5 L6 12 M11 5 L10 12 M5 7 H13 M5 10 H13',INK,width=1.4)),
 'BuilderGemListOnCrv': ('Gem List on Curve', 'Arrange a chosen list of gems along a curve', 5,
     path('M4 27 Q17 13 29 21',YELLOW,width=2)+top(8,24,4)+top(19,20,5)+top(28,23,2.4)
     +path('M3 3 H15 V13 H3 Z M6 6 H12 M6 10 H12',BADGE,width=1.8)),
 'BuilderGemOnCrvMulti': ('Gem on Curve Multi', 'Layout multiple rows of gems simultaneously', 6,
     path('M3 17 Q16 8 29 12 M3 29 Q16 20 29 24',YELLOW,width=1.7)
     +top(6,14,3)+top(16,10,3)+top(26,11,3)+top(6,26,3)+top(16,22,3)+top(26,23,3)),
 'BuilderGemProfile': ('Gem Profile Curve', 'Create a profile curve around a gem girdle', 7,
     '<g transform="translate(16 16) rotate(-28) scale(.75 1) translate(-16 -16)">'+top(16,16,9)+'</g>'
     +'<ellipse cx="16" cy="16" rx="10.5" ry="13.5" fill="none" stroke="#FFFFFF" stroke-width="2.6" transform="rotate(-28 16 16)"/>'),
 'BuilderGemGuides': ('Gem Guides', 'Show gem direction and create profile or prong guides', 8,
     top(16,16,9)+path('M3 16 H6 M26 16 H29 M16 3 V6 M16 26 V29',YELLOW,width=2)
     +path('M5 9 V5 H9 M23 5 H27 V9 M27 23 V27 H23 M9 27 H5 V23',WHITE,width=1.4)),
 'BuilderOrientToGem': ('Orient to Gem', 'Orient and scale objects to selected gems', 9,
     face('M3 5 H10 V12 H3 Z',WHITE,LIGHT,.2,1.7)+top(22,21,8)
     +arrow('M13 6 Q26 6 26 11','M23 8 L26 11 L29 8')),
 'BuilderHalo': ('Halo Builder', 'Build a halo around an existing gem', 10, halo()),
 'BuilderGemOnSurface': ('Gem on Surface', 'Place gems one at a time on a surface', 11,
     patch()+top(15,16,6)+path('M22 21 L29 23 L25 25 L24 29 Z',WHITE,width=1.5)),
 'BuilderCustomGem': ('Custom Gem Builder', 'Create a faceted gem from a custom planar outline', 12,
     diamond(CUSTOM)+points((9,4),(23,4),(16,28),color=CUSTOM_LIGHT,radius=1.9)),
 'BuilderEmerald': ('Emerald Builder', 'Create a clipped-corner emerald cut with precise dimensions', 13,
     face('M10 3 H22 L27 8 V24 L22 29 H10 L5 24 V8 Z')
     +path('M12 8 H20 L22 10 V22 L20 24 H12 L10 22 V10 Z M5 8 L10 10 M27 8 L22 10 M5 24 L10 22 M27 24 L22 22',LIGHT,width=1.6)),
 'BuilderBaguette': ('Baguette Builder', 'Create a baguette cut with independently controlled top and bottom widths', 14, baguette()),
 'BuilderTaperBagChannel': ('Baguette Channel', 'Place baguettes in a channel defined by two curves', 15,
     face('M3 4 H29 V28 H3 Z',DEEP,DEEP,.12,1.7)
     +baguette(9,16,8,17)+baguette(22,16,8,17)),
 'BuilderTaperBagBetweenTwoCurves': ('Baguettes Between Curves', 'Reference cue: a tapered baguette between two bounding curves', None,
     channel()+baguette(16,16,13,13)),
 'BuilderGemBetweenTwoCurves': ('Gems Between 2 Curves', 'Create a gem layout touching two selected curves', 16,
     channel()+top(9,16,6)+top(23,17,5.5)),
 'BuilderClusterBuilderOld': ('Cluster Builder', 'Build a cluster head with edge gems, under bezels and prongs', 17,
     top(16,16,7)+circle(16,4,2.2,WHITE)+circle(27,10,2.2,WHITE)+circle(27,23,2.2,WHITE)
     +circle(16,28,2.2,WHITE)+circle(5,23,2.2,WHITE)+circle(5,10,2.2,WHITE)
     +path('M4 16 H7 M25 16 H28',WHITE,width=2)),
 'BuilderGemReporter': ('Gem Reporter', 'View and save the list, counts and weights of gems in use', 19,
     document()+top(12,11,4)+path('M19 10 H23 M19 14 H23 M8 20 H23 M8 25 H23 M15 18 V27',BLUE,width=1.7)),
 'BuilderGemMap': ('Gem Map', 'Diagram gem sizes and locations in the design', 18,
     path('M9 11 L16 5 L24 11 L24 23 L16 28 L8 22 Z',MINT,width=1.8)
     +top(16,16,6)+circle(5,15,3,PINK)+circle(27,15,3,PINK)
     +circle(16,4,2.2,MINT)+circle(16,28,2.2,MINT)),
 'BuilderPaveDialog': ('Auto Pave Builder', 'Create interacting pave layouts with size, spacing and direction controls', 20,
     '<g transform="translate(3 3) scale(.8)">'+pave()+'</g>'
     +path('M2 8 V2 H8 M24 2 H30 V8 M30 24 V30 H24 M8 30 H2 V24',YELLOW,width=2)),
 'BuilderPaveBuilder': ('Pave Builder', 'Create a hexagonal pave layout with specified quantities, sizes and spacing', 21,
     patch()+top(9,11,4)+top(21,11,4)+top(15,21,4)+top(27,21,3)),
 'BuilderPaveAzureBuilder': ('Pave Azure Builder', 'Add underside azures and through holes to a pave layout', 22,
     face('M3 9 L24 4 L29 9 V25 L7 30 L3 25 Z',BLUE,BLUE,.12)
     +path('M3 9 L7 14 L29 9 M7 14 V30',LIGHT,width=1.5)
     +face('M11 18 L16 16 L17 22 L12 24 Z',DEEP,DEEP,.9,1.7)
     +face('M21 15 L25 14 L26 20 L22 21 Z',DEEP,DEEP,.9,1.7)),
 'BuilderPaveProngBuilder': ('Pave Prong Builder', 'Add shared and unshared prongs to a pave layout', 23,
     top(8,12,6)+top(24,12,6)+top(16,25,5)
     +points((16,12),(8,22),(24,22),color=WHITE,radius=2.1)),
 'BuilderGemSprings': ('Gem Springs', 'Edit gem positions and sizes while surrounding gems reposition', 25,
     top(5,16,3)+top(16,8,4)+top(27,16,3)+top(16,27,3)
     +path('M7 17 L10 15 L12 19 L15 16 L18 20 L22 17 L25 18',BLUE,width=2)
     +arrow('M16 16 V10','M13 13 L16 10 L19 13')),
 'BuilderGemFollow': ('Gem Follow', 'Change how gem north axes follow each other in a gem line', 24,
     top(7,24,5)+top(24,8,5)+arrow('M9 22 Q18 17 22 10','M18 12 L22 10 L22 14',GOLD)),
 'BuilderGemControl': ('Gem Control', 'Aim gem north or culet axes toward, away from or tangent to a control object', 26,
     top(9,22,6)+top(23,22,6)+points((16,4),color=WHITE)
     +arrow('M9 15 L14 7','M11 8 L14 7 L14 10')+arrow('M23 15 L18 7','M18 10 L18 7 L21 8')),
 'OthersMatchAttributes': ('Match Attributes', 'Transfer saved setting or cutter styles from one gem to another', 27,
     top(7,22,5)+top(25,22,5)+path('M3 29 H11 M21 29 H29',WHITE,width=1.7)
     +arrow('M6 12 Q16 2 26 12','M26 7 V12 H21')),
 'OthersSaveStyle': ('Save Styles', 'Save setting or cutter parameters to a style sheet', 28,
     style_file()+arrow('M29 13 V25 H20','M23 22 L20 25 L23 28',GOLD)),
 'OthersLoadStyle': ('Load Styles', 'Apply saved setting or cutter styles to the current design', 30,
     style_file()+arrow('M20 25 H28 V13','M25 16 L28 13 L30 16',GOLD)),
 'BuilderGVDGemFlow': ('Gem Flow', 'Flow gems from a base curve or surface to a target curve or surface', 29,
     path('M3 10 H14 M17 29 Q19 19 29 17',GOLD,width=2)
     +top(5,6,3)+top(12,6,3)+top(21,25,3)+top(27,19,3)
     +arrow('M18 5 Q28 5 28 12','M25 9 L28 12 L30 9')),
 'BuilderGVDGemSplop': ('Gem Splop', 'Place a gem on a surface using a reference sphere', 31,
     patch()+circle(16,16,10,WHITE,width=1.5,fill=DEEP)+top(16,17,6)
     +arrow('M26 3 V10','M23 7 L26 10 L29 7')),
 'BuilderPaveSphere': ('Pave Sphere', 'Create a spherical pave layout with or without an existing sphere', 32,
     circle(16,16,13,DEEP,width=1.8)+top(16,6,3)+top(7,13,3)+top(24,13,3)
     +top(16,17,4)+top(9,25,2.5)+top(23,25,2.5)),
 'BuilderGemUpdate': ('Gem Update', 'Restore lost gem recognition on model gems', 34,
     '<g transform="translate(0 0) scale(.68)">'+diamond()+'</g>'
     +path('M17 19 L21 15 Q25 12 28 16 Q30 19 26 22 L21 27 Q17 30 14 26 Q12 23 16 20 M18 24 L25 17',GOLD,width=2.4)),
 'BuilderGemPositioner': ('Gem Positioner', 'Move, scale, rotate and add gems with viewport handles', 33,
     top(16,17,7)+arrow('M16 9 V3','M13 6 L16 3 L19 6')
     +arrow('M24 17 H29','M26 14 L29 17 L26 20')
     +path('M6 22 Q4 27 11 29 L8 25 M11 29 H6',BLUE,width=2)
     +face('M24 25 H28 V29 H24 Z',LIGHT,LIGHT,.6,1.5)),
}

NOTES = {
 'BuilderGemLoader': 'Broad crown/table and pavilion facets retain the blue gemstone silhouette.',
 'BuilderGemOnCrv': 'Three gems on one yellow curve; no surface context.',
 'BuilderGemOnCrvAdvanced': 'Offset gem row and vertical yellow control distinguish advanced placement.',
 'BuilderGemCountOnCrv': 'Yellow count badge and three gem stations; the displayed count is a symbolic example.',
 'BuilderGemListOnCrv': 'Yellow list and a sequence of different gem sizes distinguish an explicit gem list.',
 'BuilderGemOnCrvMulti': 'Two separate yellow curves with two rows of blue gems.',
 'BuilderGemProfile': 'White closed profile surrounds the girdle of a tilted blue top-view gem; no pavilion crossing.',
 'BuilderGemGuides': 'Direction ticks and guide corners surround a gem.',
 'BuilderOrientToGem': 'White source object is transferred to a blue target gem.',
 'BuilderHalo': 'Small blue gems form a complete halo around a separate central stone.',
 'BuilderGemOnSurface': 'Single gem and picking cursor on a blue bowed surface.',
 'BuilderCustomGem': 'Custom blue outline and light-blue facet stations, distinct from the standard loader palette.',
 'BuilderEmerald': 'Clipped corners and stepped rectangular table distinguish emerald cut.',
 'BuilderBaguette': 'Long tapered rectangular outline and table distinguish baguette cut.',
 'BuilderTaperBagChannel': 'Multiple baguettes inside broad channel walls.',
 'BuilderTaperBagBetweenTwoCurves': 'A tapered baguette between two white curve boundaries; no dedicated local spec found.',
 'BuilderGemBetweenTwoCurves': 'Round stones touching two white curve boundaries.',
 'BuilderClusterBuilderOld': 'Blue center gem with white surrounding head/edge-gem cues; distinct from all-blue halo.',
 'BuilderGemReporter': 'Gem symbol on a white report document with tabular rows.',
 'BuilderGemMap': 'Spatial gem diagram with original mint and pink accents, not surface mapping.',
 'BuilderPaveDialog': 'Full layout with yellow corner controls distinguishes automatic interactive pave.',
 'BuilderPaveBuilder': 'Staggered gem stations on a surface retain the hexagonal packing cue.',
 'BuilderPaveAzureBuilder': 'Underside openings in a thick blue face distinguish azure holes from added prongs.',
 'BuilderPaveProngBuilder': 'White shared prong points between three separated blue gems.',
 'BuilderGemSprings': 'Displaced center gem and connected neighbors depict responsive layout editing, not a mechanical spring builder.',
 'BuilderGemFollow': 'Gold north-axis path connects consecutive gems.',
 'BuilderGemControl': 'Two gems point toward an external white control point; no invented fixed axis behavior.',
 'OthersMatchAttributes': 'Gem-to-gem transfer with white setting bases.',
 'OthersSaveStyle': 'Gold arrow enters the style document.',
 'OthersLoadStyle': 'Opposite gold arrow leaves the style document.',
 'BuilderGVDGemFlow': 'Straight and curved gold contexts with source and result gem stations.',
 'BuilderGVDGemSplop': 'Reference sphere around a gem on the target surface.',
 'BuilderPaveSphere': 'Curved sphere outline filled with distributed gem stations.',
 'BuilderGemUpdate': 'Blue gem and gold restored-link cue; detailed updater internals unverified.',
 'BuilderGemPositioner': 'Single gem with translation arrows, rotation arc and scale handle.',
}


def svg(key):
    label, meaning, spec, geometry = GEMS_ICONS[key]
    return (f'<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32">\n'
            f'<title>{html.escape(label)}</title>\n<desc>{html.escape(meaning)}. '
            f'OpenMatrix9 {STYLE}; authored vector geometry.</desc>\n{geometry}\n</svg>\n')


def write_asset(output_root, key, tooltip=None):
    relative = f'icons/gems-minimal/{key}.svg'
    target = Path(output_root) / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    raw = svg(key).encode('utf-8')
    target.write_bytes(raw)
    label, meaning, spec, _ = GEMS_ICONS[key]
    metadata = {'image': relative, 'tooltip': tooltip or key,
                'source': 'OpenMatrix9-authored-svg', 'mapping': 'authored-command-symbol', 'status': 'resolved'}
    record = {'symbol': key, 'label': label, 'meaning': meaning, 'style': STYLE,
              'mapping': metadata['mapping'], 'sha256': hashlib.sha256(raw).hexdigest(),
              'palette': [c for c in (BLUE, DEEP, LIGHT, CUSTOM, CUSTOM_LIGHT, YELLOW, BADGE, GOLD, MINT, PINK, INK, WHITE) if c in raw.decode()],
              'spec_id': f'OM9-GEM-{spec:03}' if spec else None,
              'semantic_source': 'local spec matched by command meaning' if spec else 'reference-cue-only',
              'reference_image': f'icons/rgb-plus5/ButtonIcons/{key}_1.png',
              'reference_type': 'selected Matrix90 RGB+5 derivative',
              'design_note': NOTES[key]}

    return metadata, {k:v for k,v in record.items() if k not in ('reference_image','reference_type','reference_images','palette_reference','reference_palette')}


def gems_keys(resources):
    menu = configparser.ConfigParser(interpolation=None)
    menu.optionxform = str
    menu.read(Path(resources) / 'menu/MainMenu.ini', encoding='utf-8')
    groups = [s for s in menu.values() if s.get('Name') == 'Gems']
    if len(groups) != 1:
        raise ValueError('Expected one Gems group')
    group = groups[0]
    keys = [v for k,v in group.items() if re.fullmatch(r'Icon\d+', k)]
    if len(keys) != int(group['IconCount']) or set(keys) != set(GEMS_ICONS):
        raise ValueError('Gems menu and authored catalog disagree')
    return keys


def contact_sheet(keys):
    columns, cell_w, cell_h = 8, 160, 132
    height = 64 + ((len(keys)+columns-1)//columns)*cell_h
    parts = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{columns*cell_w}" height="{height}">',
             '<rect width="100%" height="100%" fill="#333333"/>',
             '<text x="16" y="28" font-family="Arial,sans-serif" font-size="20" fill="white">OpenMatrix9 — Gems / minimal SVG</text>',
             '<text x="16" y="48" font-family="Arial,sans-serif" font-size="12" fill="#cccccc">35 icons · light blue gems · yellow curves and actions · mint/pink map · 64px and 24px samples</text>']
    for i,key in enumerate(keys):
        x,y = (i%columns)*cell_w,64+(i//columns)*cell_h
        label,_,_,geometry = GEMS_ICONS[key]
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
    report['gems_icon_style'] = {'name': STYLE, 'count': count,
        'source': 'OpenMatrix9-authored-svg', 'generator': 'tools/gems_icons.py',
        'geometry': 'authored geometric primitives; no tracing or embedded bitmaps',
        'palette_reference': 'selected RGB+5: gem blue #7CB8F5, deep #3E88C8, light #CDE7FF, custom #4987FF/#90B0FF, curve #FFE81A, badge #FFF305, action #FFE105, map #74E2CB/#CD7586, white #FFFFFF',
        'priority': 'authored Gems symbols precede legacy image sources'}
    prefix='authored Gems SVG first; '
    if not report.get('asset_policy','').startswith(prefix):
        report['asset_policy']=prefix+report.get('asset_policy','')


def install(project_root):
    root=Path(project_root)
    resources=root/'Resources'
    keys=gems_keys(resources)
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
    gallery=root/'docs/images/gems-icons-minimal.svg'
    gallery.parent.mkdir(parents=True,exist_ok=True)
    gallery.write_text(contact_sheet(keys),encoding='utf-8')
    return {'count':len(keys),'style':STYLE,'preview':str(gallery)}


if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--project-root',type=Path,default=Path(__file__).resolve().parents[1])
    args=parser.parse_args()
    print(json.dumps(install(args.project_root)))
