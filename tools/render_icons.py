"""Authored Render SVGs with light-specific whites and reference teal color roles."""
import argparse
import configparser
import hashlib
import html
import json
from pathlib import Path
import re
from curve_icons import WHITE,path,arrow,points

TEAL='#59C5C5'
DARK='#1C4B4B'
LIGHT='#CEFFFF'
BRIGHT='#42EBEB'
MOVIE='#1EBEC3'
CAPTURE='#71E6E1'
SAVE='#60D0D0'
WEB='#05E3D7'
ENGINE='#2BFFFF'
STYLE='minimal-render-v1-reviewed'


def face(d,color=TEAL,fill=DARK,opacity=.3,width=2.4):
    return path(d,color,width=width).replace('fill="none"',f'fill="{fill}" fill-opacity="{opacity}"')


def ellipse(x,y,rx,ry,color=TEAL,width=2.2):
    return f'<ellipse cx="{x}" cy="{y}" rx="{rx}" ry="{ry}" fill="none" stroke="{color}" stroke-width="{width}"/>'


def jewel():
    return face('M8 10 L12 4 H21 L25 10 L16 24 Z')+path('M8 10 H25 M12 4 L16 24 L21 4',LIGHT,width=1.6)


def ring():
    return ellipse(16,19,7,10)+face('M11 9 L14 4 H19 L22 9 L16 14 Z',TEAL,TEAL,.25,1.7)


def image_frame():
    return face('M3 5 H29 V27 H3 Z')+path('M5 24 L12 16 L17 21 L23 13 L27 20',LIGHT,width=1.7)


def page():
    return face('M5 3 H21 L27 9 V29 H5 Z',WHITE,TEAL,.08,1.6)+path('M21 3 V9 H27',LIGHT,width=1.5)


def spotlight():
    return face('M9 9 L23 25 L28 16 Z',WHITE,TEAL,.08,1.8)+ellipse(25.5,20.5,3,6,WHITE,1.7)+points((9,9),color=TEAL,radius=3)


def clock(x=24,y=24):
    return ellipse(x,y,6,6,WHITE,2)+path(f'M{x} {y-3} V{y} H{x+3}',WHITE,width=1.6)


RENDER_ICONS={
 'BuilderRenderVrayRenderStyler':('Render Styler','Prepare styled renders using scene and material controls',5,
     jewel()+path('M3 25 L16 30 L29 25',TEAL,width=2.2)
     +path('M3 10 L5 12 M28 5 L25 8 M28 13 H30',WHITE,width=1.6)),
 'RenderCreateSpotlight':('Create Spotlight','Create a cone-shaped spotlight with a base and source',9,spotlight()),
 'RenderCreatePointLight':('Create Point Light','Create a light emitting from one point in all directions',11,
     ellipse(16,16,5,5,WHITE,2.5)+path('M16 3 V7 M16 25 V29 M3 16 H7 M25 16 H29 M6 6 L9 9 M23 23 L26 26 M26 6 L23 9 M9 23 L6 26',WHITE,width=2)),
 'RenderCreateDirectionalLight':('Create Directional Light','Create a light with parallel rays and a specified direction',10,
     ellipse(23,6,3.5,3.5,TEAL,2)+arrow('M12 7 L4 20','M4 16 V20 H8')
     +arrow('M19 11 L11 24','M11 20 V24 H15')+arrow('M26 15 L18 28','M18 24 V28 H22')),
 'RenderSetSpotLighttoView':('Set Spotlight to View','Move the camera position and target to match a selected spotlight',13,
     face('M3 4 H13 V12 H3 Z',WHITE,WHITE,.08,1.7)
     +path('M13 6 L18 3 V13 L13 10',WHITE,width=1.6)
     +face('M18 8 L20 28 L29 20 Z',WHITE,WHITE,.05,1.6)
     +arrow('M4 20 H14','M11 17 L14 20 L11 23')),
 'RenderCreateRectangularLight':('Create Rectangular Light','Create a rectangular area light emitting in one direction',12,
     face('M8 8 H24 V24 H8 Z',WHITE,WHITE,.05,2.4)
     +path('M9 3 V5 M16 3 V5 M23 3 V5 M9 27 V29 M16 27 V29 M23 27 V29 M3 9 H5 M3 16 H5 M3 23 H5 M27 9 H29 M27 16 H29 M27 23 H29',WHITE,width=1.6)),
 'RenderCreateLinearLight':('Create Linear Light','Create a tube-like light between two endpoints',14,
     face('M5 23 L23 5 L27 9 L9 27 Z',WHITE,WHITE,.1,2.2)
     +path('M6 16 L3 13 M12 10 L9 7 M18 4 L15 2 M16 26 L19 29 M22 20 L25 23 M28 14 L30 17',WHITE,width=1.6)),
 'BuilderProps':('Props Library','Load a selected prop and place it in the viewport',15,
     face('M3 5 L27 2 V28 L3 30 Z')+path('M6 5 V28',LIGHT,width=1.5)
     +'<g transform="translate(9 7) scale(.6)">'+ring()+'</g>'),
 'RenderSetRenderer':('Set Renderer','Choose the rendering engine',16,
     face('M17 3 L26 8 L17 20 L10 15 Z',WHITE,TEAL,.25,1.7)
     +face('M10 15 L17 20 Q13 27 4 28 Q9 24 7 21 Z',ENGINE,ENGINE,.3,2.2)
     +path('M22 24 L26 28 L30 24',WHITE,width=2)),
 'AnalyzeSurfaceEnvironmentMap':('Environment Map Analysis','Evaluate surface smoothness from a reflected image',18,
     ellipse(16,16,12,12,BRIGHT,2.6)+path('M7 8 Q19 13 26 8 M4 16 Q16 21 28 16 M7 24 Q17 27 25 23',LIGHT,width=2)
     +path('M16 4 Q7 15 16 28',TEAL,width=1.6)),
 'RenderEffectsApplyEdgeSoftening':('Apply Edge Softening','Create a softened display mesh for selected geometry',17,
     face('M6 10 Q6 8 8 7 L18 3 Q20 2 22 4 L27 8 Q29 9 29 12 V22 Q29 24 27 25 L17 29 Q15 30 13 28 L6 24 Q4 23 4 21 V12 Q4 10 6 10 Z')
     +path('M6 10 L13 14 Q16 16 19 14 L27 10 M16 16 V27',LIGHT,width=1.8)),
 'BuilderAnimationBuilder':('Animation Builder','Render a model in motion as movie frames or numbered images',19,
     ring()+ellipse(16,20,13,5,WHITE,1.8)+arrow('M25 23 L28 20','M24 20 H28 V24')),
 'BuilderAnimationBuilderAdvanced':('Animation Builder Advanced','Create custom animations and reusable templates',20,
     '<g transform="translate(3 1) scale(.8)">'+ring()+'</g>'
     +path('M3 26 C10 28 6 16 15 20 S24 28 28 15',WHITE,width=1.8)
     +points((3,26),(15,20),(28,15),color=WHITE,radius=1.9)
     +path('M24 3 V9 M21 6 H27',LIGHT,width=1.7)),
 'MatrixMovieMaker':('Movie Maker','Assemble static rendered image sequences into movies',21,
     face('M3 4 H29 V28 H3 Z',MOVIE,MOVIE,.13,2.6)
     +path('M7 4 V28 M25 4 V28 M3 10 H7 M3 17 H7 M3 23 H7 M25 10 H29 M25 17 H29 M25 23 H29',LIGHT,width=1.5)
     +face('M12 10 L21 16 L12 22 Z',WHITE,WHITE,.4,1.7)),
 'BuilderRenderEditor':('Render Editor','Edit a render background, graphics, layers and watermark',22,
     image_frame()+face('M18 18 L26 10 L29 13 L21 21 L17 22 Z',WHITE,WHITE,.15,1.6)
     +path('M24 12 L27 15',TEAL,width=1.6)),
 'BuilderBatchRender':('Batch Render','Create multiple material variants from one model file',23,
     path('M3 24 V3 H23 M6 27 V6 H26',WHITE,width=1.6)
     +face('M9 9 H29 V29 H9 Z',TEAL,TEAL,.14,2.2)
     +'<g transform="translate(10 11) scale(.55)">'+ring()+'</g>'),
 'BuilderLayoutTools':('Layout Tools','Arrange viewport captures and graphics into printable layouts',24,
     page()+path('M9 8 H17',TEAL,width=2)
     +face('M9 13 H23 V21 H9 Z',TEAL,TEAL,.16,1.7)
     +path('M9 25 H14 M18 25 H23',TEAL,width=1.7)),
 'BuilderFourViewCapture':('Four View Capture','Capture or render all four viewports',25,
     face('M3 3 H29 V29 H3 Z',CAPTURE,CAPTURE,.13,2.4)
     +path('M16 3 V29 M3 16 H29',LIGHT,width=1.7)
     +path('M6 6 H13 V13 H6 Z M20 7 H25 V12 H20 Z M7 20 H12 V26 H7 Z',WHITE,width=1.4)
     +path('M19 22 L23 19 L27 22 L23 25 Z M19 22 V26 L23 28 L27 26 V22 M23 25 V28',WHITE,width=1.3)),
 'BuilderRenderScheduler':('Render Scheduler','Schedule renders from multiple saved design files',26,
     '<g transform="translate(0 -1) scale(.75)">'+page()+'</g>'
     +'<g transform="translate(6 1) scale(.55)">'+ring()+'</g>'+clock()),
 'gvAlphaErase':('Alpha Erase','Process alpha/transparent backgrounds in PNG image files',27,
     face('M3 3 H25 V25 H3 Z',TEAL,TEAL,.07,2)
     +face('M5 5 H12 V12 H5 Z',LIGHT,LIGHT,.18,1.4)
     +face('M12 12 H19 V19 H12 Z',LIGHT,LIGHT,.18,1.4)
     +face('M19 17 L29 23 L22 29 L12 23 Z',WHITE,WHITE,.1,1.8)
     +path('M16 20 L26 26',TEAL,width=1.6)),
 'RenderSaveRenderWindowAs':('Save Render Window As','Save the image in the render window to a file',29,
     face('M4 3 H23 L29 9 V29 H4 Z',SAVE,DARK,.5,2.6)
     +face('M9 3 H21 V12 H9 Z',LIGHT,LIGHT,.15,1.7)
     +face('M9 20 H24 V29 H9 Z',WHITE,WHITE,.08,1.7)
     +path('M18 6 V9',SAVE,width=2)),
 'WebViewer':('WebViewer','Reference cue: viewer window with a 3D object; detailed operation unverified',None,
     face('M3 4 H29 V28 H3 Z',WEB,WEB,.08,2.2)+path('M3 9 H29',LIGHT,width=1.5)
     +path('M9 17 L17 12 L24 16 V23 L16 27 L9 23 Z M9 17 L16 21 L24 16 M16 21 V27',WHITE,width=1.5)),
 'RenderEffectsApplyDisplacement':('Apply Displacement','Create a texture-driven displacement display mesh',28,
     face('M15 4 Q17 2 19 5 Q23 3 24 8 Q29 8 27 13 Q31 16 27 19 Q29 24 24 24 Q23 29 19 27 Q16 31 13 27 Q8 29 8 24 Q3 24 5 19 Q1 16 5 13 Q3 8 8 8 Q9 3 13 5 Z')
     +path('M8 8 Q11 12 8 16 Q11 21 8 24 M24 8 Q21 12 24 16 Q21 21 24 24 M6 12 Q12 15 16 11 Q20 15 26 12 M6 20 Q12 17 16 22 Q20 17 26 20',LIGHT,width=1.6)),
}

NOTES={
 'BuilderRenderVrayRenderStyler':'Independent teal jewelry/ground-plane symbol for the Render Styler parent; local subfeature specs support the scene/material workflow, but the parent mapping is inferred.',
 'RenderCreateSpotlight':'Teal source and white cone with an elliptical light base.',
 'RenderCreatePointLight':'White radial rays around one point; no teal recoloring of this originally white command.',
 'RenderCreateDirectionalLight':'Separate parallel white rays retain a teal directional source cue.',
 'RenderSetSpotLighttoView':'White camera, spotlight cone and transfer direction; camera follows the light, not light following camera.',
 'RenderCreateRectangularLight':'White area boundary and distributed outward ticks distinguish an area light.',
 'RenderCreateLinearLight':'White long tube and emission ticks, not a generic line creation command.',
 'BuilderProps':'Teal library book containing an authored jewelry prop; no copied product logo.',
 'RenderSetRenderer':'Bright-teal brush and white dropdown cue retain the reference while indicating engine selection.',
 'AnalyzeSurfaceEnvironmentMap':'Reflected white bands on a bright-teal surface sphere indicate smoothness analysis, not HDR scene selection.',
 'RenderEffectsApplyEdgeSoftening':'Rounded display-cube edges, without claiming a modeling fillet changes topology.',
 'BuilderAnimationBuilder':'Jewelry object and circular motion path; sequence rendering rather than a generic movie player.',
 'BuilderAnimationBuilderAdvanced':'Custom motion path and large frame stations distinguish advanced template/keyframe controls.',
 'MatrixMovieMaker':'Teal film strip and white play cue identify assembling image sequences.',
 'BuilderRenderEditor':'Image frame and foreground pencil identify image composition editing.',
 'BuilderBatchRender':'Stacked frames with a common model distinguish material variants from scheduled multi-file jobs.',
 'BuilderLayoutTools':'Page with graphic regions and text lines, distinct from four equal viewport panes.',
 'BuilderFourViewCapture':'Four framed quadrants show orthographic outlines and one perspective cube; avoid unrelated mathematical operation glyphs.',
 'BuilderRenderScheduler':'Saved model document and large white clock distinguish scheduled render jobs.',
 'gvAlphaErase':'Checkerboard and eraser identify alpha-background processing; no assertion that every mode creates transparency.',
 'RenderSaveRenderWindowAs':'Teal floppy with light save slot and white lower label retains the original save cue.',
 'WebViewer':'Viewer frame and white 3D object retain the viewer/geometry cue; exact behavior lacks a dedicated local spec.',
 'RenderEffectsApplyDisplacement':'Rounded bumps along a teal sphere and broad undulating surface lines show displacement, distinct from edge softening and faceted gems.',
}


def svg(key):
    label, meaning, spec, geometry = RENDER_ICONS[key]
    return (f'<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32">\n'
            f'<title>{html.escape(label)}</title>\n<desc>{html.escape(meaning)}. '
            f'OpenMatrix9 {STYLE}; authored vector geometry.</desc>\n{geometry}\n</svg>\n')


def write_asset(output_root, key, tooltip=None):
    relative = f'icons/render-minimal/{key}.svg'
    target = Path(output_root) / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    raw = svg(key).encode('utf-8')
    target.write_bytes(raw)
    label, meaning, spec, _ = RENDER_ICONS[key]
    metadata = {'image': relative, 'tooltip': tooltip or key,
                'source': 'OpenMatrix9-authored-svg', 'mapping': 'authored-command-symbol', 'status': 'resolved'}
    record = {'symbol': key, 'label': label, 'meaning': meaning, 'style': STYLE,
              'mapping': metadata['mapping'], 'sha256': hashlib.sha256(raw).hexdigest(),
              'palette': [c for c in (TEAL, DARK, LIGHT, BRIGHT, MOVIE, CAPTURE, SAVE, WEB, ENGINE, WHITE) if c in raw.decode()],
              'spec_id': f'OM9-RENDER-{spec:03}' if spec else None,
              'semantic_source': ('local Render subfeature workflow; parent mapping inferred' if key=='BuilderRenderVrayRenderStyler' else 'local spec matched by command meaning') if spec else 'reference-cue-only',
              'related_spec_ids': [f'OM9-RENDER-{n:03}' for n in range(1,9)] if key=='BuilderRenderVrayRenderStyler' else [],
              'reference_image': f'icons/rgb-plus5/ButtonIcons/{key}_1.png',
              'reference_type': 'selected Matrix90 RGB+5 derivative',
              'design_note': NOTES[key]}

    return metadata, {k:v for k,v in record.items() if k not in ('reference_image','reference_type','reference_images','palette_reference','reference_palette')}


def render_keys(resources):
    menu = configparser.ConfigParser(interpolation=None)
    menu.optionxform = str
    menu.read(Path(resources) / 'menu/MainMenu.ini', encoding='utf-8')
    groups = [s for s in menu.values() if s.get('Name') == 'Render']
    if len(groups) != 1:
        raise ValueError('Expected one Render group')
    group = groups[0]
    keys = [v for k,v in group.items() if re.fullmatch(r'Icon\d+', k)]
    if len(keys) != int(group['IconCount']) or set(keys) != set(RENDER_ICONS):
        raise ValueError('Render menu and authored catalog disagree')
    return keys


def contact_sheet(keys):
    columns, cell_w, cell_h = 8, 160, 132
    height = 64 + ((len(keys)+columns-1)//columns)*cell_h
    parts = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{columns*cell_w}" height="{height}">',
             '<rect width="100%" height="100%" fill="#333333"/>',
             '<text x="16" y="28" font-family="Arial,sans-serif" font-size="20" fill="white">OpenMatrix9 — Render / minimal SVG</text>',
             '<text x="16" y="48" font-family="Arial,sans-serif" font-size="12" fill="#cccccc">23 icons · teal render family · white lights · distinct view and output cues · 64px and 24px samples</text>']
    for i,key in enumerate(keys):
        x,y = (i%columns)*cell_w,64+(i//columns)*cell_h
        label,_,_,geometry = RENDER_ICONS[key]
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
    report['render_icon_style'] = {'name': STYLE, 'count': count,
        'source': 'OpenMatrix9-authored-svg', 'generator': 'tools/render_icons.py',
        'geometry': 'authored geometric primitives; no tracing or embedded bitmaps',
        'palette_reference': 'selected RGB+5: teal #59C5C5, dark #1C4B4B, light #CEFFFF, analysis #42EBEB, movie #1EBEC3, capture #71E6E1, save #60D0D0, web #05E3D7, engine #2BFFFF; white #FFFFFF lights/actions',
        'priority': 'authored Render symbols precede legacy image sources'}
    prefix='authored Render SVG first; '
    if not report.get('asset_policy','').startswith(prefix):
        report['asset_policy']=prefix+report.get('asset_policy','')


def install(project_root):
    root=Path(project_root)
    resources=root/'Resources'
    keys=render_keys(resources)
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
    gallery=root/'docs/images/render-icons-minimal.svg'
    gallery.parent.mkdir(parents=True,exist_ok=True)
    gallery.write_text(contact_sheet(keys),encoding='utf-8')
    return {'count':len(keys),'style':STYLE,'preview':str(gallery)}


if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--project-root',type=Path,default=Path(__file__).resolve().parents[1])
    args=parser.parse_args()
    print(json.dumps(install(args.project_root)))
