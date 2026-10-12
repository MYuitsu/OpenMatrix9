# Rhino5 IronPython host glue only; modeling logic runs in the Rust/native runtime.
import Rhino,System,os,ntpath,json,codecs,hashlib,time,traceback
import scriptcontext as sc
out=os.environ['OM9_PHASE3_RHINO_OUTPUT'];report={'ok':False,'checks':[],'cases':[],'pid':System.Diagnostics.Process.GetCurrentProcess().Id,'rhino_version':str(Rhino.RhinoApp.Version)}
def digest(path):
    with open(path,'rb') as f:return hashlib.sha256(f.read()).hexdigest()
def check(name,value):
    report['checks'].append({'name':name,'passed':bool(value)})
    if not value:raise RuntimeError(name)
def facts(geometry):
    check('Rhino actual geometry valid',geometry.IsValid)
    box=geometry.GetBoundingBox(True);check('Rhino finite accurate bounds',box.IsValid)
    check('Rhino native BRep preserved',isinstance(geometry,Rhino.Geometry.Brep))
    area=Rhino.Geometry.AreaMassProperties.Compute(geometry);check('Rhino area available',area is not None)
    row={'area':area.Area,'bounds':[box.Min.X,box.Min.Y,box.Min.Z,box.Max.X,box.Max.Y,box.Max.Z],'faces':geometry.Faces.Count,'solid':geometry.IsSolid};area.Dispose()
    if geometry.IsSolid:
        volume=Rhino.Geometry.VolumeMassProperties.Compute(geometry);check('Rhino volume available',volume is not None);row['volume']=volume.Volume;volume.Dispose()
    return row
def compare(row,before,label):
    check(label+' solid classification',row['solid']==bool(before['solids']))
    check(label+' current bounds',max(abs(a-b) for a,b in zip(row['bounds'],before['bounds']))<=.001)
    check(label+' area',abs(row['area']-before['area'])<=max(.001,.0001*abs(before['area'])))
    if before['solids']:check(label+' material volume',abs(row['volume']-before['volume'])<=max(.001,.0001*abs(before['volume'])))
def ordered_facts(objects,expected):
    objects=list(objects); names=[unicode(o.Attributes.Name) for o in objects]
    wanted=[row['object_name'] for row in expected]
    check('Rhino exact unique object names',len(set(names))==len(names) and len(set(wanted))==len(wanted) and set(names)==set(wanted))
    lookup=dict((unicode(o.Attributes.Name),o) for o in objects)
    return [dict(facts(lookup[name].Geometry),object_name=name,object_id=str(lookup[name].Attributes.ObjectId)) for name in wanted]
def command(action,path):
    start=time.time();result=Rhino.RhinoApp.RunScript(u'_-'+action+u' "'+ntpath.normpath(path)+u'" _Enter',False)
    return result,time.time()-start
try:
    check('actual Rhino5 version',report['rhino_version'].startswith('5.'))
    with codecs.open(os.environ['OM9_PHASE3_CASES'],'r','utf-8-sig') as f:cases=json.load(f)['cases']
    for entry in cases:
        source=entry['input'];check('fixture hash '+entry['name'],digest(source)==entry['sha256'])
        sc.doc.Modified=False;opened,seconds=command('Open',source);check('Rhino Open '+entry['name'],opened)
        objects=list(sc.doc.Objects.GetObjectList(Rhino.DocObjects.ObjectType.AnyObject));check('Rhino current selected object set',len(objects)==len(entry['expected']))
        actual=ordered_facts(objects,entry['expected'])
        for i,(row,before) in enumerate(zip(actual,entry['expected'])):compare(row,before,entry['name']+' open '+str(i))
        saved=ntpath.join(out,entry['name']+'.rhino5.3dm');ok,save_seconds=command('SaveAs',saved);check('Rhino SaveAs '+entry['name'],ok and os.path.isfile(saved))
        for obj in objects:check('Rhino object selectable',sc.doc.Objects.Select(obj.Id,True))
        selected=ntpath.join(out,entry['name']+'.rhino5-selected.3dm');ok,export_seconds=command('Export',selected);check('Rhino Export Selected '+entry['name'],ok and os.path.isfile(selected))
        outputs=[]
        for path in [saved,selected]:
            model=Rhino.FileIO.File3dm.Read(path);check('Rhino saved file readable',model is not None)
            rows=ordered_facts(model.Objects,entry['expected']);check('Rhino saved object set',len(rows)==len(entry['expected']))
            check('Rhino saved object identities preserved',[(r['object_name'],r['object_id']) for r in rows]==[(r['object_name'],r['object_id']) for r in actual])
            for i,(row,before) in enumerate(zip(rows,entry['expected'])):compare(row,before,entry['name']+' saved '+str(i))
            outputs.append({'path':path,'sha256':digest(path),'actual':rows});model.Dispose()
        report['cases'].append(dict(entry,outputs=outputs,timings={'Open':seconds,'SaveAs':save_seconds,'ExportSelected':export_seconds}))
    report['ok']=True
except:report['error']=traceback.format_exc()
finally:
    report['termination_mode']='owned verifier exits after durable report'
    with codecs.open(ntpath.join(out,'rhino5-results.json'),'w','utf-8') as f:json.dump(report,f,indent=2)
    sc.doc.Modified=False
    # Only the isolated launcher process, after every output/report is closed.
    # RhinoApp.Exit may leave its plug-in shutdown running after this script.
    System.Environment.Exit(0)
