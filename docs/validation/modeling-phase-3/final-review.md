# Independent whole-change review — 2026-10-09

Reviewer: `/root/phase3_whole_review`, fresh context, gpt-6-astra/high. Read-only; one review under executing-plans. Assessment: **not ready to declare Phase3 complete before the fix pass**.

The reviewer independently verified all83 inventory hashes, source/runtime manifest, gate/report/script/oracle and Rhino output hashes. Attempt7 is valid for its15 fixtures/599 Rhino/481 host checks; clipboard and reread use the same manifest. Strengths: Rust copied snapshots/bounded ABI/type policy, lifecycle checks/transactions/dependent protection, independent converter seam/pole/torus/cavity/knot/pcurve evidence.14 Rust test files omitted from the first package were included during this same review.

## Findings

1. **Critical:** `Gui/EditGeometry.cpp:122`, caller `EditController.cpp:143`: empty Trim output bypasses per-shape validation, permits deleting all inputs with no replacement. Reject empty results before transaction and disable empty preview; actual remove-all regression must preserve geometry/Undo/Redo.
2. **Important:** `Gui/SurfaceController.cpp:311`, `SurfaceGeometry.cpp:88`: whole-curve mouse probe has no guard, is always rejected, then swallowed; multi-edge profiles/rails become EdgeN, unlike whole-object/CMD. Capture a guard for the probe and test real closed multi-edge viewport picks.
3. **Important:** `Gui/EditController.cpp:78/83`, `SurfaceController.cpp:99`: initial selection adds individually and drops invalid/protected/mesh inputs. Remaining valid-enough subset can be committed/deleted. Initial batches must be atomic; test valid-enough plus invalid/protected objects.
4. **Important:** `Gui/EditGeometry.cpp:71`, `EditController.cpp:109`: surface Join ignores recorded/user tolerance. Part Python sewShape also drops the parsed argument (`src/Mod/Part/App/TopoShapePyImp.cpp:897`). Use actual native sewing with Rust-validated tolerance and gap below/above test, preserving inputs on failure.
5. **Important:** `tests/modeling_ring_workflow_smoke.FCMacro:49`: cutter is directly constructed and transactioned by test code. Use a supported user command/controller, test dimensions/placement and actual command Undo/Redo.
6. **Important:** same macro:54/76: expected cut bounds/area are copied from its output; the opposite half has equal volume and can pass. Carry independent negative-Y oracle: bounds[-12,-12,-2,12,0,2], volume40*pi^2, area40*pi^2+8*pi, plus region/section assertion.

## Requirement disposition

P3.1-A/C, P3.2-C/F, P3.3-C/D and recorded application/regression artifacts have scoped evidence. P3.1-B/P3.2-A/B/D/E/G/H and P3.3-A/B remain incomplete until the above fixes and matching binary/gate pass. REVIEW-P3 requires the fix pass and synchronized completion audit.

## Declined to judge

- History/render/materials/textures/Surface CV/fillet/offset: expressly excluded.
- Seven broader full-openNURBS packages: expressly future work.
- Command-first Join with no preselection: accepted Phase2 Join requires preselection; no new interaction inferred.
- Universal parity of advanced Loft/Sweep options: named workflows are evidenced, not universal parity.
- Final completion-document consistency: pending this review; root must finish it.
- Native memory safety for invalid caller pointers: valid initialized caller-owned storage is required; Rust checks do not certify arbitrary pointers.

No deferred minor findings. The reviewer did not mutate code, run applications or regenerate fixtures. Root applies one RED→GREEN fix pass and fresh whole-suite/app evidence, without a second reviewer.
