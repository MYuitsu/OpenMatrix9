{
  "schema_version": "om9.m07.l4.v1",
  "task_id": "OM9-L4-04-16-006-04",
  "parent_l3_id": "OM9-L3-04-16-006",
  "owner_module_id": "OM9-L2-04-16",
  "parent_capability_id": "cap.04-16.interactive-boolean-on-metal",
  "milestone_id": "M07",
  "target_milestone": "M07",
  "execution_stage": "E12",
  "task_name": "Kiểm thử phản kháng, evidence và kết luận release gate",
  "title": "Interactive Boolean On Metal — Kiểm thử phản kháng, evidence và kết luận release gate",
  "tag": "febe",
  "work_kind": "ADVERSARIAL_GATE",
  "phase": 4,
  "status": "PLANNED_NOT_EXECUTED",
  "readiness": "BLOCKED_BY_G00_E11_AND_LOCAL_SOURCE",
  "source_repo": "https://github.com/QymIs-Tech/QymCAD",
  "source_revision_candidate": "746a17f127897e921a57118b6caad08968602bf5",
  "source_paths_verified_locally": [],
  "reuse_decision": "PENDING_LOCAL_SOURCE_AUDIT",
  "estimated_scope": null,
  "origin_manual_refs": [
    {
      "manual_id": "book2",
      "filename": "matrix_8_manual_book2.pdf",
      "printed_pages": [
        501,
        505
      ],
      "pdf_pages_1based": [
        275,
        279
      ],
      "precision": "verified_pdf_text_in_current_session"
    }
  ],
  "purpose": "Missing/locked metal, tangential contact, malformed cutters, backend exception, batch preview, same geometry after cancel.",
  "inputs": [
    "Canonical L3: specs/07-ms07-cutters-manufacturing-check/01-febe-single-gem-seat-cutter-workflows/006-febe-interactive-boolean-on-metal/l3.json",
    "Owner module: specs/07-ms07-cutters-manufacturing-check/01-febe-single-gem-seat-cutter-workflows/module.json",
    "specs/00-contracts/MATRIX9_FIRST_CONTRACTS.md",
    "specs/00-contracts/MATRIX_WORKBENCH_UI.md",
    "planning/E12.md",
    "planning/GATES_AND_EVIDENCE.md",
    "planning/DEPENDENCY_AND_RELEASE_POLICY.md",
    "planning/M00/SOURCE_AUDIT_V012.md",
    "QymCAD checkout at pinned source SHA; local exists only if verified during execution",
    "Cutter solids, metal target EntityId, Boolean preview policy Off/On/One, option Surface."
  ],
  "expected_domain_inputs": "Cutter solids, metal target EntityId, Boolean preview policy Off/On/One, option Surface.",
  "outputs": [
    "implementation/evidence/M07/OM9-L4-04-16-006-04/adversarial-matrix.csv",
    "implementation/evidence/M07/OM9-L4-04-16-006-04/gate-review.md"
  ],
  "api_contract_and_callers": [
    "Caller capability: cap.04-16.interactive-boolean-on-metal; owner OM9-L2-04-16.",
    "Related modules: OM9-L2-04-09, OM9-L2-01-10, OM9-L2-04-14.",
    "Final source-level function names must be pinned from local checkout before code; candidates are NOT claims.",
    "Canonical geometry owner is OM9-L2-01-10; F6/menu/builder must call shared service not implement BRep/Boolean themselves."
  ],
  "steps": [
    "Tạo fixture tự sinh adversarial, không sử dụng tài nguyên proprietary từ Matrix.",
    "Chạy kiểm thử invalid BRep, borderline tolerance, Undo/Redo/Save/Open, cancel mid-process, GPU/UI as applicable.",
    "Phân biệt expected/actual quantitatively; cross-check source/target SHA, logs; lỗi phát hiện phải có repro & owner.",
    "Kết thúc bằng PASS / FAIL / BLOCKED theo evidence; chỉ có tester & reviewer thật mới sign-off manual gate.",
    "Handoff regression và cập nhật known limits; nếu fail trọng yếu khóa G-MVP."
  ],
  "preconditions": [
    "G00-01 .. G00-07 PASS + reviewer/license sign-off",
    "E01..E11 PASS with evidence; E11 source contracts available",
    "Local QymCAD checkout, pinned SHA verified; OCCT/FFI build/golden fixtures reproducible",
    "M07 source/legal and numeric tolerances approved before implementation"
  ],
  "depends_on_task_ids": [
    "OM9-L4-04-16-006-03"
  ],
  "depends_on_module_ids": [
    "OM9-L2-04-09",
    "OM9-L2-01-10",
    "OM9-L2-04-14"
  ],
  "geometry_invariants": [
    "BRep truth stays in OCCT, never GPU/render mesh.",
    "Document EntityId/FeatureId/LayerId/InstanceId stability, world units mm at manufacturing export.",
    "No unapproved geometry tweak (move/scale) to force Boolean; warn and block instead.",
    "Preview/cancel cannot mutate canonical document. One commit = one undoable transaction.",
    "Unexpected naked edges/non-manifold/crossing surfaces fail pre-export validation.",
    "Missing/locked metal, tangential contact, malformed cutters, backend exception, batch preview, same geometry after cancel."
  ],
  "error_cases": [
    "No geometry / empty selection",
    "Invalid or degenerate BRep",
    "Unsupported/unknown unit or tolerance",
    "Ambiguous/stale EntityId",
    "Locked/hidden/wrong-type target",
    "OCCT/FFI failure, cancellation, output write failure",
    "Missing validated source or dependency: BLOCKED, not silent stub"
  ],
  "files_expected_to_touch": [
    "specs/07-ms07-cutters-manufacturing-check/01-febe-single-gem-seat-cutter-workflows/module.json (READ ONLY for ownership/contract check)",
    "specs/07-ms07-cutters-manufacturing-check/01-febe-single-gem-seat-cutter-workflows/006-febe-interactive-boolean-on-metal/l3.json (READ ONLY; canonical requirement)",
    "VERIFIED_SOURCE_PATHS_TO_BE_FILLED_AFTER_LOCAL_AUDIT (no invented crate/file paths)",
    "implementation/evidence/M07/OM9-L4-04-16-006-04/*",
    "tests/fixtures/m07/* (self-authored; proposed destination, create only after repo layout audit)"
  ],
  "automated_tests": [
    "Missing/locked metal, tangential contact, malformed cutters, backend exception, batch preview, same geometry after cancel.",
    "Missing/locked metal, tangential contact, malformed cutters, backend exception, batch preview, same geometry after cancel.",
    "Cross-check zero unexpected naked edges, preserved units/IDs and non-destructive failures."
  ],
  "manual_steps_and_expected": [
    "Open real Windows OpenMatrix9 build at tested SHA; invoke Interactive Boolean cutter preview through Cutters or registered contextual F6 entry. Expected: enabled only for valid typed selection.",
    "Apply valid parameters (Cutter solids, metal target EntityId, Boolean preview policy Off/On/One, option Surface.) and move controls. Expected: preview changes deterministically; geometry uncommitted and inputs visible.",
    "Click Cancel: expected identical document IDs, feature DAG, visibility and body count; repeat then Commit: exactly one undoable transaction.",
    "Perform Undo/Redo then Save/Open; expected stable IDs, geometry and controls, no silent detach or success toast on errors.",
    "Exercise failure case (Missing/locked metal, tangential contact, malformed cutters, backend exception, batch preview, same geometry after cancel.); expected actionable warning and no partial write. Record tester/date/OS/GPU/screenshots/video/actual results."
  ],
  "fixture_ids": [
    "M07-FX-04-16-006",
    "M07-NEG-04-16-006",
    "M07-FX-E2E-RING-CUTTER-STL"
  ],
  "pass_criteria": [
    "Evidence outputs exist, include exact commands + exit codes and expected/actual assertions.",
    "No silent no-op, no geometrical success by renderer only, and no rollback/Undo/Redo regressions.",
    "Upstream source checked locally, pin SHA and actual touched files recorded, dependencies PASS.",
    "Missing/locked metal, tangential contact, malformed cutters, backend exception, batch preview, same geometry after cancel.",
    "For FEBE: manual execution and independent human acceptance required before DONE."
  ],
  "release_evidence": [
    "source_sha",
    "tested_sha",
    "timestamp_utc",
    "os_arch_gpu",
    "rust_toolchain",
    "occt_version_build_hash",
    "commands_exit_codes_logs",
    "fixture_ids_units_tolerances",
    "expected_actual_numeric_diffs",
    "screenshot_video_paths_if_febe",
    "tester",
    "reviewer",
    "known_limitations",
    "regression_result"
  ],
  "failure_policy": "BLOCKED if G00/E11/source/fixture absent; FAIL on invalid geometry, wrong units, corruption, lost references or silent success; never mark DONE without evidence.",
  "task_relpath": "planning/M07/04-16-006/04-04-adversarial-release-evidence/l4.json",
  "manual_command_verified_in_pdf": null,
  "manual_command_verification_status": "MANUAL_UI_LABEL_ONLY_NOT_AN_EXECUTABLE_ALIAS"
}
