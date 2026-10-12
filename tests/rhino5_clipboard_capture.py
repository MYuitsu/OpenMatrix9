# IronPython 2.7 in actual Rhino 5. Run in an empty test document.
import Rhino,scriptcontext as sc,os,json,traceback,ctypes,uuid,ntpath
import System
out=os.environ.get('OM9_RHINO_CLIPBOARD_OUTPUT') or 'H:/FreeCAD-src/build/rhino5-clipboard/'+uuid.uuid4().hex
if not os.path.isdir(out):os.makedirs(out)
report={'checks':[],'rhino_version':str(Rhino.RhinoApp.Version),'pid':os.getpid()}
owned_document=False
def check(name,value):
    report['checks'].append({'name':name,'passed':bool(value)})
    with open(ntpath.join(out,'capture-progress.json'),'w') as f:json.dump(report,f,indent=2)
    if not value:raise RuntimeError(name)
try:
    settings=Rhino.DocObjects.ObjectEnumeratorSettings();settings.NormalObjects=True;settings.HiddenObjects=True;settings.LockedObjects=True
    check('owned empty document',len(list(sc.doc.Objects.GetObjectList(settings)))==0 and not sc.doc.Modified)
    owned_document=True
    curve=Rhino.Geometry.LineCurve(Rhino.Geometry.Point3d(20,0,0),Rhino.Geometry.Point3d(30,0,0))
    identity=sc.doc.Objects.AddCurve(curve);check('fixture line added',identity!=System.Guid.Empty)
    sc.doc.Objects.Select(identity)
    report['stage']='before native Copy'
    with open(ntpath.join(out,'capture-progress.json'),'w') as f:json.dump(report,f,indent=2)
    check('native Rhino CopyToClipboard',Rhino.RhinoApp.RunScript('_CopyToClipboard',False))
    user=ctypes.windll.user32;kernel=ctypes.windll.kernel32
    user.RegisterClipboardFormatW.argtypes=[ctypes.c_wchar_p];user.RegisterClipboardFormatW.restype=ctypes.c_uint
    user.GetClipboardData.restype=ctypes.c_void_p;kernel.GlobalSize.argtypes=[ctypes.c_void_p];kernel.GlobalSize.restype=ctypes.c_size_t
    kernel.GlobalLock.argtypes=[ctypes.c_void_p];kernel.GlobalLock.restype=ctypes.c_void_p;kernel.GlobalUnlock.argtypes=[ctypes.c_void_p]
    fmt=user.RegisterClipboardFormatW(u'Rhino 5.0 3DM Clip global mem')
    check('native clipboard opens',user.OpenClipboard(None))
    try:
        handle=user.GetClipboardData(fmt);check('Rhino5 registered HGLOBAL format',bool(handle));size=kernel.GlobalSize(handle)
        check('bounded nonempty native payload',size>32 and size<=512*1024*1024)
        pointer=kernel.GlobalLock(handle);check('global memory locked',bool(pointer))
        try:data=ctypes.string_at(pointer,size)
        finally:kernel.GlobalUnlock(handle)
    finally:user.CloseClipboard()
    report['format']='Rhino 5.0 3DM Clip global mem';report['medium']='HGLOBAL';report['size']=size;report['header_hex']=data[:48].encode('hex')
    with open(ntpath.join(out,'rhino-copy.3dm'),'wb') as f:f.write(data)
    report['ok']=True
except:report.update(ok=False,error=traceback.format_exc())
with open(ntpath.join(out,'capture.json'),'w') as f:json.dump(report,f,indent=2)
if owned_document:sc.doc.Modified=False
print('Rhino5 clipboard capture: '+str(report['ok'])+'; report '+out)
