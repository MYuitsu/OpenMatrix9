# OpenMatrix9 development

## User instruction: Rust first (2026-10-09)

Rust is the number-one implementation choice for new and migrated OpenMatrix9 code. Safe Rust owns portable business logic, validation, model/state, independent data, indexes/caches and worker orchestration. C++ is limited to necessary FreeCAD/Qt/OCCT/openNURBS native adapters; Python or other languages require an actual bootstrap/API/test/tool constraint. Read skills/openmatrix9-workflow/references/rust-first.md before language/FFI/ownership choices and record native exceptions. Existing C++/Python convenience does not override Rust priority. Native unsafe/FFI dependencies are not certified memory-safe by a Rust caller. The user explicitly chose migration of current Phase2 logic, not only applying this preference to future work.

Rust owns catalog, state and behavior. C++/Qt integrates native FreeCAD views
and command execution. Python registers the workbench.

Preserve existing command identifiers and supported behavior. Keep unsupported
commands visible and disabled. Do not enable a command merely because an icon
or configuration entry exists.

Use the authored SVG catalogs and `tools/export_menu_assets.py`. Never import
commercial bitmaps, binary resources, Matrix manuals, decompiled source or private
models into this repository. The user-approved Rhino 5 documentation exception
is limited to the pinned guide and reference metadata in `ref/rhino5`; preserve
its original notices and follow [the core reference index](ref/rhino5/README.md).
Preserve approved color roles and command meaning.
Follow `skills/openmatrix9-icons/SKILL.md` for icon design and review.

After changes, run appropriate Rust/Python tests and
`python tools/public_source_audit.py`. Native changes require a matching FreeCAD
SDK and relevant runtime checks. Preserve all applicable license notices.

Do not publish, force-push, or change repository visibility without a human
instruction. A local source audit is not copyright clearance or proof of
complete command implementation.

Follow [the private reference policy](docs/private-reference-policy.md) for
external reference storage and historical source citations. Resolve private
inputs through the ignored local `docs/openmatrix9-reference-locations.json`;
never copy, stage or push private references into this repository.

Before implementing Rhino-compatible core behavior, use the RCORE mapping in
`ref/rhino5/CORE_REFERENCE_INDEX.json`, read the relevant guide pages and exact
Rhino 5 Command Help, then compare the FreeCAD adapter and applicable fixtures.
Keep documented behavior, decompiler inference and OM9 choices distinct. A
guide, API name or source inspection does not prove runtime parity; retain
the current capability/checkpoint status until matching evidence exists.
