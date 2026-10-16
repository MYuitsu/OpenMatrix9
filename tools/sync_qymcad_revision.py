"""Sync QymCAD revision stamps in milestone trees, paired file by file.

The merged milestone trees pin one QymCAD revision while the packages they were merged
from pin another. Matrix9's own value is
c116f8f224ebffe0ada049c91c0d15df3634f00d (M00 records, the M05/M06 packages and the
M07 overlay); the M03/M04/M05/M06 trees carry
746a17f127897e921a57118b6caad08968602bf5.

A replacement is only made where the PAIRED package file actually contains the new
value, so a tree is never rewritten on the strength of a guess. Every file touched is
listed.
"""
import json
import sys
from pathlib import Path

ROOT = Path("H:/OpenMatrix9")
OLD = "746a17f127897e921a57118b6caad08968602bf5"
NEW = "c116f8f224ebffe0ada049c91c0d15df3634f00d"
APPLY = "--apply" in sys.argv

# milestone -> (tree dir, package dir containing the same relative layout)
PACKAGES = {
    "M03": ("OM9_Milestones/M03 - Curve, Surface, Solid and 3DM Exchange",
            "build/m03-incoming/OpenMatrix9/M03 - Curve, Surface, Solid and 3DM Exchange"),
    "M04": ("OM9_Milestones/M04 - Builder and History",
            "build/m04-incoming/OpenMatrix9/M04 - Builder and History"),
    "M05": ("OM9_Milestones/M05 - Gems and Layout", "build/m05-incoming"),
    "M06": ("OM9_Milestones/M06 - Settings and Prongs", "build/m06-incoming"),
    "M07": ("OM9_Milestones/M07 - Cutters",
            "build/m07-incoming/OpenMatrix9/planning/M07"),
}

for ms, (tree_rel, pkg_rel) in PACKAGES.items():
    tree = ROOT / tree_rel
    pkg = ROOT / pkg_rel
    if not tree.is_dir():
        print(f"{ms}: tree missing")
        continue
    # The M03/M04 packages mirror the tree layout; the M05/M06 packages are flat
    # overlays, so build a task_id index of the package as a fallback pairing.
    pkg_by_task = {}
    pkg_by_name = {}
    if pkg.is_dir():
        for cand in pkg.rglob("*.json"):
            try:
                d = json.loads(cand.read_text(encoding="utf-8-sig"))
            except (OSError, json.JSONDecodeError):
                continue
            if not isinstance(d, dict):
                continue
            # the flat overlay packages key on the milestone-prefixed task id, while the
            # merged tree records keep it as overlay_task_id and drop the prefix from
            # task_id, so index every identity the record may carry
            for key in ("task_id", "id", "overlay_task_id"):
                val = d.get(key)
                if val:
                    pkg_by_task.setdefault(val, cand)
    replaced, skipped, occurrences = [], [], 0
    for p in sorted(tree.rglob("*.json")):
        text = p.read_text(encoding="utf-8", errors="replace")
        if OLD not in text:
            continue
        pair = None
        if pkg.is_dir():
            rel = p.relative_to(tree)
            for candidate in (pkg / rel, pkg / p.name):
                if candidate.is_file():
                    pair = candidate
                    break
            if pair is None:
                try:
                    d = json.loads(text)
                except json.JSONDecodeError:
                    d = {}
                for key in ("task_id", "id", "overlay_task_id"):
                    val = d.get(key)
                    if val and val in pkg_by_task:
                        pair = pkg_by_task[val]
                        break
        if pair is None:
            skipped.append(f"{p.relative_to(tree)} (no package pair)")
            continue
        pair_text = pair.read_text(encoding="utf-8-sig", errors="replace")
        if NEW not in pair_text:
            skipped.append(f"{p.relative_to(tree)} (package has no new stamp)")
            continue
        updated = text.replace(OLD, NEW)
        try:
            json.loads(updated)
        except json.JSONDecodeError as exc:
            skipped.append(f"{p.relative_to(tree)} (would break JSON: {exc})")
            continue
        if APPLY:
            p.write_text(updated, encoding="utf-8")
        occurrences += text.count(OLD)
        replaced.append(str(p.relative_to(tree)))
    print(f"{ms}: {'replaced' if APPLY else 'to replace'} {len(replaced)} files, "
          f"{occurrences} occurrences; skipped {len(skipped)}")
    for s in skipped[:4]:
        print(f"    skipped: {s}")
    for r in replaced[:3]:
        print(f"    {r}")
