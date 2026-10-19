#!/usr/bin/env python
"""Inventory `rust/` (legacy FreeCAD bridge crate) and classify every module for migration.

Output: `implementation/design/RUST-LEGACY-MIGRATION.md` (generated; do not hand-edit the table)
plus a console summary.

Classification rules (first pass, conservative):

* ``LEGACY_HOST``  - C ABI / FreeCAD / Qt glue (`*_ffi`, menu/toolbar/workbench shells). The target
  does not need the boundary, only the behaviour behind it.
* ``TEST``         - `*_tests.rs` modules: port the intent as a target test, not the code.
* ``STUB``         - under 0.5 KB and no real body.
* ``COVERED``      - the base token already exists in `crates/**` with a substantial file.
* ``PARTIAL``      - the token exists but only in a small/newer module, so behaviour needs review.
* ``PORT``         - no counterpart found in `crates/**`: candidate to migrate.
* ``BLOCKED``      - geometry that needs the OCCT bridge (M00-OCCT not approved).
"""
from __future__ import annotations

import pathlib
import re

ROOT = pathlib.Path(__file__).resolve().parents[2]
LEGACY = ROOT / "rust"
CRATES = ROOT / "crates"
OUTPUT = ROOT / "implementation/design/RUST-LEGACY-MIGRATION.md"

# Tokens that never identify a portable behaviour.
SUFFIXES = ("_ffi", "_tests", "_test", "_capability", "_ffi_contract")

BLOCKED_TOKENS = {
    "solid", "solid_construction", "solid_publication", "solid_options", "shape_validation",
    "topology_snapshot", "edge_geometry", "prepared_cad",
}

# Verified by reading the target crates during the 2026-10-17 assessment. These win over the
# heuristic; each note states what the target actually has (not what a keyword search implies).
CURATED: list[tuple[str, str, str]] = [
    (r"^units$|^numeric_units$|^tolerances$|^coordinate", "PORT",
     "target has only `Unit` + `Document.page_unit`; typed LengthMm/AngleDeg/TolerancePolicy is OM9-L2-01-08 (not implemented)"),
    (r"^cplane", "PARTIAL",
     "`CPlane` is the typed local frame for coordinate input (OM9-L3-01-08-007, done); the named-plane Registry/Snapshot of legacy cplanes.rs belongs to OM9-L2-03-01"),
    (r"^curve", "PORT",
     "target has curve data types + `evaluation.rs`; no Interp/fit/tangent/rebuild logic"),
    (r"^circle|^ellipse|^rectangle|^spline|^nurbs_basis|^polycurve|^bezier", "PORT",
     "target has NURBS data types only; the analytic/bezier/periodic construction logic is legacy-only"),
    (r"^surface|^support|^analytic_support|^derived_support|^surface_refit|^discrete", "PORT",
     "target has `NurbsSurface` data only; trim/refit/support evaluation is legacy-only"),
    (r"^gumball|^array_occurrences|^instance_graph", "PORT",
     "M02 gumball/transform is unstarted in the target (only a mention in scene/workspace)"),
    (r"^layer_clipboard|^layer_exchange|^layer_retained|^layer_legacy|^layer_codec", "PORT",
     "target has records/rules but no clipboard/exchange/retained lifecycle for layers"),
    (r"^layer", "PARTIAL",
     "`om9-core/src/layers.rs` + `om9-app/src/session/layers.rs` own records/rules/lifecycle"),
    (r"^history|^builder_history|^prepared_|^result_publication|^topology", "PARTIAL",
     "`om9-core/src/{history,result,transaction}.rs` own the graph/publication; builder semantics are legacy-only"),
    (r"^command_option|^command_completion|^command_repeat", "PARTIAL",
     "`om9-app/src/{option_parser,session}.rs` + L2-01-02 cover options/repeat/completion; schemas differ"),
    (r"^retained_archive|^core_3dm|^modeling", "PARTIAL",
     "`om9-exchange` owns the retained 3DM path; the legacy archive writing is broader"),
    (r"^core_(views|view_controls|keyboard|mouse|notes|picture_frame|snaps|objects)|^viewport_title",
     "PARTIAL", "`om9-app` UI (ui.rs, dock_layout.rs, viewport_menu.rs) covers the behaviour; no 1:1 module"),
    (r"^cage", "PORT", "no cage/SubD support in the target crates"),
    (r"^(gem|toolbar|workbench|commands)$", "STUB", "empty shell in legacy too"),
]

WAVE_BY_TOKEN = {
    "units": "W1 units/tolerance", "numeric_units": "W1 units/tolerance", "tolerances": "W1 units/tolerance",
    "coordinates": "W1 units/tolerance", "coordinate_input": "W1 units/tolerance",
    "extended_coordinates": "W1 units/tolerance", "cplanes": "W1 units/tolerance",
    "cplane_registry": "W1 units/tolerance",
    "curve": "W2 curve/geometry", "circle": "W2 curve/geometry", "ellipse": "W2 curve/geometry",
    "rectangle": "W2 curve/geometry", "spline": "W2 curve/geometry", "nurbs_basis": "W2 curve/geometry",
    "polycurve": "W2 curve/geometry", "fit_point": "W2 curve/geometry", "bezier": "W2 curve/geometry",
    "surface": "W3 surface", "support": "W3 surface", "analytic_support": "W3 surface",
    "derived_support": "W3 surface", "discrete": "W3 surface", "surface_refit": "W3 surface",
    "gumball": "W4 gumball/transform", "array": "W4 gumball/transform", "instance": "W4 gumball/transform",
    "layer": "W5 layers", "layers": "W5 layers",
    "command": "W6 command framework", "menu": "W6 command framework",
    "history": "W7 history/builder", "builder": "W7 history/builder", "prepared": "W7 history/builder",
    "result": "W7 history/builder", "reference_path": "W7 history/builder",
    "retained": "W8 exchange/3dm", "core_3dm": "W8 exchange/3dm", "modeling": "W8 exchange/3dm",
    "cage": "W9 cage/subd", "core": "W10 core ui/scene", "wire": "W10 core ui/scene",
    "workspace": "W10 core ui/scene", "viewport": "W10 core ui/scene",
}


def base_token(stem: str) -> str:
    for suffix in SUFFIXES:
        if stem.endswith(suffix):
            return stem[: -len(suffix)]
    return stem


def crate_index() -> dict[str, list[tuple[str, int]]]:
    index: dict[str, list[tuple[str, int]]] = {}
    for path in CRATES.rglob("*.rs"):
        stem = path.stem
        size = path.stat().st_size
        index.setdefault(stem, []).append((path.relative_to(ROOT).as_posix(), size))
    return index


def crate_text_index() -> dict[str, str]:
    """Token -> the crate files that mention it, so the table shows measured evidence."""
    return {}


def classify(path: pathlib.Path, index: dict[str, list[tuple[str, int]]]) -> tuple[str, str, str]:
    stem = path.stem
    token = base_token(stem)
    if stem.endswith("_ffi") or stem in {"menu", "toolbar", "workbench", "commands", "mod"}:
        return "LEGACY_HOST", "C ABI/UI glue; port the behaviour behind it, not the boundary", "—"
    if stem.endswith("_tests") or stem.endswith("_test"):
        return "TEST", "port the intent as a target test", "—"
    if path.stat().st_size < 512:
        return "STUB", "empty shell", "—"
    if token in BLOCKED_TOKENS:
        return "BLOCKED", "needs OCCT B-Rep (M00-OCCT not approved)", "OM9-L2-01-05"
    for pattern, verdict, note in CURATED:
        if re.search(pattern, stem):
            return verdict, note, "—"
    hits = index.get(stem, []) + index.get(token, [])
    hits = [(name, size) for name, size in hits if size > 4096]
    if hits:
        name, _size = max(hits, key=lambda item: item[1])
        verdict = "COVERED" if len(hits) > 1 or _size > 20000 else "PARTIAL"
        return verdict, "name match in the target (unverified)", name
    return "PORT", "no name match in crates/** (unverified)", "—"


def wave_for(stem: str) -> str:
    token = base_token(stem)
    for key, wave in WAVE_BY_TOKEN.items():
        if token.startswith(key) or key in token:
            return wave
    return "W11 review"


def main() -> int:
    index = crate_index()
    rows = []
    # The W11 "review" bucket is read by its own reproducible classifier; importing it keeps a
    # single source of truth for what was read and why.
    try:
        from classify_w11 import REVIEW as W11_REVIEW
        from classify_w11 import classify as w11_classify
    except ImportError:  # pragma: no cover - the tool lives beside this one
        W11_REVIEW, w11_classify = [], None
    for path in sorted((LEGACY / "src").rglob("*.rs")):
        verdict, why, counterpart = classify(path, index)
        module = path.relative_to(LEGACY).as_posix()
        # `module` carries the `src/` prefix; the classifier keys are relative to `src/`.
        if w11_classify is not None and module.removeprefix("src/") in W11_REVIEW:
            verdict, why = w11_classify(path)
            counterpart = "—"
        rows.append({
            "module": module,
            "kb": path.stat().st_size / 1024,
            "verdict": verdict,
            "why": why,
            "counterpart": counterpart,
            "wave": wave_for(path.stem),
        })
    tests = sorted((LEGACY / "tests").rglob("*.rs"))
    legacy_kb = sum(row["kb"] for row in rows)

    counts: dict[str, int] = {}
    kb_by_verdict: dict[str, float] = {}
    for row in rows:
        counts[row["verdict"]] = counts.get(row["verdict"], 0) + 1
        kb_by_verdict[row["verdict"]] = kb_by_verdict.get(row["verdict"], 0.0) + row["kb"]

    lines = [
        "# `rust/` (legacy FreeCAD bridge) — migration inventory",
        "",
        "Generated by `implementation/tools/assess_rust_legacy.py`. Do not hand-edit the table:",
        "re-run the tool after changing the crate or the rules.",
        "",
        f"- Legacy crate: `rust/` (`openmatrix9_rust`, staticlib+rlib, deps: serde/serde_json/sha2 only).",
        f"- Modules: **{len(rows)}** in `src/`, **{legacy_kb:.0f} KB** of Rust; tests: **{len(tests)}** files.",
        f"- Target: `crates/` at the repository root (six crates).",
        "",
        "## Verdict summary",
        "",
        "| Verdict | Modules | KB | Meaning |",
        "| --- | ---: | ---: | --- |",
    ]
    meaning = {
        "COVERED": "behaviour already has a substantial counterpart in `crates/**`",
        "PARTIAL": "counterpart exists but is small/new — review before assuming parity",
        "PORT": "no counterpart — migrate",
        "BLOCKED": "needs OCCT (M00-OCCT not approved)",
        "LEGACY_HOST": "C ABI / FreeCAD / Qt glue — port behaviour, not the boundary",
        "TEST": "test intent to port as target tests",
        "STUB": "empty shell",
    }
    for verdict in ("PORT", "PARTIAL", "COVERED", "BLOCKED", "LEGACY_HOST", "TEST", "STUB"):
        if counts.get(verdict):
            lines.append(f"| {verdict} | {counts[verdict]} | {kb_by_verdict[verdict]:.0f} | {meaning[verdict]} |")
    lines.append("")

    lines.append("## Modules")
    lines.append("")
    lines.append("| Module | KB | Verdict | Wave | Counterpart / reason |")
    lines.append("| --- | ---: | --- | --- | --- |")
    for row in sorted(rows, key=lambda r: (r["verdict"] != "PORT", -r["kb"])):
        counterpart = f"`{row['counterpart']}`" if row["counterpart"] != "—" else row["why"]
        lines.append(
            f"| `{row['module']}` | {row['kb']:.1f} | {row['verdict']} | {row['wave']} | {counterpart} |"
        )
    lines.append("")
    lines.append("## Method and limits")
    lines.append("")
    lines.append("- Verdicts for the large domains come from reading the target crates during the")
    lines.append("  2026-10-17 assessment (`CURATED` in the generator); the rest use a name-match heuristic")
    lines.append("  and are marked *unverified*.")
    lines.append("- A verdict is a **candidate**, never completion: a behaviour counts as migrated only when a")
    lines.append("  target test proves it, per AGENTS.md. `PORT` means \"no counterpart found\", not \"absent\".")
    lines.append("- `LEGACY_HOST` (`*_ffi`, menu/toolbar/workbench shells) exists only because the legacy host is")
    lines.append("  FreeCAD/Qt; the target calls Rust APIs directly, so the boundary is not ported.")
    lines.append("")
    lines.append("## Wave plan")
    lines.append("")
    lines.append("| Wave | Modules | KB | Unlocks |")
    lines.append("| --- | ---: | ---: | --- |")
    wave_notes = {
        "W1 units/tolerance": "typed LengthMm/AngleDeg/TolerancePolicy (owner OM9-L2-01-08); unblocks numeric input + M02 snapping",
        "W2 curve/geometry": "Interp/fit/tangent/circle/ellipse/rectangle construction on the existing geometry data types",
        "W3 surface": "surface trim/refit/support evaluation over `NurbsSurface`",
        "W4 gumball/transform": "M02 move/copy/rotate/scale + occurrence arrays",
        "W5 layers": "layer clipboard/exchange/retained lifecycle (records already exist)",
        "W6 command framework": "option schemas/completion beyond what L2-01-02 already ported",
        "W7 history/builder": "builder semantics on top of the existing HistoryGraph/Recipe",
        "W8 exchange/3dm": "broader 3DM archive writing on top of the retained path",
        "W9 cage/subd": "cage/SubD deformation (M08)",
        "W10 core ui/scene": "small UI/scene behaviours not yet 1:1",
        "W11 review": "unclassified — read before deciding",
    }
    by_wave: dict[str, list[dict]] = {}
    for row in rows:
        by_wave.setdefault(row["wave"], []).append(row)
    for wave in sorted(by_wave):
        bucket = by_wave[wave]
        kb = sum(row["kb"] for row in bucket)
        lines.append(f"| {wave} | {len(bucket)} | {kb:.0f} | {wave_notes.get(wave, '—')} |")
    lines.append("")
    lines.append("## Wave progress (ported so far)")
    lines.append("")
    lines.append("| Wave | Milestone / L2 owner | Converted | Design record |")
    lines.append("| --- | --- | ---: | --- |")
    progress = {
        "W1": (
            "M01 `OM9-L2-01-08`",
            "5/5 modules, 7/7 L3 accepted on automated evidence",
            "`implementation/design/M01/OM9-L2-01-08/api.md` (§8)",
        ),
        "W2": (
            "M03 `OM9-L2-02-04`",
            "12 kernel functions, 21 tests (construction + conversion)",
            "`implementation/design/M03/OM9-L2-02-04/W2-CURVE-KERNEL.md`",
        ),
        "W3": (
            "M03 `OM9-L2-02-05`",
            "13 kernel functions, 21 tests (evaluate/iso-curve/refit/loft/sweep1/sweep2 + path frames + analytic builders)",
            "`implementation/design/M03/OM9-L2-02-05/W3-SURFACE-KERNEL.md`",
        ),
        "W4": (
            "M02 `OM9-L2-03-03`",
            "handle model + constrained drag + rotation/scale + occurrence arrays + drag session (Move/Copy -> 1 Undo), 15 tests",
            "`implementation/design/M02/OM9-L2-03-03/W4-GUMBALL-KERNEL.md`",
        ),
        "W5": (
            "M01 `OM9-L2-01-12` (clipboard: no L3 owner)",
            "layer clipboard + paste remap + exchange + versioned codec, 10 tests",
            "`implementation/design/M01/W5-LAYER-CLIPBOARD.md`",
        ),
        "W6": (
            "M01 `OM9-L2-01-02`",
            "typed option schema (domain/default/required) + fill/validate + completion, 7 tests",
            "`implementation/design/M01/W6-COMMAND-SCHEMA.md`",
        ),
        "W7": (
            "M04 `OM9-L2-01-04`",
            "stale detection + dirty propagation + topological recompute plan + detach/freeze, 7 tests",
            "`implementation/design/M04/OM9-L2-01-04/W7-HISTORY-OPS.md`",
        ),
        "W8/W9/W10": (
            "M03/M08/M01",
            "modelling capability policy + cage rational evaluation + measurements, 12 tests",
            "`implementation/design/M01/W8-W11-PORT-AND-REVIEW.md`",
        ),
        "W11 review": (
            "classification only (no code)",
            "33 modules classified: 14 LEGACY_HOST, 9 PORT, 5 BLOCKED, 2 COVERED, 3 STUB",
            "`implementation/design/M01/W8-W11-PORT-AND-REVIEW.md` + `implementation/tools/classify_w11.py`",
        ),
    }
    for wave in ("W1", "W2", "W3", "W4", "W5", "W6", "W7", "W8/W9/W10", "W11 review"):
        owner, converted, record = progress[wave]
        lines.append(f"| {wave} | {owner} | {converted} | {record} |")
    lines.append("")
    lines.append("- Only W1 closed its L3 set. W2-W10 are **kernel ports**: they carry tests and survive")
    lines.append("  review, but no M03 L3 is marked accepted — closing an M03 L3 needs its own runbook")
    lines.append("  (source audit, oracle, gate), per AGENTS.md.")
    lines.append("- W3 trim/split/untrim is blocked by the *model*, not by missing code: the target has no")
    lines.append("  trimmed B-Rep record (`Brep` returns `Unsupported(\"brep geometry\")`) until M00/OCCT is")
    lines.append("  decided.")
    lines.append("")
    lines.append("## Largest PORT modules")
    lines.append("")
    ports = sorted((row for row in rows if row["verdict"] == "PORT"), key=lambda r: -r["kb"])[:10]
    for row in ports:
        lines.append(f"- `{row['module']}` — {row['kb']:.1f} KB · {row['wave']}")
    lines.append("")
    lines.append("## Test files to port as target tests")
    lines.append("")
    for path in tests:
        lines.append(f"- `{path.relative_to(LEGACY).as_posix()}`")
    lines.append("")
    OUTPUT.write_text("\n".join(lines), encoding="utf-8")
    print(f"wrote {OUTPUT.relative_to(ROOT).as_posix()}: {len(rows)} modules, {len(tests)} tests")
    for verdict in ("PORT", "PARTIAL", "COVERED", "BLOCKED", "LEGACY_HOST", "TEST", "STUB"):
        if counts.get(verdict):
            print(f"  {verdict:12} {counts[verdict]:3} modules  {kb_by_verdict[verdict]:7.0f} KB")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
