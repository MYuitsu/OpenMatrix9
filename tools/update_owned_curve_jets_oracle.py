# SPDX-License-Identifier: LGPL-2.1-or-later
"""Promote a corrected native oracle only after independent derivative checks.

Preserves the unpatched OCCT oracle, checks geometry/domain/D0/D1/D2 and
validates all 45 offset D3 queries against the existing 70-digit definition.
This never derives expected values from the Rust evaluator.
"""
import argparse,hashlib,json,math
from pathlib import Path

def fingerprint(path):
    raw=path.read_bytes();return dict(path=path.as_posix(),bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest())
def near(a,b):
    return all(math.isfinite(x)and math.isfinite(y)and abs(x-y)<=2e-9*max(1.,abs(y))for x,y in zip(a,b))

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--native-output',type=Path,required=True)
    args=parser.parse_args()
    fixtures=Path(__file__).resolve().parent.parent/'rust/tests/fixtures/owned_curve_evaluation'
    target=fixtures/'native_jets_oracle.json';backup=fixtures/'native_jets_oracle_occt_8_0_1_unpatched.json'
    assert not backup.exists(),'Preserve original oracle; do not promote twice'
    old=json.loads(target.read_text());report=json.loads((args.native_output/'results.json').read_text())
    assert report['ok']and len(report['cases'])==19
    new=report['cases'];changes=[]
    assert len(old)==19 and sum(len(c['queries'])for c in old)==183
    for a,b in zip(old,new):
        assert a['name']==b['name']and a['geometry_data']==b['geometry_data']and a['domain']==b['domain'],a['name']
        assert len(a['queries'])==len(b['queries'])
        for before,after in zip(a['queries'],b['queries']):
            assert before['parameter']==after['parameter']
            for key in ('value','derivative','second'):assert near(before[key],after[key]),(a['name'],key)
            if not near(before['third'],after['third']):
                assert a['name']in ('OffsetBezier','OffsetNurbs','OffsetTrim','OffsetNestedDirection'),a['name']
                changes.append(dict(name=a['name'],parameter=before['parameter'],before=before['third'],after=after['third']))
    assert len(changes)==36,len(changes)
    independent=json.loads((fixtures/'independent_offset_third.json').read_text());count=0
    by_name={c['name']:c for c in new}
    for case in independent:
        actual={q['parameter']:q for q in by_name[case['name']]['queries']}
        for query in case['queries']:
            assert near(query['third'],actual[query['parameter']]['third']),(case['name'],query['parameter'])
            count+=1
    assert count==45
    loaded=json.loads((args.native_output/'loaded-native-kernel.json').read_text());actual=fingerprint(Path(loaded['path']))
    assert actual['sha256']==loaded['sha256']
    backup.write_bytes(target.read_bytes())
    target.write_text(json.dumps(new,indent=2)+'\n',encoding='utf-8')
    provenance=dict(schema_version=1,method='Fresh FreeCAD/OCCT native edge values; corrected shared OCCT helper; independent 70-digit offset D3 validation; no Rust oracle generation',baseline=fingerprint(backup),current=fingerprint(target),native_report=fingerprint(args.native_output/'results.json'),loaded_kernel=actual,loaded_app=json.loads((args.native_output/'loaded-native-app.json').read_text()),selected_plugin=report['module'],cases=19,queries=183,changed_third_queries=36,independent_offset_queries=45,changes=changes)
    path=fixtures/'native_jets_corrected_provenance.json';assert not path.exists()
    path.write_text(json.dumps(provenance,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(dict(cases=19,queries=183,changed_third=36,independent=45)))

if __name__=='__main__':main()
