---
name: openmatrix9-icons
description: Create, replace or review minimal SVG menu icons for OpenMatrix9, preserving Matrix9 color identity and command meaning so migrating users recognize commands at menu size. Use for icon sets and confusing CAD symbols, not command implementation or bitmap tracing.
---

# OpenMatrix9 icons

Build recognizable vector symbols from verified command meaning. Resolve the
active OpenMatrix9 module root (the directory directly containing `Resources`
and `tools`) from the workspace; do not assume an old D: or E: location.
Preserve the original Matrix9 icon's characteristic colors by default, so users
moving from Matrix9 can identify familiar commands. A request to simplify or
redraw icons does not authorize a new palette. Explicit user color choices take
precedence, limited to the group or element they selected; do not spread an
exception to other groups. Changing artwork does not authorize changing command
behavior, menu order or disabled-command availability.

Read [the style and review guide](references/style-and-review.md) before drawing.
For an existing group, inspect `Resources/menu/MainMenu.ini`, `icons.ini`, its
generator and the local feature specs. Menu order and spec IDs can differ;
match actual command keys. Do not invent meaning from a filename: for example
`gvExtractIsocurve` extracts **at an angle**, not multiple isocurves.

Before drawing a new group, resolve its original/reference icon bindings and
inspect representative icons plus any commands with different color roles.
Sample the characteristic main, highlight and action colors from those pixels;
record the reference paths and palette values. The menu header's `Color` and
the preceding group's palette are not enough to determine icon colors. Compare
the original references and new SVGs side by side at 24px. Preserve both group
identity and meaningful per-command accents; simplify shading into flat colors
from the reference family rather than recoloring everything uniformly.

Use authored SVG geometry, not traced or recolored proprietary artwork. Preserve
the approved color roles and familiar functional cues. Reuse the active style's
primitives; keep source/result curves separated, use large direction arrows,
and omit background grids unless they distinguish the command. Preserve related
commands' necessary differences rather than using one generic category symbol.

For Curve, edit `tools/curve_icons.py`, then regenerate with:

```powershell
rtk proxy python tools/curve_icons.py
```

For Solid, use `tools/solid_icons.py` and `python tools/solid_icons.py` (with the
environment's required shell prefix). Solid retains its reference purple family;
do not reuse Curve's yellow/cyan palette. See the review guide for current values
and the scoped Curve cyan exception.

For another group, use a separate scoped catalog/generator. Do not force it into
the 58-entry Curve catalog or repaint unrelated groups. Ensure authored icons
take priority in `tools/export_menu_assets.py` so later exports cannot restore
old PNGs. Update binding provenance and hashes; keep original references unless
the user requests their removal.

Audit the chosen group with the bundled stdlib helper:

```powershell
rtk proxy python <skill-dir>/scripts/audit_icons.py --project-root <module-root> --group Curve
```

It writes an audit report and SVG sheets at 24, 32 and 64px on gray and dark
backgrounds under `build/icon-review/`. Use `--help` for output selection.
Render the sheets with available Qt or the bundled Sharp runtime and inspect
the actual 24px samples, comparing related commands side by side. Static audits,
unique paths, distinct geometry hashes and successful rendering do **not** prove
correct meaning or user recognition. Label unresolved semantic questions.

Check menu coverage, valid self-contained SVGs, manifest hashes, deterministic
regeneration and unchanged non-target bindings. If integrating into a runnable
build, copy only scoped resources, verify bytes, and exercise Qt loading, button
images and disabled colors; use the existing Curve smoke when applicable.
Record what was actually checked and link a gallery. Do not claim arbitrary DPI
support, complete feature implementation or copyright clearance from icon tests.

Maintain the repository skill and installed Codex copy together when updating
this skill, within the user's authorization. The installed copy is discoverable
as `$openmatrix9-icons`.
