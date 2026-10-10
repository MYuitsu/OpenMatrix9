"""Compose scoped evidence; retain the original failed default-volume report.

File hashes bind measurements to inputs. Decoded UUID, topology and numerical
measurements establish the tested properties; hashes alone do not do so.
"""
import hashlib, json, math
from pathlib import Path

def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def require(value,reason):
    if not value:raise ValueError(reason)
def indexed(rows):
    result={row['fixture']:row for row in rows}
    require(len(result)==len(rows),'duplicate fixture')
    return result

def compose(root):
    def read(name):return json.loads((root/name).read_text())
    oracle=read('oracle.json');gui=read('rhino5-gui-profile-results.json')
    precise=read('rhino5-curved-precision-results.json')
    host=read('freecad-reimport-results.json')
    gui_hash=sha(root/'rhino5-gui-profile-results.json')
    require(precise['gui_report_sha256']==host['gui_report_sha256']==gui_hash,'GUI report binding')
    require(precise['oracle_sha256']==sha(root/'oracle.json'),'oracle binding')
    require(precise['ok'] and precise['active_document_unchanged'],'actual precise run')
    require(host['ok'] and all(row['passed'] for row in host['checks']),'actual FreeCAD reimport')
    require(precise['volume_threshold_mm3']==1e-6,'analytical threshold changed')
    sources=indexed(oracle['cases']);raw=indexed(gui['cases']);measurements=indexed(precise['cases'])
    require(set(sources)==set(raw)==set(measurements) and len(sources)==4,'fixture inventory')
    cases=[]
    for name,wanted in sources.items():
        g=raw[name];p=measurements[name];obj=wanted['objects'][0]
        require(len(wanted['objects'])==1 and g['bad_objects']==0,'GUI object inventory')
        source_hash=sha(root/name);saved_hash=sha(root/Path(g['saved_fixture'].replace('\\','/')).name)
        require(source_hash==wanted['source_sha256']==g['source_sha256']==p['original']['source_sha256'],'source binding')
        require(saved_hash==g['saved_sha256']==p['saved']['source_sha256'],'saved binding')
        volumes=[]
        for key,gui_key in [('original','first_read'),('saved','reread')]:
            row=p[key];objects=g[gui_key]['objects']
            require(len(objects)==1,'GUI root inventory')
            old=objects[0]
            for decoded in [row,old]:
                require(decoded['uuid']==obj['uuid'] and decoded['faces']==obj['faces'],'decoded topology/UUID')
                require(decoded['valid'] and decoded['solid'] and decoded['orientation']=='Outward','decoded solid')
            require(row['input_hash_unchanged'],'input immutability')
            require(row['analytical_volume_mm3']==obj['volume_mm3'],'analytical oracle')
            require(row['default_volume_mm3']==old['volume_mm3'],'raw measurement binding')
            quadrature=row['quadrature']
            require(len(quadrature)==4,'quadrature inventory')
            for q,tolerance in zip(quadrature,[1e-8,1e-10,1e-12,1e-13]):
                require(q['relative_tolerance']==q['absolute_tolerance']==tolerance,'quadrature precision')
                require(math.isfinite(q['volume_mm3']),'finite volume')
                require(abs(q['volume_mm3']-obj['volume_mm3'])<=1e-6,'analytical volume')
            require(abs(quadrature[-1]['volume_mm3']-quadrature[-2]['volume_mm3'])<=1e-6,'convergence')
            volumes.append(quadrature[-1]['volume_mm3'])
        require(abs(volumes[0]-volumes[1])<=1e-6,'SaveAs volume')
        cases.append(dict(fixture=name,source_sha256=source_hash,saved_sha256=saved_hash,
                          uuid=obj['uuid'],faces=obj['faces'],precise_volume_mm3=volumes,
                          saveas_volume_delta_mm3=abs(volumes[0]-volumes[1])))
    return dict(scoped_exchange_verified=True,full_exchange_complete=False,
                scope='Four analytical outward two-shell BReps; original GUI lifecycle plus separately measured precise source/saved copies',
                raw_gui_ok=gui['ok'],raw_default_volume_passed=sum(row['passed'] for row in gui['cases']),
                corrected_gui_script_executed=False,precise_measurements=8,volume_threshold_mm3=1e-6,
                reports={name:sha(root/name) for name in ['oracle.json','rhino5-gui-profile-results.json','rhino5-curved-precision-results.json','freecad-reimport-results.json']},cases=cases)

if __name__=='__main__':
    root=Path(__file__).resolve().parents[1]/'docs/validation/rhino5-curved-shells-20261007'
    result=compose(root)
    (root/'combined-acceptance.json').write_text(json.dumps(result,indent=2)+'\n')
    print('Scoped curved exchange verified: four GUI lifecycles, eight precise measurements; full exchange remains incomplete')
