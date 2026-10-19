"""Sync measured primitive scope while retaining full Layers/RCORE gates."""
from pathlib import Path
import datetime as dt
import hashlib
import json
import yaml

ROOT=Path(__file__).resolve().parents[1]


def write(path,data):
    path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+"\n",encoding="utf-8")


def main():
    receipt=json.loads((ROOT/"docs/validation/2026-10-14-owned-layer-primitives.json").read_text(encoding="utf-8"))
    assert receipt["workspace_passed"]==375 and receipt["focused_tests"]==15 and not receipt["goal_complete"]
    package=ROOT/"OpenMatrix9_Codex_Spec_v2"
    audit=json.loads((package/"SOURCE_READ_AUDIT.json").read_text(encoding="utf-8"))
    for row in audit["files"]:
        path=ROOT/"OpenMatrix9_Codex_Spec_v1"/row["path"]
        assert path.stat().st_size==row["bytes"] and hashlib.sha256(path.read_bytes()).hexdigest()==row["sha256"],row["path"]
    report="""# Owned layer transaction primitives — 2026-10-14

This implements Task1 of [the complete Layers plan](../superpowers/plans/2026-10-14-owned-layers.md),
within RCORE-00 and OM9-LAYER-001. Whole Layers, M9IMG-04 and M01–M12 remain open.

`LayerPlan` captures coordinator/caller context and document owner, epoch,
session generation, revision and unit-context revision. Create/current, Update,
SetCurrent and plain-root Assign use the owned coordinator. New layer UUIDs
participate in the retired identity ledger; source ownership and all batch
permissions are validated before private document clones. Metadata read/write
overlap rejects Lock+Assign and changed-layer output batches instead of using
the old permission. Caller savepoint rollback retains retired UUID authority
and leaves the prior global Redo executable.

Current does not assign existing objects. Assign retains current, geometry and
placement, records by-layer color and rejects locked/foreign/structural/retained
sources. Lock/visibility leave explicit object visible/selectable preferences
untouched. A current no-op at a full History budget creates no publication or
Undo entry and preserves Redo. Create/update/Assign use checked revisions and
admitted storage; Cancel before preparation and after private preparation keeps
the immediate world/stack unchanged.

The scene resolves assigned color from the layer and updates after a later
swatch change. An independent actual OM9 child reopens a saved .om9 and verifies
UUID/order/current/name/color/lock/visibility, object preferences and geometry.
It runs under the existing NativeProgram job with a 10-second deadline and
64-KiB stdout cap; exited=true/active_processes=0. Its measured private commit
peak is 1,925,120 bytes for this single authored metadata fixture.

Measured receipt: [source-bound batch](2026-10-14-owned-layer-primitives.json).
375 workspace tests pass; 23 SDK tests remain ignored here. The 15 focused
checks (12 transactions, one scene, one retained refusal, one cold process) are
workspace subsets, not 15 additional distinct tests. Format and all-target
warning-strict Clippy pass. Independent review reproduced/fixed the full-History
no-op issue and verified the corrected primitive. Assign uses one admitted
source set and one retained-row scan, polls Cancel, and uses a bounded borrowed
layer permission walk; no quantitative large-scene latency claim is made.

Pending: durable preset-slot/independent Hidden schema and explicit migration;
complete current-output binding during command sessions; actual layer panel/CMD,
multi-selection, right-click actions, group toggles, Advanced dialogs and Lights;
complete UI/native metadata/renderer/performance acceptance. Older SDK/native UI
receipts retain their earlier source checkpoint and are not refreshed by this
primitive batch. No feature/capability or full acceptance gate is promoted.
"""
    (ROOT/"docs/validation/2026-10-14-owned-layer-primitives.md").write_text(report,encoding="utf-8")
    meta={"scope":"owned_layer_transaction_primitives_only","receipt":"../docs/validation/2026-10-14-owned-layer-primitives.json","report":"../docs/validation/2026-10-14-owned-layer-primitives.md","workspace_tests":375,"focused_subset":15,"full_feature_acceptance":False}
    ledger_path=package/"MIGRATION_LEDGER.json"
    ledger=json.loads(ledger_path.read_text(encoding="utf-8"));ledger["foundation_implementation"]["owned_layers_primitives"]=meta;write(ledger_path,ledger)
    for filename in ["FEATURES.json","FEATURE_LOOKUP.json"]:
        path=package/filename; data=json.loads(path.read_text(encoding="utf-8"))
        feature=next(f for f in data if f["id"]=="OM9-LAYER-001") if isinstance(data,list) else data["OM9-LAYER-001"]
        assert feature["implementation_status"]=="not_implemented_on_standalone_om9"
        feature["standalone_primitive_evidence"]=meta;write(path,data)
        if filename=="FEATURES.json":(package/"FEATURES.yaml").write_text(yaml.safe_dump(data,allow_unicode=True,sort_keys=False,width=120),encoding="utf-8")
    feature_path=package/"specs/01-core/om9-layer-001-layer-management.md"
    text=feature_path.read_text(encoding="utf-8"); marker="## Standalone primitive evidence — 2026-10-14"
    section=marker+"\n\nOwned Rust Create/Update/current/Assign primitives, stale/session/retired UUID/Cancel/budget/Undo/Redo guards and independent cold metadata have a [measured receipt](../../../docs/validation/2026-10-14-owned-layer-primitives.json). 15 focused checks are included in 375 workspace tests. Preset/Hidden schema, actual panel/CMD/modal/current-output and complete Layer acceptance remain open; feature status and every full checklist stay unchanged.\n\n"
    if marker not in text:text=text.replace("## Hợp đồng thiết kế",section+"## Hợp đồng thiết kế",1)
    feature_path.write_text(text,encoding="utf-8")
    status=package/"IMPLEMENTATION_STATUS.md";text=status.read_text(encoding="utf-8")
    note="\n\nLayers primitive checkpoint mới: 375 workspace tests; 15 kiểm tra tập trung là subset, một tiến trình OM9 mở lạnh giữ metadata. [Bằng chứng](../docs/validation/2026-10-14-owned-layer-primitives.md). Preset/Hidden schema và panel/CMD/đầy đủ OM9-LAYER-001 còn mở; các receipt native/UI trước giữ checkpoint nguồn cũ."
    if note not in text:text=text.rstrip()+note+"\n"
    status.write_text(text,encoding="utf-8")
    path=ROOT/"README.md";text=path.read_text(encoding="utf-8")
    note="Owned Layers primitive checkpoint: 375 workspace tests pass; 15 focused checks are subsets, including an independent owned-process cold metadata reopen. [Measured scope and pending panel/schema](docs/validation/2026-10-14-owned-layer-primitives.md). Native/UI/exchange receipts above retain their earlier checkpoint."
    if note not in text:text=text.replace("**Open-source jewelry CAD for FreeCAD.**",note+"\n\n**Open-source jewelry CAD for FreeCAD.**",1)
    path.write_text(text,encoding="utf-8")
    path=ROOT/"docs/openmatrix9-progress.json";data=json.loads(path.read_text(encoding="utf-8"));item=next(i for i in data["items"] if i.get("key")=="rcore00-standalone-foundation")
    data["updated_at"]=dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds")
    for value in ["docs/validation/2026-10-14-owned-layer-primitives.json","docs/validation/2026-10-14-owned-layer-primitives.md","crates/om9-core/src/layers.rs"]:
        if value not in item["outputs"]:item["outputs"].append(value)
    prefix="Owned Layers primitive;"
    item["evidence"]=[e for e in item["evidence"] if not e.startswith(prefix)]
    item["evidence"].append(prefix+" 375 workspace tests, 15 focused subsets, source-frozen receipt and one bounded actual cold OM9 child. Core Create/current/Update/plain Assign have session/retired UUID/caller/batch/permission/Cancel/budget/Undo/Redo guards; by-layer scene color and explicit preferences persist. Whole Layers/preset/Hidden/UI/current-output remain open; earlier SDK/native UI checkpoints are historical for the changed source.")
    item["next_document"]="OpenMatrix9_Codex_Spec_v2/specs/01-core/om9-layer-001-layer-management.md"
    item["next_action"]="Implement durable preset-slot and independent Hidden schema with explicit migration, then connect complete Layers panel/CMD/current-output/modal/selection behavior; continue full RCORE-00 acceptance."
    write(path,data)
    path=ROOT/"docs/validation/opennurbs-test-roadmap.json";data=json.loads(path.read_text(encoding="utf-8"))
    data["owned_layers_primitives_checkpoint"]={"status":"passed_scoped","workspace_tests":375,"focused_subset":15,"cold_om9_processes":1,"evidence":"docs/validation/2026-10-14-owned-layer-primitives.json","full_feature_acceptance":False}
    write(path,data)
    path=ROOT/"docs/validation/opennurbs-test-roadmap.md";text=path.read_text(encoding="utf-8")
    note="Current owned-layer primitive measurement: **375 workspace tests**, with **15 focused tests included** and **one bounded cold OM9 child**. Twenty-three SDK tests are ignored by this batch; no new SDK/Rhino acceptance or combined current-source native total is claimed. [Evidence](2026-10-14-owned-layer-primitives.md). Preset/Hidden/schema/UI work follows; packages2–8 and full RCORE gates remain open, and total future batches remain unknown. Earlier checkpoints below retain historical scope.\n\n"
    if note not in text:text=note+text.replace("Latest standalone checkpoint:","Historical standalone checkpoint:",1)
    path.write_text(text,encoding="utf-8")
    manifest=[]
    for path in sorted(package.rglob("*")):
        if path.is_file() and path.name!="MANIFEST.json" and not {"target","__pycache__"}.intersection(path.parts):
            contents=path.read_bytes();manifest.append({"path":path.relative_to(package).as_posix(),"bytes":len(contents),"sha256":hashlib.sha256(contents).hexdigest()})
    write(package/"MANIFEST.json",manifest)
    print(json.dumps({"workspace_tests":375,"focused_subset":15,"v1_unchanged":len(audit["files"]),"v2_manifest":len(manifest),"full_feature_acceptance":False}))


if __name__=="__main__":main()
