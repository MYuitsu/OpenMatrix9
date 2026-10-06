"""Export only independently authored SVG catalogs; never read legacy artwork."""
import argparse
import configparser
import importlib
import json
from pathlib import Path
import re

MODULES=('curve','solid','transform','surface','emboss','builder','subd','tools',
         'gems','settings','cutters','render','core')
CATALOGS=[]
for name in MODULES:
    module=importlib.import_module(name+'_icons')
    CATALOGS.append((name,getattr(module,name.upper()+'_ICONS'),module))


def public_record(record):
    """Retain design explanations; exclude private research asset locations."""
    return {k:v for k,v in record.items() if k not in (
        'reference_image','reference_type','palette_reference','reference_images',
        'reference_palette','source_path','source_manual','source_page','source_document')}


def read_shifted_icons(*args,**kwargs):
    raise ValueError('Legacy artwork import is disabled in the public exporter')


def export_assets(ref_root,output_root,named_root=None,named_bindings=None,
                  button_icons=None,slider_icons=None,allow_original=False,shifted_root=None):
    # Legacy flags remain accepted for callers, but cannot enable bitmap import.
    if any(x is not None for x in (named_root,named_bindings,button_icons,slider_icons,shifted_root)):
        raise ValueError('External artwork sources are not supported')
    source=Path(ref_root);output=Path(output_root)
    menu=configparser.ConfigParser(interpolation=None);menu.optionxform=str
    menu.read(source/'MainMenu.ini',encoding='utf-8')
    names=list(dict.fromkeys(v.strip() for s in menu.values() for k,v in s.items()
        if re.fullmatch(r'Icon\d+',k)))
    # Core includes every auxiliary/sidebar key. SolidPtOn is a separate alias.
    auxiliary=list(CATALOGS[-1][1])+['SolidPtOn']
    names=list(dict.fromkeys(names+auxiliary))
    existing=configparser.ConfigParser(interpolation=None)
    existing.read(output/'menu/icons.ini',encoding='utf-8')
    names=list(dict.fromkeys(names+existing.sections()))
    owners={}
    for _,catalog,module in CATALOGS:
        for key in catalog:owners.setdefault(key,module)
    unknown=set(names)-set(owners)
    if unknown:raise ValueError('Missing authored command symbols: '+', '.join(sorted(unknown)))
    bindings=configparser.ConfigParser(interpolation=None)
    report={'schema_version':2,'asset_policy':'Authored SVG only; external bitmap/archive import disabled',
        'resolved':[],'missing':{},'shifted_icons':{},'authored_symbols':{}}
    for key in names:
        tooltip=existing[key].get('tooltip') if existing.has_section(key) else None
        metadata,record=owners[key].write_asset(output,key,tooltip)
        if existing.has_section(key) and existing[key].get('feature_id'):
            metadata['feature_id']=existing[key]['feature_id']
        bindings[key]=metadata;report['authored_symbols'][key]=public_record(record)
        report['resolved'].append(key)
    for _,catalog,module in CATALOGS:
        count=sum(k in catalog for k in names)
        if count:module.describe_style(report,count)
    (output/'menu').mkdir(parents=True,exist_ok=True)
    with (output/'menu/icons.ini').open('w',encoding='utf-8') as stream:bindings.write(stream)
    icon_ini=output/'menu/icons.ini'
    icon_ini.write_text(icon_ini.read_text(encoding='utf-8').rstrip()+'\n',encoding='utf-8')
    (output/'menu/source-manifest.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    return report


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--menu-root',type=Path)
    parser.add_argument('--output-root',type=Path)
    parser.add_argument('--project-root',type=Path,default=Path(__file__).resolve().parents[1])
    args=parser.parse_args()
    output=args.output_root or args.project_root/'Resources'
    result=export_assets(args.menu_root or args.project_root/'Resources/menu',output)
    print(json.dumps({'authored_svg_bindings':len(result['authored_symbols']),'missing':len(result['missing'])}))
