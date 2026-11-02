import json
from datetime import datetime, timezone
from pathlib import Path

path = Path("docs/openmatrix9-progress.json")
data = json.loads(path.read_text(encoding="utf-8"))
key = "m04-history-associative-ui-2026-10-27"
ids = [f"OM9-L3-03-13-{n:03d}" for n in range(1, 7)]
out_dir = "implementation/evidence/M04"
item = {
    "key": key,
    "date": "2026-10-27",
    "stage": "feature-port",
    "status": "l4_automated_evidence_ready_manual_gate_open",
    "target": "Standalone Rust OM9; Windows x64; CARGO_TARGET_DIR=L:/om9-target; -j 2",
    "feature_ids": ids,
    "scope": (
        "L2-03-13 (04-febe-matrix-history-associative-updates): record/update toggles, "
        "broken-link report, clear single-object history, edit existing builder by history, "
        "history command line + rebuild status. Re-uses om9-core HistoryGraph/builder/producers; "
        "6 L3 co du L4 evidence, khong co chu ky, gate M04-UI van mo."
    ),
    "outputs": [f"{out_dir}/{i}/" for i in ids]
    + [
        f"{out_dir}/OM9-L2-03-13/README.md",
        f"{out_dir}/OM9-L2-03-13-COMPLETION/README.md",
        "tools/record_m04_history_l3_l4_evidence.py",
    ],
    "evidence": [
        f"{out_dir}/{i}/{name}"
        for i in ids
        for name in (
            "source_crosswalk.json",
            "api.md",
            "positive.json",
            "negative.json",
            "OM9-L4-03-13-"
            + i.rsplit("-", 1)[1]
            + "-03.json",
            "OM9-L4-03-13-" + i.rsplit("-", 1)[1] + "-04.json",
            "OM9-L4-03-13-" + i.rsplit("-", 1)[1] + "-05.json",
            "OM9-L4-03-13-" + i.rsplit("-", 1)[1] + "-06.json",
        )
    ],
    "tests": {
        "OM9-L3-03-13-001": "record off commits geometry with no recipe + Undo; record on stores recipe; toggle during builder refused",
        "OM9-L3-03-13-002": "update off leaves geometry and marks Dirty; re-enable rebuilds ring + profile once; broken dependency refuses re-enable",
        "OM9-L3-03-13-003": "broken rail input reported with role; phantom reference survives save/reopen",
        "OM9-L3-03-13-004": "clear detaches profile only, keeps geometry and ring recipe; locked recipe refused",
        "OM9-L3-03-13-005": "builder reopens at recorded 22 mm; Apply updates same object/recipe in one transaction",
        "OM9-L3-03-13-006": "command line switches/report/clear; missing selection typed error; Pending recipe blocks window close",
    },
    "uncertainties": [
        "Chua co ui-review.json co chu ky; dashboard giu 6 L3 o vang (dung chu y).",
        "Rebuild dong bo, chua co frame 'updating' tren UI; status chi complete/failed kem node id.",
        "History window/icon chua co frame hit-test tu dong; test di qua dung Controller method ma UI goi.",
    ],
    "manual_gate": "OPEN: M04-UI can nguoi review History window, icon sidebar va command line",
    "next_action": (
        "Doc muc 'Ghi chu cuoi milestone (chot 2026-10-27)' trong "
        "implementation/evidence/M04/OM9-L2-03-13-COMPLETION/README.md; chay app, thao tac 6 L3, "
        "chup History window/icon va ky ui-review.json neu dat."
    ),
    "notes_file": (
        "implementation/evidence/M04/OM9-L2-03-13-COMPLETION/README.md"
        "#ghi-chú-cuối-milestone-chốt-2026-10-27"
    ),
}
items = [entry for entry in data["items"] if entry.get("key") != key]
items.append(item)
data["items"] = items
data["active_item"] = key
data["updated_at"] = datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")
path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print("items", len(items), "active", data["active_item"])
