# OpenMatrix9 development

Rust owns catalog, state and behavior. C++/Qt integrates native FreeCAD views
and command execution. Python registers the workbench.

Preserve existing command identifiers and supported behavior. Keep unsupported
commands visible and disabled. Do not enable a command merely because an icon
or configuration entry exists.

Use the authored SVG catalogs and `tools/export_menu_assets.py`. Never import
commercial bitmaps, binary resources, manuals, decompiled source or private
models into this repository. Preserve approved color roles and command meaning.
Follow `skills/openmatrix9-icons/SKILL.md` for icon design and review.

After changes, run appropriate Rust/Python tests and
`python tools/public_source_audit.py`. Native changes require a matching FreeCAD
SDK and relevant runtime checks. Preserve all applicable license notices.

Do not publish, force-push, or change repository visibility without a human
instruction. A local source audit is not copyright clearance or proof of
complete command implementation.
