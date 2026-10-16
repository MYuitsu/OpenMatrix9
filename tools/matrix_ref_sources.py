"""Inventory the private Matrix9 reference exports used for spec crosswalks.

Read-only. Emits identifiers, paths and provenance only - no reference source
bodies are copied into the repository deliverable.
"""
import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REF = ROOT / "ref" / "matrix"
OUT = ROOT / "build" / "matrix-ref-sources.json"

FORM_RE = re.compile(r"^\s*(?:Public\s+|Private\s+)?(?:VB_Exposed\s+)?(?:Begin\s+)?VB_Name\s*=\s*\"([^\"]+)\"", re.M)
DECL_RE = re.compile(r"^(?:Public\s+|Private\s+|Friend\s+)?(?:Sub|Function|Property\s+(?:Get|Let|Set))\s+([A-Za-z_][A-Za-z0-9_]*)", re.M)
CONST_RE = re.compile(r"^(?:Public\s+|Private\s+)?Const\s+([A-Za-z_][A-Za-z0-9_]*)\s*(?:As\s+[A-Za-z0-9_.()]+)?\s*=", re.M)
CONTROL_RE = re.compile(r"^\s*Begin\s+([A-Za-z0-9_.]+)\s+([A-Za-z_][A-Za-z0-9_]*)", re.M)
CAPTION_RE = re.compile(r"^\s*Caption\s*=\s*\"([^\"]{1,80})\"", re.M)


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def dotnet_sources():
    rows = []
    base = REF / "decompiled-ui"
    for path in sorted(base.rglob("*")):
        if not path.is_file() or path.suffix.lower() not in {".cs", ".xaml"}:
            continue
        if "Designer" in path.name or path.suffix.lower() == ".resx":
            continue
        rel = path.relative_to(base).as_posix()
        project = rel.split("/")[0]
        ns = None
        symbols = []
        try:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        if path.suffix == ".cs":
            match = re.search(r"^namespace\s+([A-Za-z0-9_.]+)", text, re.M)
            ns = match.group(1) if match else None
            symbols = re.findall(
                r"^\s*(?:public|internal|protected)\s+(?:sealed\s+|abstract\s+|static\s+|partial\s+)*"
                r"(?:class|struct|enum|interface)\s+([A-Za-z_][A-Za-z0-9_]*)", text, re.M)
        rows.append({
            "kind": "dotnet_source",
            "project": project,
            "path": rel,
            "namespace": ns,
            "symbols": symbols[:12],
            "bytes": path.stat().st_size,
        })
    return rows


def vb6_sources():
    rows = []
    base = REF / "vb6-lite"
    for path in sorted(base.rglob("*")):
        if not path.is_file() or path.suffix.lower() not in {".frm", ".bas", ".cls", ".ctl"}:
            continue
        rel = path.relative_to(base).as_posix()
        try:
            text = path.read_text(encoding="latin-1", errors="replace")
        except OSError:
            continue
        name = None
        if path.suffix.lower() in {".frm", ".ctl"}:
            match = FORM_RE.search(text)
            name = match.group(1) if match else None
        if not name:
            name = path.stem
        declarations = DECL_RE.findall(text) if path.stat().st_size < 4_000_000 else []
        controls = sorted(set(CONTROL_RE.findall(text))) if path.suffix.lower() in {".frm", ".ctl"} else []
        rows.append({
            "kind": "vb6_module",
            "project": rel.split("/")[0],
            "path": rel,
            "module_name": name,
            "suffix": path.suffix.lower(),
            "bytes": path.stat().st_size,
            "declaration_count": len(declarations),
            "declarations": declarations[:40],
            "control_types": sorted({c[0] for c in controls})[:60],
            "captions": sorted(set(CAPTION_RE.findall(text)))[:40],
        })
    return rows


def native_inspection():
    rows = []
    base = REF / "decompiled-ui" / "native-inspection"
    for manifest in sorted(base.rglob("manifest.json")):
        data = json.loads(manifest.read_text(encoding="utf-8-sig"))
        rows.append({
            "kind": "native_inspection",
            "path": manifest.relative_to(ROOT).as_posix(),
            "binary": data.get("binary") or data.get("source"),
            "sha256": data.get("sha256") or data.get("source_sha256"),
            "keys": sorted(data.keys()),
        })
    return rows


def milestone_reference_sheets():
    """List the extracted reference sheets shipped with the milestone specs.

    Identifiers only (path, milestone, size) - the sheet text is authored
    behavioural summary material and is not copied anywhere by this tool.
    """
    rows = []
    for path in sorted(ROOT.joinpath("OM9_Milestones").rglob("*")):
        if not path.is_file() or path.suffix != ".md" or path.parent.name != "text":
            continue
        rel = path.relative_to(ROOT).as_posix()
        parts = rel.split("/")
        milestone = parts[1].split(" ")[0] if len(parts) > 1 else parts[0]
        rows.append({
            "milestone": milestone,
            "path": rel,
            "sheet": path.stem,
            "bytes": path.stat().st_size,
        })
    return rows


def main():
    payload = {
        "schema_version": 1,
        "generated_from": "ref/matrix (workspace-local private reference export)",
        "source_identity": {
            "dotnet_export": "ref/matrix/decompiled-ui/export-manifest.json",
            "vb6_export": "ref/matrix/vb6-lite/source-manifest.json",
            "native_inspection_root": "ref/matrix/decompiled-ui/native-inspection",
            "ghidra_listing": "ref/matrix/ghidra/Matrix90.exe.c",
            "official_help_html": "ref/matrix/vb6-lite/tool-help/html",
        },
        "dotnet_source_count": 0,
        "vb6_module_count": 0,
        "native_inspection_count": 0,
        "milestone_reference_sheet_count": 0,
        "dotnet_sources": [],
        "vb6_modules": [],
        "native_inspection": [],
        "milestone_reference_sheets": [],
    }
    payload["dotnet_sources"] = dotnet_sources()
    payload["vb6_modules"] = vb6_sources()
    payload["native_inspection"] = native_inspection()
    payload["milestone_reference_sheets"] = milestone_reference_sheets()
    payload["dotnet_source_count"] = len(payload["dotnet_sources"])
    payload["vb6_module_count"] = len(payload["vb6_modules"])
    payload["native_inspection_count"] = len(payload["native_inspection"])
    payload["milestone_reference_sheet_count"] = len(payload["milestone_reference_sheets"])
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(payload, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
    print(json.dumps({k: payload[k] for k in
                      ("dotnet_source_count", "vb6_module_count", "native_inspection_count")}))
    print("\n".join(sorted({r["module_name"] for r in payload["vb6_modules"] if r["suffix"] in {".frm", ".ctl"}})))


if __name__ == "__main__":
    main()
