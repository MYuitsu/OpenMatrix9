#!/usr/bin/env python
"""Classify the W11 "review" bucket by reading each module (wave plan: `W11 review`).

The wave plan left 33 modules unclassified ("read before deciding"). This tool applies the
reading rules that the migration record documents, so the classification is reproducible
instead of a one-off opinion:

1. ``LEGACY_HOST``  — the module is a C ABI / Qt glue shell: it exports ``extern "C"`` or
   ``no_mangle`` symbols and has (almost) no portable public API of its own.
2. ``BLOCKED``      — the portable behaviour needs OCCT: the file touches ``TopoDS``/``BRep``/
   ``TopExp``/``opencascade``. These wait for ``OM9-L2-01-05`` and the M00 OCCT decision.
3. ``COVERED``      — the target already owns the same behaviour (curated table below).
4. ``STUB``         — an empty shell with no public item.
5. ``PORT``         — portable logic that the target does not have yet.

    python implementation/tools/classify_w11.py            # table only
    python implementation/tools/classify_w11.py --json     # machine-readable
"""
from __future__ import annotations

import argparse
import json
import pathlib
import re

ROOT = pathlib.Path(__file__).resolve().parents[2]
LEGACY = ROOT / "rust" / "src"

# Modules whose behaviour the target already owns (verified during the W1-W10 work).
COVERED = {
    "solid/geometry.rs": "`crates/om9-core/src/geometry.rs` owns the geometry records",
    "lib.rs": "`crates/om9-archive/src/lib.rs` owns the container surface",
    "workspace.rs": "`crates/om9-scene/src/workspace.rs` owns the workspace grid",
}

# The plan's `W11 review` bucket (modules the wave plan left unclassified).
REVIEW = [
    "geometry_contract.rs", "ffi.rs", "phase2_snap.rs", "edit.rs", "hatch_loop_input.rs",
    "source_attributes.rs", "native_solid_capability.rs", "phase2_session.rs",
    "native_input_tree.rs", "native_edit_capability.rs", "geometry_frame.rs",
    "native_rebuild_capability.rs", "worker_budget.rs", "state.rs", "solid.rs",
    "phase2_ffi.rs", "phase2_snap_ffi.rs", "geometry_contract_ffi.rs", "topology_snapshot.rs",
    "solid/geometry.rs", "phase3_ffi.rs", "edge_geometry.rs", "topology_snapshot_ffi.rs",
    "source_attributes_ffi.rs", "shape_validation.rs", "lib.rs", "native_input_tree_ffi.rs",
    "edge_geometry_ffi.rs", "geometry_frame_ffi.rs", "ui/mod.rs", "toolbar.rs", "workbench.rs",
    "gem.rs",
]

# Reading exceptions: these modules do not touch OCCT symbols directly, but their payload is
# solid topology, so they stay blocked behind `OM9-L2-01-05` until that kernel exists.
SOLID_TOPOLOGY = {
    "topology_snapshot.rs": "payload is solid topology; waits for `OM9-L2-01-05`",
    "edge_geometry.rs": "payload is B-Rep edge geometry; waits for `OM9-L2-01-05`",
    "shape_validation.rs": "validates native shapes; waits for `OM9-L2-01-05`",
}

OCCT = re.compile(r"TopoDS|BRep|TopExp|TopTools|opencascade|occ::")
FFI = re.compile(r'#\[unsafe\(no_mangle\)\]|extern "C"')
PUB = re.compile(r"^pub fn |^pub struct |^pub enum |^pub const ", re.MULTILINE)


def classify(path: pathlib.Path) -> tuple[str, str]:
    name = path.relative_to(LEGACY).as_posix()
    if name in COVERED:
        return "COVERED", COVERED[name]
    if name in SOLID_TOPOLOGY:
        return "BLOCKED", SOLID_TOPOLOGY[name]
    text = path.read_text(encoding="utf-8", errors="replace")
    kb = len(text.encode("utf-8")) / 1024
    public = len(PUB.findall(text))
    if OCCT.search(text):
        return "BLOCKED", "needs OCCT (`OM9-L2-01-05`, M00 decision)"
    # A `*_ffi` module is the C ABI shell by naming convention, whatever helpers it holds.
    if name.endswith("_ffi.rs") or (FFI.search(text) and public <= 2):
        return "LEGACY_HOST", "C ABI glue: exports symbols, owns no portable API"
    if kb < 0.2 or public == 0:
        return "STUB", "empty shell, no public item"
    if "no kernel or host pointers" in text or "Portable" in text[:400]:
        return "PORT", "portable logic by its own module note; not in the target yet"
    return "PORT", "public Rust API with no target counterpart"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()
    rows = []
    for name in REVIEW:
        path = LEGACY / name
        if not path.exists():
            rows.append(
                {"module": name, "kb": 0.0, "verdict": "MISSING", "reason": "file absent"}
            )
            continue
        verdict, reason = classify(path)
        rows.append(
            {
                "module": name,
                "kb": round(len(path.read_text(encoding="utf-8", errors="replace").encode()) / 1024, 1),
                "verdict": verdict,
                "reason": reason,
            }
        )
    if args.json:
        print(json.dumps(rows, indent=2))
        return 0
    counts: dict[str, int] = {}
    for row in rows:
        counts[row["verdict"]] = counts.get(row["verdict"], 0) + 1
    for row in rows:
        print(f"{row['verdict']:12} {row['kb']:6.1f}  {row['module']:28} {row['reason']}")
    print("counts:", ", ".join(f"{key}={value}" for key, value in sorted(counts.items())))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
