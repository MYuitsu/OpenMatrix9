"""Independent measurement of the VB6 export: real source vs auto-generated stubs.

Cross-checks the delegated VB6 survey. Distinguishes human/event procedure names
from VB Decompiler's synthesised `Proc_<n>_<n>_<HEX>` names, and inventories the
form classes and control types. Names and counts only.
"""
import json
import re
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path("H:/OpenMatrix9")
VB6 = ROOT / "ref/matrix/vb6-lite"
OUT = ROOT / "build" / "ref-vb6-measure.json"

DECL_RE = re.compile(r"^\s*(?:Public\s+|Private\s+|Friend\s+)?(?:Static\s+)?"
                     r"(?:Sub|Function|Property\s+(?:Get|Let|Set))\s+([A-Za-z_][A-Za-z0-9_]*)", re.M)
SYNTH_RE = re.compile(r"^Proc_\d+_\d+_[0-9A-Fa-f]{5,}$")
VB_NAME_RE = re.compile(r'^\s*VB_Name\s*=\s*"([^"]+)"', re.M)
CONTROL_RE = re.compile(r"^\s*Begin\s+([A-Za-z0-9_.]+)\s+([A-Za-z_][A-Za-z0-9_]*)", re.M)
CAPTION_RE = re.compile(r'^\s*Caption\s*=\s*"([^"]{1,120})"', re.M)


def measure(path):
    text = path.read_text(encoding="latin-1", errors="replace")
    names = DECL_RE.findall(text)
    synth = [n for n in names if SYNTH_RE.match(n)]
    real = [n for n in names if not SYNTH_RE.match(n)]
    return {
        "path": path.relative_to(VB6).as_posix(),
        "suffix": path.suffix.lower(),
        "bytes": path.stat().st_size,
        "procedures": len(names),
        "real_names": len(real),
        "synthetic_names": len(synth),
        "module_name": (VB_NAME_RE.search(text).group(1) if VB_NAME_RE.search(text) else path.stem),
        "captions": len(set(CAPTION_RE.findall(text))),
        "controls": sorted({c[0] for c in CONTROL_RE.findall(text)}),
    }


def main():
    rows = [measure(p) for p in sorted(VB6.rglob("*"))
            if p.is_file() and p.suffix.lower() in {".frm", ".bas", ".cls", ".ctl"}]
    by_ext = defaultdict(Counter)
    for r in rows:
        c = by_ext[r["suffix"]]
        c["files"] += 1
        c["real_names"] += r["real_names"]
        c["synthetic_names"] += r["synthetic_names"]
        if r["real_names"] and r["real_names"] >= r["synthetic_names"]:
            c["real_source_dominant"] += 1
        elif r["synthetic_names"]:
            c["stub_dominant"] += 1
        else:
            c["no_procedures"] += 1

    real_src = sorted((r for r in rows if r["real_names"] >= max(1, r["synthetic_names"])),
                      key=lambda r: -r["real_names"])
    form_classes = Counter()
    for r in rows:
        if r["suffix"] in {".frm", ".ctl"}:
            name = r["module_name"]
            if name.startswith("frmMenu"):
                form_classes["frmMenu*"] += 1
            elif name.startswith("frmCMenu"):
                form_classes["frmCMenu*"] += 1
            elif name.startswith("frmICBuilder"):
                form_classes["frmICBuilder*"] += 1
            elif name.startswith("frmBuilder"):
                form_classes["frmBuilder*"] += 1
            elif name.startswith("frmDialog"):
                form_classes["frmDialog*"] += 1
            elif name.startswith("frmC"):
                form_classes["frmC*"] += 1
            else:
                form_classes["other"] += 1
    control_vocab = Counter()
    for r in rows:
        for c in r["controls"]:
            control_vocab[c] += 1

    payload = {
        "schema": "om9.ref_vb6_measure.v1",
        "totals": {
            "code_files": len(rows),
            "real_names": sum(r["real_names"] for r in rows),
            "synthetic_names": sum(r["synthetic_names"] for r in rows),
            "with_any_caption": sum(1 for r in rows if r["captions"]),
        },
        "by_extension": {k: dict(v) for k, v in by_ext.items()},
        "form_classes": dict(form_classes),
        "control_vocabulary": dict(control_vocab.most_common(30)),
        "modules_with_most_real_source": [
            {"path": r["path"], "module": r["module_name"], "real": r["real_names"],
             "synth": r["synthetic_names"], "bytes": r["bytes"]}
            for r in real_src[:20]
        ],
    }
    OUT.write_text(json.dumps(payload, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")

    t = payload["totals"]
    print(f"code files: {t['code_files']}  real procedure names: {t['real_names']}  "
          f"synthetic Proc_ names: {t['synthetic_names']}  files with a caption: {t['with_any_caption']}")
    print("\nby extension:")
    for ext, raw in sorted(payload["by_extension"].items()):
        c = Counter(raw)
        print(f"  {ext:6s} files={c['files']:4d} real={c['real_names']:5d} "
              f"synth={c['synthetic_names']:5d} real-dominant={c['real_source_dominant']:4d} "
              f"stub-dominant={c['stub_dominant']:4d} no-procs={c['no_procedures']:3d}")
    print("\nform classes:", payload["form_classes"])
    print("\ntop modules by real procedure names:")
    for r in payload["modules_with_most_real_source"][:12]:
        print(f"  {r['real']:4d} real / {r['synth']:4d} synth  {r['path']}")
    print("\ntop control types:")
    for name, n in list(payload["control_vocabulary"].items())[:14]:
        print(f"  {n:4d}  {name}")


if __name__ == "__main__":
    main()
