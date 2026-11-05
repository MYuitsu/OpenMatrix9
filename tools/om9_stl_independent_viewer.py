"""Independent STL read-back using trimesh (not the OM9 writer code).

Builds two fixtures through `om9 --mcp`, exports them, then validates the files with
trimesh: binary triangle count, bounds, signed volume, watertightness and winding.
Writes a JSON receipt next to the validation record.
"""
import hashlib
import importlib.metadata
import json
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
from om9_mcp_client import Client

import trimesh

EXE = r"L:\om9-target\debug\om9.exe"
OUT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "build", "command-audit"))
RECORD = os.path.abspath(
    os.path.join(
        os.path.dirname(__file__),
        "..",
        "docs",
        "validation",
        "2026-11-01-stl-independent-viewer.json",
    )
)


def sha256(path):
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def export(client, name, build, path):
    client.new(name)
    objects = build(client)
    client.select(ids=[objects[0]["id"]])
    reply = client.tool("om9_export_stl", path=path)
    if reply["is_error"]:
        raise SystemExit(f"{name}: export failed: {reply['data']}")
    receipt = reply["data"]
    mesh = trimesh.load(path, file_type="stl", force="mesh")
    result = {
        "name": name,
        "stl": os.path.relpath(path, os.path.commonpath([path, os.path.dirname(__file__)])),
        "sha256": sha256(path),
        "bytes": os.path.getsize(path),
        "export_triangles": receipt["triangles"],
        "viewer_triangles": int(len(mesh.faces)),
        "watertight": bool(mesh.is_watertight),
        "winding_consistent": bool(mesh.is_winding_consistent),
        "bounds": [list(map(float, mesh.bounds[0])), list(map(float, mesh.bounds[1]))],
        "volume_mm3": float(abs(mesh.volume)),
        "degenerate_faces": int((mesh.area_faces <= 0).sum()),
    }
    print(json.dumps(result, indent=1))
    return result


def box(client):
    client.command("Box 10 10 10")
    return client.state()["data"]["objects"]


def sphere(client):
    client.command("Sphere")
    client.point([0.0, 0.0, 0.0])
    client.command("5")
    return client.state()["data"]["objects"]


os.makedirs(OUT, exist_ok=True)
client = Client(EXE)
results = []
try:
    results.append(export(client, "box 10 mm", box, os.path.join(OUT, "viewer-box.stl")))
    results.append(export(client, "sphere r5 mm", sphere, os.path.join(OUT, "viewer-sphere.stl")))
finally:
    client.close()

box_result, sphere_result = results
checks = {
    "box_triangles": box_result["viewer_triangles"] == 12,
    "box_watertight": box_result["watertight"],
    "box_winding": box_result["winding_consistent"],
    "box_volume": abs(box_result["volume_mm3"] - 1000.0) < 1e-6,
    "box_bounds": box_result["bounds"][1] == [10.0, 10.0, 10.0],
    "sphere_watertight": sphere_result["watertight"],
    "sphere_winding": sphere_result["winding_consistent"],
    "sphere_volume": abs(sphere_result["volume_mm3"] / (4.0 / 3.0 * 3.141592653589793 * 125.0) - 1.0)
    < 0.01,
    "no_degenerate_faces": all(r["degenerate_faces"] == 0 for r in results),
    "triangle_counts_match": all(
        r["export_triangles"] == r["viewer_triangles"] for r in results
    ),
}
record = {
    "schema": "om9-stl-independent-viewer/v1",
    "date": "2026-11-01",
    "viewer": {
        "library": "trimesh",
        "version": importlib.metadata.version("trimesh"),
        "numpy_version": importlib.metadata.version("numpy"),
        "mode": "independent binary STL parse + mesh predicates",
    },
    "fixtures": results,
    "checks": checks,
    "all_passed": all(checks.values()),
}
with open(RECORD, "w", encoding="utf-8", newline="\n") as handle:
    json.dump(record, handle, indent=2)
    handle.write("\n")
print("checks", checks)
print("record", RECORD)
raise SystemExit(0 if record["all_passed"] else 1)
