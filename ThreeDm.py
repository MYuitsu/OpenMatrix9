"""Rhino 5 exchange; native openNURBS/OCC geometry and transactional host edits."""
import json
import os
import tempfile
import hashlib
import FreeCAD as App
import FreeCADGui as Gui


def import_file(path, document=None, scale=0, mode='geometry'):
    import OpenMatrix9Gui as native
    doc = document or App.ActiveDocument
    if doc is None:
        raise RuntimeError("Open an active project before importing 3DM")
    with tempfile.TemporaryDirectory(prefix="om9-3dm-") as staging:
        prepared = _prepare_import(path, staging, scale, mode)
        objects = native.commit3dm(doc.Name, prepared)
        if mode == 'preserve':
            App.Console.PrintMessage('3DM: %d editable objects, %d source-retained records. Source archive stored in FCStd; use structural preservation export.\n' % (len(prepared['host_geometry']), len(prepared['retained_records'])))
        elif mode == 'modeling':
            App.Console.PrintMessage('3DM: %d independent working objects. History, render data, materials, textures, lights, layouts and userdata omitted.\n' % len(objects))
        return objects


def _prepare_import(path, staging, scale, mode):
    """Convert verified source data without mutating a user document."""
    import OpenMatrix9Gui as native, Part, Mesh
    if mode not in ('geometry', 'preserve', 'modeling'):raise RuntimeError('Invalid 3DM import mode')
    archive = None
    if mode == 'preserve':
        archive = json.loads(native.prepare3dmArchive(os.fspath(path), staging, scale, 1))
        model = dict(items=archive['prepared_geometry'] + archive.get('definition_geometry', []))
    elif mode == 'modeling':
        from ThreeDmModeling import prepare_modeling
        model = prepare_modeling(path, staging, scale)
    else:model = json.loads(native.read3dm(os.fspath(path), staging, scale))
    # Bound all staged native clouds before constructing host properties.
    cloud_rows=[row for row in model['items'] if 'point_cloud_fields' in row]
    if archive is not None:
        cloud_rows += [row for preview in archive.get('affine_previews',[]) for row in preview.get('items',[]) if 'point_cloud_fields' in row]
    total=sum(os.path.getsize(row['point_cloud_fields']) for row in cloud_rows)
    if archive is not None:total+=sum(os.path.getsize(row['file']) for row in archive.get('point_cloud_data',{}).values())
    if total>512*1024*1024:raise RuntimeError('Combined prepared PointCloud fields exceed512MiB')
    prepared = []
    for item in model["items"]:
        item["tolerance"] = item.get('tolerance', model.get('tolerance', 1e-6))
        if "brep" in item:
            geometry = Part.Shape()
            geometry.read(item["brep"])
            if geometry.isNull() or not geometry.isValid():raise RuntimeError("Invalid imported BRep")
        elif 'point_cloud_fields' in item:
            import ThreeDmPointCloud
            geometry=ThreeDmPointCloud.read_data(item['point_cloud_fields'],item['point_cloud_sha256'])
        else:
            geometry = Mesh.Mesh()
            points = [App.Vector(*p) for p in item["vertices"]]
            for a, b, c, d in item["faces"]:
                geometry.addFacet(points[a], points[b], points[c])
                if d != c:geometry.addFacet(points[a], points[c], points[d])
        prepared.append((item, geometry))
    if archive is not None:
        import ThreeDmPointCloud
        if sum(os.path.getsize(row['file']) for row in archive.get('point_cloud_data',{}).values())>512*1024*1024:
            raise RuntimeError('Combined prepared PointCloud fields exceed512MiB')
        archive['host_point_clouds']={identity:ThreeDmPointCloud.read_data(row['file'],row['sha256'])
                                      for identity,row in archive.get('point_cloud_data',{}).items()}
        archive['host_previews'] = {}
        for preview in archive.get('affine_previews', []):
            geometries = []
            for item in preview.get('items', []):
                if 'brep' in item:
                    geometry = Part.Shape()
                    geometry.read(item['brep'])
                    if geometry.isNull() or not geometry.isValid():raise RuntimeError('Invalid native affine preview')
                elif 'point_cloud_fields' in item:
                    geometry=ThreeDmPointCloud.read_data(item['point_cloud_fields'],item['point_cloud_sha256'])
                else:
                    geometry = Mesh.Mesh()
                    points = [App.Vector(*p) for p in item['vertices']]
                    for a,b,c,d in item['faces']:
                        geometry.addFacet(points[a],points[b],points[c])
                        if d!=c:geometry.addFacet(points[a],points[c],points[d])
                geometries.append((item,geometry))
            if geometries:archive['host_previews'][preview['source_uuid']] = geometries
        from ThreeDmArchiveState import validate_prepared
        validate_prepared(archive)
        archive['host_geometry'] = prepared
        return archive
    return prepared


def _insert_prepared(name, prepared):
    """Bind native prepared geometry to FreeCAD properties; C++ owns the transaction."""
    doc = App.getDocument(name)
    if isinstance(prepared, dict):
        from ThreeDmArchiveState import bind_archive
        return bind_archive(doc, prepared)
    objects = []
    groups = {}
    for item, geometry in prepared:
        obj = doc.addObject("Part::Feature" if "brep" in item else "App::FeaturePython" if 'point_cloud_fields' in item else "Mesh::Feature", "RhinoObject")
        if "brep" in item:
            obj.Shape = geometry
        elif 'point_cloud_fields' in item:
            import ThreeDmPointCloud
            ThreeDmPointCloud.bind(obj,geometry)
            obj.addProperty('App::PropertyPlacement','OM9BlockMemberPlacement','Rhino PointCloud')
        else:
            obj.Mesh = geometry
            obj.addProperty("App::PropertyString", "OM9MeshArchive", "Rhino 5")
            obj.OM9MeshArchive = json.dumps(dict(vertices=item["vertices"], faces=item["faces"]))
            obj.addProperty("App::PropertyString", "OM9MeshSignature", "Rhino 5")
            obj.OM9MeshSignature = hashlib.sha256(repr(obj.Mesh.Topology).encode()).hexdigest()
        obj.Label = item["name"] or obj.Name
        obj.addProperty("App::PropertyFloat", "OM9Tolerance", "Rhino 5")
        obj.OM9Tolerance = item["tolerance"]
        obj.addProperty("App::PropertyString", "OM9LayerPath", "Rhino 5")
        obj.OM9LayerPath = item["layer"]
        obj.addProperty("App::PropertyBool", "OM9Locked", "Rhino 5")
        obj.OM9Locked = item["locked"]
        obj.addProperty("App::PropertyColor", "OM9Color", "Rhino 5")
        obj.OM9Color = tuple(c / 255 for c in item["color"])
        if hasattr(obj.ViewObject,'ShapeColor'):obj.ViewObject.ShapeColor = obj.OM9Color[:3]
        if hasattr(obj.ViewObject,'LineColor'):obj.ViewObject.LineColor = obj.OM9Color[:3]
        if hasattr(obj.ViewObject,'Deviation'):obj.ViewObject.Deviation=0.05
        if hasattr(obj.ViewObject,'AngularDeflection'):obj.ViewObject.AngularDeflection=5.0
        if obj.isDerivedFrom('Part::Feature'):
            obj.ViewObject.addProperty('App::PropertyInteger','OM9IsoCurveDensity','Rhino display','Display-only Rhino isocurve density (-1: boundary only)')
            obj.ViewObject.OM9IsoCurveDensity=int(item.get('wire_density',1))
        obj.ViewObject.Visibility = item["visible"]
        if item.get('working_mode'):
            from ThreeDmModeling import bind_working_metadata
            bind_working_metadata(obj,item)
        if hasattr(obj.ViewObject,'Selectable'):obj.ViewObject.Selectable = not item["locked"]
        parent = None
        path_parts = item["layer"].split("::") if item["layer"] else []
        for depth, label in enumerate(path_parts):
            key = "::".join(path_parts[:depth + 1])
            if key not in groups:
                group = doc.addObject("App::DocumentObjectGroup", "RhinoLayer")
                group.Label = label
                groups[key] = group
                if parent:
                    parent.addObject(group)
            parent = groups[key]
        if parent:
            parent.addObject(obj)
        objects.append(obj)
    doc.recompute()
    return objects


def export_file(path, objects, geometry_only=False, modeling=False):
    import OpenMatrix9Gui as native
    import Part
    if type(geometry_only) is not bool:raise RuntimeError('Geometry-only export policy must be boolean')
    if type(modeling) is not bool:raise RuntimeError('Modeling export policy must be boolean')
    if modeling:
        from ThreeDmModeling import export_file as export_working
        return export_working(path,objects)
    def geometric_selection(selected):
        for obj in selected:
            if obj.isDerivedFrom("App::DocumentObjectGroup") and not any(hasattr(obj,field) for field in ('OM9ArchiveMode','OM9DefinitionUUID','OM9NewDefinitionUUID','OM9SourceUUID')):
                yield from geometric_selection(obj.Group)
            else:
                yield obj
    selected_objects = list(objects)
    geometry = [obj for obj in selected_objects if not obj.isDerivedFrom("App::DocumentObjectGroup")]
    groups = [obj for obj in selected_objects if obj.isDerivedFrom("App::DocumentObjectGroup")]
    objects = list(dict.fromkeys(geometric_selection(geometry + groups)))
    if not objects:
        raise RuntimeError("Select whole objects to export")
    if any(obj.Document != App.ActiveDocument for obj in objects):
        raise RuntimeError("Selection must belong to the active project")
    native.validateDocument(App.ActiveDocument.Name)
    selected = set(objects)
    if any(parent in selected and (parent.isDerivedFrom("PartDesign::Body") or parent.isDerivedFrom("App::Part"))
           for obj in objects for parent in obj.InListRecursive):
        raise RuntimeError("Select a container or its contained objects, without overlapping selections")
    preserved=any(any(hasattr(obj,field) for field in ('OM9SourceUUID','OM9ArchiveMode','OM9DefinitionUUID','OM9NewDefinitionUUID','OM9NewInstanceUUID','OM9NewGeometryUUID','OM9NewMemberUUID')) or getattr(obj,'OM9DefinitionMemberProxy',False) or getattr(obj,'OM9Capability','') in ('retained','display-retained','incompatible') for obj in objects)
    if preserved and not geometry_only:return export_preserved(path,objects)
    if preserved:
        App.Console.PrintWarning('3DM geometry-only export omits source tables, block definitions, history and userdata; it exports supported current geometry.\n')
        import ThreeDmPointCloud
        if any(getattr(obj,'OM9SourceClass','')=='ON_InstanceRef' or hasattr(obj,'OM9NewInstanceUUID') or ThreeDmPointCloud.is_adapter(obj) for obj in objects):
            # Flatten a verified current native graph, not a potentially stale
            # display cache. Native mesh remains mesh after affine expansion.
            with tempfile.TemporaryDirectory(prefix='om9-flat-') as staging:
                selected_archive=os.path.join(staging,'selected.3dm')
                from ThreeDmArchiveState import preservation_request
                request=preservation_request(objects,staging)
                native.writeGeometryStaging3dm(json.dumps(request),selected_archive)
                prepared=json.loads(native.read3dm(selected_archive,staging))
                if any(not ('brep' in row or 'point_cloud_fields' in row or ('vertices' in row and 'faces' in row)) for row in prepared['items']):raise RuntimeError('Geometry-only export cannot represent retained native records; select editable geometry')
                # The native reader also returns source identity and display
                # metadata. This explicit geometry-only route omits those
                # fields; the Rust writer validates the remaining typed payload.
                geometry_keys={'name','layer','visible','locked','color','brep','vertices','faces',
                               'point_cloud_fields','point_cloud_sha256','point_cloud_transform'}
                prepared=dict(tolerance=prepared['tolerance'],items=[
                    {key:value for key,value in row.items() if key in geometry_keys}
                    for row in prepared['items']])
                return _write_geometry_atomic(native,prepared,path)
        if any(hasattr(obj,'OM9ArchiveMode') or hasattr(obj,'OM9DefinitionUUID') or hasattr(obj,'OM9NewDefinitionUUID') or getattr(obj,'OM9Capability','') in ('retained','display-retained','incompatible') for obj in objects):
            raise RuntimeError('Geometry-only export requires editable geometry or a supported placed block')
    with tempfile.TemporaryDirectory(prefix="om9-3dm-") as staging:
        return _write_geometry_atomic(native,_stage_current_geometry(objects,staging),path)


def _stage_current_geometry(objects,staging):
    import Part
    items = []
    tolerance = 1e-6
    for index, obj in enumerate(objects):
        color = getattr(obj.ViewObject, "ShapeColor", getattr(obj, "OM9Color", (0.7, 0.7, 0.7)))
        item = dict(name=obj.Label, layer=getattr(obj, "OM9LayerPath", "Default"),
                    color=[round(c * 255) for c in color[:3]],
                    visible=obj.ViewObject.Visibility, locked=getattr(obj, "OM9Locked", False))
        import ThreeDmPointCloud
        if ThreeDmPointCloud.is_adapter(obj):
            if getattr(obj,'OM9Capability','')=='display-retained':raise RuntimeError('Derived PointCloud preview is not an independent export source')
            item.update(ThreeDmPointCloud.stage(obj,None,staging,index))
            item['point_cloud_transform']=ThreeDmPointCloud.placement_matrix(obj)
        elif obj.isDerivedFrom("Mesh::Feature"):
            topology = obj.Mesh.Topology
            points, triangles = topology
            matrix = (obj.getGlobalPlacement() * obj.Placement.inverse()).toMatrix()
            stored_signature = getattr(obj, "OM9MeshSignature", "")
            signature = hashlib.sha256(repr(topology).encode()).hexdigest() if stored_signature else None
            if stored_signature and stored_signature == signature:
                original = json.loads(obj.OM9MeshArchive)
                item["vertices"] = [list(matrix.multVec(App.Vector(*p))) for p in original["vertices"]]
                item["faces"] = original["faces"]
            else:
                item["vertices"] = [list(matrix.multVec(p)) for p in points]
                item["faces"] = [[a, b, c, c] for a, b, c in triangles]
        else:
            if getattr(obj,'OM9DefinitionMemberProxy',False) and obj.isDerivedFrom('App::Link') and getattr(obj,'ElementCount',0)!=0:
                raise RuntimeError('Geometry-only signed CAD proxy does not support link arrays')
            shape = Part.getShape(obj, "", needSubElement=False, transform=True)
            if shape is None or shape.isNull():
                raise RuntimeError("Unsupported selected object: " + obj.Label)
            if getattr(obj,'OM9DefinitionMemberProxy',False) and obj.isDerivedFrom('App::Link'):
                member=obj.LinkedObject
                if member is None or not member.isDerivedFrom('Part::Feature'):
                    raise RuntimeError('Geometry-only CAD proxy requires an editable CAD member')
                # OCC's rendered reflected solid retains the member's
                # original orientation. Native signed placement also carries
                # the determinant parity; reverse topology, not just a cache.
                if member.Shape.Solids and obj.ScaleVector.x*obj.ScaleVector.y*obj.ScaleVector.z<0:
                    shape.reverse()
            # App::Link exposes Placement but not getGlobalPlacement.
            # Reuse the verified physical-parent placement resolver; the
            # shape already includes the link's own scale and placement.
            from ThreeDmArchiveState import source_placement
            parent_matrix = App.Matrix(*source_placement(obj)) * obj.Placement.inverse().toMatrix()
            shape.transformShape(parent_matrix, False)
            tolerance = max(tolerance, getattr(obj, "OM9Tolerance", 0.0), shape.getTolerance(1))
            filename = os.path.join(staging, str(index) + ".brep")
            shape.exportBrep(filename)
            item["brep"] = filename
        items.append(item)
    return dict(items=items,tolerance=tolerance)


def _write_geometry_atomic(native,prepared,path):
    if not prepared.get('items'):raise RuntimeError('Select nonempty editable geometry')
    target=os.path.abspath(os.fspath(path))
    # Stage beside the destination so replacement is atomic on its filesystem.
    descriptor,temporary=tempfile.mkstemp(prefix='.om9-',suffix='.3dm',dir=os.path.dirname(target));os.close(descriptor)
    try:
        native.write3dm(json.dumps(prepared),temporary)
        os.replace(temporary,target)
    finally:
        if os.path.exists(temporary):os.unlink(temporary)


def copy_definition(definition, name=None, copy_targets=True):
    from ThreeDmArchiveState import copy_definition as copy
    return copy(definition,name,copy_targets)


def migrate_archive(owner, rebuild_previews=False, origins=None, targets=None, fork_namespace=False, upgrades=None):
    from ThreeDmMigration import migrate_archive as migrate
    return migrate(owner,rebuild_previews,origins,targets,fork_namespace,upgrades)


def create_definition(document, members, name):
    from ThreeDmArchiveState import create_definition as create
    return create(document,members,name)

def create_instance(definition, placement=None, name='Rhino block instance'):
    from ThreeDmArchiveState import create_instance as create
    return create(definition,placement,name)

def export_preserved(path, objects):
    import OpenMatrix9Gui as native
    from ThreeDmArchiveState import preservation_request
    objects=list(dict.fromkeys(objects))
    if not objects or App.ActiveDocument is None:raise RuntimeError('Select whole CAD, mesh or source-backed objects')
    if any(obj.Document!=App.ActiveDocument for obj in objects):raise RuntimeError('Selection must belong to the active project')
    native.validateDocument(App.ActiveDocument.Name)
    with tempfile.TemporaryDirectory(prefix='om9-preserved-') as staging:
        request=preservation_request(objects,staging)
        native.writePreserved3dm(json.dumps(request),os.path.abspath(os.fspath(path)))

def export_selection(path, geometry_only=False, modeling=False):
    selected = Gui.Selection.getSelectionEx()
    if any(s.SubElementNames for s in selected):
        raise RuntimeError("Select whole objects, without subelements")
    export_file(path, [s.Object for s in selected],geometry_only=geometry_only,modeling=modeling)


def export_named(path, document_name, names, geometry_only=False, modeling=False):
    doc = App.getDocument(document_name)
    objects = [doc.getObject(name) for name in names]
    if any(obj is None for obj in objects):
        raise RuntimeError("An export object no longer exists")
    export_file(path, objects,geometry_only=geometry_only,modeling=modeling)


def insert(filename, document_name):
    """FreeCAD's registered File > Import entry point."""
    return import_file(filename, App.getDocument(document_name), mode='preserve')


def open(filename):
    """FreeCAD's registered File > Open entry point for exchange documents."""
    doc = App.newDocument(os.path.splitext(os.path.basename(filename))[0])
    try:
        import_file(filename, doc, mode='preserve')
        return doc
    except Exception:
        App.closeDocument(doc.Name)
        raise


def export(objects, filename, geometry_only=False):
    """FreeCAD's registered File > Export entry point; archive version 5."""
    export_file(filename, objects,geometry_only=geometry_only)
