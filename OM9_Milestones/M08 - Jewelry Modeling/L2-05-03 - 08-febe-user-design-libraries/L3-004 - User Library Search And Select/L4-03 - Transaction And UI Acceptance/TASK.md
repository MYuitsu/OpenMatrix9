{
  "schema_version": "om9.m08.l4.v1",
  "task_id": "OM9-L4-05-03-004-03",
  "parent_l3_id": "OM9-L3-05-03-004",
  "owner_module_id": "OM9-L2-05-03",
  "parent_capability_id": "cap.05-03.user-library-search-and-select",
  "milestone_id": "M08",
  "target_milestone": "M08",
  "execution_stage": "E13",
  "title": "User Library Search And Select — Tích hợp UI, transaction, history và nghiệm thu",
  "task_name": "Tích hợp UI, transaction, history và nghiệm thu",
  "tag": "febe",
  "implementation_role": "interactive_feature",
  "work_kind": "FEBE_ACCEPTANCE",
  "phase": 3,
  "status": "PLANNED_NOT_EXECUTED",
  "readiness": "BLOCKED_BY_G00_E12_G_MVP_AND_LOCAL_SOURCE",
  "source_repo": "https://github.com/QymIs-Tech/QymCAD",
  "source_revision_candidate": "c116f8f224ebffe0ada049c91c0d15df3634f00d",
  "source_paths_verified_locally": [],
  "reuse_decision": "PENDING_LOCAL_SOURCE_AUDIT",
  "estimated_scope": null,
  "origin_manual_refs": [
    {
      "manual_id": "m9",
      "filename": "matrix9.0_addendum_manual.pdf",
      "printed_pages": [
        85,
        85
      ],
      "pdf_pages_1based": [
        91,
        91
      ],
      "precision": "parent_l2_range",
      "provenance": "INHERITED_L3; source addendum chapter 3 PDF 86–92 consulted during planning"
    }
  ],
  "source_command_candidate_from_l3": null,
  "source_command_verification_status": "INHERITED_FROM_L3_UNCONFIRMED_IN_LOCAL_RUNTIME",
  "purpose": "search user-owned models, profiles and ornaments",
  "owner_rule": "This L3 exclusively owns the described capability within its parent L2; any other menu/panel calls it through the parent service contract.",
  "expected_domain_inputs": "Catalog query, categories, gem info filter, stable item ordering",
  "expected_behavior_oracle": "Search and gem filter return deterministic sets, category switch preserves selection rules",
  "high_risk_negative_case": "Missing gem metadata excludes or marks unknown explicitly, no fabricated gems",
  "inputs": [
    "specs/08-ms08-advanced-jewelry-modeling/08-febe-user-design-libraries/004-febe-user-library-search-and-select/l3.json",
    "specs/08-ms08-advanced-jewelry-modeling/08-febe-user-design-libraries/module.json",
    "specs/00-contracts/MATRIX9_FIRST_CONTRACTS.md",
    "specs/00-contracts/MATRIX_WORKBENCH_UI.md",
    "planning/E13.md",
    "planning/GATES_AND_EVIDENCE.md",
    "planning/DEPENDENCY_AND_RELEASE_POLICY.md",
    "planning/M00/SOURCE_AUDIT_V012.md",
    "planning/M08/LUNA_START_HERE.md",
    "Local QymCAD checkout at verified SHA (not included; check actual source)",
    "Catalog query, categories, gem info filter, stable item ordering"
  ],
  "outputs": [
    "implementation/evidence/M08/OM9-L4-05-03-004-03/ui-manual-checklist.md",
    "implementation/evidence/M08/OM9-L4-05-03-004-03/test-run.json"
  ],
  "api_contract_and_callers": [
    "Canonical owner: OM9-L2-05-03; capability: cap.05-03.user-library-search-and-select; role: interactive_feature.",
    "Dependent owner modules (check actual API after source audit): OM9-L2-01-12, OM9-L2-03-15.",
    "Typed contract MUST include IDs/units/frames/tolerance/transaction/errors, and preview snapshot; specific Rust structs/functions not asserted until local inspection.",
    "Only this L3 owns described feature. Other menus/services are callers; use shared kernel and feature graph."
  ],
  "steps": [
    "Require owner service/task phase 02 PASS and verified binary. Provide exact selection/command/builder controls consistent with manual; no UI-only simulated geometry.",
    "Exercise end-to-end interaction using: Catalog query, categories, gem info filter, stable item ordering; link preview to typed immutable data and commit to one undoable transaction.",
    "Automated deterministic e2e assertions: Search and gem filter return deterministic sets, category switch preserves selection rules; compare geometry, history, parameters and IDs before/after Save/Open, Undo/Redo.",
    "Failure path: Missing gem metadata excludes or marks unknown explicitly, no fabricated gems; verify user-visible diagnostic, Cancel identity and no phantom entities.",
    "Test physical desktop build with tester/date/build SHA; keep READY_FOR_MANUAL_TEST until a human records expected/actual and signs."
  ],
  "preconditions": [
    "G00 signed PASS + local license and OCCT-Rust FFI evidence",
    "E01–E12 signed PASS; M07 & previous MVP regression fixtures available",
    "G-MVP reviewer signed PASS; E13 must not compromise MVP baseline",
    "Local QymCAD checkout at verified repository SHA; actual Rust/OCCT toolchain, golden fixture thresholds recorded",
    "Every depends_on_task_ids task PASS with evidence; if not stop BLOCKED_DEPENDENCY"
  ],
  "depends_on_task_ids": [
    "OM9-L4-05-03-004-02"
  ],
  "depends_on_module_ids": [
    "OM9-L2-01-12",
    "OM9-L2-03-15"
  ],
  "geometry_invariants": [
    "Canonical shapes/curves via Rust domain + OCCT FFI; never treat renderer tessellation or viewport cache as authoritative BRep.",
    "Use documented source/target frames; mm model units; numeric and topology tolerances from approved G00, not guessed values.",
    "EntityId/FeatureId/LayerId/InstanceId & provenance stable; preserve non-selected entities, layer locks and instance semantics.",
    "Preview read-only against committed model; Cancel same document hash; one Commit equals one atomic Undo transaction; error rolls back.",
    "History dependency directed acyclic graph, serializable stable parameter schema, deterministic recompute, no silent source detach.",
    "MVP golden fixtures and existing M07 STL/cutters must not regress after E13.",
    "Library paths must stay within configured root; file I/O transactional with atomic write/rename; import/open must honor unit scaling and ID remapping.",
    "Do not package licensed Gemvision libraries/images or hidden user data.",
    "Search and gem filter return deterministic sets, category switch preserves selection rules"
  ],
  "error_cases": [
    "Empty/wrong type selection; document locked, missing asset/feature ID, stale reference.",
    "Zero/NaN/Inf/singular transforms, tolerance or invalid geometry.",
    "OCCT exception, FFI failure, OOM/resource limit, canceled transaction.",
    "Missing source/gate/evidence: BLOCKED, never fabricate implementation or source filenames.",
    "Missing gem metadata excludes or marks unknown explicitly, no fabricated gems"
  ],
  "files_expected_to_touch": [
    "specs/08-ms08-advanced-jewelry-modeling/08-febe-user-design-libraries/004-febe-user-library-search-and-select/l3.json (READ ONLY)",
    "specs/08-ms08-advanced-jewelry-modeling/08-febe-user-design-libraries/module.json (READ ONLY)",
    "REAL_SOURCE_FILES_MUST_BE_DISCOVERED_AND_LOGGED_IN_PHASE_01 (no invented crate paths)",
    "implementation/evidence/M08/OM9-L4-05-03-004-03/*",
    "tests/fixtures/m08/* (proposed only; create in actual implementation workspace after checkout)"
  ],
  "automated_tests": [
    "Search and gem filter return deterministic sets, category switch preserves selection rules",
    "Missing gem metadata excludes or marks unknown explicitly, no fabricated gems",
    "Preview vs Commit, Cancel, Undo/Redo, Save/Open and stable IDs/feature graph.",
    "Real UI smoke: menu/F6 where documented, keyboard/cursor interactions, text feedback, test evidence screenshots."
  ],
  "manual_steps_and_expected": [
    "With tested desktop build, select correct geometry for User Library Search And Select; open the documented menu/command/builder (verify actual alias first). Expected: only valid contextual selection enables the action.",
    "Enter typed values corresponding to Catalog query, categories, gem info filter, stable item ordering. Expected: immediately visible preview and clear constraints; raw source geometry remains unchanged before Commit.",
    "Check result: Search and gem filter return deterministic sets, category switch preserves selection rules. Record numerical/visual actual vs expected (do not use render alone to establish BRep correctness).",
    "Cancel and compare document state digest; repeat and Commit exactly once, Undo/Redo and Save/Open. Expected: history/IDs/materials/units retained.",
    "Try failure: Missing gem metadata excludes or marks unknown explicitly, no fabricated gems. Expected: explicit diagnostic and no partial side effects. Log tester, build SHA, GPU, OS, date, screenshot/video and sign-off."
  ],
  "fixture_ids": [
    "M08-FX-05-03-004",
    "M08-NEG-05-03-004",
    "M08-FX-HISTORY",
    "MVP-BASELINE-APPROVED"
  ],
  "pass_criteria": [
    "All declared dependency tasks and external G00/G-MVP gates have actual PASS/reviewer evidence before code is accepted.",
    "Actual source paths/sha, OCCT version, toolchain, commands/exit codes and typed API diff documented.",
    "Deterministic positive and failure tests satisfy the feature-specific oracle and error contract.",
    "Search and gem filter return deterministic sets, category switch preserves selection rules",
    "Missing gem metadata excludes or marks unknown explicitly, no fabricated gems",
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
  "task_relpath": "planning/M08/05-03-004/03-03-transaction-and-ui-acceptance/l4.json",
  "mvp_regression_required": true,
  "cross_milestone_owner_retained": false
}
