"""Roll up framework progress from FEATURES.json component lists.

Read-only. Prints how many component features of each framework are accepted.
A component counts as accepted when its `implementation_status` is no longer
`not_implemented_on_standalone_om9`. This reports catalog status only; it is not
runtime or semantic acceptance evidence.
"""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACKAGE = ROOT / "OpenMatrix9_Codex_Spec_v2"
NOT_DONE = "not_implemented_on_standalone_om9"


def main():
    features = json.loads((PACKAGE / "FEATURES.json").read_text(encoding="utf-8"))
    byid = {f["id"]: f for f in features}
    frameworks = [f for f in features if f.get("framework_components")]
    rows = []
    for f in frameworks:
        comps = f["framework_components"]
        done = [c for c in comps if byid[c]["implementation_status"] != NOT_DONE]
        rows.append((f["id"], f["name"], len(done), len(comps), f.get("framework_min_close", "")))
    rows.sort(key=lambda r: (-(r[3] and r[2] / r[3]), r[0]))
    print(f"{'framework':22} {'done':>7}  {'min-close':<56} open")
    for fid, name, done, total, why in rows:
        print(f"{fid:22} {done:3}/{total:<3}  {why[:56]:<56} {total - done}")
    total_links = sum(1 for f in features if f.get("part_of_frameworks"))
    print()
    print(f"frameworks: {len(rows)} | features linked to a framework: {total_links}")
    print(f"catalog status source: FEATURES.json | not-done marker: {NOT_DONE}")


if __name__ == "__main__":
    main()
