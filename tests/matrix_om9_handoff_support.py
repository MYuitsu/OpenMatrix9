"""Python2/3 application-test transport/oracles; no product layer policy."""
import json,os,uuid,math
MAX_MESSAGE=2*1024*1024
ACTIONS=('snapshot','paste','undo','redo','fixture','copy','finish')

def write_json(path,value):
    data=json.dumps(value,ensure_ascii=True,sort_keys=True,indent=2).encode('ascii')
    if len(data)>MAX_MESSAGE:raise ValueError('Test message exceeds bounded size')
    temporary=path+'.'+uuid.uuid4().hex+'.tmp'
    try:
        with open(temporary,'wb') as stream:stream.write(data)
        # Every protocol path is immutable and unique to one sequence.
        os.rename(temporary,path)
    finally:
        if os.path.isfile(temporary):os.remove(temporary)

def read_json(path):
    with open(path,'rb') as stream:data=stream.read(MAX_MESSAGE+1)
    if len(data)>MAX_MESSAGE:raise ValueError('Test message exceeds bounded size')
    return json.loads(data.decode('utf-8-sig'))

def validate_request(value,run_id,seq):
    if set(value)!=set(('version','run_id','seq','action')) or value['version']!=1 or value['run_id']!=run_id or value['seq']!=seq or value['action'] not in ACTIONS:
        raise ValueError('Foreign, out-of-order or unknown test request')
    return value['action']

def transfer_errors(source,received,tolerance=0.001):
    errors=[];layers=dict((tuple(row['path']),row) for row in received['layers'])
    for row in source['layers']:
        name='::'.join(row['path']);other=layers.get(tuple(row['path']))
        if other is None:errors.append('missing layer '+name);continue
        for key in ('rgb','locked','visible','persistent_locked','persistent_visible'):
            if row[key]!=other[key]:errors.append('layer '+name+' '+key)
    if source['active']!=received['active']:errors.append('active layer')
    objects={}
    for row in received['objects']:objects.setdefault(row['name'],[]).append(row)
    for row in source['objects']:
        matches=objects.get(row['name'],[])
        if len(matches)!=1:errors.append('object '+row['name']+' missing or ambiguous');continue
        other=matches[0]
        keys=['layer','locked','visible','color_source']
        if row['color_source']=='ByObject':keys.append('rgb')
        for key in keys:
            if row[key]!=other[key]:errors.append('object '+row['name']+' '+key)
        a=row['bounds'];b=other['bounds']
        if len(a)!=6 or len(b)!=6 or any(math.isnan(x) or math.isinf(x) or math.isnan(y) or math.isinf(y) or abs(x-y)>tolerance for x,y in zip(a,b)):
            errors.append('object '+row['name']+' bounds')
    return errors

def accepted(report):
    if not report.get('cleanup_ok') or not report.get('checks') or not all(row.get('passed') is True for row in report['checks']):return False
    for name in ('matrix_to_om9','om9_to_matrix'):
        direction=report.get('directions',{}).get(name,{})
        required=('copy','paste','transfer')
        if name=='matrix_to_om9' or report.get('undo_scope')!='om9_only':required+=('undo','redo')
        if not all(direction.get(key) is True for key in required):return False
    return True
