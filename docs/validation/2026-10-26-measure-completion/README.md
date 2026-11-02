# Measure, model-space annotation and Transform corrections — 2026-10-27

Standalone Rust OM9, Windows x64, product default OCCT features. The user authorized
L2-03-20 completion, all thirteen annotation commands, and the L2-04-01 corrections.
This is scoped implementation evidence, not a signed M03-UI or Rhino interchange gate.

## Implemented behavior

- Length measures placed curves and canonical BRep edges (all unique edges, or selected
  root edges). A selected NURBS surface opens `U=value`/`V=value`, then Enter measures
  that isocurve. Area uses OCCT canonical BRep area or converged numerical NURBS surface
  integration. Volume requires a closed canonical solid. No display-mesh bounding-box estimate.
- Radius keeps a circle/arc's constant radius. Ellipse/spline requests a point or explicit
  `Station=0..1` and labels the local osculating radius. At repeated NURBS knots, compare exact
  one-sided jets; report only when geometric tangent/curvature agree. Lines/inflections and
  inconsistent knot limits refuse a finite answer.
- Distance/Angle retain point mode; `Objects` measures two selected objects (minimum OCCT
  distance; angle of two straight lines). `Continuous` chains distances or repeats angles;
  `Ortho` constrains the next point to the dominant captured CPlane axis. Hover feedback is
  read-only, with the same points as commit and no journal/event spam per frame.
- Analyze menu includes Area, Volume and Object Summary. Summary groups root objects by
  layer and placed definition; totals exclude unplaced definitions. Gem weights sum explicit
  nonnegative finite `om9.gem.carat` values per placed occurrence and report missing weights;
  no density, volume-to-carat or unknown-field inference.
- All 13 annotation ids have an owner under L2-03-21, their original authored icon bindings,
  a Dimension menu entry, and one dispatcher. See
  `implementation/design/M03/measure-completion-20261026/ANNOTATION-CONTRACT.md` for inputs,
  aliases, OM9 defaults and scope. The typed annotation extension, canonical strokes/point,
  style, edits, Unicode, identity and native `.om9` roundtrip are tested. Transaction insertion
  now reserves extension storage. Scene wire buffers draw disconnected annotation strokes
  explicitly without inventing connectors.
- ArrayCrv adds positive `Spacing=10mm` or Count, Basepoint, NoRotation/Freeform/Roadlike,
  and Preview. Arc-length station parameters drive position and tangent directly, including
  self-intersection stations. Closed paths omit the duplicated end. Limit 512 total items.
- SetPt edits canonical point/line/NURBS/surface/mesh/cloud coordinates with SetX/Y/Z,
  World/CPlane and explicit zero-based Indices; placement stays unchanged. Unsupported solid,
  analytic-circle and annotation control editing refuses. Preview and commit share a pure
  plan; accepted work is one Undo. Surface/mesh previews use their geometry, BRep transforms
  use a bounding outline. Preview can be toggled off.

## Blockers and boundaries

- L3-019 Solid Points ON remains disabled: curve/surface grip routines are not a solid
  control-point selection/edit owner.
- L3-025 (feature OM9-TRANSFORM-026) remains disabled. The earlier claim that only forward
  surface evaluation exists was incorrect: closest-point and pullback routines exist and a
  projection fixture passes. A verified command adapter for curve/surface selection,
  on-surface tolerance, station placement and normal orientation is still missing. A fitted
  pullback at sample points does not certify the whole fitted curve lies on the surface.
- L3-037 remains disabled under its unresolved facing contract. The 22 M08-deferred rows
  retain their ownership and disabled state.
- Annotation witnesses are model-space snapshots; source-geometry association, paper-space,
  native Rhino dimension/style/layout roundtrip and full drafting parity remain open.
  Text is drawn facing the screen at a projected model height; no rotated text-layout claim.
  Dimension Options without selection changes session defaults, not a saved document style.
  `Text` remains solid Text; annotation uses TextBlock/DimText. `DimVertical` disambiguates the
  source catalog's repeated `Dim` command. Invalid input preserves a retryable run.
- Object Angle supports straight lines; general tangent/surface angles need explicit points.
  Root BRep-edge measurement does not yet follow selected instance member paths. Mesh object
  distance requires triangles. Arbitrary retained/native opaque objects are refused.
- Surface area is numerical, absolute convergence threshold 1e-7 mm²; this is not a formal
  global error bound for every pathological NURBS patch. Unsupported derivatives/periodicity
  refuse. Continuous hovering is verified semantically and separately from GPU presentation.
- SDK v1 gains no geometry export in this change. Existing public mesh-area SDK evidence is
  separate. Native calls here are host-internal synchronous OCCT adapters, not openNURBS tests.

## Verification

`combined-tests.log`: 65/65 tests passed in eight app suites before the final minor refusal/color
adjustments. `final-focused.log`, `core-regressions.log`, `scene-regressions.log`, `build.log`
and `check-results.json` record the final batch, using CARGO_TARGET_DIR=L:/om9-target and -j 2.
Do not count retries as additional cases. See `receipt.json` for final counts/fingerprints and
`runtime-launch.json` for the separately staged executable and actual UI scope.

An initial annotation test run exposed six failures: missing extension reservation and unsupported
disconnected strokes; the later passing tests exercise both fixes. The original standalone
measurement tests were extended to require an explicit spline curvature station. The first
ellipse knot test exposed the derivative continuity refusal and led to the exact one-sided
curvature comparison. No claim of a pre-existing red baseline for all added features.

No full future openNURBS matrix is enumerated; remaining total batch count is unknown.
Packages 2–8 remain open. This batch does not close Rhino5 annotation/styles/layout exchange.

## Final result

**169 distinct test functions passed**: core 88, scene 12, app 69 (the three annotation UI-frame
tests are included in 69). Retries/followups overlap these functions and are not added to the total.
The host Measure extension has **11 named fixture groups across 7 tests**, including 4 OCCT
fixture groups and zero openNURBS fixtures; see measurement-fixtures.json. The final numeric
followup checks all 12 selected edge UUIDs and nonuniformly placed box length/area/volume.
The 18-test final-guards followup covers later finite/trim refusals; it is a subset, not extra tests.

Final executable: `build/standalone-validation/ui-artifacts/measure-transform-20261027/om9-verified.exe`.
Its hash matches the built target and runtime-verified-launch.json records the live process.
Windows was unlocked by the user, but the helper still could not inject input and returned a
black capture/window chrome only. The application is launched; native GPU/menu replay is **not verified**.
The earlier runtime-launch.json preserves the first launch; the verified launch is the final build.

Trimmed NURBS surfaces carrying trim metadata refuse area/isocurve/object-distance analysis until
represented by canonical trimmed BRep; the underlying rectangular patch is never silently measured.
Polycurve curvature at joins requires selecting an individual smooth segment. Annotation/aggregate
measurements refuse nonfinite values rather than publishing an infinite label.

Prepared openNURBS batches are unchanged, packages 2–8 remain open, and the full remaining future
batch count is unknown. Next action is independent Rhino5 annotation/style/layout roundtrip in
its exchange owner and actual desktop review when the input/capture helper is available.
