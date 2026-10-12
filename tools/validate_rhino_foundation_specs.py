"""Check normalized foundation coverage without claiming runtime compatibility."""
from pathlib import Path
import hashlib
import json
import re
import subprocess
import yaml

root = Path(__file__).resolve().parents[1]
pack = root / 'ref/matrix9/OpenMatrix9_Codex_Spec_v1'
features = json.loads((pack/'FEATURES.json').read_text(encoding='utf-8'))
assert features == yaml.safe_load((pack/'FEATURES.yaml').read_text(encoding='utf-8'))
foundation = json.loads((root/'docs/rhino-foundation-requirements.json').read_text(encoding='utf-8'))
contracts = {c['id']: c for c in foundation['contracts']}
maps = {m['feature_id']: m for m in foundation['features']}
assert len(features) == len(maps) == 607
assert len(contracts) == 19
assert len({f['id'] for f in features}) == 607
assert len({f['domain'] for f in features}) == 16
guide = (pack/'RHINO5_FOUNDATION.md').read_text(encoding='utf-8')
preserved = 0
for f in features:
    body = (pack/f['spec_path']).read_text(encoding='utf-8')
    assert body.count('## Rhino 5 foundation compatibility') == 1, f['id']
    assert f['rhino_foundation'] == maps[f['id']]
    for key in maps[f['id']]['contracts']:
        assert key in contracts and f'id="{key.lower()}"' in guide
        assert key in body
        assert contracts[key]['source_behavior'] in body, f['id']
        assert contracts[key]['acceptance'] in body, f['id']
        assert f"OpenMatrix9 acceptance for {f['id']}" in body
    for target in re.findall(r'\]\(([^)]+)\)', body.split('## Rhino 5 foundation compatibility')[1]):
        assert ((pack/f['spec_path']).parent/target.split('#')[0]).resolve().exists()
    assert maps[f['id']]['acceptance_status'] == 'unverified'
    old = subprocess.run(['git','show','HEAD:ref/matrix9/OpenMatrix9_Codex_Spec_v1/'+f['spec_path']],cwd=root,capture_output=True)
    if old.returncode == 0:
        # Compare the original portion even after normalized specs are committed.
        previous = old.stdout.decode('utf-8').split('\n## Rhino 5 foundation compatibility')[0].strip()
        assert previous == body.split('\n## Rhino 5 foundation compatibility')[0].strip(), f['id']
        preserved += 1
core=json.loads((root/'docs/core-requirements.json').read_text(encoding='utf-8'))
assert len(core['items']) == 130
for item in core['items']:
    assert item['rhino_foundation'] == maps[item['id']]
for entry in json.loads((pack/'MANIFEST.json').read_text(encoding='utf-8')):
    data=(pack/entry['path']).read_bytes()
    assert len(data)==entry['bytes']
    assert hashlib.sha256(data).hexdigest()==entry['sha256']
source=foundation['source']
source_path = Path(source['path'])
if source_path.is_file():
    assert hashlib.sha256(source_path.read_bytes()).hexdigest()==source['sha256']
    source_check = 'source SHA-256 valid'
else:
    source_check = 'source SHA-256 not checked (external PDF unavailable on this machine)'
assert source['pdf_page_count']==284
for contract in contracts.values():
    assert contract['pdf_pages'] == [n+8 for n in contract['printed_pages']]
print(f'PASS: 607 specs / 16 domains / 19 contracts / 130 core records; JSON=YAML; links, anchors and manifest valid; {source_check}; {preserved} HEAD source specs preserved.')
