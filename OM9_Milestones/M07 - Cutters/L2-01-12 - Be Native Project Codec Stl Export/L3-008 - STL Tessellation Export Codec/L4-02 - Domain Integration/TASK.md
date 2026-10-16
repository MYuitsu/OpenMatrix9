{
  "schema_version": "om9.m07.l4.v1",
  "task_id": "OM9-L4-01-12-008-02",
  "parent_l3_id": "OM9-L3-01-12-008",
  "owner_module_id": "OM9-L2-01-12",
  "parent_capability_id": "cap.01-12.stl-tessellation-export-codec",
  "milestone_id": "M01",
  "target_milestone": "M07",
  "execution_stage": "E12",
  "task_name": "Nối dịch vụ/thuật toán owner hiện hữu và geometry safety",
  "title": "STL Tessellation Export Codec — Nối dịch vụ/thuật toán owner hiện hữu và geometry safety",
  "tag": "be",
  "work_kind": "OWNER_INTEGRATION",
  "phase": 2,
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
  "purpose": "Gọi tessellator dùng chung trên BRep; tập hợp triangle không âm/NaN, winding cùng normal, kiểm closed 2-manifold theo fixture; ghi file qua temporary path rồi atomic rename; sai thì không tạo file đích mới.",
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
    "implementation/evidence/M07/OM9-L4-01-12-008-02/geometry-or-service-tests.json",
    "implementation/evidence/M07/OM9-L4-01-12-008-02/source-diff-review.md"
  ],
  "api_contract_and_callers": [
    "Caller capability: cap.01-12.stl-tessellation-export-codec; owner OM9-L2-01-12.",
    "Related modules: OM9-L2-01-03.",
    "Final source-level function names must be pinned from local checkout before code; candidates are NOT claims.",
    "Tessellation & codec owns file export. Do not infer UI components or extra renderer/codec ownership."
  ],
  "steps": [
    "Đọc contract PASS của phase 01, kiểm code owner khác có thật ở pinned working copy.",
    "Viết thay đổi tối thiểu ở owner đã kiểm chứng; UI module chỉ điều phối, không copy OCCT Booleans, meshing, persistence hoặc history.",
    "Dùng typed inputs; implement validation/explicit error/preview isolation/rollback; ghi retry policy, timeout/cancellation như source thật hỗ trợ.",
    "Thêm test fixture tự tạo và assertions về hình học, units, IDs, transaction state; dùng tolerance/threshold từ G00.",
    "Chạy formatter, linter, unit/integration tests target, regressions E07/E09–E11 liên quan; ghi exact command/exit/log."
  ],
  "preconditions": [
    "G00-01 .. G00-07 PASS + reviewer/license sign-off",
    "E01..E11 PASS with evidence; E11 source contracts available",
    "Local QymCAD checkout, pinned SHA verified; OCCT/FFI build/golden fixtures reproducible",
    "M07 source/legal and numeric tolerances approved before implementation"
  ],
  "depends_on_task_ids": [
    "OM9-L4-01-12-008-01"
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
    "implementation/evidence/M07/OM9-L4-01-12-008-02/*",
    "tests/fixtures/m07/* (self-authored; proposed destination, create only after repo layout audit)"
  ],
  "automated_tests": [
    "Golden BRep + mesh fixture: bounding-box đơn vị mm, số vùng kín, triangle winding/orientation, manifold edges, no degenerate/NaN, crash-safe file, export/reopen via independent viewer; so sánh dimension không dùng screenshots làm oracle.",
    "Invalid input & OCCT failure: assert before/after document hash equality and no resource leak.",
    "Expected numerical tolerances must come from signed G00 fixtures, not invented constants."
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
  "task_relpath": "planning/M07/01-12-008/02-02-domain-integration/l4.json",
  "manual_command_verified_in_pdf": null,
  "manual_command_verification_status": "MANUAL_UI_LABEL_ONLY_NOT_AN_EXECUTABLE_ALIAS"
}
