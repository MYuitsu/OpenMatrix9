# Runs inside the installed Rhino5 IronPython host; never touches active geometry.
import os,json,hashlib,traceback,struct,codecs
import Rhino,System
out=os.environ['OM9_RHINO5_OUTPUT']
result=dict(ok=False,checks=[],process_id=System.Diagnostics.Process.GetCurrentProcess().Id)
def digest(path):
    with open(path,'rb') as stream:return hashlib.sha256(stream.read()).hexdigest()
def vector(value):return [value.X,value.Y,value.Z]
def facts(model):
    rows=[]
    for obj in model.Objects:
        geometry=obj.Geometry
        row=dict(uuid=str(obj.Attributes.ObjectId),name=obj.Attributes.Name,type=geometry.GetType().FullName)
        if isinstance(geometry,Rhino.Geometry.PointCloud):
            row.update(points=[vector(p) for p in geometry.GetPoints()],normals=[vector(n) for n in geometry.GetNormals()],colors=[[c.R,c.G,c.B,c.A] for c in geometry.GetColors()])
        rows.append(row)
    return rows
try:
    config=json.load(codecs.open(os.path.join(out,'config.json'),'r','utf-8-sig'))
    result['rhino_version']=str(Rhino.RhinoApp.Version)
    result['rhino_common_version']=str(Rhino.Geometry.PointCloud().GetType().Assembly.GetName().Version)
    result['has_get_point_values']=any(m.Name=='GetPointValues' for m in Rhino.Geometry.PointCloud().GetType().GetMethods())
    result['files']=[]
    for entry in config['files']:
        source=entry['input'];target=os.path.join(out,entry['name']+'.rhino5.3dm');before=digest(source)
        model=Rhino.FileIO.File3dm.Read(source)
        if model is None:raise RuntimeError('Rhino5 File3dm.Read failed: '+source)
        before_rows=facts(model)
        if not model.Write(target,5):raise RuntimeError('Rhino5 File3dm.Write failed: '+target)
        reread=Rhino.FileIO.File3dm.Read(target)
        if reread is None:raise RuntimeError('Rhino5 output could not be reread')
        after_rows=facts(reread)
        valid=before_rows==after_rows and digest(source)==before
        result['checks'].append(dict(name=entry['name']+' Rhino5 points/normals/colors/identity roundtrip and immutable source',passed=valid))
        if not valid:raise RuntimeError('Rhino5 public fields changed: '+entry['name'])
        result['files'].append(dict(name=entry['name'],input=source,output=target,input_sha256=before,output_sha256=digest(target),rows=before_rows))
        reread.Dispose();model.Dispose()
    result['ok']=True
except Exception:result['error']=traceback.format_exc()
finally:
    with open(os.path.join(out,'rhino5-results.json'),'w') as stream:json.dump(result,stream,indent=2)
