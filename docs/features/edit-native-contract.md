# Native Edit contract — 2026-10-09

Applies to OM9-TOP11-005/008/010 and OM9-SOLID-001..004. Rust owns identity,
permissions, ordered selection phases, options and Boolean mode. C++/Qt handles
native selection, transient Coin previews and transactions; typed Part/Mesh
calls perform geometry operations without evaluating command input.

## Verified source

Re-read `OpenMatrix9_Codex_Spec_v1/matrix8_book_1.pdf` on 2026-10-09 through
each continuation to the next heading: Explode PDF84–85, Join85–86, Trim86–87,
Union/Difference223, Intersection223–224, Boolean2Objects224. Printed pages
are ten lower. Ignored local references still exist; `rg --files` alone does
not establish absence. They are excluded from native build/resources.

The manual describes Explode components staying in their group; Join curve
endpoints moving to their midpoint; surface Join sewing naked edges without
changing surfaces; mutual Trim splitting with clicked regions removed;
Extend Lines using imaginary straight extensions; Apparent Intersections using
view projection for curves, not surfaces. Difference selects targets then
cutters, with DeleteInput controlling cutters. Boolean2Objects has Union,
A−B, B−A, Intersection and Inversion Intersection. Defaults, bounds, projection
and backend choices below are OpenMatrix9 decisions.

## Supported inputs and outputs

- Join: at least two Edge/Wire objects forming one endpoint-connected chain,
  or Face/Shell objects forming one sewn shell. Mixed curve/surface batches,
  disconnected inputs, solids and compounds are rejected. Endpoints within
  explicit tolerance move to their pairwise midpoint; ambiguous multiple
  partners reject. Lines stay lines; changed nonlinear curves become finite
  BSplines with adjusted end poles, without a tangency/curvature promise.
  Unchanged curves retain their geometry. Surface sewing preserves underlying
  surfaces and uses host tolerance. Output is one Wire or Shell.
- Explode: Wire edges, Shell/Solid faces, Compound/CompSolid children, or Mesh
  connected components. Each input needs at least two components. Mesh facets
  retain existing connectivity; unwelding is not automatic. Outputs return to
  source groups and preserve world placement. Available ShapeColor/LineColor/
  PointColor, Transparency, LineWidth, PointSize and DisplayMode are copied.
  Rich per-face styles and active-layer mapping remain unverified.
- Trim: at least two Edge/Wire/Face/Shell inputs. General Fuse supplies
  original-source split Edges/Faces for 3D intersections. A pick removes one
  unambiguous region; all unpicked regions remain, including cutter fragments.
  Entirely removing an input is supported. Each retained fragment is a native
  Part::Feature. ExtendLines adds imaginary straight cutters covering model
  bounds; outputs stay within original extents. ApparentIntersections freezes
  the starting camera's eye, right/up/forward and projection type. Rational
  BSpline projection finds intersections in orthographic or perspective views;
  mapped parameters cut original analytic curves and retain depth. Options
  combine. Virtual perspective cutters are clipped to positive camera depth.
  Original curves behind/crossing the eye plane, collapsed/overlapping
  projections, and apparent surface mode reject without document changes.
  Large scales, extreme foreshortening and exhaustive topology are unverified.
- Boolean: same-category closed solid BRep, surface-only Face/Shell BRep, or
  closed consistently oriented positive-volume native Mesh. BRep compounds
  must contain only the chosen category. Mesh/BRep and solid/surface mixed
  batches reject. Union fuses one set; Difference/Intersection fuse each
  ordered set before cutting/intersecting. Boolean2Objects requires exactly
  two inputs and cycles Union, A−B, B−A, Intersection, XOR (Inversion
  Intersection). Results are Part::Feature solids/surface-area shapes or
  Mesh::Feature meshes. Empty/non-solid volume output or surface intersection
  consisting only of edges cannot commit. OCCT accepts some coplanar/contained
  cases cautioned against in the old manual; this is a host kernel choice.

The manual sends mesh booleans to a same-type Boolean Builder. Direct routing
in these commands is an explicit OM9 extension. Native Mesh set operations
failed independent cube closedness/volume fixtures and are not used. The
supported route converts triangular facets to sewn shells, reconstructs
containment parents, subtracts direct inner shells from even-depth outers,
performs OCCT Boolean and triangulates back. Cavities, nested islands and
disjoint components are retained. Ambiguous shell overlap, invalid/nonclosed
output or a mesh/polyhedron volume mismatch is an error. Conversion preserves
geometry, not topology numbering or rich 3DM attributes. Large/ill-conditioned
meshes and exhaustive open-surface orientation semantics remain unverified.

## Special Explode adapters — scope and verification

The new adapters extend Explode beyond the previously validated geometry slice.
Native App::Link and retained 3DM embedded block instances expand only after
explicit CMD Explode; menu/F6 keeps blocks intact. Nested embedded members are
flattened with cycle/depth/missing-member guards; definition members remain.
Native geometric members retain label and supported ShapeColor/LineColor/
PointColor, Transparency, LineWidth and PointSize through component metadata.
Groups themselves and subelement selections are not general Explode inputs.

Native Draft Text/Label uses available Qt glyph contours with source rendered
strings, Center/Right alignment and line spacing; Label also retains displayed
leader/frame graphics and its current text frame. Screen mode rejects.
ShapeString uses its current native outlines with a required available font file. Current Draft
linear/angular dimension display geometry becomes detached curve components
and an editable native text object. Native arrows and Label graphics use Coin
primitive line/triangle outlines; curved arrow graphics are tessellated display
geometry. Angular dimension arcs use the retained analytic native circle,
not the display polyline. This does not promise exact analytic arrow solids.
A native OM9CageComponents link list
provides an explicit current-component adapter. This does not infer arbitrary
cage deformation history from an anonymous BRep.

Retained 3DM records are decoded from their verified import archive/manifest.
Embedded blocks, supported font outlines and current linear/angular/radial/
ordinate/center-mark dimension displays are converted through typed openNURBS
glue. Dimension labels preserve their detached affine frame as native text.
Current NURBS cage/morph-control geometry yields exact boundary surfaces and
control lattice, with source degrees, knots, weights and control points in
metadata. Captive-object deformation history is not reconstructed.

Unavailable/substituted fonts, view-dependent annotation, custom dimension
arrow blocks, linked-only/missing block definitions, cyclic/unsupported members
and invalid archives reject the entire operation before commit. Source archive
payload is retained. Outputs carry supported source attributes and provenance;
rich per-face/per-span styling and exhaustive annotation layouts remain
unverified. Preview displays drawable geometry; detached text is not a complete
annotation preview, and mixed Part/Mesh previews are limited.

Successful special Explode uses the same stale-input checks and one transaction
as ordinary Explode; it consumes the selected instance/annotation/control object,
not its retained definition members. Outputs restore source group ownership and
world placement. Final SDK retained 3DM fixtures pass40/40 and native special fixtures pass25/25.
Geometry, member-style, annotation/angular, font and stale-state fixtures validate
the supported adapter slice; broader limits above remain. Earlier
geometry reports do not certify this adapter branch.

## Options, tolerances and ownership

Coordinates/tolerances use mm. Curve Join Tolerance is finite, in [1e-9, 1e6],
initially 1e-7. An endpoint graph checks that exact bound before OCCT applies
its own precision; no silent enlargement. Sorting uses min(Tolerance, 1e-7).
General Fuse adds zero fuzzy tolerance. Surface sewing uses native tolerance;
automatic document-tolerance compatibility is unverified.

Trim ExtendLines and ApparentIntersections initially No, shown in the nonmodal
panel. Changing intersection options resets removal picks and rebuilds preview;
choose regions again. UI/CMD share Rust state. CMD accepts `ExtendLines=Yes/No`,
`ApparentIntersections=Yes/No` (alias `UseApparentIntersections`), Join
`Tolerance=number`, and Boolean `DeleteInput=Yes/No`. Options reset per session.
Invalid option kinds/nonfinite/out-of-range numbers cannot alter geometry.

CMD region picking requires distance ≤1e-4 and rejects ambiguous regions using
1e-8 distance ties. Mouse picks map rendered fragments back to analytic BRep.
Shared-boundary recognition uses 1e-4 plus the norm of one float ULP per Coin
coordinate and camera-scaled uncertainty of pick radius plus one physical pixel
at the picked depth. Projected fragments compare geometrically coincident
endpoints as well as topology identity. Zoom into visually ambiguous regions.

Join/Explode/Trim replace inputs. Difference always replaces targets;
DeleteInput controls cutters. Boolean DeleteInput initially No; Union,
Intersection and Boolean2Objects apply it to both sets (host extension).
Inputs with dependent models other than containing groups and supported Join
History records cannot be deleted. Join/Surface History records are detached inside the
Edit transaction, with Lock and warning policy. Other dependency errors leave
the entire document unchanged.

## Lifecycle, Undo and persistence

Stable OM9 menu/F6 IDs and English CMD names share one controller. Whole-object
preselection and viewport selection use the active document. Enter advances
selection/accepts preview/completes Trim. Undo removes the last selection or
Trim removal. All commands open nonmodal preview with OK/Cancel. Trim also
accepts `Object@x,y,z` in world coordinates. Boolean2Objects cycles with
viewport click, Next or its dialog button. Document Undo handles commits.

Preview adds no document objects and does not write Visibility. Transient Coin
switches hide inputs and are restored on cancel/error/document switch/close or
workbench deactivation. Esc/Cancel discards the session. Before commit, input
BRep or complete mesh topology and lossless numeric global-placement snapshots
are revalidated. One transaction creates outputs/removes intended inputs;
failure aborts it. Only successful commit records command history.

Metadata includes OM9FeatureId, OM9Command, SourceNames, BooleanMode;
JoinTolerance; TrimExtendLines, TrimApparentIntersections, TrimProjectionFrame
(eye/right/up/forward) and TrimPerspectiveProjection where applicable.
SourceNames are provenance strings, not live History links. Source edits do
not regenerate output. Native Shape/Mesh and metadata persist in FCStd;
Undo/Redo restore the transaction. Associative curve Join uses the separate
[native History contract](history-native-contract.md); ordinary Edit breaks
affected Join/Surface History links rather than recording an associative operation.
Full Matrix History remains unsupported.

## Validation

Rust permission/session fixtures: `rust/tests/edit_commands.rs` and
`rust/tests/edit_session.rs`. Native fixtures: `tests/edit_commands_smoke.FCMacro`
and `tests/edit_options_smoke.FCMacro`. Expectations use independently calculated
lengths, areas, endpoint coordinates and volumes. Actual reports, SDK build,
review and source-audit limits are in the live ledger and per-feature records.
Features remain `partially_implemented`; fixtures certify the supported slice.

Final SDK adapter evidence:

- Retained3DM:
  build/edit_3dm_special_smoke-1/2f68c35673a44bfba6f26c318c833ea7/results.json —40/40.
- Native special:
  build/edit_special_types_smoke-1/b52b0d2757dc464c8b6ccd91a864d5ef/results.json —25/25.
- History:
  build/history_smoke-1/0a28ceafabe247a8ad0389272225c714/results.json —83/83;
  cold build/history_smoke-1/2e27955c3b79451a8ee7c47bcb1a8bff/results.json —4/4.
- Geometry regressions: core
  build/edit_commands_smoke-1/2d83a4f497de4cc48327549ca2f29e87/results.json —115/115;
  options build/edit_options_smoke-1/6b25f835369b4e90be86b7a5e9e96aa2/results.json —57/57.
- Native decoder fixtures: tests/native/three_dm_explode.cpp.

Status: validated_supported_slice; broad feature statuses remain
partially_implemented. See the
[validation record](../validation/2026-10-09-edit-history-special-types.md)
for report scope, review fixes and source-audit limits.
