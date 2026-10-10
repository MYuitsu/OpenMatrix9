"""Offline, read-only reference extraction. Macro scripts are metadata, never executed."""
import argparse
import base64
import bz2
import configparser
import hashlib
import json
from pathlib import Path
import re
import struct
import xml.etree.ElementTree as ET
import zlib
from modern_icons import symbol_for_key, svg

SCRIPT_BINDINGS = {
    'FileNew': '! _New', 'FileOpen': '! _Open', 'FileSave': '! _Save',
    'FileSaveAs': '! _SaveAs', 'FileImport': '! _Import', 'FilePrint': '! _Print',
    'ViewZoomZoomExtents': '_Zoom _Extents', 'TopIconDuplicate': '! _Copy',
    'TopIconMove': '! _Move', 'TopIconMirror': '! _Mirror', 'TopIconRotate': '! _Rotate',
    'TopIconExplode': '! _Explode', 'TopIconJoin': '! _Join',
    'TopIconSplit': '! _Split', 'TopIconTrim': '! _Trim',
}

# Host-only commands have short names; the archive has their full Rhino keys.
BUTTON_ALIASES = {
    'SolidPtOn':'TransformSolidPtOn',
    'Undo':'EditUndo', 'Redo':'EditRedo',
    'ViewFront':'ViewSetCameraCPlaneFront', 'ViewTop':'ViewSetCameraCPlaneTop',
    'ViewRight':'ViewSetCameraCPlaneRight', 'FitAll':'ViewZoomZoomExtentsAll',
    'SelectAll':'EditSelectObjectsAllObjects','SelectNone':'EditSelectObjectsNone','Delete':'EditDelete',
    'ViewLeft':'ViewSetCameraCPlaneLeft','ViewRear':'ViewSetCameraCPlaneBack','ViewBottom':'ViewSetCameraCPlaneBottom',
}
SIDEBAR_BUTTONS = [
    'ToolsObjectSnap'+name for name in
    ('End','Near','Point','Midpoint','Center','Intersection','PerpendicularTo','TangentTo','Quadrant','Knot')
] + ['InfoSettingsDisplayProperties','RhinoOptions','GVObjectInfo','FileNotes','ViewGridOptions']
SIDEBAR_BUTTONS += ['ObjectProperties','AllObjectInfo','CommandHistory','ProjectNotes','SuperSelect',
    'InfoSettingsViewportTabsToggle','InfoSettingsBoxEdit','InfoSettingsLibraries','InfoSettingsSelectionFilter',
    'InfoSettingsDesignReport','InfoSettingsGumballAlignment','InfoSettingsRelocateGumball','InfoSettingsGumballON',
    'InfoSettingsSmartTargetsGumballON','RhinoSmartTrackON','RhinoHistoryON','GVHistoryUpdateON','GVHistoryRecordON',
    'InfoSettingsGVClearHistory','GridON','PreviewCutterON','PreviewShadeON','ShadeSelectedOnlyON','GVGemView_1','GVSurfaceView_1',
    'GridSnapON','OrthoSnapON','PlanarSnapON','ProjectSnapON','SnapBetween','SnapOnSurface','SnapOnPolysurface',
    'LayerArrow','LayerLock','LayerVisibility','LayerHide','LayerShow',
    'ProjectAdd','ProjectOut','ProjectIn','ProjectSave','ProjectDelete','ProjectManager','ViewIsometric']
DISPLAY_BUTTONS=['Dial_Wireframe','Dial_Shaded','Dial_Working Shade','Dial_Working Render','Dial_Ghosted','Dial_Tech Shade']
SHIFTED_ALIASES={'SnapBetween':'ToolsObjectSnapBetween','SnapOnSurface':'ToolsObjectSnapOnObjectOnSurface',
                 'SnapOnPolysurface':'ToolsObjectSnapOnObjectOnPolysurface'}

def read_button_icons(path):
    """Read the installed VB6 UBound/bzip2/StdPicture archive without executing code."""
    raw=Path(path).read_bytes(); pos=0; result={}
    def uint():
        nonlocal pos
        if pos+4>len(raw): raise ValueError('Truncated button archive')
        value=struct.unpack_from('<I',raw,pos)[0];pos+=4;return value
    count=uint()+1
    if count>20000: raise ValueError('Unreasonable button record count')
    for _ in range(count):
        record_offset=pos; length=uint()
        if not 0<length<=256 or pos+length>len(raw): raise ValueError('Invalid button name length')
        try: name=raw[pos:pos+length].decode('ascii')
        except UnicodeDecodeError as exc: raise ValueError('Non-ASCII button key') from exc
        pos+=length
        if not re.fullmatch(r'[A-Za-z0-9_ -]+',name) or name!=name.strip() or name in result: raise ValueError('Unsafe or duplicate button key')
        size=uint()+1
        if size>2000000 or pos+size>len(raw): raise ValueError('Invalid compressed button length')
        try:
            decoder=bz2.BZ2Decompressor()
            picture=decoder.decompress(raw[pos:pos+size],max_length=2000000)
            if not decoder.eof or decoder.unused_data: raise ValueError('Invalid compressed button stream')
        except (OSError,EOFError) as exc: raise ValueError('Invalid compressed button stream') from exc
        pos+=size
        # Persisted StdPicture: 42-byte OLE header, uint32 BMP size, then BMP.
        if len(picture)<100: raise ValueError('Truncated persisted picture')
        bmp=picture[46:]
        if struct.unpack_from('<I',picture,42)[0]!=len(bmp) or bmp[:2]!=b'BM': raise ValueError('Invalid persisted BMP')
        file_size=struct.unpack_from('<I',bmp,2)[0]
        pixel_offset=struct.unpack_from('<I',bmp,10)[0]
        header=struct.unpack_from('<I',bmp,14)[0]
        width,height,planes,depth,compression=struct.unpack_from('<iiHHI',bmp,18)
        stride=((width*depth+31)//32)*4
        palette_size=1024 if depth==8 else 0
        if (file_size!=len(bmp) or header!=40 or not 0<width<=256 or not 0<abs(height)<=256
            or planes!=1 or depth not in (8,24) or compression!=0 or pixel_offset<54+palette_size
            or pixel_offset+stride*abs(height)>len(bmp)): raise ValueError('Invalid button BMP dimensions/layout')
        result[name]={'bytes':bmp,'size':[width,abs(height)],'record_offset':record_offset}
    if raw[pos:]!=b'\0'*4: raise ValueError('Invalid button archive terminator')
    return result

def png_size(data):
    if not data.startswith(b'\x89PNG\r\n\x1a\n'): raise ValueError('Atlas is not PNG')
    offset=8; size=None; ended=False
    while offset+12<=len(data):
        length=struct.unpack('>I',data[offset:offset+4])[0]
        end=offset+12+length
        if end>len(data): raise ValueError('Truncated PNG chunk')
        kind=data[offset+4:offset+8]; payload=data[offset+8:offset+8+length]
        crc=struct.unpack('>I',data[offset+8+length:end])[0]
        if zlib.crc32(kind+payload)!=crc: raise ValueError('Invalid PNG CRC')
        if kind==b'IHDR':
            if length!=13: raise ValueError('Invalid PNG header')
            size=struct.unpack('>II',payload[:8])
        if kind==b'IEND': ended=True; break
        offset=end
    if not ended or not size or min(size)<=0: raise ValueError('Invalid PNG structure')
    return size

def read_rui(path):
    root=ET.parse(path).getroot()
    atlas=root.find('bitmaps/normal_bitmap')
    if atlas is None: raise ValueError('Missing normal_bitmap')
    try:
        raw=base64.b64decode(''.join(atlas.findtext('bitmap','').split()),validate=True)
        width,height=png_size(raw)
        iw,ih=int(atlas.attrib['item_width']),int(atlas.attrib['item_height'])
        if iw<=0 or ih<=0 or width%iw or height%ih: raise ValueError('Invalid atlas cell dimensions')
    except (KeyError,ValueError) as exc: raise ValueError(f'Invalid bitmap: {exc}') from exc
    items={}
    for item in atlas.findall('bitmap_item'):
        idx=int(item.attrib['index']); columns=width//iw
        rect=[idx%columns*iw,idx//columns*ih,iw,ih]
        if idx<0 or rect[1]+ih>height: raise ValueError(f'Bitmap index outside atlas: {idx}')
        if item.attrib['guid'] in items: raise ValueError('Duplicate bitmap GUID')
        items[item.attrib['guid']]=rect
    macros={}; scripts={}
    for macro in root.findall('macros/macro_item'):
        bitmap=macro.attrib.get('bitmap_id')
        entry={
            'guid':macro.attrib.get('guid',''), 'rect':items.get(bitmap),
            'tooltip':macro.findtext('tooltip/locale_1033','') or macro.findtext('text/locale_1033',''),
            'script':macro.findtext('script','').strip(),
        }
        name=macro.findtext('text/locale_1033','').strip()
        if name:
            if name in macros: macros[name]=None
            else: macros[name]=entry
        script=' '.join(entry['script'].split())
        if script in scripts: scripts[script]=None
        else: scripts[script]=entry
    return {'macros':macros,'scripts':scripts,'atlas':raw,'size':[width,height]}

def read_form(path):
    controls=[]; references=[]; stack=[]
    for line in path.read_text(encoding='utf-8-sig',errors='replace').splitlines():
        if line.strip().startswith('Attribute VB_Name'): break
        match=re.match(r'\s*Begin\s+(\S+)\s+(\S+)',line)
        if match:
            control={'type':match[1],'name':match[2],'parent':stack[-1] if stack else None}
            controls.append(control); stack.append(match[2])
        elif line.strip()=='End' and stack: stack.pop()
        match=re.search(r'"([^"\n]+\.(?:frx|ctx))":([0-9A-Fa-f]+)',line)
        if match: references.append({'control':stack[-1] if stack else None,'file':match[1],'offset':int(match[2],16)})
    return {'controls':controls,'references':references}

def read_named_icons(root, bindings):
    if root is None and bindings is None: return {}
    if root is None or bindings is None: raise ValueError('Provide both named root and bindings')
    root=Path(root).resolve(); result={}
    for key,item in json.loads(Path(bindings).read_text(encoding='utf-8')).items():
        relative=Path(item['file']); file=(root/relative).resolve()
        if relative.is_absolute() or '..' in relative.parts or not file.is_relative_to(root): raise ValueError('Named icon path escapes source root')
        raw=file.read_bytes(); size=png_size(raw)
        manifest=file.parent/'manifest.json'
        entries=json.loads(manifest.read_text(encoding='utf-8'))
        matches=[e for e in entries if e['output_file']==file.name]
        if len(matches)!=1: raise ValueError(f'No unique named crop manifest entry: {relative}')
        result[key]={**item,'bytes':raw,'size':size,'source_used':matches[0]['source_used']}
    return result

def read_shifted_icons(root):
    if root is None:return {}
    root=Path(root).resolve();manifest=json.loads((root/'manifest.json').read_text(encoding='utf-8'))
    transform=manifest.get('rgb_transform',{})
    if transform.get('delta')!=5 or transform.get('clamp')!=[0,255] or transform.get('alpha_unchanged') is not True:
        raise ValueError('Expected verified RGB+5 reference images with preserved alpha')
    result={}
    for item in manifest['records']:
        archive=item['archive'];key=item['key']
        if archive not in ('ButtonIcons.bin','SliderIcons.bin'):continue
        if not re.fullmatch(r'[A-Za-z0-9_ -]+',key) or key!=key.strip() or (archive,key) in result:
            raise ValueError('Unsafe or duplicate RGB+5 image key')
        relative=Path(item['png']);target=(root/relative).resolve()
        if relative.is_absolute() or not target.is_relative_to(root) or relative.suffix.lower()!='.png':
            raise ValueError('RGB+5 image path escapes source folder')
        raw=target.read_bytes();size=png_size(raw)
        if list(size)!=item['size'] or hashlib.sha256(raw).hexdigest()!=item['png_sha256']:
            raise ValueError('RGB+5 image hash/dimensions mismatch: '+key)
        if not re.fullmatch(r'[0-9a-f]{64}',item['original_png_sha256']):raise ValueError('Missing original image fingerprint')
        result[(archive,key)]={**item,'bytes':raw}
    return result

def export_assets(ref_root,output_root,named_root=None,named_bindings=None,button_icons=None,slider_icons=None,allow_original=False,shifted_root=None):
    ref_root=Path(ref_root); output_root=Path(output_root)
    ini=ref_root/'MainMenu.ini'; rui=ref_root/'Matrix.rui'
    text=ini.read_text(encoding='utf-8-sig')
    source=configparser.ConfigParser(interpolation=None); source.optionxform=str; source.read_string(text)
    names=list(dict.fromkeys(value.strip() for section in source for key,value in source[section].items() if re.fullmatch(r'Icon\d+',key)))
    data=read_rui(rui)
    named=read_named_icons(named_root,named_bindings)
    shifted=read_shifted_icons(shifted_root)
    button_path=Path(button_icons) if button_icons else ref_root/'ButtonIcons.bin'
    original=read_button_icons(button_path) if button_path.exists() else {}
    if button_icons and not original: raise ValueError('Button icon archive missing or empty')
    slider_path=Path(slider_icons) if slider_icons else ref_root/'SliderIcons.bin'
    sliders=read_button_icons(slider_path) if slider_path.exists() else {}
    if slider_icons and not sliders: raise ValueError('Slider icon archive missing or empty')
    (output_root/'menu').mkdir(parents=True,exist_ok=True)
    (output_root/'icons').mkdir(parents=True,exist_ok=True)
    # Retire only legacy files previously declared by this exporter. Keep any
    # unrelated user files, and reject paths/symlinks outside the output tree.
    previous=configparser.ConfigParser(interpolation=None)
    previous.read(output_root/'menu/icons.ini',encoding='utf-8')
    legacy_paths={s.get('image',s.get('atlas','')) for s in previous.values()}
    (output_root/'menu/MainMenu.ini').write_text('\n'.join(line.rstrip() for line in text.splitlines())+'\n',encoding='utf-8')
    result={'source_sha256':{p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in (ini,rui)},'resolved':[],'missing':{},'named_crops':{},'mapping_note':'Reviewed user-authored named crops take precedence. Newly authored SVGs supply command symbols or explicitly marked category fallbacks. Original artwork is excluded by default; archive/RUI matching is available only with --allow-original. Source files are preserved.'}
    if named_bindings: result['source_sha256']['named-icon-bindings.json']=hashlib.sha256(Path(named_bindings).read_bytes()).hexdigest()
    result['original_buttons']={};result['auxiliary_resolved']=[];result['auxiliary_missing']={}
    result['asset_policy']='modern-first; original artwork excluded' if not allow_original else 'modern-first; explicit original fallback allowed'
    result['shifted_icons']={}
    if shifted_root:
        result['asset_policy']='RGB+5-derived-first; named redraw and authored symbol fallbacks'
        result['mapping_note']='User selected RGB+5 reference derivatives. Exact ButtonIcons/SliderIcons names or explicitly reviewed aliases take precedence over named redraws. Colors are already transformed; PNG bytes are copied without another shift. Unsupported controls remain disabled.'
        result['source_sha256']['rgb-shift-manifest.json']=hashlib.sha256((Path(shifted_root)/'manifest.json').read_bytes()).hexdigest()
    result['authored_symbols']={};result['excluded_named']={}
    if not allow_original:
        for key,item in list(named.items()):
            if 'original' in item['source_used'].casefold():
                result['excluded_named'][key]='Manifest identifies original/fallback artwork';del named[key]
    for archive_path,records in ((button_path,original),(slider_path,sliders)):
        if records:result['source_sha256'][archive_path.name]=hashlib.sha256(archive_path.read_bytes()).hexdigest()
    result['button_archive_records']=len(original);result['slider_archive_records']=len(sliders)
    dest=configparser.ConfigParser(interpolation=None); dest.optionxform=str
    auxiliary=[name for name in dict.fromkeys(list(BUTTON_ALIASES)+SIDEBAR_BUTTONS+DISPLAY_BUTTONS+list(named)) if name not in names]
    for name in names+auxiliary:
        resolution_key='resolved' if name in names else 'auxiliary_resolved'
        shifted_alias=SHIFTED_ALIASES.get(name,BUTTON_ALIASES.get(name,name))
        shifted_record=shifted_alias+'_1';shifted_archive='ButtonIcons.bin'
        shifted_item=shifted.get((shifted_archive,shifted_record))
        if shifted_item is None:
            shifted_archive='SliderIcons.bin';shifted_record=name
            shifted_item=shifted.get((shifted_archive,shifted_record))
        if shifted_item is not None:
            relative='icons/rgb-plus5/'+shifted_archive.removesuffix('.bin')+'/'+shifted_record+'.png'
            target=output_root/relative;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(shifted_item['bytes'])
            mapping='reviewed-command-alias' if shifted_alias!=name else ('exact-slider-key' if shifted_archive=='SliderIcons.bin' else 'exact-button-key')
            dest[name]={'image':relative,'tooltip':name,'source':'Matrix90-RGB+5','source_archive':shifted_archive,'record':shifted_record,'mapping':mapping,'rgb_delta':'5','status':'resolved'}
            result['shifted_icons'][name]={'archive':shifted_archive,'record':shifted_record,'mapping':mapping,'size':shifted_item['size'],
                'sha256':shifted_item['png_sha256'],'original_png_sha256':shifted_item['original_png_sha256'],'rgb_delta':5}
            result[resolution_key].append(name)
            continue
        record=BUTTON_ALIASES.get(name,name)+'_1'
        item=original.get(record);archive_source='ButtonIcons.bin';folder=''
        if item is None and name in sliders:
            record=name;item=sliders[name];archive_source='SliderIcons.bin';folder='slider/'
        if item is not None and allow_original and name not in named:
            relative='icons/original/'+folder+record+'.bmp'
            target=output_root/relative;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(item['bytes'])
            mapping='reviewed-command-alias' if name in BUTTON_ALIASES else ('exact-slider-key' if folder else 'exact-button-key')
            dest[name]={'image':relative,'tooltip':name,'source':archive_source,'record':record,'mapping':mapping,'status':'resolved'}
            result['original_buttons'][name]={'source':archive_source,'record':record,'mapping':mapping,'record_offset':item['record_offset'],'size':item['size'],'sha256':hashlib.sha256(item['bytes']).hexdigest()}
            result[resolution_key].append(name)
            continue
        if name in named:
            item=named[name]; relative='icons/named/'+Path(item['file']).as_posix()
            target=output_root/relative;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(item['bytes'])
            dest[name]={'image':relative,'tooltip':item['feature_name'],'feature_id':item['feature_id'],'mapping':'reviewed-named-feature','source':item['source_used'],'status':'resolved'}
            result[resolution_key].append(name)
            result['named_crops'][name]={k:v for k,v in item.items() if k!='bytes'}
            result['named_crops'][name]['sha256']=hashlib.sha256(item['bytes']).hexdigest()
            continue
        entry=data['macros'].get(name); mapping='exact-name'
        if not entry and name in SCRIPT_BINDINGS:
            entry=data['scripts'].get(SCRIPT_BINDINGS[name]); mapping='explicit-script-binding'
        if allow_original and entry and entry['rect']:
            (output_root/'icons/matrix9-24.png').write_bytes(data['atlas'])
            x,y,w,h=entry['rect']
            dest[name]={'atlas':'icons/matrix9-24.png','x':str(x),'y':str(y),'width':str(w),'height':str(h),'tooltip':entry['tooltip'],'source':'Matrix.rui','guid':entry['guid'],'mapping':mapping,'status':'resolved'}
            result[resolution_key].append(name)
        else:
            symbol,exact=symbol_for_key(name);relative='icons/modern/'+symbol+'.svg'
            target=output_root/relative;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(svg(symbol).encode('utf-8'))
            mapping='authored-command-symbol' if exact else 'authored-category-symbol'
            dest[name]={'status':'resolved' if exact else 'fallback','image':relative,'tooltip':name,'source':'OpenMatrix9-authored-svg','mapping':mapping}
            result['authored_symbols'][name]={'symbol':symbol,'mapping':mapping,'sha256':hashlib.sha256(target.read_bytes()).hexdigest()}
            if exact:result[resolution_key].append(name)
            else:result['missing' if name in names else 'auxiliary_missing'][name]='No matching modern crop; authored category symbol supplied'
    if not allow_original:
        current_paths={s.get('image',s.get('atlas','')) for s in dest.values()}
        for relative in legacy_paths-current_paths:
            if not relative:continue
            target=(output_root/relative).resolve()
            if not target.is_relative_to(output_root.resolve()):raise ValueError('Managed icon path escapes output root')
            if target.is_file():target.unlink()
    with (output_root/'menu/icons.ini').open('w',encoding='utf-8') as stream: dest.write(stream)
    icons_path=output_root/'menu/icons.ini'
    icons_path.write_text(icons_path.read_text(encoding='utf-8').rstrip()+'\n',encoding='utf-8')
    (output_root/'menu/source-manifest.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    return result

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--ref-root',type=Path,required=True)
    parser.add_argument('--output-root',type=Path,required=True)
    parser.add_argument('--named-root',type=Path)
    parser.add_argument('--named-bindings',type=Path)
    parser.add_argument('--button-icons',type=Path,help='Installed Matrix90/UserInterface/ButtonIcons.bin; optional original icon source')
    parser.add_argument('--slider-icons',type=Path,help='Installed Matrix90/UserInterface/SliderIcons.bin; optional display icon source')
    parser.add_argument('--allow-original',action='store_true',help='Explicitly permit legacy artwork; modern named crops still take precedence')
    parser.add_argument('--shifted-root',type=Path,help='User-selected reference folder with verified RGB+5 PNG manifest; highest priority')
    args=parser.parse_args()
    result=export_assets(args.ref_root,args.output_root,args.named_root,args.named_bindings,args.button_icons,args.slider_icons,args.allow_original,args.shifted_root)
    print(json.dumps({'resolved':len(result['resolved']),'category_fallback':len(result['missing']),'named':len(result['named_crops']),'shifted':len(result['shifted_icons']),'original':len(result['original_buttons']),'auxiliary':len(result['auxiliary_resolved'])}))
