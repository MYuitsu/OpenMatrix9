"""Rhino 5 native geometry clipboard; GUI owns capture, commit and publication."""
import os
import json
import tempfile
import threading
import time
from pathlib import Path

FORMAT = 'Rhino 5.0 3DM Clip global mem'
MIME = 'application/x-qt-windows-mime;value="%s"' % FORMAT
MAX_PAYLOAD = 512 * 1024 * 1024
_busy = False
_filter = None
last_result = None

def check_payload_size(size):
    if size < 33 or size > MAX_PAYLOAD:
        raise RuntimeError('Rhino clipboard payload is truncated or exceeds 512 MiB')

def validate_payload(payload):
    check_payload_size(len(payload))
    # Archive parsing/geometry validation follows; a header alone is not proof.
    if payload[:24] != b'3D Geometry File Format ' or payload[24:32].strip() not in (b'5', b'50'):
        raise RuntimeError('Clipboard does not contain a supported Rhino 5 3DM archive')
    return payload

def _win32():
    import ctypes
    from ctypes import wintypes
    user = ctypes.WinDLL('user32', use_last_error=True)
    kernel = ctypes.WinDLL('kernel32', use_last_error=True)
    user.RegisterClipboardFormatW.argtypes=[wintypes.LPCWSTR];user.RegisterClipboardFormatW.restype=wintypes.UINT
    user.OpenClipboard.argtypes=[wintypes.HWND];user.OpenClipboard.restype=wintypes.BOOL
    user.GetClipboardData.argtypes=[wintypes.UINT];user.GetClipboardData.restype=wintypes.HANDLE
    user.IsClipboardFormatAvailable.argtypes=[wintypes.UINT];user.IsClipboardFormatAvailable.restype=wintypes.BOOL
    user.GetClipboardSequenceNumber.restype=wintypes.DWORD
    kernel.GlobalSize.argtypes=[wintypes.HGLOBAL];kernel.GlobalSize.restype=ctypes.c_size_t
    kernel.GlobalLock.argtypes=[wintypes.HGLOBAL];kernel.GlobalLock.restype=ctypes.c_void_p
    kernel.GlobalUnlock.argtypes=[wintypes.HGLOBAL]
    return ctypes,user,kernel,user.RegisterClipboardFormatW(FORMAT)

def has_geometry():
    if os.name != 'nt':return False
    try:
        _,user,_,format_id=_win32()
        return bool(user.IsClipboardFormatAvailable(format_id))
    except OSError:return False

def capture_payload():
    """Snapshot HGLOBAL and unlock before any SDK/geometry work."""
    ctypes,user,kernel,format_id=_win32()
    for attempt in range(2):
        before=user.GetClipboardSequenceNumber()
        if not user.OpenClipboard(None):raise RuntimeError('Windows clipboard is busy; try Paste again')
        try:
            handle=user.GetClipboardData(format_id)
            if not handle:raise RuntimeError('Clipboard has no Rhino 5 geometry')
            size=kernel.GlobalSize(handle);check_payload_size(size)
            pointer=kernel.GlobalLock(handle)
            if not pointer:raise RuntimeError('Cannot lock Rhino clipboard payload')
            try:payload=ctypes.string_at(pointer,size)
            finally:kernel.GlobalUnlock(handle)
        finally:user.CloseClipboard()
        if before==user.GetClipboardSequenceNumber():return validate_payload(payload)
    raise RuntimeError('Clipboard changed during capture; try Paste again')

def _capture_published():
    # Clipboard viewers can briefly open the new OLE data during publication.
    # Retry that transient lock for at most 20 ms; other failures remain fatal.
    for attempt in range(5):
        try:return capture_payload()
        except RuntimeError as error:
            if 'clipboard is busy' not in str(error).lower() or attempt==4:raise
            time.sleep(.005)

def publish_payload(payload):
    """Qt's OLE data object publishes the registered Windows HGLOBAL format."""
    validate_payload(payload)
    from PySide6 import QtCore,QtWidgets
    _,user,_,_=_win32()
    if not user.OpenClipboard(None):raise RuntimeError('Windows clipboard is busy; try Copy again')
    user.CloseClipboard()
    clipboard=QtWidgets.QApplication.clipboard()
    # Materialize the previous data before Qt releases its clipboard owner.
    # A borrowed QMimeData/IDataObject can become invalid after replacement.
    before=user.GetClipboardSequenceNumber()
    previous=QtCore.QMimeData();old=clipboard.mimeData()
    if old is not None:
        for name in old.formats():previous.setData(name,old.data(name))
        if old.hasImage():previous.setImageData(old.imageData())
    if before!=user.GetClipboardSequenceNumber():
        raise RuntimeError('Clipboard changed during capture; try Copy again')
    mime=QtCore.QMimeData();mime.setData(MIME,QtCore.QByteArray(payload))
    import ctypes
    clipboard.setMimeData(mime)
    published=user.GetClipboardSequenceNumber()
    try:
        if _capture_published()!=payload:raise RuntimeError('Rhino clipboard publication failed')
        # Materialize the OLE object so geometry survives the source application's exit.
        result=ctypes.OleDLL('ole32').OleFlushClipboard()
        if result != 0:raise RuntimeError('Cannot retain Rhino clipboard data after application exit')
    except BaseException:
        # Never overwrite a newer clipboard owned by another application.
        if user.GetClipboardSequenceNumber()==published and published!=before:
            clipboard.setMimeData(previous)
            flush=ctypes.WinDLL('ole32').OleFlushClipboard
            flush.restype=ctypes.c_long
            if flush()!=0:raise RuntimeError('Clipboard publication failed and previous data could not be retained')
        raise

def available(operation):
    if _busy:return False
    import FreeCAD as App,FreeCADGui as Gui,OpenMatrix9Gui as native
    doc=App.ActiveDocument
    if doc is None:return False
    try:native.validateDocument(doc.Name)
    except RuntimeError:return False
    if operation in (3,5,6,7,'copy','copy_session'):
        # Availability never walks a large scene or extracts geometry. Exact
        # scope, unsupported inventory and bindings are native/Rust preflight
        # when invoked; an empty selection can hand off the complete palette.
        return True
    return has_geometry()

def _run_preparation(operation,work,finish,staging):
    """A modal progress loop keeps GUI events alive while native code releases GIL."""
    global _busy,last_result
    if _busy:raise RuntimeError('A 3DM clipboard operation is already running')
    from PySide6 import QtCore,QtWidgets
    import FreeCADGui as Gui
    state={};_busy=True
    progress=QtWidgets.QProgressDialog(operation+' Rhino geometry…','Cancel',0,0,Gui.getMainWindow())
    progress.setWindowModality(QtCore.Qt.ApplicationModal);progress.setMinimumDuration(0)
    progress.setAutoClose(False);progress.setAutoReset(False)
    def cancel():
        Path(staging,'cancel').touch()
        progress.setLabelText('Cancelling; finishing active geometry tasks…')
    progress.canceled.connect(cancel)
    def worker():
        try:state['value']=work()
        except BaseException as error:state['error']=error
        finally:state['done']=True
    thread=threading.Thread(target=worker,name='OM9ClipboardCoordinator')
    loop=QtCore.QEventLoop();timer=QtCore.QTimer();timer.setInterval(20)
    def poll():
        if state.get('done'):loop.quit()
    timer.timeout.connect(poll)
    started=time.perf_counter()
    try:
        thread.start();timer.start();progress.show();loop.exec()
        thread.join();timer.stop()
        if Path(staging,'cancel').exists():raise RuntimeError('3DM operation cancelled')
        if 'error' in state:raise state['error']
        # Worker ended and staging remains alive throughout the GUI commit.
        value=finish(state['value'])
        last_result={'operation':operation,'seconds':time.perf_counter()-started,'ok':True}
        return value
    finally:
        if thread.is_alive():cancel();thread.join()
        timer.stop();progress.close();progress.deleteLater();_busy=False

def copy_selection():
    return _copy_scope(1)


def copy_session():
    return _copy_scope(2)


def _copy_scope(scope):
    import FreeCAD as App,FreeCADGui as Gui,OpenMatrix9Gui as native,ThreeDm
    if _busy:raise RuntimeError('A 3DM clipboard operation is already running')
    doc=App.ActiveDocument
    if doc is None:raise RuntimeError('Open an active project before Copy')
    selected=Gui.Selection.getSelectionEx() if scope==1 else []
    if any(s.SubElementNames for s in selected):raise RuntimeError('Select whole objects, without subelements')
    native.validateDocument(doc.Name)
    objects=native.collectLayerTransfer3dm(doc.Name,[s.Object for s in selected],scope)
    document_name=doc.Name
    with tempfile.TemporaryDirectory(prefix='om9-clipboard-copy-') as staging:
        # Snapshot current geometry on GUI thread; workers only receive detached files/data.
        prepared=ThreeDm._stage_current_geometry(objects,staging)
        target=Path(staging,'copy.3dm')
        def work():
            ThreeDm._write_geometry_atomic(native,prepared,target)
            check_payload_size(target.stat().st_size)
            return validate_payload(target.read_bytes())
        def finish(data):
            # Worker owns detached data. Revalidate current native bindings and
            # canonical generation before paired clipboard publication.
            native.validateDocument(document_name)
            native.validateLayerExport3dm(document_name,objects,json.dumps(prepared['layer_session']))
            return json.loads(native.publishLayerClipboard3dm(data,json.dumps(prepared['layer_session']),scope))
        return _run_preparation('Copy' if scope==1 else 'Copy Session',work,finish,staging)

def paste_selection():
    import FreeCAD as App,FreeCADGui as Gui,OpenMatrix9Gui as native,ThreeDm
    from ThreeDmModeling import validate_prepared
    if _busy:raise RuntimeError('A 3DM clipboard operation is already running')
    doc=App.ActiveDocument
    if doc is None:raise RuntimeError('Open an active project before Paste')
    native.validateDocument(doc.Name)
    with tempfile.TemporaryDirectory(prefix='om9-clipboard-paste-') as staging:
        source=Path(staging,'clipboard.3dm');metadata=Path(staging,'clipboard.layer.json')
        receipt=json.loads(native.captureLayerClipboard3dm(str(source),str(metadata)))
        def work():
            model=json.loads(native.prepareModeling3dm(str(source),staging,0.0,receipt['metadata_file'] or ''))
            validate_prepared(model['items'],allow_empty='layer_session' in model)
            return model
        def finish(model):
            if App.ActiveDocument!=doc or App.getDocument(doc.Name)!=doc:raise RuntimeError('Paste target project changed or closed')
            native.validateDocument(doc.Name)
            prepared=ThreeDm._prepare_import(source,staging,0,'modeling',model=model)
            objects=native.commit3dm(doc.Name,prepared)
            Gui.Selection.clearSelection()
            for obj in objects:Gui.Selection.addSelection(obj)
            return objects
        objects=_run_preparation('Paste',work,finish,staging)
        last_result['clipboard']=receipt
        if receipt['evidence']==1:
            App.Console.PrintMessage('3DM Paste: native Rhino clipboard has no extended layer-session metadata; unused palette and persistent-state evidence is reduced.\n')
        return objects

def shortcut_context(widget):
    from PySide6 import QtWidgets
    import FreeCADGui as Gui
    if widget is None or QtWidgets.QApplication.activeModalWidget() or QtWidgets.QApplication.activePopupWidget():return False
    if Gui.activeWorkbench().name()!='OpenMatrix9Workbench':return False
    tree=False;viewport=False
    while widget is not None:
        name=(widget.metaObject().className()+' '+widget.objectName()).lower()
        if isinstance(widget,(QtWidgets.QLineEdit,QtWidgets.QTextEdit,QtWidgets.QPlainTextEdit,QtWidgets.QAbstractSpinBox)) or 'propertyeditor' in name:return False
        tree=tree or isinstance(widget,QtWidgets.QTreeView)
        viewport=viewport or 'view3d' in name
        widget=widget.parentWidget()
    return tree or viewport

def install_shortcuts():
    global _filter
    if _filter is not None:return
    from PySide6 import QtCore,QtWidgets
    import FreeCADGui as Gui
    class ClipboardFilter(QtCore.QObject):
        def eventFilter(self,receiver,event):
            if event.type() not in (QtCore.QEvent.ShortcutOverride,QtCore.QEvent.KeyPress):return False
            if event.modifiers()!=QtCore.Qt.ControlModifier or event.key() not in (QtCore.Qt.Key_C,QtCore.Qt.Key_V):return False
            if not shortcut_context(QtWidgets.QApplication.focusWidget()):return False
            if event.type()==QtCore.QEvent.ShortcutOverride:event.accept();return True
            if not event.isAutoRepeat():
                # Same registered native command used by menu and Command area.
                Gui.runCommand('Copy3dm' if event.key()==QtCore.Qt.Key_C else 'Paste3dm')
            event.accept();return True
    _filter=ClipboardFilter(Gui.getMainWindow());QtWidgets.QApplication.instance().installEventFilter(_filter)

def remove_shortcuts():
    global _filter
    if _filter is not None:
        from PySide6 import QtWidgets
        QtWidgets.QApplication.instance().removeEventFilter(_filter);_filter.deleteLater();_filter=None
