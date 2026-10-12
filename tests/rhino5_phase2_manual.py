# Actual owned Rhino 5 / IronPython 2.7 document gate.
import Rhino,scriptcontext as sc,System,os,json,ntpath,time,traceback,hashlib,math
out=os.environ['OM9_RHINO_PHASE2_OUTPUT'];root='H:/FreeCAD-src/build/om9-perf-dev'
report={'checks':[],'cases':[],'ok':False,'rhino_version':str(Rhino.RhinoApp.Version),'pid':os.getpid()}
def check(name,value):
    report['checks'].append({'name':name,'passed':bool(value)})
    with open(ntpath.join(out,'progress.json'),'w') as stream:json.dump(report,stream,indent=2)
    if not value:raise RuntimeError(name)
def wait(process,seconds):
    start=time.time()
    while not process.HasExited and time.time()-start<seconds:Rhino.RhinoApp.Wait();System.Threading.Thread.Sleep(50)
    return process.HasExited
def launch(case_dir,config):
    runtime=os.environ.get('OM9_RHINO_PHASE2_RUNTIME','H:/FreeCAD-src/build/om9-perf-sdk')
    info=System.Diagnostics.ProcessStartInfo();info.FileName=ntpath.join(runtime,'bin/FreeCAD.exe');info.UseShellExecute=False;info.CreateNoWindow=True;info.WindowStyle=System.Diagnostics.ProcessWindowStyle.Hidden
    info.Arguments='-u "'+ntpath.join(case_dir,'user.cfg')+'" -s "'+ntpath.join(case_dir,'system.cfg')+'" "'+ntpath.join(root,'tests/modeling_phase2_rhino_roundtrip.FCMacro')+'"'
    info.EnvironmentVariables['OM9_SMOKE_OUTPUT']=case_dir;info.EnvironmentVariables['OM9_PHASE2_CASE']=config;info.EnvironmentVariables['FREECAD_USER_HOME']=ntpath.join(case_dir,'profile');info.EnvironmentVariables['PATH']='H:/FreeCAD-src/.pixi/envs/default/Library/bin;H:/FreeCAD-src/.pixi/envs/default;'+os.environ['PATH']
    process=System.Diagnostics.Process.Start(info);check('owned FreeCAD process finishes',wait(process,180));check('owned FreeCAD exit zero',process.ExitCode==0)
    with open(ntpath.join(case_dir,'results.json'),'r') as stream:data=json.load(stream)
    check('FreeCAD phase2 workflow '+case_dir,data.get('ok'));return data
def rational():
    curve=Rhino.Geometry.NurbsCurve(3,True,3,3)
    for i,(x,y,w) in enumerate([(0,0,1),(4,6,0.7),(10,0,1)]):curve.Points.SetPoint(i,x,y,0.,w)
    for i,value in enumerate([0,0,1,1]):curve.Knots[i]=value
    return curve

def inspect(geometry,expected,label):
    check('Rhino current native curve '+label,isinstance(geometry,Rhino.Geometry.Curve) and geometry.IsValid)
    report.setdefault('length_diagnostics',[]).append({'label':label,'actual':geometry.GetLength(),'expected':expected['length'],'segments':[g.GetLength() for g in geometry.DuplicateSegments()] if 'segments' in expected else []})
    check('Rhino current curve length '+label,abs(geometry.GetLength()-expected['length'])<=0.001)
    if 'segments' in expected:
        segments=list(geometry.DuplicateSegments());check('Rhino joined native segments '+label,len(segments)==len(expected['segments']))
        points=[geometry.PointAt(geometry.Domain.T0+(geometry.Domain.T1-geometry.Domain.T0)*i/64.) for i in range(65)]
        return {'segments':[inspect(g,w,label+'/'+str(i)) for i,(g,w) in enumerate(zip(segments,expected['segments']))],'full_samples':[[p.X,p.Y,p.Z] for p in points],'length':geometry.GetLength(),'degree':geometry.ToNurbsCurve().Degree}
    basis=geometry.ToNurbsCurve()
    report.setdefault('curve_diagnostics',[]).append({'label':label,'expected_periodic':expected['periodic'],'actual_periodic':geometry.IsPeriodic,'is_closed':geometry.IsClosed,'degree':basis.Degree,'poles':basis.Points.Count})
    check('Rhino current degree '+label,basis.Degree==expected['degree'])
    check('Rhino current rational flag '+label,basis.IsRational==any(abs(w-1)>1e-12 for w in expected['weights']))
    check('Rhino current periodic flag '+label,geometry.IsPeriodic==expected['periodic'])
    samples=[]
    for i in range(33):
        p=geometry.PointAt(geometry.Domain.T0+(geometry.Domain.T1-geometry.Domain.T0)*i/32.);samples.append([p.X,p.Y,p.Z])
    check('Rhino current curve sampled geometry '+label,all(abs(a-b)<=0.001 for p,q in zip(samples,expected['samples']) for a,b in zip(p,q)))
    return {'degree':basis.Degree,'poles':basis.Points.Count,'weights':[basis.Points[i].Weight for i in range(basis.Points.Count)],'periodic':geometry.IsPeriodic,'samples':samples}
try:
    check('actual Rhino5 target',str(Rhino.RhinoApp.Version).startswith('5.'))
    check('owned empty Rhino document',len(list(sc.doc.Objects))==0 and not sc.doc.Modified)
    periodic=Rhino.Geometry.NurbsCurve.Create(True,3,[Rhino.Geometry.Point3d(5*math.cos(i*2*math.pi/6),5*math.sin(i*2*math.pi/6),i%2) for i in range(6)])
    placed=rational();placed.Transform(Rhino.Geometry.Transform.Rotation(0.4,Rhino.Geometry.Vector3d.ZAxis,Rhino.Geometry.Point3d.Origin));placed.Transform(Rhino.Geometry.Transform.Translation(15,20,2))
    for name,curve in [('rational',rational()),('periodic',periodic),('placed',placed)]:
        check('valid Rhino source '+name,curve is not None and curve.IsValid)
        case_dir=ntpath.join(out,name);os.makedirs(case_dir);sc.doc.Modified=False
        for obj in list(sc.doc.Objects):sc.doc.Objects.Delete(obj.Id,True)
        sc.doc.ModelUnitSystem=Rhino.UnitSystem.Millimeters
        identifier=sc.doc.Objects.AddCurve(curve);sc.doc.Objects.Select(identifier)
        source=ntpath.join(case_dir,'rhino-selected-v5.3dm')
        check('actual Rhino Export Selected '+name,Rhino.RhinoApp.RunScript('_-Export "'+source+'" _Enter',False) and os.path.isfile(source))
        config=ntpath.join(case_dir,'case.json')
        with open(config,'w') as stream:json.dump({'input':source,'source_periodic':curve.IsPeriodic},stream)
        host=launch(case_dir,config);sc.doc.Modified=False
        check('actual Rhino Open current curves '+name,Rhino.RhinoApp.RunScript('_-Open "'+host['output']+'" _Enter',False))
        rows={}
        for obj in sc.doc.Objects:
            label=obj.Attributes.Name;geometry=obj.Geometry
            rows[label]=inspect(geometry,host['expected'][label],label)
        check('Rhino edited and new exact selection '+name,set(rows)==set(host['expected']))
        saved=ntpath.join(case_dir,'phase2.rhino5.3dm');check('actual Rhino SaveAs '+name,Rhino.RhinoApp.RunScript('_-SaveAs "'+saved+'" _Enter',False) and os.path.isfile(saved))
        report['cases'].append({'name':name,'input':source,'host_report':ntpath.join(case_dir,'results.json'),'output':host['output'],'saved':saved,'rows':rows})
    report['ok']=True
except:report['error']=traceback.format_exc()
with open(ntpath.join(out,'rhino5-results.json'),'w') as stream:json.dump(report,stream,indent=2)
Rhino.RhinoApp.WriteLine('Phase2 Rhino gate: '+str(report['ok'])+'; '+ntpath.join(out,'rhino5-results.json'))
if os.environ.get('OM9_RHINO_PHASE2_OUTPUT'):System.Environment.Exit(0 if report['ok'] else 1)
