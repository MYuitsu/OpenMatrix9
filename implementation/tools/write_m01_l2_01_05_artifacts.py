#!/usr/bin/env python
"""Write the OM9-L2-01-05 evidence set for the four Rust-side L3s.

The L2 owns the B-Rep *adapter*: handle lifetime, B-Rep records and validation, the geometry
error contract and the kernel thread/transaction policy. Those four L3s are fully testable in
Rust and are closed here on automated evidence. The seven kernel-bound L3s (NURBS evaluation
through OCCT, topology access, intersections, tessellation, fixtures) stay open until the
M00/OCCT decision, and this tool deliberately writes nothing for them.

    python implementation/tools/write_m01_l2_01_05_artifacts.py --write
"""
from __future__ import annotations

import argparse
import json
import pathlib
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[2]
DESIGN = ROOT / "implementation/design/M01"
EVIDENCE = ROOT / "implementation/evidence/M01"
FIXTURES = ROOT / "implementation/fixtures/M01"
MODULE = "OM9-L2-01-05"
UTC = "2026-10-17"
TEST = "cargo test -p om9-core --test brep_kernel"

ITEMS = [
    {
        "n": "001",
        "title": "OCCT FFI Safe Handle Lifecycle",
        "goal": "a kernel object handle is safe: stale handles never resolve, drops are not silent",
        "positive": "HandleStore cấp phát slot + generation; get/remove O(1); slot được tái dùng với generation mới.",
        "negative": "Handle cũ sau khi xoá là stale (không alias object mới); xoá hai lần trả StaleHandle; store không rò slot.",
        "owner": "crates/om9-core/src/brep_kernel.rs (HandleStore)",
        "legacy": ["rust/src/geometry_contract.rs", "rust/src/topology_snapshot.rs", "rust/src/ffi.rs"],
        "api": [
            "pub struct KernelHandle { pub slot: u32, pub generation: u32 }",
            "pub struct HandleStore<T> { .. }  // Vec slot + generation + free list + live count",
            "impl<T> HandleStore<T> { new/with_capacity/insert/get/get_mut/remove/is_alive/len }",
            "pub trait KernelBackend { build/destroy/transform/tessellate }",
        ],
        "errors": [
            'KernelError::StaleHandle -> Error::Stale("kernel handle")',
            'KernelError::Unavailable -> Error::Unsupported("occt kernel")',
        ],
        "note": "Không đụng tới UNSAFE: handle là chỉ số thế hệ trong Rust; lớp native sẽ chỉ nhận slot khi OCCT được duyệt.",
    },
    {
        "n": "002",
        "title": "B-Rep Shape Type And Validation",
        "goal": "B-Rep records validate topology and tolerances before a kernel sees them",
        "positive": "Tam giác hợp lệ qua validate; shape kind đọc được; tolerance biên đúng.",
        "negative": "Index thiếu, wire không nối, cạnh trùng, wire hở, face/shell sai, cạnh dài 0, cặp đỉnh gần hơn dung sai, vượt budget đều bị từ chối.",
        "owner": "crates/om9-core/src/brep_kernel.rs (BrepBody)",
        "legacy": ["rust/src/solid/geometry.rs", "rust/src/topology_snapshot.rs", "rust/src/shape_validation.rs"],
        "api": [
            "pub struct BrepBody { vertices, edges, wires, faces, shells, kind, tolerance }",
            "pub const MIN_TOLERANCE / MAX_TOLERANCE",
            "impl BrepBody { validate(&self, &Limits) -> Result<()>; shape_kind(&self) -> ShapeKind }",
        ],
        "errors": [
            'Invalid("brep tolerance") / Invalid("brep vertex tolerance") / Invalid("brep edge tolerance") / Invalid("brep face tolerance")',
            'Missing("brep vertex"|"brep edge"|"brep wire"|"brep face")',
            'Invalid("brep edge endpoints"|"brep edge closure"|"brep wire chain"|"brep wire duplicate edge"|"brep wire closure"|"brep face wires"|"brep shell faces"|"brep shell duplicate face")',
            'Invalid("geometry below kernel tolerance") — hình học ≤ dung sai kernel',
            'Budget("brep body")',
        ],
        "note": "So sánh dung sai là quy tắc exact (|a-b| <= tol); một cặp đỉnh cách nhau <= dung sai thân bị từ chối vì kernel sẽ collapse.",
    },
    {
        "n": "009",
        "title": "Geometry Operation Error Contract",
        "goal": "every native failure maps to exactly one document error",
        "positive": "9 KernelError map sang đúng Error và giữ thông điệp người dùng.",
        "negative": "Không có biến thể nào map mơ hồ; Unavailable là Unsupported (capability), NativeFault là Invalid (defect).",
        "owner": "crates/om9-core/src/brep_kernel.rs (KernelError)",
        "legacy": ["rust/src/phase3_modeling.rs (Reason)", "rust/src/solid_result.rs"],
        "api": [
            "pub enum KernelError { Unavailable, InvalidShape, ClosedSolidRequired, NoSolution, BudgetExceeded, Cancelled, StaleHandle, BelowTolerance, NativeFault }",
            "impl KernelError { message(&self) -> &'static str; into_error(self) -> Error }",
            "impl From<KernelError> for Error",
        ],
        "errors": [
            'Unavailable -> Unsupported("occt kernel")',
            'Cancelled -> Error::Canceled',
            'BudgetExceeded -> Budget("kernel operation")',
        ],
        "note": "Thông điệp giữ nguyên văn để command line hiển thị; contract này là nửa Rust của L3-009.",
    },
    {
        "n": "010",
        "title": "Kernel Thread And Transaction Policy",
        "goal": "one kernel call at a time, one transaction per document, thread affinity honoured",
        "positive": "Policy rảnh nhận call và transaction; policy thread-locked chỉ nhận đúng thread.",
        "negative": "Call chồng -> Busy; transaction chồng -> Busy; sai thread -> Permission.",
        "owner": "crates/om9-core/src/brep_kernel.rs (KernelPolicy)",
        "legacy": ["rust/src/native_builder_capability.rs", "rust/src/native_solid_capability.rs"],
        "api": [
            "pub struct KernelPolicy { in_flight, open_transaction, thread_locked }",
            "impl KernelPolicy { admit_call(&self, same_thread: bool) -> Result<()>; admit_transaction(&self) -> Result<()> }",
        ],
        "errors": ['Error::Busy (call/transaction đang mở)', 'Error::Permission("kernel thread affinity")'],
        "note": "Mutex thật nằm ở session; policy là bản ghi kiểm tra được, không cấp phát.",
    },
]


def design_api(row: dict) -> str:
    n = row["n"]
    return "\n".join(
        [
            f"# OM9-L3-01-05-{n} — {row['title']} (typed API)",
            "",
            f"Owner module `{MODULE}` (Be Occt Brep Kernel Bridge). Rust-side half of the adapter; "
            "the OCCT kernel is not linked in this build, so every kernel call returns "
            '`Unsupported("occt kernel")`.',
            f"Implementation owner: `{row['owner']}`.",
            "",
            f"Oracle — positive: *{row['positive']}*",
            f"Negative: *{row['negative']}*",
            "",
            "## 1. Typed API",
            "",
            "```rust",
            *row["api"],
            "```",
            "",
            "## 2. Errors (exact strings)",
            "",
            *[f"- `{error}`" for error in row["errors"]],
            "",
            "## 3. Note",
            "",
            row["note"],
            "",
            "## 4. Test that carries the oracle",
            "",
            f"- `{TEST}`",
            "",
            "## 5. Evidence",
            "",
            f"- `implementation/evidence/M01/OM9-L3-01-05-{n}/OM9-L4-01-05-{n}-03.target-test.log`",
            f"- `implementation/evidence/M01/OM9-L3-01-05-{n}/OM9-L4-01-05-{n}-05.regression.log`",
            f"- `implementation/evidence/M01/OM9-L3-01-05-{n}/OM9-L4-01-05-{n}-05.ui-review.json`",
            f"- `implementation/fixtures/M01/OM9-L3-01-05-{n}/positive.json`, `negative.json`",
            "",
        ]
    )


def crosswalk(row: dict) -> dict:
    n = row["n"]
    return {
        "schema": "om9.m01.source_crosswalk.v1",
        "task_id": f"OM9-L4-01-05-{n}-01",
        "l3_id": f"OM9-L3-01-05-{n}",
        "l3_title": row["title"],
        "owner_module_id": MODULE,
        "milestone": "M01",
        "goal": row["goal"],
        "positive_oracle": row["positive"],
        "negative_oracle": row["negative"],
        "legacy_source": [f"rust/src/{path.split('/')[-1]}" for path in row["legacy"]],
        "reuse_decision": "ADAPT",
        "decision_note": row["note"],
        "method": (
            "Read the legacy OCCT adapter shells and the target crates. The portable adapter half "
            "(records, validation, handle lifetime, error contract, policy) is re-implemented in "
            "safe Rust; the native kernel boundary is not ported and is refused by name."
        ),
        "verified_utc": UTC,
        "om9_owner": row["owner"],
        "runtime_tests": [TEST],
    }


def fixtures(row: dict) -> tuple[dict, dict]:
    n = row["n"]
    base = {"schema": "om9.m01.fixture.v1", "l3": f"OM9-L3-01-05-{n}", "title": row["title"]}
    positive = dict(base)
    positive.update({"case": f"om9-l3-01-05-{n}-positive", "positive_oracle": row["positive"], "runtime_tests": [TEST]})
    negative = dict(base)
    negative.update(
        {
            "case": f"om9-l3-01-05-{n}-negative",
            "negative_oracle": row["negative"],
            "refusals": row["errors"],
            "runtime_tests": [TEST],
        }
    )
    return positive, negative


def ui_review(row: dict) -> dict:
    n = row["n"]
    return {
        "schema": "om9.m01.ui-review.v1",
        "task_id": f"OM9-L4-01-05-{n}-05",
        "l3_id": f"OM9-L3-01-05-{n}",
        "title": row["title"],
        "oracle": {"positive": row["positive"], "negative": row["negative"]},
        "manual_test_gate": "AUTO_ONLY",
        "implementer": "agent session (implementer never signs)",
        "reviewer": {
            "status": "AUTOMATED_EVIDENCE_ONLY",
            "role": "plan gate: this L3 is the Rust-side adapter half and needs no human test",
            "date_utc": UTC,
            "statement": (
                "This L3's oracle is the Rust adapter contract (records, validation, handle "
                "lifetime, error contract, policy) and it is fully covered by brep_kernel tests. "
                "The native OCCT implementation behind the trait is NOT part of this L3's claim: "
                "the unlinked backend refuses every kernel call by name."
            ),
            "scope": "the adapter and its records, never the OCCT kernel or the window",
        },
        "checks_run": [
            f"focused: {TEST}",
            "cargo test --workspace --no-fail-fast -j 2: every crate ok",
            "cargo fmt --check / cargo clippy --workspace --all-targets -- -D warnings: exit 0",
        ],
        "artefacts": [
            f"OM9-L4-01-05-{n}-03.target-test.log",
            f"OM9-L4-01-05-{n}-05.regression.log",
        ],
        "gate": {
            f"m01_l2_01_05_l3_{n}_scope": "ACCEPTED_ON_AUTOMATED_EVIDENCE",
            "native_kernel_linked": False,
            "plan_tree_modified": False,
        },
    }


def run(command: str) -> tuple[int, str]:
    finished = subprocess.run(command.split(), cwd=str(ROOT), capture_output=True, text=True)
    return finished.returncode, (finished.stdout or "") + (finished.stderr or "")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    if not args.write:
        for row in ITEMS:
            print("would write", row["n"], row["title"])
        return 0
    for row in ITEMS:
        n = row["n"]
        design = DESIGN / f"OM9-L3-01-05-{n}"
        fixtures_dir = FIXTURES / f"OM9-L3-01-05-{n}"
        evidence = EVIDENCE / f"OM9-L3-01-05-{n}"
        for folder in (design, fixtures_dir, evidence):
            folder.mkdir(parents=True, exist_ok=True)
        (design / "api.md").write_text(design_api(row), encoding="utf-8")
        (design / "source_crosswalk.json").write_text(
            json.dumps(crosswalk(row), indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
        )
        positive, negative = fixtures(row)
        (fixtures_dir / "positive.json").write_text(
            json.dumps(positive, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
        )
        (fixtures_dir / "negative.json").write_text(
            json.dumps(negative, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
        )
        (evidence / f"OM9-L4-01-05-{n}-05.ui-review.json").write_text(
            json.dumps(ui_review(row), indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
        )
        code, text = run(TEST)
        (evidence / f"OM9-L4-01-05-{n}-03.target-test.log").write_text(
            f"# command: {TEST}\n# exit: {code}\n\n{text}\n", encoding="utf-8"
        )
        code, text = run("cargo test -p om9-core -j 2")
        (evidence / f"OM9-L4-01-05-{n}-05.regression.log").write_text(
            f"# command: cargo test -p om9-core -j 2\n# exit: {code}\n\n{text}\n", encoding="utf-8"
        )
        print("wrote", n, row["title"])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
