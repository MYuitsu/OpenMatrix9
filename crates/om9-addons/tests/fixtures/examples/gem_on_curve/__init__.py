"""Gem On Curve — triển khai độc lập cho OpenMatrix9.

Đường cong được host đưa vào dưới dạng polyline đã tessellate (danh sách
điểm 3D). Thuật toán chỉ dùng hình học cơ bản: tham số hoá theo độ dài cung,
chia đều khoảng cách tâm–tâm, trả về khung (vị trí, tiếp tuyến) cho từng viên.
"""
from __future__ import annotations

import math
from typing import Sequence

Vec = tuple[float, float, float]


def _sub(a: Vec, b: Vec) -> Vec:
    return (a[0] - b[0], a[1] - b[1], a[2] - b[2])


def _len(a: Vec) -> float:
    return math.sqrt(a[0] ** 2 + a[1] ** 2 + a[2] ** 2)


def _lerp(a: Vec, b: Vec, t: float) -> Vec:
    return (a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t)


def curve_length(pts: Sequence[Vec], closed: bool) -> float:
    seq = list(pts) + ([pts[0]] if closed else [])
    return sum(_len(_sub(seq[i + 1], seq[i])) for i in range(len(seq) - 1))


def gem_count(length: float, diameter: float, gap: float, closed: bool) -> tuple[int, float]:
    """Số viên tối đa và khe thực sau khi chia đều.

    Kín:  n = floor(L / (d + g)),       g' = L/n - d
    Hở:   n = floor((L + g) / (d + g)), g' = (L - n·d)/(n - 1)
    (đầu và cuối viên chạm hai đầu đường cong khi hở)
    """
    if diameter <= 0 or gap < 0 or length <= 0:
        raise ValueError("cần diameter > 0, gap >= 0, length > 0")
    if closed:
        n = math.floor(length / (diameter + gap) + 1e-12)
        if n < 1:
            return 0, 0.0
        return n, length / n - diameter
    n = math.floor((length + gap) / (diameter + gap) + 1e-12)
    if n < 1:
        return 0, 0.0
    return n, ((length - n * diameter) / (n - 1)) if n > 1 else 0.0


def _unit(a: Vec) -> Vec:
    l = _len(a)
    return (a[0] / l, a[1] / l, a[2] / l) if l > 0 else (0.0, 0.0, 0.0)


def _vertex_tangents(seq: Sequence[Vec], closed: bool) -> list[Vec]:
    """Tiếp tuyến tại đỉnh = trung bình hướng hai đoạn kề (sai số O(h²) thay vì O(h))."""
    m = len(seq)
    dirs = [_unit(_sub(seq[i + 1], seq[i])) for i in range(m - 1)]
    tans: list[Vec] = []
    for i in range(m):
        if closed:
            a = dirs[i - 1] if i > 0 else dirs[-1]
            b = dirs[i] if i < m - 1 else dirs[0]
        else:
            a = dirs[i - 1] if i > 0 else dirs[0]
            b = dirs[i] if i < m - 1 else dirs[-1]
        avg = _unit((a[0] + b[0], a[1] + b[1], a[2] + b[2]))
        tans.append(avg if avg != (0.0, 0.0, 0.0) else a)  # đoạn gập ngược 180°: giữ hướng trước
    return tans


def point_at_length(pts: Sequence[Vec], s: float, closed: bool) -> tuple[Vec, Vec]:
    """Điểm và tiếp tuyến đơn vị tại độ dài cung s."""
    seq = list(pts) + ([pts[0]] if closed else [])
    tans = _vertex_tangents(seq, closed)
    acc = 0.0
    for i in range(len(seq) - 1):
        l = _len(_sub(seq[i + 1], seq[i]))
        if l == 0:
            continue
        if acc + l >= s - 1e-12 or i == len(seq) - 2:
            t = min(max((s - acc) / l, 0.0), 1.0)
            tan = _unit(_lerp(tans[i], tans[i + 1], t))
            return _lerp(seq[i], seq[i + 1], t), tan
        acc += l
    raise ValueError("polyline suy biến")


def place_gems(pts: Sequence[Vec], diameter: float, gap: float, closed: bool, layout: str = "fit"):
    """Trả về danh sách {center, tangent, diameter} cho từng viên đá.

    layout="fit"       : chia đều, khe thực g' ≥ gap sao cho khít đường cong.
    layout="fixed_gap" : giữ đúng khe gap; phần dư nằm ở cuối (đường hở) hoặc tại mối nối (đường kín).
    """
    if layout not in ("fit", "fixed_gap"):
        raise ValueError(f"layout không hợp lệ: {layout!r}")
    L = curve_length(pts, closed)
    n, g = gem_count(L, diameter, gap, closed)
    if layout == "fixed_gap":
        g = gap
    pitch = diameter + g
    start = 0.0 if closed else diameter / 2
    out = []
    for k in range(n):
        c, t = point_at_length(pts, start + k * pitch, closed)
        out.append({"center": c, "tangent": t, "diameter": diameter})
    return out


# ---- điểm nối với host OM9 (host gọi register khi nạp add-on) ----
# ctx.settings: giá trị hiệu lực của [[settings]] trong manifest (host đã kiểm tra kiểu/khoảng).

def gem_on_curve(ctx):
    s = ctx.settings
    pts, closed = ctx.pick_curve_polyline("Chọn đường cong đặt đá")
    d = ctx.ask_number("Đường kính đá (mm)", default=s["diameter"], minimum=0.3)
    g = ctx.ask_number("Khe giữa hai viên (mm)", default=s["gap"], minimum=0.0)
    for gem in place_gems(pts, d, g, closed, s["layout"]):
        ctx.add_round_gem(gem["center"], gem["tangent"], gem["diameter"])


def gem_count_on_curve(ctx):
    s = ctx.settings
    pts, closed = ctx.pick_curve_polyline("Chọn đường cong")
    d = ctx.ask_number("Đường kính đá (mm)", default=s["diameter"], minimum=0.3)
    g = ctx.ask_number("Khe tối thiểu (mm)", default=s["gap"], minimum=0.0)
    n, real_gap = gem_count(curve_length(pts, closed), d, g, closed)
    ctx.message(f"Đặt được {n} viên, khe thực {real_gap:.3f} mm")


def register(api):
    api.command("BuilderGemOnCrv", gem_on_curve)
    api.command("BuilderGemCountOnCrv", gem_count_on_curve)
