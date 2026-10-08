"""Double current PointCloud data; Coin display is a derived float preview."""
import hashlib
import json
import os
import struct

PROPERTIES=dict(points='OM9CloudPoints',normals='OM9CloudNormals',colors='OM9CloudRGBA',
                values='OM9CloudValues',plane='OM9CloudPlane',ordered='OM9CloudOrdered',
                has_plane='OM9CloudHasPlane')

def is_adapter(obj):return 'OM9PointCloudSchema' in obj.PropertiesList

def fields(obj, record=None):
    if not is_adapter(obj) or obj.OM9PointCloudSchema!=1 or not obj.isDerivedFrom('App::FeaturePython'):
        raise RuntimeError('PointCloud has no supported current-field schema')
    if record is not None and (record.get('class_name')!='ON_PointCloud' or json.loads(obj.OM9SourceRecord)!=record):
        raise RuntimeError('PointCloud native source record differs from verified archive')
    result={key:getattr(obj,name) for key,name in PROPERTIES.items()}
    for key in ('points','normals'):result[key]=[list(vector) for vector in result[key]]
    result['schema_version']=1
    return result

def signature(obj):
    data=fields(obj);digest=hashlib.sha256()
    for key in ('points','normals'):
        digest.update(struct.pack('<I',len(data[key])))
        for vector in data[key]:digest.update(struct.pack('<3d',*vector))
    digest.update(struct.pack('<I',len(data['colors'])))
    for value in data['colors']:digest.update(struct.pack('<i',value))
    digest.update(struct.pack('<I',len(data['values'])))
    for value in data['values']:digest.update(struct.pack('<d',value))
    digest.update(struct.pack('<16d??',*data['plane'],data['ordered'],data['has_plane']))
    return digest.hexdigest()

def changed(obj,record,manifest):
    fields(obj,record)
    import ThreeDmArchiveState as state
    return state.source_signature(obj,metadata_override=json.loads(obj.OM9SourceMetadata))!=obj.OM9SourceSignature

def read_data(path,digest):
    if os.path.getsize(path)>512*1024*1024:raise RuntimeError('PointCloud current fields exceed512MiB')
    with open(path,'rb') as stream:payload=stream.read()
    if hashlib.sha256(payload).hexdigest()!=digest:raise RuntimeError('PointCloud prepared field hash mismatch')
    result=json.loads(payload)
    if result.get('schema_version')!=1:raise RuntimeError('Unsupported PointCloud prepared field schema')
    return result

def stage(obj,record,staging,index):
    path=os.path.join(staging,'point-cloud-'+obj.Name+'-'+str(index)+'.json')
    payload=json.dumps(fields(obj,record),separators=(',',':'),allow_nan=False).encode('utf-8')
    if len(payload)>512*1024*1024:raise RuntimeError('PointCloud current fields exceed512MiB')
    with open(path,'wb') as stream:stream.write(payload)
    return dict(point_cloud_fields=path,point_cloud_sha256=hashlib.sha256(payload).hexdigest())

def bind(obj,data):
    import FreeCAD as App
    from ThreeDmArchiveState import _property
    _property(obj,'Integer','OM9PointCloudSchema',1)
    for key,name in PROPERTIES.items():
        value=data[key]
        kind='FloatList'
        if key in ('points','normals'):kind,value='VectorList',[App.Vector(*row) for row in value]
        elif key=='colors':kind='IntegerList'
        elif key in ('ordered','has_plane'):kind='Bool'
        obj.addProperty('App::Property'+kind,name,'Rhino PointCloud')
        setattr(obj,name,value)
    # Runtime point hiding belongs to FreeCAD display persistence, not3DM.
    obj.ViewObject.addProperty('App::PropertyBoolList','OM9HiddenPoints','Rhino PointCloud display',
                               'Local display only. openNURBS does not store hidden-point flags in3DM.')
    obj.ViewObject.OM9HiddenPoints=[]
    obj.ViewObject.addProperty('App::PropertyFloat','PointSize','Rhino PointCloud display')
    obj.ViewObject.PointSize=3
    obj.ViewObject.Proxy=ViewProvider()

def placement_matrix(obj):
    """Native local placement plus the physical App::Part ancestry, once."""
    import FreeCAD as App
    placement=getattr(obj,'OM9BlockMemberPlacement',App.Placement())
    parents=[o for o in obj.InList if o.isDerivedFrom('App::Part') and obj in o.Group]
    if len(parents)>1:raise RuntimeError('PointCloud has ambiguous physical parents')
    if parents:placement=parents[0].getGlobalPlacement()*placement
    return list(placement.toMatrix().A)

def transformed_fields(obj,matrix):
    """Derive display fields with the same native affine rules as export."""
    import tempfile,OpenMatrix9Gui as native
    with tempfile.TemporaryDirectory(prefix='om9-cloud-preview-') as staging:
        request=stage(obj,None,staging,0);request['point_cloud_transform']=list(matrix.A)
        row=json.loads(native.transform3dmPointCloud(json.dumps(request),staging))
        return read_data(row['point_cloud_fields'],row['point_cloud_sha256'])

def update_fields(obj,data):
    import FreeCAD as App
    for key,name in PROPERTIES.items():
        value=data[key]
        if key in ('points','normals'):value=[App.Vector(*row) for row in value]
        setattr(obj,name,value)

class ViewProvider:
    def attach(self,view):
        from pivy import coin
        self.Object=view.Object;self.root=coin.SoSeparator();self.coordinates=coin.SoCoordinate3()
        self.material=coin.SoMaterial();self.binding=coin.SoMaterialBinding();self.style=coin.SoDrawStyle();self.points=coin.SoPointSet()
        for node in (self.style,self.material,self.binding,self.coordinates,self.points):self.root.addChild(node)
        view.addDisplayMode(self.root,'PointCloud');self.update()
    def update(self):
        if not hasattr(self,'Object'):return
        from pivy import coin
        obj=self.Object
        try:
            points=obj.OM9CloudPoints;hidden=obj.ViewObject.OM9HiddenPoints
            if len(hidden) not in (0,len(points)):raise RuntimeError('PointCloud local hidden count differs from points')
            colors=obj.OM9CloudRGBA
            if len(colors) not in (0,4*len(points)):raise RuntimeError('PointCloud display color count differs from points')
            indices=[i for i in range(len(points)) if not hidden or not hidden[i]]
            placement=getattr(obj,'OM9BlockMemberPlacement',None)
            self.coordinates.point.setNum(len(indices))
            self.coordinates.point.setValues(0,[list(placement.multVec(points[i]) if placement else points[i]) for i in indices])
            self.points.numPoints=len(indices);self.style.pointSize=max(1,float(obj.ViewObject.PointSize))
            if colors:
                self.binding.value=coin.SoMaterialBinding.PER_VERTEX
                self.material.diffuseColor.setNum(len(indices));self.material.transparency.setNum(len(indices))
                self.material.diffuseColor.setValues(0,[[colors[4*i+j]/255.0 for j in range(3)] for i in indices])
                self.material.transparency.setValues(0,[colors[4*i+3]/255.0 for i in indices])
            else:
                self.binding.value=coin.SoMaterialBinding.OVERALL
                self.material.diffuseColor.setNum(1);self.material.diffuseColor.setValue(*list(getattr(obj,'OM9Color',(0.7,0.7,0.7)))[:3]);self.material.transparency.setNum(1);self.material.transparency.setValue(0)
        except (AttributeError,RuntimeError,ValueError):self.points.numPoints=0
    def updateData(self,obj,prop):self.update()
    def onChanged(self,view,prop):
        if prop in ('PointSize','OM9HiddenPoints'):self.update()
    def getDisplayModes(self,view):return ['PointCloud']
    def getDefaultDisplayMode(self):return 'PointCloud'
    def setDisplayMode(self,mode):return mode
    def __getstate__(self):return None
    def __setstate__(self,state):pass
