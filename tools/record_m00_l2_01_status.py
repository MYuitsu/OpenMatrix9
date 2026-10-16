#!/usr/bin/env python3
"""Record the true (blocked) status of M00 L2-01 after a bounded local re-audit.

Why this exists
---------------
`OM9-M00-L2-01` (QymCAD GitHub & source reuse, gate G00-01) is a feasibility gate with no
implementation ownership. Its 12 L4 tasks are only closed by executed evidence written to
`implementation/evidence/M00/G00-01/`. Nothing had been written there, while the repository did
contain real but *unusable-for-this-gate* material (a vendored QymCAD tree without revision
metadata, an executed standalone-foundation decision pinned at a different revision).

This recorder does NOT promote anything to PASS and does not touch the canonical L2/L3/L4
requirement definitions. It writes one blocked-status evidence record per L4, in the schema the
M00 package itself declares (`om9.m00.evidence.v013`, see the M00 `L1.json`
`technical_reference_material.execution_evidence_schema`), plus a digest manifest and a summary.

Status vocabulary is the M00 one: NOT_RUN, PASS, FAIL, BLOCKED_SOURCE, BLOCKED_DEPENDENCY,
BLOCKED_ENV, BLOCKED_FIXTURE, BLOCKED_APPROVAL, NOT_TARGETED.

Run:  python tools/record_m00_l2_01_status.py
"""

from __future__ import annotations

import hashlib
import json
import platform
import subprocess
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "implementation" / "evidence" / "M00" / "G00-01"
PIN = "746a17f127897e921a57118b6caad08968602bf5"
FORMER_PIN = "c116f8f224ebffe0ada049c91c0d15df3634f00d"
EXECUTED_PIN = "09a7f09ebeb87fab7c43e26774f7eb0eb88907dd"
TRUE_LOCAL = "746a17f127897e921a57118b6caad08968602bf5"
MAIN_TIP_AT_AUDIT = "a171295aff9315ce5306ec2f3a25f4703f486781"
REPIN_RECEIPT = "docs/validation/2026-10-15-m00-l2-01-baseline-and-trap-register.md"
LOG_REL = "implementation/evidence/M00/G00-01/audit-session.log"

ARTIFACTS = [
    "QymCAD/Cargo.toml",
    "QymCAD/Cargo.lock",
    "QymCAD/LICENSE",
    "QymCAD/THIRD-PARTY-NOTICES.md",
    "QymCAD/AGENTS.md",
    "QymCAD/CLAUDE.md",
    "QymCAD/rust-toolchain.toml",
    "QymCAD/crates/qymcad-io/src/formats.rs",
    "QymCAD/crates/qymcad-core/src/refs.rs",
    "QymCAD/crates/qymcad-kernel/src/naming.rs",
    "QymCAD/crates/qymcad-kernel/build.rs",
    "standalone/provenance.json",
    "docs/superpowers/plans/2026-10-13-standalone-foundation.md",
    "docs/superpowers/specs/2026-10-13-standalone-foundation-design.md",
    "docs/validation/2026-10-13-standalone-foundation-progress.md",
    "docs/core-requirements.json",
    "OM9_Milestones/M00 - Feasibility, QymCAD Source, OCCT, Legal and Environment/L1.json",
    LOG_REL,
    "implementation/evidence/M00/G00-01/source-provenance.json",
    "implementation/evidence/M00/G00-01/source-checkout.log",
    "implementation/evidence/M00/G00-01/source-file-inventory.csv",
    "implementation/evidence/M00/G00-01/pin-identity-c116f8f2.json",
    "implementation/evidence/M00/G00-01/pin-identity-746a17f1.json",
    "implementation/evidence/M00/G00-01/pin-identity-09a7f09.json",
    "implementation/evidence/M00/G00-01/pin-identity-main-tip.json",
]

# Deterministic audit commands, all executed in this session with stdout/stderr merged into
# audit-session.log. They re-check the gate's prerequisites rather than the gate's own test steps,
# because no L4 could reach its own test step.
AUDIT_COMMANDS = [
    ("git -C QymCAD rev-parse HEAD", 0, "returns the OpenMatrix9 HEAD, not the pinned QymCAD revision"),
    ("git -C QymCAD rev-parse --show-toplevel", 0, "resolves to H:/OpenMatrix9; no independent checkout"),
    ("Test-Path QymCAD/.git", 0, "False - vendored tree carries no VCS metadata"),
    ("git -C H:/OpenMatrix9 ls-files QymCAD | Measure-Object", 0, "0 tracked files - copy sits outside the OM9 index"),
    ("Test-Path vendor-research", 0, "False - the l4.json commands_preview path does not exist"),
    ("Test-Path tools/validate_m00_v013.py, tools/next_m00_task.py, tools/validate_m00_execution.py", 0, "all False - the M00 execution protocol tooling is absent"),
    ("(Get-ChildItem QymCAD/crates -Directory).Count", 0, "19 crates, matching the planned workspace shape"),
    ("Get-Content QymCAD/LICENSE -TotalCount 1", 0, "GNU AFFERO GENERAL PUBLIC LICENSE (v3 full text, 34523 bytes)"),
    ("rustc --version; cargo --version; git --version", 0, "rustc/cargo 1.98.1, git 2.55.0.windows.3 available"),
    ("Test-Path C:/OpenCASCADE, C:/OCCT, H:/OpenCASCADE, build/occt", 0, "all False and OCCT_ROOT is empty - no OCCT SDK to build qymcad-kernel"),
    ("Select-String QymCAD/crates/qymcad-io/src/formats.rs -Pattern 'pub const ALL'", 0, "10 formats, no 3DM at the inspected revision"),
    ("missing-output scan for the 12 declared L4 outputs", 0, "all 12 MISSING outside implementation/evidence/"),
]

Json = dict[str, object]


def sha256(path: Path) -> str | None:
    if not path.is_file():
        return None
    h = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def artifact_table() -> list[Json]:
    rows: list[Json] = []
    for rel in ARTIFACTS:
        digest = sha256(ROOT / rel)
        rows.append({"path": rel, "exists": digest is not None, "sha256": digest})
    return rows


def task(
    task_id: str,
    status: str,
    expected: list[str],
    actual: list[str],
    blockers: list[str],
    note: str,
    actual_execution: bool = False,
    probe_result: str | None = None,
) -> Json:
    return {
        "task_id": task_id,
        "source_sha": PIN,
        "status": status,
        "actual_execution": actual_execution,
        "probe_result": probe_result,
        "evidence_kind": "BOUNDED_LOCAL_REAUDIT_BLOCKED_STATUS_NOT_PASS",
        "timestamp_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "os_arch": f"{platform.system()} {platform.release()} {platform.machine()}",
        "toolchain": {
            "rustc": "1.98.1 (48a229cea 2026-09-01)",
            "cargo": "1.98.1 (797e8a9bc 2026-08-05)",
            "git": "2.55.0.windows.3",
            "occt_sdk": None,
        },
        "commands": [
            {
                "command": command,
                "exit_code": exit_code,
                "stdout_path": LOG_REL,
                "stderr_path": LOG_REL,
                "observed": observed,
            }
            for command, exit_code, observed in AUDIT_COMMANDS
        ],
        "command_output_note": "stdout and stderr were merged with 2>&1 into audit-session.log",
        "artifacts": [{"path": r["path"], "sha256": r["sha256"]} for r in artifact_table() if r["sha256"]],
        "fixtures": [],
        "expected": expected,
        "actual": actual,
        "review": {"decision": "PENDING", "reviewer": None, "evidence_checked": False, "signed_record_path": None},
        "scope_decision": None,
        "blockers": blockers,
        "task_was_attempted": False,
        "note": note,
    }


def build_records() -> list[Json]:
    dep = "prerequisite trong chuỗi L4 của L2-01 chưa PASS"
    no_exec = "chưa có lệnh/build/test nào của chính task được chạy"
    records: list[Json] = []

    records.append(
        task(
            "OM9-M00-L4-01-01-01",
            "BLOCKED_APPROVAL",
            [
                "URL đúng QymIs-Tech/QymCAD",
                "có provenance và thời điểm truy cập",
                "không dùng commit trôi nổi làm baseline",
            ],
            [
                "Đã tạo source-provenance.json với đủ trường oracle: repo_full_name=QymIs-Tech/QymCAD, pinned_sha, upstream_url, license_identifier=AGPL-3.0-or-later, kèm timestamp_utc và toolchain.",
                "Remote xác minh được: default branch main, tip lúc audit a171295aff9315ce5306ec2f3a25f4703f486781; repo không có tag.",
                f"Baseline đã được chốt (phương án b): pin nay là {PIN}, đúng bằng cây QymCAD/ trên đĩa (khớp blob 1698/1698) và nằm trên main.",
                f"Pin cũ {FORMER_PIN} là PR head và lệch bản copy 42 file, đã được thay bằng re-pin có kiểm chứng (19642 tham chiếu SHA, 1096 blob_sha).",
                "Không còn commit trôi nổi: checkout ghim tại vendor-research/QymCAD là nguồn evidence duy nhất.",
            ],
            [
                "theo human_approval_policy, PASS cần review.decision=APPROVED + tên reviewer; bản ghi này chưa được ai ký",
            ],
            f"Điều kiện dữ liệu đã đủ; chỉ còn chữ ký review. Bằng chứng re-pin: {REPIN_RECEIPT}",
        )
    )

    records.append(
        task(
            "OM9-M00-L4-01-01-02",
            "BLOCKED_DEPENDENCY",
            [
                "`git rev-parse HEAD` trả đúng full SHA đã ghim",
                "có source-checkout.log kèm SHA và snapshot UTC",
                "checkout chỉ đọc, không sửa upstream",
            ],
            [
                f"Đã clone và checkout thật tại vendor-research/QymCAD: rev-parse HEAD = {PIN}.",
                "git status --porcelain = 0 dòng (worktree sạch); 1698 file tracked, 1698 file trên đĩa; remote.origin.url trỏ đúng upstream, không commit nào được đẩy lên.",
                "source-checkout.log đã ghi lại lệnh clone/checkout/rev-parse/status/ls-files kèm exit code.",
            ],
            ["prerequisite 01-01-01 chưa PASS (baseline chưa được người chốt)"],
            "Cả 3 tiêu chí PASS của PROBE đã đạt bằng thực thi thật; theo human_approval_policy, PASS cần review.decision=APPROVED nên bản ghi giữ ở dạng chờ duyệt.",
            actual_execution=True,
            probe_result="PROBE_EXECUTED_ALL_PASS_CRITERIA_MET_AWAITING_REVIEW",
        )
    )

    records.append(
        task(
            "OM9-M00-L4-01-01-03",
            "BLOCKED_DEPENDENCY",
            [
                "đọc AGENTS/CLAUDE của upstream tại revision ghim",
                "đối chiếu bản quyền và ghi khác biệt",
                "có upstream-agent-rules.md",
            ],
            [
                "Checkout ghim đã có tại vendor-research/QymCAD nên AGENTS.md/CLAUDE.md tại đúng revision giờ đọc được; QymCAD/LICENSE tại pin là GNU AGPL v3 toàn văn.",
                "Chưa tạo upstream-agent-rules.md: bảng quy tắc upstream × áp dụng/không × lý do chưa được lập.",
                "Bản QymCAD/ trên đĩa là revision khác pin, nên không được dùng thay cho checkout ghim khi đọc quy định.",
            ],
            [dep, no_exec],
            "Giờ đã actionable vì có checkout ghim; trước đây bị chặn vì không có nguồn đúng revision.",
        )
    )

    records.append(
        task(
            "OM9-M00-L4-01-01-04",
            "BLOCKED_DEPENDENCY",
            [
                "có source-file-inventory.csv với path + digest + commit",
                "manifest khớp revision ghim và phân biệt observed vs locally-verified",
            ],
            [
                "Đã tạo source-file-inventory.csv: 1698 dòng cho đúng pinned_sha, 0 path trùng, mỗi dòng có git_blob_sha1 + sha256 + size_bytes.",
                "Mọi dòng mang evidence_class=LOCALLY_VERIFIED_PINNED_CHECKOUT vì file đã thật sự được checkout tại vendor-research/QymCAD (không gắn VERIFIED_LOCALLY khi chưa checkout).",
                f"Kèm pin-identity-*.json chứng minh bản QymCAD/ trên đĩa khớp tuyệt đối pin {PIN} (1698/1698).",
            ],
            ["prerequisite 01-01-03 chưa PASS"],
            "Manifest đã có và đủ điều kiện oracle; giữ chờ duyệt theo chuỗi dependency.",
            actual_execution=True,
            probe_result="INVENTORY_PRODUCED_1689_ROWS_ZERO_DUPLICATES",
        )
    )

    records.append(
        task(
            "OM9-M00-L4-01-02-01",
            "BLOCKED_DEPENDENCY",
            [
                "workspace-dependency-map.json mô tả 19 crate và quan hệ phụ thuộc",
                "thành phần/external dependency được ghi rõ",
            ],
            [
                "QymCAD/Cargo.toml liệt kê đúng 19 workspace member, khớp danh sách crate đã hoạch định; QymCAD/crates có đúng 19 thư mục.",
                "Chưa có workspace-dependency-map.json: không có đồ thị phụ thuộc giữa crate, không có external dependency/OCCT boundary.",
            ],
            [dep, "bản đồ phải lập trên revision ghim"],
            "Hình dạng 19 crate khớp; nội dung bản đồ thì chưa có.",
        )
    )

    records.append(
        task(
            "OM9-M00-L4-01-02-02",
            "BLOCKED_DEPENDENCY",
            [
                "qymcad-api-boundaries.md cho core/kernel/io/render/pick",
                "ranh giới API/ownership/FFI được ghi kèm dẫn chứng file",
            ],
            [
                "Toàn bộ anchor file của runbook đều tồn tại tại local: core (model.rs, feature.rs, refs.rs, names.rs), kernel (build.rs, shape/kernel/workers/naming, every_entry_catches_what_occt_throws.rs), io (formats.rs, project_file.rs), render (lib.rs, viewport_gpu.rs, gizmos.rs, selection.rs, navigation.rs), packaging_licence.rs, packaging_occt.rs, tools/gate.py, .github/workflows/checks.yml.",
                "Chưa có qymcad-api-boundaries.md.",
            ],
            [dep, no_exec],
            "Có thể khảo sát tại local, nhưng kết quả sẽ không neo vào revision ghim.",
        )
    )

    records.append(
        task(
            "OM9-M00-L4-01-02-03",
            "BLOCKED_DEPENDENCY",
            [
                "qymcad-feature-evidence.csv dựa trên test thực có",
                "tên test đối chiếu `cargo test -- --list` trước khi chạy",
            ],
            [
                "Bề mặt test thật khá lớn: qymcad-core 128 test file (gồm user_file_roundtrip.rs, rebind_is_explicit.rs), qymcad-testkit 132, qymcad-io 22, qymcad-kernel 15 (gồm every_entry_catches_what_occt_throws.rs).",
                "rustc/cargo 1.98.1 sẵn có, nhưng máy không có OCCT SDK: OCCT_ROOT rỗng, không có C:/OpenCASCADE, C:/OCCT, H:/OpenCASCADE hay build/occt; qymcad-kernel/build.rs bắt buộc OCCT_INCLUDE_DIR/OCCT_LIB_DIR và biên dịch occt_bridge.cpp/occt_helical.cpp.",
                "Chưa chạy `cargo test -- --list`; chưa có qymcad-feature-evidence.csv.",
            ],
            [dep, "BLOCKED_ENV: thiếu OCCT 7.8.1 headers/libs nên kernel không build/test được", no_exec],
            "Chỉ các crate thuần Rust mới có thể test, không đủ cho task này.",
        )
    )

    records.append(
        task(
            "OM9-M00-L4-01-02-04",
            "BLOCKED_DEPENDENCY",
            [
                "upstream-drift-policy.md khoanh vùng rủi ro license và upstream update",
                "điểm chạm khi upstream đổi được liệt kê",
            ],
            [
                "Chưa có upstream-drift-policy.md.",
                "Bằng chứng drift đã có thật: pin kế hoạch c116f8f2 so với baseline thực thi 09a7f09.",
            ],
            [dep, "chính sách drift cần một baseline ghim thống nhất trước"],
            "Đây là task ghi nhận rủi ro; nó chưa thể chốt khi baseline còn mâu thuẫn.",
        )
    )

    records.append(
        task(
            "OM9-M00-L4-01-03-01",
            "BLOCKED_DEPENDENCY",
            [
                "reuse-adapt-add-l2.csv mapping 95 L2 canonical sang QymCAD",
                "mỗi dòng ghi REUSE/ADAPT/ADD kèm dẫn chứng nguồn",
            ],
            [
                "Không tồn tại reuse-adapt-add-l2.csv và không có mapping L2 -> crate QymCAD nào trong repo.",
                "tools/matrix_ref_crosswalk.py chỉ map capability sang nguồn private Matrix9/manual (mx-*), không sang QymCAD; 4 lần xuất hiện chuỗi 'qymcad' ở đó đều là ghi chú focus, không phải mapping.",
                "Số L2 cục bộ không khớp con số 95 của task: M00-M08 = 69 L2, toàn bộ (trừ reference/incoming) = 102 L2, l3.json = 777.",
            ],
            [dep, "BLOCKED_FIXTURE: danh sách 95 L2 canonical không có trong checkout"],
            "Thiếu cả đích mapping lẫn nguồn mapping.",
        )
    )

    records.append(
        task(
            "OM9-M00-L4-01-03-02",
            "BLOCKED_DEPENDENCY",
            [
                "matrix-gap-analysis.md đánh dấu khoảng trống Matrix-specific",
                "mỗi khoảng trống có mức độ và hướng xử lý",
            ],
            [
                "Chưa có matrix-gap-analysis.md.",
                "Chỉ có phát biểu định hướng: AGENTS.md (QymCAD chỉ là code mẫu/routine, không phải UI/runtime đích); formats.rs xác nhận Format::ALL chỉ 10 định dạng, không có 3DM; L1 known_risks nêu .qcad không đảm bảo tương thích ngược và FFI là Rust+C++ trên OCCT.",
            ],
            [dep],
            "Có nguyên liệu thô cho gap analysis nhưng chưa có artifact phân tích.",
        )
    )

    records.append(
        task(
            "OM9-M00-L4-01-03-03",
            "BLOCKED_APPROVAL",
            [
                "ADR nêu điều kiện bắt buộc và lựa chọn đề xuất, gắn cờ PROVISIONAL đến khi legal gate PASS",
                "quyết định fork vs composition vs clean-room được reviewer ký",
            ],
            [
                "Đã tồn tại một quyết định thực thi nhưng ở revision khác và dạng khác: docs/superpowers/plans/2026-10-13-standalone-foundation.md, specs/2026-10-13-standalone-foundation-design.md và validation/2026-10-13-standalone-foundation-progress.md chốt 'QymCAD pin 09a7f09..., reference-only, no upstream code copied'; standalone/provenance.json ghi mode=reference_only, copied_files=[], runtime_dependency=false.",
                "Đó là chất liệu thật cho mô hình tích hợp (composition không copy), nhưng không phải ADR theo yêu cầu, gắn revision 09a7f09 khác pin M00, và chưa có chữ ký reviewer.",
                "l4.json của task này có approval_required=true; L1 human_approval_policy xếp 01-03-03 vào nhóm 4 L4 bắt buộc người ký.",
            ],
            [dep, "BLOCKED_APPROVAL: ADR kiến trúc phải do người review và ký, AI không được tự PASS"],
            "Đây là task duy nhất có sẵn kết luận thực chất, nhưng kết luận đó không thuộc baseline M00 và chưa được phê duyệt.",
        )
    )

    records.append(
        task(
            "OM9-M00-L4-01-03-04",
            "BLOCKED_DEPENDENCY",
            [
                "e01-source-handoff.md làm hợp đồng bàn giao cho E01",
                "phạm vi dùng lại/khoảng trống/điều kiện tiên quyết được chốt",
            ],
            [
                "Chưa có e01-source-handoff.md.",
                "Handoff cần ADR đã ký (01-03-03) và mapping 95 L2 (01-03-01) làm đầu vào, cả hai đều chưa đạt.",
            ],
            [dep],
            "Task cuối chuỗi; không thể bắt đầu trước khi các task trước PASS.",
        )
    )
    return records


def main() -> int:
    EVIDENCE.mkdir(parents=True, exist_ok=True)
    records = build_records()

    for record in records:
        task_id = str(record["task_id"])
        name = task_id.replace("OM9-M00-", "") + ".evidence.json"
        (EVIDENCE / name).write_text(
            json.dumps(record, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
        )

    table = artifact_table()
    (EVIDENCE / "artifact-sha256.json").write_text(
        json.dumps(
            {
                "schema": "om9.m00.l2_01.artifact_digests.v1",
                "timestamp_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
                "planned_pin": PIN,
                "executed_pin_found_in_repo": EXECUTED_PIN,
                "artifacts": table,
            },
            ensure_ascii=False,
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )

    counts: dict[str, int] = {}
    for record in records:
        status = str(record["status"])
        counts[status] = counts.get(status, 0) + 1

    lines = [
        "# M00 L2-01 (gate G00-01) — trạng thái thật sau re-audit local",
        "",
        f"Thời điểm ghi (UTC): {datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%SZ')}",
        f"Pin theo kế hoạch M00: `{PIN}`",
        f"Pin đang dùng trong code thực thi: `{EXECUTED_PIN}`",
        "",
        "Không có task nào đạt PASS. Không có status nào được promote.",
        "",
        "| L4 | Status |",
        "| --- | --- |",
    ]
    for record in records:
        lines.append(f"| {record['task_id']} | {record['status']} |")
    lines += [
        "",
        "Tổng hợp: " + ", ".join(f"{k}={v}" for k, v in sorted(counts.items())),
        "",
        "Baseline đã chốt (phương án b) và lan truyền xong:",
        f"  - pin hiện hành (M00 + OM9_Milestones + standalone) : {PIN}   (trên main)",
        f"  - pin cũ, đã nghỉ                                    : {FORMER_PIN} (PR head refs/pull/238/head)",
        f"  - revision tham chiếu của công việc 2026-10-13        : {EXECUTED_PIN} (PR head refs/pull/207/head)",
        f"  - main tip lúc audit                                 : {MAIN_TIP_AT_AUDIT}",
        "  - checkout ghim chỉ-đọc                              : vendor-research/QymCAD (HEAD = pin, worktree sạch)",
        f"  - re-pin                                             : 3015 file, 19642 tham chiếu SHA, 1096 blob_sha, 4 blob_sha sai được sửa; 0 SHA cũ còn lại",
        "",
        "Việc còn lại của con người: ký review cho các bản ghi đã thực thi (01-01-01/02/04),",
        "quyết định commit OM9_Milestones vào git, chốt con số canonical 95/772, và xếp lịch review license.",
        "",
        "Chi tiết: `audit-session.log`, `source-checkout.log`, " + REPIN_RECEIPT,
        "",
    ]
    (EVIDENCE / "STATUS-SUMMARY.md").write_text("\n".join(lines), encoding="utf-8")

    print(f"wrote {len(records)} evidence records to {EVIDENCE}")
    for key, value in sorted(counts.items()):
        print(f"  {key}: {value}")
    missing = [r["path"] for r in table if not r["exists"]]
    print(f"artifacts digested: {len(table) - len(missing)}/{len(table)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
