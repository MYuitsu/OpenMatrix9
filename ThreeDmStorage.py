# SPDX-License-Identifier: LGPL-2.1-or-later
"""Plugin-owned FCStd payloads; paths are disposable, verified converter caches."""
import base64
import binascii
import hashlib
import math
import os
import tempfile

CHUNK_BYTES = 64 * 1024
LIMITS = {'OM9SourceArchive':512 * 1024 * 1024,
          'OM9HatchLoopFile':32 * 1024 * 1024}
_directory = tempfile.TemporaryDirectory(prefix='om9-payload-cache-')
_cache = {}
_observer = None

def _discard(key):
    cached=_cache.get(key)
    if cached:
        try:os.unlink(cached[1])
        except FileNotFoundError:pass
        _cache.pop(key,None)

def _encode(raw, limit):
    if not isinstance(raw, bytes) or len(raw)>limit:
        raise RuntimeError('Native payload exceeds its byte limit or is not bytes')
    return [base64.b64encode(raw[i:i+CHUNK_BYTES]).decode('ascii')
            for i in range(0,len(raw),CHUNK_BYTES)] or ['']

def _chunks(obj,name,limit):
    if getattr(obj,name+'StorageVersion',None)!=1:
        raise RuntimeError('Unsupported plugin payload storage version')
    chunks=getattr(obj,name+'Chunks')
    if not isinstance(chunks,(list,tuple)) or not chunks or len(chunks)>max(1,math.ceil(limit/CHUNK_BYTES)):
        raise RuntimeError('Invalid native payload chunk count')
    digest=getattr(obj,name+'Digest')
    if not isinstance(digest,str) or len(digest)!=64 or any(c not in '0123456789abcdef' for c in digest):
        raise RuntimeError('Invalid native payload digest')
    return chunks,digest

def _decode(chunks,digest,limit):
    size=0;parts=[];check=hashlib.sha256()
    try:
        for index,text in enumerate(chunks):
            if not isinstance(text,str) or len(text)>4*((CHUNK_BYTES+2)//3):
                raise RuntimeError('Invalid native payload chunk size')
            part=base64.b64decode(text.encode('ascii'),validate=True)
            if len(part)>CHUNK_BYTES or (index<len(chunks)-1 and len(part)!=CHUNK_BYTES):
                raise RuntimeError('Invalid native payload chunk boundaries')
            size+=len(part)
            if size>limit:raise RuntimeError('Native payload exceeds its byte limit')
            check.update(part);parts.append(part)
    except (ValueError,UnicodeError,binascii.Error) as error:
        raise RuntimeError('Invalid native payload encoding') from error
    if check.hexdigest()!=digest:raise RuntimeError('Native payload digest differs')
    return b''.join(parts)

def read(obj,name,limit):
    if name+'Chunks' in obj.PropertiesList:
        return _decode(*_chunks(obj,name,limit),limit)
    # Legacy FCStd support is read-only until document-wide preflight/migration.
    try:
        with open(getattr(obj,name),'rb') as stream:raw=stream.read(limit+1)
    except (OSError,AttributeError) as error:
        raise RuntimeError('Cannot read legacy native payload') from error
    if len(raw)>limit:raise RuntimeError('Legacy native payload exceeds its byte limit')
    return raw

def _file_hash(pathname):
    digest=hashlib.sha256()
    with open(pathname,'rb') as stream:
        for part in iter(lambda:stream.read(CHUNK_BYTES),b''):digest.update(part)
    return digest.hexdigest()

def _materialize(obj,name,chunks,digest,limit):
    # Copying strings never transfers file ownership. Each host cache is separate.
    key=(obj.Document.Name,obj.Name,name)
    token=hashlib.sha256()
    for text in chunks:
        if not isinstance(text,str) or len(text)>4*((CHUNK_BYTES+2)//3):
            raise RuntimeError('Invalid native payload chunk size')
        try:token.update(text.encode('ascii'))
        except UnicodeError as error:raise RuntimeError('Invalid native payload encoding') from error
        token.update(b'\0')
    identity=(token.hexdigest(),digest,limit)
    cached=_cache.get(key)
    if cached and cached[0]==identity and os.path.isfile(cached[1]):
        if _file_hash(cached[1])!=digest:raise RuntimeError('Materialized native payload digest differs')
        return cached[1]
    raw=_decode(chunks,digest,limit)
    # Reuse the host's path, so undo/redo never accumulates payload copies.
    # Validation above completes before replacing any existing bytes.
    descriptor,filename=tempfile.mkstemp(prefix='payload-',dir=_directory.name)
    try:
        with os.fdopen(descriptor,'wb') as stream:stream.write(raw)
        if cached:
            os.replace(filename,cached[1]);filename=cached[1]
    except Exception:
        os.unlink(filename);raise
    _cache[key]=(identity,filename)
    return filename

def path(obj,name,limit):
    if name+'Chunks' not in obj.PropertiesList:
        read(obj,name,limit)
        return getattr(obj,name)
    return _materialize(obj,name,*_chunks(obj,name,limit),limit)

def _property(obj,kind,name,value):
    obj.addProperty('App::Property'+kind,name,'Rhino source')
    setattr(obj,name,value);obj.setEditorMode(name,1)

def bind(obj,name,raw,limit):
    chunks=_encode(raw,limit);digest=hashlib.sha256(raw).hexdigest()
    filename=_materialize(obj,name,chunks,digest,limit)
    legacy=name in obj.PropertiesList
    if legacy:
        if obj.getTypeIdOfProperty(name)!='App::PropertyFileIncluded':
            raise RuntimeError('Native payload path property has an incompatible type')
    _property(obj,'StringList',name+'Chunks',chunks)
    _property(obj,'String',name+'Digest',digest)
    _property(obj,'Integer',name+'StorageVersion',1)
    cache_name=name+'CachePath' if legacy else name
    _property(obj,'String',cache_name,filename)
    obj.setPropertyStatus(cache_name,'Transient')
    obj.Proxy=PayloadProxy()

def write(obj,name,raw,limit):
    chunks=_encode(raw,limit);digest=hashlib.sha256(raw).hexdigest()
    if name+'Chunks' not in obj.PropertiesList:
        raise RuntimeError('Reopen or migrate the legacy project before editing native payloads')
    _chunks(obj,name,limit)
    filename=_materialize(obj,name,chunks,digest,limit)
    setattr(obj,name+'Chunks',chunks);setattr(obj,name+'Digest',digest)
    setattr(obj,name+'CachePath' if name+'CachePath' in obj.PropertiesList else name,filename)

class PayloadProxy:
    def execute(self,obj):pass
    def onDocumentRestored(self,obj):
        for name,limit in LIMITS.items():
            if name+'Chunks' in obj.PropertiesList:
                setattr(obj,name+'CachePath' if name+'CachePath' in obj.PropertiesList else name,path(obj,name,limit))
        _legacy_status(obj)
    def dumps(self):return None
    def loads(self,state):pass

def migrate_document(document):
    """Capture every legacy file before replacing any file-owning property."""
    pending=[];values={}
    for obj in document.Objects:
        for name,limit in LIMITS.items():
            if name not in obj.PropertiesList or name+'Chunks' in obj.PropertiesList:continue
            if obj.getTypeIdOfProperty(name)!='App::PropertyFileIncluded':continue
            expected=getattr(obj,'OM9ArchiveHash' if name=='OM9SourceArchive' else 'OM9HatchLoopHash','')
            key=(getattr(obj,name),limit,expected)
            if key not in values:
                raw=read(obj,name,limit)
                if hashlib.sha256(raw).hexdigest()!=expected:
                    raise RuntimeError('Legacy native payload digest differs; project was not migrated')
                values[key]=raw
            raw=values[key]
            _materialize(obj,name,_encode(raw,limit),expected,limit)
            pending.append((obj,name,raw,limit))
    if not pending:return 0
    if document.HasPendingTransaction:
        raise RuntimeError('Finish the current transaction before migrating legacy payload storage')
    undo_mode=document.UndoMode
    if not undo_mode:document.UndoMode=1
    document.openTransaction('Migrate OM9 native payload storage')
    try:
        for obj,name,raw,limit in pending:bind(obj,name,raw,limit)
        document.commitTransaction()
        for obj,name,raw,limit in pending:_legacy_status(obj)
    except Exception:
        document.abortTransaction()
        for obj,name,raw,limit in pending:_legacy_status(obj)
        raise
    finally:
        if not undo_mode:document.UndoMode=undo_mode
    return len(pending)

def _legacy_status(obj):
    # Never remove, reassign or record a file-owning legacy property: stock
    # copies may alias its path. Chunks are authoritative; Transient omits the
    # old bytes from save/copy. Undo restores legacy persistence when chunks go.
    for name in LIMITS:
        if name in obj.PropertiesList and obj.getTypeIdOfProperty(name)=='App::PropertyFileIncluded':
            obj.setPropertyStatus(name,'Transient' if name+'Chunks' in obj.PropertiesList else '-Transient')

class _Observer:
    def _migrate(self,document):
        if document.Restoring or document.HasPendingTransaction:return False
        try:migrate_document(document)
        except RuntimeError as error:
            import FreeCAD as App
            App.Console.PrintError('OM9 payload storage: '+str(error)+'\n')
        return True
    def slotCreatedDocument(self,document):
        # The Python observer API does not expose FinishRestoreDocument.
        # GUI event delivery runs after Open has restored all embedded files.
        import FreeCAD as App
        if App.GuiUp:
            from PySide6 import QtCore
            def restored():
                if document not in App.listDocuments().values():return
                # Open may pump GUI events before restoration has finished.
                if not self._migrate(document):QtCore.QTimer.singleShot(50,restored)
            QtCore.QTimer.singleShot(0,restored)
    def slotBeforeRecomputeDocument(self,document):self._migrate(document)
    def slotUndoDocument(self,document):
        for obj in document.Objects:_legacy_status(obj)
    def slotRedoDocument(self,document):self.slotUndoDocument(document)
    def slotDeletedObject(self,obj):
        for key in list(_cache):
            if key[:2]==(obj.Document.Name,obj.Name):_discard(key)
    def slotDeletedDocument(self,document):
        for key in list(_cache):
            if key[0]==document.Name:_discard(key)

def ensure_observer():
    global _observer
    import FreeCAD as App
    if _observer is None:
        _observer=_Observer();App.addDocumentObserver(_observer)
    for document in App.listDocuments().values():
        _observer._migrate(document)
