"""Cold native geometry conversion checks, always inside the owned native job.

Fixtures are authored here and stored only in a temporary private directory.
This evidence does not certify retained merge, native memory safety, or Rhino UI.
"""
import argparse
import copy
import hashlib
import json
import math
from pathlib import Path
import subprocess
import tempfile
from validate_standalone_foundation import pe_imports

ROOT = Path(__file__).resolve().parents[1]
IDENTITY = [[1, 0, 0, 0], [0, 1, 0, 0], [0, 0, 1, 0], [0, 0, 0, 1]]


def uid(n):
    return f"10000000-0000-4000-8000-{n:012x}"


def entity(n, label, geometry, placement=None, layer=1):
    return {"id": uid(n), "label": label, "generation": 1,
            "geometry_revision": 1, "attributes_revision": 1,
            "geometry": geometry, "placement": placement or IDENTITY,
            "attributes": {"layer": uid(layer), "visible": True,
                           "selectable": True, "locked": False,
                           "color": [11, 22, 33, 200]}}


def fixture():
    w = math.sqrt(.5)
    arc = {"degree": 2, "poles": [[10, 0, 0], [10, 10, 0], [0, 10, 0]],
           "weights": [1, w, 1], "knots": [2, 2, 2, 4, 4, 4],
           "domain": [2, 4], "periodic": False, "reversed": False, "seam": None}
    surface = {"degree": [1, 1], "size": [2, 3],
               "poles": [[0, 0, 0], [0, 1, 1], [0, 2, 0],
                         [5, 0, 2], [5, 1, 3], [5, 2, 2]],
               "weights": [1, 2, 1, 1, 3, 1],
               "knots": [[0, 0, 2, 2], [3, 3, 4, 5, 5]],
               "domain": [[0, 2], [3, 5]], "periodic": [False, False], "reversed": False}
    line = {"Line": {"start": [0, 0, 0], "end": [3, 4, 0], "domain": [7, 9]}}
    geometry = [
        (10, "Điểm", {"Point": [1, 2, 3]}, [[1, 0, 0, 10], [0, 1, 0, 20], [0, 0, 1, 30], [0, 0, 0, 1]]),
        (11, "Line", {"Curve": line}, None),
        (12, "Circle", {"Curve": {"Circle": {"center": [2, 3, 4], "x": [1, 0, 0], "y": [0, 1, 0], "radius": 6, "domain": [0, 8], "angle_domain": [0, math.tau]}}}, None),
        (13, "RationalArc", {"Curve": {"Nurbs": arc}}, None),
        (14, "Surface", {"Surface": surface}, None),
        (15, "Mesh", {"Mesh": {"vertices": [[0, 0, 0], [2, 0, 0], [2, 2, 0], [0, 2, 0]], "faces": [[0, 1, 2], [0, 1, 2, 3]], "normals": [[0, 0, 1]] * 4, "colors": [[50, 100, 150, 210]] * 4}}, None),
        (16, "Cloud", {"Cloud": {"points": [[2, 3, 4], [5, 6, 7]], "normals": [[0, 0, 1], [0, 1, 0]], "colors": [[255, 0, 0, 255], [0, 255, 0, 128]], "hidden": []}}, None),
        (17, "Polycurve", {"Curve": {"Polycurve": {"segments": [{"Line": {"start": [0, 0, 0], "end": [1, 0, 0], "domain": [0, 1]}}, {"Line": {"start": [1, 0, 0], "end": [1, 1, 0], "domain": [1, 2]}}], "reversed": False}}}, None),
        (18, "Nested", {"Instance": uid(31)}, [[-2, 0, 0, 10], [0, 3, 0, 20], [0, 0, 4, 30], [0, 0, 0, 1]]),
        (19, "Shared", {"Instance": uid(30)}, IDENTITY),
    ]
    entities = {uid(n): entity(n, label, geom, matrix) for n, label, geom, matrix in geometry}
    entities[uid(11)]["attributes"]["locked"] = True
    entities[uid(12)]["attributes"]["visible"] = False
    entities[uid(12)]["attributes"]["color"] = None
    layers = {uid(1): {"id": uid(1), "name": "Parent", "parent": None, "visible": True, "selectable": True, "locked": False, "color": [100, 110, 120, 200], "revision": 1},
              uid(2): {"id": uid(2), "name": "Con", "parent": uid(1), "visible": False, "selectable": True, "locked": True, "color": [200, 210, 220, 255], "revision": 1}}
    definitions = {uid(30): {"id": uid(30), "label": "Master", "revision": 1,
                             "members": [entity(40, "Member", {"Point": [1, 2, 3]}, [[1, 0, 0, 1], [0, 1, 0, 0], [0, 0, 1, 0], [0, 0, 0, 1]])]},
                   uid(31): {"id": uid(31), "label": "NestedMaster", "revision": 1,
                             "members": [entity(41, "NestedMember", {"Instance": uid(30)}, [[1, 0, 0, 0], [0, 1, 0, 5], [0, 0, 1, 0], [0, 0, 0, 1]])]}}
    return {"schema_version": 1, "mode": "geometry_only_current",
            "document": {"id": uid(100), "name": "Authored conversion", "page_unit": "Inch",
                         "current_layer": uid(1), "layer_order": [uid(1), uid(2)],
                         "layers": layers, "entities": entities, "definitions": definitions}}


def close(actual, expected):
    if isinstance(expected, list):
        assert len(actual) == len(expected), (actual, expected)
        for a, e in zip(actual, expected):
            close(a, e)
    elif isinstance(expected, (float, int)):
        assert math.isclose(actual, expected, rel_tol=1e-7, abs_tol=1e-10), (actual, expected)
    else:
        assert actual == expected, (actual, expected)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, default=ROOT / "build/standalone-native/Release/om9-exchange-native.exe")
    parser.add_argument("--runner", type=Path, default=ROOT / "target/release/om9-native-runner.exe")
    parser.add_argument("--receipt", type=Path, default=ROOT / "build/standalone-native/native-conversion-validation.json")
    args = parser.parse_args()
    binary = args.binary.resolve(strict=True)
    runner = args.runner.resolve(strict=True)
    binary_hash = hashlib.sha256(binary.read_bytes()).hexdigest()
    runs = []

    def run(arguments, success=True):
        result = subprocess.run([str(runner), "--program", str(binary), "--sha256", binary_hash,
                                 "--", *map(str, arguments)], capture_output=True, timeout=40)
        report = json.loads(result.stdout)
        assert report["stats"]["exited"] and report["stats"]["active_processes"] == 0, report
        runs.append(report)
        if success:
            assert result.returncode == 0 and report["status"] == "native_owner_result", report
            return json.loads(report["stdout"])
        assert result.returncode != 0 and report["status"] == "native_owner_rejected", report
        assert not report.get("stdout"), report
        return report

    with tempfile.TemporaryDirectory(prefix="om9-native-conversion-") as tmp:
        packet = fixture()
        path = Path(tmp) / "packet.json"
        model = Path(tmp) / "private.3dm"
        path.write_text(json.dumps(packet, ensure_ascii=False), encoding="utf-8")
        written = run(["--write", path, model])
        assert written["status"] == "written" and written["objects"] == 12
        data = run(["--inspect", model])
        assert data["status"] == "inspected" and data["archive_version"] == 50
        assert data["model_unit"] == "Millimeter" and data["page_unit"] == "Inch"
        objects = {o["id"]: o for o in data["objects"]}
        assert len(objects) == 12
        assert sum(o["is_definition_member"] for o in objects.values()) == 2
        close(objects[uid(10)]["geometry"]["Point"], [11, 22, 33])
        assert objects[uid(10)]["label"] == "Điểm"
        assert objects[uid(11)]["native_kind"] == "ON_LineCurve"
        assert objects[uid(12)]["native_kind"] == "ON_ArcCurve"
        assert objects[uid(13)]["native_kind"] == "ON_NurbsCurve"
        assert objects[uid(14)]["native_kind"] == "ON_NurbsSurface"
        assert objects[uid(17)]["native_kind"] == "ON_PolyCurve"
        for n in [11, 12, 13, 14, 15, 16, 17]:
            close_geometry = packet["document"]["entities"][uid(n)]["geometry"]
            def compare(a, e):
                if isinstance(e, dict):
                    for key, value in e.items():
                        compare(a[key], value)
                else:
                    close(a, e)
            compare(objects[uid(n)]["geometry"], close_geometry)
        assert objects[uid(11)]["attributes"]["locked"] and objects[uid(11)]["attributes"]["visible"]
        assert not objects[uid(12)]["attributes"]["visible"]
        close(objects[uid(10)]["attributes"]["color"], [11, 22, 33, 200])
        assert objects[uid(12)]["attributes"]["color_source"] == "layer"
        for n in [18, 19, 41]:
            source = (packet["document"]["entities"][uid(n)] if n < 40 else packet["document"]["definitions"][uid(31)]["members"][0])
            assert objects[uid(n)]["geometry"] == source["geometry"]
            close(objects[uid(n)]["placement"], source["placement"])
        close(objects[uid(40)]["geometry"]["Point"], [2, 2, 3])
        layers = {l["id"]: l for l in data["layers"]}
        for n in [1, 2]:
            for field in ["name", "parent", "visible", "locked", "color"]:
                close(layers[uid(n)][field], packet["document"]["layers"][uid(n)][field])
        assert data["current_layer"] == uid(1)
        defs = {d["id"]: d for d in data["definitions"]}
        assert defs[uid(30)]["members"] == [uid(40)] and defs[uid(31)]["members"] == [uid(41)]
        # Admission must work from current state after edit/delete, independent
        # of any preceding archive. These private files have no retained records.
        changed = copy.deepcopy(packet)
        changed["document"]["entities"][uid(10)]["geometry"] = {"Point": [4, 5, 6]}
        del changed["document"]["entities"][uid(11)]
        path.write_text(json.dumps(changed), encoding="utf-8")
        edited_model = Path(tmp) / "edited.3dm"
        run(["--write", path, edited_model])
        edited = {o["id"]: o for o in run(["--inspect", edited_model])["objects"]}
        assert uid(11) not in edited and len(edited) == 11
        close(edited[uid(10)]["geometry"]["Point"], [14, 25, 36])
        transformed = copy.deepcopy(packet)
        rotation = [[0, -3, 0, 10], [2, 0, 0, 20], [0, 0, 4, 30], [0, 0, 0, 1]]
        transformed["document"]["entities"][uid(16)]["placement"] = rotation
        transformed["document"]["entities"][uid(15)]["placement"] = rotation
        transformed["document"]["entities"][uid(12)]["placement"] = [[-2, 0, 0, 10], [0, 2, 0, 20], [0, 0, 2, 30], [0, 0, 0, 1]]
        transformed["document"]["entities"][uid(13)]["geometry"]["Curve"]["Nurbs"] = {
            "degree": 2, "poles": [[0, 0, 0], [1, 0, 0], [0, 1, 0], [0, 0, 0], [1, 0, 0]],
            "weights": [1] * 5, "knots": [-1, -1, 0, 1, 2, 3, 4, 4],
            "domain": [0, 3], "periodic": True, "reversed": False, "seam": None}
        periodic_rows = [(0, 0, 1), (1, 0, 2), (0, 1, 3), (0, 0, 1), (1, 0, 2)]
        transformed["document"]["entities"][uid(14)]["geometry"]["Surface"] = {
            "degree": [2, 1], "size": [5, 3],
            "poles": [[x, y, z] for x, y, _ in periodic_rows for z in [0, 1, 2]],
            "weights": [w for _, _, w in periodic_rows for _ in range(3)],
            "knots": [[-1, -1, 0, 1, 2, 3, 4, 4], [3, 3, 4, 5, 5]],
            "domain": [[0, 3], [3, 5]], "periodic": [True, False], "reversed": False}
        path.write_text(json.dumps(transformed), encoding="utf-8")
        transformed_model = Path(tmp) / "transformed.3dm"
        run(["--write", path, transformed_model])
        tx = {o["id"]: o for o in run(["--inspect", transformed_model])["objects"]}
        close(tx[uid(16)]["geometry"]["Cloud"]["points"], [[1, 24, 46], [-8, 30, 58]])
        close(tx[uid(16)]["geometry"]["Cloud"]["normals"], [[0, 0, 1], [-1, 0, 0]])
        close(tx[uid(15)]["geometry"]["Mesh"]["normals"], [[0, 0, 1]] * 4)
        close(tx[uid(12)]["geometry"]["Curve"]["Circle"]["center"], [6, 26, 38])
        close(tx[uid(12)]["geometry"]["Curve"]["Circle"]["x"], [-1, 0, 0])
        close(tx[uid(12)]["geometry"]["Curve"]["Circle"]["radius"], 12)
        assert tx[uid(13)]["geometry"]["Curve"]["Nurbs"]["periodic"]
        assert tx[uid(14)]["geometry"]["Surface"]["periodic"] == [True, False]
        close(tx[uid(14)]["geometry"]["Surface"]["poles"], transformed["document"]["entities"][uid(14)]["geometry"]["Surface"]["poles"])
        close(tx[uid(14)]["geometry"]["Surface"]["weights"], transformed["document"]["entities"][uid(14)]["geometry"]["Surface"]["weights"])
        mirrored = copy.deepcopy(packet)
        mirrored["document"]["entities"][uid(15)]["placement"] = [[-2, 0, 0, 10], [0, 3, 0, 20], [0, 0, 4, 30], [0, 0, 0, 1]]
        mirrored["document"]["entities"][uid(15)]["geometry"]["Mesh"]["normals"] = [[0, 0, 2]] * 4
        mirrored["document"]["entities"][uid(15)]["geometry"]["Mesh"]["vertices"][0][0] = 1.234567890123
        path.write_text(json.dumps(mirrored), encoding="utf-8")
        mirrored_model = Path(tmp) / "mirrored.3dm"
        run(["--write", path, mirrored_model])
        mirrored_data = {o["id"]: o for o in run(["--inspect", mirrored_model])["objects"]}
        close(mirrored_data[uid(15)]["geometry"]["Mesh"]["normals"], [[0, 0, 1]] * 4)
        assert abs(mirrored_data[uid(15)]["geometry"]["Mesh"]["vertices"][0][0] - (10 - 2 * 1.234567890123)) < 1e-12
        assert mirrored_data[uid(15)]["geometry"]["Mesh"]["faces"] == packet["document"]["entities"][uid(15)]["geometry"]["Mesh"]["faces"]
        unlayered = copy.deepcopy(packet)
        unlayered["document"].update(layers={}, layer_order=[], current_layer=None)
        for e in unlayered["document"]["entities"].values():
            e["attributes"]["layer"] = None
        for definition in unlayered["document"]["definitions"].values():
            for e in definition["members"]:
                e["attributes"]["layer"] = None
        path.write_text(json.dumps(unlayered), encoding="utf-8")
        default_model = Path(tmp) / "default.3dm"
        default_write = run(["--write", path, default_model])
        default_data = run(["--inspect", default_model])
        assert default_write["diagnostics"]
        assert len(default_data["layers"]) == 1
        assert default_data["current_layer"] == "ffffffff-ffff-4fff-8fff-ffffffffffff"
        assert all(o["attributes"]["layer"] == default_data["current_layer"] for o in default_data["objects"])
        original = model.read_bytes()
        rejection_cases = []

        def reject(name, value=None, raw=None):
            path.write_text(raw if raw is not None else json.dumps(value), encoding="utf-8")
            destination = Path(tmp) / f"bad-{name}.3dm"
            run(["--write", path, destination], False)
            assert not destination.exists(), name
            assert model.read_bytes() == original
            rejection_cases.append(name)

        reject("duplicate-key", raw='{"schema_version":1,"schema_version":1}')
        reject("nested-duplicate-key", raw=json.dumps(packet).replace('"radius": 6', '"radius": 6, "radius": 7'))
        reject("trailing", raw=json.dumps(packet) + " {}")
        reject("depth", raw='[' * 100 + '0' + ']' * 100)
        for name, mutate in [
            ("unknown", lambda p: p.update(extra=1)),
            ("schema", lambda p: p.update(schema_version=2)),
            ("mode", lambda p: p.update(mode="retained_merge")),
            ("selectable", lambda p: p["document"]["entities"][uid(10)]["attributes"].update(selectable=False)),
            ("hidden-locked", lambda p: p["document"]["entities"][uid(11)]["attributes"].update(visible=False)),
            ("hidden", lambda p: p["document"]["entities"][uid(16)]["geometry"]["Cloud"].update(hidden=[False, True])),
            ("mesh-index", lambda p: p["document"]["entities"][uid(15)]["geometry"]["Mesh"].update(faces=[[0, 1, 99]])),
            ("seam", lambda p: p["document"]["entities"][uid(13)]["geometry"]["Curve"]["Nurbs"].update(seam=3)),
            ("periodic", lambda p: p["document"]["entities"][uid(13)]["geometry"]["Curve"]["Nurbs"].update(periodic=True)),
            ("reversed", lambda p: p["document"]["entities"][uid(13)]["geometry"]["Curve"]["Nurbs"].update(reversed=True)),
            ("missing-definition", lambda p: p["document"]["entities"][uid(19)].update(geometry={"Instance": uid(90)})),
            ("cycle", lambda p: p["document"]["definitions"][uid(30)]["members"][0].update(geometry={"Instance": uid(31)})),
            ("layer-cycle", lambda p: p["document"]["layers"][uid(1)].update(parent=uid(2))),
            ("nonuniform-circle", lambda p: p["document"]["entities"][uid(12)].update(placement=[[2, 0, 0, 0], [0, 1, 0, 0], [0, 0, 1, 0], [0, 0, 0, 1]])),
            ("singular", lambda p: p["document"]["entities"][uid(10)].update(placement=[[0, 0, 0, 0], [0, 1, 0, 0], [0, 0, 1, 0], [0, 0, 0, 1]])),
            ("bad-frame", lambda p: p["document"]["entities"][uid(12)]["geometry"]["Curve"]["Circle"].update(x=[2, 0, 0])),
            ("bad-uuid", lambda p: p["document"].update(id="invalid")),
            ("bad-knots", lambda p: p["document"]["entities"][uid(13)]["geometry"]["Curve"]["Nurbs"].update(knots=[2, 2, 3, 2, 4, 4])),
            ("outer-knots", lambda p: p["document"]["entities"][uid(13)]["geometry"]["Curve"]["Nurbs"].update(knots=[1, 2, 2, 4, 4, 5])),
            ("trim-domain", lambda p: p["document"]["entities"][uid(13)]["geometry"]["Curve"]["Nurbs"].update(domain=[2.5, 3.5])),
            ("unknown-geometry", lambda p: p["document"]["entities"][uid(10)].update(geometry={"Brep": {}})),
            ("negative-revision", lambda p: p["document"]["entities"][uid(10)].update(generation=-1)),
            ("member-locked", lambda p: p["document"]["definitions"][uid(30)]["members"][0]["attributes"].update(locked=True)),
            ("layer-selectable", lambda p: p["document"]["layers"][uid(1)].update(selectable=False)),
            ("surface-reversed", lambda p: p["document"]["entities"][uid(14)]["geometry"]["Surface"].update(reversed=True)),
            ("curve-extra-field", lambda p: p["document"]["entities"][uid(11)]["geometry"]["Curve"]["Line"].update(extra=1)),
            ("definition-name-normalization", lambda p: p["document"]["definitions"][uid(30)].update(label="  Master  ")),
        ]:
            bad = copy.deepcopy(packet)
            mutate(bad)
            reject(name, bad)
        reject("string-budget", raw=json.dumps(packet).replace("Authored conversion", "x" * 4097))
        reject("array-budget", raw='[' + ','.join(['0'] * 65537) + ']')
        reject("node-budget", raw=json.dumps([[0] * 65536] * 4))
        reject("nonfinite", raw=json.dumps(packet).replace('[1, 2, 3]', '[1e999, 2, 3]', 1))
        collision = copy.deepcopy(packet)
        collision["document"]["layers"]["ffffffff-ffff-4fff-8fff-ffffffffffff"] = collision["document"]["layers"].pop(uid(1))
        collision["document"]["layers"]["ffffffff-ffff-4fff-8fff-ffffffffffff"]["id"] = "ffffffff-ffff-4fff-8fff-ffffffffffff"
        collision["document"]["layer_order"][0] = "ffffffff-ffff-4fff-8fff-ffffffffffff"
        reject("default-collision", collision)
        depth_chain = copy.deepcopy(packet)
        previous = uid(30)
        for n in range(60, 95):
            depth_chain["document"]["definitions"][uid(n)] = {
                "id": uid(n), "label": f"Depth{n}", "revision": 1,
                "members": [entity(1000 + n, f"DepthMember{n}", {"Instance": previous})]}
            previous = uid(n)
        reject("definition-depth", depth_chain)
        path.write_text(json.dumps(packet), encoding="utf-8")
        run(["--write", path, model], False)
        assert model.read_bytes() == original
        rejection_cases.append("existing-output")
        model.write_bytes(original[:len(original) // 2])
        run(["--inspect", model], False)
        rejection_cases.append("truncated-archive")
        model.write_bytes(b"corrupt")
        run(["--inspect", model], False)
        imports = pe_imports(binary)
        forbidden = ("freecad", "qymcad", "qt5", "qt6", "coin", "python", "tkernel", "tkbrep")
        assert not any(any(name in library.lower() for name in forbidden) for library in imports), imports
        def digest(file):
            return hashlib.sha256(file.read_bytes()).hexdigest()
        dependency = ROOT / "build/standalone-deps/opennurbs"
        commit = subprocess.check_output(["rtk", "git", "-C", str(dependency), "rev-parse", "HEAD"], text=True).strip()
        assert commit == "eb92af3ba1806b0a34a99aba0d3bda83e3d46083", commit
        assert subprocess.run(["rtk", "git", "-C", str(dependency), "diff", "--quiet", "HEAD"], capture_output=True).returncode == 0
        json_dependency = ROOT / "build/standalone-deps/nlohmann-json-3.11.3"
        assert digest(json_dependency / "json.hpp") == "9bea4c8066ef4a1c206b2be5a36302f8926f7fdc6087af5d20b417d0cf103ea6"
        assert digest(json_dependency / "LICENSE.MIT") == "86b998c792894ccb911a1cb7994f7a9652894e7a094c0b5e45be2f553f45cf14"
        receipt = {"scope": "geometry_only_current_authored_conversion", "binary_sha256": binary_hash,
                   "pe_imports": imports,
                   "runner_sha256": digest(runner),
                   "authored_sources": [{"path": file.relative_to(ROOT).as_posix(), "sha256": digest(file)}
                                        for file in [ROOT / "native/native_exchange.cpp", ROOT / "native/retained_merge.hpp", ROOT / "native/CMakeLists.txt", Path(__file__).resolve()]],
                   "dependencies": {"opennurbs_commit": commit, "opennurbs_worktree_clean": True,
                                    "opennurbs_license_sha256": digest(dependency / "LICENSE"),
                                    "json_version": "3.11.3", "json_header_sha256": digest(json_dependency / "json.hpp"),
                                    "json_license_sha256": digest(json_dependency / "LICENSE.MIT")},
                   "native_packet_limits": {"bytes": 16777216, "depth": 32, "nodes": 262144,
                                            "container_entries": 65536, "string_bytes": 4096,
                                            "objects_layers_definitions": 4096, "degree": 16},
                   "supported_native_families": ["ON_Point", "ON_LineCurve", "ON_ArcCurve full Circle",
                                                  "ON_NurbsCurve", "ON_PolyCurve contiguous segments", "ON_NurbsSurface",
                                                  "ON_Mesh triangle/quad", "ON_PointCloud", "ON_InstanceRef shared/nested"],
                   "inventory": data, "rejection_cases": rejection_cases + ["corrupt-archive"],
                   "runs": runs, "limitations": ["No retained merge", "No Rhino 5 UI acceptance", "No native memory-safety claim",
                       "Independent selectable=false, hidden+locked entities, locked definition members refused",
                       "Direct nonuniform circles with nonidentity angle correspondence refused",
                       "Reversed/seam metadata, unmatched periodicity, trimmed NURBS domains and noncanonical outer knots refused",
                       "Cloud hidden=true refused; false-only runtime flags absent from native archive",
                       "Native normal channels normalized by inverse transpose; mesh normals use native single precision"]}
        args.receipt.parent.mkdir(parents=True, exist_ok=True)
        args.receipt.write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
        print(json.dumps({"status": "validated", "objects": len(objects), "archive_version": 50,
                          "rejected": len(rejection_cases) + 1, "native_runs": len(runs),
                          "peak_committed_bytes": max(r["stats"]["peak_committed_bytes"] for r in runs)}))


if __name__ == "__main__":
    main()
