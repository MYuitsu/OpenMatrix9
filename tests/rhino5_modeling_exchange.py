# Installed Rhino5 process, owned isolated document; generated fixtures only.
import os,json,traceback,codecs,math
import Rhino,System
out=os.environ.get('OM9_RHINO5_OUTPUT',os.path.dirname(os.path.abspath(__file__)));report={'checks':[],'cases':[],'ok':False}
def check(name,value):
    report['checks'].append({'name':name,'passed':bool(value)})
    if not value:raise RuntimeError(name)
def facts(geometry):
    bounds=geometry.GetBoundingBox(True)
    row={'class':str(geometry.GetType().FullName),'bounds':[float(v) for v in (bounds.Min.X,bounds.Min.Y,bounds.Min.Z,bounds.Max.X,bounds.Max.Y,bounds.Max.Z)]}
    check('Rhino geometry valid '+row['class'],geometry.IsValid)
    if isinstance(geometry,Rhino.Geometry.Curve):row['length']=float(geometry.GetLength())
    if isinstance(geometry,Rhino.Geometry.PointCloud):row['points']=[[float(p.X),float(p.Y),float(p.Z)] for p in geometry.GetPoints()]
    if isinstance(geometry,Rhino.Geometry.Brep):
        row['faces']=int(geometry.Faces.Count)
        area=Rhino.Geometry.AreaMassProperties.Compute(geometry)
        if area:row['area']=float(area.Area);area.Dispose()
    return row
try:
    print('OM9 Rhino5 modeling exchange started (opens test documents)')
    config=json.load(codecs.open(os.path.join(out,'config.json'),'r','utf-8-sig'))
    report['rhino_version']=str(Rhino.RhinoApp.Version)
    for entry in config['files']:
        source=entry['input'];target=os.path.join(out,entry['name']+'.rhino5.3dm')
        check('Rhino Open '+entry['name'],Rhino.RhinoApp.RunScript('_-Open "'+source+'" _Enter',False))
        doc=Rhino.RhinoDoc.ActiveDoc
        rows=[facts(o.Geometry) for o in doc.Objects if not o.IsDeleted]
        check('Rhino object count '+entry['name'],len(rows)==entry['count'])
        check('Rhino SaveAs '+entry['name'],Rhino.RhinoApp.RunScript('_-SaveAs "'+target+'" _Enter',False) and os.path.isfile(target))
        model=Rhino.FileIO.File3dm.Read(target)
        check('Rhino saved file reread '+entry['name'],model is not None)
        received=[facts(o.Geometry) for o in model.Objects]
        check('Rhino saved count '+entry['name'],len(received)==len(rows))
        for a,b in zip(rows,received):
            check('Rhino saved bounds '+entry['name'],all(abs(x-y)<0.001 for x,y in zip(a['bounds'],b['bounds'])))
            if 'length' in a:check('Rhino saved length '+entry['name'],abs(a['length']-b['length'])<1e-6)
            if 'points' in a:check('Rhino saved cloud fields '+entry['name'],a['points']==b['points'])
        report['cases'].append({'name':entry['name'],'input':source,'output':target,'rows':rows});model.Dispose()
    report['ok']=True
except Exception:report['error']=traceback.format_exc()
finally:
    with open(os.path.join(out,'rhino5-results.json'),'w') as stream:json.dump(report,stream,indent=2)
    print('OM9 Rhino5 modeling exchange output: '+out)
    print('OM9 Rhino5 modeling exchange completed: '+str(report['ok']))
