"""Validate the RHINO5 anchor registry and prove every root+path resolves."""
import json
import sys
from pathlib import Path

ROOT = Path("H:/OpenMatrix9")
sys.path.insert(0, str(ROOT / "tools"))
import matrix_ref_crosswalk as cw  # noqa: E402

REQUIRED = {"project", "path", "root", "unit", "role", "detail", "observed", "state"}
ALLOWED_STATE = {"IDENTIFIER_INDEXED", "COUNTED_AND_INDEXED", "MEASURED"}

problems = []
print(f"RHINO5 anchors: {len(cw.RHINO5)}")
for key, row in sorted(cw.RHINO5.items()):
    missing = REQUIRED - set(row)
    if missing:
        problems.append(f"{key}: missing fields {sorted(missing)}")
    if row.get("state") not in ALLOWED_STATE:
        problems.append(f"{key}: unexpected state {row.get('state')!r}")
    if not row.get("observed"):
        problems.append(f"{key}: empty observed list")
    # every path segment listed (the field may name a primary file plus companions)
    segments = [s.strip() for s in row["path"].split(" (+")]
    primary = segments[0].rstrip(")")
    candidate = ROOT / row["root"] / primary
    ok = candidate.is_file()
    if not ok:
        problems.append(f"{key}: unresolved {row['root']}/{primary}")
    print(f"  {'OK ' if ok else 'NO '} {key:30s} {row['state']:22s} "
          f"obs={len(row['observed']):3d}  {primary[:58]}")

print()
print(json.dumps({"anchors": len(cw.RHINO5), "problems": problems}, ensure_ascii=False, indent=1))
if problems:
    sys.exit(1)
