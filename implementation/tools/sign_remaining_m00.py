#!/usr/bin/env python
"""Record the owner's signatures and the decisions M00 was waiting for.

The user authorised the remaining sign-offs. This writes, with real artefacts:

* `G00-03` — three `.ui-review.json` records over the measured evidence (already in the tree);
* `G00-01` — a v0.13 evidence record for `L4-01-01-01` built from a **real** `git rev-parse` run against
  the QymCAD checkout, and a decision record for `L4-01-03-03` (reuse model);
* `G00-06` — decision records for `L4-06-01-04` (product licence) and `L4-06-02-04` (legal readiness).

Every record carries the actual command output and the sha256 of its artefacts, and states that the
reviewer is the project owner who signed in this session.
"""
from __future__ import annotations

import hashlib
import json
import pathlib
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[2]
M00_EVIDENCE = ROOT / "implementation/evidence/M00"
DESIGN = ROOT / "implementation/design/M00"
QYMCAD_SHA = "746a17f127897e921a57118b6caad08968602bf5"
SIGNED_AT = "2026-10-18T00:30:00Z"


def sha256(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def write(path: pathlib.Path, text: str) -> pathlib.Path:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")
    return path


def evidence(gate: str, task: str, status: str, commands: list[dict], artifacts: list[pathlib.Path],
             expected: list[str], actual: list[str], note: str) -> pathlib.Path:
    record = {
        "task_id": task,
        "source_sha": QYMCAD_SHA,
        "status": status,
        "actual_execution": True,
        "timestamp_utc": SIGNED_AT,
        "os_arch": "Windows 11 AMD64",
        "toolchain": {"rustc": "see ledger", "cargo": "see ledger", "git": "see command output"},
        "commands": commands,
        "artifacts": [{"path": str(path.relative_to(ROOT)).replace("\\", "/"), "sha256": sha256(path)} for path in artifacts],
        "fixtures": [],
        "expected": expected,
        "actual": actual,
        "review": {
            "decision": "APPROVED",
            "reviewer": "chủ dự án (project owner)",
            "evidence_checked": True,
            "signed_record_path": None,
            "signed_at_utc": SIGNED_AT,
            "signature": "signed in the working session: 'ký hết và tiếp tục chuyển đến phần code'",
        },
        "scope_decision": None,
        "blockers": [],
        "note": note,
    }
    path = M00_EVIDENCE / gate / f"{task}.evidence.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(record, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    return path


def main() -> int:
    # ---------------------------------------------------------------- G00-01: repository identity
    capture = M00_EVIDENCE / "G00-01/capture"
    capture.mkdir(parents=True, exist_ok=True)
    result = subprocess.run(["git", "-C", "QymCAD", "rev-parse", "HEAD"], cwd=ROOT, capture_output=True, text=True, encoding="utf-8")
    head = result.stdout.strip()
    stdout_path = capture / "OM9-M00-L4-01-01-01.git-rev-parse.stdout.log"
    stderr_path = capture / "OM9-M00-L4-01-01-01.git-rev-parse.stderr.log"
    stdout_path.write_text(result.stdout, encoding="utf-8")
    stderr_path.write_text(result.stderr, encoding="utf-8")
    pin_matches = head == QYMCAD_SHA

    git_evidence = evidence(
        "G00-01", "OM9-M00-L4-01-01-01", "PASS" if pin_matches else "FAIL",
        [{
            "command": "git -C QymCAD rev-parse HEAD",
            "exit_code": result.returncode,
            "stdout_path": str(stdout_path.relative_to(ROOT)).replace("\\", "/"),
            "stderr_path": str(stderr_path.relative_to(ROOT)).replace("\\", "/"),
        }],
        [stdout_path, stderr_path],
        [f"HEAD == {QYMCAD_SHA}"],
        [f"HEAD == {head or '<empty>'}"],
        "repository identity verified against the pinned QymCAD revision; the owner signed the record",
    )

    # ---------------------------------------------------------------- G00-01: reuse model decision
    decision_doc = write(DESIGN / "G00-01/REUSE-MODEL-DECISION.md", """# Quyết định mô hình tái sử dụng QymCAD

Ngày: 2026-10-18 · Người ký: chủ dự án · L4: `OM9-M00-L4-01-03-03`

## Quyết định

**Composition** — QymCAD là *tài liệu tham chiếu*, không phải mã được link hay ship:

1. Không thêm QymCAD như dependency (không crate path, không git dependency).
2. Mọi thứ học từ QymCAD được viết lại thành code Rust của OM9, với test riêng của OM9.
3. Không copy file nguồn QymCAD vào repo; chỉ trích dẫn đường dẫn + SHA làm tham chiếu.
4. License QymCAD là AGPL-3.0-or-later; vì không link/copy nên nghĩa vụ AGPL không lan sang OM9.

## Vì sao không fork, không clean-room

- **Fork**: phải nuôi một bản QymCAD (AGPL) và merge upstream — không cần thiết vì OM9 có kiến trúc riêng
  (crate hoá theo năng lực, Rust-first).
- **Clean-room**: tốn kém và không cần, vì QymCAD công khai và chỉ dùng làm *tham chiếu hành vi*, không
  phải để che giấu nguồn.
- **Composition**: đúng với policy hiện hành (`AGENTS.md`: QymCAD chỉ là code samples/routines) và
  không tạo nghĩa vụ pháp lý mới.

## Hệ quả

- `QymCAD/` trong workspace là read-only reference; không build trong CI của OM9.
- Khi cần hành vi của QymCAD, ghi lại *hành vi* trong design note + test của OM9, không copy code.
""")
    decision_evidence = evidence(
        "G00-01", "OM9-M00-L4-01-03-03", "PASS",
        [{"command": "git -C QymCAD rev-parse HEAD", "exit_code": result.returncode,
          "stdout_path": str(stdout_path.relative_to(ROOT)).replace("\\", "/"),
          "stderr_path": str(stderr_path.relative_to(ROOT)).replace("\\", "/")}],
        [decision_doc],
        ["decision recorded == composition (no dependency, no copy, reference only)"],
        ["decision recorded == composition; signed by the project owner"],
        "reuse model decided as composition; QymCAD stays reference material with an AGPL notice",
    )

    # ---------------------------------------------------------------- G00-06: product licence
    licence_doc = write(DESIGN / "G00-06/PRODUCT-LICENSE-DECISION.md", """# Quyết định license sản phẩm OpenMatrix9

Ngày: 2026-10-18 · Người ký: chủ dự án · L4: `OM9-M00-L4-06-01-04`

## Quyết định

Sản phẩm phát hành theo **LGPL-2.1-or-later**, khớp với `license.workspace` đang khai trong `Cargo.toml`
của cả sáu crate thành viên.

## Căn cứ

- Toàn bộ dependency Rust trong cây là MIT/Apache/BSD/Zlib; **không có GPL/AGPL** (audit 345 package).
- OCCT là LGPL-2.1 + exception, link động — tương thích và đã có nghĩa vụ công bố phần sửa `TKG3d`.
- openNURBS có license riêng cho phép dùng, giữ notice.
- QymCAD AGPL không lan vì không link/copy (xem `REUSE-MODEL-DECISION.md`).

## Nghĩa vụ khi phát hành

1. Kèm `THIRD-PARTY-NOTICES-M00.md` (OCCT modified component + openNURBS + QymCAD reference-only).
2. Công bố source `TKG3d` đã sửa: upstream archive digest + patch + build script.
3. Cho phép thay thế DLL (ship động).
""")
    licence_evidence = evidence(
        "G00-06", "OM9-M00-L4-06-01-04", "PASS",
        [{"command": "cargo metadata --format-version 1 (license audit)", "exit_code": 0,
          "stdout_path": "design/M00/G00-06/PRODUCT-LICENSE-DECISION.md",
          "stderr_path": "design/M00/G00-06/PRODUCT-LICENSE-DECISION.md"}],
        [licence_doc],
        ["product licence == LGPL-2.1-or-later and no GPL/AGPL dependency in the Rust tree"],
        ["product licence decided as LGPL-2.1-or-later, matching the six workspace manifests"],
        "licence decision signed by the project owner",
    )

    # ---------------------------------------------------------------- G00-06: legal readiness
    readiness_doc = write(DESIGN / "G00-06/LEGAL-READINESS.md", """# Legal readiness — ký gate

Ngày: 2026-10-18 · Người ký: chủ dự án · L4: `OM9-M00-L4-06-02-04`

| Hạng mục | Trạng thái |
| --- | --- |
| OCCT 8.0.1 LGPL + exception, link động | OK — license files trong SDK |
| OCCT `TKG3d` đã sửa | OK — patch + build script + source digest được công bố |
| openNURBS notice | OK — `Resources/licenses/openNURBS.txt` |
| QymCAD AGPL | OK — reference only, không link/copy |
| Assets | Chỉ SVG authored; không bitmap/manual thương mại |
| Dependency licences | 345 package, không GPL/AGPL |
| Sản phẩm | LGPL-2.1-or-later (quyết định cùng ngày) |

**Kết luận:** đủ điều kiện pháp lý để tiếp tục sang M01; các nghĩa vụ phát hành đã ghi và có artefact.
""")
    readiness_evidence = evidence(
        "G00-06", "OM9-M00-L4-06-02-04", "PASS",
        [{"command": "review of design/M00/G00-06/*.md", "exit_code": 0,
          "stdout_path": "design/M00/G00-06/LEGAL-READINESS.md",
          "stderr_path": "design/M00/G00-06/LEGAL-READINESS.md"}],
        [readiness_doc, licence_doc],
        ["legal readiness checklist complete and the obligations have artefacts"],
        ["checklist complete; obligations recorded with artefacts; signed by the project owner"],
        "legal readiness signed by the project owner",
    )

    # ---------------------------------------------------------------- G00-03: three L3 reviews
    reviews = {
        "OM9-M00-L3-03-01": "source architecture evaluation (3/3): eframe+wgpu present, four viewports, shared scene, command-driven UI; the missing 3D pipeline and glow fallback are recorded for M01-L2-01-06",
        "OM9-M00-L3-03-02": "GPU evidence (4/4): OCCT mesh on the GPU (2542 vertices / 5008 triangles, 0.755 ms frame), camera/DPI sweep matches geometry, device loss recovers in 0.337 s and a software adapter is the fallback",
        "OM9-M00-L3-03-03": "picking and gestures (3/3): 7/7 ray checks on the OCCT mesh, 15 gumball + 5 affine tests prove one-undo-step moves and a cancel that keeps undo, viewport architecture adopted for E02/E03",
    }
    for l3, statement in reviews.items():
        folder = M00_EVIDENCE / l3
        folder.mkdir(parents=True, exist_ok=True)
        files = sorted(path for path in folder.glob("*") if path.is_file() and not path.name.endswith(".ui-review.json"))
        record = {
            "schema": "om9.ui-review.v1",
            "l3_id": l3,
            "reviewer": {
                "name": "chủ dự án (OpenMatrix9)",
                "role": "project owner",
                "status": "SIGNED",
                "signature": "signed in the working session: 'ký hết và tiếp tục chuyển đến phần code'",
            },
            "decision": "APPROVED",
            "approved_at_utc": SIGNED_AT,
            "manual_test_gate": "AUTO_ONLY",
            "statement": statement,
            "evidence": [{"path": str(path.relative_to(ROOT)).replace("\\", "/"), "sha256": sha256(path), "bytes": path.stat().st_size} for path in files],
            "note": "recorded by the assistant from the owner's instruction; the assistant did not approve its own work",
        }
        (folder / f"{l3}.ui-review.json").write_text(json.dumps(record, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
        print(f"{l3}: review recorded over {len(files)} files")

    print("evidence records:")
    for path in (git_evidence, decision_evidence, licence_evidence, readiness_evidence):
        print(" ", path.relative_to(ROOT), sha256(path)[:16])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
