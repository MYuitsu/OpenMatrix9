{
  "schema_version": "om9.m07.l4.v1",
  "task_id": "OM9-L4-01-12-008-03",
  "parent_l3_id": "OM9-L3-01-12-008",
  "owner_module_id": "OM9-L2-01-12",
  "parent_capability_id": "cap.01-12.stl-tessellation-export-codec",
  "milestone_id": "M01",
  "target_milestone": "M07",
  "execution_stage": "E12",
  "task_name": "Thực hiện interaction có quan sát được và QA theo tag",
  "title": "STL Tessellation Export Codec — Thực hiện interaction có quan sát được và QA theo tag",
  "tag": "be",
  "work_kind": "BE_EXPORT_ACCEPTANCE",
  "phase": 3,
  "status": "PLANNED_NOT_EXECUTED",
  "readiness": "BLOCKED_BY_G00_E11_AND_LOCAL_SOURCE",
  "source_repo": "https://github.com/QymIs-Tech/QymCAD",
  "source_revision_candidate": "746a17f127897e921a57118b6caad08968602bf5",
  "source_paths_verified_locally": [],
  "reuse_decision": "PENDING_LOCAL_SOURCE_AUDIT",
  "estimated_scope": null,
  "origin_manual_refs": [
    {
      "manual_id": "book1",
      "filename": "matrix8_book_1.pdf",
      "printed_pages": [
        79,
        86
      ],
      "pdf_pages_1based": [
        89,
        96
      ],
      "precision": "parent_l2_range"
    },
    {
      "manual_id": "book1",
      "filename": "matrix8_book_1.pdf",
      "printed_pages": [
        48,
        56
      ],
      "pdf_pages_1based": [
        58,
        66
      ],
      "precision": "parent_l2_range"
    }
  ],
  "purpose": "Không có UI riêng trong L3 BE. Tạo command/service API có trạng thái success/error và artifacts để màn hình export hiện lỗi từ owner, không viết nút UI ở đây.",
  "inputs": [
    "Canonical L3: specs/01-ms01-workbench-shell/05-be-native-project-codec-stl-export/008-be-stl-tessellation-export-codec/l3.json",
    "Owner module: specs/01-ms01-workbench-shell/05-be-native-project-codec-stl-export/module.json",
    "specs/00-contracts/MATRIX9_FIRST_CONTRACTS.md",
    "specs/00-contracts/MATRIX_WORKBENCH_UI.md",
    "planning/E12.md",
    "planning/GATES_AND_EVIDENCE.md",
    "planning/DEPENDENCY_AND_RELEASE_POLICY.md",
    "planning/M00/SOURCE_AUDIT_V012.md",
    "QymCAD checkout at pinned source SHA; local exists only if verified during execution",
    "Danh sách EntityId/ShapeId được phép xuất; Document units; meshing tolerances; output path; lựa chọn binary/ASCII đã được phê duyệt."
  ],
  "expected_domain_inputs": "Danh sách EntityId/ShapeId được phép xuất; Document units; meshing tolerances; output path; lựa chọn binary/ASCII đã được phê duyệt.",
  "outputs": [
    "implementation/evidence/M07/OM9-L4-01-12-008-03/export-e2e.md",
    "implementation/evidence/M07/OM9-L4-01-12-008-03/test-run.json"
  ],
  "api_contract_and_callers": [
    "Caller capability: cap.01-12.stl-tessellation-export-codec; owner OM9-L2-01-12.",
    "Related modules: OM9-L2-01-03.",
    "Final source-level function names must be pinned from local checkout before code; candidates are NOT claims.",
    "Tessellation & codec owns file export. Do not infer UI components or extra renderer/codec ownership."
  ],
  "steps": [
    "Kiểm phase 02 PASS; chỉ bind UI tới command/service đã có (hoặc với STL BE, chạy export end-to-end từ test harness).",
    "Nếu FEBE: register menu/F6 as appropriate, render typed params, preview, Undo/Redo, Cancel và errors; disabled nếu chưa supported.",
    "Nếu BE: bổ sung end-to-end export test/independent viewer procedure thay vì viết UI không thuộc L3.",
    "Test command and app smoke; có build Windows thực mới lập checklist hành động tay/timestamps/screenshots; chưa ký người dùng.",
    "Trả READY_FOR_MANUAL_TEST (FEBE) hoặc READY_FOR_REVIEW (BE), không tự DONE nếu chưa evidence đầy đủ."
  ],
  "preconditions": [
    "G00-01 .. G00-07 PASS + reviewer/license sign-off",
    "E01..E11 PASS with evidence; E11 source contracts available",
    "Local QymCAD checkout, pinned SHA verified; OCCT/FFI build/golden fixtures reproducible",
    "M07 source/legal and numeric tolerances approved before implementation"
  ],
  "depends_on_task_ids": [
    "OM9-L4-01-12-008-02"
  ],
  "depends_on_module_ids": [
    "OM9-L2-01-03"
  ],
  "geometry_invariants": [
    "BRep truth stays in OCCT, never GPU/render mesh.",
    "Document EntityId/FeatureId/LayerId/InstanceId stability, world units mm at manufacturing export.",
    "No unapproved geometry tweak (move/scale) to force Boolean; warn and block instead.",
    "Preview/cancel cannot mutate canonical document. One commit = one undoable transaction.",
    "Unexpected naked edges/non-manifold/crossing surfaces fail pre-export validation.",
    "Tessellation-to-STL cần bảo đảm hình học OCCT chính xác và đơn vị tệp; external viewer không đồng nghĩa kiểm chứng sản xuất thật."
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
    "specs/01-ms01-workbench-shell/05-be-native-project-codec-stl-export/module.json (READ ONLY for ownership/contract check)",
    "specs/01-ms01-workbench-shell/05-be-native-project-codec-stl-export/008-be-stl-tessellation-export-codec/l3.json (READ ONLY; canonical requirement)",
    "VERIFIED_SOURCE_PATHS_TO_BE_FILLED_AFTER_LOCAL_AUDIT (no invented crate/file paths)",
    "implementation/evidence/M07/OM9-L4-01-12-008-03/*",
    "tests/fixtures/m07/* (self-authored; proposed destination, create only after repo layout audit)"
  ],
  "automated_tests": [
    "Golden BRep + mesh fixture: bounding-box đơn vị mm, số vùng kín, triangle winding/orientation, manifold edges, no degenerate/NaN, crash-safe file, export/reopen via independent viewer; so sánh dimension không dùng screenshots làm oracle.",
    "Binary/ASCII STL can be parsed independently; verify units, manifold/orientation and byte reproducibility policy."
  ],
  "manual_steps_and_expected": [],
  "fixture_ids": [
    "M07-FX-01-12-008",
    "M07-NEG-01-12-008",
    "M07-FX-ROLLBACK"
  ],
  "pass_criteria": [
    "Evidence outputs exist, include exact commands + exit codes and expected/actual assertions.",
    "No silent no-op, no geometrical success by renderer only, and no rollback/Undo/Redo regressions.",
    "Upstream source checked locally, pin SHA and actual touched files recorded, dependencies PASS.",
    "Golden BRep + mesh fixture: bounding-box đơn vị mm, số vùng kín, triangle winding/orientation, manifold edges, no degenerate/NaN, crash-safe file, export/reopen via independent viewer; so sánh dimension không dùng screenshots làm oracle.",
    "For BE export: independent STL parser/viewer check required before release PASS."
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
  "task_relpath": "planning/M07/01-12-008/03-03-ui-or-external-acceptance/l4.json",
  "manual_command_verified_in_pdf": null,
  "manual_command_verification_status": "MANUAL_UI_LABEL_ONLY_NOT_AN_EXECUTABLE_ALIAS"
}
