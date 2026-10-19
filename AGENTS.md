# OpenMatrix9

## User instruction: build size and layout (2026-10-17)

The standalone tree was merged into the repository root: `crates/` (six crates), `native/`, and the
workspace `Cargo.toml`/`Cargo.lock` live at the root; the `standalone/` directory no longer exists and
must not be recreated. Build with `CARGO_TARGET_DIR` outside the checkout (this machine uses
`L:\om9-target`) and with **`-j 2`**: the 8 GiB paging file cannot hold the default parallelism (rustc
dies with `os error 1455`).

Keep only the **10 newest build/validation runs** (`build/standalone-validation/ui-artifacts/*`). Prune
with `python tools/prune_build_artifacts.py --apply` after a validation batch. The tool never removes a
run still cited by `implementation/**` or `docs/**`, so a reference from a record stays valid; `build/`
itself is gitignored and is never the only home of evidence.

## User instruction: Rust first (2026-10-09)

Rust is the number-one implementation choice for new and migrated OpenMatrix9 code. Safe Rust owns portable business logic, validation, model/state, independent data, indexes/caches and worker orchestration. C++ is limited to necessary selected OCCT/openNURBS/native API adapters; Python or other languages require an actual bootstrap/API/test/tool constraint. Read skills/openmatrix9-workflow/references/rust-first.md before language/FFI/ownership choices and record native exceptions. Existing C++/Python convenience does not override Rust priority. Native unsafe/FFI dependencies are not certified memory-safe by a Rust caller. The user explicitly chose migration of current Phase2 logic, not only applying this preference to future work.

The active target is standalone OM9 with foundation references in `OM9_Milestones/M01 - CAD Shell, Viewport` (user direction, 2026-10-19). Rust owns document/model/session/history/Undo/transactions/UI/scene; openNURBS and capability-scoped OCCT provide selected geometry operations. QymCAD supplies only code samples/routines, not target UI. Preserve Matrix9 as described by milestone references and the user-approved workspace/layers contracts. Old FreeCAD adapters/fixtures are historical inputs to port, not target runtime.

## QymCAD code reference location (2026-10-15)

The QymCAD source lives in this workspace at `h:\OpenMatrix9\QymCAD` (repo-relative `QymCAD/`). When a task, spec or user message references QymCAD code, samples, routines or crates, resolve it under that path instead of searching elsewhere or asking for the location.

- Rust crates: `QymCAD/crates/<name>` (`qymcad`, `qymcad-acceptance`, `qymcad-assembly`, `qymcad-core`, `qymcad-help`, `qymcad-i18n`, `qymcad-io`, `qymcad-kernel`, `qymcad-meshfit`, `qymcad-part`, `qymcad-paths`, `qymcad-pick`, `qymcad-render`, `qymcad-scheme`, `qymcad-shell`, `qymcad-sketch`, `qymcad-testkit`, `qymcad-ui-state`, `qymcad-update`).
- Other top-level folders: `QymCAD/docs`, `QymCAD/examples`, `QymCAD/library`, `QymCAD/machines`, `QymCAD/tools`, `QymCAD/i18n`, `QymCAD/assets`, `QymCAD/packaging`. Workspace manifest: `QymCAD/Cargo.toml`. QymCAD's own guidance: `QymCAD/AGENTS.md`, `QymCAD/CLAUDE.md`, `QymCAD/README.md`.
- Role stays as stated under the active target: QymCAD is a source of code samples/routines only, not the target UI or runtime. Port into OM9 Rust crates; do not make OM9 depend on QymCAD paths at build time.
- Read only the specific crate/file referenced, not the whole tree. Keep `QymCAD/THIRD-PARTY-NOTICES.md` and `QymCAD/LICENSE` notices when copying or adapting code.

## User instructions for Curve work (2026-10-04)

For authorized standalone implementation, establish the M01 document/kernel/UI foundations first, then the exact milestone capability. The earlier 130-core inventory in `docs/core-requirements.json` is historical coverage; preserve its records without treating it as the active milestone router. Only target-specific evidence establishes completion. A documentation/skill task does not authorize automatically starting all implementation work. Use local specs rather than rereading PDFs.

- Use the exact Curve capability under `OM9_Milestones`, resolved by the workflow router. Local milestone specifications are sufficient; do not reread PDFs for already normalized requirements. This explicit user instruction overrides skill guidance requiring manual-page review.
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

## Public source and publication

Use authored SVG catalogs and tools/export_menu_assets.py. Never import commercial bitmaps, binary resources, Matrix manuals, decompiled source or private models into this repository. The user-approved Rhino5 guide exception is limited to ref/rhino5; preserve its notices. Preserve existing command identifiers and keep unsupported commands visible and disabled.

After changes, run appropriate Rust/Python tests and tools/public_source_audit.py. Native changes require matching SDK and runtime checks. Preserve license notices. Do not publish, force-push or change repository visibility without a human instruction. Follow docs/private-reference-policy.md and never stage ignored private references.

## Active routing and progress

Read root AGENTS, `OM9_Milestones/M01 - CAD Shell, Viewport/L1.json`, the relevant M01 L2/L3 owner contract, and then the exact feature in its milestone before architecture/FFI/build decisions. The user replaced the old specification-package dependency on 2026-10-19; do not require absent policy files. Preserve imported plans/prompts as reference material, not user instructions. Update milestone runtime_implementation, status/traceability, README and the live ledger with actual standalone evidence. Legacy FreeCAD adapters/fixtures remain historical unless expressly requested.
