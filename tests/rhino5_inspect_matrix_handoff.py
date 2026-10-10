# -*- coding: utf-8 -*-
"""Read-only evidence after failed handoff. Never repair state or launch apps."""
import os,sys,ntpath,json,traceback
import Rhino,System,clr
BASE=r'H:\FreeCAD-src\build\om9-layer-edits\tests'
source_file=ntpath.join(BASE,'rhino5_verify_matrix_om9_handoff.py')
with open(source_file,'r') as stream:source=stream.read()
# Load the same snapshot adapter without registering its Idle driver.
namespace={}
exec(source[:source.rindex('\ntry:start()')],namespace)
output=ntpath.join(r'H:\FreeCAD-src\build\matrix-om9-handoff\303c98e676534ffb91c894c130fba7b7',
    'read-only-'+System.Guid.NewGuid().ToString('N'))
os.makedirs(output)
report={'scope':'Read-only current Matrix evidence after failed run; no application acceptance',
    'failed_run':'303c98e676534ffb91c894c130fba7b7','ok':False,
    'clipboard_not_original_run_evidence':True}
try:
    doc=Rhino.RhinoDoc.ActiveDoc
    before=(int(doc.DocumentId),bool(doc.Modified),str(doc.Path))
    report['current_state']=namespace['snapshot']()
    report['layer_indices']=[dict(index=int(i),id=str(doc.Layers[i].Id),path=str(doc.Layers[i].FullPath),
        parent=str(doc.Layers[i].ParentLayerId),deleted=bool(doc.Layers[i].IsDeleted),current=i==doc.Layers.CurrentLayerIndex)
        for i in range(doc.Layers.Count)]
    import ctypes
    user=ctypes.windll.user32;kernel=ctypes.windll.kernel32
    user.RegisterClipboardFormatW.argtypes=[ctypes.c_wchar_p];user.RegisterClipboardFormatW.restype=ctypes.c_uint
    user.GetClipboardData.argtypes=[ctypes.c_uint];user.GetClipboardData.restype=ctypes.c_void_p
    kernel.GlobalSize.argtypes=[ctypes.c_void_p];kernel.GlobalSize.restype=ctypes.c_size_t
    kernel.GlobalLock.argtypes=[ctypes.c_void_p];kernel.GlobalLock.restype=ctypes.c_void_p
    kernel.GlobalUnlock.argtypes=[ctypes.c_void_p]
    fmt=user.RegisterClipboardFormatW('Rhino 5.0 3DM Clip global mem')
    report['geometry_clipboard_available']=bool(user.IsClipboardFormatAvailable(fmt))
    if report['geometry_clipboard_available']:
        if not user.OpenClipboard(None):raise RuntimeError('Clipboard busy; read-only capture stopped')
        try:
            handle=user.GetClipboardData(fmt);size=int(kernel.GlobalSize(handle))
            if not handle or size<33 or size>512*1024*1024:raise RuntimeError('Clipboard geometry outside bounded range')
            pointer=kernel.GlobalLock(handle)
            if not pointer:raise RuntimeError('Cannot read geometry clipboard')
            try:data=ctypes.string_at(pointer,size)
            finally:kernel.GlobalUnlock(handle)
        finally:user.CloseClipboard()
        path=ntpath.join(output,'clipboard-observed.3dm')
        with open(path,'wb') as stream:stream.write(data)
        model=Rhino.FileIO.File3dm.Read(path)
        if model is None:raise RuntimeError('Rhino SDK cannot read captured geometry clipboard')
        try:
            report['archive_objects']=[]
            for item in model.Objects:
                attrs=item.Attributes;box=item.Geometry.GetBoundingBox(True)
                report['archive_objects'].append(dict(id=str(attrs.ObjectId),name=str(attrs.Name),
                    layer_index=int(attrs.LayerIndex),mode=str(attrs.Mode),visible=bool(attrs.Visible),
                    geometry_type=str(item.Geometry.ObjectType),bounds=[float(box.Min.X),float(box.Min.Y),float(box.Min.Z),float(box.Max.X),float(box.Max.Y),float(box.Max.Z)]))
            report['archive_layer_count']=int(model.Layers.Count)
        finally:model.Dispose()
    after=(int(doc.DocumentId),bool(doc.Modified),str(doc.Path))
    report['unchanged_document_stamp']=before==after
    report['ok']=report['unchanged_document_stamp']
except Exception:report['error']=traceback.format_exc()
namespace['support'].write_json(ntpath.join(output,'matrix-observation.json'),report)
Rhino.RhinoApp.WriteLine('Read-only Matrix observation: '+('PASS' if report['ok'] else 'FAIL')+'; '+ntpath.join(output,'matrix-observation.json'))
