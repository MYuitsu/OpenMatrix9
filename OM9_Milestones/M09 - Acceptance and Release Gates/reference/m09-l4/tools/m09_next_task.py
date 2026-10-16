#!/usr/bin/env python3
"""Find one M09 task eligible for review from EXTERNALLY SUPPLIED signed state.

This is a scheduler helper, not a signature validator or gate certifier.
Never infers PASS from planned task metadata; state file must be explicit.
"""
from pathlib import Path
import argparse, json, sys
R=Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--gate',choices=['G-MVP','G-FULL'],default='G-MVP')
p.add_argument('--state',required=True,help='Runtime evidence state JSON created by a human/CI evidence collector (not a template)')
a=p.parse_args()
sfile=Path(a.state)
if not sfile.is_file():p.error('state file not found')
s=json.loads(sfile.read_text(encoding='utf8'))
if s.get('template_only') is True:
 print('BLOCKED_STATE: Template is not signed evidence. Supply real approved gate/task records.');sys.exit(2)
root=s.get('gate_evidence',{})
required=['G00']+[f'E{i:02d}' for i in range(1,13)] if a.gate=='G-MVP' else ['G-MVP','E13']
blocked=[]
for g in required:
 x=root.get(g,{})
 # Structural sanity only; reviewer signature authenticity must be checked independently.
 if x.get('status')!='PASS' or not all(x.get(k) for k in ('reviewer','signature_reference','source_sha')):
  blocked.append(g)
if blocked:
 print('BLOCKED_GATE:',', '.join(blocked),'are not recorded as signed PASS in supplied state. NO task is release-ready.')
 sys.exit(2)
manifest=json.loads((R/'planning/M09/M09_EXECUTION_MANIFEST.json').read_text('utf8'))
res=s.get('task_evidence',{})
records=[x for x in manifest['items'] if x['parent_gate_id']==a.gate]
for t in records:
 i=t['task_id'];already=res.get(i,{})
 if already.get('status')=='PASS' and all(already.get(k) for k in ('reviewer','evidence_path','source_sha')):continue
 deps=t['depends_on_task_ids']
 unready=[d for d in deps if res.get(d,{}).get('status')!='PASS' or not all(res.get(d,{}).get(k) for k in ('reviewer','evidence_path','source_sha'))]
 if unready:continue
 if t['task_kind']=='SIGNOFF':
  print('AWAITING_HUMAN_REVIEW (not automatic GO):',i)
 else:
  print('NEXT_ELIGIBLE_FOR_VERIFICATION:',i)
 print('TASK:',t['task_path']);print('REQUIRES_REVIEWED_EVIDENCE:',t['evidence_target'])
 print('NOTE: metadata is not cryptographic signature verification and cannot approve a gate.')
 break
else:
 left=[t for t in records if res.get(t['task_id'],{}).get('status')!='PASS']
 if left:print('BLOCKED_TASK_DEPENDENCY: no uncompleted task has reviewed PASS dependencies; inspect evidence.')
 else:print('ALL_TASKS_RECORDED_PASS: independent human gate signing remains mandatory. Not a GO decision.')
