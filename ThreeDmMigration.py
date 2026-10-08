"""Explicit restoration of legacy 3dm baselines from verified native snapshots."""
import json
import math
import uuid
import tempfile


def migrate_archive(owner, rebuild_previews=False, origins=None, targets=None, fork_namespace=False, upgrades=None):
    """Restore provenance; origins maps native UUIDs to original host Names.

    Explicit origins resolve legacy duplicate identities without using Labels.
    Geometry and physical reference targets remain the user's current objects.
    targets maps affine reference Names to explicit target definition Names.
    fork_namespace separates an explicitly copied archive owner and its graph.
    """
    import FreeCAD as App
    import ThreeDm
    import ThreeDmArchiveState as state
    import ThreeDmUpgrade as upgrade

    if owner is None:raise RuntimeError('Missing source archive to migrate')
    document=owner.Document;state._validate_creation(document)
    if type(rebuild_previews) is not bool:raise RuntimeError('Preview rebuild policy must be boolean')
    if type(fork_namespace) is not bool:raise RuntimeError('Namespace fork policy must be boolean')
    upgrades=upgrade.policies(upgrades)
    if origins is None:origins={}
    if not isinstance(origins,dict) or any(not isinstance(k,str) or not isinstance(v,str) or not v for k,v in origins.items()):raise RuntimeError('Migration origins must map native UUIDs to host Names')
    if targets is None:targets={}
    if not isinstance(targets,dict) or any(not isinstance(k,str) or not isinstance(v,str) or not k or not v for k,v in targets.items()):raise RuntimeError('Migration targets must map affine host Names to definition Names')
    if document.HasPendingTransaction:raise RuntimeError('Finish the current transaction before archive migration')
    if getattr(owner,'OM9ArchiveSchema',1)!=1 or getattr(owner,'OM9ArchiveMode',1)!=1:raise RuntimeError('Unsupported legacy archive schema or mode')
    try:old=json.loads(owner.OM9ArchiveManifest)
    except (ValueError,AttributeError):raise RuntimeError('Legacy archive has no valid manifest')
    state.validate_manifest(old)
    from ThreeDmStorage import path
    snapshot=path(owner,'OM9SourceArchive',state.MAX_ARCHIVE);digest=state._hash(snapshot)
    if digest!=old['archive_sha256'] or getattr(owner,'OM9ArchiveHash',digest)!=digest:raise RuntimeError('Legacy source snapshot integrity check failed')
    scale=old.get('scale_mm')
    if not isinstance(scale,(int,float)) or isinstance(scale,bool) or not math.isfinite(scale) or scale<=0:raise RuntimeError('Legacy archive has no verified unit scale')
    namespace=getattr(owner,'OM9ImportNamespace','')
    if not namespace:
        candidates={getattr(o,'OM9ImportNamespace','') for o in document.Objects if getattr(o,'OM9ArchiveOwner',None)==owner and not getattr(o,'OM9DefinitionMemberProxy',False)}-{''}
        if len(candidates)!=1:raise RuntimeError('Legacy archive namespace is ambiguous')
        namespace=next(iter(candidates))
    try:parsed=uuid.UUID(namespace)
    except (ValueError,TypeError,AttributeError):raise RuntimeError('Invalid legacy import namespace')
    if not parsed.int or str(parsed)!=namespace:raise RuntimeError('Invalid legacy import namespace')
    source_namespace=namespace;forked_namespace=False;namespace_origin=None
    if fork_namespace:
        for field in ('OM9NamespaceOrigin','OM9NamespaceHostID'):
            if field in owner.PropertiesList and owner.getTypeIdOfProperty(field)!='App::PropertyString':raise RuntimeError('Namespace fork provenance has an incompatible type')
        if getattr(owner,'OM9NamespaceHostID','')==owner.Name:
            namespace_origin=getattr(owner,'OM9NamespaceOrigin','')
            try:origin=uuid.UUID(namespace_origin)
            except (ValueError,TypeError,AttributeError):raise RuntimeError('Namespace fork has no verified origin')
            if not origin.int or str(origin)!=namespace_origin or namespace_origin==namespace:raise RuntimeError('Invalid namespace fork origin')
        else:
            namespace_origin=namespace;namespace=str(uuid.uuid4());forked_namespace=True
        sources=[o for o in document.Objects if 'OM9SourceUUID' in o.PropertiesList and not getattr(o,'OM9DefinitionMemberProxy',False) and state.archive_identity(o) and getattr(o,'OM9ArchiveOwner',None)==owner]
        if any(o.OM9ImportNamespace!=source_namespace for o in sources):raise RuntimeError('Forked owner contains conflicting source namespaces')
    else:
        sources=[o for o in document.Objects if 'OM9SourceUUID' in o.PropertiesList and not getattr(o,'OM9DefinitionMemberProxy',False) and state.archive_identity(o) and o.OM9ImportNamespace==namespace]
    if not sources:raise RuntimeError('Legacy archive has no unambiguous source bindings')
    if set(upgrades)-{o.Name for o in sources}:raise RuntimeError('Migration upgrade host is not an owned source binding')
    for obj in sources:
        if getattr(obj,'OM9ArchiveOwner',None) not in (None,owner):raise RuntimeError('Legacy source ownership conflicts with its namespace')

    reference=None
    plans=[];preview_resets=[];replacements=[]
    def plan(obj,kind,name,value):
        if name in obj.PropertiesList:
            expected='App::Property'+kind;actual=obj.getTypeIdOfProperty(name)
            allowed={expected,'App::PropertyLink'} if kind=='LinkHidden' else {expected}
            if actual not in allowed:raise RuntimeError('Legacy property has an incompatible type: '+obj.Name+'.'+name)
        plans.append((obj,kind,name,value))
    def payload_fingerprint(obj,root=True):
        payload=dict(type=obj.TypeId)
        if not root:
            metadata=state.source_metadata(obj);metadata.pop('name')
            placement=getattr(obj,'Placement',getattr(obj,'OM9BlockMemberPlacement',App.Placement()))
            payload.update(placement=list(placement.toMatrix().A),metadata=metadata)
        if obj.isDerivedFrom('App::Part'):payload['children']=[payload_fingerprint(child,False) for child in obj.Group]
        else:payload['signature']=state.preview_signature(obj)
        return payload
    try:
        reference=App.newDocument('OM9MigrationReference',hidden=True,temp=True)
        with tempfile.TemporaryDirectory(prefix='om9-migration-') as staging:
            prepared=ThreeDm._prepare_import(snapshot,staging,scale,'preserve')
            state.bind_archive(reference,prepared)
        ref_owner=next(o for o in reference.Objects if 'OM9ArchiveManifest' in o.PropertiesList)
        fresh=json.loads(ref_owner.OM9ArchiveManifest)
        if fresh['archive_sha256']!=digest or fresh['scale_mm']!=scale:raise RuntimeError('Migration reference differs from verified snapshot or scale')
        records={r['source_uuid']:r for r in fresh['records']}
        native_definitions={r['source_uuid']:r for r in fresh['components'] if r['class_name']=='ON_InstanceDefinition'}
        ref_sources={o.OM9SourceUUID:o for o in reference.Objects if state.archive_identity(o) and not getattr(o,'OM9DefinitionMemberProxy',False)}
        ref_definitions={o.OM9DefinitionUUID:o for o in reference.Objects if 'OM9DefinitionUUID' in o.PropertiesList}
        native_previews={identity:payload_fingerprint(obj) for identity,obj in ref_sources.items() if hasattr(obj,'OM9PreviewSignature')}
        # Both initial native conversion and canonical refresh are legitimate
        # display layouts (e.g. one vertex versus a one-vertex compound).
        # Generate the alternative only from the verified native reference.
        state.refresh_affine_previews(reference,ref_owner)
        used_origins=set()
        def original(identity,group,host_property,message):
            witnesses=[o for o in group if o.Name==getattr(o,host_property,'')]
            if identity in origins:
                selected=next((o for o in group if o.Name==origins[identity]),None)
                if selected is None:raise RuntimeError('Migration origin is not a matching object in this archive: '+identity)
                if witnesses and witnesses!=[selected]:raise RuntimeError('Migration origin contradicts an existing original host binding')
                used_origins.add(identity);return selected
            if len(witnesses)==1:return witnesses[0]
            if len(group)==1:return group[0]
            raise RuntimeError(message+': '+identity)
        canonical={}
        for obj in sources:
            canonical.setdefault(obj.OM9SourceUUID,[]).append(obj)
        for identity,group in canonical.items():
            canonical[identity]=original(identity,group,'OM9SourceHostID','Legacy copied source identity is ambiguous')
        definition_groups={}
        for obj in document.Objects:
            identity=getattr(obj,'OM9DefinitionUUID','')
            if not obj.isDerivedFrom('App::Part') or identity not in native_definitions or 'OM9NewDefinitionUUID' in obj.PropertiesList:continue
            bound=getattr(obj,'OM9ArchiveOwner',None)
            if bound is not None and bound!=owner:continue
            if fork_namespace and bound is None:
                payloads=[m.LinkedObject if getattr(m,'OM9DefinitionMemberProxy',False) else m for m in obj.Group]
                if not payloads or any(p not in sources for p in payloads):continue
            member_namespaces={getattr(m.LinkedObject if getattr(m,'OM9DefinitionMemberProxy',False) else m,'OM9ImportNamespace','') for m in obj.Group}-{''}
            if bound is None and member_namespaces and source_namespace not in member_namespaces:continue
            if bound is None and len(member_namespaces)>1:raise RuntimeError('Legacy definition combines archive namespaces')
            definition_groups.setdefault(identity,[]).append(obj)
        definitions={};copied_definitions=[]
        for identity,group in definition_groups.items():
            selected=original(identity,group,'OM9DefinitionHostID','Legacy copied definition provenance is ambiguous')
            definitions[identity]=selected
            copied_definitions.extend((identity,o) for o in group if o!=selected)
        if set(origins)!=used_origins:raise RuntimeError('Migration origin UUID is absent from this archive binding')
        for obj in sources:
            identity=obj.OM9SourceUUID;record=records.get(identity);ref=ref_sources.get(identity)
            if record is None or ref is None:raise RuntimeError('Legacy source UUID is absent from verified snapshot')
            if hasattr(obj,'OM9SourceRecord'):
                try:stored=json.loads(obj.OM9SourceRecord)
                except ValueError:raise RuntimeError('Invalid legacy native record')
                for key,value in stored.items():
                    if key=='capability':continue
                    if key not in record or value!=record[key]:raise RuntimeError('Legacy native record differs from verified snapshot')
            if getattr(obj,'OM9SourceClass',record['class_name'])!=record['class_name']:raise RuntimeError('Legacy native class differs from verified snapshot')
            replacement=upgrade.prepare(document,obj,ref,record,upgrades[obj.Name],definitions,rebuild_previews,owner) if obj.Name in upgrades else None
            if replacement is not None:replacements.append(replacement)
            if record['capability']=='editable':
                expected='Mesh::Feature' if ref.isDerivedFrom('Mesh::Feature') else 'Part::Feature'
                if not obj.isDerivedFrom(expected) and replacement is None:raise RuntimeError('Legacy geometry representation requires an explicit object upgrade')
                if replacement is None:
                    if expected=='Part::Feature' and (obj.Shape.isNull() or not obj.Shape.isValid()):raise RuntimeError('Invalid legacy CAD geometry')
                    if expected=='Mesh::Feature' and not obj.Mesh.CountFacets:raise RuntimeError('Invalid legacy mesh geometry')
            if record['class_name']=='ON_InstanceRef':
                if obj.TypeId!=ref.TypeId and replacement is None:raise RuntimeError('Legacy instance representation requires an explicit object upgrade')
                if fork_namespace and obj.isDerivedFrom('App::Link'):
                    target=obj.LinkedObject
                    if target is None or not target.isDerivedFrom('App::Part'):raise RuntimeError('Forked reference has no supported definition target')
                    target_owner=getattr(target,'OM9ArchiveOwner',None)
                    if target_owner not in (None,owner) or (target_owner is None and target not in definitions.values() and 'OM9NewDefinitionUUID' not in target.PropertiesList):raise RuntimeError('Forked reference shares a foreign archive definition')
                if hasattr(obj,'OM9InstanceMatrix') and list(obj.OM9InstanceMatrix)!=list(ref.OM9InstanceMatrix):raise RuntimeError('Legacy native instance matrix differs from snapshot')
                plan(obj,'FloatList','OM9InstanceMatrix',list(ref.OM9InstanceMatrix))
                if not ref.isDerivedFrom('App::Link'):
                    if not hasattr(obj,'OM9DefinitionTargetUUID'):plan(obj,'String','OM9DefinitionTargetUUID',record['instance_definition_uuid'])
                    if hasattr(ref,'OM9PreviewSignature'):
                        if 'OM9PreviewStatus' in obj.PropertiesList and obj.getTypeIdOfProperty('OM9PreviewStatus')!='App::PropertyString':raise RuntimeError('Legacy preview status has an incompatible type')
                        if 'OM9PreviewSignature' in obj.PropertiesList and obj.getTypeIdOfProperty('OM9PreviewSignature')!='App::PropertyString':raise RuntimeError('Legacy preview baseline has an incompatible type')
                        current_preview=payload_fingerprint(obj)
                        if current_preview!=native_previews[identity] and current_preview!=payload_fingerprint(ref) and not rebuild_previews:raise RuntimeError('Legacy preview is unverified; explicitly rebuild derived previews')
                        preview_resets.append((obj,ref,rebuild_previews))
            metadata=state.source_metadata(ref)
            if ref.Label==ref.Name:metadata['name']=canonical[identity].Name
            for kind,name,value in [
                ('LinkHidden','OM9ArchiveOwner',owner),('String','OM9SourceHostID',canonical[identity].Name),
                ('String','OM9SourceClass',record['class_name']),('String','OM9Capability',ref.OM9Capability),
                ('String','OM9SourceRecord',json.dumps(record,sort_keys=True)),
                ('String','OM9SourceMetadata',json.dumps(metadata,sort_keys=True)),
                ('String','OM9SourcePlacement',json.dumps(state.source_placement(ref))),
                ('String','OM9SourceSignature',state.source_signature(ref,metadata_override=metadata))]:plan(obj,kind,name,value)
            if fork_namespace:plan(obj,'String','OM9ImportNamespace',namespace)
        for identity,obj in definitions.items():
            ref=ref_definitions[identity]
            plan(obj,'LinkHidden','OM9ArchiveOwner',owner)
            plan(obj,'String','OM9DefinitionHostID',obj.Name)
            plan(obj,'String','OM9DefinitionSignature',state.definition_signature(ref))
            if fork_namespace:
                for member in obj.Group:
                    payload=member.LinkedObject if getattr(member,'OM9DefinitionMemberProxy',False) else member
                    if state.archive_identity(payload) and payload not in sources:raise RuntimeError('Forked definition shares a foreign source member')
                    if state._is_new_instance(member) and getattr(member.LinkedObject,'OM9ArchiveOwner',None) not in (None,owner):raise RuntimeError('Forked definition shares a foreign new reference')
            for proxy in obj.Group:
                if not getattr(proxy,'OM9DefinitionMemberProxy',False):continue
                target=proxy.LinkedObject
                if target is None or target not in sources:raise RuntimeError('Legacy proxy has an ambiguous canonical source')
                original=next((p for p in ref.Group if getattr(p,'OM9DefinitionMemberProxy',False) and p.LinkedObject.OM9SourceUUID==target.OM9SourceUUID),None)
                if original is None:raise RuntimeError('Legacy proxy baseline requires explicit membership provenance')
                metadata=state.source_metadata(original)
                if original.Label==original.Name:metadata['name']=proxy.Name
                plan(proxy,'String','OM9ProxyMetadata',json.dumps(metadata,sort_keys=True))
        member_bindings=set();definition_targets={obj:identity for identity,obj in definitions.items()}
        occupied_names={o.Label.casefold() for o in document.Objects if o.isDerivedFrom('App::Part') and o not in [d for _,d in copied_definitions] and any(n in o.PropertiesList for n in ('OM9DefinitionUUID','OM9NewDefinitionUUID'))}
        for identity,obj in copied_definitions:
            plan(obj,'LinkHidden','OM9ArchiveOwner',owner)
            plan(obj,'String','OM9DefinitionHostID',definitions[identity].Name)
            identity_new=str(uuid.uuid4());definition_targets[obj]=identity_new
            plan(obj,'String','OM9NewDefinitionUUID',identity_new)
            plan(obj,'String','OM9NewDefinitionHostID',obj.Name)
            plan(obj,'String','OM9CopiedDefinitionSourceUUID',identity)
            name=obj.Label;index=1
            while name.casefold() in occupied_names:
                name=obj.Label+' copy '+str(index);index+=1
            if not name.strip() or len(name)>1024:raise RuntimeError('Legacy copied definition has an invalid native name')
            occupied_names.add(name.casefold())
            if name!=obj.Label:plan(obj,'String','Label',name)
            for member in obj.Group:
                proxy=getattr(member,'OM9DefinitionMemberProxy',False)
                source=member.LinkedObject if proxy else member
                if source not in sources:raise RuntimeError('Legacy copied definition has an unverified member')
                record=records[source.OM9SourceUUID];ref=ref_sources[source.OM9SourceUUID]
                if proxy:
                    if not member.isDerivedFrom('App::Link'):raise RuntimeError('Legacy copied member proxy is invalid')
                    original_proxy=next((p for p in ref_definitions[identity].Group if getattr(p,'OM9DefinitionMemberProxy',False) and p.LinkedObject.OM9SourceUUID==source.OM9SourceUUID),None)
                    if original_proxy is None:raise RuntimeError('Legacy copied proxy requires explicit membership provenance')
                    metadata=state.source_metadata(original_proxy)
                    if original_proxy.Label==original_proxy.Name:metadata['name']=member.Name
                    plan(member,'String','OM9ProxyMetadata',json.dumps(metadata,sort_keys=True))
                if member in member_bindings or 'OM9NewMemberUUID' in member.PropertiesList:continue
                member_bindings.add(member)
                plan(member,'String','OM9NewMemberUUID',str(uuid.uuid4()))
                plan(member,'String','OM9NewMemberHostID',member.Name)
                if not proxy:
                    baseline=state._source_member_geometry_signature(ref,record)
                    replacement=next((r for r in replacements if r['name']==member.Name),None)
                    current=upgrade.geometry_signature(replacement,state,record) if replacement else state._source_member_geometry_signature(member,record)
                    plan(member,'Bool','OM9BlockMemberNativeGeometry',current==baseline)
                    plan(member,'String','OM9BlockMemberGeometrySignature',baseline)
                    plan(member,'String','OM9BlockMemberSourcePlacement',json.dumps(list(ref.Placement.toMatrix().A) if hasattr(ref,'Placement') else state.IDENTITY_MATRIX))
                    if not hasattr(member,'Placement'):
                        plan(member,'Placement','OM9BlockMemberPlacement',App.Placement())
        occupied_ids={r['source_uuid'] for r in fresh['components']+fresh['records']}
        existing_new={}
        for obj in document.Objects if targets else ():
            if not obj.isDerivedFrom('App::Part') or 'OM9NewDefinitionUUID' not in obj.PropertiesList:continue
            if getattr(obj,'OM9ArchiveOwner',None) not in (None,owner):continue
            try:identity=state._new_identity(obj,'Definition')
            except RuntimeError:continue
            existing_new.setdefault(identity,[]).append(obj)
        mapped=[]
        for host_name,target_name in targets.items():
            obj=document.getObject(host_name);target=document.getObject(target_name)
            if obj not in sources or records[obj.OM9SourceUUID]['class_name']!='ON_InstanceRef' or obj.isDerivedFrom('App::Link') or not hasattr(ref_sources[obj.OM9SourceUUID],'OM9PreviewSignature'):
                raise RuntimeError('Migration target source must be a supported affine reference in this archive')
            if target is None or not target.isDerivedFrom('App::Part') or getattr(target,'OM9ArchiveOwner',None) not in (None,owner):raise RuntimeError('Migration target definition is missing or belongs to another archive')
            identity=definition_targets.get(target)
            if identity is None and 'OM9NewDefinitionUUID' in target.PropertiesList:
                identity=state._new_identity(target,'Definition')
                if identity in occupied_ids or existing_new.get(identity)!=[target]:raise RuntimeError('Migration new target identity is colliding or ambiguous')
            if identity is None:raise RuntimeError('Migration target has no verified definition identity')
            plan(obj,'String','OM9DefinitionTargetUUID',identity);mapped.append(obj)
        for kind,name,value in [('Integer','OM9ArchiveSchema',1),('Integer','OM9ArchiveMode',1),('String','OM9ArchiveHash',digest),('String','OM9ImportNamespace',namespace),('String','OM9ArchiveManifest',state.validate_manifest(fresh))]:plan(owner,kind,name,value)
        if fork_namespace:
            plan(owner,'String','OM9NamespaceOrigin',namespace_origin)
            plan(owner,'String','OM9NamespaceHostID',owner.Name)
        incoming=upgrade.dependencies(document,replacements)
        # Store host Names before removal invalidates the original Python wrappers.
        named_plans=[(obj.Name,kind,name,value) for obj,kind,name,value in plans]
        named_previews=[(obj.Name,ref,reset) for obj,ref,reset in preview_resets]
        named_mapped=[obj.Name for obj in mapped]
        App.setActiveDocument(document.Name);state._validate_creation(document)
        observer=state._preview_observer;was_busy=observer.busy
        document.openTransaction('Migrate Rhino archive provenance')
        observer.busy=True
        try:
            upgrade.replace(document,replacements,incoming)
            preview_resets=[(document.getObject(name),ref,reset) for name,ref,reset in named_previews]
            mapped=[document.getObject(name) for name in named_mapped]
            for host,kind,name,value in named_plans:
                obj=document.getObject(host)
                if name in obj.PropertiesList:
                    setattr(obj,name,value);obj.setEditorMode(name,1)
                else:state._property(obj,kind,name,value)
                if name in ('OM9DefinitionTargetUUID','Label'):obj.setEditorMode(name,0)
            for obj,ref,reset in preview_resets:
                if reset:_reset_preview(document,obj,ref,state)
                if 'OM9PreviewSignature' in obj.PropertiesList:obj.OM9PreviewSignature=state.preview_signature(obj)
                else:state._property(obj,'String','OM9PreviewSignature',state.preview_signature(obj))
            if preview_resets or replacements:
                state.refresh_affine_previews(document,owner)
            if rebuild_previews or fork_namespace:
                for obj,ref,reset in preview_resets:
                    if obj.OM9PreviewStatus!='Current canonical members':raise RuntimeError('Current legacy graph cannot rebuild a verified native preview: '+obj.Name+': '+obj.OM9PreviewStatus)
            for obj in mapped:
                if obj.OM9PreviewStatus!='Current canonical members':raise RuntimeError('Mapped legacy target cannot rebuild a verified native preview: '+obj.OM9PreviewStatus)
            document.recompute();document.commitTransaction()
        except Exception:
            document.abortTransaction();raise
        finally:observer.busy=was_busy
        return dict(objects=len(sources),definitions=len(definitions),copied_definitions=len(copied_definitions),targets=len(mapped),previews=len(preview_resets),namespace=namespace,forked_namespace=forked_namespace,upgrades=len(replacements))
    finally:
        if reference is not None:App.closeDocument(reference.Name)
        App.setActiveDocument(document.Name)


def _reset_preview(document,obj,reference,state):
    import FreeCAD as App
    placement=App.Placement(obj.Placement.toMatrix())
    if obj.isDerivedFrom('Part::Feature'):obj.Shape=reference.Shape.copy()
    elif obj.isDerivedFrom('App::Part'):
        previous=list(obj.Group);children=[]
        for source in reference.Group:
            child=document.addObject(source.TypeId,'RhinoMigratedPreview')
            if source.isDerivedFrom('Part::Feature'):child.Shape=source.Shape.copy()
            elif source.isDerivedFrom('Mesh::Feature'):child.Mesh=source.Mesh.copy()
            else:raise RuntimeError('Unsupported native migration preview child')
            child.Label=source.Label;child.Placement=source.Placement
            state._property(child,'String','OM9Capability','display-retained')
            child.ViewObject.Visibility=source.ViewObject.Visibility
            if hasattr(source.ViewObject,'ShapeColor'):child.ViewObject.ShapeColor=source.ViewObject.ShapeColor
            children.append(child)
        obj.Group=children
        for child in previous:
            if getattr(child,'OM9Capability','')=='display-retained' and not state.archive_identity(child) and not child.InList:document.removeObject(child.Name)
    else:raise RuntimeError('Unsupported legacy migration preview')
    obj.Placement=placement
