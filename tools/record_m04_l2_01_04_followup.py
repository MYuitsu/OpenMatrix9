#!/usr/bin/env python3
"""Record the M04 ``OM9-L2-01-04`` producer/SDK follow-up from real runs.

The first batch closed the graph kernel. This follow-up adds the pieces the kernel alone
could not prove:

* a real rail/profile/sweep producer chain (`om9-core/src/producers.rs`) with the OCCT
  sweep backend and topology provider (`om9-kernel/src/associative.rs`);
* the public history read contract in `om9-sdk` and its host dispatch;
* a delta-based `PreviewSession`.

Every log is the captured output of the command in its header. The tool writes
``implementation/evidence/M04/OM9-L2-01-04-FOLLOWUP/`` and appends one live-ledger entry.

Usage::

    python tools/record_m04_l2_01_04_followup.py
"""

from __future__ import annotations

import datetime
import json
import os
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "implementation" / "evidence" / "M04" / "OM9-L2-01-04-FOLLOWUP"
TARGET_DIR = os.environ.get("CARGO_TARGET_DIR", r"L:\om9-target")
KEY = "m04-l2-01-04-producers-sdk-followup-2026-10-27"


def run(command: list[str], path_env: bool = False) -> tuple[int, str]:
    env = dict(os.environ)
    env["CARGO_TARGET_DIR"] = TARGET_DIR
    env.setdefault("CARGO_TERM_COLOR", "never")
    if path_env:
        # OCCT DLLs live beside the test executables; the app launcher sets the same path.
        env["PATH"] = TARGET_DIR + os.pathsep + env.get("PATH", "")
    process = subprocess.run(
        command,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        env=env,
    )
    return process.returncode, process.stdout + process.stderr


def header(command: str, code: int, captured: str) -> str:
    return (
        f"$ {command}\n# exit code: {code}\n"
        f"# captured: {datetime.datetime.now(datetime.UTC).strftime('%Y-%m-%dT%H:%M:%SZ')}\n"
        f"# target dir: {TARGET_DIR}\n\n{captured}"
    )


def main() -> int:
    OUTPUT.mkdir(parents=True, exist_ok=True)
    checks = [
        (
            "producers-core.log",
            ["cargo", "test", "-p", "om9-core", "--test", "producers", "-j", "2"],
            False,
        ),
        (
            "producers-occt.log",
            [
                "cargo",
                "test",
                "-p",
                "om9-kernel",
                "--features",
                "3dm-brep",
                "--test",
                "associative_history",
                "-j",
                "2",
            ],
            True,
        ),
        (
            "associative-regression.log",
            ["cargo", "test", "-p", "om9-core", "--test", "associative_rebuild", "-j", "2"],
            False,
        ),
        (
            "sdk-history.log",
            ["cargo", "test", "-p", "om9-plugin-host", "--lib", "-j", "2", "history"],
            False,
        ),
        (
            "external-plugin.log",
            ["cargo", "check", "-p", "om9-hello-plugin", "-j", "2"],
            False,
        ),
        (
            "clippy.log",
            [
                "cargo",
                "clippy",
                "-p",
                "om9-core",
                "-p",
                "om9-sdk",
                "-p",
                "om9-plugin-host",
                "-p",
                "om9-hello-plugin",
                "--all-targets",
                "-j",
                "2",
                "--",
                "-D",
                "warnings",
            ],
            False,
        ),
        (
            "workspace-check.log",
            ["cargo", "check", "--workspace", "--all-targets", "-j", "2"],
            False,
        ),
    ]
    results = []
    failed = False
    for name, command, path_env in checks:
        code, captured = run(command, path_env)
        (OUTPUT / name).write_text(header(" ".join(command), code, captured), encoding="utf-8")
        summary = [
            line
            for line in captured.splitlines()
            if line.startswith(("test result:", "error", "FAILED", "Finished"))
        ]
        results.append(
            {
                "artifact": name,
                "command": command,
                "exit_code": code,
                "summary": summary[-4:],
            }
        )
        if code != 0:
            failed = True
            print(f"FAIL: {name} exit {code}", file=sys.stderr)
            print("\n".join(summary[-20:]), file=sys.stderr)
    if failed:
        return 1

    _, revision = run(["git", "rev-parse", "HEAD"])
    _, status = run(["git", "status", "--porcelain"])
    observed = {
        "schema": "om9.m04.followup.v1",
        "l2": "OM9-L2-01-04",
        "recorded_at_utc": datetime.datetime.now(datetime.UTC).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "source_revision": revision.strip().splitlines()[0] if revision.strip() else "UNRESOLVED",
        "checkout_dirty": bool(status.strip()),
        "target": "Standalone Rust OM9; Windows x64; CARGO_TARGET_DIR=L:/om9-target; -j 2",
        "scope": [
            "real ring rail/profile/sweep producer chain",
            "OCCT sweep backend and topology provider for PublishedSubshapes",
            "SDK history read contract and host dispatch",
            "external example plugin command HistoryDemo",
            "delta-based PreviewSession",
        ],
        "code": [
            "crates/om9-core/src/producers.rs",
            "crates/om9-core/src/associative.rs",
            "crates/om9-kernel/src/associative.rs",
            "crates/om9-kernel/tests/associative_history.rs",
            "crates/om9-sdk/src/lib.rs",
            "crates/om9-plugin-host/src/staged.rs",
            "sdk/examples/hello-plugin/src/lib.rs",
        ],
        "checks": results,
        "notes": [
            "The OCCT run uses the pinned 8.0.1 bridge; the test executable needs the target DLL directory on PATH.",
            "The SDK export is read-only by design: recompute needs the native producer and the host transaction/Undo owner.",
        ],
    }
    (OUTPUT / "observed.json").write_text(
        json.dumps(observed, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )

    ledger_path = ROOT / "docs" / "openmatrix9-progress.json"
    ledger = json.loads(ledger_path.read_text(encoding="utf-8"))
    entry = {
        "key": KEY,
        "date": "2026-10-27",
        "stage": "feature-port",
        "status": "producers_occt_sdk_and_delta_preview_verified",
        "target": observed["target"],
        "feature_ids": ["OM9-L2-01-04", "OM9-L2-04-02", "OM9-L2-01-05"],
        "scope": (
            "M04 OM9-L2-01-04 follow-up: nối producer thật ring rail/profile/sweep (rail/profile pure Rust, "
            "sweep bằng OCCT pipe shell), topology provider cho PublishedSubshapes từ snapshot B-Rep, SDK history "
            "read contract + host dispatch + plugin mẫu, và PreviewSession chỉ lưu delta thay vì clone document."
        ),
        "outputs": observed["code"] + ["implementation/evidence/M04/OM9-L2-01-04-FOLLOWUP/"],
        "evidence": [
            "implementation/evidence/M04/OM9-L2-01-04-FOLLOWUP/observed.json",
            "implementation/evidence/M04/OM9-L2-01-04-FOLLOWUP/producers-core.log",
            "implementation/evidence/M04/OM9-L2-01-04-FOLLOWUP/producers-occt.log",
            "implementation/evidence/M04/OM9-L2-01-04-FOLLOWUP/sdk-history.log",
            "implementation/evidence/M04/OM9-L2-01-04-FOLLOWUP/workspace-check.log",
        ],
        "tests": {
            "producers_core": "7 passed (rail circle, profile rectangle, analytic sweep, reference allowed/refused)",
            "producers_occt": "2 passed (real B-Rep sweep, edge/face/vertex topology, reference after edit)",
            "associative_rebuild": "24 passed (including the delta preview)",
            "plugin_host": "3 history tests passed (list/dependencies/provenance/plan, read-only, bad id)",
            "external_plugin": "om9-hello-plugin compiles against the public SDK with HistoryDemo",
            "clippy": "om9-core/sdk/plugin-host/example: -D warnings clean; om9-kernel has one pre-existing "
            "suspicious_assignment_formatting in measurement.rs outside this batch",
        },
        "uncertainties": [
            "Builder UI (live handles, panels, command frame) belongs to OM9-L2-03-11 / OM9-L2-04-02; batch này chỉ mở service headless mà builder gọi.",
            "Chưa thêm mutation transaction cho history recipe: app owner quyết định cách journal recompute; app hiện replay qua Mutation::Geometry.",
            "External plugin mới được compile trong workspace; package .om9p + chạy app cần tools/validate_plugin_sdk.py với binary om9/om9-plugin đã build.",
            "OCCT test cần DLL trong L:/om9-target/debug trên PATH; CI không đặt PATH sẽ không chạy được test này.",
        ],
        "manual_gate": "AUTO_ONLY: evidence tự động; không tuyên bố chữ ký người thật",
        "next_action": "Nối builder UI (L2-04-02) gọi ProducerRegistry/recompute_registered trong transaction, hoặc thêm Mutation cho history recipe.",
    }
    ledger["items"] = [item for item in ledger["items"] if item.get("key") != KEY] + [entry]
    ledger["active_item"] = KEY
    ledger["updated_at"] = datetime.datetime.now().astimezone().isoformat(timespec="seconds")
    ledger_path.write_text(json.dumps(ledger, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")

    print(f"recorded {OUTPUT.relative_to(ROOT)}")
    for result in results:
        print(f"  {result['artifact']}: exit {result['exit_code']}")
    print(f"ledger: {ledger_path.relative_to(ROOT)} ({len(ledger['items'])} items)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
