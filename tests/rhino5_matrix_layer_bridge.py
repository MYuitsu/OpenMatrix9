# -*- coding: utf-8 -*-
"""Bootstrap only; actual Rhino adapter invokes shared Safe Rust policy."""
import os,json,hashlib,ntpath
import System,clr,Rhino
ROOT=r'H:\FreeCAD-src\build\om9-layer-edits\bridge\rhino5\runtime'
assembly=None
def load():
    global assembly
    with open(ntpath.join(ROOT,'manifest.json'),'rb') as stream:manifest=json.loads(stream.read().decode('utf-8-sig'))
    for field in ('adapter','rust'):
        name=manifest[field]
        if name!=ntpath.basename(name):raise RuntimeError('Foreign bridge binary path')
        path=ntpath.join(ROOT,name)
        with open(path,'rb') as stream:digest=hashlib.sha256(stream.read()).hexdigest()
        if digest!=manifest[field+'_sha256']:raise RuntimeError('Bridge runtime hash mismatch')
    import ctypes
    kernel=ctypes.windll.kernel32;kernel.LoadLibraryW.argtypes=[ctypes.c_wchar_p];kernel.LoadLibraryW.restype=ctypes.c_void_p
    if not kernel.LoadLibraryW(ntpath.join(ROOT,manifest['rust'])):raise RuntimeError('Cannot load exact Rust layer ABI DLL')
    assembly=System.Reflection.Assembly.LoadFile(ntpath.join(ROOT,manifest['adapter']))
    call('NativeLayerApi','VerifyAbi')
    call('RhinoLayerAdapter','RetirePreviousDiagnostics')
    return manifest
def call(kind,method,*args):
    if assembly is None:load()
    try:return assembly.GetType('OM9LayerTransfer.'+kind,True).GetMethod(method).Invoke(None,System.Array[System.Object](list(args)))
    except System.Reflection.TargetInvocationException as error:raise RuntimeError(str(error.InnerException))
def copy_selected(doc):
    pair=call('RhinoLayerAdapter','Serialize',doc,System.Boolean(False))
    call('NativeClipboard','Publish',pair[0],pair[1])
def receive(doc,fault=''):
    pair=call('NativeClipboard','Capture')
    call('RhinoLayerAdapter','Receive',doc,pair[0],pair[1],fault)
