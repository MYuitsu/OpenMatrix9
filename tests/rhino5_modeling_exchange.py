# Installed Rhino5 process, owned isolated document; generated fixtures only.
import os,json,traceback,codecs,math,time,hashlib,ntpath
import Rhino,System
out=os.environ['OM9_RHINO5_OUTPUT'];report={'checks':[],'cases':[],'open_attempts':[],'ok':False}
def check(name,value):
    report['checks'].append({'name':name,'passed':bool(value)})
    if not value:raise RuntimeError(name)
def digest(path):
    with open(path,'rb') as stream:return hashlib.sha256(stream.read()).hexdigest()

def rhino_file_command(action,path):
    # Rhino5's command filename parser needs backslash directory separators.
    # File3dm accepts slash paths, so a successful SDK read does not test this.
    return u'_-{0} "{1}" _Enter'.format(action,ntpath.normpath(path))
def facts(geometry):
    bounds=geometry.GetBoundingBox(True)
    check('Rhino finite valid geometry bounds',bounds.IsValid)
    row={'class':geometry.GetType().FullName,'bounds':[bounds.Min.X,bounds.Min.Y,bounds.Min.Z,bounds.Max.X,bounds.Max.Y,bounds.Max.Z]}
    check('Rhino geometry valid '+row['class'],geometry.IsValid)
    if isinstance(geometry,Rhino.Geometry.Curve):row['length']=geometry.GetLength()
    if isinstance(geometry,Rhino.Geometry.PointCloud):row['points']=[[p.X,p.Y,p.Z] for p in geometry.GetPoints()]
    if isinstance(geometry,Rhino.Geometry.Point):row['point']=[geometry.Location.X,geometry.Location.Y,geometry.Location.Z]
    if isinstance(geometry,Rhino.Geometry.Mesh):row['mesh_vertices']=geometry.Vertices.Count;row['mesh_faces']=geometry.Faces.Count
    if isinstance(geometry,Rhino.Geometry.InstanceReferenceGeometry):
        row['definition_id']=str(geometry.ParentIdefId)
        row['transform']=[geometry.Xform[i,j] for i in range(4) for j in range(4)]
    if isinstance(geometry,Rhino.Geometry.Brep):
        row['faces']=geometry.Faces.Count
        row['solid']=geometry.IsSolid
        area=Rhino.Geometry.AreaMassProperties.Compute(geometry)
        if area:row['area']=area.Area;area.Dispose()
        if geometry.IsSolid:
            volume=Rhino.Geometry.VolumeMassProperties.Compute(geometry)
            check('Rhino solid volume measurement available',volume is not None)
            row['volume']=volume.Volume;volume.Dispose()
    return row

def object_facts(obj):
    row=facts(obj.Geometry)
    row['object_id']=str(obj.Attributes.ObjectId)
    row['name']=unicode(obj.Attributes.Name or '')
    row['definition_member']=obj.Attributes.Mode==Rhino.DocObjects.ObjectMode.InstanceDefinitionObject
    return row

def geometry_bounds(row):
    if 'definition_id' in row:
        check('Rhino instance resolved geometry bounds available', 'instance_geometry_bounds' in row)
        return row['instance_geometry_bounds']
    return row['bounds']

def document_definitions(doc):
    definitions={}
    for index in range(doc.InstanceDefinitions.Count):
        definition=doc.InstanceDefinitions[index]
        if definition is not None and not definition.IsDeleted:
            definitions[str(definition.Id)]=[str(obj.Id) for obj in definition.GetObjects()]
    return definitions

def model_definitions(model):
    return {str(definition.Id):[str(value) for value in definition.GetObjectIds()]
            for definition in model.InstanceDefinitions}

def resolved_geometry_bounds(geometry,objects,definitions,transform,active):
    if isinstance(geometry,Rhino.Geometry.InstanceReferenceGeometry):
        definition_id=str(geometry.ParentIdefId)
        check('Rhino instance definition exists',definition_id in definitions)
        check('Rhino instance definition has no cycle',definition_id not in active)
        member_ids=definitions[definition_id]
        check('Rhino instance definition has members',bool(member_ids))
        check('Rhino instance members are unique',len(set(member_ids))==len(member_ids))
        check('Rhino instance member geometry available',all(value in objects for value in member_ids))
        check('Rhino instance members have definition mode',all(objects[value].Attributes.Mode==Rhino.DocObjects.ObjectMode.InstanceDefinitionObject for value in member_ids))
        combined=transform*geometry.Xform
        boxes=[resolved_geometry_bounds(objects[value].Geometry,objects,definitions,combined,active+[definition_id]) for value in member_ids]
        return [min(box[index] for box in boxes) if index<3 else max(box[index] for box in boxes) for index in range(6)]
    # Rhino5 documents this overload as accurate transformed geometry bounds;
    # it does not alter geometry and avoids the InstanceReference bbox cache.
    box=geometry.GetBoundingBox(transform)
    check('Rhino instance transformed member bounds valid',box.IsValid)
    return [box.Min.X,box.Min.Y,box.Min.Z,box.Max.X,box.Max.Y,box.Max.Z]

def object_rows(objects,definitions):
    objects=list(objects)
    lookup={str(obj.Attributes.ObjectId):obj for obj in objects}
    check('Rhino measured object identities unique',len(lookup)==len(objects))
    rows=[]
    for obj in objects:
        row=object_facts(obj)
        if 'definition_id' in row:
            row['instance_geometry_bounds']=resolved_geometry_bounds(obj.Geometry,lookup,definitions,Rhino.Geometry.Transform.Identity,[])
            row['instance_definition_objects']=sorted(definitions[row['definition_id']])
        rows.append(row)
    return rows

def root_rows(rows):
    return [row for row in rows if not row.get('definition_member',False)]

def export_rows(case_name,rows):
    if case_name=='current-ring':
        selected=[row for row in rows if row.get('name')=='NewRingCircle']
        check('Rhino ring Selected subset is the new circle',len(selected)==1)
        return selected
    return rows

def document_objects(doc):
    return document_helpers['document_objects'](doc)

def check_edited_curves(rows):
    lines=[row for row in rows if row['class']=='Rhino.Geometry.LineCurve']
    circles=[row for row in rows if row['class']=='Rhino.Geometry.ArcCurve']
    check('Rhino identifies moved curve independently of object order',len(lines)==1)
    check('Rhino identifies new circle independently of object order',len(circles)==1)
    line=lines[0];circle=circles[0]
    check('Rhino sees current moved curve',abs(line['bounds'][0]-20)<1e-7 and abs(line['bounds'][3]-30)<1e-7)
    check('Rhino sees new exact circle',abs(circle['length']-6*math.pi)<1e-6)

def saved_pairs(rows,received):
    original={row['object_id']:row for row in rows}
    saved={row['object_id']:row for row in received}
    check('Rhino object identities unique before SaveAs',len(original)==len(rows))
    check('Rhino object identities unique after SaveAs',len(saved)==len(received))
    check('Rhino saved object identities preserved',set(original)==set(saved))
    return [(row,saved[row['object_id']]) for row in rows]

try:
    owner_path=os.path.join(out,'owned-pid.txt')
    for _attempt in range(40):
        if os.path.isfile(owner_path):break
        time.sleep(0.05)
    expected_pid=int(open(owner_path).read().strip())
    actual_pid=System.Diagnostics.Process.GetCurrentProcess().Id
    check('application process owned by this test launcher',actual_pid==expected_pid)
    report['application_pid']=actual_pid
    config=json.load(codecs.open(os.path.join(out,'config.json'),'r','utf-8-sig'))
    check('document enumerator helper matches prepared checksum',digest(config['document_helper'])==config['document_helper_sha256'])
    document_helpers={'__name__':'document_helpers','__file__':config['document_helper']}
    execfile(config['document_helper'],document_helpers,document_helpers)
    initial_doc=Rhino.RhinoDoc.ActiveDoc
    check('application starts with isolated empty document',initial_doc is not None and not initial_doc.Path and len(document_objects(initial_doc))==0)
    report['requested_cases']=[entry['name'] for entry in config['files']]
    report['diagnostic_subset']=config.get('diagnostic_subset')
    check('application probe script matches prepared checksum',digest(config['script'])==config['script_sha256'])
    report['source_bindings']={'script_sha256':config['script_sha256'],'launcher_sha256':config['launcher_sha256'],'host_module_sha256':config['host_module_sha256'],'document_helper_sha256':config['document_helper_sha256']}
    report['rhino_version']=str(Rhino.RhinoApp.Version)
    for entry in config['files']:
        source=entry['input'];target=os.path.join(out,entry['name']+'.rhino5.3dm')
        check('Rhino input checksum '+entry['name'],digest(source)==entry['input_sha256'])
        attempt={'name':entry['name'],'input':source,'header':open(source,'rb').read(32),
                 'document_before':unicode(Rhino.RhinoDoc.ActiveDoc.Path or ''),
                 'document_modified_before':bool(Rhino.RhinoDoc.ActiveDoc.Modified)}
        report['open_attempts'].append(attempt)
        attempt['command']=rhino_file_command('Open',source)
        opened=Rhino.RhinoApp.RunScript(attempt['command'],config.get('echo_commands',True))
        attempt['run_script_result']=bool(opened)
        attempt['last_command_result']=str(Rhino.Commands.Command.LastCommandResult)
        attempt['document_after']=unicode(Rhino.RhinoDoc.ActiveDoc.Path or '')
        attempt['document_modified_after']=bool(Rhino.RhinoDoc.ActiveDoc.Modified)
        attempt['command_history_tail']=unicode(Rhino.RhinoApp.CommandHistoryWindowText)[-12000:]
        if not opened:
            model=Rhino.FileIO.File3dm.Read(source)
            attempt['file3dm_read']=model is not None
            if model is not None:
                try:attempt['file3dm_rows']=[object_facts(o) for o in model.Objects]
                finally:model.Dispose()
        check('Rhino Open '+entry['name'],opened)
        doc=Rhino.RhinoDoc.ActiveDoc
        doc_objects=document_objects(doc)
        all_rows=object_rows(doc_objects,document_definitions(doc))
        rows=root_rows(all_rows)
        case={'name':entry['name'],'input':source,'input_sha256':entry['input_sha256'],'output':target,'rows':rows,'all_rows':all_rows}
        report['cases'].append(case)
        check('Rhino object count '+entry['name'],len(rows)==entry['count'])
        check('Rhino document millimeters '+entry['name'],doc.ModelUnitSystem==Rhino.UnitSystem.Millimeters)
        if entry['name']=='edited-and-new':
            check_edited_curves(rows)
        if entry['name']=='edited-cloud':check('Rhino sees current cloud fields',rows[0]['points']==[[32,23,34],[35,26,37]])
        if entry['name']=='placed-cad':check('Rhino sees current physical parent placement',abs(rows[0]['bounds'][0]-120)<1e-7 and abs(rows[0]['bounds'][1]-5)<1e-7)
        if entry['name']=='source-and-link':check('Rhino source and placed copy distinct',[round(r['bounds'][0],6) for r in sorted(rows,key=lambda r:r['bounds'][0])]==[120,200])
        if entry['name']=='current-point':check('Rhino current native point placement',rows[0].get('point')==[17,28,39])
        if entry['name']=='current-extrusion':check('Rhino current extrusion remains solid CAD',rows[0]['class']=='Rhino.Geometry.Brep' and rows[0].get('solid') and abs(rows[0]['volume']-160*math.pi)<=max(.001,.0001*160*math.pi))
        if entry['name']=='current-mesh':check('Rhino mesh remains native mesh with 20000 faces',rows[0]['class']=='Rhino.Geometry.Mesh' and rows[0]['mesh_faces']==20000)
        if entry['name']=='current-ring':check('Rhino real ring has new exact circle',len([r for r in rows if r['class']=='Rhino.Geometry.ArcCurve' and abs(r.get('length',0)-4*math.pi)<1e-6 and all(abs(x-y)<.001 for x,y in zip(r['bounds'],[15,2,1,19,6,1]))])==1)
        case['save_command']=rhino_file_command('SaveAs',target)
        check('Rhino SaveAs '+entry['name'],Rhino.RhinoApp.RunScript(case['save_command'],config.get('echo_commands',True)) and os.path.isfile(target))
        model=Rhino.FileIO.File3dm.Read(target)
        check('Rhino saved file reread '+entry['name'],model is not None)
        saved_all=object_rows(model.Objects,model_definitions(model))
        received=root_rows(saved_all)
        case['saved_rows']=received
        case['saved_all_rows']=saved_all
        check('Rhino saved count '+entry['name'],len(received)==len(rows))
        for a,b in saved_pairs(all_rows,saved_all):
            check('Rhino saved geometry class '+entry['name'],a['class']==b['class'])
            check('Rhino saved bounds '+entry['name'],all(abs(x-y)<0.001 for x,y in zip(geometry_bounds(a),geometry_bounds(b))))
            if 'length' in a:check('Rhino saved length '+entry['name'],abs(a['length']-b['length'])<1e-6)
            if 'points' in a:check('Rhino saved cloud fields '+entry['name'],a['points']==b['points'])
            if 'point' in a:check('Rhino saved native point '+entry['name'],a['point']==b['point'])
            for field in ['faces','solid','mesh_vertices','mesh_faces','definition_id','definition_member','instance_definition_objects']:
                if field in a:check('Rhino saved '+field+' '+entry['name'],a[field]==b[field])
            for field in ['area','volume']:
                if field in a:check('Rhino saved '+field+' '+entry['name'],abs(a[field]-b[field])<=max(.001,.0001*abs(a[field])))
            if 'transform' in a:check('Rhino saved instance transform '+entry['name'],all(abs(x-y)<1e-10 for x,y in zip(a['transform'],b['transform'])))
        if config.get('export_selected'):
            doc.Objects.UnselectAll()
            selected_expected=export_rows(entry['name'],rows)
            selected_ids=set(row['object_id'] for row in selected_expected)
            roots=[o for o in doc_objects if str(o.Id) in selected_ids]
            case['selected_rows']=selected_expected
            case['selected_names']=[row['name'] for row in selected_expected]
            check('Rhino Selected names bind host reimport '+entry['name'],all(case['selected_names']) and len(set(case['selected_names']))==len(selected_expected))
            for obj in roots:check('Rhino select root '+entry['name'],obj.Select(True,True,True,True,True,True)>0)
            selected_target=os.path.join(out,entry['name']+'.rhino5-selected.3dm')
            case['export_selected_command']=rhino_file_command('Export',selected_target)
            check('Rhino Export Selected '+entry['name'],Rhino.RhinoApp.RunScript(case['export_selected_command'],True) and os.path.isfile(selected_target))
            selected_model=Rhino.FileIO.File3dm.Read(selected_target)
            check('Rhino Export Selected file reread '+entry['name'],selected_model is not None)
            selected_rows=object_rows(selected_model.Objects,model_definitions(selected_model))
            check('Rhino Export Selected root count '+entry['name'],len(root_rows(selected_rows))==len(selected_expected))
            for a,b in saved_pairs(selected_expected,root_rows(selected_rows)):
                check('Rhino Export Selected bounds '+entry['name'],all(abs(x-y)<.001 for x,y in zip(geometry_bounds(a),geometry_bounds(b))))
                for field in ['area','volume','length']:
                    if field in a:check('Rhino Export Selected '+field+' '+entry['name'],abs(a[field]-b[field])<=max(.001,.0001*abs(a[field])))
            case['rhino_export_selected']=selected_target
            case['rhino_export_selected_sha256']=digest(selected_target)
            selected_model.Dispose()
        check('Rhino source unchanged '+entry['name'],digest(source)==entry['input_sha256'])
        case['output_sha256']=digest(target);case['ok']=True;model.Dispose()
    report['ok']=True
except Exception:report['error']=traceback.format_exc()
finally:
    with open(os.path.join(out,'rhino5-results.json'),'w') as stream:json.dump(report,stream,indent=2)
