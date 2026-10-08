"""FreeCAD binding for immutable native 3DM snapshots and scoped identities."""
import hashlib
import json
import os
import uuid

MAX_ARCHIVE = 512 * 1024 * 1024
MAX_MANIFEST = 32 * 1024 * 1024

def _hash(path):
    if os.path.getsize(path) > MAX_ARCHIVE:
        raise RuntimeError('Source archive exceeds 512 MiB')
    digest = hashlib.sha256()
    with open(path, 'rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            digest.update(chunk)
    return digest.hexdigest()

def _property(obj, kind, name, value):
    obj.addProperty('App::Property' + kind, name, 'Rhino source')
    setattr(obj, name, value)
    obj.setEditorMode(name, 1)

def archive_identity(obj):
    namespace = getattr(obj, 'OM9ImportNamespace', '')
    source = getattr(obj, 'OM9SourceUUID', '')
    return (namespace, source) if namespace and source else None

def _native_only(obj, record):
    # The immutable inventory describes the class adapter. A particular valid
    # source may lack an equivalent editable host representation.
    return record['capability']!='editable' or (getattr(obj,'OM9Capability','')=='retained' and bool(getattr(obj,'OM9RepresentationIssue','')))

def source_metadata(obj):
    view = obj.ViewObject
    color = getattr(view, 'ShapeColor', getattr(obj, 'OM9Color', (0.7, 0.7, 0.7)))
    return dict(name=obj.Label, layer=getattr(obj, 'OM9LayerPath', ''),
                color=[round(channel*255) for channel in color[:3]], visible=bool(view.Visibility),
                locked=bool(getattr(obj, 'OM9Locked', False)))

def source_placement(obj):
    if not hasattr(obj, 'Placement'):
        return None
    if hasattr(obj, 'getGlobalPlacement'):
        placement = obj.getGlobalPlacement()
    else:
        placement = obj.Placement
        for parent in obj.InList:
            if parent.isDerivedFrom('App::Part') and obj in parent.Group:
                placement = parent.getGlobalPlacement() * placement
                break
    matrix = placement.toMatrix()
    return list(matrix.A)

def _cad_fingerprint(shape):
    """Exact V1 CAD value fingerprint, excluding OCC's Checked cache bit."""
    import re
    brep=shape.copy().exportBrepToString()
    # IEEE signed zeros denote the same coordinate. Keep every nonzero digit.
    brep=re.sub(r'(?<!\S)-0(?!\S)','0',brep)
    header,topology=brep.split('TShapes ',1)
    # V1 TShape flags: Free, Modified, Checked, Orientable, Closed,
    # Infinite, Convex. Validation may update Checked without a CAD edit.
    topology=re.sub(r'^([01]{2})[01]([01]{4})$',r'\g<1>0\g<2>',topology,flags=re.MULTILINE)
    return hashlib.sha256((header+'TShapes '+topology).encode('utf-8')).hexdigest()

def source_signature(obj, name_override=None, metadata_override=None):
    """Hash actual host geometry and metadata without temporary file paths."""
    payload = dict(metadata=source_metadata(obj), placement=source_placement(obj))
    record=json.loads(getattr(obj,'OM9SourceRecord','{}'))
    if record.get('role')=='definition-member' and hasattr(obj,'Placement'):
        payload['placement']=list(obj.Placement.toMatrix().A)
    if obj.isDerivedFrom('App::Link'):
        scale=list(obj.ScaleVector)
        if scale!=[1.0,1.0,1.0]:payload['link_scale']=scale
        if obj.LinkTransform and obj.LinkedObject is not None and not obj.LinkedObject.Placement.isIdentity():
            payload['linked_definition_placement']=list(obj.LinkedObject.Placement.toMatrix().A)
    if name_override is not None:payload['metadata']['name']=name_override
    if metadata_override is not None:payload['metadata']=metadata_override
    if hasattr(obj, 'OM9InstanceMatrix'):
        payload['instance_matrix'] = list(obj.OM9InstanceMatrix)
    if record.get('class_name')=='ON_InstanceRef':
        # Derived preview topology is not the authoritative instance payload.
        # Its integrity is checked separately by preview_signature at export.
        payload['record']=getattr(obj,'OM9SourceRecord','')
        target=_definition_identity(_instance_definition(obj,record))
        if target!=record['instance_definition_uuid']:payload['instance_definition_uuid']=target
    elif obj.isDerivedFrom('Part::Feature'):
        if 'OM9SourceShapeBaseline' in obj.PropertiesList:
            # Both Part shape properties pass through the same FCStd writer and
            # reader. Compare their exact decoded CAD values after restoration;
            # never round coordinates or reset the baseline to an edited shape.
            current=_cad_fingerprint(obj.Shape)
            baseline=_cad_fingerprint(obj.OM9SourceShapeBaseline)
            payload['brep']=dict(source=obj.OM9SourceShapeToken,unchanged=current==baseline)
            if current!=baseline:payload['brep']['current']=current
        else:
            # Older projects keep their original exact signature contract.
            payload['brep'] = obj.Shape.copy().exportBrepToString()
    elif obj.isDerivedFrom('Mesh::Feature'):
        points, triangles = obj.Mesh.Topology
        payload['mesh'] = dict(vertices=[list(point) for point in points], faces=triangles)
    else:
        payload['record'] = getattr(obj, 'OM9SourceRecord', '')
        import ThreeDmTextDot,ThreeDmPointCloud,ThreeDmHatch,ThreeDmNativeFields
        ThreeDmNativeFields.adapter(obj)
        if ThreeDmTextDot.is_adapter(obj):payload['text_dot']=ThreeDmTextDot.fields(obj)
        if ThreeDmPointCloud.is_adapter(obj):payload['point_cloud']=ThreeDmPointCloud.signature(obj)
        if ThreeDmHatch.is_adapter(obj):payload['hatch_fields']=ThreeDmHatch.fields(obj)
        if obj.isDerivedFrom('App::Part') and hasattr(obj, 'OM9InstanceMatrix'):
            payload['preview'] = [source_signature(child) for child in obj.Group]
    encoded = json.dumps(payload, sort_keys=True, separators=(',', ':'), allow_nan=False).encode('utf-8')
    return hashlib.sha256(encoded).hexdigest()

def definition_signature(obj):
    payload = dict(name=obj.Label, placement=source_placement(obj),
                   members=[getattr(member, 'OM9SourceUUID', '') for member in obj.Group])
    proxy_changes={}
    for member in obj.Group:
        if getattr(member,'OM9DefinitionMemberProxy',False):
            matrix,metadata=_proxy_state(member)
            if matrix!=IDENTITY_MATRIX or metadata:proxy_changes[member.Name]=dict(matrix=matrix,metadata=metadata)
    if proxy_changes:payload['proxy_changes']=proxy_changes
    return hashlib.sha256(json.dumps(payload,sort_keys=True).encode()).hexdigest()

def preview_signature(obj, root=True, legacy=False):
    """Detect preview edits independently of the root's rigid placement delta."""
    import FreeCAD as App
    payload = {}
    if not root:
        payload.update(metadata=source_metadata(obj), placement=list(obj.Placement.toMatrix().A) if hasattr(obj,'Placement') else None)
    if obj.isDerivedFrom('Part::Feature'):
        import Part
        shape=Part.Shape(obj.Shape);shape.Placement=App.Placement();shape=shape.copy()
        brep=shape.exportBrepToString()
        if not legacy:
            # FCStd restore normalizes OCC directions, changing a few final
            # decimal digits without a geometry edit. Quantize only the derived
            # display fingerprint; source/native geometry remains untouched.
            import re
            def number(match):
                value=float(match.group())
                return format(value,'.12g') if value else '0'
            brep=re.sub(r'(?<!\S)[+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?(?!\S)',number,brep)
        payload['brep']=brep
    elif obj.isDerivedFrom('Mesh::Feature'):
        mesh=obj.Mesh.copy();mesh.transform(obj.Placement.inverse().toMatrix())
        points,faces=mesh.Topology
        payload['mesh']=dict(vertices=[list(p) for p in points],faces=faces)
    elif obj.isDerivedFrom('App::Part'):
        payload['children']=[preview_signature(child,False,legacy) for child in obj.Group]
    elif obj.isDerivedFrom('App::FeaturePython') and 'OM9PointCloudSchema' in obj.PropertiesList:
        import ThreeDmPointCloud
        payload['point_cloud']=ThreeDmPointCloud.signature(obj)
        payload['native_placement']=list(getattr(obj,'OM9BlockMemberPlacement',App.Placement()).toMatrix().A)
    else:
        raise RuntimeError('Instance has no editable rigid preview placement')
    return hashlib.sha256(json.dumps(payload,sort_keys=True,allow_nan=False).encode()).hexdigest()

def _preview_matches(obj):
    baseline=getattr(obj,'OM9PreviewSignature','')
    return baseline==preview_signature(obj) or baseline==preview_signature(obj,legacy=True)

def preservation_request(objects, staging):
    import FreeCAD as App, Part
    sources, selected, dependency_overlays, definition_overlays = {}, [], {}, {}
    new_geometry, new_members, new_member_ids, unsourced = [], {}, set(), []
    member_copies,copy_members,copy_ids=[],{},set()
    new_definitions,new_instances=[],[]
    new_reference_members={}
    new_definition_targets={}
    namespace_owners={}
    def bind_owner(owner):
        namespace=owner.OM9ImportNamespace
        if namespace in namespace_owners and namespace_owners[namespace]!=owner:
            raise RuntimeError('Different archive owners share one namespace; explicitly fork the copied archive namespace')
        namespace_owners[namespace]=owner
    def collect_dependencies(obj,owner,manifest,record):
        bind_owner(owner)
        namespace=owner.OM9ImportNamespace
        # Never silently restore changed/deleted block dependencies from snapshot.
        records={r['source_uuid']:r for r in manifest['records']+manifest['components']}
        geometry_records={r['source_uuid'] for r in manifest['records']}
        pending=_reference_dependencies(obj,record);seen=set()
        while pending:
            identity=pending.pop()
            if identity in seen:continue
            seen.add(identity)
            if (namespace,identity) in new_member_ids or (namespace,identity) in copy_ids:continue
            dependency=records.get(identity)
            if dependency is None:
                definition=_resolve_definition_target(obj.Document,owner,identity)
                if 'OM9NewDefinitionUUID' not in definition.PropertiesList:
                    raise RuntimeError('Missing current native dependency')
                new_definition_targets[(namespace,definition.Name)]=definition
                continue
            if dependency['class_name']=='ON_InstanceDefinition':
                definition=next((candidate for candidate in obj.Document.Objects if candidate.isDerivedFrom('App::Part') and getattr(candidate,'OM9DefinitionUUID','')==identity and getattr(candidate,'OM9ArchiveOwner',None)==owner),None)
                if definition is None:raise RuntimeError('Deleted block definition is still referenced by selected instances')
                members=[];copied=False
                for member_id in _definition_members(definition,owner,records,allow_new=True):
                    if member_id.startswith('copy:'):
                        member=definition.Document.getObject(member_id[5:]);key=(namespace,member.Name)
                        if key not in copy_members:
                            copy=_independent_member_copy(member,records[member.OM9SourceUUID],manifest,staging,len(member_copies))
                            member_copies.append(copy);copy_members[key]=copy['output_uuid'];copy_ids.add((namespace,copy['output_uuid']))
                        pending.extend(_reference_dependencies(member,records[member.OM9SourceUUID]));member_id=copy_members[key];copied=True
                    elif member_id.startswith('proxy:'):
                        proxy=definition.Document.getObject(member_id[6:]);canonical=proxy.LinkedObject
                        matrix,metadata=_proxy_state(proxy);member_id=canonical.OM9SourceUUID
                        independent=canonical.Name!=getattr(canonical,'OM9SourceHostID','')
                        if independent or matrix!=IDENTITY_MATRIX or metadata or member_id in members:
                            key=(namespace,proxy.Name)
                            if key not in copy_members:
                                if independent:
                                    copy=_independent_member_copy(canonical,records[member_id],manifest,staging,len(member_copies))
                                    if _native_current(canonical,records[member_id]):matrix=list((App.Matrix(*matrix)*App.Matrix(*copy['member_matrix'])).A)
                                    copy.update(host_id=proxy.Name,output_uuid=_source_copy_identity(namespace,member_id,proxy.Name),member_matrix=matrix);copy['metadata'].update(metadata)
                                else:
                                    copy=dict(namespace=namespace,host_id=proxy.Name,source_uuid=member_id,
                                        output_uuid=_source_copy_identity(namespace,member_id,proxy.Name),member_matrix=matrix)
                                    if metadata:copy['metadata']=metadata
                                member_copies.append(copy);copy_members[key]=copy['output_uuid'];copy_ids.add((namespace,copy['output_uuid']))
                            if independent:pending.extend(_reference_dependencies(canonical,records[member_id]))
                            else:pending.append(member_id)
                            member_id=copy_members[key];copied=True
                    elif member_id.startswith('new:'):
                        member=definition.Document.getObject(member_id[4:]);key=(namespace,member.Name)
                        if key not in new_members:
                            geometry=_stage_new_geometry(member,namespace,staging,len(new_geometry),'definition-member')
                            new_geometry.append(geometry);new_members[key]=(namespace,geometry['source_uuid'])
                            new_member_ids.add((namespace,geometry['source_uuid']))
                        member_id=new_members[key][1]
                    elif member_id.startswith('newref:'):
                        member=definition.Document.getObject(member_id[7:]);key=(namespace,member.Name)
                        member_id=_new_identity(member,'Instance')
                        new_reference_members[key]=member
                        new_member_ids.add((namespace,member_id))
                    members.append(member_id)
                pending.extend(members)
                if copied or definition_signature(definition)!=getattr(definition,'OM9DefinitionSignature',''):
                    definition_overlays[(namespace,identity)]=dict(host_id=definition.Name,namespace=namespace,
                        source_uuid=identity,name=definition.Label,member_uuids=members)
            elif dependency.get('role')=='definition-member':
                member=next((candidate for candidate in obj.Document.Objects if archive_identity(candidate)==(namespace,identity) and candidate.Name==getattr(candidate,'OM9SourceHostID','')),None)
                if member is None:raise RuntimeError('Deleted block member requires definition membership overlay support')
                pending.extend(_reference_dependencies(member,dependency))
                if dependency['class_name']=='ON_InstanceRef' or 'OM9BlockMemberPlacement' in member.PropertiesList or source_signature(member)!=member.OM9SourceSignature:
                    key=(namespace,identity)
                    if key not in dependency_overlays:
                        dependency_overlays[key]=_member_overlay(member,dependency,manifest,staging,len(dependency_overlays))
            elif identity in geometry_records and dependency.get('role')=='top-level':
                # A native curve owner is part of the current graph, even when
                # it was not selected. Never resurrect an edited/deleted owner
                # from the immutable archive and silently change semantics.
                member=next((candidate for candidate in obj.Document.Objects
                    if archive_identity(candidate)==(namespace,identity)
                    and candidate.Name==getattr(candidate,'OM9SourceHostID','')
                    and getattr(candidate,'OM9ArchiveOwner',None)==owner),None)
                if member is None:raise RuntimeError('Deleted native reference owner is still required by selected geometry')
                baseline=getattr(member,'OM9SourceSignature','')
                metadata=json.loads(member.OM9SourceMetadata)
                if not baseline or source_signature(member,metadata_override=metadata)!=baseline:
                    raise RuntimeError('Changed native reference owner requires a verified topology/domain-preserving overlay')
                current=source_metadata(member)
                changes={key:current[key] for key in current if current[key]!=metadata[key]}
                if changes and member not in objects:dependency_overlays[(namespace,identity)]=dict(host_id=member.Name,namespace=namespace,
                    source_uuid=identity,action='unchanged',metadata=changes)
                pending.extend(_reference_dependencies(member,dependency))
            else:pending.extend(dependency['dependencies'])

    new_roots=[]
    for index, obj in enumerate(objects):
        if _is_new_instance(obj):
            new_roots.append(obj)
            continue
        owner = getattr(obj, 'OM9ArchiveOwner', None)
        if owner is None and _is_new_geometry(obj):
            unsourced.append(obj)
            continue
        proxy=getattr(obj,'OM9DefinitionMemberProxy',False)
        if proxy or (owner is not None and archive_identity(obj) and json.loads(getattr(obj,'OM9SourceRecord','{}')).get('role')=='definition-member'):
            owner,loaded,record=_source_proxy_record(obj) if proxy else _source_block_record(obj)
            bind_owner(owner);namespace=owner.OM9ImportNamespace;manifest=loaded['manifest']
            sources[namespace]=dict(namespace=namespace,snapshot=loaded['snapshot'],archive_sha256=manifest['archive_sha256'],scale_mm=manifest['scale_mm'])
            copy=_selected_member_copy(obj,record,manifest,staging,len(member_copies))
            member_copies.append(copy)
            selected.append(dict(host_id='selected-root:'+obj.Name,namespace=namespace,source_uuid=copy['output_uuid'],action='unchanged'))
            collect_dependencies(obj.LinkedObject if proxy else obj,owner,manifest,record)
            continue
        if owner is None or not archive_identity(obj) or getattr(obj, 'OM9DefinitionMemberProxy', False):
            raise RuntimeError('Select supported whole geometry with valid archive ownership')
        namespace, source = archive_identity(obj)
        if namespace != owner.OM9ImportNamespace:raise RuntimeError('Source namespace and owner differ')
        bind_owner(owner)
        loaded = load_archive_state(owner)
        manifest = loaded['manifest']
        record = next((row for row in manifest['records'] if row['source_uuid']==source), None)
        if record is None or record['role']!='top-level':raise RuntimeError('Select source top-level objects')
        sources[namespace] = dict(namespace=namespace,snapshot=loaded['snapshot'],archive_sha256=manifest['archive_sha256'],scale_mm=manifest['scale_mm'])
        baseline = getattr(obj,'OM9SourceSignature','')
        if not baseline:raise RuntimeError('Source object has no verified edit baseline')
        current_metadata=source_metadata(obj);baseline_metadata=json.loads(obj.OM9SourceMetadata)
        changed = source_signature(obj,metadata_override=baseline_metadata)!=baseline
        duplicate = obj.Name!=getattr(obj,'OM9SourceHostID',obj.Name)
        row = dict(host_id=obj.Name,namespace=namespace,source_uuid=source,action='unchanged')
        metadata={key:current_metadata[key] for key in current_metadata if current_metadata[key]!=baseline_metadata[key]}
        if metadata:row['metadata']=metadata
        if record['class_name']=='ON_InstanceRef':
            matrix=_instance_overlay_matrix(obj,record,manifest,world=True)
            definition=_instance_definition(obj,record)
            if changed or (not obj.isDerivedFrom('App::Link') and not definition.Placement.isIdentity()):
                row.update(action='instance',instance_matrix=matrix)
                target=_definition_identity(definition)
                if target!=record['instance_definition_uuid']:row['instance_definition_uuid']=target
        elif _native_only(obj,record):
            import ThreeDmNativeFields
            adapter=ThreeDmNativeFields.adapter(obj,record)
            if changed and adapter is None:raise RuntimeError('Retained geometry cannot be replaced')
            if hasattr(obj,'Placement'):raise RuntimeError('Retained native geometry requires its explicit native placement property')
            placement=_member_world_placement(obj)
            if not placement.isIdentity():row.update(action='transform',geometry_matrix=list(placement.toMatrix().A))
            if adapter is not None and adapter.changed(obj,record,manifest):
                row.update(action='transform',geometry_matrix=list(placement.toMatrix().A))
                row.update(ThreeDmNativeFields.stage(obj,record,staging,index))
        elif changed:
            if record['capability']!='editable':raise RuntimeError('Retained geometry cannot be replaced')
            row['action']='replace'
            if obj.isDerivedFrom('Mesh::Feature'):
                points,faces=obj.Mesh.Topology
                matrix=(obj.getGlobalPlacement()*obj.Placement.inverse()).toMatrix()
                row.update(vertices=[list(matrix.multVec(point)) for point in points],faces=[list(face) for face in faces])
            elif obj.isDerivedFrom('Part::Feature'):
                shape=Part.getShape(obj,'',needSubElement=False,transform=True)
                if shape is None or shape.isNull():raise RuntimeError('Invalid selected source geometry')
                matrix=(obj.getGlobalPlacement()*obj.Placement.inverse()).toMatrix()
                shape.transformShape(matrix,False)
                filename=os.path.join(staging,str(index)+'.brep');shape.exportBrep(filename);row['brep']=filename
            else:raise RuntimeError('Unsupported edited source object')
        if duplicate:
            row['duplicate_action']=row['action']
            row['action']='duplicate'
        collect_dependencies(obj,owner,manifest,record)
        selected.append(row)
    document_namespace=next(iter(sources),str(uuid.uuid4()))
    def source_target(reference,definition):
        owner=getattr(definition,'OM9ArchiveOwner',None) if definition is not None else getattr(reference,'OM9ArchiveOwner',None)
        if owner is None:raise RuntimeError('Imported block definition has no archive ownership')
        bind_owner(owner)
        loaded=load_archive_state(owner);manifest=loaded['manifest'];namespace=owner.OM9ImportNamespace
        sources[namespace]=dict(namespace=namespace,snapshot=loaded['snapshot'],archive_sha256=manifest['archive_sha256'],scale_mm=manifest['scale_mm'])
        if definition is None:return
        target=definition.OM9DefinitionUUID
        collect_dependencies(reference,owner,manifest,dict(class_name='ON_InstanceRef',instance_definition_uuid=target,dependencies=[target]))
    def source_member(member,namespace):
        proxy=getattr(member,'OM9DefinitionMemberProxy',False)
        canonical=member.LinkedObject if proxy else member
        owner,loaded,record=_source_proxy_record(member) if proxy else _source_block_record(member)
        if owner.OM9ImportNamespace!=namespace:raise RuntimeError('Source block member belongs to another archive namespace')
        bind_owner(owner)
        manifest=loaded['manifest'];sources[namespace]=dict(namespace=namespace,snapshot=loaded['snapshot'],archive_sha256=manifest['archive_sha256'],scale_mm=manifest['scale_mm'])
        key=(namespace,member.Name)
        if key not in copy_members:
            copy=_source_proxy_member_copy(member,record,manifest,staging,len(member_copies)) if proxy else _source_block_member_copy(member,record,manifest,staging,len(member_copies))
            member_copies.append(copy);copy_members[key]=copy['output_uuid'];copy_ids.add((namespace,copy['output_uuid']))
        collect_dependencies(canonical,owner,manifest,record)
        return copy_members[key]
    _stage_new_blocks(new_roots,document_namespace,staging,new_geometry,new_definitions,new_instances,source_target,new_reference_members,new_definition_targets,source_member)
    for obj in unsourced:
        row=_stage_new_geometry(obj,document_namespace,staging,len(new_geometry),'top-level')
        if any(any(hasattr(parent,field) for field in ('OM9DefinitionUUID','OM9NewDefinitionUUID')) for parent in obj.InList):
            row['source_uuid']=_source_copy_identity(row['source_uuid'],row['source_uuid'],'selected:'+obj.Name)
            row['host_id']='selected-new:'+obj.Name
        new_geometry.append(row)
    return dict(schema_version=1,document_namespace=document_namespace,sources=list(sources.values()),selected=selected,
                dependency_overlays=list(dependency_overlays.values()),
                definition_overlays=list(definition_overlays.values()),new_geometry=new_geometry,member_copies=member_copies,
                new_definitions=new_definitions,new_instances=new_instances)

def _is_new_instance(obj):
    return obj is not None and obj.isDerivedFrom('App::Link') and 'OM9NewInstanceUUID' in obj.PropertiesList

def _new_identity(obj, kind):
    value=getattr(obj,'OM9New'+kind+'UUID')
    try:identity=uuid.UUID(value)
    except (ValueError,TypeError,AttributeError):raise RuntimeError('Invalid new block identity')
    if identity.int==0 or str(identity)!=value:raise RuntimeError('Invalid new block identity')
    # FreeCAD copies custom properties. Scope a copied host identity separately
    # without mutating either original or copy during an export.
    host=getattr(obj,'OM9New'+kind+'HostID','')
    if not host:raise RuntimeError('New block identity has no original host binding')
    return value if obj.Name==host else str(uuid.uuid5(identity,obj.Name))

def _bind_new_identity(obj, kind):
    _property(obj,'String','OM9New'+kind+'UUID',str(uuid.uuid4()))
    _property(obj,'String','OM9New'+kind+'HostID',obj.Name)

def _validate_creation(document):
    import FreeCAD as App, OpenMatrix9Gui as native
    if document is None or document!=App.ActiveDocument:raise RuntimeError('Block creation requires the active project')
    native.validateDocument(document.Name)

def _source_block_record(member):
    owner=getattr(member,'OM9ArchiveOwner',None);identity=archive_identity(member)
    if getattr(member,'OM9DefinitionMemberProxy',False) or owner is None or not identity or identity[0]!=owner.OM9ImportNamespace:
        raise RuntimeError('Source block member has no unambiguous archive ownership')
    loaded=load_archive_state(owner)
    record=next((r for r in loaded['manifest']['records'] if r['source_uuid']==identity[1]),None)
    try:stored=json.loads(member.OM9SourceRecord)
    except (ValueError,AttributeError):raise RuntimeError('Source block member has no verified native record')
    if record is None or stored!=record or record['capability']=='incompatible':raise RuntimeError('Source block member record differs from verified archive')
    if not getattr(member,'OM9SourceSignature','') or not hasattr(member,'OM9SourceMetadata'):raise RuntimeError('Source block member requires verified provenance; explicit legacy migration is needed')
    return owner,loaded,record

def _source_proxy_record(proxy):
    """Resolve a verified shared member without giving its wrapper source identity."""
    import math, FreeCAD as App
    if not getattr(proxy,'OM9DefinitionMemberProxy',False) or not proxy.isDerivedFrom('App::Link'):
        raise RuntimeError('Invalid source member proxy')
    canonical=proxy.LinkedObject
    if canonical is None or canonical.Document!=proxy.Document or getattr(canonical,'OM9DefinitionMemberProxy',False):
        raise RuntimeError('Source member proxy has no unambiguous canonical payload')
    owner,loaded,record=_source_block_record(canonical)
    if record['role']!='definition-member':raise RuntimeError('Source proxy target is not a native definition member')
    matrix,metadata=_proxy_state(proxy)
    determinant=App.Matrix(*matrix).determinant()
    if not all(math.isfinite(v) for v in matrix) or not math.isfinite(determinant) or determinant==0:
        raise RuntimeError('Source member proxy has a singular or invalid transform')
    if record['class_name']=='ON_InstanceRef':_instance_overlay_matrix(canonical,record,loaded['manifest'])
    elif record['capability']!='editable' and not _native_current(canonical,record) and source_signature(canonical,metadata_override=json.loads(canonical.OM9SourceMetadata))!=canonical.OM9SourceSignature:
        raise RuntimeError('Retained proxy payload was modified')
    elif record['capability']=='editable':
        if canonical.isDerivedFrom('Part::Feature') and (canonical.Shape.isNull() or not canonical.Shape.isValid()):raise RuntimeError('Invalid canonical proxy CAD geometry')
        if canonical.isDerivedFrom('Mesh::Feature') and not canonical.Mesh.CountFacets:raise RuntimeError('Invalid empty canonical proxy mesh')
    return owner,loaded,record

def _source_proxy_member_copy(proxy,record,manifest,staging,index):
    if 'OM9NewMemberUUID' not in proxy.PropertiesList:raise RuntimeError('Source proxy has no explicit creation identity')
    canonical=proxy.LinkedObject;matrix,metadata=_proxy_state(proxy)
    unchanged=source_signature(canonical,metadata_override=json.loads(canonical.OM9SourceMetadata))==canonical.OM9SourceSignature
    if record['class_name']=='ON_InstanceRef' or _native_current(canonical,record) or (record['capability']=='editable' and not unchanged):
        copy=_independent_member_copy(canonical,record,manifest,staging,index)
    else:
        copy=dict(namespace=canonical.OM9ImportNamespace,source_uuid=canonical.OM9SourceUUID,
                  follow_canonical=False,metadata=source_metadata(canonical))
    if _native_current(canonical,record):
        import FreeCAD as App
        matrix=list((App.Matrix(*matrix)*App.Matrix(*copy['member_matrix'])).A)
    elif _native_only(canonical,record) and record['class_name']!='ON_InstanceRef':
        # Source-retained native classes without a field adapter still own an
        # explicit local placement. Promoting their proxy must compose it just
        # as selected-member export does, rather than copying the baseline alone.
        import FreeCAD as App
        placement=getattr(canonical,'OM9BlockMemberPlacement',App.Placement())
        if not isinstance(placement,App.Placement):raise RuntimeError('Retained native placement must be a FreeCAD Placement')
        matrix=list((App.Matrix(*matrix)*placement.toMatrix()).A)
    copy.update(host_id='new-proxy:'+proxy.Name,output_uuid=_new_identity(proxy,'Member'),member_matrix=matrix)
    copy['metadata'].update(metadata)
    return copy

def _source_member_geometry_signature(member, record=None):
    import FreeCAD as App, Part
    if record is None:record=json.loads(member.OM9SourceRecord)
    if record['class_name']=='ON_InstanceRef':payload=dict(record=record)
    elif member.isDerivedFrom('Part::Feature'):
        shape=Part.Shape(member.Shape);shape.Placement=App.Placement();payload=dict(brep=shape.copy().exportBrepToString())
    elif member.isDerivedFrom('Mesh::Feature'):
        mesh=member.Mesh.copy();mesh.transform(member.Placement.inverse().toMatrix());points,faces=mesh.Topology
        payload=dict(vertices=[list(p) for p in points],faces=[list(f) for f in faces])
    else:
        payload=dict(record=record)
        if _native_current(member,record):
            import ThreeDmTextDot,ThreeDmPointCloud,ThreeDmHatch
            if ThreeDmTextDot.is_adapter(member):payload['text_dot']=ThreeDmTextDot.fields(member,record)
            elif ThreeDmHatch.is_adapter(member):payload['hatch_fields']=ThreeDmHatch.fields(member,record)
            else:payload['point_cloud']=ThreeDmPointCloud.signature(member)
    return hashlib.sha256(json.dumps(payload,sort_keys=True,allow_nan=False).encode()).hexdigest()

def _member_world_placement(member):
    import FreeCAD as App
    if hasattr(member,'Placement'):return App.Placement(App.Matrix(*source_placement(member)))
    placement=getattr(member,'OM9BlockMemberPlacement',App.Placement())
    if not isinstance(placement,App.Placement):raise RuntimeError('Retained native placement must be a FreeCAD Placement')
    for parent in member.InList:
        if parent.isDerivedFrom('App::Part') and member in parent.Group:return parent.getGlobalPlacement()*placement
    return placement

def _source_block_member_copy(member,record,manifest,staging,index):
    import FreeCAD as App
    if 'OM9NewMemberUUID' not in member.PropertiesList:raise RuntimeError('Source member has no explicit creation identity')
    copy=dict(namespace=member.OM9ImportNamespace,host_id='new-member:'+member.Name,source_uuid=member.OM9SourceUUID,
              output_uuid=_new_identity(member,'Member'),member_matrix=IDENTITY_MATRIX)
    old=json.loads(member.OM9SourceMetadata);current=source_metadata(member)
    metadata={key:current[key] for key in current if current[key]!=old[key]}
    if metadata:copy['metadata']=metadata
    if _native_current(member,record):
        own=_independent_member_copy(member,record,manifest,staging,index)
        copy['member_matrix']=own['member_matrix']
        if 'independent_overlay' in own:copy['independent_overlay']=own['independent_overlay']
        else:copy['follow_canonical']=False
    elif record['class_name']=='ON_InstanceRef':
        own=_independent_member_copy(member,record,manifest,staging,index);copy['independent_overlay']=own['independent_overlay']
    elif member.OM9BlockMemberNativeGeometry and _source_member_geometry_signature(member)==member.OM9BlockMemberGeometrySignature:
        placement=member.Placement if hasattr(member,'Placement') else member.OM9BlockMemberPlacement
        initial=App.Matrix(*json.loads(member.OM9BlockMemberSourcePlacement))
        copy.update(follow_canonical=False,member_matrix=list((placement.toMatrix()*initial.inverse()).A))
    elif record['capability']=='editable':
        own=_independent_member_copy(member,record,manifest,staging,index);copy['independent_overlay']=own['independent_overlay']
    else:raise RuntimeError('Retained source block member geometry cannot be replaced')
    return copy

def create_definition(document, members, name):
    """Create a native reusable definition from new or verified source members."""
    import FreeCAD as App
    _validate_creation(document);members=list(members)
    if not isinstance(name,str) or not name.strip() or len(name)>1024:raise RuntimeError('Invalid block definition name')
    if len(set(members))!=len(members):raise RuntimeError('Duplicate block member')
    owners=set();source_members={}
    for member in members:
        if member.Document!=document:raise RuntimeError('Block member belongs to another project')
        if getattr(member,'OM9DefinitionMemberProxy',False):
            owner,loaded,record=_source_proxy_record(member);owners.add(owner)
            source_members[member.Name]=(record,False)
            continue
        if not (_is_new_geometry(member) or _is_new_instance(member)):
            owner,loaded,record=_source_block_record(member);owners.add(owner)
            native_unchanged=source_signature(member,metadata_override=json.loads(member.OM9SourceMetadata))==member.OM9SourceSignature
            if record['class_name']=='ON_InstanceRef':_instance_overlay_matrix(member,record,loaded['manifest'])
            elif record['capability']!='editable' and not _native_current(member,record) and not native_unchanged:raise RuntimeError('Retained source member was modified')
            source_members[member.Name]=(record,native_unchanged)
        if member.isDerivedFrom('Part::Feature') and (member.Name not in source_members or source_members[member.Name][0]['capability']=='editable') and (member.Shape.isNull() or not member.Shape.isValid()):
            raise RuntimeError('New definition member has invalid CAD geometry')
        if _is_new_instance(member):
            target=member.LinkedObject
            if target is None or target.Document!=document or not target.isDerivedFrom('App::Part'):raise RuntimeError('Invalid new block member target')
            owner=getattr(target,'OM9ArchiveOwner',None)
            if owner is not None:owners.add(owner)
    if len(owners)>1:raise RuntimeError('New definition cannot combine foreign archive namespaces')
    for candidate in document.Objects:
        if candidate.isDerivedFrom('App::Part') and any(hasattr(candidate,key) for key in ('OM9DefinitionUUID','OM9NewDefinitionUUID')) and candidate.Label.casefold()==name.casefold():
            raise RuntimeError('Block definition name collision')
    placements={member.Name:_member_world_placement(member) for member in members}
    document.openTransaction('Create Rhino block definition')
    try:
        definition=document.addObject('App::Part','RhinoNewBlockDefinition');definition.Label=name
        _bind_new_identity(definition,'Definition')
        _property(definition,'LinkHidden','OM9ArchiveOwner',next(iter(owners),None))
        for member in members:
            if _is_new_geometry(member) and not hasattr(member,'OM9NewGeometryUUID'):_bind_new_identity(member,'Geometry')
            if member.Name in source_members and 'OM9NewMemberUUID' not in member.PropertiesList:
                _bind_new_identity(member,'Member')
                if not getattr(member,'OM9DefinitionMemberProxy',False):
                    _property(member,'Bool','OM9BlockMemberNativeGeometry',source_members[member.Name][1])
                    _property(member,'String','OM9BlockMemberGeometrySignature',_source_member_geometry_signature(member))
                    _property(member,'String','OM9BlockMemberSourcePlacement',json.dumps(list(member.Placement.toMatrix().A) if hasattr(member,'Placement') else IDENTITY_MATRIX))
                    if not hasattr(member,'Placement') and 'OM9BlockMemberPlacement' not in member.PropertiesList:
                        member.addProperty('App::PropertyPlacement','OM9BlockMemberPlacement','Rhino block member')
            definition.addObject(member)
            if hasattr(member,'Placement'):member.Placement=placements[member.Name]
            else:member.OM9BlockMemberPlacement=placements[member.Name]
        definition.ViewObject.Visibility=False;document.recompute();document.commitTransaction();return definition
    except Exception:
        document.abortTransaction();raise

def copy_definition(definition, name=None, copy_targets=True):
    """Copy an explicit native definition family in one Undo transaction.

    Geometry/member identities are independent; shared nested targets are copied
    once when requested. Source snapshots/records remain verified provenance.
    """
    import FreeCAD as App
    if definition is None:raise RuntimeError('Missing block definition to copy')
    document=definition.Document;_validate_creation(document)
    if type(copy_targets) is not bool:raise RuntimeError('Block target copy policy must be boolean')
    owners=set();verified={};targets={};visited=set()
    def validate(current,path):
        if current.Document!=document or not current.isDerivedFrom('App::Part') or not any(key in current.PropertiesList for key in ('OM9DefinitionUUID','OM9NewDefinitionUUID')):
            raise RuntimeError('Copy requires an explicit definition in the active project')
        if current.Name in path or len(path)>=64:raise RuntimeError('Cyclic or excessive copied block nesting')
        if current.Name in visited:return
        if len(visited)>=1000000:raise RuntimeError('Copied block object limit exceeded')
        visited.add(current.Name)
        owner=getattr(current,'OM9ArchiveOwner',None)
        source_id=getattr(current,'OM9CopiedDefinitionSourceUUID','') or (getattr(current,'OM9DefinitionUUID','') if 'OM9NewDefinitionUUID' not in current.PropertiesList else '')
        if owner is not None:
            loaded=load_archive_state(owner);owners.add(owner)
            if source_id and not any(r['source_uuid']==source_id and r['class_name']=='ON_InstanceDefinition' for r in loaded['manifest']['components']):raise RuntimeError('Copied definition has no verified native provenance')
        elif source_id:raise RuntimeError('Copied source definition has no archive ownership')
        verified[current.Name]=(owner,source_id)
        for member in current.Group:
            if member.Document!=document:raise RuntimeError('Foreign copied definition member')
            record=None;payload=member.LinkedObject if getattr(member,'OM9DefinitionMemberProxy',False) else member
            if getattr(member,'OM9DefinitionMemberProxy',False) or archive_identity(member):
                member_owner,loaded,record=_source_proxy_record(member) if payload!=member else _source_block_record(member);owners.add(member_owner)
                if record['class_name']=='ON_InstanceRef':target=_instance_definition(payload,record);_instance_overlay_matrix(payload,record,loaded['manifest'])
                elif record['capability']!='editable' and not _native_current(payload,record) and source_signature(payload,metadata_override=json.loads(payload.OM9SourceMetadata))!=payload.OM9SourceSignature:raise RuntimeError('Retained source member was modified')
            elif _is_new_instance(member):target=member.LinkedObject
            elif not _is_new_geometry(member):raise RuntimeError('Unsupported copied definition member')
            if _is_new_instance(member) or (record is not None and record['class_name']=='ON_InstanceRef'):
                if target is None or target.Document!=document or not target.isDerivedFrom('App::Part'):raise RuntimeError('Missing copied block target')
                targets[member.Name]=target
                target_owner=getattr(target,'OM9ArchiveOwner',None)
                if target_owner is not None:owners.add(target_owner)
                if copy_targets:validate(target,path+(current.Name,))
            elif payload.isDerivedFrom('Part::Feature') and (payload.Shape.isNull() or not payload.Shape.isValid()):raise RuntimeError('Invalid copied CAD member')
            elif payload.isDerivedFrom('Mesh::Feature') and not payload.Mesh.CountFacets:raise RuntimeError('Invalid empty copied mesh member')
    validate(definition,())
    if len(owners)>1:raise RuntimeError('Copied definition family cannot combine foreign archive namespaces')
    occupied={o.Label.casefold() for o in document.Objects if o.isDerivedFrom('App::Part') and any(key in o.PropertiesList for key in ('OM9DefinitionUUID','OM9NewDefinitionUUID'))}
    for owner in owners:occupied.update(r['name'].casefold() for r in load_archive_state(owner)['manifest']['components'] if r['class_name']=='ON_InstanceDefinition')
    def unique_label(label):
        candidate=label+' (copy)';suffix=2
        while candidate.casefold() in occupied:candidate=label+' (copy '+str(suffix)+')';suffix+=1
        if len(candidate)>1024:raise RuntimeError('Copied definition name too long')
        occupied.add(candidate.casefold());return candidate
    if name is None:name=unique_label(definition.Label)
    elif not isinstance(name,str) or not name.strip() or len(name)>1024 or name.casefold() in occupied:raise RuntimeError('Invalid or colliding copied definition name')
    else:occupied.add(name.casefold())
    labels={definition.Name:name}
    for key in visited:
        if key not in labels:labels[key]=unique_label(document.getObject(key).Label)
    def fresh_identity(obj,kind):
        for field,value in [('OM9New'+kind+'UUID',str(uuid.uuid4())),('OM9New'+kind+'HostID',obj.Name)]:
            if field in obj.PropertiesList:setattr(obj,field,value)
            else:_property(obj,'String',field,value)
    copies={};members={};payload_copies={}
    def preview_copy(obj):
        copied=document.copyObject(obj,False)
        # Copying creates a distinct internal Name, but generated display
        # child labels participate in the verified preview fingerprint.
        copied.Label=obj.Label
        if obj.isDerivedFrom('App::Part'):copied.Group=[preview_copy(child) for child in obj.Group]
        return copied
    def copied_preview_matches(original,copied):
        def local(obj):return list(getattr(obj,'Placement',getattr(obj,'OM9BlockMemberPlacement',App.Placement())).toMatrix().A)
        if original.TypeId!=copied.TypeId or local(original)!=local(copied):return False
        old=source_metadata(original);new=source_metadata(copied);old.pop('name');new.pop('name')
        if old!=new:return False
        if original.isDerivedFrom('App::Part'):
            return len(original.Group)==len(copied.Group) and all(copied_preview_matches(a,b) for a,b in zip(original.Group,copied.Group))
        return preview_signature(original)==preview_signature(copied)
    def source_payload_copy(member):
        if member.Name in payload_copies:return payload_copies[member.Name]
        child=document.copyObject(member,False)
        payload_copies[member.Name]=child
        owner,loaded,record=_source_block_record(member);fresh_identity(child,'Member')
        if 'OM9BlockMemberNativeGeometry' not in child.PropertiesList:
            _property(child,'Bool','OM9BlockMemberNativeGeometry',source_signature(member,metadata_override=json.loads(member.OM9SourceMetadata))==member.OM9SourceSignature)
            _property(child,'String','OM9BlockMemberGeometrySignature',_source_member_geometry_signature(member))
            _property(child,'String','OM9BlockMemberSourcePlacement',json.dumps(list(member.Placement.toMatrix().A) if hasattr(member,'Placement') else IDENTITY_MATRIX))
        if not hasattr(child,'Placement') and 'OM9BlockMemberPlacement' not in child.PropertiesList:child.addProperty('App::PropertyPlacement','OM9BlockMemberPlacement','Rhino block member')
        if member.isDerivedFrom('App::Part'):
            child.Group=[preview_copy(o) for o in member.Group]
            if not copied_preview_matches(member,child):raise RuntimeError('Copied affine display payload differs from its verified source')
            child.OM9PreviewSignature=preview_signature(child)
        return child
    def clone(current):
        if current.Name in copies:return copies[current.Name]
        copied=document.addObject('App::Part','RhinoCopiedBlockDefinition');copied.Label=labels[current.Name];copied.Placement=App.Placement(current.Placement.toMatrix());fresh_identity(copied,'Definition');copies[current.Name]=copied
        owner,source_id=verified[current.Name];owner=owner or next(iter(owners),None)
        _property(copied,'LinkHidden','OM9ArchiveOwner',owner)
        if source_id:_property(copied,'String','OM9CopiedDefinitionSourceUUID',source_id)
        for member in current.Group:
            if member.Name not in members:
                if getattr(member,'OM9DefinitionMemberProxy',False):
                    child=document.copyObject(member,False);payload=source_payload_copy(member.LinkedObject);child.setLink(payload);fresh_identity(child,'Member')
                elif archive_identity(member):child=source_payload_copy(member);payload=child
                else:
                    child=document.copyObject(member,False);payload=child
                    fresh_identity(child,'Instance' if _is_new_instance(member) else 'Geometry')
                members[member.Name]=child
                if member.Name in targets and copy_targets:
                    target=clone(targets[member.Name])
                    if payload.isDerivedFrom('App::Link'):payload.setLink(target)
                    else:payload.OM9DefinitionTargetUUID=_definition_identity(target)
            copied.addObject(members[member.Name])
        copied.ViewObject.Visibility=False;return copied
    document.openTransaction('Copy Rhino block definition')
    try:copied=clone(definition);document.recompute();document.commitTransaction();return copied
    except Exception:document.abortTransaction();raise


def create_instance(definition, placement=None, name='Rhino block instance'):
    """Place a new native reference to a new or verified imported definition."""
    import FreeCAD as App
    if definition is None:raise RuntimeError('Missing block definition')
    document=definition.Document;_validate_creation(document)
    if not definition.isDerivedFrom('App::Part') or not any(hasattr(definition,key) for key in ('OM9DefinitionUUID','OM9NewDefinitionUUID')):raise RuntimeError('Target must be an explicit block definition')
    if not isinstance(name,str) or not name:raise RuntimeError('Invalid block instance name')
    if placement is not None and not isinstance(placement,App.Placement):raise RuntimeError('Block placement must be a FreeCAD Placement')
    owner=getattr(definition,'OM9ArchiveOwner',None)
    if hasattr(definition,'OM9DefinitionUUID'):
        if owner is None:raise RuntimeError('Imported definition has no verified archive ownership')
        manifest=load_archive_state(owner)['manifest']
        if not any(r['source_uuid']==definition.OM9DefinitionUUID and r['class_name']=='ON_InstanceDefinition' for r in manifest['components']):raise RuntimeError('Imported definition is not present in its source archive')
    document.openTransaction('Create Rhino block instance')
    try:
        instance=document.addObject('App::Link','RhinoNewBlockInstance');instance.setLink(definition);instance.LinkTransform=True
        instance.Label=name;instance.Placement=placement or App.Placement()
        _bind_new_identity(instance,'Instance');_property(instance,'LinkHidden','OM9ArchiveOwner',owner)
        document.recompute();document.commitTransaction();return instance
    except Exception:
        document.abortTransaction();raise

def _stage_new_blocks(roots, default_namespace, staging, geometry, definitions, instances, source_target, member_roots=None, definition_roots=None, source_member=None):
    import FreeCAD as App
    visited={};staged_geometry={};staged_instances={}
    def reference(obj,namespace,role,path):
        if not _is_new_instance(obj):raise RuntimeError('Unsupported new native block reference')
        target=obj.LinkedObject
        if target is None or target.Document!=obj.Document or not target.isDerivedFrom('App::Part'):raise RuntimeError('Missing new instance definition target')
        owner=getattr(target,'OM9ArchiveOwner',None)
        if owner is not None and owner.OM9ImportNamespace!=namespace:raise RuntimeError('New block target belongs to another archive namespace')
        bound_owner=getattr(obj,'OM9ArchiveOwner',None)
        if bound_owner is not None and bound_owner.OM9ImportNamespace!=namespace:raise RuntimeError('New reference belongs to another archive namespace')
        if bound_owner is not None and owner is None:source_target(obj,None)
        if hasattr(target,'OM9NewDefinitionUUID'):
            if owner is not None:source_target(target,None)
            identity=definition(target,namespace,path)
        elif hasattr(target,'OM9DefinitionUUID'):
            identity=_instance_definition(obj,dict(instance_definition_uuid=target.OM9DefinitionUUID)).OM9DefinitionUUID
            source_target(obj,target)
        else:raise RuntimeError('New instance target is not an explicit definition')
        key=(namespace,obj.Name)
        if key in staged_instances:
            if staged_instances[key][1]!=role:raise RuntimeError('New block instance cannot be both top-level and a definition member')
            return staged_instances[key][0]
        matrix=_new_instance_matrix(obj,world=role=='top-level')
        metadata=source_metadata(obj)
        if not metadata['layer']:metadata['layer']='OpenMatrix9'
        identity_instance=_new_identity(obj,'Instance')
        instances.append(dict(namespace=namespace,host_id=obj.Name,source_uuid=identity_instance,role=role,instance_definition_uuid=identity,instance_matrix=list(matrix.A),metadata=metadata))
        staged_instances[key]=(identity_instance,role)
        return identity_instance
    def definition(obj,namespace,path):
        identity=_new_identity(obj,'Definition');key=(namespace,obj.Name)
        if key in path or len(path)>=64:raise RuntimeError('Cyclic or excessive new block nesting')
        if key in visited:return visited[key]
        if len(visited)+len(staged_instances)+len(staged_geometry)>=1000000:raise RuntimeError('New block object limit exceeded')
        visited[key]=identity;members=[]
        for child in obj.Group:
            if child.Document!=obj.Document:raise RuntimeError('Foreign new block member')
            if _is_new_instance(child):member=reference(child,namespace,'definition-member',path+(key,))
            elif _is_new_geometry(child):
                child_key=(namespace,child.Name)
                if child_key not in staged_geometry:
                    row=_stage_new_geometry(child,namespace,staging,len(geometry),'definition-member');geometry.append(row);staged_geometry[child_key]=row['source_uuid']
                member=staged_geometry[child_key]
            elif (archive_identity(child) or getattr(child,'OM9DefinitionMemberProxy',False)) and source_member is not None:
                member=source_member(child,namespace)
            else:raise RuntimeError('Unsupported or sourced new definition member')
            if member in members:raise RuntimeError('Duplicate new block member identity')
            members.append(member)
        row=dict(namespace=namespace,host_id=obj.Name,source_uuid=identity,name=obj.Label,member_uuids=members)
        if hasattr(obj,'OM9CopiedDefinitionSourceUUID'):row['source_definition_uuid']=obj.OM9CopiedDefinitionSourceUUID
        definitions.append(row)
        return identity
    for root in roots:
        owner=getattr(root.LinkedObject,'OM9ArchiveOwner',None) if root.LinkedObject is not None else None
        if owner is None:owner=getattr(root,'OM9ArchiveOwner',None)
        namespace=owner.OM9ImportNamespace if owner is not None else default_namespace
        reference(root,namespace,'top-level',())
    # Source-target callbacks can discover additional new references in imported
    # definitions. Consume that growing worklist without staging a member twice.
    processed=set();processed_definitions=set()
    while member_roots or definition_roots:
        pending=[key for key in (member_roots or {}) if key not in processed]
        pending_definitions=[key for key in (definition_roots or {}) if key not in processed_definitions]
        if not pending and not pending_definitions:break
        for key in pending_definitions:
            processed_definitions.add(key)
            definition(definition_roots[key],key[0],())
        for key in pending:
            processed.add(key)
            reference(member_roots[key],key[0],'definition-member',())

def _is_new_geometry(obj):
    if obj is None or any(hasattr(obj,name) for name in ('OM9SourceUUID','OM9ArchiveOwner','OM9DefinitionUUID','OM9InstanceMatrix','OM9Capability')):
        return False
    return obj.isDerivedFrom('Part::Feature') or obj.isDerivedFrom('Mesh::Feature')

def _new_instance_matrix(obj, world=False):
    import FreeCAD as App
    matrix=App.Matrix(*source_placement(obj)) if world else obj.Placement.toMatrix()
    scale=App.Matrix();scale.scale(obj.ScaleVector);matrix=matrix*scale
    if obj.LinkTransform:matrix=matrix*obj.LinkedObject.Placement.toMatrix()
    return matrix

def _stage_new_geometry(obj, namespace, staging, index, role):
    """Stage a new native CAD/mesh without changing document provenance."""
    import Part
    if not _is_new_geometry(obj):raise RuntimeError('Unsupported or ambiguously sourced new geometry')
    metadata=source_metadata(obj)
    if not metadata['layer']:metadata['layer']='OpenMatrix9'
    identity=_new_identity(obj,'Geometry') if hasattr(obj,'OM9NewGeometryUUID') else str(uuid.uuid4())
    row=dict(host_id=obj.Name,namespace=namespace,source_uuid=identity,role=role,metadata=metadata)
    if obj.isDerivedFrom('Mesh::Feature'):
        points,faces=obj.Mesh.Topology
        if role=='top-level':
            matrix=(obj.getGlobalPlacement()*obj.Placement.inverse()).toMatrix()
            points=[matrix.multVec(point) for point in points]
        row.update(vertices=[list(point) for point in points],faces=[list(face) for face in faces])
    else:
        shape=Part.getShape(obj,'',needSubElement=False,transform=True) if role=='top-level' else obj.Shape.copy()
        if shape is None or shape.isNull():raise RuntimeError('Invalid new CAD geometry')
        if role=='top-level':shape.transformShape((obj.getGlobalPlacement()*obj.Placement.inverse()).toMatrix(),False)
        filename=os.path.join(staging,'new-'+str(index)+'.brep');shape.exportBrep(filename);row['brep']=filename
    return row

def _definition_members(definition, owner, records, allow_new=False):
    import FreeCAD as App
    members=[]
    for child in definition.Group:
        member=child
        if allow_new and _is_new_instance(child):
            target=child.LinkedObject
            if target is None or target.Document!=definition.Document or not target.isDerivedFrom('App::Part'):
                raise RuntimeError('Missing new definition member target')
            target_owner=getattr(target,'OM9ArchiveOwner',None)
            if target_owner is not None and target_owner!=owner:
                raise RuntimeError('New definition member target belongs to another source archive')
            bound_owner=getattr(child,'OM9ArchiveOwner',None)
            if bound_owner is not None and bound_owner!=owner:raise RuntimeError('New member belongs to another source archive')
            source='newref:'+child.Name
            if source in members:raise RuntimeError('Duplicate new instance member')
            members.append(source)
            continue
        if getattr(child,'OM9DefinitionMemberProxy',False):
            member=child.LinkedObject
        if allow_new and member==child and _is_new_geometry(member):
            source='new:'+member.Name
            if source in members:raise RuntimeError('Duplicate new definition member')
            members.append(source)
            continue
        identity=archive_identity(member) if member is not None else None
        if not identity or getattr(member,'OM9ArchiveOwner',None)!=owner or identity[0]!=owner.OM9ImportNamespace:
            raise RuntimeError('Definition members must belong to the same source archive')
        independent=member.Name!=getattr(member,'OM9SourceHostID','')
        if independent and not allow_new:
            raise RuntimeError('Copied source member needs an independent geometry overlay; canonical identity is ambiguous')
        source=identity[1]
        if source not in records or records[source].get('role')!='definition-member':
            raise RuntimeError('Invalid or duplicate source definition member')
        if getattr(child,'OM9DefinitionMemberProxy',False):source='proxy:'+child.Name
        elif independent:source='copy:'+member.Name
        if source in members:raise RuntimeError('Duplicate definition member')
        members.append(source)
    return members

IDENTITY_MATRIX=[1.0,0.0,0.0,0.0,0.0,1.0,0.0,0.0,0.0,0.0,1.0,0.0,0.0,0.0,0.0,1.0]

def _source_copy_identity(namespace,source,host):
    """Stable scoped output identity; export does not mutate source hosts."""
    return str(uuid.uuid5(uuid.UUID(namespace),'OpenMatrix9.3dm.source-copy:'+source+':'+host))

def _native_current(obj,record):
    import ThreeDmNativeFields
    return ThreeDmNativeFields.adapter(obj,record) is not None

def _selected_member_copy(obj,record,manifest,staging,index):
    """Export one physical member as an independent native world-space root."""
    import FreeCAD as App
    proxy=getattr(obj,'OM9DefinitionMemberProxy',False)
    if obj.isDerivedFrom('App::Link') and getattr(obj,'ElementCount',0):raise RuntimeError('Selected link arrays require explicit element export support')
    member=obj.LinkedObject if proxy else obj
    baseline_metadata=json.loads(member.OM9SourceMetadata)
    unchanged=source_signature(member,metadata_override=baseline_metadata)==member.OM9SourceSignature
    if record['class_name']=='ON_InstanceRef' or _native_current(member,record) or (record['capability']=='editable' and not unchanged):
        copy=_independent_member_copy(member,record,manifest,staging,index)
    else:
        if not unchanged:raise RuntimeError('Retained selected member payload was modified')
        copy=dict(namespace=member.OM9ImportNamespace,source_uuid=member.OM9SourceUUID,
                  follow_canonical=False,metadata=source_metadata(member))
    if hasattr(obj,'Placement'):
        parent=App.Matrix(*source_placement(obj))*obj.Placement.inverse().toMatrix()
    else:
        parents=[p for p in obj.InList if p.isDerivedFrom('App::Part') and obj in p.Group]
        if len(parents)>1:raise RuntimeError('Selected retained member has ambiguous physical parents')
        parent=parents[0].getGlobalPlacement().toMatrix() if parents else App.Matrix()
    matrix=parent
    if proxy:
        local,metadata=_proxy_state(obj);matrix=matrix*App.Matrix(*local);copy['metadata'].update(metadata)
    if _native_only(member,record) and record['class_name']!='ON_InstanceRef':
        if hasattr(member,'Placement'):raise RuntimeError('Retained native geometry requires its explicit native placement property')
        placement=getattr(member,'OM9BlockMemberPlacement',App.Placement())
        if not isinstance(placement,App.Placement):raise RuntimeError('Retained native placement must be a FreeCAD Placement')
        matrix=matrix*placement.toMatrix()
    copy.update(host_id='selected-copy:'+obj.Name,output_uuid=_source_copy_identity(member.OM9ImportNamespace,member.OM9SourceUUID,'selected:'+obj.Name),role='top-level',member_matrix=list(matrix.A))
    return copy


def _independent_member_copy(member, record, manifest, staging, index):
    """Stage the live copy, never an overlay on its canonical host identity."""
    copy=dict(namespace=member.OM9ImportNamespace,host_id=member.Name,source_uuid=member.OM9SourceUUID,
              output_uuid=_source_copy_identity(member.OM9ImportNamespace,member.OM9SourceUUID,member.Name),member_matrix=IDENTITY_MATRIX,metadata=source_metadata(member))
    if _native_current(member,record):
        import ThreeDmNativeFields,FreeCAD as App
        placement=getattr(member,'OM9BlockMemberPlacement',App.Placement())
        if not isinstance(placement,App.Placement):raise RuntimeError('Native current geometry placement must be a FreeCAD Placement')
        copy['member_matrix']=list(placement.toMatrix().A)
        if not ThreeDmNativeFields.adapter(member,record).changed(member,record,manifest):
            copy['follow_canonical']=False
            return copy
        edit=dict(action='transform',**ThreeDmNativeFields.stage(member,record,staging,index))
    elif record['class_name']=='ON_InstanceRef':
        edit=dict(action='instance',instance_matrix=_instance_overlay_matrix(member,record,manifest))
        definition=_instance_definition(member,record)
        target=_definition_identity(definition)
        if target!=record['instance_definition_uuid']:edit['instance_definition_uuid']=target
    elif _native_only(member,record):
        import FreeCAD as App
        if source_signature(member,metadata_override=json.loads(member.OM9SourceMetadata))!=member.OM9SourceSignature:
            raise RuntimeError('Retained copied member payload was modified')
        if hasattr(member,'Placement'):raise RuntimeError('Retained native geometry requires its explicit native placement property')
        placement=getattr(member,'OM9BlockMemberPlacement',App.Placement())
        if not isinstance(placement,App.Placement):raise RuntimeError('Retained native placement must be a FreeCAD Placement')
        copy.update(follow_canonical=False,member_matrix=list(placement.toMatrix().A))
        return copy
    elif record['capability']=='editable' and member.isDerivedFrom('Part::Feature'):
        shape=member.Shape.copy()
        if shape.isNull():raise RuntimeError('Invalid independent copied member geometry')
        filename=os.path.join(staging,'independent-member-'+str(index)+'.brep');shape.exportBrep(filename)
        edit=dict(action='replace',brep=filename)
    elif record['capability']=='editable' and member.isDerivedFrom('Mesh::Feature'):
        points,faces=member.Mesh.Topology
        edit=dict(action='replace',vertices=[list(point) for point in points],faces=[list(face) for face in faces])
    else:raise RuntimeError('Independent copied member has no supported geometry overlay')
    copy['independent_overlay']=edit
    return copy

def _proxy_matrix(proxy):
    import FreeCAD as App
    if not proxy.isDerivedFrom('App::Link') or proxy.LinkedObject is None:raise RuntimeError('Invalid shared member proxy')
    matrix=proxy.Placement.toMatrix();scale=App.Matrix();scale.scale(proxy.ScaleVector);matrix=matrix*scale
    if not proxy.LinkTransform and hasattr(proxy.LinkedObject,'Placement'):
        matrix=matrix*proxy.LinkedObject.Placement.inverse().toMatrix()
    return list(matrix.A)

def _proxy_state(proxy):
    if not hasattr(proxy,'OM9ProxyMetadata'):raise RuntimeError('Shared proxy has no verified metadata baseline; legacy migration is required')
    baseline=json.loads(proxy.OM9ProxyMetadata);current=source_metadata(proxy)
    return _proxy_matrix(proxy),{key:current[key] for key in current if current[key]!=baseline[key]}

def _definition_identity(definition):
    if 'OM9NewDefinitionUUID' in definition.PropertiesList:return _new_identity(definition,'Definition')
    return definition.OM9DefinitionUUID

def _resolve_definition_target(document, owner, target, new_only=False):
    try:identity=uuid.UUID(target)
    except (ValueError,TypeError,AttributeError):raise RuntimeError('Invalid block definition target UUID')
    if identity.int==0 or str(identity)!=target:raise RuntimeError('Invalid block definition target UUID')
    manifest=json.loads(owner.OM9ArchiveManifest)
    source_ids={row['source_uuid'] for row in manifest['components'] if row['class_name']=='ON_InstanceDefinition'}
    occupied={row['source_uuid'] for row in manifest['components']+manifest['records']}
    if new_only and target in occupied:raise RuntimeError('New definition identity collides with its source archive namespace')
    definitions=[]
    for candidate in document.Objects:
        if not candidate.isDerivedFrom('App::Part'):continue
        candidate_owner=getattr(candidate,'OM9ArchiveOwner',None)
        if 'OM9NewDefinitionUUID' in candidate.PropertiesList:
            # A local new definition can coexist with its reimported native
            # counterpart. Source UUIDs resolve inside the verified namespace;
            # that unsourced original is not a second source mapping.
            if target in occupied:continue
            if candidate_owner is not None and candidate_owner!=owner:continue
            try:value=_new_identity(candidate,'Definition')
            except RuntimeError:continue
        elif 'OM9DefinitionUUID' in candidate.PropertiesList and candidate_owner==owner:
            if new_only:continue
            value=candidate.OM9DefinitionUUID
            if value not in source_ids:continue
        else:continue
        if value==target:definitions.append(candidate)
    if len(definitions)!=1:raise RuntimeError('Missing or ambiguous verified block definition target')
    return definitions[0]

def _instance_definition(obj, record):
    owner=getattr(obj,'OM9ArchiveOwner',None)
    if owner is None and _is_new_instance(obj) and obj.LinkedObject is not None:
        owner=getattr(obj.LinkedObject,'OM9ArchiveOwner',None)
    if owner is None:raise RuntimeError('Missing source block ownership')
    if obj.isDerivedFrom('App::Link'):
        definition=obj.LinkedObject
        if definition is None or definition.Document!=obj.Document or not definition.isDerivedFrom('App::Part'):
            raise RuntimeError('Block target must belong to the same source archive namespace')
        target_owner=getattr(definition,'OM9ArchiveOwner',None)
        if (target_owner is not None and target_owner!=owner) or ('OM9NewDefinitionUUID' not in definition.PropertiesList and target_owner!=owner):
            raise RuntimeError('Block target must belong to the same source archive namespace')
        target=_definition_identity(definition)
    else:target=getattr(obj,'OM9DefinitionTargetUUID',record['instance_definition_uuid'])
    new_only=obj.isDerivedFrom('App::Link') and 'OM9NewDefinitionUUID' in obj.LinkedObject.PropertiesList
    definition=_resolve_definition_target(obj.Document,owner,target,new_only)
    if obj.isDerivedFrom('App::Link') and obj.LinkedObject!=definition:raise RuntimeError('Ambiguous block reference target')
    return definition

def _reference_dependencies(obj, record):
    dependencies=list(record['dependencies'])
    if record['class_name']=='ON_InstanceRef':
        target=_definition_identity(_instance_definition(obj,record))
        dependencies=[target if item==record['instance_definition_uuid'] else item for item in dependencies]
    return dependencies

def _instance_overlay_matrix(obj, record, manifest, world=False, preview_root=False):
    import FreeCAD as App
    original=list(record['instance_matrix'])
    for position in (3,7,11):original[position]*=manifest['scale_mm']
    if list(obj.OM9InstanceMatrix)!=original:raise RuntimeError('Native instance matrix was modified outside the placement overlay')
    definition=_instance_definition(obj,record)
    if obj.isDerivedFrom('App::Link'):
        matrix=App.Matrix(*source_placement(obj)) if world else obj.Placement.toMatrix()
        scale=App.Matrix();scale.scale(obj.ScaleVector);matrix=matrix*scale
        if obj.LinkTransform:matrix=matrix*definition.Placement.toMatrix()
    else:
        if not _preview_matches(obj):raise RuntimeError('Affine preview geometry was edited or has no verified baseline')
        if preview_root:matrix=App.Matrix(*original)
        else:
            current=App.Matrix(*source_placement(obj)) if world else obj.Placement.toMatrix()
            initial=App.Matrix(*json.loads(obj.OM9SourcePlacement))
            matrix=current*initial.inverse()*App.Matrix(*original)
        matrix=matrix*definition.Placement.toMatrix()
    return list(matrix.A)

def refresh_affine_previews(document, owner):
    """Rebuild retained display geometry from current canonical members.

    Source matrices and signatures remain provenance. Only the derived preview
    integrity baseline changes; a manually edited preview is never overwritten.
    """
    import FreeCAD as App, Part, Mesh
    manifest=json.loads(owner.OM9ArchiveManifest)
    records={r['source_uuid']:r for r in manifest['records']+manifest['components']}
    members={o.OM9SourceUUID:o for o in document.Objects
             if archive_identity(o) and getattr(o,'OM9ArchiveOwner',None)==owner
             and o.Name==getattr(o,'OM9SourceHostID','')}
    roots=[o for o in document.Objects if getattr(o,'OM9ArchiveOwner',None)==owner
           and getattr(o,'OM9SourceClass','')=='ON_InstanceRef' and not o.isDerivedFrom('App::Link')
           and hasattr(o,'OM9PreviewSignature')]
    def native_matrix(obj, root=False):
        if _is_new_instance(obj):return _new_instance_matrix(obj)
        return App.Matrix(*_instance_overlay_matrix(obj,records[obj.OM9SourceUUID],manifest,preview_root=root))
    for root in roots:
        try:
            if not _preview_matches(root):
                raise RuntimeError('Preview geometry was edited; canonical member refresh refused')
            rendered=[]
            def visit(instance, transform, path):
                if _is_new_instance(instance):
                    definition=instance.LinkedObject
                    if definition is None or definition.Document!=document or not definition.isDerivedFrom('App::Part'):
                        raise RuntimeError('Missing new preview definition target')
                    target_owner=getattr(definition,'OM9ArchiveOwner',None)
                    if target_owner is not None and target_owner!=owner:raise RuntimeError('Foreign new preview target')
                    if hasattr(definition,'OM9NewDefinitionUUID'):identity=_new_identity(definition,'Definition')
                    else:
                        definition=_instance_definition(instance,dict(instance_definition_uuid=definition.OM9DefinitionUUID));identity=definition.OM9DefinitionUUID
                else:
                    definition=_instance_definition(instance,records[instance.OM9SourceUUID]);identity=_definition_identity(definition)
                if identity in path or len(path)>=64:raise RuntimeError('Cyclic or excessive block preview nesting')
                if hasattr(definition,'OM9NewDefinitionUUID'):
                    sources=[]
                    for child in definition.Group:
                        if _is_new_instance(child):sources.append('newref:'+child.Name)
                        elif _is_new_geometry(child):sources.append('new:'+child.Name)
                        elif getattr(child,'OM9DefinitionMemberProxy',False) and 'OM9NewMemberUUID' in child.PropertiesList:sources.append('proxy:'+child.Name)
                        elif archive_identity(child) and 'OM9NewMemberUUID' in child.PropertiesList:sources.append('sourced:'+child.Name)
                        else:raise RuntimeError('Unsupported new preview definition member')
                else:sources=_definition_members(definition,owner,records,allow_new=True)
                for source in sources:
                    member_transform=transform
                    if source.startswith('proxy:'):
                        proxy=document.getObject(source[6:]);member=proxy.LinkedObject;source=member.OM9SourceUUID
                        member_transform=transform*App.Matrix(*_proxy_matrix(proxy))
                    elif source.startswith('copy:'):
                        member=document.getObject(source[5:]);source=member.OM9SourceUUID
                    elif source.startswith('newref:'):member=document.getObject(source[7:])
                    elif source.startswith('sourced:'):
                        member=document.getObject(source[8:]);source=member.OM9SourceUUID
                    else:member=document.getObject(source[4:]) if source.startswith('new:') else members.get(source)
                    if member is None:raise RuntimeError('Missing canonical preview member')
                    if _is_new_instance(member) or (source in records and records[source]['class_name']=='ON_InstanceRef'):
                        visit(member,member_transform*native_matrix(member),path+(identity,))
                    elif member.isDerivedFrom('Part::Feature'):
                        # Strip the wrapper Location before copying, then bake
                        # that placement into the full display transform. OCC
                        # can retain the original Location through GTransform;
                        # resetting the preview child would otherwise lose it.
                        shape=Part.Shape(member.Shape);local=shape.Placement.toMatrix();shape.Placement=App.Placement()
                        rendered.append(('Part::Feature',shape.copy().transformGeometry(member_transform*local)))
                    elif member.isDerivedFrom('Mesh::Feature'):
                        # Topology includes the canonical Mesh object's Placement.
                        # Mesh.transform only changes its kernel, so copying then
                        # resetting the preview Placement would lose that transform.
                        points,faces=member.Mesh.Topology
                        mesh=Mesh.Mesh()
                        for face in faces:
                            mesh.addFacet(*(member_transform.multVec(points[i]) for i in face))
                        rendered.append(('Mesh::Feature',mesh))
                    elif 'OM9PointCloudSchema' in member.PropertiesList:
                        import ThreeDmPointCloud
                        local=getattr(member,'OM9BlockMemberPlacement',App.Placement()).toMatrix()
                        rendered.append(('App::FeaturePython',ThreeDmPointCloud.transformed_fields(member,member_transform*local)))
                    else:raise RuntimeError('Native member has no supported display geometry')
                    if len(rendered)>1000000:raise RuntimeError('Preview object limit exceeded')
            visit(root,native_matrix(root,True),())
            placement=root.Placement
            if root.isDerivedFrom('Part::Feature'):
                shapes=[]
                for kind,geometry in rendered:
                    if kind=='Mesh::Feature':
                        # Faceted display of native mesh, never an export source.
                        shape=Part.Shape();shape.makeShapeFromMesh(geometry.Topology,0.0);shapes.append(shape)
                    elif kind=='Part::Feature':shapes.append(geometry)
                    else:raise RuntimeError('Legacy Part affine preview cannot display PointCloud; rebuild it as a mixed native preview')
                root.Shape=Part.makeCompound(shapes) if shapes else Part.Shape()
                root.Placement=placement
            else:
                old=list(root.Group);used=[]
                for kind,geometry in rendered:
                    child=next((o for o in old if o.TypeId==kind and o not in used),None)
                    if child is None:
                        child=document.addObject(kind,'RhinoBlockPreview')
                        _property(child,'String','OM9Capability','display-retained')
                    if kind=='Part::Feature':child.Shape=geometry
                    elif kind=='Mesh::Feature':child.Mesh=geometry
                    else:
                        import ThreeDmPointCloud
                        if ThreeDmPointCloud.is_adapter(child):ThreeDmPointCloud.update_fields(child,geometry)
                        else:
                            ThreeDmPointCloud.bind(child,geometry)
                            _property(child,'Placement','OM9BlockMemberPlacement',App.Placement())
                        child.OM9BlockMemberPlacement=App.Placement()
                    if hasattr(child,'Placement'):child.Placement=App.Placement()
                    used.append(child)
                root.Group=used
                for child in old:
                    if child not in used:document.removeObject(child.Name)
                root.Placement=placement
            root.OM9PreviewSignature=preview_signature(root)
            if not hasattr(root,'OM9PreviewStatus'):_property(root,'String','OM9PreviewStatus','')
            root.OM9PreviewStatus='Current canonical members'
        except Exception as error:
            if not hasattr(root,'OM9PreviewStatus'):_property(root,'String','OM9PreviewStatus','')
            root.OM9PreviewStatus=str(error)

class _AffinePreviewObserver:
    def __init__(self):self.pending=set();self.busy=False;self.placement_witnesses={}
    @staticmethod
    def _root(obj):
        return getattr(obj,'OM9SourceClass','')=='ON_InstanceRef' and not obj.isDerivedFrom('App::Link') and hasattr(obj,'OM9PreviewSignature')
    def slotBeforeChangeObject(self,obj,prop):
        if self.busy or prop!='Placement' or not self._root(obj):return
        key=(obj.Document.Name,obj.Name)
        try:
            if _preview_matches(obj):self.placement_witnesses[key]=obj.OM9PreviewSignature
            else:self.placement_witnesses.pop(key,None)
        except Exception:self.placement_witnesses.pop(key,None)
    def slotChangedObject(self,obj,prop):
        if self.busy or prop not in ('Shape','Mesh','Placement','OM9BlockMemberPlacement','LinkPlacement','LinkTransform','Scale','ScaleVector','Group','Label','LinkedObject','OM9DefinitionTargetUUID','OM9CloudPoints','OM9CloudNormals','OM9CloudRGBA','OM9CloudValues','OM9CloudPlane','OM9CloudOrdered','OM9CloudHasPlane','OM9PointCloudSchema'):return
        try:
            # A payload/child edit after the placement witness must never be
            # mistaken for OCC's equivalent cache changes during a rigid move.
            for candidate in [obj]+list(obj.InList):
                if self._root(candidate) and not (candidate==obj and prop=='Placement'):
                    self.placement_witnesses.pop((obj.Document.Name,candidate.Name),None)
            owner=getattr(obj,'OM9ArchiveOwner',None)
            if owner is None:
                if any(name in obj.PropertiesList for name in ('OM9NewGeometryUUID','OM9NewDefinitionUUID','OM9NewInstanceUUID')):
                    # New definitions may be shared by imported blocks without
                    # owning a snapshot themselves. Refresh verified previews;
                    # their integrity guards still refuse edited display data.
                    for candidate in obj.Document.Objects:
                        if 'OM9ArchiveManifest' in candidate.PropertiesList:self.pending.add((obj.Document.Name,candidate.Name))
                for parent in obj.InList:
                    if hasattr(parent,'OM9DefinitionUUID'):
                        definition_owner=getattr(parent,'OM9ArchiveOwner',None)
                        if definition_owner is not None:self.pending.add((obj.Document.Name,definition_owner.Name))
                return
            if _is_new_instance(obj) or 'OM9NewMemberUUID' in obj.PropertiesList or 'OM9NewDefinitionUUID' in obj.PropertiesList or getattr(obj,'OM9DefinitionMemberProxy',False) or hasattr(obj,'OM9DefinitionUUID') or (
                json.loads(getattr(obj,'OM9SourceRecord','{}')).get('role')=='definition-member') or prop=='OM9DefinitionTargetUUID' or (prop=='Placement' and self._root(obj)):
                self.pending.add((obj.Document.Name,owner.Name))
        except (AttributeError,ReferenceError,ValueError):pass
    def slotDeletedObject(self,obj):self.slotChangedObject(obj,'Group')
    def slotRecomputedDocument(self,document):
        if self.busy:return
        witnesses=[key for key in self.placement_witnesses if key[0]==document.Name]
        for key in witnesses:
            baseline=self.placement_witnesses.pop(key);root=document.getObject(key[1])
            if root is not None and getattr(root,'OM9PreviewSignature','')==baseline:
                # Placement can make OCC add identity Locations and equivalent
                # trimmed pcurves. Only a verified pre-change preview, with no
                # later payload edit, may refresh that display-only baseline.
                root.OM9PreviewSignature=preview_signature(root)
        owners=[name for doc,name in self.pending if doc==document.Name]
        self.pending={item for item in self.pending if item[0]!=document.Name}
        self.busy=True
        try:
            for name in owners:
                owner=document.getObject(name)
                if owner is not None:refresh_affine_previews(document,owner)
        finally:self.busy=False
    def slotDeletedDocument(self,document):
        self.pending={item for item in self.pending if item[0]!=document.Name}
        self.placement_witnesses={key:value for key,value in self.placement_witnesses.items() if key[0]!=document.Name}

_preview_observer=None
def ensure_preview_observer():
    global _preview_observer
    if _preview_observer is None:
        import FreeCAD as App
        _preview_observer=_AffinePreviewObserver();App.addDocumentObserver(_preview_observer)

def _member_overlay(member, record, manifest, staging, index):
    """Stage a canonical definition member in native definition-local mm."""
    import FreeCAD as App
    baseline=getattr(member,'OM9SourceSignature','')
    if not baseline:raise RuntimeError('Block member has no verified edit baseline')
    old=json.loads(member.OM9SourceMetadata);current=source_metadata(member)
    row=dict(host_id=member.Name,namespace=member.OM9ImportNamespace,
             source_uuid=member.OM9SourceUUID,action='unchanged')
    metadata={key:current[key] for key in current if current[key]!=old[key]}
    if metadata:row['metadata']=metadata
    changed=source_signature(member,metadata_override=old)!=baseline
    if record['class_name']=='ON_InstanceRef':
        definition=_instance_definition(member,record)
        matrix=_instance_overlay_matrix(member,record,manifest)
        if not member.isDerivedFrom('App::Link'):changed=changed or not definition.Placement.isIdentity()
    elif _native_only(member,record):
        import ThreeDmNativeFields
        adapter=ThreeDmNativeFields.adapter(member,record)
        if changed and adapter is None:raise RuntimeError('Retained block member geometry cannot be replaced')
        if hasattr(member,'Placement'):raise RuntimeError('Retained native geometry requires its explicit native placement property')
        placement=getattr(member,'OM9BlockMemberPlacement',App.Placement())
        if not isinstance(placement,App.Placement):raise RuntimeError('Retained native placement must be a FreeCAD Placement')
        if not placement.isIdentity():row.update(action='transform',geometry_matrix=list(placement.toMatrix().A))
        if adapter is not None and adapter.changed(member,record,manifest):
            row.update(action='transform',geometry_matrix=list(placement.toMatrix().A))
            row.update(ThreeDmNativeFields.stage(member,record,staging,index))
        return row
    if not changed:return row
    if record['class_name']=='ON_InstanceRef':
        row.update(action='instance',instance_matrix=matrix)
        target=_definition_identity(definition)
        if target!=record['instance_definition_uuid']:row['instance_definition_uuid']=target
    elif record['capability']=='editable' and member.isDerivedFrom('Part::Feature'):
        shape=member.Shape.copy()
        if shape.isNull():raise RuntimeError('Invalid block member geometry')
        filename=os.path.join(staging,'member-'+str(index)+'.brep');shape.exportBrep(filename)
        row.update(action='replace',brep=filename)
    elif record['capability']=='editable' and member.isDerivedFrom('Mesh::Feature'):
        points,faces=member.Mesh.Topology
        row.update(action='replace',vertices=[list(point) for point in points],faces=[list(face) for face in faces])
    else:raise RuntimeError('Retained block member geometry cannot be replaced')
    return row

def validate_manifest(manifest):
    if not isinstance(manifest, dict):
        raise RuntimeError('Invalid source archive manifest')
    text = json.dumps(manifest, ensure_ascii=False, separators=(',', ':'))
    if manifest.get('schema_version') != 1 or len(text.encode('utf-8')) > MAX_MANIFEST:
        raise RuntimeError('Invalid source archive schema or manifest size')
    digest = manifest.get('archive_sha256', '')
    if len(digest) != 64 or any(c not in '0123456789abcdef' for c in digest):
        raise RuntimeError('Invalid source archive hash')
    seen = set()
    for category in ('records', 'components'):
        rows = manifest.get(category)
        if not isinstance(rows, list):
            raise RuntimeError('Invalid source record table')
        for row in rows:
            if not isinstance(row, dict) or row.get('capability') not in ('editable', 'display-retained', 'retained', 'incompatible'):
                raise RuntimeError('Invalid source record capability')
            value = row.get('source_uuid', '')
            try:
                identity = uuid.UUID(value)
            except (ValueError, TypeError, AttributeError):
                raise RuntimeError('Invalid source UUID') from None
            if not identity.int or str(identity) != value or value in seen:
                raise RuntimeError('Invalid or duplicate source UUID')
            seen.add(value)
    for category in ('resources', 'issues'):
        if not isinstance(manifest.get(category), list):
            raise RuntimeError('Invalid archive report table')
    if not isinstance(manifest.get('settings'), dict):
        raise RuntimeError('Invalid archive settings')
    return text

def validate_prepared(prepared):
    validate_manifest(prepared['manifest'])
    if _hash(prepared['snapshot']) != prepared['manifest']['archive_sha256']:
        raise RuntimeError('Source snapshot integrity check failed')
    records = {row['source_uuid']: row for row in prepared['manifest']['records']}
    definitions = {row['source_uuid']: row for row in prepared['manifest']['components'] if row['class_name'] == 'ON_InstanceDefinition'}
    def visit(record, path):
        if record['class_name'] != 'ON_InstanceRef':return
        identity = record.get('instance_definition_uuid')
        if identity in path:raise RuntimeError('Cyclic block definition')
        if len(path) >= 64:raise RuntimeError('Block nesting exceeds64 levels')
        definition = definitions.get(identity)
        if not definition:raise RuntimeError('Missing embedded block definition')
        if not definition.get('member_uuids') and any(resource.get('owner_uuid')==identity and resource.get('kind')=='linked_block_reference' for resource in prepared['manifest']['resources']):raise RuntimeError('External-only block definition has no embedded geometry')
        for member in definition['member_uuids']:
            if member not in records:raise RuntimeError('Missing block member geometry')
            visit(records[member], path+(identity,))
    for record in records.values():
        if record['role'] == 'top-level':visit(record, ())

def load_archive_state(container):
    text = container.OM9ArchiveManifest
    if len(text.encode('utf-8')) > MAX_MANIFEST or container.OM9ArchiveSchema != 1:
        raise RuntimeError('Invalid source archive schema or manifest size')
    manifest = json.loads(text)
    validate_manifest(manifest)
    snapshot = container.OM9SourceArchive
    if manifest.get('schema_version') != 1 or _hash(snapshot) != container.OM9ArchiveHash or manifest.get('archive_sha256') != container.OM9ArchiveHash:
        raise RuntimeError('Source archive integrity check failed')
    return dict(manifest=manifest, snapshot=snapshot)

def bind_archive(document, prepared):
    import ThreeDm
    ensure_preview_observer()
    manifest = prepared['manifest']
    text = validate_manifest(manifest)
    validate_prepared(prepared)
    namespace = str(uuid.uuid4())
    container = document.addObject('App::FeaturePython', 'RhinoSourceArchive')
    container.Label = 'Rhino source archive (storage only)'
    _property(container, 'Integer', 'OM9ArchiveSchema', 1)
    _property(container, 'String', 'OM9ImportNamespace', namespace)
    _property(container, 'FileIncluded', 'OM9SourceArchive', prepared['snapshot'])
    _property(container, 'String', 'OM9ArchiveManifest', text)
    _property(container, 'String', 'OM9ArchiveHash', manifest['archive_sha256'])
    _property(container, 'Integer', 'OM9ArchiveMode', 1)
    objects = ThreeDm._insert_prepared(document.Name, prepared['host_geometry'])
    pairs = list(zip(objects, (item for item, geometry in prepared['host_geometry'])))
    native_records = {row['source_uuid']: row for row in manifest['records']}
    definitions = {}
    for row in manifest['components']:
        if row['class_name'] == 'ON_InstanceDefinition':
            definition = document.addObject('App::Part', 'RhinoBlockDefinition')
            definition.Label = row['name'] or 'Rhino block definition'
            definition.ViewObject.Visibility = False
            _property(definition, 'String', 'OM9DefinitionUUID', row['source_uuid'])
            _property(definition, 'LinkHidden', 'OM9ArchiveOwner', container)
            definitions[row['source_uuid']] = definition
    import FreeCAD as App
    for row in prepared['retained_records'] + prepared.get('definition_retained', []):
        native = native_records[row['source_uuid']]
        matrix = native.get('instance_matrix')
        placement = None
        if matrix:
            matrix = list(matrix)
            for index in (3, 7, 11):matrix[index] *= manifest['scale_mm']
            candidate = App.Matrix(*matrix)
            rigid = all(abs(sum(matrix[i*4+k]*matrix[j*4+k] for k in range(3))-(1 if i==j else 0)) < 1e-9 for i in range(3) for j in range(3))
            if rigid and abs(candidate.determinant()-1) < 1e-9:
                placement = App.Placement(candidate)
        if placement is not None and native.get('instance_definition_uuid') in definitions:
            obj = document.addObject('App::Link', 'RhinoBlockInstance')
            obj.setLink(definitions[native['instance_definition_uuid']])
            obj.Placement = placement
            obj.Label = row['name'] or 'Rhino block instance'
        else:
            preview = prepared.get('host_previews', {}).get(row['source_uuid'], [])
            if preview:
                import Part
                row = dict(row, capability='display-retained')
                if all('brep' in item for item, geometry in preview):
                    obj = document.addObject('Part::Feature', 'RhinoAffineBlock')
                    obj.Shape = Part.makeCompound([geometry for item, geometry in preview])
                    obj.setEditorMode('Shape', 1)
                else:
                    obj = document.addObject('App::Part', 'RhinoAffineBlock')
                    for item, geometry in preview:
                        child = document.addObject('Part::Feature' if 'brep' in item else 'App::FeaturePython' if 'point_cloud_fields' in item else 'Mesh::Feature', 'RhinoBlockPreview')
                        if 'brep' in item:child.Shape = geometry
                        elif 'point_cloud_fields' in item:
                            import ThreeDmPointCloud
                            ThreeDmPointCloud.bind(child,geometry)
                            _property(child,'Placement','OM9BlockMemberPlacement',App.Placement())
                        else:child.Mesh = geometry
                        _property(child, 'String', 'OM9Capability', 'display-retained')
                        obj.addObject(child)
            else:
                # Stable container allows an empty affine block to gain members.
                obj = document.addObject('App::Part' if matrix else 'App::FeaturePython', 'RhinoRetainedRecord')
            obj.Label = (row['name'] or row['class_name']) + ' [source retained]'
        if row.get('representation_issue'):
            _property(obj,'String','OM9RepresentationIssue',row['representation_issue'])
            obj.setEditorMode('OM9RepresentationIssue',1)
        if matrix:
            _property(obj, 'FloatList', 'OM9InstanceMatrix', matrix)
            if not obj.isDerivedFrom('App::Link'):
                _property(obj,'String','OM9DefinitionTargetUUID',native['instance_definition_uuid'])
                obj.setEditorMode('OM9DefinitionTargetUUID',0)
        # Retained records bypass the editable geometry inserter, so bind their
        # native attributes here as well as their structural source identity.
        _property(obj, 'String', 'OM9LayerPath', row.get('layer', ''))
        _property(obj, 'Bool', 'OM9Locked', bool(row.get('locked', False)))
        color=[channel/255.0 for channel in row.get('color', [178,178,178])]
        _property(obj, 'FloatList', 'OM9Color', color)
        if hasattr(obj.ViewObject,'ShapeColor'):obj.ViewObject.ShapeColor=tuple(color)
        if hasattr(obj.ViewObject,'LineColor'):obj.ViewObject.LineColor=tuple(color)
        if hasattr(obj.ViewObject,'Deviation'):obj.ViewObject.Deviation=0.05
        if hasattr(obj.ViewObject,'AngularDeflection'):obj.ViewObject.AngularDeflection=5.0
        obj.ViewObject.Visibility=bool(row.get('visible', True))
        if native['class_name']=='ON_TextDot':
            import ThreeDmTextDot
            ThreeDmTextDot.bind(obj,native,manifest['scale_mm'])
            obj.Label=row['name'] or 'Rhino TextDot'
        elif native['class_name']=='ON_PointCloud':
            import ThreeDmPointCloud
            data=prepared.get('host_point_clouds',{}).get(row['source_uuid'])
            if data is None:raise RuntimeError('PointCloud has no verified prepared native current fields')
            ThreeDmPointCloud.bind(obj,data)
            obj.Label=row['name'] or 'Rhino PointCloud'
        elif native['class_name']=='ON_Hatch' and 'hatch_current' in native:
            import ThreeDmHatch
            ThreeDmHatch.bind(obj,native,manifest['hatch_pattern_choices'])
            obj.Label=row['name'] or 'Rhino Hatch'
        objects.append(obj)
        pairs.append((obj, row))
    by_uuid = {row['source_uuid']: obj for obj, row in pairs}
    member_uses = {}
    for row in manifest['components']:
        for member in row.get('member_uuids', []):member_uses[member] = member_uses.get(member, 0)+1
    for row in manifest['components']:
        if row['source_uuid'] in definitions:
            for member in row['member_uuids']:
                if member in by_uuid:
                    obj = by_uuid[member]
                    if member_uses[member] > 1:
                        proxy = document.addObject('App::Link', 'RhinoSharedMember')
                        proxy.setLink(obj)
                        proxy.LinkTransform=True
                        _property(proxy, 'Bool', 'OM9DefinitionMemberProxy', True)
                        definitions[row['source_uuid']].addObject(proxy)
                        obj.ViewObject.Visibility = False
                    else:
                        definitions[row['source_uuid']].addObject(obj)
    objects = [obj for obj, row in pairs if native_records[row['source_uuid']]['role'] == 'top-level']
    document.recompute()
    for obj, row in pairs:
        _property(obj, 'String', 'OM9SourceUUID', row['source_uuid'])
        _property(obj, 'String', 'OM9ImportNamespace', namespace)
        _property(obj, 'String', 'OM9SourceClass', row['class_name'])
        _property(obj, 'String', 'OM9SourceHostID', obj.Name)
        _property(obj, 'String', 'OM9Capability', row['capability'])
        record = next(record for record in manifest['records'] if record['source_uuid'] == row['source_uuid'])
        if obj.isDerivedFrom('Part::Feature'):
            if 'OM9IsoCurveDensity' not in obj.ViewObject.PropertiesList:
                obj.ViewObject.addProperty('App::PropertyInteger','OM9IsoCurveDensity','Rhino display','Display-only Rhino isocurve density (-1: boundary only)')
            obj.ViewObject.OM9IsoCurveDensity=int(record.get('wire_density',1))
        _property(obj, 'LinkHidden', 'OM9ArchiveOwner', container)
        _property(obj, 'String', 'OM9SourceRecord', json.dumps(record, sort_keys=True))
        if _native_only(obj,record) and record['class_name']!='ON_InstanceRef' and not hasattr(obj,'Placement'):
            _property(obj,'Placement','OM9BlockMemberPlacement',App.Placement())
            obj.setEditorMode('OM9BlockMemberPlacement',0)
        _property(obj, 'String', 'OM9SourceMetadata', json.dumps(source_metadata(obj), sort_keys=True))
        _property(obj, 'String', 'OM9SourcePlacement', json.dumps(source_placement(obj)))
        if obj.isDerivedFrom('Part::Feature') and record['class_name']!='ON_InstanceRef':
            obj.addProperty('Part::PropertyPartShape','OM9SourceShapeBaseline','Rhino source')
            obj.OM9SourceShapeBaseline=obj.Shape.copy()
            obj.setEditorMode('OM9SourceShapeBaseline',3)
            _property(obj,'String','OM9SourceShapeToken',_cad_fingerprint(obj.Shape))
        # Sign once below, after every source identity and baseline is bound.
        # This also lets references resolve regardless of insertion order.
        _property(obj, 'String', 'OM9SourceSignature', '')
        if record['class_name']=='ON_InstanceRef' and not obj.isDerivedFrom('App::Link') and hasattr(obj,'Placement'):
            _property(obj, 'String', 'OM9PreviewSignature', preview_signature(obj))
    for obj, row in pairs:
        obj.OM9SourceSignature = source_signature(obj)
    for definition in definitions.values():
        for member in definition.Group:
            if getattr(member,'OM9DefinitionMemberProxy',False):
                _property(member,'String','OM9ProxyMetadata',json.dumps(source_metadata(member),sort_keys=True))
        _property(definition, 'String', 'OM9DefinitionSignature', definition_signature(definition))
    return objects
