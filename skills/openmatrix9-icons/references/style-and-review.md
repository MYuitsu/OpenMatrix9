# Minimal icon style and recognition checks

## Current OpenMatrix9 convention

Use transparent SVG with `viewBox="0 0 32 32"`, no raster images, gradients or
shadows. Preserve Matrix9 color identity while replacing detailed artwork with
simple functional geometry. This helps migrating users recognize commands.

## Reference palette selection

Resolve actual icon files through the menu bindings/provenance and inspect the
pixels before selecting colors. Inspect multiple examples in the group and
commands with special accents. Exclude transparent/background pixels when
sampling; choose representative interior colors rather than isolated edge
antialiasing pixels. Original shading can become a small flat palette, retaining
the same hue family and the source/result/selection/action distinctions.

Record the chosen hexadecimal colors, their roles and the sampled reference
paths in validation evidence or generator metadata. Prefer original assets when
available. If using the user's selected RGB+5 references, identify that source
accurately; do not claim that sampled values are exact unshifted original colors.
Do not derive an entire palette from `MainMenu.ini`'s header `Color` alone.

Current approved palettes:

| Group / element | Color roles | Scope |
|---|---|---|
| Curve | Yellow `#FFFF05` curves/results; white `#FFFFFF` points, construction and arrows; gold `#FFD705` history | Curve convention |
| Curve surface/object context | Cyan `#59CBE8` | Explicit user-selected exception for Curve supporting geometry; not a default Surface/Solid palette |
| Solid | Purple `#A668D1` solids; light purple `#C78AF4` result/selected feature; white actions/grips | Sampled from selected Solid RGB+5 references, including `SolidBoxCornertoCornerHeight_1.png` and `SolidUnion_1.png`; user requested retaining original purple identity |
| Other groups | Sample the original/reference icons for that group and command | Do not inherit Curve or Solid colors automatically |

Before accepting a group, compare the reference icons and new vectors at native
24px on supported backgrounds. Check characteristic hue, relative emphasis,
semantic color roles and recognition together. Improve spacing, silhouette,
stroke weight or reference-family highlights when contrast is weak. A different
hue family needs an explicit user request. If reference assets cannot be found,
report the affected palette gap and request the missing reference or a color
choice; do not silently invent a palette.

## Geometry at menu size

At the native 24px size, the current generator uses primary strokes of 2.6
viewBox units, secondary strokes at least 1.4, arrow strokes 1.8, round joins
and caps. Round points use radius at least 1.9; square control handles use 4×4.
Keep painted strokes and markers inside the viewBox with a useful margin.
These are defaults to tune by visual inspection, not proof of readability.

Use one dominant result and only supporting geometry needed to explain
the operation. Prefer large forms and broad spacing to dense hatch/grid lines.
For a surface, an outlined bowed patch is often enough. A mesh needs a few
large triangles; an isocurve may need a parameter line. Keep arrowheads away
from the background and do not let thick strokes merge adjacent curves.

## Related-command review

| Family | Distinction to retain |
|---|---|
| Offset / on surface / normal | Separated source/result; surface boundary for on-surface; outward arrow and lifted result for normal |
| Pull / Project | Nearest-point slanted connector versus clearly parallel projection rays |
| Interpolate / control-point curve | Points on the curve versus square off-curve handles; do not invent behavior to differentiate related interpolation tools |
| Rebuild / Divide | Reconstruction cue versus regularly spaced division stations |
| Sketch / surface / mesh | Pencil/freehand foreground, curved surface boundary versus large mesh triangles |
| Create UV / Apply UV | Same flat/curved contexts, opposite transfer direction; template extraction versus wrapping back onto surface |
| Single / angle isocurve | Parameter curve versus oblique curve with angle cue; verify specs rather than infer from GV |
| Duplicate edge / border | Single copied edge versus a whole copied boundary |
| Section / cross-section profiles | Cut through object versus section across several profile curves |
| Fillet / chamfer / arc blend | Rounded joint versus straight bevel; arc blend uses two arcs according to local spec |
| Boolean / silhouette / wireframe | Generic overlap operation unless a mode is verified; outer contour versus relevant wire curves |

## Review evidence

Review every selected entry against command behavior, not just labels. Write
a per-command list: key, spec/source, meaning, verdict and reason for redesign.
Existing spec order differs from menu order; never zip them by index. Leave
ambiguity explicit when evidence is missing.

Inspect the complete set and confusable families at actual 24px on supported
backgrounds. Large previews help check geometry but cannot replace menu-size
inspection. Do not change the requested palette for contrast on an unsupported
background: use spacing/geometry or report the palette/background limitation.

For Curve runtime checks, verify the executable/dependency prefix, then use
`tests/run_menu_smoke.ps1 -Macro curve_icons_smoke.FCMacro` with those paths.
For Solid, use `-Macro solid_icons_smoke.FCMacro`.
Check actual output, exit code and screenshot. Resource-only updates need no
Rust rebuild; if the loader changes, test it appropriately.
