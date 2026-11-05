#!/usr/bin/env python3
"""Sinh bảng size nhẫn cho add-on library `ring_sizes` từ công thức chuẩn, có tự kiểm tra.

- US/Canada: đường kính trong d = 11.63 + 0.8128 × size (mm); size 0 = 0.458 in, mỗi size +0.032 in.
- ISO 8653 (EU/VN): số size = chu vi trong (mm) → d = size / π.

Không đưa UK/JP vào vì cần bảng chuẩn gốc để đối chiếu.
"""
import json
import math
import sys
from pathlib import Path

US_D0, US_STEP = 0.458 * 25.4, 0.032 * 25.4  # 11.6332, 0.8128


def us_diameter(size: float) -> float:
    return US_D0 + US_STEP * size


def build():
    us = []
    for q in range(0, 16 * 4 + 1):  # 0 … 16, bước 1/4
        size = q / 4
        d = us_diameter(size)
        us.append({"size": size, "diameter_mm": round(d, 3), "circumference_mm": round(math.pi * d, 2),
                   "iso_nearest": round(math.pi * d)})
    iso = [{"size": s, "diameter_mm": round(s / math.pi, 3), "circumference_mm": float(s)} for s in range(38, 77)]
    return {
        "schema": 1,
        "unit": "mm",
        "systems": {
            "US": {"formula": "d = 11.6332 + 0.8128*size", "sizes": us},
            "ISO8653": {"formula": "d = size/pi (size = inner circumference mm)", "sizes": iso},
        },
    }


def self_check(t):
    us = {r["size"]: r["diameter_mm"] for r in t["systems"]["US"]["sizes"]}
    iso = {r["size"]: r["diameter_mm"] for r in t["systems"]["ISO8653"]["sizes"]}
    # Giá trị tham chiếu thường gặp trên bảng size (làm tròn 0.1 mm)
    ref_us = {5: 15.7, 6: 16.5, 7: 17.3, 8: 18.1, 9: 18.9, 10: 19.8}
    for s, d in ref_us.items():
        assert abs(us[s] - d) <= 0.06, (s, us[s], d)
    assert abs(iso[54] - 17.19) < 0.01 and abs(iso[57] - 18.14) < 0.01
    # US 7 ≈ ISO 54.4 → gần nhất 54
    assert next(r for r in t["systems"]["US"]["sizes"] if r["size"] == 7)["iso_nearest"] == 54
    diffs = [b - a for a, b in zip(list(us.values()), list(us.values())[1:])]
    assert all(abs(x - US_STEP / 4) < 0.002 for x in diffs)


if __name__ == "__main__":
    out = Path(sys.argv[1] if len(sys.argv) > 1 else "examples/ring_sizes/tables/ring_sizes.json")
    t = build()
    self_check(t)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(t, indent=1), encoding="utf-8")
    print(f"OK {out}: {len(t['systems']['US']['sizes'])} size US, {len(t['systems']['ISO8653']['sizes'])} size ISO")
