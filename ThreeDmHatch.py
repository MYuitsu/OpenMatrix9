"""Persistent native Hatch fields; boundary BRep/Coin data is derived display."""
import hashlib,json,math,os,tempfile
MAX_LOOPS=32*1024*1024
PROPERTIES=('OM9HatchOrigin','OM9HatchXAxis','OM9HatchYAxis','OM9HatchBasePoint',
            'OM9HatchRotation','OM9HatchScale','OM9HatchPattern')
def is_adapter(obj):return 'OM9HatchSchema' in obj.PropertiesList
def fields(obj,record=None):
    if not is_adapter(obj) or obj.OM9HatchSchema!=1 or not obj.isDerivedFrom('App::FeaturePython'):
        raise RuntimeError('Hatch adapter has no supported current-field schema')
    saved=json.loads(getattr(obj,'OM9SourceRecord','{}'))
    if record is not None and saved!=record:raise RuntimeError('Hatch source record differs from verified archive')
    native=record if record is not None else saved
    baseline=json.loads(obj.OM9HatchBaseline)
    if native and (native.get('class_name')!='ON_Hatch' or baseline!=native.get('hatch_current')):
        raise RuntimeError('Hatch baseline differs from native source record')
    choices=json.loads(obj.OM9HatchPatternChoices)
    owner=getattr(obj,'OM9ArchiveOwner',None)
    if owner is not None and choices!=json.loads(owner.OM9ArchiveManifest).get('hatch_pattern_choices'):
        raise RuntimeError('Hatch pattern choices differ from source archive')
    import FreeCAD as App
    origin,x,y=[getattr(obj,p) for p in PROPERTIES[:3]]
    if list(origin)==baseline['plane'][:3] and list(x)==baseline['plane'][3:6] and list(y)==baseline['plane'][6:9]:
        plane=list(baseline['plane'])
    else:
        z=x.cross(y)
        plane=list(origin)+list(x)+list(y)+list(z)+list(z)+[-z.dot(origin)]
    base=list(obj.OM9HatchBasePoint)
    angle=obj.OM9HatchRotation.Value
    rotation=baseline['rotation'] if angle==math.degrees(baseline['rotation']) else math.radians(angle)
    scale=obj.OM9HatchScale
    if base[2]!=0 or scale<=0 or not all(math.isfinite(v) for v in plane+base+[rotation,scale]):
        raise RuntimeError('Invalid Hatch plane/base/rotation/positive scale')
    # Native writer repeats frame validation without silently normalizing axes.
    if abs(x.Length-1)>1e-10 or abs(y.Length-1)>1e-10 or abs(x.dot(y))>1e-10:
        raise RuntimeError('Hatch axes require an orthonormal frame')
    pattern=next((c['uuid'] for c in choices if c['label']==str(obj.OM9HatchPattern)),None)
    if pattern is None:raise RuntimeError('Unknown Hatch pattern choice')
    current=dict(plane=plane,base_point=base[:2],rotation=rotation,scale=scale,pattern_uuid=pattern)
    if 'hatch_loop_current' in saved:
        data=loops(obj,saved)
        if data!=saved['hatch_loop_current']:current['loops']=data
    return current

def _encoded(data):
    try:raw=json.dumps(data,sort_keys=True,separators=(',',':'),allow_nan=False).encode('utf-8')
    except (ValueError,TypeError) as error:raise RuntimeError('Invalid native Hatch loop data') from error
    if len(raw)>MAX_LOOPS:raise RuntimeError('Hatch loop data exceeds 32 MiB')
    return raw

def loops(obj,record=None):
    if record is None:record=json.loads(obj.OM9SourceRecord)
    if json.loads(obj.OM9SourceRecord)!=record:raise RuntimeError('Hatch loop source record differs from archive')
    baseline=record.get('hatch_loop_current')
    if baseline is None:raise RuntimeError('Source has no editable native Hatch loop baseline')
    if getattr(obj,'OM9HatchLoopSchema',None)!=1:raise RuntimeError('Unsupported Hatch loop host schema')
    if hashlib.sha256(_encoded(baseline)).hexdigest()!=obj.OM9HatchLoopBaselineHash:
        raise RuntimeError('Hatch loop baseline differs from native source record')
    try:
        from ThreeDmStorage import read
        raw=read(obj,'OM9HatchLoopFile',MAX_LOOPS)
        if len(raw)>MAX_LOOPS:raise RuntimeError('Hatch loop file exceeds 32 MiB')
        if hashlib.sha256(raw).hexdigest()!=obj.OM9HatchLoopHash:raise RuntimeError('Hatch loop file digest differs')
        data=json.loads(raw)
    except (OSError,ValueError,AttributeError) as error:raise RuntimeError('Cannot read embedded Hatch loop data') from error
    if not isinstance(data,dict) or data.get('schema_version')!=1 or set(data)!= {'schema_version','loops'}:
        raise RuntimeError('Unsupported native Hatch loop schema')
    if not isinstance(data['loops'],list) or not data['loops']:raise RuntimeError('Hatch requires native loops')
    _encoded(data)
    return data

def _store_loops(obj,raw):
    from ThreeDmStorage import write
    write(obj,'OM9HatchLoopFile',raw,MAX_LOOPS)
    obj.OM9HatchLoopHash=hashlib.sha256(raw).hexdigest()

def update_loops(obj,data):
    raw=_encoded(data);current=fields(obj);current['loops']=data
    # Native clone/topology checks precede any document property mutation.
    _boundary(obj,current)
    _store_loops(obj,raw)

def edit_loops_ui(obj):
    """Edit detached native values; commit only after native preflight, in one undo."""
    import FreeCAD as App,OpenMatrix9Gui as native
    doc=obj.Document
    if doc is None or App.ActiveDocument!=doc:raise RuntimeError('Activate the Hatch document before editing')
    native.validateDocument(doc.Name)
    original=loops(obj);original_fields=fields(obj);name=obj.Name
    def validate(encoded):
        native.validateDocument(doc.Name)
        if doc.getObject(name)!=obj or loops(obj)!=original or fields(obj)!=original_fields:
            raise RuntimeError('Hatch changed while the editor was open; cancel and reopen it')
        proposed=json.loads(encoded);current=dict(original_fields,loops=proposed)
        _boundary(obj,current)
    proposed=original;message=''
    while True:
        result=native.editHatchLoops3dm(_encoded(proposed).decode('utf-8'),message)
        if result is None:return False
        proposed=json.loads(result)
        try:validate(result)
        except RuntimeError as error:
            message=str(error);continue
        data=proposed;break
    if data==original:return False
    doc.openTransaction('Edit native Hatch boundaries')
    try:
        update_loops(obj,data);doc.recompute();doc.commitTransaction()
    except Exception:
        doc.abortTransaction();raise
    return True
def changed(obj,record,manifest):return fields(obj,record)!=record['hatch_current']
def stage(obj,record,staging,index):return dict(hatch_fields=fields(obj,record))
def bind(obj,record,choices):
    import FreeCAD as App
    from ThreeDmArchiveState import _property
    data=record['hatch_current'];plane=data['plane']
    _property(obj,'Integer','OM9HatchSchema',1)
    _property(obj,'String','OM9HatchBaseline',json.dumps(data,sort_keys=True))
    _property(obj,'String','OM9HatchPatternChoices',json.dumps(choices,sort_keys=True))
    for name,value in zip(PROPERTIES[:4],(plane[:3],plane[3:6],plane[6:9],data['base_point']+[0])):
        _property(obj,'Vector',name,App.Vector(*value));obj.setEditorMode(name,0)
    _property(obj,'Angle','OM9HatchRotation',math.degrees(data['rotation']));obj.setEditorMode('OM9HatchRotation',0)
    _property(obj,'Float','OM9HatchScale',data['scale']);obj.setEditorMode('OM9HatchScale',0)
    _property(obj,'Enumeration','OM9HatchPattern',[c['label'] for c in choices])
    obj.OM9HatchPattern=next(c['label'] for c in choices if c['uuid']==data['pattern_uuid']);obj.setEditorMode('OM9HatchPattern',0)
    if 'hatch_loop_current' in record:
        raw=_encoded(record['hatch_loop_current'])
        _property(obj,'Integer','OM9HatchLoopSchema',1)
        _property(obj,'String','OM9HatchLoopBaselineHash',hashlib.sha256(raw).hexdigest())
        from ThreeDmStorage import bind
        bind(obj,'OM9HatchLoopFile',raw,MAX_LOOPS)
        _property(obj,'String','OM9HatchLoopHash','')
        _store_loops(obj,raw)
    obj.ViewObject.Proxy=ViewProvider(obj.ViewObject)
def boundary(obj):
    return _boundary(obj,fields(obj))

def _boundary(obj,current):
    import OpenMatrix9Gui as native,Part
    from ThreeDmArchiveState import load_archive_state
    owner=obj.OM9ArchiveOwner;loaded=load_archive_state(owner);manifest=loaded['manifest']
    record=next(r for r in manifest['records'] if r['source_uuid']==obj.OM9SourceUUID)
    fields(obj,record)  # Verify host provenance even for a proposed loop overlay.
    request=dict(snapshot=loaded['snapshot'],archive_sha256=manifest['archive_sha256'],scale_mm=manifest['scale_mm'],
                 source_uuid=record['source_uuid'],hatch_fields=current)
    if hasattr(obj,'OM9BlockMemberPlacement'):request['geometry_matrix']=list(obj.OM9BlockMemberPlacement.toMatrix().A)
    with tempfile.TemporaryDirectory(prefix='om9-hatch-boundary-') as staging:
        row=json.loads(native.hatchBoundary3dm(json.dumps(request,allow_nan=False),staging))
        shape=Part.Shape();shape.read(row['brep'])
        if shape.isNull() or not shape.isValid():raise RuntimeError('Invalid derived native Hatch boundary')
        return shape
class ViewProvider:
    def __init__(self,view):self.attach(view)
    def attach(self,view):
        from pivy import coin
        self.Object=view.Object;self.root=coin.SoSeparator();self.cache=None
        view.addDisplayMode(self.root,'HatchBoundary');self.update()
    def update(self):
        if not hasattr(self,'Object'):return
        obj=self.Object
        try:
            signature=json.dumps(dict(fields=fields(obj),placement=list(obj.OM9BlockMemberPlacement.toMatrix().A),
                                     color=list(getattr(obj,'OM9Color',[0.7,0.7,0.7]))),sort_keys=True,allow_nan=False)
            if self.cache==signature:return
            shape=boundary(obj)
            from pivy import coin
            coords=[];counts=[]
            for edge in shape.Edges:
                points=edge.discretize(Deflection=0.05);coords.extend(tuple(p) for p in points);counts.append(len(points))
            color=coin.SoBaseColor();color.rgb.setValue(*list(getattr(obj,'OM9Color',[0.7,0.7,0.7]))[:3])
            positions=coin.SoCoordinate3();positions.point.setValues(0,coords)
            lines=coin.SoLineSet();lines.numVertices.setValues(0,counts)
            self.root.removeAllChildren()
            for node in (color,positions,lines):self.root.addChild(node)
            self.cache=signature
        except (AttributeError,RuntimeError,ValueError,KeyError,StopIteration):
            self.root.removeAllChildren();self.cache=None
    def updateData(self,obj,prop):self.update()
    def doubleClicked(self,view):
        try:edit_loops_ui(view.Object)
        except RuntimeError as error:
            import FreeCAD as App
            App.Console.PrintError(str(error)+'\n')
        return True
    def setupContextMenu(self,view,menu):
        action=menu.addAction('Edit native Hatch boundaries…')
        action.triggered.connect(lambda checked=False:self.doubleClicked(view))
    def getDisplayModes(self,obj):return ['HatchBoundary']
    def getDefaultDisplayMode(self):return 'HatchBoundary'
    def setDisplayMode(self,mode):return mode
    def __getstate__(self):return None
    def __setstate__(self,state):pass
