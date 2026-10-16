{
  "schema_version": "om9.m08.l4.v1",
  "task_id": "OM9-L4-05-02-004-04",
  "parent_l3_id": "OM9-L3-05-02-004",
  "owner_module_id": "OM9-L2-05-02",
  "parent_capability_id": "cap.05-02.rope-builder-end-cap-modes",
  "milestone_id": "M08",
  "target_milestone": "M08",
  "execution_stage": "E13",
  "title": "Rope Builder End Cap Modes — Phá thử, regression và chặn lỗi tái phát",
  "task_name": "Phá thử, regression và chặn lỗi tái phát",
  "tag": "febe",
  "implementation_role": "interactive_feature",
  "work_kind": "ADVERSARIAL_GATE",
  "phase": 4,
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
        82,
        82
      ],
      "pdf_pages_1based": [
        88,
        88
      ],
      "precision": "parent_l2_range",
      "provenance": "INHERITED_L3; source addendum chapter 3 PDF 86–92 consulted during planning"
    }
  ],
  "source_command_candidate_from_l3": null,
  "source_command_verification_status": "INHERITED_FROM_L3_UNCONFIRMED_IN_LOCAL_RUNTIME",
  "purpose": "none/flat/rounded thread end caps",
  "owner_rule": "This L3 exclusively owns the described capability within its parent L2; any other menu/panel calls it through the parent service contract.",
  "expected_domain_inputs": "Cap mode None/Flat/Round and strand endpoints",
  "expected_behavior_oracle": "None open as documented, Flat planar closure, Round end cap geometry validated",
  "high_risk_negative_case": "Do not auto-label None water-tight; invalid end caps shown before STL export",
  "inputs": [
    "specs/08-ms08-advanced-jewelry-modeling/07-febe-matrix9-rope-builder/004-febe-rope-builder-end-cap-modes/l3.json",
    "specs/08-ms08-advanced-jewelry-modeling/07-febe-matrix9-rope-builder/module.json",
    "specs/00-contracts/MATRIX9_FIRST_CONTRACTS.md",
    "specs/00-contracts/MATRIX_WORKBENCH_UI.md",
    "planning/E13.md",
    "planning/GATES_AND_EVIDENCE.md",
    "planning/DEPENDENCY_AND_RELEASE_POLICY.md",
    "planning/M00/SOURCE_AUDIT_V012.md",
    "planning/M08/LUNA_START_HERE.md",
    "Local QymCAD checkout at verified SHA (not included; check actual source)",
    "Cap mode None/Flat/Round and strand endpoints"
  ],
  "outputs": [
    "implementation/evidence/M08/OM9-L4-05-02-004-04/adversarial-report.md",
    "implementation/evidence/M08/OM9-L4-05-02-004-04/regression-run.json"
  ],
  "api_contract_and_callers": [
    "Canonical owner: OM9-L2-05-02; capability: cap.05-02.rope-builder-end-cap-modes; role: interactive_feature.",
    "Dependent owner modules (check actual API after source audit): OM9-L2-02-04, OM9-L2-03-11.",
    "Typed contract MUST include IDs/units/frames/tolerance/transaction/errors, and preview snapshot; specific Rust structs/functions not asserted until local inspection.",
    "Only this L3 owns described feature. Other menus/services are callers; use shared kernel and feature graph."
  ],
  "steps": [
    "Require phase 03 manual/e2e evidence and upstream feature prerequisites PASS; run high-risk scenario in clean application state.",
    "Adversarial/semantic check: None open as documented, Flat planar closure, Round end cap geometry validated; include reversed/closed paths, seam, tolerance and multi-object where applicable.",
    "Fault injection: Do not auto-label None water-tight; invalid end caps shown before STL export; test cancellation during recompute and I/O error if serializing/saving.",
    "Cross-MVP regression: reopen signed G-MVP golden models, compare geometry/document hashes and prior-feature outputs; no E13 breaking change to MVP.",
    "Require independent reviewer for manual/video + numeric log; FAILED gate remains BLOCKED, never self-sign G-FULL."
  ],
  "preconditions": [
    "G00 signed PASS + local license and OCCT-Rust FFI evidence",
    "E01–E12 signed PASS; M07 & previous MVP regression fixtures available",
    "G-MVP reviewer signed PASS; E13 must not compromise MVP baseline",
    "Local QymCAD checkout at verified repository SHA; actual Rust/OCCT toolchain, golden fixture thresholds recorded",
    "Every depends_on_task_ids task PASS with evidence; if not stop BLOCKED_DEPENDENCY"
  ],
  "depends_on_task_ids": [
    "OM9-L4-05-02-004-03"
  ],
  "depends_on_module_ids": [
    "OM9-L2-02-04",
    "OM9-L2-03-11"
  ],
  "geometry_invariants": [
    "Canonical shapes/curves via Rust domain + OCCT FFI; never treat renderer tessellation or viewport cache as authoritative BRep.",
    "Use documented source/target frames; mm model units; numeric and topology tolerances from approved G00, not guessed values.",
    "EntityId/FeatureId/LayerId/InstanceId & provenance stable; preserve non-selected entities, layer locks and instance semantics.",
    "Preview read-only against committed model; Cancel same document hash; one Commit equals one atomic Undo transaction; error rolls back.",
    "History dependency directed acyclic graph, serializable stable parameter schema, deterministic recompute, no silent source detach.",
    "MVP golden fixtures and existing M07 STL/cutters must not regress after E13.",
    "Open rope (Cap=None) stays visibly non-watertight; Flat/Round cap validity verified before exporting to STL.",
    "None open as documented, Flat planar closure, Round end cap geometry validated"
  ],
  "error_cases": [
    "Empty/wrong type selection; document locked, missing asset/feature ID, stale reference.",
    "Zero/NaN/Inf/singular transforms, tolerance or invalid geometry.",
    "OCCT exception, FFI failure, OOM/resource limit, canceled transaction.",
    "Missing source/gate/evidence: BLOCKED, never fabricate implementation or source filenames.",
    "Do not auto-label None water-tight; invalid end caps shown before STL export"
  ],
  "files_expected_to_touch": [
    "specs/08-ms08-advanced-jewelry-modeling/07-febe-matrix9-rope-builder/004-febe-rope-builder-end-cap-modes/l3.json (READ ONLY)",
    "specs/08-ms08-advanced-jewelry-modeling/07-febe-matrix9-rope-builder/module.json (READ ONLY)",
    "REAL_SOURCE_FILES_MUST_BE_DISCOVERED_AND_LOGGED_IN_PHASE_01 (no invented crate paths)",
    "implementation/evidence/M08/OM9-L4-05-02-004-04/*",
    "tests/fixtures/m08/* (proposed only; create in actual implementation workspace after checkout)"
  ],
  "automated_tests": [
    "None open as documented, Flat planar closure, Round end cap geometry validated",
    "Do not auto-label None water-tight; invalid end caps shown before STL export",
    "Randomized/property tests with deterministic seeds, repeat 2× save/open and Undo/Redo without topology drift.",
    "Cross-MVP fixtures unchanged under E13 unless versioned migration + reviewer approval."
  ],
  "manual_steps_and_expected": [
    "With tested desktop build, select correct geometry for Rope Builder End Cap Modes; open the documented menu/command/builder (verify actual alias first). Expected: only valid contextual selection enables the action.",
    "Enter typed values corresponding to Cap mode None/Flat/Round and strand endpoints. Expected: immediately visible preview and clear constraints; raw source geometry remains unchanged before Commit.",
    "Check result: None open as documented, Flat planar closure, Round end cap geometry validated. Record numerical/visual actual vs expected (do not use render alone to establish BRep correctness).",
    "Cancel and compare document state digest; repeat and Commit exactly once, Undo/Redo and Save/Open. Expected: history/IDs/materials/units retained.",
    "Try failure: Do not auto-label None water-tight; invalid end caps shown before STL export. Expected: explicit diagnostic and no partial side effects. Log tester, build SHA, GPU, OS, date, screenshot/video and sign-off."
  ],
  "fixture_ids": [
    "M08-FX-05-02-004",
    "M08-NEG-05-02-004",
    "M08-FX-HISTORY",
    "MVP-BASELINE-APPROVED"
  ],
  "pass_criteria": [
    "All declared dependency tasks and external G00/G-MVP gates have actual PASS/reviewer evidence before code is accepted.",
    "Actual source paths/sha, OCCT version, toolchain, commands/exit codes and typed API diff documented.",
    "Deterministic positive and failure tests satisfy the feature-specific oracle and error contract.",
    "None open as documented, Flat planar closure, Round end cap geometry validated",
    "Do not auto-label None water-tight; invalid end caps shown before STL export",
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
  "task_relpath": "planning/M08/05-02-004/04-04-adversarial-history-regression/l4.json",
  "mvp_regression_required": true,
  "cross_milestone_owner_retained": false
}
