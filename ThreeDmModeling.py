"""Independent working geometry; current shapes are the only export source."""
import json
import os
import uuid

SCOPE_OMISSIONS = ('history', 'render', 'materials', 'textures', 'lights', 'layouts', 'userdata')

def validate_prepared(rows):
    if not rows:
        raise RuntimeError('3DM archive has no supported working geometry')
    for row in rows:
        kind = row.get('geometry_kind', 9)
        cad = 'brep' in row
        mesh = 'vertices' in row and 'faces' in row
        cloud = 'point_cloud_fields' in row
        if sum((cad, mesh, cloud)) != 1 or not (
            cad and kind in (1,2,3) or mesh and kind == 4 or cloud and kind == 5
        ):
            raise RuntimeError("Unsupported working object '%s' [%s]" %
                               (row.get('name', ''),row.get('class_name', 'unknown')))

def prepare_modeling(path, staging, scale=0):
    import OpenMatrix9Gui as native
    model = json.loads(native.prepareModeling3dm(os.fspath(path), staging, scale))
    validate_prepared(model['items'])
    return model

def bind_working_metadata(obj, item):
    import OpenMatrix9Gui as native
    values = [('App::PropertyString','OM9WorkingUUID',str(uuid.uuid4())),
              ('App::PropertyInteger','OM9GeometryKind',item['geometry_kind']),
              ('App::PropertyString','OM9Representation',item['representation']),
              ('App::PropertyString','OM9Operations',native.modelingCapabilities3dm(item['geometry_kind'])),
              ('App::PropertyString','OM9OriginClass',item.get('class_name',''))]
    for prop,name,value in values:
        obj.addProperty(prop,name,'3DM Working Geometry')
        setattr(obj,name,value)
        obj.setEditorMode(name,1)

def working_selection(objects):
    """Reject ambiguous selection before expanding ordinary layer containers."""
    import FreeCAD as App
    selected=list(objects)
    if not selected or App.ActiveDocument is None:
        raise RuntimeError('Select whole working objects to export')
    if len(set(selected)) != len(selected):
        raise RuntimeError('Duplicate selected object')
    if any(o is None or o.Document != App.ActiveDocument for o in selected):
        raise RuntimeError('Selection must belong to the active project')
    roots=set(selected)
    def is_container(obj):
        return any(obj.isDerivedFrom(kind) for kind in ('App::DocumentObjectGroup','App::Part','PartDesign::Body'))
    for container in selected:
        if not is_container(container): continue
        pending=list(container.Group); seen=set()
        while pending:
            child=pending.pop()
            if child in roots:
                raise RuntimeError('Select a container or its members, without overlapping selections')
            if child in seen: continue
            seen.add(child)
            if is_container(child): pending.extend(child.Group)
    expanded=[]
    visiting=set()
    def visit(obj):
        if obj in visiting: raise RuntimeError('Cyclic selection container')
        if obj.isDerivedFrom('App::DocumentObjectGroup') or (
            obj.isDerivedFrom('App::Part') and not obj.isDerivedFrom('PartDesign::Body')
        ):
            if any(hasattr(obj,name) for name in ('OM9ArchiveMode','OM9DefinitionUUID','OM9NewDefinitionUUID')):
                raise RuntimeError('Select independently editable geometry inside this archive container')
            visiting.add(obj)
            for child in obj.Group: visit(child)
            visiting.remove(obj)
        else:
            if obj in expanded: raise RuntimeError('Overlapping selection containers')
            if getattr(obj,'OM9Capability','') in ('retained','display-retained','incompatible'):
                raise RuntimeError('Retained geometry is not independent working geometry: '+obj.Label)
            expanded.append(obj)
    for obj in selected: visit(obj)
    if not expanded: raise RuntimeError('Select nonempty working geometry')
    return expanded

def stage_modeling_selection(objects, staging):
    import ThreeDm
    return ThreeDm._stage_current_geometry(working_selection(objects),staging)

def export_file(path, objects):
    import FreeCAD as App
    import OpenMatrix9Gui as native
    import ThreeDm
    import tempfile
    objects=working_selection(objects)
    native.validateDocument(App.ActiveDocument.Name)
    with tempfile.TemporaryDirectory(prefix='om9-modeling-') as staging:
        prepared=ThreeDm._stage_current_geometry(objects,staging)
        return ThreeDm._write_geometry_atomic(native,prepared,path)
