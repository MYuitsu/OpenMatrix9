"""Rhino 5 exchange; native openNURBS/OCC geometry and transactional host edits."""
import json
import os
import tempfile
import hashlib
import FreeCAD as App
import FreeCADGui as Gui


def import_file(path, document=None, scale=0, mode='geometry'):
    import OpenMatrix9Gui as native
    import Part
    import Mesh
    doc = document or App.ActiveDocument
    if doc is None:
        raise RuntimeError("Open an active project before importing 3DM")
    with tempfile.TemporaryDirectory(prefix="om9-3dm-") as staging:
        if mode not in ('geometry', 'preserve'):
            raise RuntimeError('Invalid 3DM import mode')
        archive = None
        if mode == 'preserve':
            archive = json.loads(native.prepare3dmArchive(os.fspath(path), staging, scale, 1))
            model = dict(items=archive['prepared_geometry'])
        else:
            model = json.loads(native.read3dm(os.fspath(path), staging, scale))
        # Finish conversion before starting the single Undo transaction.
        prepared = []
        for item in model["items"]:
            item["tolerance"] = item.get('tolerance', model.get('tolerance', 1e-6))
            if "brep" in item:
                geometry = Part.Shape()
                geometry.read(item["brep"])
                if geometry.isNull() or not geometry.isValid():
                    raise RuntimeError("Invalid imported BRep")
            else:
                geometry = Mesh.Mesh()
                points = [App.Vector(*p) for p in item["vertices"]]
                for a, b, c, d in item["faces"]:
                    geometry.addFacet(points[a], points[b], points[c])
                    if d != c:
                        geometry.addFacet(points[a], points[c], points[d])
            prepared.append((item, geometry))
        if archive is not None:
            from ThreeDmArchiveState import validate_prepared
            validate_prepared(archive)
            archive['host_geometry'] = prepared
            objects = native.commit3dm(doc.Name, archive)
            App.Console.PrintMessage('3DM: %d editable objects, %d source-retained records. Source archive stored in FCStd; merged preservation export is pending.\n' % (len(prepared), len(archive['retained_records'])))
            return objects
        return native.commit3dm(doc.Name, prepared)


def _insert_prepared(name, prepared):
    """Bind native prepared geometry to FreeCAD properties; C++ owns the transaction."""
    doc = App.getDocument(name)
    if isinstance(prepared, dict):
        from ThreeDmArchiveState import bind_archive
        return bind_archive(doc, prepared)
    objects = []
    groups = {}
    for item, geometry in prepared:
        obj = doc.addObject("Part::Feature" if "brep" in item else "Mesh::Feature", "RhinoObject")
        if "brep" in item:
            obj.Shape = geometry
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
        obj.ViewObject.ShapeColor = obj.OM9Color[:3]
        obj.ViewObject.Visibility = item["visible"]
        obj.ViewObject.Selectable = not item["locked"]
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


def export_file(path, objects, geometry_only=False):
    import OpenMatrix9Gui as native
    import Part
    def geometric_selection(selected):
        for obj in selected:
            if obj.isDerivedFrom("App::DocumentObjectGroup"):
                yield from geometric_selection(obj.Group)
            else:
                yield obj
    selected_objects = list(objects)
    geometry = [obj for obj in selected_objects if not obj.isDerivedFrom("App::DocumentObjectGroup")]
    groups = [obj for obj in selected_objects if obj.isDerivedFrom("App::DocumentObjectGroup")]
    objects = list(dict.fromkeys(geometric_selection(geometry + groups)))
    if any(hasattr(obj, 'OM9ArchiveMode') or getattr(obj, 'OM9Capability', '') in ('retained', 'display-retained', 'incompatible') for obj in objects):
        raise RuntimeError('Source-retained content requires merged preservation export, which is not implemented yet')
    if any(hasattr(obj, 'OM9SourceUUID') for obj in objects):
        if not geometry_only and not native.archiveLegacyExportAllowed(1):
            raise RuntimeError('Preserved source data would be omitted. Explicitly choose geometry-only export for editable objects')
        App.Console.PrintWarning('3DM geometry-only export omits source tables, retained records, history and userdata.\n')
    if not objects:
        raise RuntimeError("Select whole objects to export")
    if any(obj.Document != App.ActiveDocument for obj in objects):
        raise RuntimeError("Selection must belong to the active project")
    native.validateDocument(App.ActiveDocument.Name)
    selected = set(objects)
    if any(parent in selected and (parent.isDerivedFrom("PartDesign::Body") or parent.isDerivedFrom("App::Part"))
           for obj in objects for parent in obj.InListRecursive):
        raise RuntimeError("Select a container or its contained objects, without overlapping selections")
    target = os.path.abspath(os.fspath(path))
    with tempfile.TemporaryDirectory(prefix="om9-3dm-") as staging:
        items = []
        tolerance = 1e-6
        for index, obj in enumerate(objects):
            color = getattr(obj.ViewObject, "ShapeColor", getattr(obj, "OM9Color", (0.7, 0.7, 0.7)))
            item = dict(name=obj.Label, layer=getattr(obj, "OM9LayerPath", "Default"),
                        color=[round(c * 255) for c in color[:3]],
                        visible=obj.ViewObject.Visibility, locked=getattr(obj, "OM9Locked", False))
            if obj.isDerivedFrom("Mesh::Feature"):
                points, triangles = obj.Mesh.Topology
                matrix = (obj.getGlobalPlacement() * obj.Placement.inverse()).toMatrix()
                signature = hashlib.sha256(repr(obj.Mesh.Topology).encode()).hexdigest()
                if getattr(obj, "OM9MeshSignature", "") == signature:
                    original = json.loads(obj.OM9MeshArchive)
                    item["vertices"] = [list(matrix.multVec(App.Vector(*p))) for p in original["vertices"]]
                    item["faces"] = original["faces"]
                else:
                    item["vertices"] = [list(matrix.multVec(p)) for p in points]
                    item["faces"] = [[a, b, c, c] for a, b, c in triangles]
            else:
                shape = Part.getShape(obj, "", needSubElement=False, transform=True)
                if shape is None or shape.isNull():
                    raise RuntimeError("Unsupported selected object: " + obj.Label)
                parent_matrix = (obj.getGlobalPlacement() * obj.Placement.inverse()).toMatrix()
                shape.transformShape(parent_matrix, False)
                tolerance = max(tolerance, getattr(obj, "OM9Tolerance", 0.0), shape.getTolerance(1))
                filename = os.path.join(staging, str(index) + ".brep")
                shape.exportBrep(filename)
                item["brep"] = filename
            items.append(item)
        # Stage beside the destination so replacement is atomic on its filesystem.
        descriptor, temporary = tempfile.mkstemp(prefix=".om9-", suffix=".3dm", dir=os.path.dirname(target))
        os.close(descriptor)
        try:
            native.write3dm(json.dumps(dict(items=items, tolerance=tolerance)), temporary)
            os.replace(temporary, target)
        finally:
            if os.path.exists(temporary):
                os.unlink(temporary)


def export_selection(path):
    selected = Gui.Selection.getSelectionEx()
    if any(s.SubElementNames for s in selected):
        raise RuntimeError("Select whole objects, without subelements")
    export_file(path, [s.Object for s in selected])


def export_named(path, document_name, names):
    doc = App.getDocument(document_name)
    objects = [doc.getObject(name) for name in names]
    if any(obj is None for obj in objects):
        raise RuntimeError("An export object no longer exists")
    export_file(path, objects)


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


def export(objects, filename):
    """FreeCAD's registered File > Export entry point; archive version 5."""
    export_file(filename, objects)
