#!/usr/bin/env python3
"""Check M09 release-planning L4 structure; intentionally NOT an app/release test."""
from pathlib import Path
from collections import Counter
import json,csv,sys
R=Path(__file__).resolve().parents[1];B=R/'planning'/'M09'
def read(p):return json.loads(p.read_text(encoding='utf8'))
M=read(R/'planning/EXECUTION_MANIFEST.json')
canon={v['l3_id']:v for v in M['items']}
assert len(canon)==772, 'canonical L3 count changed'
assert sum(v['execution_stage'] in ('E'+str(i).zfill(2) for i in range(1,13)) for v in canon.values())==529
assert sum(v['execution_stage']=='E13' for v in canon.values())==73
reg=read(R/'specs/_meta/l3-registry.json')
assert (reg['l2_count'],reg['l3_count'],reg['l4_created_count'])==(95,772,0)
plan=read(B/'M09_EXECUTION_MANIFEST.json');items=plan['items']
assert plan['schema_version']=='om9.m09.release_plan.v1'
assert plan['canonical_l2_added']==plan['canonical_l3_added']==0
assert len(items)==plan['release_l4_count']==30
assert Counter(v['parent_gate_id'] for v in items)=={'G-MVP':18,'G-FULL':12}
assert plan['app_source_in_zip'] is False and plan['source_revision'] is None
assert plan['gate_dependency_graph']['G-FULL']==['G-MVP','E13']
assert plan['gate_dependency_graph']['G-MVP']==['G00']+['E'+str(i).zfill(2) for i in range(1,13)]
ids={x['task_id'] for x in items};assert len(ids)==30
assert len({x['path'] for x in items})==30 and len({x['task_path'] for x in items})==30
cases=[]; graph={};total_refs=set();case_ids=set()
for item in items:
 t=read(R/item['path']);text=(R/item['task_path']).read_text('utf8')
 for k,wanted in [('task_id',item['task_id']),('title',item['title']),('parent_gate_id',item['parent_gate_id']),('task_kind',item['task_kind']),('status','PLANNED_NOT_EXECUTED'),('owner_milestone','M09'),('owner_type','RELEASE_GATE_ONLY'),('evidence_target',item['evidence_target']),('depends_on_task_ids',item['depends_on_task_ids'])]:
  assert t[k]==wanted,(item['task_id'],k)
 assert t['gate_status']=='NOT_TESTED' and t['release_decision']=='UNDECIDED'
 assert t['source_revision'] is None and t['source_paths_verified_locally']==[] and t['toolchain_verified'] is False
 assert t['estimated_effort'] is None and t['evidence_state']=='NOT_CREATED'
 assert 'parent_l3_id' not in t and 'owner_module_id' not in t and 'execution_stage' not in t
 assert t['external_gate_preconditions']==(['G00_PASS','E01..E12_PASS'] if t['parent_gate_id']=='G-MVP' else ['G_MVP_SIGNED_PASS','E13_PASS'])
 assert item['status']=='PLANNED_NOT_EXECUTED' and item['source_revision'] is None
 assert [r['l3_id'] for r in t['canonical_traceability']]==item['l3_refs']
 for r in t['canonical_traceability']:
  c=canon[r['l3_id']]; total_refs.add(r['l3_id'])
  assert (r['module_id'],r['capability_id'],r['execution_stage'])==(c['module_id'],c['capability_id'],c['execution_stage'])
  assert r['l3_path']==c['l3_path'].rstrip('/')+'/l3.json' and (R/r['l3_path']).is_file()
  assert c['execution_stage'] in ['E'+str(i).zfill(2) for i in range(1,14)], 'AFTER cannot gate release'
 for m in t['origin_manual_navigation']:
  assert m['filename'] in {'matrix8_book_1.pdf','matrix_8_manual_book2.pdf','rhino5_manual.pdf','matrix9.0_addendum_manual.pdf'}
  assert all(1<=p<={'matrix8_book_1.pdf':246,'matrix_8_manual_book2.pdf':342,'rhino5_manual.pdf':284,'matrix9.0_addendum_manual.pdf':94}[m['filename']] for p in m['pdf_pages_1based'])
 assert len(t['execution_steps'])>=3 and len(t['negative_cases_and_blockers'])>=2
 assert len(t['test_cases'])==len(t['execution_steps'])+len(t['negative_cases_and_blockers'])
 assert t['specific_pass_oracle'] and all(c['status']=='NOT_EXECUTED' for c in t['test_cases'])
 assert all(c['case_id'] not in case_ids and not case_ids.add(c['case_id']) for c in t['test_cases'])
 assert t['evidence_target'].startswith('implementation/evidence/M09/'+t['parent_gate_id']+'/')
 assert set(t['depends_on_task_ids'])<=ids and item['task_id'] not in t['depends_on_task_ids']
 assert all(item['parent_gate_id']==next(u['parent_gate_id'] for u in items if u['task_id']==d) for d in t['depends_on_task_ids']), 'cross-gate internal dependency invalid; use signed external prerequisite'
 assert all(x in text for x in [item['task_id'],'## Thứ tự làm','## Acceptance','## Bằng chứng','## Ca kiểm thử có mã định danh','**Oracle cụ thể:**'])
 assert not any(x in text for x in ['**Result:** PASS','**Trạng thái:** `DONE`'])
 graph[item['task_id']]=t['depends_on_task_ids']
 cases.extend((item,c) for c in t['test_cases'])
assert len(total_refs)>=50
visited=set();busy=set();top=[]
def walk(i):
 if i in visited:return
 assert i not in busy, 'dependency cycle: '+i
 busy.add(i)
 for d in graph[i]:walk(d)
 busy.remove(i);visited.add(i);top.append(i)
for i in graph:walk(i)
assert len(top)==30
assert graph['OM9-M09-L4-GMVP-018']==['OM9-M09-L4-GMVP-001','OM9-M09-L4-GMVP-017']
assert 'OM9-M09-L4-GFULL-010' in graph['OM9-M09-L4-GFULL-012']
with (B/'M09_EXECUTION_MANIFEST.csv').open(encoding='utf-8-sig',newline='') as f:rows=list(csv.DictReader(f))
assert len(rows)==len(items)
ix={x['task_id']:x for x in items}
for r in rows:
 i=ix[r['task_id']]
 for field in ['task_id','parent_gate_id','task_kind','title','status','path','task_path','evidence_target']:
  assert r[field]==i[field],(i['task_id'],field)
 assert r['l3_refs'].split(';')==i['l3_refs']
 assert [x for x in r['depends_on_task_ids'].split(';') if x]==i['depends_on_task_ids']
with (B/'M09_TEST_CASES.csv').open(encoding='utf-8-sig',newline='') as f:rows=list(csv.DictReader(f))
assert len(rows)==len(cases)==177
expected={c['case_id']:(t,c) for t,c in cases};assert len(expected)==len(rows)
for row in rows:
 task,case=expected[row['case_id']]
 assert row['task_id']==task['task_id'] and row['gate_id']==task['parent_gate_id']
 assert row['case_kind']==case['case_kind'] and row['action']==case['action']
 assert row['status']=='NOT_EXECUTED' and row['evidence_target']==task['evidence_target']
 assert row['expected_oracle']==case['expected']
for p in ['M09_EVIDENCE_TEMPLATE.json','M09_GATE_SIGNOFF_TEMPLATE.json']:
 d=read(B/p);assert d['template_only'] is True
assert read(B/'M09_GATE_SIGNOFF_TEMPLATE.json')['decision']=='UNDECIDED'
assert not (R/'implementation'/'evidence'/'M09').exists(), 'plan must not fake evidence'
for g in ['G_MVP_CHECKLIST.md','G_FULL_CHECKLIST.md']:
 t=(B/g).read_text('utf8');assert 'NOT_TESTED' in t
assert 'M09/README.md' in (R/'planning'/'M09_RELEASE_GATES.md').read_text('utf8')
print(f'PASS M09 planning validation: 30 L4 / 177 cases / {len(total_refs)} unique canonical L3 refs / 2 release gates / acyclic DAG')
print('NO CLAIMS: G00/E12/E13 signed, actual Rust/OCCT build, geometry test PASS, human signoff, release ready')
