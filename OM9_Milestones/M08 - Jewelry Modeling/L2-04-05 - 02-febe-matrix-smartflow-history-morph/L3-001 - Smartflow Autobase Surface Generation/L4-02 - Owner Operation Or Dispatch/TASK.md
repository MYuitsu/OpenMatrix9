{
  "schema_version": "om9.m08.l4.v1",
  "task_id": "OM9-L4-04-05-001-02",
  "parent_l3_id": "OM9-L3-04-05-001",
  "owner_module_id": "OM9-L2-04-05",
  "parent_capability_id": "cap.04-05.smartflow-autobase-surface-generation",
  "milestone_id": "M08",
  "target_milestone": "M08",
  "execution_stage": "E13",
  "title": "Smartflow Autobase Surface Generation — Triển khai service hoặc adapter đúng owner",
  "task_name": "Triển khai service hoặc adapter đúng owner",
  "tag": "febe",
  "implementation_role": "ui_command_adapter",
  "work_kind": "DOMAIN_IMPLEMENTATION",
  "phase": 2,
  "status": "PLANNED_NOT_EXECUTED",
  "readiness": "BLOCKED_BY_G00_E12_G_MVP_AND_LOCAL_SOURCE",
  "source_repo": "https://github.com/QymIs-Tech/QymCAD",
  "source_revision_candidate": "c116f8f224ebffe0ada049c91c0d15df3634f00d",
  "source_paths_verified_locally": [],
  "reuse_decision": "PENDING_LOCAL_SOURCE_AUDIT",
  "estimated_scope": null,
  "origin_manual_refs": [
    {
      "manual_id": "book2",
      "filename": "matrix_8_manual_book2.pdf",
      "printed_page": 242,
      "pdf_page_1based": 16,
      "precision": "section_heading",
      "provenance": "INHERITED_FROM_CANONICAL_L3; verify exact pages at execution before implementation"
    },
    {
      "manual_id": "book2",
      "filename": "matrix_8_manual_book2.pdf",
      "printed_pages": [
        242,
        245
      ],
      "pdf_pages_1based": [
        16,
        19
      ],
      "precision": "parent_l2_range",
      "provenance": "INHERITED_FROM_CANONICAL_L3; verify exact pages at execution before implementation"
    }
  ],
  "source_command_candidate_from_l3": null,
  "source_command_verification_status": "INHERITED_FROM_L3_UNCONFIRMED_IN_LOCAL_RUNTIME",
  "purpose": "create flat base surface from destination UV geometry",
  "owner_rule": "The named command is a GUI/command-line action only. Call the designated implementation owner; do not create a second geometry kernel or duplicate data stores.",
  "expected_domain_inputs": "Destination SurfaceRef, UV domain, direction/normal, base-generation policy",
  "expected_behavior_oracle": "Generate base corresponding to selected surface; assert unambiguous UV orientation and mapped corners",
  "high_risk_negative_case": "Trimmed/periodic/seam-crossing surface requires explicit domain handling",
  "inputs": [
    "specs/08-ms08-advanced-jewelry-modeling/02-febe-matrix-smartflow-history-morph/001-febe-smartflow-autobase-surface-generation/l3.json",
    "specs/08-ms08-advanced-jewelry-modeling/02-febe-matrix-smartflow-history-morph/module.json",
    "specs/00-contracts/MATRIX9_FIRST_CONTRACTS.md",
    "specs/00-contracts/MATRIX_WORKBENCH_UI.md",
    "planning/E13.md",
    "planning/GATES_AND_EVIDENCE.md",
    "planning/DEPENDENCY_AND_RELEASE_POLICY.md",
    "planning/M00/SOURCE_AUDIT_V012.md",
    "planning/M08/LUNA_START_HERE.md",
    "Local QymCAD checkout at verified SHA (not included; check actual source)",
    "Destination SurfaceRef, UV domain, direction/normal, base-generation policy"
  ],
  "outputs": [
    "implementation/evidence/M08/OM9-L4-04-05-001-02/build-and-tests.json",
    "implementation/evidence/M08/OM9-L4-04-05-001-02/operation-review.md"
  ],
  "api_contract_and_callers": [
    "Canonical owner: OM9-L2-04-05; capability: cap.04-05.smartflow-autobase-surface-generation; role: ui_command_adapter.",
    "Dependent owner modules (check actual API after source audit): OM9-L2-01-04, OM9-L2-01-05, OM9-L2-02-05, OM9-L2-04-07.",
    "Typed contract MUST include IDs/units/frames/tolerance/transaction/errors, and preview snapshot; specific Rust structs/functions not asserted until local inspection.",
    "Only this L3 owns described feature. Other menus/services are callers; use shared kernel and feature graph."
  ],
  "steps": [
    "Require phase 01 contract PASS plus listed in-stage owner prerequisites; audit current checkout and exact files before edit.",
    "Implement owning service for typed inputs: Destination SurfaceRef, UV domain, direction/normal, base-generation policy. Reuse OCCT shape/transform/curve primitives and signed history graph rather than approximating with rendering mesh.",
    "Build deterministic fixture oracle: Generate base corresponding to selected surface; assert unambiguous UV orientation and mapped corners. Compare numeric expected/actual using tolerances approved at G00, not fabricated constants.",
    "Add negative input/error contract: Trimmed/periodic/seam-crossing surface requires explicit domain handling. Ensure rollback, stable entity IDs, no leaked handles and no silent success.",
    "Run clean build + unit/integration tests in actual workspace; store toolchain, commands, fixture hashes and exit codes. Stop at READY_FOR_REVIEW until verified."
  ],
  "preconditions": [
    "G00 signed PASS + local license and OCCT-Rust FFI evidence",
    "E01–E12 signed PASS; M07 & previous MVP regression fixtures available",
    "G-MVP reviewer signed PASS; E13 must not compromise MVP baseline",
    "Local QymCAD checkout at verified repository SHA; actual Rust/OCCT toolchain, golden fixture thresholds recorded",
    "Every depends_on_task_ids task PASS with evidence; if not stop BLOCKED_DEPENDENCY"
  ],
  "depends_on_task_ids": [
    "OM9-L4-04-05-001-01"
  ],
  "depends_on_module_ids": [
    "OM9-L2-01-04",
    "OM9-L2-01-05",
    "OM9-L2-02-05",
    "OM9-L2-04-07"
  ],
  "geometry_invariants": [
    "Canonical shapes/curves via Rust domain + OCCT FFI; never treat renderer tessellation or viewport cache as authoritative BRep.",
    "Use documented source/target frames; mm model units; numeric and topology tolerances from approved G00, not guessed values.",
    "EntityId/FeatureId/LayerId/InstanceId & provenance stable; preserve non-selected entities, layer locks and instance semantics.",
    "Preview read-only against committed model; Cancel same document hash; one Commit equals one atomic Undo transaction; error rolls back.",
    "History dependency directed acyclic graph, serializable stable parameter schema, deterministic recompute, no silent source detach.",
    "MVP golden fixtures and existing M07 STL/cutters must not regress after E13.",
    "Base/destination UV directions, normal/winding, trimmed boundaries, periodic seams and rigid gem placement explicitly validated.",
    "Generate base corresponding to selected surface; assert unambiguous UV orientation and mapped corners"
  ],
  "error_cases": [
    "Empty/wrong type selection; document locked, missing asset/feature ID, stale reference.",
    "Zero/NaN/Inf/singular transforms, tolerance or invalid geometry.",
    "OCCT exception, FFI failure, OOM/resource limit, canceled transaction.",
    "Missing source/gate/evidence: BLOCKED, never fabricate implementation or source filenames.",
    "Trimmed/periodic/seam-crossing surface requires explicit domain handling"
  ],
  "files_expected_to_touch": [
    "specs/08-ms08-advanced-jewelry-modeling/02-febe-matrix-smartflow-history-morph/001-febe-smartflow-autobase-surface-generation/l3.json (READ ONLY)",
    "specs/08-ms08-advanced-jewelry-modeling/02-febe-matrix-smartflow-history-morph/module.json (READ ONLY)",
    "REAL_SOURCE_FILES_MUST_BE_DISCOVERED_AND_LOGGED_IN_PHASE_01 (no invented crate paths)",
    "implementation/evidence/M08/OM9-L4-04-05-001-02/*",
    "tests/fixtures/m08/* (proposed only; create in actual implementation workspace after checkout)"
  ],
  "automated_tests": [
    "Generate base corresponding to selected surface; assert unambiguous UV orientation and mapped corners",
    "Trimmed/periodic/seam-crossing surface requires explicit domain handling",
    "Kernel valid check (when a solid is expected), parent-feature dependency graph consistency, no geometry from viewport mesh.",
    "Nonfinite/degenerate inputs explicitly handled; failure preserves stable ID set and document hash."
  ],
  "manual_steps_and_expected": [
    "With tested desktop build, select correct geometry for Smartflow Autobase Surface Generation; open the documented menu/command/builder (verify actual alias first). Expected: only valid contextual selection enables the action.",
    "Enter typed values corresponding to Destination SurfaceRef, UV domain, direction/normal, base-generation policy. Expected: immediately visible preview and clear constraints; raw source geometry remains unchanged before Commit.",
    "Check result: Generate base corresponding to selected surface; assert unambiguous UV orientation and mapped corners. Record numerical/visual actual vs expected (do not use render alone to establish BRep correctness).",
    "Cancel and compare document state digest; repeat and Commit exactly once, Undo/Redo and Save/Open. Expected: history/IDs/materials/units retained.",
    "Try failure: Trimmed/periodic/seam-crossing surface requires explicit domain handling. Expected: explicit diagnostic and no partial side effects. Log tester, build SHA, GPU, OS, date, screenshot/video and sign-off."
  ],
  "fixture_ids": [
    "M08-FX-04-05-001",
    "M08-NEG-04-05-001",
    "M08-FX-HISTORY",
    "MVP-BASELINE-APPROVED"
  ],
  "pass_criteria": [
    "All declared dependency tasks and external G00/G-MVP gates have actual PASS/reviewer evidence before code is accepted.",
    "Actual source paths/sha, OCCT version, toolchain, commands/exit codes and typed API diff documented.",
    "Deterministic positive and failure tests satisfy the feature-specific oracle and error contract.",
    "Generate base corresponding to selected surface; assert unambiguous UV orientation and mapped corners",
    "Trimmed/periodic/seam-crossing surface requires explicit domain handling",
    "No silent success, no document corruption, no history cycle, no broken MVP regression.",
    "FEBE requires independent tester evidence before task PASS; reviewer signs separately."
  ],
  "release_evidence": [
    "source_sha",
    "tested_sha",
    "timestamp_utc",
    "os_arch_gpu",
    "rust_toolchain",
    "occt_version_build_hash",
    "license_ownership_review",
    "commands_exit_codes_logs",
    "fixture_ids_units_tolerances",
    "expected_actual_numeric_diffs",
    "document_history_hash_before_after",
    "mvp_regression_baseline",
    "screenshot_video_paths_if_febe",
    "tester",
    "reviewer",
    "known_limitations"
  ],
  "failure_policy": "If G00/E12/G-MVP/source/legal/golden fixtures or upstream task PASS missing: BLOCKED. If data corruption, incorrect rigid gem geometry, UV flip/undetected distortion, failed atomic save/history regression: FAIL; never mark DONE from plans alone.",
  "task_relpath": "planning/M08/04-05-001/02-02-owner-operation-or-dispatch/l4.json",
  "mvp_regression_required": true,
  "cross_milestone_owner_retained": false
}
