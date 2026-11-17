# Curve/Solid commands affected by source/display separation

Date: 2026-11-13. Read-only command inventory and static caller review; no geometry tests or behavior changes in this batch. Menu membership comes from `docs/analysis/matrix-command-layers.json`, the previously recovered Matrix menu bindings. Membership is not evidence that a command is enabled or fully implemented in standalone OM9. Command families below are a review queue, not a new implementation commitment or a completion percentage.

The separation concerns canonical Curve/NURBS/B-rep geometry, derived viewport data and the binding between a displayed pick and its source. Calculation may still be approximate when it starts from source. For user-authored/imported mesh, the mesh itself can be canonical geometry; do not replace it with an invented NURBS surface.

## Curve menu

| Menu group / commands | Relationship | Current-source evidence / review boundary |
|---|---|---|
| Curve from Objects: Duplicate Edge, Duplicate Border, Duplicate Face Border | Bind picked source topology; output source curves rather than display chords. | `wireframe.rs`, `dup_border.rs` cover the reviewed DupEdge/DupBorder routes. Duplicate Face Border full parity is not established here. |
| Extract Wireframe | Displayed wire set determines which curves; geometry comes from source faces/edges, with density and trim consistency. | Existing scoped surface-wire work and `wireframe.rs`; B-rep paths still extract topology edges only. |
| Extract Isocurve, Extract Angle Isocurve | Source surface parameters and trim; viewport picking must retain source binding. | `isocurve.rs`, `angle_isocurve.rs`; no general angle/trim parity claim. |
| Project, Pullback | Surface projection/closest-point computation must have its own tolerance and source placement. | `project.rs` currently samples 64 curve points and projects to the construction plane, not arbitrary target faces. `pull.rs` passes a 64-sample request to source curve/surface operations. These paths are narrower than full surface/polysurface projection semantics and do not become exact by avoiding viewport buffers. |
| Intersection, Intersect Two Sets | Source curve/surface intersections; display segments may locate candidates but should not define the answer. | `intersect.rs` reviewed route accepts two curves, not arbitrary B-reps; `intersect_two_sets.rs` is a separate owner. Broader type support requires review. |
| Contour, Section, Cross-Section Profiles | Cut source geometry by planes; calculation tolerance independent of display density. | `section.rs` samples an untrimmed surface on a 48-by-48 parameter grid and assembles crossings; `cross_section.rs` exists. Other menu variants remain catalog candidates here. |
| Silhouette | Viewing direction legitimately affects the result; source normals/trim should govern accuracy. | `silhouette.rs` uses a 48-by-48 source sample grid and CPlane normal in the reviewed route. Actual-camera semantics and general accuracy are not established. |
| Create UV Curves, Apply UV Curves | UV domains, mapping and trim belong to source geometry. | `uv_curves.rs` reviewed Create UV route emits a rectangle from the underlying domain. General trim loops/Apply UV behavior not established. |
| Short Path | Path on source surface, with a bounded approximation/tolerance contract. | Menu candidate; full implementation not reviewed in this batch. |
| Free-Form: Interp Curve on Surface, Polyline on Surface, Sketch on Surface; Extend Curve: Curve on Surface | Picks/projections must refer to source surface, trim and placement. | `interp_on_surface.rs`, `polyline_on_surface.rs`, `sketch_on_surface.rs`, `extend_curve.rs`. Two reviewed point-based routes use closest-point surface operations; general placement/trim is not certified. A polyline is intentionally piecewise linear. |
| Offset Curve on Surface, Offset Normal to Surface | Offsets and normals must be evaluated on source, with approximation controlled by operation tolerance. | `offset_on_surface.rs` samples the source curve and evaluates surface normals. General guarantees and `offset_normal.rs` require their own audit. |
| Line: Normal to Surface, tangent/perpendicular variants; Circle/Arc: tangent variants and Around Curve; Helix Around Curve | Source tangent/normal/curve derivatives govern construction when existing geometry is used. | Menu-semantic candidates. Shared source-edge and point-picking infrastructure is relevant; individual modes not certified by this inventory. |
| Closest Point, Drape Points; Fit Points, Extract Points, Extract Curve | Must distinguish true source vertices/control points from tessellation vertices. | Menu-semantic candidates; source meaning depends on each command. Fit Sphere source-vertex fixes do not establish these Curve modes. |
| Duplicate Mesh Edge, Mesh Outline, Polyline On Mesh, Sketch On Mesh | Mesh is the selected source; camera may affect outlines, not the underlying mesh identity. | Mesh-specific behavior must be retained; standalone coverage not established by menu presence. |

Pure curve operations (Offset Curve, Curve Boolean, Fillet/Chamfer/Blend Curves, Extend, Refit/Rebuild, Convert Curve to Lines/Arcs, Tween, Curve from 2 Views, Soft Edit and seam editing) are also affected by the common source-curve and picking contracts, but do not inherently need a B-rep mesh split. Conversion/fit output can intentionally be approximate. Ordinary Line/Circle/Arc/Rectangle/Polygon and point commands inherit the source-verified snap path when points are picked on existing geometry; that does not justify separately rewriting every primitive.

## Solid menu

| Menu group / commands | Relationship | Current-source evidence / review boundary |
|---|---|---|
| Boolean Union, Difference, Intersection, Boolean Two Objects, Boolean Split, Create Solid | Operate on source topology/geometry, then derive result display. | `solid.rs`, `solid_boolean.rs`, `solid_text_stages.rs`; current scoped checked paths are not general parity. |
| Fillet Edge, Blend Edge, Chamfer Edge | Resolve selected source edge and adjacent faces; display IDs alone are insufficient. | Existing Fillet owner/source-binding work is relevant. Blend/Chamfer menu membership does not certify standalone support. |
| Shell, Extract Surface/Copy, Cap Planar | Source faces, naked boundary loops, orientation and trim govern output. | `solid.rs` and kernel editing callers; existing scoped tests do not prove all surface types. |
| Pipe, Pipe ChainEdges, Solid Multi Pipe | Rail curves/selected B-rep edges must retain source domains, orientation and placement. | `solid_pipe.rs` and existing PipeChain source-edge checks; general modes not reviewed. |
| Extrude Planar Curve: Straight, Along Curve, To Point, Tapered; Solid All Extrude | Use source profile/rail, closure and placement; preview can be cheaper than committed geometry. | `solid.rs` and `solid_geometry.rs` include reviewed straight/all paths. Other variants require separate checks. |
| Extrude Surface and Extrude Surface to Boundary: Straight, Along Curve, To Point, Tapered | Must use actual face/trim/boundary rather than triangles or underlying untrimmed support alone. | `extrude_surface.rs` exists; no full mode/type coverage established. |
| Solid Edit Tools / Edges: Move, Rotate, Scale, Move Untrimmed Edge | Edit exact topology and update selection identity, scene and history together. | Menu-semantic candidates; enabled state and implementation not audited here. |
| Solid Edit Tools / Faces: Move, Move to Boundary, Shear, Fold, Merge, Merge All, Split, Move Untrimmed Face | Edit source faces and trim; derived display and stale picks must be invalidated. | Menu-semantic candidates; do not infer runtime support. |
| Solid Edit Tools / Holes: Make, Place, Revolved, Round, Copy, Move, Rotate, Delete, Array, Polar Array; Wire Cut | Source cut geometry and face selection, followed by topology/scene/history updates. | Menu-semantic candidates; no new implementation/accuracy claim. |
| Boss, Rib, Slab | Use source profiles/faces and intersections/offsets. | Registered/runtime paths need mode-specific review; source dependence follows menu semantics. |
| Box, Sphere, Ellipsoid, Cylinder, Cone, Truncated Cone, Pyramid, Tube, Torus, Paraboloid, Text | Produce canonical geometry then derive display. Source-linked modes (Around Curve, Tangent, Fit Sphere to Points) require source inputs. | `solid.rs`, primitive and text owners. Not every primitive/mode is established by the Sphere/Box fixtures. |

## Priority for the existing plan

1. Retain completed source-binding work on DupEdge, DupBorder, Isocurve, Fillet, Shell, Extract Surface, PipeChain and Boolean with their exact evidence.
2. Treat B-rep wireframe interior curves/density as a documented feature gap.
3. Before enlarging the feature set, audit source/placement/trim/tolerance on Project, Pullback, Section, Silhouette and surface-curve operations. Fixed source sampling is a separate issue from viewport dependence.
4. Keep mesh-source commands and deliberate curve-to-polyline conversions explicit. Shared snap validation benefits many drawing commands.
5. Face/edge/hole editing and unreviewed menu variants remain bounded candidates; they do not automatically enter this implementation batch. Source-only history/eviction stays the separate deferred Plan B.
