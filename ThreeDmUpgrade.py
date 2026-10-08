"""Preflight and transactional replacement of explicit legacy archive hosts."""
import json


def policies(upgrades):
    if upgrades is None:return {}
    if not isinstance(upgrades,dict):raise RuntimeError('Migration upgrades must map host Names to explicit policies')
    for name,policy in upgrades.items():
        if not isinstance(name,str) or not name or not isinstance(policy,dict):raise RuntimeError('Invalid legacy host upgrade policy')
        if set(policy)-{'type','placement_mode','target'} or policy.get('type') not in ('App::Link','App::Part','Part::Feature','Mesh::Feature'):raise RuntimeError('Unsupported legacy host upgrade policy')
        if 'placement_mode' in policy and policy['placement_mode'] not in ('delta','instance'):raise RuntimeError('Legacy instance placement mode must be delta or instance')
        if 'target' in policy and (not isinstance(policy['target'],str) or not policy['target']):raise RuntimeError('Legacy upgrade target must be a definition host Name')
    return upgrades


def prepare(document,obj,reference,record,policy,definitions,rebuild,owner):
    import FreeCAD as App
    expected=reference.TypeId
    if record['capability']=='editable':expected='Mesh::Feature' if reference.isDerivedFrom('Mesh::Feature') else 'Part::Feature'
    if policy['type']!=expected:raise RuntimeError('Upgrade type differs from verified native representation: '+obj.Name)
    instance=record['class_name']=='ON_InstanceRef'
    if instance and 'placement_mode' not in policy:raise RuntimeError('Legacy instance upgrade requires an explicit placement mode')
    if not instance and 'placement_mode' in policy:raise RuntimeError('Geometry upgrade does not accept an instance placement mode')
    if 'target' in policy and expected!='App::Link':raise RuntimeError('Upgrade target is only supported for physical instance hosts; use migration targets for affine previews')
    if obj.TypeId==expected or (not instance and obj.isDerivedFrom(expected)):
        if 'target' in policy and getattr(getattr(obj,'LinkedObject',None),'Name',None)!=policy['target']:raise RuntimeError('Upgrade target conflicts with an already upgraded physical host')
        return None
    if obj.TypeId not in ('Part::Feature','Mesh::Feature','App::FeaturePython','App::Feature'):
        raise RuntimeError('Unsupported legacy host type for replacement: '+obj.TypeId)
    def scripted(host,view=False):
        proxy=getattr(host,'Proxy',None)
        # ViewProviderFeaturePython::onDocumentRestored uses integer 1 for the
        # default provider when no Python implementation was stored.
        return proxy is not None and not (view and type(proxy) is int and proxy==1)
    if scripted(obj) or scripted(obj.ViewObject,True) or getattr(obj,'ExpressionEngine',[]):raise RuntimeError('Legacy scripted or expression-driven host requires a dedicated upgrade adapter')
    if instance and not rebuild:raise RuntimeError('Legacy instance replacement requires explicit derived preview rebuild')
    placement=App.Placement(obj.Placement.toMatrix()) if hasattr(obj,'Placement') else App.Placement()
    target=None;shape=None;mesh=None
    if instance:
        # The verified native Placement is the rigid physical instance matrix.
        # Affine preview hosts use a separate rigid delta over immutable native A.
        if expected=='App::Link':
            if 'target' in policy:target=document.getObject(policy['target'])
            elif 'LinkedObject' in obj.PropertiesList and obj.LinkedObject is not None:target=obj.LinkedObject
            else:
                identity=getattr(obj,'OM9DefinitionTargetUUID',record['instance_definition_uuid'])
                candidates=[d for d in document.Objects if d.isDerivedFrom('App::Part') and getattr(d,'OM9NewDefinitionUUID','')==identity and getattr(d,'OM9ArchiveOwner',None) in (None,owner)]
                if len(candidates)>1:raise RuntimeError('Legacy instance upgrade target identity is ambiguous')
                target=definitions.get(identity) or (candidates[0] if candidates else None)
            if target is None:raise RuntimeError('Legacy instance upgrade has no verified definition target')
            if target.Document!=document or not target.isDerivedFrom('App::Part') or getattr(target,'OM9ArchiveOwner',None) not in (None,owner):raise RuntimeError('Legacy instance upgrade target belongs to another archive or document')
            if target not in definitions.values() and not (getattr(target,'OM9DefinitionUUID','') in definitions or 'OM9NewDefinitionUUID' in target.PropertiesList):raise RuntimeError('Legacy instance upgrade target has no verified definition provenance')
            if 'OM9NewDefinitionUUID' in target.PropertiesList:
                import ThreeDmArchiveState as state
                identity=state._new_identity(target,'Definition')
                manifest=json.loads(reference.OM9ArchiveOwner.OM9ArchiveManifest)
                if identity in {r['source_uuid'] for r in manifest['components']+manifest['records']} or len([d for d in document.Objects if getattr(d,'OM9NewDefinitionUUID','')==identity and getattr(d,'OM9ArchiveOwner',None) in (None,owner)])!=1:raise RuntimeError('Legacy upgrade target identity is colliding or ambiguous')
            if policy['placement_mode']=='delta':placement=placement.multiply(reference.Placement)
        elif policy['placement_mode']=='instance':
            native=list(reference.OM9InstanceMatrix)
            scale=json.loads(reference.OM9ArchiveOwner.OM9ArchiveManifest)['scale_mm']
            for index in (3,7,11):native[index]*=scale
            delta=placement.toMatrix()*App.Matrix(*native).inverse()
            # Placement would silently discard scale/shear. Reject before conversion.
            converted=App.Placement(delta).toMatrix()
            if any(abs(a-b)>1e-8 for a,b in zip(delta.A,converted.A)):raise RuntimeError('Legacy instance placement cannot represent a rigid delta over its native affine matrix')
            placement=App.Placement(delta)
    elif expected=='Part::Feature':
        if not hasattr(obj,'Shape') or obj.Shape.isNull() or not obj.Shape.isValid():raise RuntimeError('Legacy geometry upgrade requires valid current CAD payload; native snapshot cannot replace user geometry implicitly')
        shape=obj.Shape.copy()
        if not hasattr(obj,'Placement'):placement=App.Placement(shape.Placement.toMatrix())
        elif not shape.Placement.isIdentity() and any(abs(a-b)>1e-8 for a,b in zip(shape.Placement.toMatrix().A,placement.toMatrix().A)):raise RuntimeError('Legacy CAD holder has ambiguous independent shape and host placements')
    elif expected=='Mesh::Feature':
        if not hasattr(obj,'Mesh') or not obj.Mesh.CountFacets:raise RuntimeError('Legacy mesh upgrade requires a current mesh payload; implicit CAD tessellation is unsupported')
        mesh=obj.Mesh.copy()
    dynamic=[]
    for field in obj.PropertiesList:
        if field in ('Shape','Mesh','Placement','Proxy','ExpressionEngine','Group','LinkedObject'):continue
        status=obj.getPropertyStatus(field)
        # FreeCAD Property::PropDynamic is exposed as integer 21, not a name.
        if not field.startswith('OM9') and 21 not in status:continue
        kind=obj.getTypeIdOfProperty(field)
        if kind.startswith(('App::PropertyLink','App::PropertyXLink')):
            if kind not in LINK_TYPES:raise RuntimeError('Legacy upgrade has an unsupported reference property: '+field)
            value=_encode(getattr(obj,field),kind,document)
        elif kind not in VALUE_TYPES:raise RuntimeError('Legacy upgrade has an unsupported custom property: '+field)
        else:
            value=getattr(obj,field)
            if hasattr(value,'copy'):value=value.copy()
        attributes=sum(flag for bit,flag in {22:32,23:16,24:1,25:2,26:4,27:8,28:64}.items() if bit in status)
        runtime_status=[s for s in status if not isinstance(s,int) or s<21 or s>=29]
        dynamic.append((field,kind,value,obj.getEditorMode(field),obj.getGroupOfProperty(field),obj.getDocumentationOfProperty(field),attributes,runtime_status))
    view={p:getattr(obj.ViewObject,p) for p in ('Visibility','ShapeColor','LineColor','PointColor','Transparency') if hasattr(obj.ViewObject,p)}
    return dict(name=obj.Name,type=expected,label=obj.Label,placement=placement,target=target.Name if target else None,shape=shape,mesh=mesh,properties=dynamic,view=view)


LINK_TYPES={'App::PropertyLink','App::PropertyLinkHidden','App::PropertyLinkList','App::PropertyLinkListHidden','App::PropertyXLink'}
VALUE_TYPES={'App::PropertyString','App::PropertyStringList','App::PropertyBool','App::PropertyInteger','App::PropertyIntegerList','App::PropertyFloat','App::PropertyFloatList','App::PropertyVector','App::PropertyVectorList','App::PropertyPlacement','App::PropertyMatrix','App::PropertyColor','App::PropertyColorList'}


def _encode(value,kind,document):
    def one(obj):
        if obj is None:return None
        if not hasattr(obj,'Document') or obj.Document!=document:raise RuntimeError('Legacy upgrade cannot rewrite a cross-document or subelement reference')
        return obj.Name
    return [one(o) for o in value] if 'List' in kind else one(value)


def _decode(value,kind,document):
    def one(name):return document.getObject(name) if name is not None else None
    return [one(n) for n in value] if 'List' in kind else one(value)


def dependencies(document,replacements):
    names={p['name'] for p in replacements};answer=[]
    if not names:return answer
    for obj in document.Objects:
        for prop in obj.PropertiesList:
            kind=obj.getTypeIdOfProperty(prop)
            if not kind.startswith(('App::PropertyLink','App::PropertyXLink')):continue
            value=getattr(obj,prop)
            # LinkSub and extension/external properties need dedicated semantics.
            def touches(v):
                if hasattr(v,'Name'):return v.Document==document and v.Name in names
                return isinstance(v,(tuple,list)) and any(touches(i) for i in v)
            if not touches(value):continue
            if kind not in LINK_TYPES:raise RuntimeError('Legacy upgrade cannot rewrite this dependency: '+obj.Name+'.'+prop)
            if getattr(obj,'ExpressionEngine',[]):raise RuntimeError('Legacy upgrade cannot rewrite expression-driven dependencies')
            if obj.isDerivedFrom('App::Link') and (getattr(obj,'ElementCount',0) or getattr(obj,'LinkSub','')):raise RuntimeError('Legacy upgrade cannot rewrite array or subelement links')
            answer.append((obj.Name,prop,kind,_encode(value,kind,document)))
        # Expressions can depend on a shape without an App::PropertyLink edge.
        if getattr(obj,'ExpressionEngine',[]):raise RuntimeError('Legacy host upgrade requires a document without expression dependencies')
    return answer


def replace(document,replacements,incoming):
    result={}
    for saved in replacements:
        name=saved['name'];document.removeObject(name)
        obj=document.addObject(saved['type'],name)
        if obj.Name!=name:raise RuntimeError('Legacy upgrade could not retain its internal host identity')
        result[name]=obj;obj.Label=saved['label']
    for saved in replacements:
        obj=result[saved['name']]
        for field,kind,value,mode,group,documentation,attributes,status in saved['properties']:
            if field not in obj.PropertiesList:obj.addProperty(kind,field,group,documentation,attributes)
            if obj.getTypeIdOfProperty(field)!=kind:raise RuntimeError('Replacement property type conflict: '+field)
            setattr(obj,field,_decode(value,kind,document) if kind in LINK_TYPES else value);obj.setEditorMode(field,mode)
            if status:obj.setPropertyStatus(field,status)
        if saved['target']:obj.setLink(document.getObject(saved['target']))
        if saved['shape'] is not None:obj.Shape=saved['shape']
        if saved['mesh'] is not None:obj.Mesh=saved['mesh']
        obj.Placement=saved['placement']
        for field,value in saved['view'].items():
            if hasattr(obj.ViewObject,field):setattr(obj.ViewObject,field,value)
        if 'ShapeColor' in saved['view'] and not hasattr(obj.ViewObject,'ShapeColor'):
            if 'OM9Color' not in obj.PropertiesList:obj.addProperty('App::PropertyColor','OM9Color','Rhino source')
            kind=obj.getTypeIdOfProperty('OM9Color')
            if kind not in ('App::PropertyColor','App::PropertyFloatList'):raise RuntimeError('Replacement color provenance has an incompatible type')
            obj.OM9Color=list(saved['view']['ShapeColor'][:3]) if kind=='App::PropertyFloatList' else saved['view']['ShapeColor']
            if hasattr(obj.ViewObject,'OverrideMaterial') and hasattr(obj.ViewObject,'ShapeAppearance'):
                appearance=list(obj.ViewObject.ShapeAppearance)
                if appearance:
                    appearance[0].DiffuseColor=tuple(saved['view']['ShapeColor'][:3])
                    if 'Transparency' in saved['view']:appearance[0].Transparency=saved['view']['Transparency']/100.0
                    obj.ViewObject.ShapeAppearance=appearance;obj.ViewObject.OverrideMaterial=True
    for host,prop,kind,value in incoming:
        obj=document.getObject(host)
        if obj is None:raise RuntimeError('Legacy upgrade lost an incoming dependency host')
        if prop=='LinkedObject' and obj.isDerivedFrom('App::Link'):obj.setLink(_decode(value,kind,document))
        else:setattr(obj,prop,_decode(value,kind,document))
    return result


def geometry_signature(saved,state,record):
    """Fingerprint the future local payload without changing the live document."""
    from types import SimpleNamespace
    shape=saved['shape'].copy() if saved['shape'] is not None else None
    mesh=saved['mesh'].copy() if saved['mesh'] is not None else None
    if shape is not None:shape.Placement=saved['placement']
    if mesh is not None:mesh.transform(saved['placement'].toMatrix())
    candidate=SimpleNamespace(Shape=shape,Mesh=mesh,Placement=saved['placement'],isDerivedFrom=lambda kind:kind==saved['type'])
    return state._source_member_geometry_signature(candidate,record)
