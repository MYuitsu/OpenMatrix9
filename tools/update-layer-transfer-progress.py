"""Update candidate progress without changing accepted Phase1-3 evidence."""
from pathlib import Path
from datetime import datetime, timezone
import json

path = Path('H:/FreeCAD-src/build/om9-layer-edits/docs/openmatrix9-progress.json')
data = json.loads(path.read_text(encoding='utf-8-sig'))
key = 'layer-session-handoff-20261010'
data['updated_at'] = datetime.now(timezone.utc).isoformat()
data['active_item'] = key
item = dict(key=key, stage='build-validation', feature_ids=['OM9-LAYER-001'],
    status='in_progress', implementation_verified=False, application_accepted=False,
    outputs=['docs/superpowers/plans/2026-10-10-layer-session-handoff.md',
             'docs/validation/layer-session/execution-ledger.md',
             'docs/validation/layer-session/transfer-slice.json'],
    evidence=['Scoped working-object public Selected/Session/palette transfer: Rust253/50 groups, fmt/strict Clippy, native8, final-plugin shared-host250/11 reports.',
              'Real Command Copy asserts current selected geometry records2; single viewport Ctrl+V and FCStd reopen retain expected6 objects.',
              'Shared FreeCAD host unchanged; candidate plugin outputs/profile isolated.',
              'Actual Matrix frontend/core loaded in old af408 report; full API gate failed obsolete detached persistent assertion. Corrected source/staging SHA match; no newer actual gate PASS.'],
    uncertainty=['Retained archive/block full Session provenance and dependency context remains unsupported.',
                 'Remaining native property/tree/Delete/transform guards and receive failure after layer projection.',
                 'Actual Matrix Rust/C# product bridge, both-direction file/clipboard acceptance, heavy metadata/A-B timings and final whole-source review remain pending.'],
    next_action='Read new Administrator Matrix API diagnostic report; independently finish retained native collector and remaining guards/rollback before full acceptance.')
data['items'] = [entry for entry in data['items'] if entry.get('key') != key] + [item]
path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
print('Layer candidate progress recorded; accepted Phase1-3 entries retained.')
