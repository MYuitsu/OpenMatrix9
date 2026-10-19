"""Actual schema4 retained-source preparation; every native run is job-owned.

This validates original wire identity/byte ownership, not retained merge or Rhino
interoperability. Authored 3DM files and outputs stay on H: in private build data.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import tempfile

from validate_native_source_inventory import ROOT, CMAKE, PIN, digest, uid

TYPES = {name: n for n, name in enumerate([
    "Image", "TextureMapping", "RenderMaterial", "LinePattern", "Layer", "Group",
    "TextStyle", "DimStyle", "RenderLight", "HatchPattern", "InstanceDefinition",
    "ModelGeometry", "HistoryRecord"], 1)}
FLAGS = {"master_userdata_present", "attribute_userdata_present", "unknown_userdata_present",
         "unrepresented_record_chunks_present", "opaque_identity_remap_requires_admission",
         "opaque_length_remap_requires_admission", "selected_serializer_context_required"}


def build_retained_fixture():
    project = ROOT / "build/native-retained-source-fixture-project"
    project.mkdir(parents=True, exist_ok=True)
    (project / "CMakeLists.txt").write_text(f'''cmake_minimum_required(VERSION 3.25)
project(OM9RetainedSourceFixture LANGUAGES CXX)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreadedDLL")
add_executable(om9-retained-source-fixture "{ROOT.as_posix()}/native/retained_source_fixture.cpp")
target_include_directories(om9-retained-source-fixture PRIVATE "{ROOT.as_posix()}/build/standalone-deps/opennurbs" "{ROOT.as_posix()}/build/standalone-deps/nlohmann-json-3.11.3")
target_compile_definitions(om9-retained-source-fixture PRIVATE ON_CMAKE_BUILD NOMINMAX)
target_compile_options(om9-retained-source-fixture PRIVATE /W4 /WX /utf-8 /EHsc)
target_link_libraries(om9-retained-source-fixture PRIVATE "{ROOT.as_posix()}/build/standalone-native/opennurbs/Release/opennurbsStatic.lib" Shlwapi Usp10 Bcrypt "{ROOT.as_posix()}/build/standalone-native/opennurbs/zlib/Release/zlib.lib")
target_link_options(om9-retained-source-fixture PRIVATE /SUBSYSTEM:WINDOWS /ENTRY:wmainCRTStartup)
''', encoding="utf-8")
    out = ROOT / "build/native-retained-source-fixture"
    for args in [["-S", str(project), "-B", str(out), "-G", "Visual Studio 17 2022", "-A", "x64"],
                 ["--build", str(out), "--config", "Release", "--target", "om9-retained-source-fixture"]]:
        subprocess.run(["rtk", "proxy", str(CMAKE), *args], check=True)


def top_spans(raw):
    assert raw[:24] == b"3D Geometry File Format "
    position = 32
    result = []
    while position < len(raw):
        begin = position
        code, length = struct.unpack_from("<IQ", raw, position)
        assert not code & 0x80000000
        position += 12 + length
        assert position <= len(raw)
        result.append({"table_type": code, "offset": begin, "byte_length": position-begin,
                       "sha256": hashlib.sha256(raw[begin:position]).hexdigest(),
                       "anonymous_user_table": code == 0x10000017})
    assert position == len(raw) and result[0]["table_type"] == 1
    # Read3dmStartSection owns the file header and full comment chunk together.
    end = result.pop(0)["offset"] + 12 + struct.unpack_from("<Q", raw, 36)[0]
    return [{"table_type": 0, "offset": 0, "byte_length": end,
             "sha256": hashlib.sha256(raw[:end]).hexdigest(),
             "anonymous_user_table": False}, *result]


def verify(data, archive, decoded):
    assert set(data) == {"schema_version", "status", "decoder_policy", "archive", "original_units",
                         "records", "top_level_spans", "admission_required", "host_ready"}
    assert data["schema_version"] == 4 and data["status"] == "retained_source_prepared"
    assert data["decoder_policy"] == "om9-retained-source-v1"
    assert data["admission_required"] is True and data["host_ready"] is False
    assert set(data["archive"]) == {"version", "opennurbs_version", "file_bytes", "endmark_bytes", "source_sha256"}
    assert data["archive"]["source_sha256"] == digest(archive)
    assert data["archive"]["file_bytes"] == data["archive"]["endmark_bytes"] == archive.stat().st_size
    assert data["archive"]["version"] == 50
    assert data["original_units"] == decoded["units"]
    raw = archive.read_bytes()
    assert data["top_level_spans"] == top_spans(raw)
    keys = {(TYPES[r.get("component_type", "ModelGeometry")], r["id"], r["native_index"])
            for r in [*decoded["components"], *decoded["objects"]]}
    assert len(data["records"]) == len(keys)
    actual = set()
    for record in data["records"]:
        assert set(record) == {"source", "wire", "native_kind", "native_class_uuid", "name", "userdata",
                               "attribute_userdata", "preservation_flags", "definition_units"}
        key = record["source"]
        assert set(key) == {"native_component_type", "id", "native_index"}
        actual.add((key["native_component_type"], key["id"], key["native_index"]))
        wire = record["wire"]
        assert set(wire) == {"id", "serialized_index", "archive_reference_index", "sdk_destination_index",
                             "table_type", "record_type", "offset", "byte_length", "sha256"}
        assert wire["id"] == key["id"] and wire["byte_length"] > 12
        begin, end = wire["offset"], wire["offset"] + wire["byte_length"]
        assert hashlib.sha256(raw[begin:end]).hexdigest() == wire["sha256"]
        assert struct.unpack_from("<I", raw, begin)[0] == wire["record_type"]
        assert any(span["table_type"] == wire["table_type"] and span["offset"] < begin < end <= span["offset"]+span["byte_length"]
                   for span in data["top_level_spans"])
        flags = record["preservation_flags"]
        assert set(flags) == FLAGS and all(isinstance(value, bool) for value in flags.values())
        assert flags["master_userdata_present"] == bool(record["userdata"])
        assert flags["attribute_userdata_present"] == bool(record["attribute_userdata"])
        assert flags["unknown_userdata_present"] == any(u["unknown"] for u in [*record["userdata"], *record["attribute_userdata"]])
        if record["userdata"] or record["attribute_userdata"]:
            assert flags["opaque_identity_remap_requires_admission"] and flags["opaque_length_remap_requires_admission"]
        for u in [*record["userdata"], *record["attribute_userdata"]]:
            required = {"class_id", "application_id", "item_id", "unknown", "copy_count", "native_sizeof_bytes", "opaque_references"}
            if u["unknown"]:
                required |= {"byte_length", "payload_sha1", "archive_version", "opennurbs_version"}
            assert set(u) == required
            assert u["opaque_references"] is True
        if key["native_component_type"] == 11:
            assert set(record["definition_units"]) == {"native_enum", "name", "declared_mm_per_unit"}
        else:
            assert record["definition_units"] is None
        if key["native_component_type"] == 12:
            assert key["native_index"] is None and wire["serialized_index"] is None
    assert actual == keys
    return {r["source"]["id"]: r for r in data["records"]}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--build-fixture", action="store_true")
    parser.add_argument("--binary", type=Path, default=ROOT/"build/standalone-native/Release/om9-exchange-native.exe")
    parser.add_argument("--fixture-binary", type=Path, default=ROOT/"build/native-source-fixture/Release/om9-source-fixture.exe")
    parser.add_argument("--retained-fixture-binary", type=Path, default=ROOT/"build/native-retained-source-fixture/Release/om9-retained-source-fixture.exe")
    parser.add_argument("--runner", type=Path, default=ROOT/"target/release/om9-native-runner.exe")
    parser.add_argument("--receipt", type=Path, default=ROOT/"build/standalone-native/native-retained-source-preparation-validation.json")
    args = parser.parse_args()
    actual_pin = subprocess.check_output(["rtk", "proxy", "git", "-C", str(ROOT/"build/standalone-deps/opennurbs"), "rev-parse", "HEAD"], text=True).strip()
    assert actual_pin == PIN, actual_pin
    dependencies = [ROOT/"build/standalone-deps/nlohmann-json-3.11.3/json.hpp",
                    ROOT/"build/standalone-deps/nlohmann-json-3.11.3/LICENSE.MIT",
                    ROOT/"build/standalone-native/opennurbs/Release/opennurbsStatic.lib",
                    ROOT/"build/standalone-native/opennurbs/zlib/Release/zlib.lib"]
    assert digest(dependencies[0]) == "9bea4c8066ef4a1c206b2be5a36302f8926f7fdc6087af5d20b417d0cf103ea6"
    assert digest(dependencies[1]) == "86b998c792894ccb911a1cb7994f7a9652894e7a094c0b5e45be2f553f45cf14"
    if args.build_fixture:
        build_retained_fixture()
    runs, cases = [], {}

    def run(program, arguments, success=True):
        p = subprocess.run(["rtk", "proxy", str(args.runner), "--program", str(program), "--sha256", digest(program), "--", *map(str, arguments)], capture_output=True, timeout=40)
        r = json.loads(p.stdout)
        r["program_sha256"] = digest(program)
        r["arguments"] = [a.name if isinstance(a, Path) else str(a) for a in arguments]
        r["stdout_sha256"] = hashlib.sha256(r.get("stdout", "").encode()).hexdigest()
        runs.append(r)
        assert r["stats"]["exited"] and r["stats"]["active_processes"] == 0, r
        assert r["stats"]["peak_committed_bytes"] <= 128*1024*1024, r
        if success:
            assert p.returncode == 0 and r["status"] == "native_owner_result" and not r["stderr"], r
            assert r["limits"]["native_committed_bytes"] == 128*1024*1024
            return json.loads(r["stdout"])
        assert p.returncode != 0 and r["status"] == "native_owner_rejected" and not r.get("stdout"), r
        return r

    private = ROOT/"build/retained-source-validation-private"
    private.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="prepared-", dir=private) as directory:
        directory = Path(directory)
        source = directory/"complex.3dm"
        run(args.fixture_binary, ["mm", source])
        decoded = run(args.binary, ["--inventory-source", source])
        data = run(args.binary, ["--prepare-retained-source", source])
        by_id = verify(data, source, decoded)
        assert by_id[uid(6)]["native_kind"] == "ON_V5x_DimStyle"
        assert by_id[uid(20)]["preservation_flags"]["selected_serializer_context_required"]
        assert by_id[uid(22)]["userdata"][0]["unknown"]
        # Nondefault render/group fields and compound child geometry must not
        # be classified as reference/length-free just because top userdata=[]
        assert by_id[uid(10)]["preservation_flags"]["opaque_identity_remap_requires_admission"]
        assert by_id[uid(17)]["preservation_flags"]["selected_serializer_context_required"]
        assert data == run(args.binary, ["--prepare-retained-source", source]), "cold preparation must be stable"
        cases["complex-mm"] = data
        # Saved in H: scratch for strict Rust DTO integration, without byte echo.
        sample = ROOT/"build/native-retained-source-schema4-example.json"
        sample.write_text(json.dumps(data, indent=2), encoding="utf-8")
        for mode in ["clean-mm", "clean-inch", "clean-custom", "clean-unitless", "clean-unset", "clean-mixed", "clean-unresolved"]:
            path = directory/f"{mode}.3dm"
            run(args.fixture_binary, [mode, path])
            original = run(args.binary, ["--inventory-source", path])
            result = run(args.binary, ["--prepare-retained-source", path])
            rows = verify(result, path, original)
            assert len([r for r in rows.values() if r["source"]["native_component_type"] == 12]) == (17 if mode == "clean-unresolved" else 15)
            assert not any(r["userdata"] or r["attribute_userdata"] for r in rows.values())
            cases[mode] = result
        held_oracle = run(args.retained_fixture_binary, ["oracle-held-source", source, directory/"clean-inch.3dm"])
        assert held_oracle == decoded, "strict scan must use held primary bytes, not alternate path metadata"
        for mode in ["copycount-zero", "sparse", "sparse-bad", "light"]:
            path = directory/f"{mode}.3dm"
            run(args.retained_fixture_binary, [mode, source, path])
            if mode == "sparse-bad":
                run(args.binary, ["--prepare-retained-source", path], False)
                continue
            original = run(args.binary, ["--inventory-source", path])
            result = run(args.binary, ["--prepare-retained-source", path])
            rows = verify(result, path, original)
            if mode == "sparse":
                assert [rows[uid(2)]["wire"][k] for k in ["serialized_index", "archive_reference_index", "sdk_destination_index"]] == [7,1,1]
                assert rows[uid(2)]["source"]["native_index"] == 1
                assert [rows[uid(5)]["wire"][k] for k in ["serialized_index", "archive_reference_index", "sdk_destination_index"]] == [5,5,0]
                assert rows[uid(5)]["source"]["native_index"] == 0
            elif mode == "light":
                light = rows[uid(51)]
                assert light["source"] == {"native_component_type":9, "id":uid(51), "native_index":None}
                assert light["wire"]["serialized_index"] == light["wire"]["sdk_destination_index"] == 7, light
                assert light["wire"]["archive_reference_index"] == 0
            else:
                assert next(r for r in original["components"] if r["id"] == uid(6))["userdata"] == []
                for id_, field in [(uid(6), "userdata"), (uid(10), "attribute_userdata")]:
                    u = rows[id_][field][0]
                    assert u["copy_count"] == 0 and u["unknown"] and u["byte_length"] > 0
                oracle = run(args.retained_fixture_binary, ["oracle-copycount-zero", path, directory/"unused"])
                assert oracle["dimstyle_copy_count"] == oracle["attribute_copy_count"] == 0
                assert oracle["dimstyle_payload"] == oracle["attribute_payload"] == "OM9 authored original copycount-zero attachment"
            cases[mode] = result
            (ROOT/f"build/native-retained-source-schema4-{mode}-example.json").write_text(json.dumps(result, indent=2), encoding="utf-8")
        for mode in ["unknown-class", "unknown-table", "record-budget", "polycurve-point-budget", "duplicate-id", "missing-dependency", "definition-cycle", "invalid-tolerance"]:
            path = directory/f"bad-{mode}.3dm"
            run(args.fixture_binary, [mode, path])
            run(args.binary, ["--prepare-retained-source", path], False)
        raw = source.read_bytes()
        for name, payload in [("empty", b""), ("truncated", raw[:-7]), ("no-endmark", raw[:-20]),
                              ("trailing", raw+b"x"), ("invalid-header", b"x"+raw[1:]),
                              ("stale-endmark", raw[:-8]+struct.pack("<Q", len(raw)+1))]:
            path = directory/f"{name}.3dm"
            path.write_bytes(payload)
            run(args.binary, ["--prepare-retained-source", path], False)
        font = next(span for span in top_spans(raw) if span["table_type"] == 0x10000019)
        damaged = bytearray(raw)
        damaged[font["offset"]+font["byte_length"]-13] ^= 1
        path = directory/"bad-font-crc.3dm"
        path.write_bytes(damaged)
        run(args.binary, ["--prepare-retained-source", path], False)
        run(args.binary, ["--prepare-retained-source", directory/"missing.3dm"], False)
        run(args.binary, ["--prepare-retained-source", directory], False)
        run(args.binary, ["--prepare-retained-source", source, "extra"], False)
        oversized = directory/"oversized.3dm"
        with oversized.open("wb") as f:
            f.truncate(64*1024*1024+1)
        run(args.binary, ["--prepare-retained-source", oversized], False)
    receipt = {"schema_version":1, "scope":"actual_native_retained_source_preparation", "goal_complete":False,
               "opennurbs_commit":PIN, "binary_sha256":digest(args.binary), "fixture_binary_sha256":digest(args.fixture_binary),
               "retained_fixture_binary_sha256":digest(args.retained_fixture_binary), "runner_sha256":digest(args.runner),
               "source_sha256":digest(ROOT/"native/native_exchange.cpp"),
               "header_sha256":digest(ROOT/"native/retained_merge.hpp"),
               "retained_fixture_source_sha256":digest(ROOT/"native/retained_source_fixture.cpp"),
               "dependencies":[{"path":str(path.relative_to(ROOT)), "sha256":digest(path)} for path in dependencies],
               "owner_limits":next(r["limits"] for r in runs if r["status"] == "native_owner_result"),
               "producer_limits":{"archive_bytes":64*1024*1024, "aggregate_records":4096, "aggregate_points":65536,
                                  "json_nodes":262144, "depth":32, "stdout_bytes":8*1024*1024},
               "validator_sha256":digest(Path(__file__)), "runs":len(runs),
               "rejections":sum(r["status"] == "native_owner_rejected" for r in runs),
               "peak_committed_bytes":max(r["stats"]["peak_committed_bytes"] for r in runs),
               "cases":cases, "runs_detail":[{k:v for k,v in r.items() if k != "stdout"} for r in runs],
               "limitations":["No current-state merge operation or host-ready admission", "No multiple-source opaque remap or physical frame policy admission",
                              "Dependency-dirty bounds codec and independent oracle remain required", "No Rhino UI interoperability or native memory-safety claim"]}
    args.receipt.parent.mkdir(parents=True, exist_ok=True)
    args.receipt.write_text(json.dumps(receipt, indent=2), encoding="utf-8")
    print(json.dumps({k:v for k,v in receipt.items() if k not in {"cases", "runs_detail"}}))


if __name__ == "__main__":
    main()
