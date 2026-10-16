"""Synchronize scoped RCORE-00 receipts without promoting feature completion."""
import datetime as dt
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def write(path, data):
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def main():
    workspace = json.loads((ROOT / "docs/validation/2026-10-13-standalone-workspace.json").read_text(encoding="utf-8"))
    native = json.loads((ROOT / "docs/validation/2026-10-13-standalone-exchange.json").read_text(encoding="utf-8"))
    imported = json.loads((ROOT / "docs/validation/2026-10-13-standalone-import.json").read_text(encoding="utf-8"))
    retained = json.loads((ROOT / "docs/validation/2026-10-14-standalone-retained-inputs.json").read_text(encoding="utf-8"))
    retained_current = json.loads((ROOT / "docs/validation/2026-10-14-standalone-retained-current.json").read_text(encoding="utf-8"))
    resources = json.loads((ROOT / "docs/validation/2026-10-13-standalone-resources.json").read_text(encoding="utf-8"))
    projection = json.loads((ROOT / "docs/validation/2026-10-14-native-retained-current-projection.json").read_text(encoding="utf-8"))
    assert projection["runs"] == 82 and projection["rejections"] == 32 and not projection["goal_complete"]
    ui = json.loads((ROOT / "docs/validation/2026-10-13-standalone-ui/ui-receipt.json").read_text(encoding="utf-8"))
    view_menu = json.loads((ROOT / "docs/validation/2026-10-14-viewport-title-menu/ui-receipt.json").read_text(encoding="utf-8"))
    assert view_menu["status"] == "validated_viewport_title_menu_slice" and len(view_menu["checks"]) == 6 and not view_menu["goal_complete"]
    assert workspace["status"] == "validated_workspace_tests"
    workspace_tests = workspace["passed"]
    native_tests = native["rust_lifecycle_tests_passed"]
    import_tests = imported["rust_native_import_tests_passed"]
    retained_tests = retained["rust_tests_passed"]
    retained_current_tests = retained_current["rust_tests_passed"]
    assert retained_current["status"] == "validated_selected_retained_current_cold_publication" and not retained_current["goal_complete"]
    assert native_tests + import_tests + retained_tests + retained_current_tests == workspace["ignored"]
    total_tests = workspace_tests + native_tests + import_tests + retained_tests + retained_current_tests
    progress_path = ROOT / "docs/openmatrix9-progress.json"
    progress = json.loads(progress_path.read_text(encoding="utf-8"))
    progress["updated_at"] = dt.datetime.now(dt.timezone(dt.timedelta(hours=7))).isoformat(timespec="seconds")
    item = next(i for i in progress["items"] if i.get("key") == "rcore00-standalone-foundation")
    item["scope"] = "RCORE-00 owned foundation/schema3 resources/source namespace ledger/archive/scene/session, actual Matrix9 UI, source import and selected current-geometry/retained-current export slices"
    for path in ["standalone/crates/om9-scene", "tools/validate_standalone_ui.py",
                 "docs/validation/2026-10-13-standalone-ui/ui-receipt.json",
                 "standalone/native", "tools/validate_standalone_opennurbs_probe.py",
                 "docs/validation/2026-10-13-standalone-opennurbs-probe.json", "standalone/crates/om9-native-owner",
                 "standalone/crates/om9-exchange", "tools/validate_standalone_exchange.py", "tools/validate_native_exchange_conversion.py",
                 "docs/validation/2026-10-13-standalone-exchange.json", "docs/validation/2026-10-13-standalone-native-conversion.json",
                 "tools/validate_standalone_resources.py", "docs/validation/2026-10-13-standalone-resources.json",
                 "tools/validate_native_source_inventory.py", "docs/validation/2026-10-13-standalone-native-source-inventory.json",
                 "tools/validate_standalone_workspace.py", "docs/validation/2026-10-13-standalone-workspace.json",
                 "tools/validate_standalone_import.py", "docs/validation/2026-10-13-standalone-import.json",
                 "tools/validate_native_source_projection.py", "docs/validation/2026-10-13-standalone-native-source-projection.json"]:
        if path not in item["outputs"]:
            item["outputs"].append(path)
    item["evidence"] = [e for e in item["evidence"] if not e.startswith("Startup current=None;")]
    item["evidence"].append("Startup current=None; four cameras/CPlanes/finite grids; exact native subset/occurrence picks and stale guards; immutable affine buffers/Cancel/one Undo/no-op Redo preservation; shared Line/Point sessions and protected CMD widget event tests; actual native window/framebuffer/event replay receipt.")
    item["evidence"] = [e for e in item["evidence"] if not e.startswith("Matrix9 reference workspace;")]
    item["evidence"].append("Matrix9 reference workspace; compact gray 300-point left sidebar, green pinned/unwrapped CMD with one protected document/editor state, authored SVG icons, exact view order/XZ-YZ CPlanes/positive colored axes, truthful disabled controls and numeric-draft focus restoration. Actual pointer/clipboard/Unicode history/IME ordering regressions pass; full UI/DPI/OS IME and performance acceptance stays open.")
    item["evidence"] = [e for e in item["evidence"] if not e.startswith("Matrix9 S6 viewport title menu;")]
    item["evidence"].append("Matrix9 S6 viewport title menu; exact ordered typed registry, visible unsupported preset/submenu gates, bounded scrolling and target radio state. Maximize/restore and Wireframe bind document/view/layout owners and independent action generation; clone rollback/fresh layout aliases refuse, menu keyboard owns focus, dismissal pointer/Escape preserve pending command and draft. 17 focused tests are a workspace subset; actual native window replay has six checks. Full Shaded/render/DPI/task-modal acceptance remains open.")
    for path in ["standalone/crates/om9-app/src/viewport_menu.rs", "standalone/crates/om9-app/tests/viewport_title_menu.rs", "docs/validation/2026-10-14-viewport-title-menu/ui-receipt.json", "docs/validation/2026-10-14-viewport-title-menu.md"]:
        if path not in item["outputs"]: item["outputs"].append(path)
    item["evidence"] = [e for e in item["evidence"] if not e.startswith("Native owner containment;")]
    item["evidence"].append("Native owner containment; suspended job assignment before entry, committed-memory/output/deadline/Cancel/descendant limits, exclusive nonblocking pipe captures and one bounded teardown clock for process/pipes/observed job accounting. No waiting Drop/reader thread; deterministic actual-pipe/failed-stop/accounting cases and 128 concurrent short jobs pass. Sorted session identity authority survives Undo/caller rollback; fresh UUID owner sessions prevent close/reopen and equal-counter cold aliases; legacy nil format data requires explicit runtime restore.")
    item["evidence"] = [e for e in item["evidence"] if not e.startswith("Owned current-geometry exchange;")]
    item["evidence"].append("Owned current-geometry exchange; immutable coordinator capture/bounded packet/aggregate retention reservation, cold native IDs/types/geometry/layers/definitions/units correspondence, synced hashed staging and revalidated same-volume replacement; real current edits/delete/new, stale+Undo/close, Cancel/readonly/tamper/unsupported/budget rejection. No retained merge/Rhino5/app capability acceptance.")
    item["evidence"] = [e for e in item["evidence"] if not e.startswith("Schema2 embedded resources;")]
    item["evidence"].append("Schema2 embedded resources; immutable ResourceId/hash/length bytes in binary chunks, typed canonical UUID maps, aggregate hydration/restoration admission, resource transactions and retired identities, held-handle Windows save, explicit schema1 migration after runtime preflight. Actual independent cold processes hydrate after original removal, preserve durable IDs and reject counter overflow before replacement; missing modules stay unresolved and persistent option fingerprints stay distinct. Native source correspondence/import/merge and full archive acceptance remain open.")
    item["evidence"]=[e for e in item["evidence"] if not e.startswith("Schema3 source ledger;")]
    item["evidence"].append("Schema3 source ledger; document-owned SourceId and fresh ImportNamespaceId, original resolved model/page units, native-local typed keys/relations and streaming host baselines. Atomic import bundles participate in Undo/caller scopes and retired authority; direct edits/solver Publish reconcile sticky Edited/Deleted slots, definition members stay local, resource/retained transfer guards and cold resurrection/alias rejection. Explicit2-to3 migration preserves bytes/IDs and creates empty provenance. Structural ownership plus separately measured native subset import; full native families and merged preservation remain pending.")
    item["evidence"]=[e for e in item["evidence"] if not e.startswith("Strict native source inventory;")]
    item["evidence"].append("Strict native source inventory; schema2 DTO reads registered tables/keys/relations/original units/userdata descriptors with low-level completeness rejection. Exact geometry projection still requires host attribute/dependency/unit admission; SDK-decoded indices are explicit and original bytes remain authoritative. Independent native phased/custom-domain/mirrored Circle D0-D3 correspondence and exact full-turn/near-full policy verified. Full native family admission, opaque payload remap and merged preservation remain open.")
    item["evidence"] = [e for e in item["evidence"] if not e.startswith("Actual native source import;")]
    item["evidence"].append("Actual native source import; immutable held source snapshots/hash-bound schema3, explicit model/page/definition resolutions and conjugated nested placement, strict native relation/projection closure. Default Preserve retains uncovered records; explicit Geometry diagnoses bound object/member/layer losses including parent/material color selectors. Fresh namespaces, atomic Undo/caller ownership, original cancellation, coordinator authority, private scene and receipt budgets verified; cold .om9 keeps bytes/definition contexts after source removal. Current-state native merge/full families/clipboard/Rhino5/app import remain open.")
    item["evidence"] = [e for e in item["evidence"] if not e.startswith("Original-wire preparation;")]
    item["evidence"].append("Original-wire preparation; version50 schema4 source/record/table/start/end hashes and complete decoded catalog bind immutable Resource bytes to same-held-FILE native owners. Rust keeps serialized/reference/SDK indexes distinct, validates original units and strict Known/Unknown attachment DTOs including copy-count-zero, owns Preserve/Geometry field overlays and dependency-cache invalidation, and retains original Cancel after preparation. Exact allocator regressions cover peer document/Undo/incoming owners and late-escaped JSON value/key growth. Logical output identities/frames still require actual wire allocation, native action/cache serialization and independent cold output proof.")
    item["evidence"] = [e for e in item["evidence"] if not e.startswith("Selected retained-current exchange;")]
    item["evidence"].append("Selected retained-current exchange; Rust allocates all native UUID/wire/reference contexts and parent frames, encodes bounded borrowed schema4 requests, holds embedded source/request/output files, and independently verifies fresh native geometry/attributes/units/raw spans/nested conservative bounds. Current edits/delete/new and original unavailable source cases pass; exact loss acknowledgement and same verified OS handle atomic publication preserve old targets on refusal. Tiny/translated thin cache regressions reject without an absolute tolerance floor. Unsupported opaque remaps, full native families, independent tight BRep bounds, clipboard, global concurrent budgets, Rhino5 and app capability acceptance remain open.")
    for path in ["tools/validate_standalone_retained_inputs.py", "tools/validate_native_retained_source.py", "tools/record_standalone_retained_checkpoint.py", "docs/validation/2026-10-14-standalone-retained-inputs.json", "docs/validation/2026-10-14-standalone-retained-infrastructure.json"]:
        if path not in item["outputs"]:
            item["outputs"].append(path)
    for path in ["tools/validate_standalone_retained_current.py", "tools/validate_native_retained_current.py",
                 "docs/validation/2026-10-14-standalone-retained-current.json",
                 "docs/validation/2026-10-14-native-retained-current-selected.json",
                 "docs/superpowers/plans/2026-10-14-retained-current-writer-abi.md"]:
        if path not in item["outputs"]:
            item["outputs"].append(path)
    for test in item["tests"]:
        if "cargo test" in test["command"]:
            test["result"] = f"{workspace_tests} passed ({workspace['suites']} suites); {native_tests} native export, {import_tests} native import, {retained_tests} original-wire input and {retained_current_tests} retained-current tests explicitly ignored here and executed separately; {workspace['rustc']}; {workspace['cargo']}"
        if "validate_standalone_foundation.py" in test["command"]:
            test["result"] = "4 authored fixture objects cold semantic equal; 30 live loaded modules; zero children/legacy dependencies; headless slice only"
    item["tests"] = [t for t in item["tests"] if "validate_standalone_ui.py" not in t["command"]]
    item["tests"].append({"command": "rtk proxy python tools/validate_standalone_ui.py", "result": f"actual native root window; RTX 3060/Vulkan; literal CMD endpoints/one Undo/Undo/Redo/Esc; rendered 1600x1000 PNG; {len(ui['loaded_modules'])} loaded modules, zero children/legacy host dependencies; initial UI slice only"})
    item["tests"] = [t for t in item["tests"] if "validate_standalone_opennurbs_probe.py" not in t["command"]]
    item["tests"].append({"command": "rtk proxy python tools/validate_standalone_opennurbs_probe.py", "result": "five authored native families/version50/cold equal/corrupt rejection through bounded native owner; actual private-commit peaks and zero active processes after return; probe not integrated into import/export"})
    item["tests"] = [t for t in item["tests"] if not any(s in t["command"] for s in ["validate_standalone_exchange.py", "validate_native_exchange_conversion.py"])]
    item["tests"].append({"command": "rtk proxy python tools/validate_standalone_exchange.py", "result": "7 actual native lifecycle Rust tests passed; 16 owned write/inspect jobs; every current master variant, current edit/delete/new, full-source stale/Cancel/readonly/tamper rejection and zero active native processes"})
    item["tests"].append({"command": "rtk proxy python tools/validate_native_exchange_conversion.py", "result": "50 actual bounded native jobs, 40 rejection cases, 12 primary cold native records, version50, dependencies/CRC/depth/output exclusivity/field matrix verified; no retained merge or Rhino5 runtime acceptance"})
    item["tests"]=[t for t in item["tests"] if "validate_native_source_inventory.py" not in t["command"]]
    item["tests"].append({"command":"rtk proxy python tools/validate_native_source_inventory.py",
                          "result":"68 actual bounded native jobs, 17 rejections; five original unit contexts/registered tables/relations/userdata descriptors, aggregate nested-curve budget/completeness rejection, phased/mapped/mirrored Circle native D0-D3 and near-full retained-only policy; projection DTO only, no host import or merged preservation"})
    item["tests"] = [t for t in item["tests"] if "validate_standalone_resources.py" not in t["command"]]
    item["tests"].append({"command": "rtk proxy python tools/validate_standalone_resources.py", "result": f"{resources['rust_tests_passed']} scoped Rust tests; 6 receipt cases from actual cold processes: embedded payload survives source removal, explicit schema1/schema2 migration preserves durable IDs/payload/unresolved status and empty provenance, overflow preserves original file, persistent option parameter changes fingerprint, schema3 edited/deleted/new ledger and original units survive child reopen; peer/Undo/incoming-owner and JSON parser peak probes are subsets of workspace tests"})
    item["tests"] = [t for t in item["tests"] if not any(s in t["command"] for s in ["validate_standalone_import.py", "validate_native_source_projection.py"])]
    item["tests"].append({"command": "rtk proxy python tools/validate_standalone_import.py", "result": f"{import_tests} actual native import Rust tests; {imported['fixtures']} authored source files; canonical mixed/explicit units, shared nested instances, original source retention, Preserve/Geometry losses, target/caller/Cancel/budget rejection and fresh-process source-deleted .om9 reopen"})
    item["tests"] = [t for t in item["tests"] if t.get("command") != "rtk proxy python tools/validate_standalone_retained_inputs.py"]
    item["tests"].append({"command": "rtk proxy python tools/validate_standalone_retained_inputs.py", "result": f"{retained_tests} original-wire Rust tests; {retained['fixtures']} authored files/{len(retained['source_scenarios'])} source scenarios/{retained['native_runs']} owned jobs; version50 source bytes/units/complete catalog/spans, sparse indexes, copy-count-zero userdata and operation/staging refusal; no merge writer acceptance"})
    item["tests"] = [t for t in item["tests"] if t.get("command") != "rtk proxy python tools/validate_standalone_retained_current.py"]
    item["tests"].append({"command": "rtk proxy python tools/validate_standalone_retained_current.py", "result": f"{retained_current_tests} Rust integration tests; {retained_current['authored_source_files']} SDK source files/{retained_current['successful_private_output_files']} private outputs/{retained_current['semantic_scenarios']} scenarios/{retained_current['native_runs']} owned jobs; independent cold original/current geometry, native identities/frames/units/raw preservation and dirty bounds, {retained_current['published_output_files']} atomic publications and {retained_current['expected_publication_refusals']} old-target-preserving refusals; selected codec scope only"})
    item["tests"].append({"command": "rtk proxy python tools/validate_native_source_projection.py", "result": "82 owned native jobs / 32 rejections; hash-bound schema3 projection and original/per-definition explicit unit context/classification/conjugated placements; producer projection only, no merged export acceptance"})
    item["uncertainties"] = ["M01-M12 remain open for complete scope; full Matrix9/context F6/Layers/Builder/docking/DPI/OS IME/modes/snaps pending.",
                             "Initial painter still projects/tessellates on CPU; GPU geometry caching/warm pointer/frame/commit targets and large-scene spatial picking remain unaccepted.",
                             "Full native import/merged preservation/clipboard/archive resource families/further migrations, general native CAD/periodic evaluation/trimmed/contact/topology families, exact payload sharing/concurrent global reservations and Rhino5 actual open/edit remain pending."]
    item["next_document"] = "OpenMatrix9_Codex_Spec_v2/specs/00-rhino-core/10-persistence-3dm-clipboard.md"
    item["next_action"] = "Expand retained native family/field/remapping matrices, resource families and clipboard; continue full RCORE-00/Matrix9 acceptance. Selected retained-current cold publication is tested; native app commands remain disabled pending capability acceptance."
    write(progress_path, progress)
    package = ROOT / "OpenMatrix9_Codex_Spec_v2"
    ledger_path = package / "MIGRATION_LEDGER.json"
    ledger = json.loads(ledger_path.read_text(encoding="utf-8"))
    foundation = ledger["foundation_implementation"]
    foundation.update(runtime_validation="validated_supported_ui_slice", rust_tests_passed=total_tests,
                      viewport_title_menu_receipt="../docs/validation/2026-10-14-viewport-title-menu/ui-receipt.json",
                      viewport_title_menu_status="validated_scoped_menu_only_full_acceptance_open",
                      rust_workspace_tests_passed=workspace_tests, rust_native_lifecycle_tests_passed=native_tests,
                      rust_native_import_tests_passed=import_tests,
                      rust_native_retained_input_tests_passed=retained_tests,
                      rust_native_retained_current_tests_passed=retained_current_tests,
                      native_retained_current_receipt="../docs/validation/2026-10-14-standalone-retained-current.json",
                      native_retained_current_status="validated_selected_retained_current_cold_publication_not_app_capability",
                      native_retained_input_receipt="../docs/validation/2026-10-14-standalone-retained-inputs.json",
                      retained_infrastructure_receipt="../docs/validation/2026-10-14-standalone-retained-infrastructure.json",
                      native_import_receipt="../docs/validation/2026-10-13-standalone-import.json",
                      native_import_status="validated_native_to_host_import_slice_not_app_capability",
                      native_source_projection_receipt="../docs/validation/2026-10-13-standalone-native-source-projection.json",
                      rust_workspace_receipt="../docs/validation/2026-10-13-standalone-workspace.json",
                      ui_receipt="../docs/validation/2026-10-13-standalone-ui/ui-receipt.json",
                      native_probe_receipt="../docs/validation/2026-10-13-standalone-opennurbs-probe.json",
                      native_probe_status="validated_probe_not_integrated",
                      native_exchange_receipt="../docs/validation/2026-10-13-standalone-exchange.json",
                      native_conversion_receipt="../docs/validation/2026-10-13-standalone-native-conversion.json",
                      native_exchange_status="validated_owned_current_geometry_export_slice_not_app_capability",
                      resource_receipt="../docs/validation/2026-10-13-standalone-resources.json",
                      resource_status="validated_resource_schema3_source_ledger_slice",
                      native_source_inventory_receipt="../docs/validation/2026-10-13-standalone-native-source-inventory.json",
                      native_source_inventory_status="validated_source_inventory_and_circle_correspondence_slice")
    write(ledger_path, ledger)
    baseline = json.loads((package / "SOURCE_READ_AUDIT.json").read_text(encoding="utf-8"))
    for row in baseline["files"]:
        source = ROOT / "OpenMatrix9_Codex_Spec_v1" / row["path"]
        if source.stat().st_size != row["bytes"] or hashlib.sha256(source.read_bytes()).hexdigest() != row["sha256"]:
            raise AssertionError("V1 baseline changed: " + row["path"])
    rows = []
    for path in sorted(package.rglob("*")):
        if path.is_file() and path.name != "MANIFEST.json" and not {"target", "__pycache__"}.intersection(path.parts):
            data = path.read_bytes()
            rows.append({"path": path.relative_to(package).as_posix(), "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()})
    write(package / "MANIFEST.json", rows)
    print(json.dumps({"v1_unchanged": len(baseline["files"]), "v2_manifest": len(rows), "foundation_tests": total_tests, "workspace_tests": workspace_tests, "native_lifecycle_tests": native_tests, "native_import_tests": import_tests, "completion_claim": False}))


if __name__ == "__main__":
    main()
