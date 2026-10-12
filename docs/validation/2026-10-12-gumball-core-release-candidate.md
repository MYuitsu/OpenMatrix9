# Gumball core release candidate — 2026-10-12

Historical pending draft, superseded after final native gates by [the completion evidence](2026-10-12-gumball-core-completion.md). The original pending statements below describe the draft stage only.

OM9-INFO-020, OM9-INFO-021 and OM9-INFO-022 remain
`partially_implemented`. `validated_supported_slice` in the catalog refers to
the [2026-10-11 four-view baseline](2026-10-11-gumball-four-view-completion.md).
The current clean-source candidate is `awaiting_native_gates`: source/fixture
presence and this documentation review do not establish a passing build or
native behavior. Release version and completion counters are unset here.

The original Matrix 8 Book 1 PDF45–47/printed35–37 design and the approved
Rhino5 four-view/color choices remain distinct from OM9 implementation notes.
No private source/manual/artwork is added by this draft.

## Candidate behavior

| ID | Candidate scope | Required native gates |
|---|---|---|
| OM9-INFO-020 | Native primitive dimensions/expressions; representable Link occurrence Scale; admitted standard FreeCAD Link/Scale/Compound affine result retaining definitions, supported dependencies/subelements, nested groups and Body/Tip trees; typed document Camera/Light transforms | Final clean build; primitive/owner/callback/reentrancy suites; associative graph warm and fresh-process cold FCStd; frame-object warm/cold |
| OM9-INFO-021 | Shared World/Object/CPlane; actual Ctrl+Shift Mesh vertex/edge/planar facet identities from all source views including empty selection; typed document Camera/Light physical Object frames | Actual pointer suite; Mesh frame suite; typed property/view-provider/unchanged-source and warm/cold suites |
| OM9-INFO-022 | Shared origin/X/optional-Y Relocate and Ctrl-start widget-only on admitted owners; captured CPlane/document units and `w` world; cancel/invalidation and approved process-local cache | Relocate pointer, four-view/lifecycle/DPI, expanded-owner and typed-frame unchanged-source replay |

Primitive dimensions are admitted by representation: Box/Plane/Ellipsoid/Wedge
by owner axes; Sphere/Torus uniform; Cylinder/Cone/Prism equal radial plus
independent axial factors. A native expression may be multiplied without
flattening its dependencies. Coupled changed dimensions, attachment/bound
Placement, oblique or unrepresentable owner transforms use an admitted native
associative result. Unsupported native graph/property/geometry cases must
reject, never silently tessellate to a Mesh.

Associative output retains the native source definition and uses FreeCAD
`App::Link`/`Part::Scale`, with `Part::Compound` only where occurrence
normalization requires it. Supported incoming links and subelements retain
their semantics; groups and Body/Tip definition trees remain native. Alt Copy
retains the original and its consumers. Recompute, source edits, Undo/Redo,
batch publication guards and fresh-process FCStd must prove these behaviors.
This is native FreeCAD dependency preservation, not complete Matrix History.

Camera/Light scope is explicit: typed document `OpenMatrix9Gui::GumballCamera`
and `GumballLight` store physical Placement and parameters. Camera source
properties are FocalDistance (length), Height (length), HeightAngle (angle),
Perspective (bool) and AspectRatio (scalar). Light has SizeU/SizeV/Reach
(lengths) and Power (scalar). Display dimensions must be finite and positive;
power is finite and nonnegative. Object alignment reads the physical frame;
Gumball parameter Scale and native object FCStd restoration are separate from
the transient widget cache.

## Fixture map — evidence pending

These are test inputs to run on the final compiled plugin, not passing results:

- [Primitive dimensions/expressions](../../tests/gumball_parametric_smoke.FCMacro).
- [Associative native graph warm/cold](../../tests/gumball_associative_scale_smoke.FCMacro): native types/definitions, source edits, subelements, grouped/nested Copy, selected dependent owners, native Body/Tip and fresh-process reopen.
- [Actual Mesh Ctrl+Shift pointer](../../tests/gumball_component_pointer_smoke.FCMacro) and [Mesh frames](../../tests/gumball_mesh_frame_smoke.FCMacro).
- [Typed Camera/Light warm](../../tests/gumball_frame_objects_smoke.FCMacro) and [cold](../../tests/gumball_frame_objects_cold_smoke.FCMacro).
- [Owner Scale](../../tests/gumball_owner_scale_smoke.FCMacro), [owner callbacks](../../tests/gumball_owner_callback_smoke.FCMacro), [reentrant publication](../../tests/gumball_reentrancy_smoke.FCMacro) and [ownership](../../tests/gumball_ownership_smoke.FCMacro).
- [Four-view](../../tests/gumball_four_view_smoke.FCMacro), [view lifecycle](../../tests/gumball_view_lifecycle_smoke.FCMacro), [Relocate pointer](../../tests/gumball_relocate_pointer_smoke.FCMacro), [settings](../../tests/gumball_settings_smoke.FCMacro), [Snappy](../../tests/gumball_snappy_smoke.FCMacro), [visual](../../tests/gumball_visual_smoke.FCMacro) and [core cold](../../tests/gumball_core_cold_smoke.FCMacro).

The [stock native CI suite](../../tools/ci/run-gumball.ps1) selects the actual
plugin runtime and gates reports/module hashes. Root records native reports,
binary hash, clean build and supported-host limitations before changing any
completion status. Earlier native counts/hashes remain dated history.

## Documentation checks

Scoped verification passed for JSON/YAML equality across 607 catalog entries,
the unchanged 604 other features, all three source designs/code guides, three
partial feature statuses, six updated manifest records and 30 candidate links.
The historical ledger entries and release.json were preserved. Public-source
audit reported 2284 source files, 511 icon bindings, 510 SVG files and zero errors;
`git diff --check` passed. These checks validate the documentation draft only.
No native candidate result is recorded by this report.

## General applicability

| Contract | Applies / evidence or N/A reason |
|---|---|
| OM9-CMD | Applies to all three: shared router, numeric prompts, modifier timing/cancel. Dated baseline evidence; final-source command replay pending. |
| OM9-OBJECT | Applies: source identity/topology, native properties and FCStd. 021/022 are read-only. New primitive/graph/frame-object gates pending. |
| OM9-SELECT | Applies: same-document editable source, canonical hierarchy, permission/stale identity and native component picks. Baseline ownership evidence; actual new Mesh pointer gate pending. |
| OM9-VIEW | Applies: shared world frame/four cameras, first-click, DPI/focus/maximize, active/passive view cleanup. Baseline four-view/lifecycle evidence; final-source replay pending. |
| OM9-DISPLAY | Applies to temporary overlays/preview and native frame-object providers. Baseline logical-pixel visual evidence; new symbol/visibility/pickability gates pending. N/A for a new render/shading mode. |
| OM9-PICK | Applies: native component identity, source-view ray and supported Osnap. Baseline End/Mid/Point proof; actual Mesh Ctrl+Shift gate pending. No complete snap-combination claim. |
| OM9-COORD | Applies: units, captured CPlane versus world and owner/group Placement; baseline parser/frame proof, new physical-source frame gates pending. |
| OM9-SURFACE | N/A for rail/profile surface builders. Limited planar Extrude belongs to 020 and remains separately capability-gated with baseline geometry evidence. |
| OM9-EDIT | N/A for join/split/trim/Boolean editing commands. Atomic transform publication/rollback remains required under 020. |
| OM9-POINTS | N/A for control-point or Mesh vertex deformation. Mesh component selection/frame derivation is read-only and does not authorize grip editing. |
| OM9-TRANSFORM | Applies to 020 native transform/result/Copy; final dimension/graph gates pending. N/A for geometry writes by 021/022 view-state frame changes. |
| OM9-ANALYSIS | N/A for an analysis/measurement command: no independent diagnostic object or measurement output is created. Oracle values validate fixture geometry only. |
| OM9-ORGANIZE | Applies to permission/layer/group/Link/Body ownership in 020; new graph warm/cold gates pending. 021/022 only read permission/hierarchy, with no organizational writes. |
| OM9-ANNOTATE | N/A for document text/dimensions. Transient numeric editors use viewport logical pixels and are covered by baseline UI/DPI evidence. |
| OM9-MAKE2D | N/A: no projected document curves, pages or layouts are created; screenshots are visual checks only. |
| OM9-RENDER | Typed document Camera/Light physical data only. N/A for renderer/export/media/backend integration and live host modeling-camera widgets; those require separate specs and native gates. |
| OM9-SOLID | Applies only to validity/volume of supported solid transform/Extrude results in 020; baseline geometry oracle, expanded primitive/graph gates pending. N/A for a new solid builder. |
| OM9-PICTURE | N/A: no reference-picture object, relink or image-to-geometry workflow. |
| OM9-FLOW | N/A: no UV mapping/deformation policy or solver. |

## Approved limits and completion gate

- Bounded256 single-owner pivot cache is deliberately process-local. It is not
  serialized to FCStd; source stamp, explicit Alignment and Reset invalidate it.
  Native Camera/Light objects and dependency graphs do require cold persistence.
- External-document/native graph admission and unsupported/ambiguous/degenerate
  component frames reject explicitly. No implicit World frame or Mesh fallback.
- FreeCAD opening a transaction clears old Redo; callback failure after opening
  it can restore model/Undo but cannot restore old Redo. Pretransaction rejection
  must retain Redo. This host limit remains visible to reviewers.
- Clayoo/SmartTargets, Rhino internal roll, complete original-host representation
  or snap parity, a full Render feature and live host camera-widget parity are
  outside these three core specs. No final native safety or parity claim here.

After final native gates, replace pending statements with exact report evidence
and limitations, update all three status representations/ledgers consistently,
refresh package manifest hashes and set the release timestamp from actual
Asia/Saigon completion time. This draft changes neither release.json nor version
counters.
