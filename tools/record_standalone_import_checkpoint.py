"""Record measured standalone import slices alongside untouched legacy history."""
import datetime as dt
import json
from pathlib import Path
import shutil

ROOT = Path(__file__).resolve().parents[1]


def read(path):
    return json.loads((ROOT / path).read_text(encoding="utf-8"))


def main():
    workspace = read("docs/validation/2026-10-13-standalone-workspace.json")
    imported = read("docs/validation/2026-10-13-standalone-import.json")
    exported = read("docs/validation/2026-10-13-standalone-exchange.json")
    projection = read("docs/validation/2026-10-13-standalone-native-source-projection.json")
    inventory = read("docs/validation/2026-10-13-standalone-native-source-inventory.json")
    conversion = read("build/standalone-native/native-conversion-validation.json")
    conversion_peak = max(run["stats"]["peak_committed_bytes"] for run in conversion["runs"])
    assert workspace["ignored"] == imported["rust_native_import_tests_passed"] + exported["rust_lifecycle_tests_passed"]
    assert (projection["runs"], projection["rejections"], inventory["runs"], inventory["rejections"], len(conversion["runs"]), len(conversion["rejection_cases"])) == (82,32,68,17,50,40)
    shutil.copyfile(ROOT / "build/standalone-native/native-conversion-validation.json",
                    ROOT / "docs/validation/2026-10-13-standalone-native-conversion.json")
    total = workspace["passed"] + workspace["ignored"]
    ledger_path = ROOT / "docs/validation/opennurbs-test-roadmap.json"
    ledger = json.loads(ledger_path.read_text(encoding="utf-8"))
    checkpoint = {
        "status": "partially_implemented", "full_rcore00_complete": False,
        "rust_workspace_passed": workspace["passed"], "rust_suites": workspace["suites"],
        "rust_native_export_passed": 7, "rust_native_import_passed": 8,
        "distinct_rust_tests_exercised": total, "source_import_fixtures": imported["fixtures"],
        "native_projection_runs": 82, "native_projection_rejections": 32,
        "native_inventory_runs": 68, "native_current_conversion_runs": 50,
        "prepared_standalone_pending_batches": [], "complete_rcore00_gates_unchecked": 12,
        "remaining_total_test_batches": None,
        "next_action": "Prepare current-state retained merge/remapping/edit/delete/new matrix; full resources/clipboard/performance and actual application acceptance follow.",
        "evidence": "docs/validation/2026-10-13-standalone-import.json",
    }
    ledger["updated_at"] = dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds")
    ledger["standalone_foundation_checkpoint"] = checkpoint
    for batch in [{"id":"rcore00-native-source-projection-v3","package":7,"standalone":True,"native_runs":82,"rejections":32,"status":"passed_scoped","evidence":"docs/validation/2026-10-13-standalone-native-source-projection.json"},
                  {"id":"rcore00-native-to-host-import-v3","package":7,"standalone":True,"fixtures":15,"completed_cases":8,"passed":8,"failed":0,"status":"passed_scoped","evidence":"docs/validation/2026-10-13-standalone-import.json"}]:
        ledger["completed_batches"] = [b for b in ledger["completed_batches"] if b["id"] != batch["id"]]
        ledger["completed_batches"].append(batch)
    ledger_path.write_text(json.dumps(ledger,ensure_ascii=False,indent=2)+"\n",encoding="utf-8")
    marker = "<!-- standalone-import-checkpoint -->"
    roadmap = ROOT / "docs/validation/opennurbs-test-roadmap.md"
    previous = roadmap.read_text(encoding="utf-8")
    if marker in previous: previous = previous.split(marker,1)[1].lstrip()
    prefix = f"""Standalone RCORE-00 import checkpoint — 2026-10-13: **{workspace['passed']} workspace Rust +7 native export +8 native import = {total} distinct tests**; 54 workspace suites/15 native tests explicitly ignored in portable run and executed separately. Import batch: **15 authored sources/8 passed cases/0 failed**, actual source namespaces, mixed/explicit definition units, Preserve/Geometry loss policy, ownership/Cancel/budgets and fresh-process .om9 after external source removal. Producer batch: **82 native runs/32 expected rejections**, compatibility inventory68 and current conversion50. [Import evidence](2026-10-13-standalone-import.json), [projection](2026-10-13-standalone-native-source-projection.json).

Current standalone prepared pending batches: **0**; historical application queue below remains separate. Broad openNURBS packages2–8 stay open; all12 complete RCORE-00 gates stay unchecked and total future batches remain unknown. Next: current-state retained merge/remapping/edit/delete/new, then full resource/clipboard/application matrices. These counts do not promote legacy application proofs or complete feature statuses.

{marker}

"""
    roadmap.write_text(prefix+previous,encoding="utf-8")
    progress = ROOT / "docs/validation/2026-10-13-standalone-foundation-progress.md"
    text = progress.read_text(encoding="utf-8")
    title = "## Actual source import, durable definition units and guarded ownership"
    if title in text: text = text.split(title,1)[0].rstrip()+"\n"
    text += f"""
{title}

Latest verification: **{workspace['passed']} workspace Rust tests passed ({workspace['suites']} suites), 7 actual native export tests and 8 actual native import tests: {total} distinct tests exercised**. Fifteen native tests are explicitly ignored only in the portable workspace run and separately executed. Resource54 and its six cold-process cases are subsets, not additional distinct tests. Formatting, warning-strict workspace Clippy and Release build passed on H:. Actual headless30/UI86 module observations still have zero children; the UI framebuffer keeps the supplied Matrix9 layout.

[Actual import receipt](2026-10-13-standalone-import.json) measures **15 source files/8 tests/0 failures**. The SDK fixture produces mm/inch/custom/None/Unset, mixed definition units, unresolved closure, complex preserved fields and parent/material color selectors. Rust captures an immutable source under held read handles, writes a private synced snapshot/resolution packet, checks the independently read native SHA-256, validates strict typed relation/projection coverage and bounded acyclic dependencies, and admits represented geometry with fresh source/namespace identities. Shared members remain local; layers retain decoded order. Default Preserve keeps uncovered records retained-only. Explicit Geometry reports omitted bound object/member/layer fields; parent/material color loss is named. Original bytes and original/resolved model/page/definition contexts survive a real fresh-process .om9 open after the source was deleted.

Target/caller capture freezes coordinator authority, epoch, caller generation/version and immutable document identity/revision. Original cancellation cannot be replaced by another token; the bounded union remains checked through commit. Plain operations reject active caller scopes; scoped imports prepare/publish only through their original caller. Private scene and external receipt storage are admitted before publication. Review reproduced foreign equal-counter coordinator publication, then verified the fix. Three additional export regressions independently reject foreign equal-counter/shared-document origins, an aborted caller roundtrip with unchanged public epoch, and an active caller bypass. Review also reproduced original export cancellation substitution with an actual native file; its refined existing lifecycle case observed RED then GREEN after binding the original token through held-file hashing and rename. Rejections preserve public state, the good destination and executable Undo/Redo.

[Native projection](2026-10-13-standalone-native-source-projection.json) has **82 owned jobs/32 expected rejections**, peak {projection['peak_committed_bytes']:,} committed bytes. Schema3 does not assert host readiness: it reports actual source hash, units/resolution, numeric SDK component types, canonical placements and concrete default/nondefault classifications. Original fields/tolerances remain original metadata. Native definition bbox is its actual cache in native units, not an invented canonical box. Projection cannot serialize opaque table writers without full context; immutable source bytes own those bodies. Compatibility batches freshly pass **inventory68/17 rejection**, peak {inventory['peak_committed_bytes']:,}, and **current conversion50/40 rejection**, peak {conversion_peak:,}. The existing Rust export lifecycle batch remains7/16 owned jobs. Every recorded native operation is exited with observed active_processes=0.

This is a selected synchronous import/library slice. The app import dialog, general native CAD/annotation/page/material/history/resource editing, retained-only scene interaction, full asynchronous/global reservations, current-state retained merge/remapping/export, structural/OS clipboard, exact payload sharing, GPU warm-cache performance and actual Rhino5 interoperability remain required. No complete M01–M12/607-feature/148-capability gate is promoted. Zero standalone prepared batches remain pending at this checkpoint; broader seven openNURBS packages and12 full RCORE gates remain unclosed, with total future batches unknown. Next exact work: current-state source merge with edited/deleted/new records and dependency/material/namespace remapping under owned native staging. The full RCORE-00 goal remains active.
"""
    progress.write_text(text,encoding="utf-8")
    print(json.dumps({"distinct_tests":total,"fixtures":15,"goal_complete":False,"full_gates_open":12}))


if __name__ == "__main__":
    main()
