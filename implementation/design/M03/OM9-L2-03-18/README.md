# Solid menu validation

All 21 bindings continue to dispatch through the existing Rust Controller and OCCT owners.

Ellipsoid correction (2026-10-23): safe Rust constructs a rational biquadratic 9×5
net; the native adapter builds one closed B-Rep solid. Geometry thresholds, serialized
source reload, menu/CMD, Undo/Redo and `.om9` persistence pass. The native Windows
Ellipsoid workflow was observed in four GPU views, including invalid input and cancel.

[Measured report](../../../../docs/validation/2026-10-23-exact-ellipsoid.md) ·
[Receipt](../../../evidence/M03/ellipsoid-exact-2026-10-23/receipt.json) ·
[UI review awaiting human signoff](../../../evidence/M03/ellipsoid-exact-2026-10-23/ui-review.json)

The other twenty Solid UI workflows are not newly certified by this Ellipsoid run.
M03-UI remains pending until a human reviewer records acceptance.

## Solid follow-up — 2026-10-23

Exact curve extrusion and one-transaction ExtrudeAll, Text/Boolean menu prompts, and span-aware volume integration are verified. Kernel 71 passed; app 191 passed; native Text/Boolean/ExtrudeAll saved files cold-measured. The earlier Ellipsoid-only checkpoint above is historical.

[Latest report](../../../../docs/validation/2026-10-23-solid-ui-followup.md) · [Human worksheet](../../../../implementation/evidence/M03/solid-ui-followup-2026-10-23/M03-UI-CHECKLIST.md). M03-UI remains unsigned.

## 2026-10-28 Solid audit

The 21 L3 bindings do not cover all recovered Solid menu entries or variants. The in-progress inventory and partial Boolean Difference and Union BRep selection/DeleteInput implementations are recorded in [the Solid audit](../../../../docs/validation/2026-10-28-solid-audit.md). No manual UI gate is promoted.

Boolean Intersection now also has a partial standalone BRep implementation with
two staged sets, group fusion, `DeleteInput` and empty-result rejection. The
same audit records automated evidence and outstanding acceptance boundaries.

Boolean Two Objects now accepts viewport BRep selection and pending Undo, but
its five-state preview/cycling and DeleteInput remain open in standalone.
The B−A and symmetric-difference calculations are now implemented for BReps,
with operation-option and geometry checks; the preview lifecycle remains open.
The later Boolean Two Objects checkpoint adds partial BRep preview, cycling,
Enter confirmation and DeleteInput. Input hiding and UI acceptance remain open.
The following checkpoint adds transient GPU source masking; actual desktop and
human UI acceptance remain open.

The palette now exposes Boolean Split and Create Solid as disabled entries;
their real solvers, specific authored icons and parameter contracts remain open.
See the Solid audit for the bounded recovered registration evidence.

One-target/one-cutter closed-BRep partition and piece extraction are now
available in the kernel. Boolean Split remains disabled until its multi-set,
multi-output, selection, options and lifecycle are integrated and tested.
The kernel now also handles multiple closed-solid inputs with deduplicated
cells; this does not itself activate the command.

The later BRep-only Boolean Split checkpoint activates the original command
key for menu and CMD with staged sets, separate outputs and DeleteInput.
See the audit for its icon, tests and remaining geometry/UI limitations.

The next checkpoint enables a partial Create Solid BRep region workflow via
menu/CMD. See the audit for remaining category, SDK and UI gates.

Cap Planar now stages multiple BRep shell selections and commits atomically;
its existing one-boundary native adapter and actual UI acceptance remain
limited. See `docs/validation/2026-10-28-solid-audit.md`.

The later Cap checkpoint adds tested independent loops and a circular edge;
nested coplanar loops and the actual UI gate remain open.

The subsequent checkpoint adds a tested nested annular cap; general nesting
and the actual UI gate remain open.

The latest checkpoint also caps the planar boundary of a tested mixed shell
while retaining its nonplanar opening, and tests a face with two holes.

Extract Surface now separates a numeric face selection by default and has a
typed Copy=Yes path; viewport face selection, right-click and options remain
open. See `docs/validation/2026-10-28-solid-audit.md`.

The later Extract checkpoint adds Copy and OutputLayer choices and a right-click
Copy binding. An egui preselection case fails, and actual UI acceptance and
viewport face selection remain open.

The following checkpoint corrects that fixture's history-icon offset and tests
right-click with preselection, plus one viewport face with Enter/Undo/Cancel.
Multiple faces and actual desktop UI acceptance remain open.

The subsequent checkpoint supports tested multiple faces and sources in one
transaction, including Copy=Yes and all-face removal. Instance/ambiguous picks
and actual desktop UI acceptance remain open.

The later Extract checkpoint offers a chooser for overlapping root BReps and
revalidates its candidates. Instance paths, multiple hits within one root and
actual desktop UI acceptance remain open.

Extract now supports Ctrl + primary click to remove a pending face, including
through the overlapping-root chooser. Fillet Edge's numeric-only implementation
and its documented staged edge/handle workflow remain to be reconciled.

The next Fillet checkpoint adds staged root-edge selection, explicit constant
radius, preview and commit. Handle editing, open shells and other documented
options remain open.

The following Fillet checkpoint captures NextRadius per picked independent
edge. Handle editing, tangent-contour variable radii and open shells remain open.

A later Fillet checkpoint supports the tested shared edge of a joined open
shell. General shell topology and actual desktop acceptance remain open.

The following checkpoint adds one sampled handle per picked edge and
CurrentRadius preview editing. Dragging, additional handles and continuously
varying radius remain open.

The subsequent checkpoint tests linear radii between oriented endpoint handles
of an independent edge. Handle drag and multi-edge tangent contours remain open.
# Fillet endpoint classification (2026-10-28)

The closed-edge case is now distinguished from lookup errors before selection.
An automated cylinder fixture checks its single sampled handle, commit and
Undo; moving the handle and general closed curves remain open. See
`docs/validation/2026-10-28-solid-audit.md`.

Fillet ShowRadius now toggles preview radius labels without removing handles.
Keyboard, pointer and typed option routes and preview state were tested; the
original default and actual desktop rendering remain unverified.

Fillet `SetAll=` updates the implemented handles in preview; two independent
edges, invalid-radius rollback, commit and Undo passed automated checks. Full
handle coverage and actual desktop input remain open.

Fillet LinkHandles and SetAll keyboard, token and typed routes passed simulated
UI tests on two independent edges, including rollback and Undo; actual desktop
input and additional handle types remain open.

Fillet SelectEdges returns to selection retaining prior radius edits. Simulated
option routes and viewport picking with two independent edges, commit and Undo
passed; actual desktop and general topology remain unverified.

Fillet variable radius now has a tested two-segment open tangent contour in
the OCCT bridge. Shared vertex radii must agree; app editing of both coincident
handles together and general curved/closed contours remain open.

A later checkpoint synchronizes shared tangent-vertex endpoint radii when a
handle is edited; rollback and a non-tangent corner were tested. General
curved/closed contours, large-model performance and desktop UI remain open.

Shared Fillet endpoint classification now batches candidate edges per OCCT
query; regression tests pass. Large-BRep latency and native memory are not yet
measured.

Fillet shared tangent handles now resolve as one viewport hit when OCCT confirms
the same vertex and contour; unrelated close hits remain ambiguous. An egui
regression passed; actual desktop picking remains open.

The Fillet OCCT adapter now accepts interior radius controls and was checked
on straight and split straight edges. AddHandle interaction and actual desktop
projection are not yet connected.

The subsequent AddHandle checkpoint connects preview and egui viewport picks
on a selected open edge; translated straight-edge geometry and added-handle
editing were tested. General curved/closed edges and actual desktop remain open.

Fillet RemoveHandle now removes added controls while protecting default
handles; simulated option routes, preview volume and Undo passed. Actual desktop
and native rebuild-failure rollback remain unverified.

Fillet CopyHandle now copies a chosen radius to repeated new or existing
locations in preview; simulated option routes, BRep result and Undo passed.
Actual desktop and general curves remain unverified.
## Shell multi-face kernel checkpoint (2026-10-28)

OCCT shell accepts multiple removed faces on one closed solid, tested on a box
with two opposite openings. App multi-face selection and desktop UI remain open;
see `docs/validation/2026-10-28-solid-shell-source-map.md`.

Shell now exposes staged root-face picks and Thickness in the application;
simulated input, geometry and Undo/Cancel checks passed. Instance faces and
actual desktop acceptance remain open.

2026-10-28 Pipe placement: sampled rail points now include entity placement; translated and rotated controller regressions passed for menu/CMD, with exact BRep bounds, volume and Undo. Pipe options and interactive/desktop acceptance remain open. See `docs/validation/2026-10-28-solid-pipe-source-map.md`.

2026-10-28 Pipe follow-up: single rail can be selected in the active command in egui pointer simulation; selection Undo and Cancel do not commit. Menu/CMD and BRep bounds/volume were checked. Overlap disambiguation, remaining Pipe options and desktop acceptance remain open.

2026-10-28 Pipe overlap follow-up: simulated picking opens the existing disambiguation popup; chosen rail geometry and stale session rejection checked. Desktop acceptance remains open.

2026-10-28 Pipe Global follow-up: explicit Global mode stages two numeric radii and creates a flat-capped straight-rail frustum; controller bounds/volume/Undo and kernel regressions passed. General rails, intermediate stations, other options and desktop acceptance remain open.

2026-10-28 Pipe Cap follow-up: None/Flat produce open/closed straight-rail BReps in constant and Global modes; menu/CMD controller tests and Undo passed. Round and actual desktop remain open.

2026-10-28 Pipe Diameter follow-up: controller supports numeric Radius/Diameter, including mixed input for straight Global endpoints; measured geometry and Undo passed. Desktop and token interaction remain open.

2026-10-28 Pipe option follow-up: simulated keyboard, token and typed routes verified for Diameter, Cap and Global; reselecting Global retains its entered radius. Actual desktop and remaining options remain open.

2026-10-28 Pipe Thick follow-up: four endpoint radii generate a hollow flat-capped straight Global pipe; measured geometry, invalid retry, Undo and simulated option routes passed. Other rails/caps and desktop remain open.

2026-10-28 Pipe Thick open follow-up: Cap=None creates two open walls for straight constant/Global cases; exact topology, area and Undo tested. Orientation, other rails and desktop remain open.

2026-10-28 Pipe Thick orientation follow-up: OCCT reverses the inner wall before sewing; opposing display winding, sub-tolerance rejection and existing rejoin regression passed. Exact normal sampling and desktop remain open.

2026-10-28 Pipe Round follow-up: straight constant-radius capsule with two hemispherical ends measured in menu/CMD controller tests; option cycle simulated. Varying radius, Thick, curved rails and desktop remain open.

2026-10-28 Pipe Round Global follow-up: straight conical pipes with endpoint hemispheres, including Thick, passed native analytical volume/bounds and menu/CMD Undo tests. General rails, continuity and desktop remain open.

2026-10-28 Pipe interaction follow-up: interactive default stages start/end radii and Enter; different radii require explicit Global. Direct Pipe 1 remains one-shot compatibility. Intermediate stations and desktop remain open.

2026-10-28 Pipe point follow-up: Radius mode accepts endpoint points on straight rails via egui viewport and CMD; BRep volume/Undo tested. Section-plane projection is an OM9 interpretation, native equivalence and desktop remain open.

2026-10-28 Pipe station follow-up: straight non-Thick Global accepts repeated intermediate position/radius; ruled-section kernel and simulated CMD/viewport geometry and Undo tested. C0 vs source blending and remaining variants remain open.

2026-10-28 Pipe Multiple follow-up: staged multi-curve selection and shared numeric parameters create outputs atomically; measured variants, late failure and Undo tested. Shared input semantics vs original, multi-point input and desktop remain open.

2026-10-28 Pipe ChainEdges follow-up: right-click and `_Pipe _C` start partial ordered connected collinear root-Line chaining; volume, rejection and Undo tested in simulation. BRep edges, curved chains and desktop remain open.

2026-10-28 Pipe BRep ChainEdges follow-up: exact straight topology selection, connected collinear chain, selection Undo/Cancel and output volume validated in simulation. Curved edges, mixed chains and desktop remain open.

2026-10-28 Pipe mixed chain follow-up: root Lines and BRep line edges share one selection order and Undo; both orders and BRep output tested in simulation. Curved/bent chains and desktop remain open.

2026-10-28 Pipe chain-point follow-up: supported straight mixed chains accept endpoint Radius points and Global intermediate stations; simulated geometry and Undo passed. Native point semantics and desktop remain open.

2026-10-28 Pipe bent-chain follow-up: OCCT circular mitered sweep supports constant-radius connected straight segments and Flat/None caps; 90-degree BRep volume/Undo validated in simulation. Original transition semantics and broader cases remain open.

2026-10-28 Pipe bent Thick follow-up: constant inner/outer radii and Flat/None caps supported on straight-segment bends; 90-degree hollow volume, wall area/winding and Undo tested in simulation. Original miter and other variants remain open.

2026-10-28 Pipe bent Round follow-up: constant-radius miter chains, including Thick, support endpoint spherical caps with a minimum endpoint segment length; 90-degree volume and Undo validated in simulation. Complex rails, original semantics and desktop remain open.

2026-10-28 Pipe bent endpoint-point follow-up: points use local end-segment tangents; equal-required radii respect document tolerance. CMD/viewport BRep volume/Undo checked. Original semantics and desktop remain open.

2026-10-28 Pipe bent Round geometry: six controller cases validate measured volume and Undo at 45 and 120 degrees and on a spatial chain, for thin and Thick. Original transition equivalence, complex rails and desktop remain open.

2026-10-28 Thick Pipe input-order correction: equal-required checks match inner/outer radii by magnitude rather than input order; measured straight/bent chain regressions and Undo passed. Native semantics and desktop remain open.

2026-10-28 Pipe full-circle correction: constant-radius Cap=Flat uses an exact oriented torus for circularity-preserving placement; menu/CMD measured area, volume, bounds and Undo passed. Other variants and desktop remain open.

2026-10-28 Pipe full-circle Thick: constant inner/outer radii with Cap=Flat create an exact hollow torus; menu/CMD measured area, volume, closure and Undo passed. Original and desktop remain unverified.

2026-10-28 Pipe full-circle Cap=None: constant-radius thin/Thick produce exact surface walls without naked seam edges; CMD area, display winding and Undo checked. Original, other frames and desktop remain open.

2026-10-28 Pipe full-circle Cap=Round: constant-radius closed rails use the Flat torus result, including Thick; controller and simulated viewport volume/Undo passed. Original closed-rail cap semantics and desktop remain unverified.

2026-10-28 Pipe full-circle placement: six None/Flat/Round thin/Thick cases with scale, reflection and translation pass measured geometry and Undo. The user approved the Round closed-circle skill rule; Matrix original and desktop remain unverified.

2026-10-28 Boss checkpoint: bounded manual and menu registration comparison recorded in `docs/validation/2026-10-28-solid-boss-source-map.md`. Boss lacks a handler and profile-to-boundary trim/join solver; targeted catalog/availability test confirms Boss, Rib and Slab unavailable. Sidebar click and implementation remain open.

2026-10-28 Boss kernel foundation: selected OCCT Cut bridge can trim surface BRep by a closed solid tool, including tested planar and cylindrical faces; 31 kernel editing tests passed. Boss wall trimming, join, command and desktop remain open.

2026-10-28 Boss parallel polygon kernel: signed-direction wall construction and boundary join pass geometry/topology fixtures. Repaired missing closing wall in uncapped polygon extrusion. Selected BRep, general geometry, command and desktop remain open.

2026-10-28 Boss planar BRep kernel: exact plane query accepts one existing planar BRep face and preserves its existing hole; area/topology and invalid cases tested. App selection, other geometry and desktop remain open.

2026-10-28 Boss command slice: menu/CMD staged single polygon and planar BRep selection, viewport simulation, atomic boundary replacement and Undo tested. General geometry, multiple profiles and desktop remain open.

2026-10-28 Boss multi-profile: disjoint polygons on one planar BRep, selection Undo and one transaction tested in controller and simulated viewport. Overlaps, general geometry and desktop remain open.

2026-10-28 Boss circle: exact full circles can mix with polygons on a planar boundary; analytical area and Undo pass in kernel/controller/simulated viewport. General curves, curved boundaries and desktop remain open.

2026-10-28 Boss planar NURBS: rational ellipse face/topology, translated controller and simulated selection/Undo tested. OCCT ellipse wall-area measurement differs about 0.42% from independent perimeter integration; measurement, general curves and desktop remain open.

2026-10-28 extrusion area follow-up: temporary C2 interval splitting fixes the primary 0.42% discrepancy; three ellipse checks pass 3e-5 mm², with residual error on the more eccentric case. General surface accuracy and desktop remain open.

2026-10-28 Boss mixed line/arc polycurve: connected and reversed scene selection, exact semicircle fixture area and Undo pass in kernel/controller/viewport simulation. Other polycurves, curved boundary and desktop remain open.

2026-10-28 Boss polycurve display admission: incremental scene updates now reserve the actual arc sample count using the same calculation as the display cache; a point-budget boundary regression and full scene suite pass. This does not close remaining Boss geometry or desktop gates.

2026-10-28 Boss NURBS/line: a quadratic Bézier arch plus line is selectable and commits in both directions, with placement and Undo tested. Area differs ~0.000367 mm² from an independent oracle; general NURBS display, curved boundary and desktop remain open.

2026-10-28 NURBS wall measurement follow-up: temporary OCCT extrusion-face subdivision reduces the Bézier area discrepancy to below 1e-7 mm², also passing three ellipse fixtures. General surface measurements, native peak memory and desktop remain open.

2026-10-28 Boss NURBS polycurve display: segment reversal and each nonempty knot span are respected, including short-span peak and bounded point reservation; four direction combinations pass controller/viewport fixtures. General display accuracy and Boss desktop remain open.

2026-10-28 Boss coplanar polysurface: connected two-face boundary with profile on one face or crossing the seam passes geometry tests; controller placement/Undo and simulated viewport pass. Bent/curved boundary and desktop remain open.

2026-10-28 Boss inclined planar face: polygon extrusion is clipped by a finite planar-side tool and joined; both signed directions, projected area, translated controller/Undo and simulated viewport tested. Curve profiles with inclined face, curved boundary and desktop remain open.

2026-10-28 Boss inclined circle and NURBS ellipse: both signed directions, independent area, translated controller/Undo and simulated viewport tested. Conservative curve bounds can reject near-plane profiles; general curves, curved boundary and desktop remain open.

2026-10-28 Boss near-plane NURBS: intersect projected BRep bounds with the positive-weight control hull; a previously refused rotated ellipse now passes area/controller/viewport/Undo, while a crossing ellipse is rejected. Bounds remain conservative; general cases and desktop remain open.

2026-10-28 Boss piecewise NURBS near-plane: union of segment hulls for NURBS/line polycurves reduces false refusal; four-segment ellipse and mixed curve area, crossing rejection, both directions and viewport/Undo tested. General curves and curved boundary remain open.

2026-10-28 Boss cylindrical face: one interior polygon with normal perpendicular to the cylinder axis is extruded in its positive normal, trimmed to the derived cylinder carrier and sewn to the selected complete face. Analytic area, both directions, placed controller/Undo and simulated viewport tested; original Matrix direction, general curved boundaries and desktop remain open.

2026-10-28 Boss cylindrical multi-profile: multiple disjoint polygons sew atomically; same/opposite-side geometry, overlap and nonmanifold refusal, controller/viewport and Undo tested. General curved boundaries and desktop remain open.

2026-10-28 Boss cylindrical circle: exact circle and mixed circle/polygon pass independent area and Undo tests. Rational ellipse currently fails continuity splitting during area measurement and is refused atomically; general curves and desktop remain open.

2026-10-28 Boss cylindrical ellipse follow-up: exactly split suitable clamped C0 NURBS into Bezier knot intervals; ellipse now passes independent area, direction, controller/viewport and Undo tests. Output has multiple wall faces; general knots and desktop remain open.

2026-10-28 Boss cylindrical smooth NURBS: exact knot insertion extends the Bezier decomposition to clamped rational curves with simple interior knots; uniform/nonuniform fixture, area, direction, budget and Undo checked. Output has split wall faces; periodic/trimmed domains and desktop remain open.

2026-10-28 Boss cylindrical smooth NURBS/line polycurve: exact refinement and flattening preserve four direction combinations, independent area and viewport/Undo. General nested curves, periodic/trimmed NURBS and desktop remain open.

2026-10-28 Boss oblique cylinder: inclined polygon in both directions matches independent areas; finite-end miss and partial join are rejected. Menu/CMD controller and simulated viewport Undo pass. Other oblique curves, near-axis behavior, original Matrix direction and desktop remain open.

2026-10-28 Boss oblique circle and ellipse on cylinder: independent area matches in both directions; finite-end crossing is rejected, translated menu/CMD and simulated circle viewport Undo pass. General curves, near-axis behavior, original Matrix direction and desktop remain open.

2026-10-28 Truncated Cone: staged center, base radius, signed height and top radius with captured CPlane and Diameter; `_TCone` alias, retry and Undo. See `docs/validation/2026-10-28-solid-truncated-cone.md` for supported input paths and unimplemented options.

2026-10-28 Truncated Cone 2Point: endpoint selection, right-click icon, shared option routes, signed preview and simulated cross-viewport mouse height. Other modes and desktop remain open.

2026-10-28 Truncated Cone 3Point: captured-frame circumcircle, collinear retry, staged Undo, signed geometry and simulated viewport checked; the 3Point Radius suboption and other modes remain open.

2026-10-28 Truncated Cone 3Point Radius: numeric/point radius followed by side selection, both branches, invalid retry, Undo, Front frame, shared options and simulated viewport checked. Interpretation and other modes remain open.

2026-10-28 Truncated Cone Area: numeric base area in squared document units, Radius/Diameter switching, invalid retry, independent volume and shared option routes checked. Other modes and desktop remain open.

2026-10-28 Truncated Cone Solid: Yes closed, No open side face, equal radii cylinder, analytic area/volume and shared option routes checked. Original Matrix open/default semantics, extreme scales and desktop remain open.

2026-10-28 Truncated Cone FitPoints: direct and selected placed points/mesh/curve/surface controls, shared algebraic fit, preview, budget, Undo and simulated viewport checked; individual subobject controls, general fitting accuracy and desktop remain open.

2026-10-28 Truncated Cone AroundCurve: line/circle tangent-normal frame, point or OnCurve selection, placed source and simulated viewport/Undo checked; general curve accuracy and desktop remain open.

2026-10-28 Truncated Cone Tangent + Point: single-curve contact plus point, both sides, placed circle, invalid retry and simulated viewport checked; multi-curve tangencies and desktop remain open.
# Truncated Cone input integrity follow-up (2026-10-29)

Source-lifetime follow-up: a deleted captured curve invalidates preview and commit for AroundCurve and Tangent; see the Truncated Cone validation record for the limited deletion fixture.

Selected base modes cannot be bypassed with the direct three-number form; Tangent requires its base constraint before accepting a dimension. See `docs/validation/2026-10-28-solid-truncated-cone.md` for tested scope and remaining gaps.

# Pipe Diameter point input (2026-10-29)

Diameter changes numerical input; a section point specifies the wall position. Straight, Thick and intermediate station fixtures are in `docs/validation/2026-10-28-solid-pipe-source-map.md`; scope is partial.

# Pipe curved endpoint point input (2026-10-29)

A quarter arc and a quadratic NURBS now have endpoint-point and approximate-volume evidence; the general Pipe path samples the curve.

Endpoint point sizing uses the curve tangent in world coordinates; full-circle fixtures and remaining limitations are in `docs/validation/2026-10-28-solid-pipe-source-map.md`.

# Pipe Global variable radius on open curves (2026-10-29)

The existing interpolated-section loft handles a sampled non-line open curve; arc fixtures and limitations are in `docs/validation/2026-10-28-solid-pipe-source-map.md`.

# Thick Global Pipe on open curves (2026-10-29)

One quadratic NURBS fixture checks varying thin and Thick modes with Flat/None and Undo; see the later curve-stations entry for intermediate-section support.

Inner and outer sampled-curve lofts support capped and open output; arc fixtures and limitations are in `docs/validation/2026-10-28-solid-pipe-source-map.md`.

# Pipe stations on open curves (2026-10-29)

A nonuniform-speed quadratic NURBS fixture distinguishes length-based interpolation from parameter-linear interpolation; general accuracy remains open.

Two out-of-order arc stations and Cap=None have controller fixtures; closed curves and non-similarity placement are rejected atomically.

Non-Thick Global Pipe inserts intermediate sections into a sampled open curve and interpolates by sampled length. See `docs/validation/2026-10-28-solid-pipe-source-map.md` for evidence and limits.

# Pipe Multiple point input (2026-10-29)

Curved Round construction carries prior output usage into temporary checks; the output-count limit has an atomic refusal fixture, while native peak memory remains unmeasured.

Point radii use the first selected rail as the shared reference; see `docs/validation/2026-10-28-solid-pipe-source-map.md` for verified fixtures and limits.

# Pipe Multiple Global straight stations (2026-10-29)

Station fractions from the first straight rail apply across selected straight rails in one transaction; see `docs/validation/2026-10-28-solid-pipe-source-map.md` for scope and limits.

# Pipe Multiple Global mixed stations (2026-10-29)

Non-Thick Multiple Global stations also support Round caps on mixed open rails; see Pipe source map for evidence and limits.

Two stations on a nonuniform-speed NURBS/line pair and Cap=None on an arc/line pair have controller evidence; original Matrix mapping remains unknown.

Mixed open curve/line batches share a normalized parameter station and commit atomically; evidence and limitations are in `docs/validation/2026-10-28-solid-pipe-source-map.md`.

# Pipe Local varying radius (2026-10-29)

Global Round varying thin/Thick on one sampled open curve and Local Round with two NURBS stations have additional fixtures in the Pipe source map.

Round-cap support extends to non-Thick Local/Global intermediate stations on a single open rail; line and arc fixtures are recorded in the Pipe source map.

Local Round additionally caps sampled open curves using each endpoint section direction; quarter-arc thin/Thick fixtures are recorded in the Pipe source map.

Local Round on a straight rail uses a dedicated OCCT hemisphere adapter and fuses caps to the sampled body, including Thick; see Pipe source map for evidence and native ownership.

Two out-of-order Local stations on one quadratic NURBS have independent volume and Undo evidence; Round compatibility remains open.

Local non-Thick Pipe additionally supports intermediate stations on one open curve, with piecewise sampled smoothstep.

Local Thick also supports differing end wall sizes on one open curve using two lofts; scope and evidence are in the Pipe source map.

One open rail uses a sampled smoothstep interpretation; verified fixtures and limitations are in `docs/validation/2026-10-28-solid-pipe-source-map.md`.

# Pipe endpoint section preview (2026-10-29)

Broad Solid regression and dispatcher stack-safety evidence is in `docs/validation/2026-10-29-solid-wide-regression.md`.

Cone staged base/apex construction has partial evidence in `docs/validation/2026-10-29-solid-cone-basic.md`.

Pyramid Star and its Rust-profile/OCCT adapter have partial evidence in `docs/validation/2026-10-29-solid-pyramid-star.md`.

Pyramid Edge and Flip have partial evidence in `docs/validation/2026-10-29-solid-pyramid-edge.md`.

Pyramid Circumscribed base has partial evidence in `docs/validation/2026-10-29-solid-pyramid-circumscribed.md`.

Pyramid staged Inscribed construction and wire preview have partial evidence in `docs/validation/2026-10-29-solid-pyramid-inscribed.md`.

Tube base Area has partial evidence in `docs/validation/2026-10-29-solid-tube-area.md`.

Tube WallThickness has partial evidence in `docs/validation/2026-10-29-solid-tube-wall-thickness.md`.

Tube BothSides has partial evidence in `docs/validation/2026-10-29-solid-tube-both-sides.md`.

Tube Tangent + Point has partial evidence in `docs/validation/2026-10-29-solid-tube-tangent-point.md`.

Tube FitPoints and AroundCurve have partial evidence in `docs/validation/2026-10-29-solid-tube-fit-around.md`.

Tube 3Point Radius, wire preview and right-click binding have partial evidence in `docs/validation/2026-10-29-solid-tube-radius-preview.md`.

Tube Center, 2Point and 3Point use staged CPlane input; evidence and remaining gaps are in `docs/validation/2026-10-29-solid-tube-basic.md`.

Cylinder exposes the existing return to Center in its option interface; see `docs/validation/2026-10-29-solid-cylinder-center-return.md`.

Cylinder height now has a signed wire-body preview; controller and simulated viewport evidence is in `docs/validation/2026-10-29-solid-cylinder-wire-preview.md`.

Cylinder Tangent + Point supports one planar curve and a point constraint; see `docs/validation/2026-10-29-solid-cylinder-tangent-point.md`.

Cylinder FitPoints uses the bounded shared circle fit; see `docs/validation/2026-10-29-solid-cylinder-fit-points.md`.

Cylinder AroundCurve uses a tangent-aligned frame and source freshness checks; see `docs/validation/2026-10-29-solid-cylinder-around-curve.md`.

Cylinder 3Point and the Radius/side branch use the shared base-circle solver; see `docs/validation/2026-10-29-solid-cylinder-three-point.md`.

Cylinder Center and 2Point support staged input and Solid=Yes/No; see `docs/validation/2026-10-29-solid-cylinder-center.md`.

Torus Vertical constructs a ring perpendicular to the captured CPlane; see `docs/validation/2026-10-29-solid-torus-vertical.md`.

Torus offers bounded wire preview during tube-radius selection; see `docs/validation/2026-10-29-solid-torus-wire-preview.md`.

Torus AroundCurve uses a tangent-aligned frame and source freshness checks; see `docs/validation/2026-10-29-solid-torus-around-curve.md`.

Torus 3Point supports the Radius/side branch after two points; see `docs/validation/2026-10-29-solid-torus-three-point-radius.md`.

Torus 3Point constructs its ring from three input points; see `docs/validation/2026-10-29-solid-torus-three-point.md`.

Torus Center and 2Point support point and numeric input plus the recovered right-click binding; see `docs/validation/2026-10-29-solid-torus-center.md`.

Ellipsoid final-axis preview uses three section ellipses and shares the committed geometry calculation; see `docs/validation/2026-10-29-solid-ellipsoid-wire-preview.md`.

Ellipsoid AroundCurve captures a tangent-aligned frame and validates its source; see `docs/validation/2026-10-29-solid-ellipsoid-around-curve.md`.

Ellipsoid Corner supports two enclosing-box corners and the third axis; see `docs/validation/2026-10-29-solid-ellipsoid-corner.md`.

Ellipsoid FromFoci and MarkFoci support planar input and atomic point/solid output; see `docs/validation/2026-10-29-solid-ellipsoid-foci.md`.

Ellipsoid Diameter supports two axis endpoints and the recovered icon secondary-click binding; see `docs/validation/2026-10-29-solid-ellipsoid-diameter.md`.

Ellipsoid from Center supports point-based oriented axes in the captured CPlane. The tested scope and outstanding options are recorded in `docs/validation/2026-10-29-solid-ellipsoid-center.md`.

Sphere 3Point and 4Point now retain valid picks on invalid input, and 4Point offers a wire preview; standalone controller evidence and limits are in `docs/validation/2026-10-29-solid-sphere-retry-preview.md`.

Straight Pipe body wire preview now shows Thick inner/outer profiles and Round meridians after sufficient radius input.

Thin straight Pipe body wire preview additionally shows Round cap meridians; it is not a BRep preview.

Thin straight Flat/None Pipe has an approximate Local/Global body wire preview through confirmation; other modes remain scoped separately.

Straight BRep edge endpoints are cached during selection and revalidated before use; repeated preview and source deletion have simulated fixtures, not a performance measurement.

Chain preview uses endpoint segment directions or the collinear station axis; BRep-edge preview extraction performance remains unmeasured.

Section rings now include Multiple's first reference rail and an active intermediate station; Chain and full-body preview remain open.

Single-curve point input shows a bounded section ring; details and unverified display paths are in `docs/validation/2026-10-28-solid-pipe-source-map.md`.
