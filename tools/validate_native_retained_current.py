"""Named actual retained-current adapter batches; all native calls are job-owned.

The private 3DM fixtures are authored SDK files. This does not certify Rhino
interoperability or enable a host capability.
"""
import argparse
import copy
import hashlib
import json
import math
from pathlib import Path
import subprocess
import tempfile

from validate_native_source_inventory import ROOT, PIN, CMAKE, digest, uid

IDENTITY = [[1, 0, 0, 0], [0, 1, 0, 0], [0, 0, 1, 0], [0, 0, 0, 1]]


def build_fixture():
    project = ROOT / "build/native-retained-current-fixture-project"
    project.mkdir(parents=True, exist_ok=True)
    (project / "CMakeLists.txt").write_text(f'''cmake_minimum_required(VERSION 3.25)
project(OM9RetainedCurrentFixture LANGUAGES CXX)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreadedDLL")
add_executable(om9-retained-current-fixture "{ROOT.as_posix()}/native/retained_current_fixture.cpp")
target_include_directories(om9-retained-current-fixture PRIVATE "{ROOT.as_posix()}/build/standalone-deps/opennurbs" "{ROOT.as_posix()}/build/standalone-deps/nlohmann-json-3.11.3")
target_compile_definitions(om9-retained-current-fixture PRIVATE ON_CMAKE_BUILD NOMINMAX)
target_compile_options(om9-retained-current-fixture PRIVATE /W4 /WX /utf-8 /EHsc)
target_link_libraries(om9-retained-current-fixture PRIVATE "{ROOT.as_posix()}/build/standalone-native/opennurbs/Release/opennurbsStatic.lib" Shlwapi Usp10 Bcrypt "{ROOT.as_posix()}/build/standalone-native/opennurbs/zlib/Release/zlib.lib")
target_link_options(om9-retained-current-fixture PRIVATE /SUBSYSTEM:WINDOWS /ENTRY:wmainCRTStartup)
''', encoding="utf-8")
    directory = ROOT / "build/native-retained-current-fixture"
    for args in [["-S", str(project), "-B", str(directory), "-G", "Visual Studio 17 2022", "-A", "x64"],
                 ["--build", str(directory), "--config", "Release"]]:
        subprocess.run(["rtk", "proxy", str(CMAKE), *args], check=True)


def output(kind, ident, index=None, reference=None):
    return {"native_component_type": kind, "id": ident, "serialized_index": index,
            "archive_reference_index": reference, "sdk_destination_index": reference}


def addition(kind, value, index=None, reference=None, refresh=False):
    return {"output": output(kind, value["id"], index, reference),
            "needs_cache_refresh": refresh,
            "current": {{5: "Layer", 11: "Definition", 12: "Entity"}[kind]: value}}


def baseline():
    layer = {"id": uid(1), "name": "Default", "parent": None, "visible": True,
             "selectable": True, "locked": False, "color": [20, 30, 40, 255]}
    point = {"id": uid(10), "label": "Current point", "geometry": {"Point": [25.4, 50.8, 0]},
             "placement": IDENTITY, "attributes": {"layer_index": 0, "visible": True,
             "selectable": True, "locked": False, "color": None}, "definition_member": False}
    return {"schema_version": 4, "writer_policy": "om9-retained-current-v1",
            "metadata_policy": "new_mm", "bounds_policy": "cold_leaf_hull_v1",
            "primary_namespace": None, "model_mm_per_unit": 1,
            "document": {"id": uid(1000), "name": "Authored retained current", "page_unit": "Inch",
                         "current_layer": {"policy": "explicit_output", "id": uid(1),
                                           "archive_reference_index": 0}},
            "sources": [], "additions": [addition(5, layer, 0, 0), addition(12, point)],
            "default_layer": None, "table_order": {"5": [uid(1)], "12": [uid(10)]}}


def primary_request(prepared, namespace=uid(9000)):
    packet = baseline()
    packet.update(metadata_policy="primary_archive", primary_namespace=namespace,
                  model_mm_per_unit=prepared["original_units"]["model"]["millimeters_per_unit"] or 25.4,
                  additions=[], table_order={})
    packet["document"]["current_layer"] = {"policy": "preserve_primary", "id": None,
                                            "archive_reference_index": None}
    records = []
    for original in prepared["records"]:
        key, wire = original["source"], original["wire"]
        kind = key["native_component_type"]
        records.append({"source": key, "expected_wire": wire, "action": "retain",
                        "changed": dict.fromkeys(["geometry", "placement", "attributes", "label"], False),
                        "needs_cache_refresh": False,
                        "output": output(kind, key["id"], wire["serialized_index"], wire["archive_reference_index"]),
                        "definition_mm_per_unit": (original["definition_units"]["declared_mm_per_unit"]
                                                   if kind == 11 else None), "current": None})
        records[-1]["output"]["sdk_destination_index"] = wire["sdk_destination_index"]
        packet["table_order"].setdefault(str(kind), []).append(key["id"])
    packet["sources"] = [{"namespace": namespace, "source_id": uid(9001),
        "sha256": prepared["archive"]["source_sha256"],
        "model_mm_per_unit": packet["model_mm_per_unit"],
        "page_mm_per_unit": prepared["original_units"]["page"]["millimeters_per_unit"] or 25.4,
        "records": records}]
    return packet


def point_payload(ident, label, position, member=False):
    return {"Entity": {"id": ident, "label": label, "geometry": {"Point": position},
            "placement": IDENTITY, "definition_member": member,
            "attributes": {"layer_index": 0, "visible": True, "selectable": True,
                           "locked": False, "color": None}}}


def cache_scale_packet(translated=False):
    packet = baseline()
    member = point_payload(uid(40), "Authored scale-sensitive member", [1e-100, 2e-100, 3e-100], True)["Entity"]
    definition = {"id": uid(30), "label": "Scale-sensitive master", "mm_per_unit": 1, "members": [uid(40)]}
    root = point_payload(uid(18), "Authored scale-sensitive instance", [0, 0, 0])["Entity"]
    root["geometry"] = {"Instance": uid(30)}
    if translated:
        member["geometry"] = {"Curve": {"Line": {"start": [1e9, 0, 0], "end": [1e9 + 1e-3, 0, 0], "domain": [0, 1]}}}
    else:
        root["placement"] = [[1e100, 0, 0, 0], [0, 1e100, 0, 0], [0, 0, 1e100, 0], [0, 0, 0, 1]]
    packet["additions"] = [packet["additions"][0], addition(11, definition, None, 0, True),
                           addition(12, member), addition(12, root, refresh=True)]
    packet["table_order"] = {"5": [uid(1)], "11": [uid(30)], "12": [uid(40), uid(18)]}
    return packet


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--binary", type=Path, default=ROOT / "build/standalone-native/Release/om9-exchange-native.exe")
    p.add_argument("--runner", type=Path, default=ROOT / "target/release/om9-native-runner.exe")
    p.add_argument("--receipt", type=Path, default=ROOT / "build/standalone-native/native-retained-current-validation.json")
    p.add_argument("--fixture", type=Path, default=ROOT / "build/native-source-fixture/Release/om9-source-fixture.exe")
    p.add_argument("--retained-fixture", type=Path, default=ROOT / "build/native-retained-source-fixture/Release/om9-retained-source-fixture.exe")
    p.add_argument("--cache-fixture", type=Path, default=ROOT / "build/native-retained-current-fixture/Release/om9-retained-current-fixture.exe")
    p.add_argument("--build-fixture", action="store_true")
    p.add_argument("--red", action="store_true")
    p.add_argument("--cache-tolerance-red", action="store_true")
    p.add_argument("--cache-tolerance-only", action="store_true")
    args = p.parse_args()
    if args.build_fixture:
        build_fixture()
    runs = []

    def run(*arguments, success=True, program=None):
        program = program or args.binary
        owned = subprocess.run(["rtk", "proxy", str(args.runner), "--program", str(program),
                                "--sha256", digest(program), "--", *map(str, arguments)],
                               capture_output=True, timeout=40)
        result = json.loads(owned.stdout)
        result["program_sha256"] = digest(program)
        result["arguments"] = [a.name if isinstance(a, Path) else str(a) for a in arguments]
        runs.append(result)
        assert result["stats"]["exited"] and result["stats"]["active_processes"] == 0, result
        assert result["stats"]["peak_committed_bytes"] <= 128 * 1024 * 1024, result
        if success:
            assert owned.returncode == 0 and result["status"] == "native_owner_result", result
            data = json.loads(result["stdout"])
            if arguments[0] == "--inspect-retained-current":
                raw = Path(arguments[-1]).read_bytes()
                assert data["output_sha256"] == hashlib.sha256(raw).hexdigest()
                for span in data["raw_spans"]:
                    assert set(span) == {"kind", "native_component_type", "id", "table_type", "record_type", "offset", "byte_length", "sha256"}
                    begin, end = span["offset"], span["offset"] + span["byte_length"]
                    assert 0 <= begin < end <= len(raw)
                    assert hashlib.sha256(raw[begin:end]).hexdigest() == span["sha256"]
            return data
        assert owned.returncode != 0 and result["status"] == "native_owner_rejected", result
        assert not result.get("stdout"), result
        return result

    with tempfile.TemporaryDirectory(prefix="retained-current-", dir=ROOT / "build") as private:
        directory = Path(private)
        request, archive = directory / "request.json", directory / "current.3dm"
        request.write_text(json.dumps(baseline()), encoding="utf-8")
        if args.cache_tolerance_red:
            tiny = cache_scale_packet()
            request.write_text(json.dumps(tiny), encoding="utf-8")
            run("--write-retained", request, archive)
            valid = run("--inspect-retained-current", request, archive)
            assert next(b for b in valid["actual"]["bounds"] if b["id"] == uid(18))["computed"] == [[1, 2, 3], [1, 2, 3]]
            corrupt = directory / "zero-definition-cache.3dm"
            run("tamper-zero-definition", archive, corrupt, program=args.cache_fixture)
            accepted = run("--inspect-retained-current", request, corrupt)
            bound = next(b for b in accepted["actual"]["bounds"] if b["id"] == uid(30))
            assert bound["matched"] and bound["stored"] == [[0, 0, 0], [0, 0, 0]]
            assert bound["computed"] == [[1e-100, 2e-100, 3e-100]] * 2
            (ROOT / "build/native-retained-cache-tolerance-red.json").write_text(json.dumps({"request": tiny,
                "valid": valid, "incorrectly_accepted": accepted, "owned_runs": runs}, indent=2), encoding="utf-8")
            print("RED proved: admitted native1e-100 member with1e100 ancestor accepts zero definition cache")
            return
        if args.red:
            rejection = run("--write-retained", request, archive, success=False)
            assert "unsupported command" in rejection["stderr"]
            assert not archive.exists()
            print("RED observed: actual retained-current operation absent; exclusive output untouched")
            return
        cache_scale_evidence = []
        for name, translated, tamper in [("tiny-amplified", False, "tamper-zero-definition"),
                                          ("thin-translated", True, "tamper-collapse-definition")]:
            packet = cache_scale_packet(translated)
            cache_request, cache_output = directory / f"{name}.json", directory / f"{name}.3dm"
            cache_request.write_text(json.dumps(packet), encoding="utf-8")
            run("--write-retained", cache_request, cache_output)
            valid = run("--inspect-retained-current", cache_request, cache_output)
            bounds = {b["id"]: b for b in valid["actual"]["bounds"]}
            expected = [[1e9, 0, 0], [1e9 + 1e-3, 0, 0]] if translated else [[1, 2, 3]] * 2
            assert bounds[uid(18)]["computed"] == expected
            assert bounds[uid(30)]["computed"] == (expected if translated else [[1e-100, 2e-100, 3e-100]] * 2)
            corrupt = directory / f"{name}-invalid-cache.3dm"
            run(tamper, cache_output, corrupt, program=args.cache_fixture)
            rejected = run("--inspect-retained-current", cache_request, corrupt, success=False)
            assert "cold stored cache differs" in rejected["stderr"]
            cache_scale_evidence.append({"scenario": name, "request": packet, "valid": valid, "rejection": rejected})
        (ROOT / "build/native-retained-cache-tolerance-green.json").write_text(json.dumps({
            "batch": "retained-cache-scale-v1", "total_runs": len(runs),
            "expected_rejections": 2, "successful_writer_output_files": 2,
            "authored_valid_stale_cache_output_files": 2, "semantic_scenarios": 4,
            "binary_sha256": digest(args.binary), "header_sha256": digest(ROOT / "native/retained_merge.hpp"),
            "evidence": cache_scale_evidence, "owned_runs": runs}, indent=2), encoding="utf-8")
        if args.cache_tolerance_only:
            print("PASS native tiny amplified and thin translated cache regressions:8 owned jobs,2 rejections")
            return
        write = run("--write-retained", request, archive)
        assert write["status"] == "written_retained_current" and write["objects"] == 1
        cold = run("--inspect-retained-current", request, archive)
        assert cold["status"] == "validated_retained_current"
        assert cold["request_sha256"] == digest(request) and cold["output_sha256"] == digest(archive)
        assert cold["actual"]["objects"][0]["geometry"] == {"Point": [25.4, 50.8, 0]}
        (ROOT / "build/native-retained-current-schema4-example.json").write_text(json.dumps(cold, indent=2), encoding="utf-8")
        (ROOT / "build/native-retained-current-schema4-request-example.json").write_text(json.dumps(baseline(), indent=2), encoding="utf-8")
        before = digest(archive)
        run("--write-retained", request, archive, success=False)
        assert digest(archive) == before
        for name, change in [
            ("schema", lambda x: x.update(schema_version=3)),
            ("policy", lambda x: x.update(bounds_policy="tight_bbox")),
            ("unknown", lambda x: x.update(extra=True)),
            ("unit", lambda x: x.update(model_mm_per_unit=25.4)),
            ("missing-order", lambda x: x["table_order"].pop("12")),
            ("wrong-ref", lambda x: x["additions"][0]["output"].update(archive_reference_index=1)),
            ("bad-layer", lambda x: x["additions"][1]["current"]["Entity"]["attributes"].update(layer_index=2)),
            ("selectable", lambda x: x["additions"][1]["current"]["Entity"]["attributes"].update(selectable=False)),
        ]:
            invalid = baseline()
            change(invalid)
            request.write_text(json.dumps(invalid), encoding="utf-8")
            absent = directory / f"{name}.3dm"
            run("--write-retained", request, absent, success=False)
            assert not absent.exists(), name
        request.write_text('{"schema_version":4,"schema_version":4}', encoding="utf-8")
        run("--write-retained", request, directory / "duplicate.3dm", success=False)
        request.write_text(json.dumps(baseline()) + " trailing", encoding="utf-8")
        run("--write-retained", request, directory / "trailing.3dm", success=False)
        clean = directory / "clean-mm.3dm"
        run("clean-mm", clean, program=args.fixture)
        resolution = directory / "resolution.json"
        resolution.write_text(json.dumps({"schema_version": 1, "model_mm_per_unit": None,
            "page_mm_per_unit": None, "definition_mm_per_unit": {}}), encoding="utf-8")
        projected = run("--project-source", clean, resolution)
        typed = baseline()
        typed.update(additions=[], table_order={})
        for layer in projected["layers"]:
            value = {k: layer[k] for k in ["id", "name", "parent", "visible", "locked", "color"]}
            value["selectable"] = True
            typed["additions"].append(addition(5, value, layer["native_index"], layer["native_index"]))
            typed["table_order"].setdefault("5", []).append(layer["id"])
        for ordinal, definition in enumerate(projected["definitions"]):
            value = {k: definition[k] for k in ["id", "label", "members"]}
            value["mm_per_unit"] = 1
            typed["additions"].append(addition(11, value, None, ordinal, True))
            typed["table_order"].setdefault("11", []).append(definition["id"])
        for entity in projected["objects"]:
            value = {k: entity[k] for k in ["id", "label", "geometry", "placement"]}
            value["definition_member"] = entity["is_definition_member"]
            a = entity["attributes"]
            value["attributes"] = {"layer_index": a["native_layer_index"], "visible": a["visible"],
                                   "locked": a["locked"], "selectable": True, "color": a["color"]}
            typed["additions"].append(addition(12, value, refresh="Instance" in value["geometry"]))
            typed["table_order"].setdefault("12", []).append(entity["id"])
        request.write_text(json.dumps(typed), encoding="utf-8")
        typed_output = directory / "typed-current.3dm"
        run("--write-retained", request, typed_output)
        typed_report = run("--inspect-retained-current", request, typed_output)
        assert typed_report["objects"] == 15 and typed_report["definitions"] == 3
        assert {r["native_kind"] for r in typed_report["actual"]["objects"]} == {
            "ON_Point", "ON_LineCurve", "ON_ArcCurve", "ON_NurbsCurve", "ON_NurbsSurface",
            "ON_PolyCurve", "ON_Mesh", "ON_PointCloud", "ON_InstanceRef"}
        # Two independent authored sources with repeated local UUIDs. Native
        # receives every identity, frame and omission/regeneration from the plan.
        multi = baseline()
        multi.update(additions=[], sources=[], table_order={})
        for source_number, mode in enumerate(["clean-mm", "clean-inch"]):
            namespace = uid(9100 + source_number)
            actual_source = directory / f"{namespace}.3dm"
            run(mode, actual_source, program=args.fixture)
            preparation = run("--prepare-retained-source", actual_source)
            projection = run("--project-source", actual_source, resolution)
            catalog = primary_request(preparation, namespace)["sources"][0]
            remap = {r["source"]["id"]: uid(int(r["source"]["id"].split("-")[-1], 16) + source_number * 1000)
                     for r in catalog["records"]}
            layer_slots = {r["id"]: source_number + i for i, r in enumerate(projection["layers"])}
            by_current = {}
            for layer in projection["layers"]:
                value = {k: layer[k] for k in ["id", "name", "parent", "visible", "locked", "color"]}
                value.update(id=remap[layer["id"]], name=layer["name"] + str(source_number),
                             parent=remap.get(layer["parent"]), selectable=True)
                by_current[layer["id"]] = {"Layer": value}
            for definition in projection["definitions"]:
                by_current[definition["id"]] = {"Definition": {"id": remap[definition["id"]],
                    "label": definition["label"] + str(source_number), "mm_per_unit": 1,
                    "members": [remap[m] for m in definition["members"]]}}
            for entity in projection["objects"]:
                value = {k: entity[k] for k in ["id", "label", "geometry", "placement"]}
                value.update(id=remap[entity["id"]], definition_member=entity["is_definition_member"])
                if "Instance" in value["geometry"]:
                    value["geometry"] = {"Instance": remap[value["geometry"]["Instance"]]}
                a = entity["attributes"]
                value["attributes"] = {"layer_index": layer_slots[a["layer"]], "visible": a["visible"],
                                       "locked": a["locked"], "selectable": True, "color": a["color"]}
                by_current[entity["id"]] = {"Entity": value}
            ordinals = {5: source_number, 11: source_number * 3}
            for row in catalog["records"]:
                kind, old_id = row["source"]["native_component_type"], row["source"]["id"]
                assert kind in [5, 11, 12]
                ordinal = ordinals.get(kind)
                if kind in ordinals:
                    ordinals[kind] += 1
                row.update(action="regenerate", current=by_current[old_id],
                    output=output(kind, remap[old_id], ordinal if kind == 5 else None, ordinal),
                    definition_mm_per_unit=1 if kind == 11 else None,
                    needs_cache_refresh=kind == 11 or (kind == 12 and "Instance" in by_current[old_id]["Entity"]["geometry"]))
                multi["table_order"].setdefault(str(kind), []).append(remap[old_id])
            multi["sources"].append(catalog)
        request.write_text(json.dumps(multi), encoding="utf-8")
        multi_output = directory / "two-namespaces.3dm"
        run("--write-retained", request, multi_output)
        multi_report = run("--inspect-retained-current", request, multi_output)
        assert multi_report["objects"] == 30 and multi_report["definitions"] == 6 and multi_report["layers"] == 2
        multi_objects = {r["id"]: r for r in multi_report["actual"]["objects"]}
        for ident in [uid(10), uid(1010)]:
            assert all(math.isclose(a, b, abs_tol=1e-10) for a, b in zip(multi_objects[ident]["geometry"]["Point"], [25.4, 50.8, 0]))
        source = directory / "complex.3dm"
        run("mm", source, program=args.fixture)
        variants = [source]
        for mode in ["copycount-zero", "sparse", "light"]:
            variant = directory / f"{mode}.3dm"
            run(mode, source, variant, program=args.retained_fixture)
            variants.append(variant)
        cache_source = directory / "cache-copycount-zero.3dm"
        run("cache-copycount-zero", source, cache_source, program=args.cache_fixture)
        variants.append(cache_source)
        for source in variants:
            prepared = run("--prepare-retained-source", source)
            packet = primary_request(prepared)
            (directory / f'{packet["primary_namespace"]}.3dm').write_bytes(source.read_bytes())
            ignored = copy.deepcopy(packet)
            ignored_row = next(r for r in ignored["sources"][0]["records"] if r["source"]["id"] == uid(10))
            ignored_row["current"] = point_payload(uid(10), "Exact point", [1, 2, 3])
            ignored_row["current"]["Entity"]["geometry"] = {"UnknownGeometry": {"ignored": True}}
            request.write_text(json.dumps(ignored), encoding="utf-8")
            run("--write-retained", request, directory / f"{source.stem}-ignored-payload.3dm", success=False)
            request.write_text(json.dumps(packet), encoding="utf-8")
            kept = directory / f"{source.stem}-kept.3dm"
            run("--write-retained", request, kept)
            report = run("--inspect-retained-current", request, kept)
            assert report["checks"]["raw_records"] == len(prepared["records"])
            assert report["checks"]["raw_tables"] >= 10
            assert next(r for r in report["actual"]["objects"] if r["id"] == uid(17))["geometry"] is None, "native Polycurve segment domains require a parameter map absent from Core geometry"
            assert all(next(r for r in report["actual"]["objects"] if r["id"] == uid(i))["geometry"] is None for i in [25, 26]), "unrepresented Mesh/Cloud native channels remain retained-only"
            edited = copy.deepcopy(packet)
            rows = {r["source"]["id"]: r for r in edited["sources"][0]["records"]}
            row = rows[uid(10)]
            row.update(action="patch_entity", current=point_payload(uid(10), "Exact point", [101, 202, 303]))
            row["changed"]["geometry"] = True
            rows[uid(11)].update(action="delete", output=None, current=None)
            edited["table_order"]["12"].remove(uid(11))
            new = point_payload(uid(90), "Added C", [7, 8, 9])["Entity"]
            edited["additions"] = [addition(12, new)]
            edited["table_order"]["12"].append(uid(90))
            request.write_text(json.dumps(edited), encoding="utf-8")
            changed = directory / f"{source.stem}-changed.3dm"
            run("--write-retained", request, changed)
            report = run("--inspect-retained-current", request, changed)
            actual = {r["id"]: r for r in report["actual"]["objects"]}
            assert uid(11) not in actual and actual[uid(10)]["geometry"] == {"Point": [101, 202, 303]}
            assert actual[uid(90)]["geometry"] == {"Point": [7, 8, 9]}
            if source.stem == "complex":
                mesh_edit = copy.deepcopy(packet)
                mesh_row = next(r for r in mesh_edit["sources"][0]["records"] if r["source"]["id"] == uid(15))
                mesh_current = point_payload(uid(15), "Exact mesh", [0, 0, 0])
                mesh_current["Entity"]["geometry"] = {"Mesh": {"vertices": [[0.1, 0.2, 0.3], [1.1, 0.2, 0.3], [1.1, 2.2, 0.3], [0.1, 2.2, 0.3]],
                    "faces": [[0, 1, 2, 3]], "normals": [], "colors": []}}
                mesh_row.update(action="patch_entity", current=mesh_current)
                mesh_row["changed"]["geometry"] = True
                request.write_text(json.dumps(mesh_edit), encoding="utf-8")
                decimal_mesh = directory / "decimal-mesh.3dm"
                run("--write-retained", request, decimal_mesh)
                mesh_report = run("--inspect-retained-current", request, decimal_mesh)
                assert next(r for r in mesh_report["actual"]["objects"] if r["id"] == uid(15))["geometry"] == mesh_current["Entity"]["geometry"]
                appended = copy.deepcopy(packet)
                new_layer = {"id": uid(91), "name": "Appended layer Ω", "parent": uid(1),
                    "visible": True, "selectable": True, "locked": False, "color": [7, 8, 9, 200]}
                new_member = point_payload(uid(93), "New member", [3, 4, 5], True)["Entity"]
                new_member["attributes"]["layer_index"] = 2
                new_definition = {"id": uid(92), "label": "Appended native definition", "mm_per_unit": 1, "members": [uid(93)]}
                new_root = point_payload(uid(94), "New mirrored instance", [0, 0, 0])["Entity"]
                new_root["geometry"] = {"Instance": uid(92)}
                new_root["placement"] = [[-2, 0, 0, 10], [0, 3, 0, 2], [0, 0, 4, 1], [0, 0, 0, 1]]
                new_root["attributes"]["layer_index"] = 2
                appended["additions"] = [addition(5, new_layer, 2, 2), addition(11, new_definition, None, 4, True),
                    addition(12, new_member), addition(12, new_root, refresh=True)]
                for kind, ids in {"5": [uid(91)], "11": [uid(92)], "12": [uid(93), uid(94)]}.items():
                    appended["table_order"][kind].extend(ids)
                request.write_text(json.dumps(appended), encoding="utf-8")
                appended_output = directory / "appended-native-components.3dm"
                run("--write-retained", request, appended_output)
                appended_report = run("--inspect-retained-current", request, appended_output)
                assert next(b for b in appended_report["actual"]["bounds"] if b["id"] == uid(94))["computed"] == [[4, 14, 21], [4, 14, 21]]
                assert next(d for d in appended_report["actual"]["definitions"] if d["id"] == uid(92))["wire"]["archive_reference_index"] == 4
                # Explicitly closed unsupported and original-identity gates.
                for name, mutate in [
                    ("wrong-wire", lambda x: x["sources"][0]["records"][0]["expected_wire"].update(sha256="0" * 64)),
                    ("missing-record", lambda x: x["sources"][0]["records"].pop()),
                    ("source-hash", lambda x: x["sources"][0].update(sha256="0" * 64)),
                    ("page-context", lambda x: x["sources"][0].update(page_mm_per_unit=1)),
                    ("layer-delete", lambda x: next(r for r in x["sources"][0]["records"] if r["source"]["id"] == uid(2)).update(action="delete", output=None)),
                    ("opaque-geometry", lambda x: next(r for r in x["sources"][0]["records"] if r["source"]["id"] == uid(22)).update(action="patch_entity", current=point_payload(uid(22), "Retained opaque attachment", [99, 99, 99]), changed={"geometry": True, "placement": False, "attributes": False, "label": False})),
                ]:
                    invalid = copy.deepcopy(packet)
                    mutate(invalid)
                    request.write_text(json.dumps(invalid), encoding="utf-8")
                    refused = directory / f"{name}.3dm"
                    run("--write-retained", request, refused, success=False)
                    assert not refused.exists()
            if source.stem == "copycount-zero":
                oracle = run("oracle-copycount-zero", changed, changed, program=args.retained_fixture)
                assert oracle["dimstyle_copy_count"] == oracle["attribute_copy_count"] == 0
            if source.stem == "sparse":
                child = next(r for r in report["actual"]["layers"] if r["id"] == uid(2))
                assert child["wire"] == {"serialized_index": 7, "archive_reference_index": 1, "sdk_destination_index": 1}
                assert actual[uid(10)]["attributes"]["groups"] == [5]
            # Definition member edit requires every ancestor/Instance cache.
            cached = copy.deepcopy(packet)
            rows = {r["source"]["id"]: r for r in cached["sources"][0]["records"]}
            rows[uid(40)].update(action="patch_entity", current=point_payload(uid(40), "Definition point", [101, 202, 303], True))
            rows[uid(40)]["changed"]["geometry"] = True
            for ident in [30, 31, 41, 18, 19]:
                rows[uid(ident)]["needs_cache_refresh"] = True
            request.write_text(json.dumps(cached), encoding="utf-8")
            cached_output = directory / f"{source.stem}-cached.3dm"
            run("--write-retained", request, cached_output)
            report = run("--inspect-retained-current", request, cached_output)
            boxes = {r["id"]: r["computed"] for r in report["actual"]["bounds"]}
            assert boxes[uid(30)] == [[0, 0, 0], [101, 202, 303]]
            assert boxes[uid(31)] == [[0, 5, 0], [101, 207, 303]]
            assert boxes[uid(18)] == [[-192, 15, 0], [10, 621, 1212]]
            assert all(r["matched"] for r in report["actual"]["bounds"])
            tampered = directory / f"{source.stem}-stale-cache.3dm"
            run("tamper-cache", cached_output, tampered, program=args.cache_fixture)
            run("--inspect-retained-current", request, tampered, success=False)
            if source.stem == "cache-copycount-zero":
                oracle = run("oracle", cached_output, cached_output, program=args.cache_fixture)
                assert all(r["copy_count"] == 0 for r in oracle["records"])
                assert {r["id"]: r["stored_bbox"] for r in oracle["records"]}[uid(18)] == [[-192, 15, 0], [10, 621, 1212]]
            if source.stem == "complex":
                (ROOT / "build/native-retained-current-complex-cache-example.json").write_text(json.dumps(report, indent=2), encoding="utf-8")
                empty = copy.deepcopy(packet)
                for row in empty["sources"][0]["records"]:
                    if row["source"]["native_component_type"] in [11, 12]:
                        row.update(action="delete", output=None, current=None, definition_mm_per_unit=None)
                empty["table_order"].pop("11")
                empty["table_order"].pop("12")
                request.write_text(json.dumps(empty), encoding="utf-8")
                smaller = directory / "empty-object-table.3dm"
                run("--write-retained", request, smaller)
                empty_report = run("--inspect-retained-current", request, smaller)
                assert empty_report["objects"] == empty_report["definitions"] == 0
                assert smaller.stat().st_size < source.stat().st_size
                request.write_text(json.dumps(cached), encoding="utf-8")
            cached["sources"][0]["records"][next(i for i, r in enumerate(cached["sources"][0]["records"]) if r["source"]["id"] == uid(18))]["needs_cache_refresh"] = False
            request.write_text(json.dumps(cached), encoding="utf-8")
            omitted = directory / f"{source.stem}-missing-cache.3dm"
            run("--write-retained", request, omitted, success=False)
            assert not omitted.exists()
    assert all(r["program_sha256"] == digest(args.binary) for r in runs if r["arguments"][0].startswith("--"))
    dependencies = [ROOT / "build/standalone-deps/nlohmann-json-3.11.3/json.hpp",
        ROOT / "build/standalone-deps/nlohmann-json-3.11.3/LICENSE.MIT",
        ROOT / "build/standalone-native/opennurbs/Release/opennurbsStatic.lib",
        ROOT / "build/standalone-native/opennurbs/zlib/Release/zlib.lib"]
    assert digest(dependencies[0]) == "9bea4c8066ef4a1c206b2be5a36302f8926f7fdc6087af5d20b417d0cf103ea6"
    assert digest(dependencies[1]) == "86b998c792894ccb911a1cb7994f7a9652894e7a094c0b5e45be2f553f45cf14"
    args.receipt.write_text(json.dumps({"batch": "retained-current-selected-v1", "sdk_commit": PIN,
        "total_runs": len(runs), "expected_rejections": sum(r["status"] == "native_owner_rejected" for r in runs),
        "sdk_authored_source_files": 8, "successful_writer_output_files": 23,
        "authored_valid_stale_cache_output_files": 7, "semantic_scenarios": 57,
        "count_definition": "Distinct source/output paths and semantic cases; setup, deterministic repeats and independent oracle calls are not extra semantic scenarios",
        "binary_sha256": digest(args.binary), "source_sha256": digest(ROOT / "native/native_exchange.cpp"),
        "header_sha256": digest(ROOT / "native/retained_merge.hpp"),
        "fixture_source_sha256": digest(ROOT / "native/retained_current_fixture.cpp"),
        "fixture_binary_sha256": digest(args.cache_fixture), "validator_sha256": digest(Path(__file__)),
        "runner_sha256": digest(args.runner), "dependencies": {str(p.relative_to(ROOT)): digest(p) for p in dependencies},
        "scope": "Selected strict current serializers, primary raw bytes, typed secondary regeneration, cache leaf hull; no Rhino interoperability or arbitrary opaque semantic certification", "runs": runs}, indent=2), encoding="utf-8")
    print(f"PASS retained-current-selected-v1: {len(runs)} owned executions")


if __name__ == "__main__":
    main()
