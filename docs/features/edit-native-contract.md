# Native Edit contract — 2026-10-06

Applies to OM9-TOP11-005/008/010 and OM9-SOLID-001..004. Rust owns command
identity, permissions, ordered selection phases, options and Boolean mode.
C++/Qt owns native selection, transient Coin previews and transactions; typed
calls into FreeCAD Part perform geometry operations without evaluating input.

## Verified source

Read Matrix 8 Book 1 from the local Spec v1 package. PDF pp.84–85 cover Explode;
pp.85–86 cover Join; pp.86–87 cover Trim; p.223 covers Union and Difference;
pp.223–224 cover Intersection; p.224 covers Boolean2Objects through Cap Planar.
The corresponding printed pages are PDF page minus ten.

The source requires component separation for Explode, endpoint/naked-edge
continuity for Join, and mutually split inputs with clicked regions removed
for Trim. Difference selects targets then cutters; its DeleteInput controls
cutters. Boolean2Objects includes Union, Intersection, both ordered differences
and Inversion Intersection. These are source facts; the decisions below are
the OpenMatrix9 implementation contract.

## Supported inputs and outputs

- Join: at least two native Edge/Wire inputs forming one endpoint-connected
  chain, or Face/Shell inputs sewn into one shell. Mixed curve/surface batches,
  disconnected curves/surfaces, solids and compounds are rejected. Native
  sewing preserves the underlying surfaces. Output is one Wire or Shell.
- Explode: one or more native Wires with multiple edges, or Shells/Solids with
  multiple faces. Output consists of individual original Edges/Faces.
- Trim: at least two native Edge/Wire/Face/Shell inputs. The native General Fuse
  image map supplies split Edges/Faces for each source. A world-space pick
  removes one unambiguous remaining region. Each retained fragment becomes a
  native Part::Feature; unpicked regions are retained. Entirely removing an
  input is supported. Extend Lines and apparent/view-projected intersections
  are not implemented. Kernel failure and absent 3D intersections are errors.
- Boolean: valid closed native solids, including compounds consisting entirely
  of solids. Union fuses one set. Difference and Intersection fuse each ordered
  set before cutting/intersecting. Boolean2Objects requires exactly two inputs
  and cycles Union, A−B, B−A, Intersection, XOR. XOR is interpreted as Inversion
  Intersection, retaining both unshared parts. Each result solid becomes one
  Part::Feature; empty/non-solid results cannot commit. Open-surface and mesh
  Boolean semantics are unsupported. OCCT accepts some coplanar/contained
  inputs that the old Matrix manual cautions against; this is a kernel choice.

Blocks/links, groups as input objects, text, dimensions, cages, mesh objects and
subelement references are unsupported. Inputs within App::Part are transformed
to world coordinates by applying their parent frame once. Outputs are document
root snapshots. Group membership, source display styling and active layers are
not propagated. Blocks are preserved by rejecting them, including CMD Explode.

## Options, tolerances and ownership

World coordinates and tolerances use mm. Curve sorting uses 1e-7 mm; native
surface sewing uses its host tolerance. General Fuse uses zero additional
fuzzy tolerance. CMD region picking accepts distance ≤1e-4 mm, rejects ambiguous
region boundaries, and uses a 1e-8 mm distance tie threshold. Mouse picks map
each rendered fragment back to its BRep; tessellation does not set the analytic
curve acceptance distance. Shared-boundary mouse recognition uses 1e-4 mm plus
the norm of one float ULP per Coin coordinate and camera-scaled pixel uncertainty
(the active pick radius plus one physical pixel at the picked depth). Visually
ambiguous small regions require zooming in. Extreme foreshortening is not a
verified slice. These tolerances
are host decisions, not recovered Matrix defaults. Near-endpoint midpoint
snapping and configurable document tolerances remain unsupported.

Join/Explode/Trim replace selected inputs. Difference always replaces targets;
DeleteInput controls whether cutters are removed. Boolean DeleteInput initially
is No, a conservative host choice; Union/Intersection/Boolean2Objects apply it
to both input sets. This retention option for Union/Intersection is a host
extension. Deletion is rejected if an input has a dependent model other than
its containing App::Part/group. The command leaves all objects unchanged when
this check fails. No dependent models are silently detached.

## Lifecycle, Undo and persistence

Original English names and stable OM9 menu IDs invoke the same controller.
Preselection and viewport selection use whole objects from the active document.
Enter advances selection; Undo removes the last selection or trimmed region.
Join/Explode/Booleans open a nonmodal preview with OK/Cancel. Boolean2Objects
cycles through results with viewport clicks, Next or the dialog button. Trim
uses pickable transient geometry and commits with Enter; CMD Object@x,y,z offers
the same world-coordinate region operation. Selection-time Undo cannot change
already committed geometry; document Undo handles committed operations.

Preview adds no document object and does not write Visibility. Coin display
switches hide selected inputs temporarily and are restored on cancellation,
error, document switch/close or workbench deactivation. Esc/Cancel discards the
entire session. Before commit, the input BRep and global-placement snapshots
are revalidated. Matrix entries use lossless hexadecimal floating-point values
so small parent-placement changes cannot pass through rounded display text.
One native transaction creates outputs and removes intended
inputs; failure aborts it. Only successful commit records command history.

Output metadata includes OM9FeatureId, OM9Command, SourceNames and BooleanMode.
SourceNames are provenance strings, not live History links. Source edits do
not regenerate output. Native Shape and metadata persist in FCStd; Undo/Redo
restore the whole transaction. Full Matrix History compatibility is unverified.

## Validation

`rust/tests/edit_commands.rs` checks required document write permission and
captions. `rust/tests/edit_session.rs` checks ordered sets, duplicate rejection,
minimum selection, selection Undo and five Boolean modes. Native geometry and
lifecycle fixtures are in `tests/edit_commands_smoke.FCMacro`; their actual
report, build and regression results are recorded in the implementation ledger.
Fixtures use independently calculated lengths, areas and volumes, not outputs
from the same construction algorithm as expectations.

Read the per-feature records and live ledger for verified scope. Full specialist
options, advanced topology and Matrix/Rhino compatibility remain incomplete.
