"""Command-checklist audit over the headless MCP server.

For every checklist command it records (a) the menu/feature dispatch route,
(b) the typed Command Line route and (c) the geometry that route produced.
Results are written as JSON plus a Markdown table.
"""
import json
import math
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
from om9_mcp_client import Client

EXE = r"L:\om9-target\debug\om9.exe"
OUT = os.path.join(os.path.dirname(__file__), "..", "build", "command-audit")
TOL = 1e-6


def close(a, b, tol=TOL):
    return abs(a - b) <= tol


def point_close(a, b, tol=TOL):
    return all(close(x, y, tol) for x, y in zip(a, b))


# --- geometry extraction from the full report projection -------------------
def curve_points(curve):
    if "Line" in curve:
        line = curve["Line"]
        return [line["start"], line["end"]]
    if "Circle" in curve:
        c = curve["Circle"]
        center, r = c["center"], c["radius"]
        x, y = c["x"], c["y"]
        start, end = c.get("angle_domain") or [0.0, 2.0 * math.pi]
        angles = [start, end]
        for k in range(-2, 4):
            for base in (0.0, 0.5 * math.pi, math.pi, 1.5 * math.pi):
                angle = base + k * 2.0 * math.pi
                if start < angle < end:
                    angles.append(angle)
        return [
            [
                center[i]
                + r * math.cos(angle) * x[i]
                + r * math.sin(angle) * y[i]
                for i in range(3)
            ]
            for angle in angles
        ]
    if "Nurbs" in curve:
        return curve["Nurbs"]["poles"]
    if "Polycurve" in curve:
        pts = []
        for segment in curve["Polycurve"]["segments"]:
            pts.extend(curve_points(segment))
        return pts
    return []


def geometry_points(geometry):
    if "Brep" in geometry:
        return geometry["Brep"]["display"]["points"]
    if "Point" in geometry:
        return [geometry["Point"]]
    if "Curve" in geometry:
        return curve_points(geometry["Curve"])
    if "Surface" in geometry:
        return geometry["Surface"]["poles"]
    if "Mesh" in geometry:
        return geometry["Mesh"]["vertices"]
    if "Cloud" in geometry:
        return geometry["Cloud"]["points"]
    return []


def world_points(entity):
    placement = entity["placement"]
    points = geometry_points(entity["geometry"])
    out = []
    for p in points:
        out.append(
            [
                sum(placement[i][j] * p[j] for j in range(3)) + placement[i][3]
                for i in range(3)
            ]
        )
    return out


def bounds(points):
    if not points:
        raise AssertionError("geometry has no points")
    return (
        [min(p[i] for p in points) for i in range(3)],
        [max(p[i] for p in points) for i in range(3)],
    )


def brep_volume(entity):
    display = entity["geometry"]["Brep"]["display"]
    placement = entity["placement"]
    pts = [
        [
            sum(placement[i][j] * p[j] for j in range(3)) + placement[i][3]
            for i in range(3)
        ]
        for p in display["points"]
    ]
    volume = 0.0
    for triangle in display["triangles"]:
        a, b, c = (pts[triangle[i]] for i in range(3))
        volume += (
            a[0] * (b[1] * c[2] - b[2] * c[1])
            - a[1] * (b[0] * c[2] - b[2] * c[0])
            + a[2] * (b[0] * c[1] - b[1] * c[0])
        ) / 6.0
    return abs(volume)


def curve_segments(curve):
    if "Polycurve" in curve:
        return len(curve["Polycurve"]["segments"])
    return 1


class Audit:
    def __init__(self):
        self.client = Client(EXE)
        self.results = []
        os.makedirs(OUT, exist_ok=True)

    def report(self):
        path = os.path.join(OUT, "report.json")
        self.client.tool("om9_report", path=path)
        with open(path, encoding="utf-8") as handle:
            return json.load(handle)

    def objects(self):
        return self.client.state(limit=400)["data"]["objects"]

    def fresh(self, name):
        self.client.new(name)

    def do(self, step):
        data = self.client.command(step)["data"]
        result = data.get("result", {})
        if not result.get("ok"):
            raise AssertionError(f"{step!r}: {result.get('error')}")
        return data

    def enter(self):
        data = self.client.command("")["data"]
        result = data.get("result", {})
        if not result.get("ok"):
            raise AssertionError(f"Enter: {result.get('error')}")
        return data

    def point(self, point):
        data = self.client.point(point)["data"]
        if not data.get("result", {}).get("ok"):
            raise AssertionError(f"point {point}: {data.get('result', {}).get('error')}")

    def draw_line(self, start, end):
        self.do("Line")
        self.point(start)
        self.point(end)
        return self.ids_by_label("Line")[-1]

    def ids_by_label(self, label):
        return [o["id"] for o in self.objects() if o["label"] == label]

    def select(self, ids):
        reply = self.client.select(ids=ids)
        if reply["is_error"]:
            raise AssertionError(f"select: {reply['data']}")

    def run(self, item, group, feature, typed_name, prereq, steps, check):
        """Check menu start, typed start and produced geometry from a clean document."""
        menu_status = "n/a"
        if feature:
            self.fresh(f"menu {item}")
            prereq(self)
            data = self.client.dispatch(feature, origin="menu")["data"]
            result = data.get("result", {})
            menu_status = "PASS" if result.get("ok") else f"FAIL {result}"
        typed_status = "n/a"
        if typed_name:
            self.fresh(f"cmd {item}")
            prereq(self)
            try:
                self.do(typed_name)
                typed_status = "PASS"
            except Exception as error:  # noqa: BLE001
                typed_status = f"FAIL {error}"
        geometry = "PASS"
        detail = []
        try:
            self.fresh(f"geom {item}")
            prereq(self)
            steps(self)
            check(self)
        except Exception as error:  # noqa: BLE001 - audit records every failure
            geometry = "FAIL"
            detail.append(str(error))
        finally:
            try:
                self.client.command("Cancel")
            except Exception:  # noqa: BLE001
                pass
        self.results.append(
            {
                "item": item,
                "group": group,
                "feature": feature,
                "typed": typed_name,
                "menu": menu_status,
                "command_line": typed_status,
                "geometry": geometry,
                "detail": "; ".join(detail),
            }
        )
        print(f"[{geometry}] {item:34} menu={menu_status:6} cmd={typed_status:6} {detail[0] if detail else ''}")

    def run_extra(self, item, group, feature, prereq, steps, note):
        """Menu-only rows (no standalone command line) with a geometry result."""
        self.fresh(f"menu {item}")
        prereq(self)
        data = self.client.dispatch(feature, origin="menu")["data"]
        result = data.get("result", {})
        status = "PASS" if result.get("ok") else f"FAIL {result}"
        geometry = "n/a"
        detail = note
        try:
            steps(self)
            geometry = "PASS"
        except Exception as error:  # noqa: BLE001
            geometry = "FAIL"
            detail = f"{note}; {error}"
        self.results.append(
            {
                "item": item,
                "group": group,
                "feature": feature,
                "typed": None,
                "menu": status,
                "command_line": "n/a",
                "geometry": geometry,
                "detail": detail,
            }
        )
        print(f"[{geometry}] {item:34} menu={status:14} {detail}")


A = Audit()


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def no_prereq(_a):
    return None


# ---------------------------------------------------------------- scenarios
def box_steps(a):
    a.do("Box")
    a.point([0.0, 0.0, 0.0])
    a.point([10.0, 20.0, 0.0])
    a.do("5")


def check_box(a):
    entity = a.report()["objects"][0]
    require("Brep" in entity["geometry"], f"geometry {list(entity['geometry'])}")
    lo, hi = bounds(world_points(entity))
    require(point_close(lo, [0, 0, 0]), f"min {lo}")
    require(point_close(hi, [10, 20, 5]), f"max {hi}")
    require(close(brep_volume(entity), 1000.0, 1e-6), f"volume {brep_volume(entity)}")


def rectangle_steps(a):
    a.do("Rectangle")
    a.point([0.0, 0.0, 0.0])
    a.point([10.0, 20.0, 0.0])


def check_rectangle(a):
    entity = a.report()["objects"][0]
    curve = entity["geometry"]["Curve"]
    require(curve_segments(curve) == 4, f"segments {curve_segments(curve)}")
    lo, hi = bounds(world_points(entity))
    require(point_close(lo, [0, 0, 0]), f"min {lo}")
    require(point_close(hi, [10, 20, 0]), f"max {hi}")


def sphere_steps(a):
    a.do("Sphere")
    a.point([0.0, 0.0, 0.0])
    a.do("5")


def check_sphere(a):
    entity = a.report()["objects"][0]
    require("Brep" in entity["geometry"], "not a Brep")
    lo, hi = bounds(world_points(entity))
    require(close(lo[0], -5.0, 0.01) and close(hi[0], 5.0, 0.01), f"x {lo} {hi}")
    require(close(lo[1], -5.0, 0.01) and close(hi[1], 5.0, 0.01), f"y {lo} {hi}")
    require(close(lo[2], -5.0, 0.01) and close(hi[2], 5.0, 0.01), f"z {lo} {hi}")
    volume = brep_volume(entity)
    expected = 4.0 / 3.0 * math.pi * 125.0
    require(abs(volume / expected - 1.0) < 0.02, f"volume {volume} vs {expected}")


def line_prereq(a):
    a.draw_line([0.0, 0.0, 0.0], [10.0, 0.0, 0.0])
    a.select(a.ids_by_label("Line"))


def copy_steps(a):
    a.do("Copy")
    a.point([0.0, 0.0, 0.0])
    a.point([0.0, 0.0, 10.0])
    a.enter()


def check_copy(a):
    report = a.report()
    require(len(report["objects"]) == 2, f"objects {len(report['objects'])}")
    moved = []
    for entity in report["objects"]:
        lo, hi = bounds(world_points(entity))
        if close(lo[2], 10.0) and close(hi[2], 10.0):
            moved.append((lo, hi))
    require(len(moved) == 1, f"copy z=10 not found: {moved}")
    lo, hi = moved[0]
    require(close(lo[0], 0.0) and close(hi[0], 10.0), f"bounds {lo} {hi}")


def polyline_steps(a):
    a.do("Polyline")
    a.point([0.0, 0.0, 0.0])
    a.point([10.0, 0.0, 0.0])
    a.point([10.0, 10.0, 0.0])
    a.enter()


def check_polyline(a):
    entity = a.report()["objects"][0]
    curve = entity["geometry"]["Curve"]
    require(curve_segments(curve) == 2, f"segments {curve_segments(curve)}")
    lo, hi = bounds(world_points(entity))
    require(point_close(lo, [0, 0, 0]) and point_close(hi, [10, 10, 0]), f"{lo} {hi}")


def offset_steps(a):
    a.do("Offset")
    a.do("Distance=2")
    a.enter()


def check_offset(a):
    report = a.report()
    require(len(report["objects"]) == 2, f"objects {len(report['objects'])}")
    moved = []
    for entity in report["objects"]:
        lo, hi = bounds(world_points(entity))
        if close(abs(lo[1]), 2.0) and close(abs(hi[1]), 2.0):
            moved.append((lo, hi))
    require(len(moved) == 1, f"offset y=+-2 not found: {moved}")
    lo, hi = moved[0]
    require(close(lo[0], 0.0) and close(hi[0], 10.0), f"{lo} {hi}")


def mirror_prereq(a):
    a.draw_line([1.0, 0.0, 0.0], [4.0, 0.0, 0.0])
    a.select(a.ids_by_label("Line"))


def mirror_steps(a):
    a.do("Mirror")
    a.point([0.0, -5.0, 0.0])
    a.point([0.0, 5.0, 0.0])


def check_mirror(a):
    report = a.report()
    require(len(report["objects"]) == 2, f"objects {len(report['objects'])}")
    mirrored = []
    for entity in report["objects"]:
        lo, hi = bounds(world_points(entity))
        if close(lo[0], -4.0) and close(hi[0], -1.0):
            mirrored.append((lo, hi))
    require(len(mirrored) == 1, f"mirrored copy not found: {mirrored}")


def move_prereq(a):
    a.draw_line([0.0, 0.0, 0.0], [10.0, 0.0, 0.0])
    a.select(a.ids_by_label("Line"))


def move_steps(a):
    a.do("Move")
    a.point([0.0, 0.0, 0.0])
    a.point([5.0, -2.0, 0.0])


def check_move(a):
    entity = a.report()["objects"][0]
    lo, hi = bounds(world_points(entity))
    require(point_close(lo, [5, -2, 0]) and point_close(hi, [15, -2, 0]), f"{lo} {hi}")


def rotate_prereq(a):
    a.draw_line([0.0, 0.0, 0.0], [10.0, 0.0, 0.0])
    a.select(a.ids_by_label("Line"))


def rotate_steps(a):
    a.do("Rotate")
    a.point([0.0, 0.0, 0.0])
    a.point([10.0, 0.0, 0.0])
    a.point([0.0, 10.0, 0.0])


def check_rotate(a):
    entity = a.report()["objects"][0]
    lo, hi = bounds(world_points(entity))
    require(close(lo[0], 0.0) and close(hi[0], 0.0), f"x {lo} {hi}")
    require(close(lo[1], 0.0) and close(hi[1], 10.0), f"y {lo} {hi}")


def rotate3d_steps(a):
    a.do("Rotate3D")
    a.point([0.0, 0.0, 0.0])
    a.do("90")


def plane_steps(a):
    a.do("Plane")
    a.point([0.0, 0.0, 0.0])
    a.point([10.0, 0.0, 0.0])
    a.point([10.0, 20.0, 0.0])


def check_plane(a):
    entity = a.report()["objects"][0]
    require("Surface" in entity["geometry"], f"geometry {entity['geometry']}")
    lo, hi = bounds(world_points(entity))
    require(point_close(lo, [0, 0, 0]) and point_close(hi, [10, 20, 0]), f"{lo} {hi}")


def extrude_prereq(a):
    a.do("Rectangle")
    a.point([0.0, 0.0, 0.0])
    a.point([10.0, 20.0, 0.0])
    a.select([o["id"] for o in a.objects()])


def extrude_steps(a):
    a.do("ExtrudeCrv")
    a.do("5")


def check_extrude(a):
    report = a.report()
    bodies = [o for o in report["objects"] if "Brep" in o["geometry"]]
    require(len(bodies) == 1, f"breps {len(bodies)}")
    entity = bodies[0]
    lo, hi = bounds(world_points(entity))
    require(point_close(lo, [0, 0, 0]), f"min {lo}")
    require(point_close(hi, [10, 20, 5]), f"max {hi}")
    require(close(brep_volume(entity), 1000.0, 1e-6), f"volume {brep_volume(entity)}")


def difference_button_prereq(a):
    a.do("Box 10 10 10")
    a.do("Box 10 10 10")
    ids = [o["id"] for o in a.objects()]
    a.client.select(ids=ids[1:])
    a.do("Move")
    a.point([0.0, 0.0, 0.0])
    a.point([5.0, 0.0, 0.0])
    a.select(ids[:1])


def difference_button_check(a):
    a.enter()
    prompt = a.client.state()["data"]["command"]["prompt"]
    require("subtract" in prompt.lower() or "second set" in prompt.lower(), prompt)


def plane_prereq(a):
    a.do("Plane")
    a.point([0.0, 0.0, 0.0])
    a.point([10.0, 0.0, 0.0])
    a.point([10.0, 20.0, 0.0])
    a.select(a.ids_by_label("Plane from Corners"))

def offset_srf_steps(a):
    a.do("OffsetSrf")
    a.do("5")


def check_offset_srf(a):
    report = a.report()
    surfaces = [o for o in report["objects"] if "Surface" in o["geometry"]]
    require(len(surfaces) == 2, f"surfaces {len(surfaces)}")
    heights = []
    for surface in surfaces:
        lo, hi = bounds(world_points(surface))
        heights.append(lo[2])
    require(
        sum(1 for z in heights if close(abs(z), 5.0, 1e-6)) == 1,
        f"offset plane not at |z|=5: {heights}",
    )
    require(sum(1 for z in heights if close(z, 0.0, 1e-6)) == 1, f"source plane: {heights}")


def box_prereq(a):
    a.do("Box 10 10 10")
    a.select(a.ids_by_label("Box"))


def shell_steps(a):
    a.do("Shell 0 1")


def check_shell(a):
    entity = a.report()["objects"][0]
    require("Brep" in entity["geometry"], "not a Brep")
    lo, hi = bounds(world_points(entity))
    require(point_close(lo, [0, 0, 0]), f"min {lo}")
    require(point_close(hi, [10, 10, 10]), f"max {hi}")
    volume = brep_volume(entity)
    require(abs(volume - 424.0) < 2.0, f"volume {volume}")


def dupedge_steps(a):
    a.do("DupEdge")


def check_dupedge(a):
    report = a.report()
    curves = [o for o in report["objects"] if "Curve" in o["geometry"]]
    require(len(curves) == 12, f"curves {len(curves)}")
    for curve in curves:
        lo, hi = bounds(world_points(curve))
        require(all(-1e-9 <= v <= 10.0 + 1e-9 for v in lo + hi), f"curve outside box {lo} {hi}")


def join_prereq(a):
    a.draw_line([0.0, 0.0, 0.0], [10.0, 0.0, 0.0])
    a.draw_line([10.0, 0.0, 0.0], [10.0, 10.0, 0.0])
    a.select(a.ids_by_label("Line"))


def join_steps(a):
    a.do("Join")


def check_join(a):
    report = a.report()
    require(len(report["objects"]) == 1, f"objects {len(report['objects'])}")
    curve = report["objects"][0]["geometry"]["Curve"]
    require(curve_segments(curve) == 2, f"segments {curve_segments(curve)}")
    lo, hi = bounds(world_points(report["objects"][0]))
    require(point_close(lo, [0, 0, 0]) and point_close(hi, [10, 10, 0]), f"{lo} {hi}")


def split_prereq(a):
    a.do("Circle")
    a.point([0.0, 0.0, 0.0])
    a.do("5")
    a.select(a.ids_by_label("Circle"))


def split_steps(a):
    a.do("Split")
    a.do("Point")
    a.do("W0,5,0")
    a.do("W0,-5,0")
    a.enter()


def check_split(a):
    report = a.report()
    require(len(report["objects"]) == 2, f"objects {len(report['objects'])}")
    bounds_list = [bounds(world_points(entity)) for entity in report["objects"]]
    left = [b for b in bounds_list if close(b[0][0], -5.0, 1e-6) and close(b[1][0], 0.0, 1e-6)]
    right = [b for b in bounds_list if close(b[0][0], 0.0, 1e-6) and close(b[1][0], 5.0, 1e-6)]
    require(len(left) == 1 and len(right) == 1, f"arc halves {bounds_list}")
    for lo, hi in bounds_list:
        require(close(lo[1], -5.0, 1e-6) and close(hi[1], 5.0, 1e-6), f"arc y {lo} {hi}")


def trim_prereq(a):
    a.draw_line([-10.0, 0.0, 0.0], [10.0, 0.0, 0.0])
    a.draw_line([0.0, -10.0, 0.0], [0.0, 10.0, 0.0])
    a.select(a.ids_by_label("Line"))


def trim_steps(a):
    a.do("Trim")
    a.point([-5.0, 0.0, 0.0])
    a.enter()


def check_trim(a):
    report = a.report()
    require(len(report["objects"]) == 2, f"objects {len(report['objects'])}")
    horizontals = []
    for entity in report["objects"]:
        lo, hi = bounds(world_points(entity))
        if close(lo[1], 0.0) and close(hi[1], 0.0):
            horizontals.append((lo, hi))
    require(len(horizontals) == 1, f"horizontals {horizontals}")
    lo, hi = horizontals[0]
    require(close(lo[0], 0.0, 1e-6) and close(hi[0], 10.0, 1e-6), f"{lo} {hi}")


def difference_prereq(a):
    a.do("Box 10 10 10")
    a.do("Box 10 10 10")
    ids = [o["id"] for o in a.objects()]
    a.client.select(ids=ids[1:])
    a.do("Move")
    a.point([0.0, 0.0, 0.0])
    a.point([5.0, 0.0, 0.0])
    a.select(ids)


def difference_steps(a):
    a.do("Boolean2Objects")
    a.do("DeleteInput=Yes")
    a.do("Difference")
    a.enter()


def check_difference(a):
    report = a.report()
    require(len(report["objects"]) == 1, f"objects {len(report['objects'])}")
    volume = brep_volume(report["objects"][0])
    require(abs(volume - 500.0) < 1e-6, f"volume {volume}")


def stl_check(a):
    target = os.path.join(OUT, "audit-export.stl")
    if os.path.exists(target):
        os.remove(target)
    reply = a.client.tool("om9_export_stl", path=target)
    require(not reply["is_error"], f"export failed: {reply['data']}")
    receipt = reply["data"]
    require(receipt["triangles"] == 12, f"triangles {receipt['triangles']}")
    require(receipt["units"] == "mm", "units")
    with open(target, "rb") as handle:
        import struct

        data = handle.read()
    require(len(data) == 84 + 12 * 50, "binary STL size")
    require(struct.unpack_from("<I", data, 80)[0] == 12, "binary STL count")
    os.remove(target)


def run_all():
    a = A
    a.run("solid -> box", "solid", "OM9-SOLID-012", "Box", no_prereq, box_steps, check_box)
    a.run("curve -> rectangle", "curve", "OM9-CURVE-004", "Rectangle", no_prereq, rectangle_steps, check_rectangle)
    a.run("solid -> sphere", "solid", "OM9-SOLID-014", "Sphere", no_prereq, sphere_steps, check_sphere)
    a.run("top11 -> copy", "top11", "OM9-TOP11-002", "Copy", line_prereq, copy_steps, check_copy)
    a.run("curve -> polyline", "curve", "OM9-CURVE-001", "Polyline", no_prereq, polyline_steps, check_polyline)
    a.run("curve -> offset", "curve", "OM9-CURVE-018", "Offset", line_prereq, offset_steps, check_offset)
    a.run("top11 -> mirror", "top11", "OM9-TOP11-004", "Mirror", mirror_prereq, mirror_steps, check_mirror)
    a.run("top11 -> move", "top11", "OM9-TOP11-006", "Move", move_prereq, move_steps, check_move)
    a.run("top11 -> rotate", "top11", "OM9-TOP11-007", "Rotate", rotate_prereq, rotate_steps, check_rotate)
    a.run("transform -> rotate 3d", "transform", "OM9-TRANSFORM-008", "Rotate3D", rotate_prereq, rotate3d_steps, check_rotate)
    a.run("surface -> plane from corner", "surface", "OM9-SURFACE-015", "Plane", no_prereq, plane_steps, check_plane)
    a.run("solid -> extrude curve straight", "solid", "OM9-SOLID-009", "ExtrudeCrv", extrude_prereq, extrude_steps, check_extrude)
    a.run("surface -> offset surface", "surface", "OM9-SURFACE-025", "OffsetSrf", plane_prereq, offset_srf_steps, check_offset_srf)
    a.run("solid -> shell", "solid", "OM9-SOLID-029", "Shell", box_prereq, shell_steps, check_shell)
    a.run("curve -> dupEdge", "curve", "OM9-CURVE-040", "DupEdge", box_prereq, dupedge_steps, check_dupedge)
    a.run("top11 -> join", "top11", "OM9-TOP11-008", "Join", join_prereq, join_steps, check_join)
    a.run("top11 -> split", "top11", "OM9-TOP11-009", "Split", split_prereq, split_steps, check_split)
    a.run("top11 -> trim", "top11", "OM9-TOP11-010", "Trim", trim_prereq, trim_steps, check_trim)
    a.run("solid -> different", "solid", "OM9-SOLID-004", "Boolean2Objects", difference_prereq, difference_steps, check_difference)
    a.run("file -> export stl", "file", "OM9-FILE-001", "ExportSTL", box_prereq, lambda _a: None, stl_check)
    a.run_extra(
        "solid -> different (button)",
        "solid",
        "OM9-SOLID-001",
        difference_button_prereq,
        difference_button_check,
        "menu starts and waits for the second set; exact 500 mm3 difference is measured by solid_workflows::immediate_boolean_menu_actions_have_measured_results",
    )

    a.client.close()
    path = os.path.join(OUT, "audit.json")
    with open(path, "w", encoding="utf-8") as handle:
        json.dump(a.results, handle, indent=2)
    print(f"wrote {path}")


if __name__ == "__main__":
    run_all()
