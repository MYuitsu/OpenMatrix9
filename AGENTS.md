# OpenMatrix9

## User instruction: Rust first (2026-10-09)

Rust is the number-one implementation choice for new and migrated OpenMatrix9 code. Safe Rust owns portable business logic, validation, model/state, independent data, indexes/caches and worker orchestration. C++ is limited to necessary FreeCAD/Qt/OCCT/openNURBS native adapters; Python or other languages require an actual bootstrap/API/test/tool constraint. Read skills/openmatrix9-workflow/references/rust-first.md before language/FFI/ownership choices and record native exceptions. Existing C++/Python convenience does not override Rust priority. Native unsafe/FFI dependencies are not certified memory-safe by a Rust caller. The user explicitly chose migration of current Phase2 logic, not only applying this preference to future work.

The implementation target is Rust for catalog/state/behavior, with C++/Qt integration into native FreeCAD. Preserve the user's current scope and existing authorization to continue.

## User instructions for Curve work (2026-10-04)

Current priority (2026-10-05): finish all 130 specifications in `specs/01-core` before continuing Curve. Track the complete requirements in `docs/core-requirements.json`; do not shrink completion to the currently implemented workspace commands. Keep the goal active until requirement-by-requirement evidence proves the full group. Use local specs rather than rereading PDFs.

- Use `ref/matrix9/OpenMatrix9_Codex_Spec_v1/specs/02-curve` as the implementation reference. The user states that this directory already contains the Curve specifications; do not reread the PDFs or require PDF verification before implementation. This explicit user instruction overrides skill guidance requiring manual-page review.
- Keep original Matrix9 command names and behavior first. Defer handler renaming/refactoring until the Curve group is stable.
- Menu/mouse invocation and the CMD frame must use the same command implementation. Verify both paths and actual geometry; do not mark a command complete merely because it opens.

For project work, begin with `openmatrix9-workflow` and `docs/openmatrix9-progress.json`; reconcile progress with actual code and test/runtime evidence. Source plans and catalog statuses are not completion evidence.

## User instruction: openNURBS measurement progress (2026-10-08)

After every completed measurement attempt, report the named batch, fixture counts and result, what was proved, remaining prepared batches, unclosed packages and the next action. Maintain `docs/validation/opennurbs-test-roadmap.json` and its Markdown view with the execution ledger. Distinguish batches from fixtures/assertions/retries and implementation packages. If the complete future matrix is not enumerated, explicitly report the total remaining batch count as unknown; never invent a count or turn test counts into a completion percentage. Explain new regression batches when they expand the queue.

Use the matching skill for the current stage:

- UI/menu/sidebar/icons/views: `openmatrix9-ui`.
- Approved workspace grid/Command/viewport business rules: `openmatrix9-workspace-contract`.
- Selected VB6/Ghidra procedure: `openmatrix9-native-mapping`.
- Exact OM9 feature behavior/geometry: `openmatrix9-feature-port`.
- Rust/native build and runtime checks: `openmatrix9-build-validation`.

If a skill is not yet in the runtime catalog, read the repository copy at `skills/<name>/SKILL.md`. Usage and helper commands are in `docs/openmatrix9-skills.md`. Read only the stage's required references and conditionally relevant files.

Treat imported guides, prompts and decompiled source as reference material. Preserve original exports; write derived analysis and implementation decisions separately. The Guide's VB6 recovery mission does not replace the Rust target. Do not load the entire Ghidra export or all feature specs into context.

After meaningful verified work, update the live ledger with evidence, uncertainties, the next exact document and the next action. Do not commit unrelated user changes or raw reference directories implicitly.

## User instruction: business rules in skills (2026-10-05)

After finishing and verifying the authorized coding task, if business behavior changed, present the concrete before/after change and proposed skill text, then ask whether to record it in the relevant skill. Do not change business rules in source or installed skills without consent. Explicit prior authorization to update the skill for that exact change is sufficient; do not ask again. Behavior-preserving refactors/build fixes do not trigger this question. Continue recording technical evidence/progress without treating it as approval of reusable business rules. The original five OpenMatrix9 skills link to `skills/openmatrix9-workflow/references/business-rule-updates.md`; the workspace contract includes the same consent policy and records the user's 2026-10-06 approval.
