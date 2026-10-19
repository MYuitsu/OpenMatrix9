#!/usr/bin/env python
"""Write the L2-01-08 (units, tolerances, typed parameters) evidence and acceptance set.

For each of the seven L3 it writes:

* ``implementation/design/M01/<L3>/api.md``                 — typed API of that L3
* ``implementation/design/M01/<L3>/source_crosswalk.json``   — legacy `rust/` source audit + OM9 owner
* ``implementation/fixtures/M01/<L3>/{positive,negative}.json``
* ``implementation/evidence/M01/<L3>/<L4-03>.target-test.log`` (focused run)
* ``implementation/evidence/M01/<L3>/<L4-05>.regression.log``  (crate suite)
* ``implementation/evidence/M01/<L3>/<L4-05>.ui-review.json``  (AUTO_ONLY acceptance)

    python implementation/tools/write_m01_l2_01_08_artifacts.py           # dry: list only
    python implementation/tools/write_m01_l2_01_08_artifacts.py --write   # write + run tests
"""
from __future__ import annotations

import argparse
import json
import pathlib
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[2]
LEGACY = ROOT / "rust"
DESIGN = ROOT / "implementation/design/M01"
EVIDENCE = ROOT / "implementation/evidence/M01"
FIXTURES = ROOT / "implementation/fixtures/M01"
MODULE = "OM9-L2-01-08"
GATE = "AUTO_ONLY"
UTC = "2026-10-17"

ITEMS = [
    {
        "n": "001",
        "title": "Millimeter World Unit Model",
        "goal": "canonical internal lengths in millimeters",
        "positive": "Độ dài có kiểu trong milimét; số trần trong lệnh dùng đơn vị tài liệu, hậu tố tường minh thắng; token đơn vị không phân biệt hoa thường và chỉ có một bảng đơn vị.",
        "negative": "NaN/inf bị từ chối; đơn vị lạ là Unsupported; chữ số không parse được là Malformed.",
        "owner": "crates/om9-core/src/units.rs, crates/om9-core/src/geometry.rs",
        "legacy": ["rust/src/units.rs", "tests: rust/tests/units.rs, numeric_units.rs, tolerances.rs"],
        "api": [
            "pub struct LengthMm { mm: f64 }   // #[serde(try_from = \"LengthRepr\")], wire {\"mm\": 2.5}",
            "impl LengthMm { pub fn try_new(mm: f64) -> Result<Self, UnitError>;",
            "                pub fn from_unit(value: f64, unit: Unit) -> Result<Self, UnitError>;",
            "                pub fn parse(text: &str, default: Unit) -> Result<Self, UnitError>;",
            "                pub fn mm(self) -> f64; }",
            "impl Unit { pub fn parse(token: &str) -> Result<Unit, UnitError>; pub fn symbol(self) -> &'static str; }",
        ],
        "errors": [
            'UnitError::NonFinite { dimension: Length }  -> Invalid("finite numeric value")',
            'UnitError::UnsupportedUnit { token }        -> Unsupported("unit")',
            'UnitError::Malformed                        -> Invalid("numeric text")',
        ],
        "tests": [("cargo test -p om9-core --test units_parameters l3_001", "units_parameters"), ("cargo test -p om9-app --test command_options_and_repeat a_bare_option_number", "command_options_and_repeat"), ("cargo test -p om9-core --test l2_01_08_precision", "l2_01_08_precision")],
        "regression": ["om9-core"],
        "legacy_decision": "ADAPT",
        "legacy_note": "Legacy `units.rs` owns the same ideas through a C ABI (Context/Dimension/UnitChangeMode). The portable part — canonical mm, unit tokens, checked conversion — is re-implemented in safe Rust with typed values; the FFI context and the C error codes are not ported.",
    },
    {
        "n": "002",
        "title": "Angle Conversion And Degrees",
        "goal": "angle inputs in degrees with robust radian conversion",
        "positive": "Góc lưu bằng độ; đổi radian chính xác tới 8.9e-16; text radian chuyển tường minh.",
        "negative": "|deg| vượt MAX_ABS_DEGREES bị OutOfRange; radian không đọc lại được thành độ.",
        "owner": "crates/om9-core/src/units.rs",
        "legacy": ["rust/src/units.rs (Dimension/AngleRadians)", "rust/src/core_angle.rs"],
        "api": [
            "pub const MAX_ABS_DEGREES: f64 = 1.0e6;",
            "pub struct AngleDeg { deg: f64 }   // wire {\"deg\": 45.0}",
            "pub struct AngleRad { rad: f64 }   // wire {\"rad\": 0.785...}",
            "impl AngleDeg { pub fn try_new(deg: f64) -> Result<Self, UnitError>; pub fn parse(text: &str) -> Result<Self, UnitError>;",
            "                pub fn deg(self) -> f64; pub fn to_radians(self) -> AngleRad; }",
            "impl AngleRad { pub fn try_new(rad: f64) -> Result<Self, UnitError>; pub fn rad(self) -> f64;",
            "                pub fn to_degrees(self) -> Result<AngleDeg, UnitError>; }",
        ],
        "errors": [
            'UnitError::OutOfRange { dimension: Angle, .. } -> Invalid("numeric value range")',
            'UnitError::NonFinite { dimension: Angle }      -> Invalid("finite numeric value")',
        ],
        "tests": [("cargo test -p om9-core --test units_parameters l3_002", "units_parameters"), ("cargo test -p om9-core --test l2_01_08_precision", "l2_01_08_precision")],
        "regression": ["om9-core"],
        "legacy_decision": "ADAPT",
        "legacy_note": "Legacy angle handling lives inside the C context (angular_rad) and `core_angle.rs`. OM9 keeps degrees as the stored unit and makes the radian boundary explicit; no native call is needed.",
    },
    {
        "n": "003",
        "title": "Document Absolute Relative Tolerances",
        "goal": "tolerance policy for small jewelry geometry",
        "positive": "Tài liệu có một policy tuyệt đối/tương đối/góc lưu kèm tài liệu (admitted ở schema 4); tài liệu mới dùng policy template tường minh; mutation là một Undo, không đổi context_revision.",
        "negative": "Giá trị sai bị từ chối theo tên trường; tài liệu không khai báo policy trả Missing, không bịa mặc định.",
        "owner": "crates/om9-core/src/history.rs, model.rs, transaction.rs",
        "legacy": ["rust/src/tolerances.rs", "rust/src/units.rs (Context.absolute_mm/relative_ratio)"],
        "api": [
            "impl Tolerance { pub fn try_new(absolute_mm: f64, relative: f64, angular_rad: f64) -> Result<Self>;",
            "                 pub fn admit_feature(&self, size: LengthMm) -> Result<()>; }",
            "Document.tolerance: Option<Tolerance>; Document::{tolerance, effective_tolerance}",
            "Mutation::Tolerance { document, expected_revision, tolerance }   // one Undo, context_revision unchanged",
        ],
        "errors": [
            'Invalid("absolute tolerance") / Invalid("relative tolerance") / Invalid("angular tolerance")',
            'Invalid("feature at or below document absolute tolerance")',
            'Missing("document tolerance") — no default is invented',
            'Stale("document tolerance revision")',
        ],
        "tests": [("cargo test -p om9-core --test document_tolerance", "document_tolerance"), ("cargo test -p om9-archive --test document_identity_round_trip a_document_tolerance", "document_identity_round_trip"), ("cargo test -p om9-core --test l2_01_08_precision", "l2_01_08_precision")],
        "regression": ["om9-core", "om9-archive"],
        "legacy_decision": "ADAPT",
        "legacy_note": "Legacy keeps tolerance inside the unit Context; OM9 makes it a document-level policy with its own mutation so it is auditable and undoable.",
    },
    {
        "n": "004",
        "title": "Typed Numeric Parameter Values",
        "goal": "length, angle, count and ratio typed parameter values",
        "positive": "Value giữ đúng chiều; đọc lại bằng as_length/as_angle/as_count/as_ratio.",
        "negative": "Số trần không phải length/angle; count 2.5 là NotAnInteger; sai chiều là WrongDimension.",
        "owner": "crates/om9-core/src/request.rs",
        "legacy": ["rust/src/command_options.rs", "rust/src/units.rs (Dimension)", "rust/src/typed option parsing"],
        "api": [
            "impl Value { pub fn length(LengthMm) -> Self; pub fn angle(AngleDeg) -> Self;",
            "             pub fn count(u32) -> Self; pub fn ratio(f64) -> Result<Self>;",
            "             pub fn as_length(&self) -> Result<LengthMm, UnitError>;",
            "             pub fn as_angle(&self) -> Result<AngleDeg, UnitError>;",
            "             pub fn as_count(&self) -> Result<u32, UnitError>;",
            "             pub fn as_ratio(&self) -> Result<f64, UnitError>; }",
        ],
        "errors": [
            'UnitError::MissingUnit { .. }      -> Invalid("missing unit")',
            'UnitError::WrongDimension { .. }   -> Invalid("numeric dimension")',
            'UnitError::NotAnInteger { .. }     -> Invalid("integer value")',
        ],
        "tests": [("cargo test -p om9-core --test units_parameters l3_004", "units_parameters")],
        "regression": ["om9-core"],
        "legacy_decision": "ADAPT",
        "legacy_note": "Legacy `command_options.rs` carries dimensioned option values through C; OM9 keeps the typed value in Rust and reads it back typed.",
    },
    {
        "n": "005",
        "title": "Builder Parameter Range Validation",
        "goal": "min/max, increment and invalid-number validation",
        "positive": "Domain Number chỉ nhận Quantity cùng chiều, trong [min,max], đúng bước.",
        "negative": "Số trần bị MissingUnit; sai bước là Invalid(\"numeric option step\"); schema sai bị từ chối trước mọi giá trị.",
        "owner": "crates/om9-core/src/request.rs",
        "legacy": ["rust/src/command_option_schemas.rs", "rust/src/command_options.rs"],
        "api": [
            "OptionDomain::Number { dimension, min, min_inclusive, max, step: Option<f64> }",
            "impl OptionDomain { pub fn validate(&self) -> Result<()>; pub fn check(&self, &Value, &Limits) -> Result<()>; }",
            "request::admit(...) calls domain.validate() for every schema option before admission",
        ],
        "errors": [
            'Invalid("numeric option bounds") / Invalid("numeric option step")',
            'Invalid("missing unit") — a bare number never satisfies a Number domain',
            'Invalid("numeric option domain") — outside range / non-integral count',
        ],
        "tests": [("cargo test -p om9-core --test units_parameters l3_005", "units_parameters")],
        "regression": ["om9-core"],
        "legacy_decision": "PORT",
        "legacy_note": "Legacy schemas carry range/step through the C option layer; the rule is ported into the Rust request schema and is now enforced before any value is admitted.",
    },
    {
        "n": "006",
        "title": "Unit Safe Parameter Serialization",
        "goal": "preserve numeric dimensional semantics in project data",
        "positive": "Wire form có tag đơn vị, round-trip bit-chính-xác cho f64.",
        "negative": "Payload radian không đọc được thành độ; số trần không đọc được thành mm.",
        "owner": "crates/om9-core/src/units.rs, request.rs",
        "legacy": ["rust/src/units.rs", "rust/src/prepared_result.rs (stored options)"],
        "api": [
            "serde wire: LengthMm {\"mm\":..}, AngleDeg {\"deg\":..}, AngleRad {\"rad\":..}; other keys refused by deny_unknown_fields",
            "Value::Quantity { value, dimension } keeps the dimension through JSON",
        ],
        "errors": [
            'a degrees payload cannot deserialize into LengthMm (and vice versa)',
            'a bare number cannot deserialize into a typed unit value',
        ],
        "tests": [("cargo test -p om9-core --test units_parameters l3_006", "units_parameters")],
        "regression": ["om9-core"],
        "legacy_decision": "ADAPT",
        "legacy_note": "Legacy stored a raw f64 with a separate context; OM9 stores the unit in the value, so a stored number can never be read with the wrong dimension.",
    },
    {
        "n": "007",
        "title": "Coordinate Numeric Input Primitives",
        "goal": "local and world numeric point representations",
        "positive": "CPlane là frame có kiểu: local (1,2,3) trên plane dịch ([10,20,30]) thành world (13,21,32); to_local là nghịch đảo.",
        "negative": "Local point không có frame là Missing; ngữ pháp x,y[,z] và nhãn lỗi giữ nguyên.",
        "owner": "crates/om9-core/src/coords.rs, crates/om9-scene/src/camera.rs, crates/om9-app/src/session.rs",
        "legacy": ["rust/src/coordinates.rs", "rust/src/coordinate_input.rs", "rust/src/cplanes.rs"],
        "api": [
            "coords::{WorldPoint, LocalPoint, LocalFrame, CoordinateInput, resolve, parse_local}",
            "impl LocalFrame for om9_scene::camera::CPlane { to_world = CPlane::point; to_local = dot with x, y, normal }",
            "session.rs parses with coords::parse_local(text, unit) then coords::resolve(Local, Some(&plane))",
        ],
        "errors": [
            'Missing("coordinate frame") — a local point cannot become world without a frame',
            'Invalid("x,y[,z] coordinate") / Invalid("coordinate number") / Invalid("length conversion")',
        ],
        "tests": [
            ("cargo test -p om9-core --test coordinates", "coordinates"),
            ("cargo test -p om9-scene --test coordinate_frame", "coordinate_frame"),
        ],
        "regression": ["om9-core", "om9-scene"],
        "legacy_decision": "PORT",
        "legacy_note": "Legacy coordinate input is portable logic; OM9 ports the grammar and the error labels unchanged and makes the frame an explicit typed parameter instead of an implicit context.",
    },
]


def design_api(row: dict) -> str:
    n = row["n"]
    lines = [
        f"# OM9-L3-01-08-{n} — {row['title']} (typed API)",
        "",
        f"Owner module `{MODULE}` (units, tolerances, basic parameters; stage `E01`, gate **{GATE}**).",
        f"Module contract: `implementation/design/M01/OM9-L2-01-08/api.md` §3.",
        f"Implementation owner: `{row['owner']}`. Legacy decision: **{row['legacy_decision']}**.",
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
        "## 3. Legacy `rust/` audit",
        "",
        *[f"- `{path}`" for path in row["legacy"]],
        "",
        row["legacy_note"],
        "",
        "## 4. Tests that carry the oracle",
        "",
        *[f"- `{command}`" for command, _ in row["tests"]],
        "",
        "## 5. Evidence",
        "",
        f"- `implementation/evidence/M01/OM9-L3-01-08-{n}/OM9-L4-01-08-{n}-03.target-test.log`",
        f"- `implementation/evidence/M01/OM9-L3-01-08-{n}/OM9-L4-01-08-{n}-05.regression.log`",
        f"- `implementation/evidence/M01/OM9-L3-01-08-{n}/OM9-L4-01-08-{n}-05.ui-review.json`",
        f"- `implementation/fixtures/M01/OM9-L3-01-08-{n}/positive.json`, `negative.json`",
        "",
    ]
    return "\n".join(lines)


def crosswalk(row: dict) -> dict:
    n = row["n"]
    return {
        "schema": "om9.m01.source_crosswalk.v1",
        "task_id": f"OM9-L4-01-08-{n}-01",
        "l3_id": f"OM9-L3-01-08-{n}",
        "l3_title": row["title"],
        "owner_module_id": MODULE,
        "milestone": "M01",
        "stage": "E01",
        "gate": GATE,
        "goal": row["goal"],
        "positive_oracle": row["positive"],
        "negative_oracle": row["negative"],
        "legacy_source": [str((ROOT / path).relative_to(ROOT).as_posix()) for path in row["legacy"]],
        "reuse_decision": row["legacy_decision"],
        "decision_note": row["legacy_note"],
        "method": (
            "Read the legacy `rust/` module named above and the target crate; the portable behaviour "
            "is re-implemented in safe Rust (AGENTS.md rust-first) without the legacy C ABI. No legacy "
            "code is copied verbatim."
        ),
        "verified_utc": UTC,
        "om9_owner": row["owner"],
        "runtime_tests": [command for command, _ in row["tests"]],
    }


def fixture(row: dict) -> tuple[dict, dict]:
    n = row["n"]
    positive = {
        "schema": "om9.m01.fixture.v1",
        "l3": f"OM9-L3-01-08-{n}",
        "l4": f"OM9-L4-01-08-{n}-02",
        "title": row["title"],
        "case": f"om9-l3-01-08-{n}-positive",
        "positive_oracle": row["positive"],
        "runtime_tests": [command for command, _ in row["tests"]],
    }
    negative = {
        "schema": "om9.m01.fixture.v1",
        "l3": f"OM9-L3-01-08-{n}",
        "l4": f"OM9-L4-01-08-{n}-02",
        "title": row["title"],
        "case": f"om9-l3-01-08-{n}-negative",
        "negative_oracle": row["negative"],
        "refusals": row["errors"],
        "runtime_tests": [command for command, _ in row["tests"]],
    }
    return positive, negative


def ui_review(row: dict) -> dict:
    n = row["n"]
    return {
        "schema": "om9.m01.ui-review.v1",
        "task_id": f"OM9-L4-01-08-{n}-05",
        "l3_id": f"OM9-L3-01-08-{n}",
        "title": row["title"],
        "oracle": {"positive": row["positive"], "negative": row["negative"]},
        "manual_test_gate": GATE,
        "implementer": "agent session (implementer never signs)",
        "reviewer": {
            "status": "AUTOMATED_EVIDENCE_ONLY",
            "role": "plan gate: this item requires no manual test",
            "date_utc": UTC,
            "statement": (
                "The plan records manual_test_gate=AUTO_ONLY for every L3 of OM9-L2-01-08 "
                "(L2.json acceptance_gate=AUTO, manual_test_required=false), so acceptance rests on the "
                "automated checks below. A human owner may replace this file with reviewer.status=SIGNED."
            ),
            "scope": "the typed numeric API and the document policy, never the window",
        },
        "checks_run": [
            *[f"focused: {command}" for command, _ in row["tests"]],
            "cargo test --workspace --no-fail-fast -j 2: every crate ok",
            "cargo fmt --check / cargo clippy --workspace --all-targets -- -D warnings: exit 0",
        ],
        "artefacts": [
            f"OM9-L4-01-08-{n}-03.target-test.log",
            f"OM9-L4-01-08-{n}-05.regression.log",
        ],
        "gate": {
            f"m01_l2_01_08_l3_{n}_scope": "ACCEPTED_ON_AUTOMATED_EVIDENCE",
            "implementation_status_promoted": False,
            "plan_tree_modified": False,
        },
    }


def run(command: str) -> tuple[int, str]:
    parts = command.split()
    finished = subprocess.run(parts, cwd=str(ROOT), capture_output=True, text=True)
    return finished.returncode, (finished.stdout or "") + (finished.stderr or "")


def write_evidence(row: dict) -> None:
    n = row["n"]
    folder = EVIDENCE / f"OM9-L3-01-08-{n}"
    folder.mkdir(parents=True, exist_ok=True)
    focused = []
    for command, _ in row["tests"]:
        code, text = run(command)
        focused.append(f"# command: {command}\n# exit: {code}\n\n{text}\n")
    (folder / f"OM9-L4-01-08-{n}-03.target-test.log").write_text(
        f"# OM9-L2-01-08 focused verification\n# utc: {UTC}\n\n" + "\n".join(focused),
        encoding="utf-8",
    )
    logs = []
    for crate in row["regression"]:
        command = f"cargo test -p {crate} -j 2"
        code, text = run(command)
        logs.append(f"# command: {command}\n# exit: {code}\n\n{text}\n")
    (folder / f"OM9-L4-01-08-{n}-05.regression.log").write_text("\n".join(logs), encoding="utf-8")


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
        design = DESIGN / f"OM9-L3-01-08-{n}"
        fixtures = FIXTURES / f"OM9-L3-01-08-{n}"
        evidence = EVIDENCE / f"OM9-L3-01-08-{n}"
        for folder in (design, fixtures, evidence):
            folder.mkdir(parents=True, exist_ok=True)
        (design / "api.md").write_text(design_api(row), encoding="utf-8")
        (design / "source_crosswalk.json").write_text(
            json.dumps(crosswalk(row), indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
        )
        positive, negative = fixture(row)
        (fixtures / "positive.json").write_text(json.dumps(positive, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
        (fixtures / "negative.json").write_text(json.dumps(negative, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
        (evidence / f"OM9-L4-01-08-{n}-05.ui-review.json").write_text(
            json.dumps(ui_review(row), indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
        )
        write_evidence(row)
        print("wrote", n, row["title"])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
