"""Read-only OpenMatrix9 document routing. Uses only the Python standard library."""

import argparse
import json
from pathlib import Path
import sys


SPEC = Path("ref/matrix9/OpenMatrix9_Codex_Spec_v1")
ROUTES = Path(__file__).resolve().parents[1] / "references/stages.json"


def read_json(path):
    try:
        return json.loads(path.read_text(encoding="utf-8-sig"))
    except (OSError, ValueError) as exc:
        raise ValueError(f"Cannot read {path}: {exc}") from exc


def catalog(root):
    entries = read_json(root / SPEC / "FEATURES.json")
    if not isinstance(entries, list):
        raise ValueError("FEATURES.json must contain a list")
    ids = [entry.get("id") for entry in entries if isinstance(entry, dict)]
    if len(ids) != len(entries) or any(not isinstance(i, str) or not i for i in ids):
        raise ValueError("Every feature must be an object with a nonempty string id")
    if len(set(ids)) != len(ids):
        raise ValueError("Duplicate feature IDs in FEATURES.json")
    return entries


def spec_file(root, feature):
    package = (root / SPEC).resolve()
    relative = feature.get("spec_path")
    if not isinstance(relative, str) or not relative:
        raise ValueError(f"Missing spec_path: {feature['id']}")
    path = (package / relative).resolve()
    if not path.is_relative_to(package):
        raise ValueError(f"Spec path outside package: {relative}")
    if not path.is_file():
        raise ValueError(f"Missing spec file: {path}")
    return str(path)


def path_record(root, path):
    absolute = (root / path).resolve()
    return {"path": str(absolute), "exists": absolute.exists()}


def route(root, stage):
    routes = read_json(ROUTES)
    if stage not in routes["stages"]:
        raise ValueError(f"Unknown stage {stage}; choose {', '.join(routes['stages'])}")
    definition = routes["stages"][stage]
    return {
        "stage": stage,
        "skill": definition["skill"],
        "read_now": [path_record(root, p) for p in definition["read_now"]],
        "read_if": [
            {"when": item["when"], "files": [path_record(root, p) for p in item["paths"]]}
            for item in definition.get("read_if", [])
        ],
        "exit_criteria": definition["exit_criteria"],
        "note": "A document's existence is not evidence of completed implementation.",
    }


def status(root):
    paths = {
        "menu_design_exists": "docs/superpowers/specs/2026-10-03-matrix9-menu-design.md",
        "menu_plan_exists": "docs/superpowers/plans/2026-10-03-matrix9-menu.md",
        "rust_menu_exists": "rust/src/menu.rs",
        "rust_state_exists": "rust/src/state.rs",
        "rust_ffi_exists": "rust/src/ffi.rs",
        "qt_sidebar_exists": "Gui/MatrixSidebar.cpp",
        "ghidra_export_exists": "ref/matrix9/ghidra/Matrix90.exe.c",
    }
    ledger_path = root / "docs/openmatrix9-progress.json"
    ledger = read_json(ledger_path) if ledger_path.exists() else None
    return {
        "project_root": str(root),
        "ledger_file": str(ledger_path),
        "ledger": ledger,
        "observations": {key: (root / path).is_file() for key, path in paths.items()},
        "note": "Presence checks are hints. Reconcile ledger with code, tests and runtime evidence.",
    }


def feature(root, identifier):
    matches = [entry for entry in catalog(root) if entry["id"] == identifier]
    if not matches:
        raise ValueError(f"Unknown feature ID: {identifier}")
    entry = matches[0]
    return {
        "feature": entry,
        "spec_file": spec_file(root, entry),
        "read_next": [path_record(root, str(SPEC / name)) for name in (
            "IMPLEMENTATION_RULES.md", "SOURCES.md"
        )],
        "note": "Catalog status and option cues do not establish live progress or option defaults. PDF pages are 1-based.",
    }


def search(root, query, domain, limit):
    query = query.casefold()
    found = []
    for entry in catalog(root):
        if domain and entry.get("domain") != domain:
            continue
        values = [str(entry.get(key) or "").casefold() for key in ("id", "name", "command")]
        if any(query in value for value in values):
            found.append((0 if query in values else 1, entry))
    found.sort(key=lambda item: (item[0], item[1]["id"]))
    fields = ("id", "name", "command", "domain", "spec_path")
    return {
        "query": query, "domain": domain, "total": len(found),
        "features": [{key: entry.get(key) for key in fields} for _, entry in found[:limit]],
    }


def audit(root):
    entries = catalog(root)
    errors = []
    for entry in entries:
        try:
            spec_file(root, entry)
        except ValueError as exc:
            errors.append(str(exc))
    stages = read_json(ROUTES)["stages"]
    missing = []
    conditional_missing = []
    for name in stages:
        result = route(root, name)
        missing.extend({"stage": name, **item} for item in result["read_now"] if not item["exists"])
        for group in result["read_if"]:
            conditional_missing.extend(
                {"stage": name, "when": group["when"], **item}
                for item in group["files"] if not item["exists"]
            )
    return {
        "feature_count": len(entries),
        "domain_count": len({entry.get("domain") for entry in entries}),
        "stage_count": len(stages), "invalid_specs": errors,
        "missing_required_documents": missing,
        "missing_conditional_documents": conditional_missing,
        "ok": not errors and not missing,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--project-root", required=True, type=Path,
                        help="Explicit OpenMatrix9 checkout; never inferred from the installed skill")
    commands = parser.add_subparsers(dest="command", required=True)
    lookup = commands.add_parser("feature", help="Resolve an exact OM9 feature ID")
    lookup.add_argument("id")
    find = commands.add_parser("search", help="Search name, command or ID without reading all specs")
    find.add_argument("query")
    find.add_argument("--domain")
    find.add_argument("--limit", type=int, default=5)
    stages = commands.add_parser("route", help="List documents required now and conditional later reads")
    stages.add_argument("stage")
    commands.add_parser("status", help="Read live ledger plus file presence hints")
    commands.add_parser("audit", help="Check all feature paths and required route documents")
    args = parser.parse_args()
    root = args.project_root.expanduser().resolve()
    try:
        if not root.is_dir():
            raise ValueError(f"Project root does not exist: {root}")
        if args.command == "feature":
            result = feature(root, args.id)
        elif args.command == "search":
            if args.limit < 1:
                raise ValueError("--limit must be positive")
            result = search(root, args.query, args.domain, args.limit)
        elif args.command == "route":
            result = route(root, args.stage)
        elif args.command == "audit":
            result = audit(root)
        else:
            result = status(root)
        print(json.dumps(result, ensure_ascii=False, indent=2))
        return 2 if args.command == "audit" and not result["ok"] else 0
    except (ValueError, KeyError, TypeError) as exc:
        print(json.dumps({"error": str(exc), "project_root": str(root)}, ensure_ascii=False))
        return 2


if __name__ == "__main__":
    sys.stdout.reconfigure(encoding="utf-8")
    sys.exit(main())
