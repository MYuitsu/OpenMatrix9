"""Authored real 3DM source inventory checks; every native run uses the owner.

Fixtures live in a private temporary directory. This is inventory evidence,
not merged export, application admission, or Rhino interoperability acceptance.
"""
import argparse
import hashlib
import json
import math
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
CMAKE = Path("C:/Program Files (x86)/Microsoft Visual Studio/2022/BuildTools/Common7/IDE/CommonExtensions/Microsoft/CMake/CMake/bin/cmake.exe")
PIN = "eb92af3ba1806b0a34a99aba0d3bda83e3d46083"


def uid(n):
    return f"20000000-0000-4000-8000-{n:012x}"


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def circle_packet(circle, placement=None):
    identity = [[1,0,0,0],[0,1,0,0],[0,0,1,0],[0,0,0,1]]
    record = {"id":uid(29),"label":"Authored phase circle","generation":1,
              "geometry_revision":1,"attributes_revision":1,
              "geometry":{"Curve":{"Circle":circle}},"placement":placement or identity,
              "attributes":{"layer":None,"visible":True,"selectable":True,"locked":False,"color":None}}
    return {"schema_version":1,"mode":"geometry_only_current","document":{
        "id":uid(100),"name":"Authored circle parameter mapping","page_unit":"Inch",
        "current_layer":None,"layer_order":[],"layers":{},"entities":{uid(29):record},"definitions":{}}}


def verify_circle_oracle(oracle, circle):
    """Independent analytic angular-chain derivative check against native D0-D3."""
    assert oracle["status"] == "native_circle_oracle"
    assert oracle["domain"] == circle["domain"]
    angles = circle.get("angle_domain") or circle["domain"]
    assert oracle["angle_domain"] == angles
    assert oracle["native_is_circle"] is True
    rate = (angles[1]-angles[0])/(circle["domain"][1]-circle["domain"][0])
    assert [s["u"] for s in oracle["samples"]] == [circle["domain"][0],sum(circle["domain"])/2,circle["domain"][1]]
    for sample in oracle["samples"]:
        angle = angles[0]+(sample["u"]-circle["domain"][0])*rate
        c, s = math.cos(angle), math.sin(angle)
        coefficients = [(c,s),(-s,c),(-c,-s),(s,-c)]
        expected = [[(circle["center"][i] if d == 0 else 0) + circle["radius"]*rate**d*(cx*circle["x"][i]+cy*circle["y"][i])
                     for i in range(3)] for d,(cx,cy) in enumerate(coefficients)]
        for actual, derived in zip(sample["derivatives"],expected):
            assert all(math.isclose(a,e,rel_tol=2e-13,abs_tol=2e-13) for a,e in zip(actual,derived)), (sample,expected)


def build_fixture():
    actual = subprocess.check_output(["rtk", "proxy", "git", "-C", str(ROOT / "build/standalone-deps/opennurbs"), "rev-parse", "HEAD"], text=True).strip()
    assert actual == PIN, actual
    project = ROOT / "build/native-source-fixture-project"
    project.mkdir(parents=True, exist_ok=True)
    code = f'''cmake_minimum_required(VERSION 3.25)
project(OM9AuthoredSourceFixture LANGUAGES CXX)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreadedDLL")
add_executable(om9-source-fixture "{ROOT.as_posix()}/native/source_inventory_fixture.cpp")
target_include_directories(om9-source-fixture PRIVATE "{ROOT.as_posix()}/build/standalone-deps/opennurbs")
target_compile_definitions(om9-source-fixture PRIVATE ON_CMAKE_BUILD NOMINMAX)
target_compile_options(om9-source-fixture PRIVATE /W4 /WX /utf-8 /EHsc)
target_link_libraries(om9-source-fixture PRIVATE "{ROOT.as_posix()}/build/standalone-native/opennurbs/Release/opennurbsStatic.lib" Shlwapi Usp10 "{ROOT.as_posix()}/build/standalone-native/opennurbs/zlib/Release/zlib.lib")
target_link_options(om9-source-fixture PRIVATE /SUBSYSTEM:WINDOWS /ENTRY:wmainCRTStartup)
'''
    (project / "CMakeLists.txt").write_text(code, encoding="utf-8")
    out = ROOT / "build/native-source-fixture"
    for args in [["-S", str(project), "-B", str(out), "-G", "Visual Studio 17 2022", "-A", "x64"],
                 ["--build", str(out), "--config", "Release", "--target", "om9-source-fixture"]]:
        subprocess.run(["rtk", "proxy", str(CMAKE), *args], check=True)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--build-fixture", action="store_true")
    parser.add_argument("--binary", type=Path, default=ROOT / "build/standalone-native/Release/om9-exchange-native.exe")
    parser.add_argument("--fixture-binary", type=Path, default=ROOT / "build/native-source-fixture/Release/om9-source-fixture.exe")
    parser.add_argument("--runner", type=Path, default=ROOT / "target/release/om9-native-runner.exe")
    parser.add_argument("--receipt", type=Path, default=ROOT / "build/standalone-native/native-source-inventory-validation.json")
    args = parser.parse_args()
    if args.build_fixture:
        build_fixture()
    binary = args.binary.resolve(strict=True)
    fixture_binary = args.fixture_binary.resolve(strict=True)
    runner = args.runner.resolve(strict=True)
    runs = []

    def run(program, arguments, success=True):
        p = subprocess.run(["rtk", "proxy", str(runner), "--program", str(program), "--sha256", digest(program), "--", *map(str, arguments)], capture_output=True, timeout=40)
        report = json.loads(p.stdout)
        assert report["stats"]["exited"] and report["stats"]["active_processes"] == 0, report
        report["program_sha256"] = digest(program)
        report["arguments"] = [Path(str(a)).name if isinstance(a, Path) else str(a) for a in arguments]
        if "stdout" in report:
            report["stdout_sha256"] = hashlib.sha256(report["stdout"].encode("utf-8")).hexdigest()
        runs.append(report)
        if success:
            assert p.returncode == 0 and report["status"] == "native_owner_result", report
            return json.loads(report["stdout"])
        assert p.returncode != 0 and report["status"] == "native_owner_rejected", report
        assert not report.get("stdout"), report
        return report

    inventories, circle_oracles = {}, {}
    with tempfile.TemporaryDirectory(prefix="om9-native-source-") as tmp:
        tmp = Path(tmp)
        # Authored expectations precede production mapping. Canonical output
        # remains field-compatible; shifted radians and custom native domains
        # must be checked with actual native derivatives, not serialized fields.
        base_circle = {"center":[2,3,4],"x":[0,1,0],"y":[0,0,1],"radius":6,"domain":[0,math.tau]}
        for name, circle, placement in [
            ("canonical",base_circle,None),
            ("canonical-null",dict(base_circle,angle_domain=None),None),
            ("shifted-radians",dict(base_circle,domain=[1,1+math.tau]),None),
            ("phase-custom-domain",dict(base_circle,domain=[11,17],angle_domain=[1,1+math.tau]),None),
            ("identical-explicit",dict(base_circle,domain=[1,1+math.tau],angle_domain=[1,1+math.tau]),None),
            ("mapped-mirrored",dict(base_circle,domain=[11,17],angle_domain=[1,1+math.tau]),[[0,-2,0,10],[-2,0,0,20],[0,0,2,30],[0,0,0,1]]),
        ]:
            packet, output = tmp / f"{name}.json", tmp / f"{name}.3dm"
            packet.write_text(json.dumps(circle_packet(circle,placement)),encoding="utf-8")
            written = run(binary,["--write",packet,output])
            assert written["objects"] == 1
            inspected = run(binary,["--inspect",output])
            actual = inspected["objects"][0]["geometry"]["Curve"]["Circle"]
            assert actual["domain"] == circle["domain"]
            angles = circle.get("angle_domain") or circle["domain"]
            if angles == circle["domain"]:
                assert "angle_domain" not in actual
            else:
                assert actual["angle_domain"] == angles
            oracle = run(fixture_binary,["--oracle-circle",output,"29"])
            expected = dict(circle,center=[4,16,38],x=[-1,0,0],y=[0,0,1],radius=12) if placement else circle
            verify_circle_oracle(oracle,expected)
            circle_oracles[name] = oracle
        for name, circle in [
            ("near-full",dict(base_circle,angle_domain=[0,math.tau-1e-13])),
            ("partial",dict(base_circle,angle_domain=[0,math.pi])),
            ("legacy-non-full",dict(base_circle,domain=[0,8])),
            ("unknown-angle-field",dict(base_circle,angle_interval=[0,math.tau])),
        ]:
            packet, output = tmp / f"{name}.json", tmp / f"{name}.3dm"
            packet.write_text(json.dumps(circle_packet(circle)),encoding="utf-8")
            rejected = run(binary,["--write",packet,output],False)
            assert not output.exists(), rejected
            assert "angular span" in rejected["stderr"] if name != "unknown-angle-field" else "fields" in rejected["stderr"]
        for mode, scale in [("mm", 1), ("inch", 25.4), ("custom", 2.5), ("unitless", None), ("unset", None)]:
            path = tmp / f"{mode}.3dm"
            run(fixture_binary, [mode, path])
            data = run(binary, ["--inventory-source", path])
            assert data["schema_version"] == 2 and data["status"] == "source_inventoried"
            assert data["complete_read"] is True
            assert data["archive"]["version"] == 50
            assert data["archive"]["file_bytes"] == data["archive"]["endmark_bytes"] == path.stat().st_size
            assert len(data["objects"]) == 24
            records = {o["id"]: o for o in data["objects"]}
            assert set(records) == {uid(i) for i in [10,11,12,13,14,15,16,17,18,19,20,21,22,24,25,26,27,29,34,40,41,42,43,44]}
            assert sum(o["is_definition_member"] for o in records.values()) == 5
            assert data["admission_required"] is True and all(o["admission_required"] for o in records.values())
            assert data["units"]["model"]["absolute_tolerance"] == 0.002
            assert data["units"]["model"]["angle_tolerance_radians"] == 0.02
            assert data["units"]["model"]["relative_tolerance"] == 0.005
            assert data["units"]["model"]["requires_resolution"] == (scale is None)
            if scale is not None:
                assert math.isclose(data["units"]["model"]["millimeters_per_unit"], scale)
                assert records[uid(10)]["capability"] == "geometry_projection_exact"
                assert records[uid(10)]["geometry"]["Point"] == [scale, 2*scale, 3*scale]
                assert records[uid(12)]["geometry"]["Curve"]["Circle"]["angle_domain"] == [0,math.tau]
                phased = records[uid(29)]["geometry"]["Curve"]["Circle"]
                assert phased["domain"] == [11,17] and phased["angle_domain"] == [1,1+math.tau]
                assert phased["center"] == [2*scale,3*scale,4*scale] and phased["radius"] == 6*scale
                curve = records[uid(13)]["geometry"]["Curve"]["Nurbs"]
                assert curve["knots"] == [2,2,2,4,4,4] and curve["domain"] == [2,4]
                assert math.isclose(curve["weights"][1], math.sqrt(.5))
                assert curve["poles"][1] == [10*scale, 10*scale, 0]
                surface = records[uid(14)]["geometry"]["Surface"]
                assert surface["size"] == [2,3]
                assert surface["poles"] == [[0,0,0],[0,scale,scale],[0,2*scale,2*scale],[5*scale,0,scale],[5*scale,scale,2*scale],[5*scale,2*scale,3*scale]]
                assert surface["weights"] == [1,2,3,2,3,4]
                assert records[uid(18)]["geometry"] == {"Instance":uid(31)}
                assert records[uid(18)]["placement"][0] == [-2,0,0,10*scale]
                assert records[uid(27)]["capability"] == "geometry_projection_exact"
                assert records[uid(27)]["dependency_readiness"]["retained_only_members"] == []
                assert records[uid(27)]["dependency_readiness"]["unit_policy_definitions"] == []
            else:
                assert all(o["capability"] == "retained_only" and o["geometry"] is None for o in records.values())
            for n in [20,21,22,24,25,26,34,42]:
                assert records[uid(n)]["capability"] == "retained_only" and records[uid(n)]["geometry"] is None
            assert records[uid(20)]["native_kind"] == "ON_Brep"
            assert records[uid(21)]["native_kind"] == "ON_ArcCurve"
            if scale is not None:
                assert any("angular span" in d for d in records[uid(34)]["diagnostics"])
            assert records[uid(22)]["userdata"][0]["unknown"] is True
            assert records[uid(26)]["channels"]["scalar_values"] == 2
            assert records[uid(25)]["channels"]["texture_coordinates"] == 4
            layers = {l["id"]: l for l in data["layers"]}
            assert layers[uid(2)]["parent"] == uid(1)
            defs = {d["id"]: d for d in data["definitions"]}
            assert defs[uid(30)]["members"] == [uid(40),uid(42)] and defs[uid(31)]["members"] == [uid(41)]
            if scale is not None:
                assert defs[uid(30)]["dependency_readiness"]["retained_only_members"] == [uid(42)]
            assert uid(42) in defs[uid(31)]["dependency_readiness"]["retained_only_members"]
            assert defs[uid(33)]["dependency_readiness"]["unit_policy_definitions"] == [uid(33)]
            assert all(d["dependency_readiness"]["host_ready"] is False for d in defs.values())
            assert all("absolute_tolerance" not in d["units"] for d in defs.values())
            assert "groups" in records[uid(10)]["unrepresented_attributes"] and "native_material_index" in records[uid(10)]["unrepresented_attributes"]
            assert any(r["role"] == "dimension_style" and r["id"] == uid(6) for r in records[uid(24)]["relations"])
            styles = [c for c in data["components"] if c["component_type"] == "DimStyle"]
            assert len(styles) == 1 and styles[0]["id"] == uid(6)
            assert styles[0]["name"] == "Source nondefault dimension style"
            assert styles[0]["settings"]["text_height"] == 2.75 and styles[0]["settings"]["length_resolution"] == 5
            dim_receipt = next(t for t in data["table_receipts"] if t["table"] == "dimension_styles")
            assert dim_receipt["read_records"] == dim_receipt["retained_records"] == 1 and dim_receipt["derived_records"] == 0
            assert any(r["role"] == "instance_definition" and r["id"] == uid(30) for r in records[uid(41)]["relations"])
            assert any(r["role"] == "material" and r["id"] == uid(3) for r in records[uid(10)]["relations"])
            assert any(r["role"] == "group" and r["id"] == uid(5) for r in records[uid(10)]["relations"])
            user_tables = {t["application_id"]: t for t in data["user_tables"]}
            assert uid(1003) in user_tables, (data["archive"],data["user_tables"])
            assert user_tables[uid(1003)]["byte_length"] == 8
            assert user_tables[uid(1003)]["payload_sha1"] == hashlib.sha1(b"OM9 goo!").hexdigest()
            assert all(t["crc_errors"] == t["recoverable_errors"] == t["critical_errors"] == 0 for t in data["table_receipts"])
            cold = run(binary, ["--inventory-source", path])
            assert cold == data, "cold inventory must preserve stable source identity and actual records"
            oracle = run(fixture_binary,["--oracle-circle",path,"29"])
            verify_circle_oracle(oracle,dict(base_circle,domain=[11,17],angle_domain=[1,1+math.tau]))
            circle_oracles[f"source-{mode}"] = oracle
            near = run(fixture_binary,["--oracle-circle",path,"34"])
            assert near["native_is_circle"] is True and near["angle_domain"][1]-near["angle_domain"][0] != math.tau
            circle_oracles[f"source-{mode}-near-full"] = near
            inventories[mode] = data
            if mode == "mm":
                base = path.read_bytes()
        for mode in ["unknown-class", "unknown-table", "record-budget", "polycurve-point-budget", "duplicate-id", "missing-dependency", "definition-cycle", "invalid-tolerance"]:
            path = tmp / f"{mode}.3dm"
            run(fixture_binary, [mode, path])
            rejected = run(binary, ["--inventory-source", path], False)
            if mode == "polycurve-point-budget":
                assert "point budget" in rejected["stderr"] and rejected["reason"] == "NativeExit(1)", rejected
                assert rejected["stats"]["peak_committed_bytes"] < 128 * 1024 * 1024
        for name, bytes_ in [("truncated",base[:-30]),("trailing",base+b"unowned trailing data"),("corrupt",base[:len(base)//2]+bytes([base[len(base)//2]^0x40])+base[len(base)//2+1:]),("not-3dm",b"not a native archive")]:
            path = tmp / f"{name}.3dm"
            path.write_bytes(bytes_)
            run(binary, ["--inventory-source", path], False)
        oversized = tmp / "oversize.3dm"
        with oversized.open("wb") as file:
            file.seek(64 * 1024 * 1024)
            file.write(b"x")
        rejected = run(binary, ["--inventory-source", oversized], False)
        assert "file-byte budget" in rejected["stderr"], rejected
    receipt = {"schema_version":1,"scope":"actual_native_source_inventory_only",
               "status":"validated_source_inventory_and_circle_correspondence_slice","goal_complete":False,
               "opennurbs_commit":PIN,"runner_sha256":digest(runner),
               "owner_sources":[{"path":p.relative_to(ROOT).as_posix(),"sha256":digest(p)}
                                for p in sorted((ROOT / "crates/om9-native-owner").rglob("*")) if p.is_file()],
               "binary_sha256":digest(binary),"fixture_binary_sha256":digest(fixture_binary),
               "source_sha256":digest(ROOT / "native/native_exchange.cpp"),
               "header_sha256":digest(ROOT / "native/retained_merge.hpp"),
               "fixture_source_sha256":digest(ROOT / "native/source_inventory_fixture.cpp"),
               "validator_sha256":digest(Path(__file__)),"runs":len(runs),"rejections":sum(r["status"]=="native_owner_rejected" for r in runs),
               "peak_committed_bytes":max(r["stats"]["peak_committed_bytes"] for r in runs),
               "operation_receipts":[{k:v for k,v in r.items() if k!="stdout"} for r in runs],"inventories":inventories,
               "circle_oracles":circle_oracles,
               "limitations":["Geometry projection is not host-ready/editable native import admission.",
                              "SDK-decoded indices are explicit; immutable original source bytes remain authoritative.",
                              "Unrepresented attributes, opaque payload remaps and dependency/unit admission remain required.",
                              "No merged preservation export, application 3DM command or Rhino 5 actual open/edit acceptance.",
                              "No native memory-safety or concurrent global reservation certification."]}
    args.receipt.parent.mkdir(parents=True, exist_ok=True)
    args.receipt.write_text(json.dumps(receipt,indent=2,ensure_ascii=False),encoding="utf-8")
    print(json.dumps({k:v for k,v in receipt.items() if k not in {"inventories","operation_receipts","circle_oracles"}}))


if __name__ == "__main__":
    main()
