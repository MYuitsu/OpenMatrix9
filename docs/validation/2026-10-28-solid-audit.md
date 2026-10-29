# Solid audit — 2026-10-28 (in progress)

2026-10-29: Cylinder Tangent + Point supports a planar tangent source and one point constraint; see `2026-10-29-solid-cylinder-tangent-point.md`.

2026-10-29: Cylinder FitPoints supports direct and preselected points using the shared bounded fit; see `2026-10-29-solid-cylinder-fit-points.md`.

2026-10-29: Cylinder AroundCurve supports selected-curve center and source freshness; see `2026-10-29-solid-cylinder-around-curve.md`.

2026-10-29: Cylinder 3Point and its two-point Radius branch use the shared base solver; see `2026-10-29-solid-cylinder-three-point.md`.

2026-10-29: Cylinder Center and 2Point support staged input, Solid and the recovered secondary-click binding; see `2026-10-29-solid-cylinder-center.md`.

2026-10-29: Torus Vertical constructs a ring plane perpendicular to the captured CPlane; see `2026-10-29-solid-torus-vertical.md`.

2026-10-29: Torus has shared-calculation wire preview during tube-radius input; see `2026-10-29-solid-torus-wire-preview.md`.

2026-10-29: Torus AroundCurve accepts a curve and tangent-aligned frame with source freshness checks; see `2026-10-29-solid-torus-around-curve.md`.

2026-10-29: Torus 3Point Radius accepts a radius and side after two picks; see `2026-10-29-solid-torus-three-point-radius.md`.

2026-10-29: Torus 3Point constructs a base circle and allows invalid-point retry; see `2026-10-29-solid-torus-three-point.md`.

2026-10-29: Torus Center and 2Point now support point input and the recovered right-click binding; see `2026-10-29-solid-torus-center.md`.

2026-10-29: Ellipsoid now has final-axis wire preview across five modes, sharing the committed geometry calculation; see `2026-10-29-solid-ellipsoid-wire-preview.md`.

2026-10-29: Ellipsoid AroundCurve supports a chosen curve and tangent-aligned frame with stale-source checks; see `2026-10-29-solid-ellipsoid-around-curve.md`.

2026-10-29: Ellipsoid Corner constructs an ellipsoid from opposite enclosing-box corners and a third axis; see `2026-10-29-solid-ellipsoid-corner.md`.

2026-10-29: Ellipsoid FromFoci and atomic MarkFoci are supported for the captured planar frame; see `2026-10-29-solid-ellipsoid-foci.md` for tests and limits.

2026-10-29: Ellipsoid Diameter supports the recovered right-click and CMD alias, and two diameter endpoints; see `2026-10-29-solid-ellipsoid-diameter.md`.

2026-10-29: Ellipsoid from center supports interactive center and three oriented axes; see `2026-10-29-solid-ellipsoid-center.md` for geometry tests and unimplemented options.

2026-10-29: Sphere 3Point/4Point invalid inputs can be retried; 4Point has a shared-calculation wire preview. See `2026-10-29-solid-sphere-retry-preview.md` for controller evidence and limitations.

2026-10-29: straight Pipe body wire preview now includes Thick inner/outer walls and Round meridians; see Pipe source map for scope.

2026-10-29: thin straight Pipe body wire preview now depicts Round cap meridians; see Pipe source map for tested scope.

2026-10-29: thin straight Pipe Flat/None now has a five-section Local/Global body wire preview through confirmation; see the Pipe source map for limits.

2026-10-29: Pipe Chain caches selected straight BRep edge endpoints, checks reference freshness before reuse and suppresses stale previews; see Pipe source map for tests and unmeasured performance.

2026-10-29: Pipe Chain section preview covers endpoint segment directions and collinear stations; BRep-edge evaluation cost remains unmeasured. See Pipe source map.

2026-10-29: Pipe section preview now covers first-reference Multiple input and active stations; see the Pipe source map for scope.

2026-10-29: Pipe now previews the point-based endpoint section for a single curve with a bounded 64-edge ring; see Pipe source map for geometric fixtures and limitations.

2026-10-29: Multiple curved Round Pipe accounts for prior retained outputs in temporary budget checks; output-limit atomicity verified, native peak and precise retained-memory threshold remain open.

2026-10-29: Global Round Pipe on one sampled open curve now handles varying thin/Thick radii; quarter-arc and viewport fixtures pass. Local Round additionally has a two-station NURBS volume fixture; see Pipe source map.

2026-10-29: Multiple Global Pipe with stations and Round now supports mixed open rails; both selection orders, viewport and closed-member atomic rejection are recorded in the Pipe source map.

2026-10-29: non-Thick Local/Global Pipe with Round caps accepts one or more station inputs on a single open curve; line/arc and viewport fixtures cover one station. See Pipe source map.

2026-10-29: Local Round varying Pipe uses sampled endpoint directions on open curves; quarter-arc thin/Thick and simulated viewport fixtures pass. See Pipe source map for approximation and remaining limits.

2026-10-29: Local Round Pipe on a straight rail now fuses separately constructed hemispheres to the sampled body, including Thick; kernel, controller and simulated viewport fixtures pass. See Pipe source map for native ownership and remaining gaps.

2026-10-29: Local Pipe multiple stations checked on a nonuniform-speed quadratic NURBS against a piecewise smoothstep integral, with selection and document Undo. Round cap compatibility remains open; see Pipe source map.

2026-10-29: Local non-Thick Pipe accepts an intermediate station on one open rail with sampled piecewise smoothstep; line, arc, Flat/None, viewport and Undo fixtures pass. See Pipe source map.

2026-10-29: Local Thick varying radii now use two sampled smoothstep lofts; line/arc Flat/None, invalid-wall retry and viewport simulation are recorded in the Pipe source map.

2026-10-29: Local Pipe now supports differing end radii on one open rail with a sampled smoothstep interpretation; straight, arc, option-route and viewport fixtures are in the Pipe source map.

2026-10-29: Multiple Global station parameter mapping checked with two NURBS/line stations and independent volumes; mixed arc/line Cap=None topology also passes. Mapping remains an OM9 interpretation.

2026-10-29: Multiple Global stations now handle mixed open curve/line batches using a shared normalized parameter; see Pipe source map for approximation and limits.

2026-10-29: non-Thick Multiple Global Pipe supports shared relative stations on straight rails with atomic multi-output commit; unsupported curved members reject the batch. See Pipe source map.

2026-10-29: Multiple Pipe point-based radii use the first selected rail as the shared reference. Two line rails and a circle-first mixed pair, viewport simulation and Undo are recorded in the Pipe source map; original behavior remains unverified.

2026-10-29: two stations on one nonuniform-speed quadratic NURBS check sampled-length interpolation against an independent analytic volume, distinguishable from parameter-linear interpolation; see Pipe source map.

2026-10-29: curved Pipe Global stations additionally checked for out-of-order selection, Undo, open cap and unsupported source rejection; see Pipe source map.

2026-10-29: non-Thick Global Pipe now accepts intermediate stations on a sampled open curve; arc, quadratic NURBS and simulated viewport fixtures pass. See Pipe source map for limits.

2026-10-29: varying Global thin and Thick Pipe on one quadratic NURBS passed capped-volume and uncapped-topology fixtures. Curved intermediate stations remain unimplemented; see Pipe source map.

2026-10-29: Thick Global Pipe now accepts sampled open curves; quarter-arc closed volume and open-wall fixtures, viewport and Undo pass. See the Pipe source map for restrictions.

2026-10-29: Pipe Global unequal endpoint radii now loft along an open sampled curve; arc and viewport fixtures pass, while closed varying curves are rejected. See Pipe source map for limitations.

2026-10-29: endpoint point sizing is additionally checked on a quarter arc and one quadratic NURBS, including distinct end tangents and approximate volume; general NURBS and original behavior remain open.

2026-10-29: Pipe endpoint point sizing evaluates the transformed curve tangent; full circle geometry and viewport simulation verified. Arc/NURBS and original behavior remain open; see Pipe source map.

2026-10-29: Pipe Diameter no longer rejects point-based section sizing for supported rails. Straight, Thick and intermediate-station geometry, viewport simulation and Undo are recorded in the Pipe source map; general curves, Matrix and desktop remain open.

2026-10-29: captured curve deletion suppresses AroundCurve/Tangent preview and blocks commit; source modification and desktop remain unverified. See the Truncated Cone record.

## Boss smooth NURBS/line polycurve on cylindrical face

A closed polycurve consisting of a smooth rational NURBS arch with a simple interior knot and a line previously failed OCCT area integration. Rust now refines eligible NURBS segments, flattens their exact temporary Bezier intervals into the containing polycurve, and preserves segment/whole-curve reversal. An independent numerical area oracle, topology and both-side probes pass all four direction combinations; translated controller, simulated viewport and Undo pass. Original source stays unchanged; output gains wall faces at knots. General nested polycurves, periodic/trimmed NURBS, native peak memory, Matrix original topology and desktop remain open. Skill synchronized automatically.

## Boss smooth rational NURBS on cylindrical face

The prior Bezier-only decomposition now refines clamped, full-domain, nonperiodic NURBS at existing knots before exact segmentation. Preflight budgets the intermediate knot-insertion buffers and segment count; the input is unchanged and the output has separate wall faces at knots. A rational quadratic closed curve with simple internal knots, both uniform and nonuniform, passes 129 sampled point comparisons before/after insertion and independent numerical surface-area integrals within 2e-5 mm². Both reversed directions cut the expected cylinder side; an output limit is rejected without source mutation. Translated controller and simulated viewport/Undo are tested. Trimmed domains, periodic/discontinuous knots, general accuracy and native peak memory remain open; original Matrix topology and desktop remain unverified. Skill synchronized automatically.

## Boss NURBS ellipse on a cylindrical face

The previously failing rational ellipse succeeds when its clamped C0 NURBS Bezier chain is split exactly at existing knots before extrusion. Safe Rust validates knot multiplicities, allocates within a budget, preserves weights/poles and source, and retains the original for unsupported knot forms. The resulting BRep has four wall faces plus the cylindrical face, instead of one wall face: this changes topology while preserving the tested analytic geometry. The original ellipse and a nonuniform-knot equivalent match independent area integrals within 1e-5 mm²; reversed direction, output limit, controller, simulated viewport and Undo are checked. General NURBS knot structures, native peak memory, original Matrix topology/behavior and desktop remain open. Skill synchronized automatically.

## Boss circle on a cylindrical face

The cylindrical Boss path now accepts a full circle alone or mixed with a disjoint polygon. Kernel results match independent numerical integrals of the curved patch and wall within 1e-5 mm²; controller checks mixed atomic commit/Undo and simulated viewport checks circle selection. A rational NURBS ellipse exposes an OCCT measurement failure: continuity splitting of its cylinder-clipped extrusion face fails, and trying direct area subdivision also failed. The production measurement fallback was not relaxed; this ellipse is refused without document mutation or an Undo entry. Bounds for curved profiles are conservative. General curve profiles, full direction semantics, native peak memory and actual desktop remain open. Skill synchronized automatically.

## Boss multiple polygons on a cylindrical face

The cylindrical Boss path accepts multiple disjoint polygon profiles and builds the entire result before one transaction. Each profile uses its positive normal. Kernel fixtures check two disjoint profiles on the same or opposite sides, independent area and topology; overlapping patches and coincident opposite profiles that would make a nonmanifold join are rejected. Controller checks failure without document mutation or Undo entry; simulated viewport checks two selections and one Undo. The rule is an OM9 interpretation; original Matrix direction/overlap semantics, curved profiles, general cylindrical faces, cumulative native peak memory and desktop remain open. Skill synchronized automatically.

## Boss polygon on a cylindrical face

OM9 now supports one polygon inside one complete cylindrical face, with polygon normal perpendicular to the cylinder axis. The chosen direction is the positive polygon normal; reversing polygon order changes direction. This is an OM9 interpretation, not evidence of Matrix's original choice. The OCCT bridge derives a bounded solid carrier from the cylindrical face; Rust validates the profile and uses the carrier to clip extrusion walls before sewing, then publishes one atomic geometry replacement. A radius-5, height-10 cylinder and 2×2 profile match `100*pi + 6*sqrt(24) + 30*asin(0.2)` mm², one shell/five faces, both polygon directions, placed controller, simulated viewport and Undo. Outside, axially out-of-range, axial-normal and already trimmed face fixtures are rejected. General cylindrical trims, multiple profiles, curved profiles, ambiguous direction, spline-converted cylinder, native peak memory/performance, original behavior and desktop remain open. Skill synchronized automatically.

## Boss piecewise NURBS near an inclined boundary

A rotated ellipse represented by four rational NURBS segments was refused by the axis-aligned bounding box even though the curve lies wholly on one side of the inclined boundary. Boss now intersects the projected BRep box with the union of line endpoint and positive-weight NURBS control hulls; unsupported segment families keep the BRep-box fallback. The four-segment ellipse matches an independent area oracle, both polycurve directions work in controller, viewport simulation and Undo, and a truly crossing boundary is rejected. Bounds remain conservative; mixed analytic-arc hulls, general nested curves, curved boundaries, native memory and desktop remain open. Skill synchronized automatically.

## Boss NURBS near an inclined boundary

A rotated ellipse wholly on one side of the boundary was rejected because the world-axis bounding box straddled the plane. Positive-weight NURBS now intersect the projected control-hull interval with the existing BRep-box interval, including rounding padding; both are conservative bounds. The rotated ellipse fixture passes an independent area oracle, translated controller and simulated viewport selection/Undo. A genuinely crossing ellipse is rejected without mutation. A loose control hull or box may still refuse valid curves, and this does not establish arbitrary polycurves, curved boundaries, desktop or original Matrix behavior. Skill synchronized automatically.

## Boss curved profiles on inclined planar boundary

Boss now supports a full circle and a closed NURBS ellipse on an inclined planar boundary. Circle intersection extrema are analytic; other exact curve profiles use a conservative BRep bounding box, which may refuse valid profiles close to the boundary plane. An ellipse exposed invalid temporary four-part subdivision during area measurement; the bridge retries with two parts and still requires a valid BRep. Kernel fixtures on both sides match independent circle/ellipse area references within 1e-5 mm²; a crossing circle is rejected. Translated controller and simulated viewport fixtures cover selection and Undo. General NURBS, other polycurves, near-plane false refusals, arbitrary surfaces, native peak memory/performance, original semantics and desktop remain open. Skill synchronized automatically.

## Boss polygon on an inclined planar boundary

Boss can extrude a polygon normal to its plane and terminate it against an inclined planar BRep face. Rust checks the profile lies wholly on one side, calculates the projected area and extension, trims the boundary, cuts the overlong wall by the plane, and joins one transaction. A temporary bounded OCCT prism performs the plane cut: an attempted infinite half-space cut returned both sides of the open shell, so it was not used. Fixtures in both directions check area `96*sqrt(1.04)+43.2` mm², five faces, shell topology, translated controller, viewport selection and Undo. Crossing or partially covered profiles are rejected. Curved profiles on inclined boundaries, curved boundaries, general topology, memory, original Matrix semantics and actual desktop remain open. Skill synchronized automatically.

## Boss coplanar polysurface boundary

Boss accepts a connected multi-face coplanar shell parallel to the profile. Two joined 5×10 mm faces with a 2×2 mm polygon on one face or crossing their seam produce one joined shell of area 136 mm²; a bent shell and shell with an extra disconnected face are rejected. A translated controller fixture and simulated viewport selection verify transaction and Undo. This extends the documented surface/polysurface direction, but does not establish curved or noncoplanar boundaries, general topology, original Matrix solver behavior or desktop acceptance. OCCT's synchronous bridge checks native face planes and shell membership; Rust owns geometry validation, operation and publication. Skill synchronized automatically.

## Boss NURBS segment direction and knot spans

Scene connectivity and display now honor the reversal flag on each non-periodic NURBS segment, consistent with the exact extrusion adapter. NURBS polycurve display samples each nonempty knot span separately (256 steps per span), preserving internal knot points even when a span is much shorter than 1/256 of the whole domain. A degree-one fixture with knots at 0.001 and 0.002 retains its 10 mm peak, in both segment directions, and checks incremental reservation against the actual 770 points. Controller and simulated viewport Boss fixtures cover all four combinations of segment and polycurve direction, area and Undo. This fixed sampling still has no general geometric error guarantee; nested or periodic polycurves, curved boundary, original behavior and actual desktop remain open. Skill synchronized automatically.

## Boss NURBS wall-area integration follow-up

The quadratic Bézier profile face measures 5.333333333333 mm², matching 16/3; its extrusion wall previously measured 49.579223951962 mm² versus the independent closed-form 49.578857150892 mm². The discrepancy was isolated to wall integration. The OCCT measurement adapter now subdivides temporary extrusion faces for integration, after continuity splitting where needed, without changing the serialized source BRep. The wall now measures 49.578857150897 mm² and the joined result 144.245523817564 mm² versus 144.245523817559 mm². Tests use 1e-7 mm² tolerance; three ellipses also pass the tightened tolerance. Windows `-j 2`, `3dm-brep`: 42 editing, 16 creation and two extrusion tests passed. Other surface families, shared-face compounds, native peak memory, performance and desktop remain unverified. Skill synchronized automatically.

## Boss NURBS/line polycurve

Scene publication now samples connected non-periodic NURBS segments without a seam alongside lines/arcs; it rejects fully repeated internal knots rather than drawing a false connector across a discontinuity. A closed quadratic Bézier arch and straight segment can be selected and joined to a parallel planar boundary in either polycurve direction, with translated placements, menu/CMD dispatch, viewport simulation and Undo. The result has three faces and area 144.245890619 mm² versus independent analytical/numerical 144.245523729 mm² (difference ~0.000367 mm²); the test tolerance is 0.0005 mm², not proof of precise area. Sampling uses 256 steps per NURBS segment; general knot distributions and display error are not established. Nested/periodic polycurves, curved boundaries, original behavior and desktop remain open. Skill synchronized automatically.

## Boss polycurve incremental display budget

Incremental scene addition previously reserved `segments + 1` display points for connected polycurves even when an arc needed many samples. It now shares the checked point-count calculation with display cache construction. A mixed line/arc fixture compares incremental reservation to actual cached points and checks rejection when the point budget is one short. Windows `-j 2`: full scene suite, 44 Solid workflows and 38 simulated viewport tests passed; one native UI fixture ignored. This is a resource-accounting fix, not a new Boss geometry or desktop acceptance claim. Original Matrix behavior and remaining Boss geometry limitations are unchanged.

## Boss mixed arc/line polycurve

Boss now accepts a closed connected line/arc polycurve and preserves its analytic segments under supported placement. A semicircle/line fixture measures 120+8π mm² and three faces, tests both orientations through menu/CMD and simulated viewport, and undoes the result. Reversed connected polycurves previously were not published by the scene; they now display in reversed order and remain selectable. Windows `-j 2`: full scene suite, 41 kernel editing, two exact extrusion, 44 workflows and 38 viewport passed; one native UI fixture ignored. Nested/NURBS polycurve scene admission, non-circular transforms, curved boundaries, original behavior and desktop remain open. Skill synchronized automatically; see `docs/validation/2026-10-28-solid-boss-source-map.md`.

## Ellipse extrusion surface-area integration correction

The 0.42% Boss ellipse wall discrepancy came from integrating a C0 extrusion without splitting at continuity intervals: temporary C2 splitting gives 79.327197914724 mm² vs independent 79.3271979465 mm². Measurement now applies that split only to C0/C1 SurfaceOfExtrusion faces, without changing the source. The main app oracle tightened to 1e-5 mm²; three kernel ellipses pass 3e-5 mm², with the 1×4 mm case still differing about 0.00002035 mm². Windows `-j 2`: 40 kernel editing, 16 creation, 2 exact extrusion, 43 app workflows and 37 viewport passed; one native UI fixture ignored. General surfaces, shared-face compounds, peak memory and desktop are not closed. Skill synchronized automatically; see `docs/validation/2026-10-28-solid-boss-source-map.md`.

## Boss planar NURBS profile and area discrepancy

Boss accepts closed planar NURBS via an exact OCCT planar-face/wire path; a translated rational ellipse fixture verifies face area, mesh points on the ellipse, joined topology, simulated selection and Undo. An independent numerical perimeter check exposes an unresolved wall-area discrepancy: OCCT 79.6584154731 mm² vs 79.3271979465 mm², approximately 0.42%. The tolerance in the app regression is 0.4 mm², not an exact-area proof. Windows `-j 2`: 39 kernel editing, two exact extrusion, 43 workflows and 37 simulated viewport passed; one native UI fixture ignored. General NURBS, polycurves, curved boundary, measurement accuracy, desktop and original behavior remain open. Skill synchronized automatically; see `docs/validation/2026-10-28-solid-boss-source-map.md`.

## Boss exact circular profiles

Boss now accepts exact full-circle profiles, including mixed selections with polygons, on a parallel planar boundary. The exact extrusion bridge builds uncapped circular side walls without tessellating the source; Rust transforms circle frames into boundary-local space and refuses partial arcs or placements that do not preserve circularity. Kernel, translated controller and simulated viewport fixtures check analytical area, topology and Undo. Windows `-j 2`: 38 kernel editing, two exact extrusion, 42 workflow and 36 viewport tests passed; one native UI fixture ignored. General curves, curved boundaries, native peak memory, original behavior and desktop remain open. See `docs/validation/2026-10-28-solid-boss-source-map.md`; skill synchronized automatically.

## Boss multiple polygon profiles

The staged Boss session now accepts multiple disjoint polygon profiles before selecting one planar BRep boundary. Kernel cuts and sews all profiles before one geometry transaction; overlapping or partially contained profiles are refused atomically. A two-profile fixture measures 172 mm² and nine faces; controller and egui tests verify selection Undo/reselect and document Undo. Windows `-j 2`: 36 kernel editing, 41 workflows and 35 simulated viewport passed; one native UI fixture ignored. General curves/boundaries, overlap behavior in Matrix, cumulative native peak memory, actual icon click and desktop remain open. Skill synchronized automatically; see `docs/validation/2026-10-28-solid-boss-source-map.md`.

## Boss staged planar-polygon command slice

Boss is now available for one polygon profile and one parallel planar BRep face: select profile, Enter, then select boundary. It keeps the curve and atomically replaces the boundary geometry; selection Undo/Cancel does not commit. Both placements are handled in boundary-local space for similarity transforms. Menu dispatch, CMD, translated inputs, viewport pointer simulation, invalid boundary, area and document Undo pass. Windows `-j 2`: 40 workflows, 34 simulated viewport and two focused Boss option tests passed; one native UI fixture ignored. `--no-default-features` fails in ten Box/Sphere symbols outside this slice. Multiple profiles, curved geometry, non-similarity placement, icon click, actual desktop and Matrix original semantics remain open. See `docs/validation/2026-10-28-solid-boss-source-map.md`; skill synchronized automatically.

## Boss existing planar BRep foundation

`boss_parallel_planar_face` now accepts an existing single planar BRep face rather than reconstructing it from a polygon. The bounded OCCT query returns the exact plane to Rust; the existing cut/sew path preserves a pre-existing hole. Tests verify 135 mm², five faces, one shell, 12 naked edges, source immutability and rejection of a profile over the hole, a solid boundary and a curved boundary. Windows `-j 2`: 35 kernel editing, 16 creation and one app workflow smoke passed. This is not a command/UI test; profile must be polygonal and parallel, and one planar face only. Placement, general geometry, multi-selection, Undo, desktop and original semantics remain open. Feature-port skills synchronized automatically.

## Boss parallel polygon foundation and uncapped extrusion repair

Boss kernel can now cut a planar polygon boundary and join the walls of a contained parallel polygon profile in either normal direction. It refuses nonparallel, coplanar, disjoint or partially outside profiles and verifies joined topology. The initial joined fixture failed because uncapped extrusion used an open wire for a closed polygon and missed its final wall. The selected OCCT bridge now distinguishes open wire, closed wire and capped face; measured closed/open regressions pass. Windows `-j 2`: 34 kernel editing, 16 creation and 38 app workflow tests passed; one native UI fixture ignored. This is a kernel-only, reconstructed polygon fixture, not selected BRep Boss, UI or original solver evidence. Feature-port skills synchronized automatically per user instruction.

## Boss boundary-trim kernel foundation

Added `editing::trim_surface_by_solid`, using the existing selected OCCT Cut bridge while Rust validates surface-only input, closed tool and owned snapshot. Planar and cylindrical boundary fixtures check partial cut, exact planar area/topology, disjoint and complete removal, invalid types and source preservation; all 31 kernel editing tests pass with `--features 3dm-brep` and `-j 2`. Initial invocation without the matching feature failed on existing 3DM bridge link symbols. This is not a complete Boss: cutting/choosing the extrusion walls, joining the boundary, command selection/transaction and actual UI remain open. See `docs/validation/2026-10-28-solid-boss-source-map.md`.

## Boss source and availability checkpoint

The normalized Boss text requires closed planar profiles extruded normal to their plane toward a surface/polysurface boundary, with that boundary trimmed and joined. Recovered menu registration associates `SolidBoss` with `_Boss`; it does not identify the original solver. OM9 has the palette icon but no Boss handler or boundary-trimming solver. A targeted Rust test verifies Boss, Rib and Slab palette catalog entries are unavailable; an initial UI test setup incorrectly assumed Solid was a toolbar tab and was replaced by the scoped catalog/availability test. Actual sidebar clicking remains untested. See `docs/validation/2026-10-28-solid-boss-source-map.md`; do not treat presence of an icon or plain extrusion as Boss completion.

## Full-circle Pipe under scaled, reflected placement

The user approved recording only the full-circle constant-radius Round rule; repository and installed feature-port skills now contain the same text and pass skill validation. A new controller regression applies a scale of two and reflection plus translation to an analytic circle, for None/Flat/Round and thin/Thick. Six cases measure area, applicable volume, no naked edges for None, world bounds and Undo; Windows `-j 2` targeted test passed. Radius input remains in world units. These controller results do not prove original Matrix behavior or desktop interaction; other affine and curve cases remain open.

## Pipe Cap=Round on a full circle

Previously constant-radius Round on a full circle was refused as requiring a line. OM9 now treats a closed circular rail as having no ends to cap and produces the same exact closed torus as Flat, including constant Thick. A controller regression first failed and now measures the closed BRep volume and Undo; a simulated viewport fixture selects the circle and checks both thin and Thick volume and Undo. Windows `-j 2`: 37 workflows and 33 simulated viewport tests passed; one native UI fixture ignored. This closed-rail cap interpretation is not established by the Matrix registration or manual; other frames, varying radii, partial arcs and actual desktop remain open. The user approved recording this rule; repository and installed feature-port skills were updated.

## Pipe Cap=None on a full circle

The sampled circular rail previously produced two naked seam edges and approximate area. Cap=None for a full analytic circle with circularity-preserving placement now extracts the exact toroidal face. Thick uses two disconnected faces with the inner topology reversed. A red-to-green controller fixture checks no solid, no naked edges, exact area, opposite display triangle winding for Thick and Undo. Windows `-j 2`: 36 workflows and 32 simulated viewport tests passed; one native UI fixture ignored. Test uses CMD and an untransformed circle; exact differential normals, transformed/viewport input, original Matrix semantics, Round, varying radius and actual desktop remain open. Skill update requested.

## Thick constant Pipe on a full circle

Previously Thick on a circular rail was rejected as requiring a line. A full analytic circle with circularity-preserving placement and Cap=Flat now produces a hollow solid by subtracting exact inner and outer torus bodies in the circle frame; inner and outer radii must differ and the outer radius must be smaller than the transformed ring radius. A red-to-green fixture checks reversed entry order, measured BRep area/volume, one closed solid and single-transaction Undo for menu dispatch and CMD on a tilted, placed circle. Windows `-j 2`: 35 workflows and 32 simulated viewport tests passed; one native UI fixture ignored. Other cap modes, varying radii, partial arcs, non-circular affine placement, original Matrix behavior and actual desktop remain open. Skill update requested.

## Exact constant Pipe on a full circle

The capped constant-radius branch previously sampled the circular rail and lofted through the samples: a placed tilted circle produced volume 98.69584183197594 mm³ rather than 10π² mm³, with additional seam faces. For a full analytic circle whose placement preserves circularity, Cap=Flat now creates an exact torus in the transformed circle frame; the tube radius must be less than the ring radius. A regression failed before the change and now checks exact BRep volume, area, bounds, one solid and Undo through menu dispatch and CMD. Windows `-j 2`: 34 workflows and 32 simulated viewport tests passed; one native UI fixture ignored. Cap=None, Round, Thick, variable radius, partial arcs and non-circular affine placement remain separate cases; original Matrix implementation and actual desktop are not established. Skill update requested.

## Thick Pipe endpoint radius order

Previously the equal-radius validation paired inputs by entry order even though the geometry builder determines inner and outer radii by size. Reversing their order at the end caused an erroneous refusal. The equal-required branch now compares and normalizes inner against inner and outer against outer within document absolute tolerance; Global straight taper remains unchanged. A regression first failed on the old code and now verifies closed BRep volume and document Undo on a straight and a bent chain with reversed, near-equal endpoint inputs. A separate short 120-degree Round case is rejected by OCCT as an invalid sweep without mutation; this is not proof of general short-chain support. Windows `-j 2`: 33 workflows, 32 simulated viewport, and 29 command option tests passed; one native UI fixture ignored. Matrix's original entry-order and tolerance semantics and real desktop remain unverified; skill update requested.

## Pipe Round caps on non-right-angle and spatial chains

Measured the existing constant-radius miter and spherical-cap implementation on 45-degree and 120-degree turns and a three-segment spatial chain, for both thin and Thick Pipe. All six controller cases create one closed BRep with the analytical volume and restore the original document on Undo. Windows `CARGO_TARGET_DIR=L:/om9-target`, `cargo test -j 2 -p om9-app --test solid_workflows`: 31 passed, one native UI fixture ignored. This extends automated geometry coverage, not proof of Matrix's original transition behavior or desktop input. Overlapping, short, self-intersecting and curved rails, other caps and varying radii need separate evidence.

## Pipe endpoint radius points on bent chains

Radius points at the start and end now use the corresponding first/last segment tangent for the section plane, including Thick. A CMD fixture with axial offsets checks thin and Thick volume; an egui BRep-edge fixture picks endpoint points and checks volume/Undo. The first viewport run failed because projected radii differed by roughly 4e-6 mm. For the branch requiring equal radii, values within the document absolute tolerance are now normalized to the first value; missing document tolerance uses OM9 template 0.001 mm. A measured straight Global chain regression preserves a 0.0005 mm radius difference. This is OM9 behavior, not a recovered original rule. Windows `-j 2`: 30 workflows, 32 viewport, 6 Pipe option and 56 Solid input passed; one native UI fixture ignored. Other CPlanes/orientations, radius-point projection semantics vs native, Diameter points, intermediate points on bends and desktop remain open. Skill update requested, awaiting consent.

## Pipe Round caps on bent constant-radius chains

Straight-segment bent chains with constant radius now accept Round caps, including constant-inner/outer Thick. The existing miter sweep is united with endpoint spheres; for Thick the inner rounded body is subtracted. Endpoint segments must be at least the outer radius to keep each inward sphere half inside its straight sweep segment. A 90-degree egui BRep-edge fixture checks closed solid volume and Undo for thin and Thick; a short endpoint segment is refused without mutation. Windows `-j 2`: 28 workflows, 31 viewport and 6 Pipe option passed; one native UI fixture ignored. Overlapping ends/other parts, acute and self-intersecting chains, original miter/cap semantics and actual desktop remain unverified. Varying radius on bends remains unsupported. Skill update requested, awaiting consent.

## Pipe Thick on bent linear chains

For connected straight-segment chains using the OM9 miter transition, Thick now accepts constant inner and outer radii. Flat caps produce a hollow solid by Boolean Difference; None produces two open walls with the inner topology reversed. An egui 90-degree BRep-edge fixture checks closed volume, open area, opposite display-winding signs and document Undo. A controller fixture rejects varying endpoint radii without mutation and retries successfully. Windows `-j 2`: 27 workflows, 30 viewport, 6 Pipe option and 16 kernel creation passed; one native UI fixture ignored. Exact differential normals, acute/complex rails, original miter semantics, Round/variable-radius bends and actual desktop remain open. Skill update requested, awaiting consent.

## Pipe bent linear ChainEdges

Connected Line/straight BRep-edge chains can now bend. Rust validates selection, joins and reversals; a selected OCCT pipe-shell adapter sweeps a true circle along the polyline with RightCorner (miter) transitions. Constant radius and Flat/None caps are supported; unequal radius, Thick, Round and point/station input on bent chains remain unsupported. A 90-degree kernel fixture checks closed volume, open output and reversal rejection; an egui fixture selects two BRep edges and checks volume, open/closed result and Undo. Controller tests reject unsupported variants atomically. Windows `-j 2`: 16 kernel creation, 26 workflows, 29 viewport and 6 Pipe option passed; one native UI fixture ignored. Miter choice is OM9 interpretation, not verified original solver/default; acute/complex/self-intersecting paths, source topology and actual desktop remain open. Skill update requested, awaiting consent.

## Pipe chain endpoint and intermediate points

The accepted ChainEdges geometry now supplies the common straight rail to Radius point handling and Global intermediate stations, including mixed root curves and BRep edges. A simulated viewport fixture selects a mixed chain, picks both endpoint radii, supplies a middle station via CMD, and verifies analytical BRep volume, highlight cleanup and document Undo. Windows `-j 2`: 25 workflows, 28 viewport, 6 Pipe option and 56 Solid input passed; one native UI fixture ignored. Projection and C0 station interpolation are OM9 interpretations not verified from the original; only straight collinear chains and non-Thick Global Flat/None stations are supported. Other views, Diameter-by-point, Thick/Round stations, general chains and actual desktop remain open. Skill update requested, awaiting consent.

## Pipe mixed curve/BRep edge chains

ChainEdges now keeps one ordered list of root Line curves and BRep line subobjects, with source stamps and edge publication checks. Mixed selections can be made in either order; Undo removes the last selected item regardless of kind. The existing connected forward-collinear limitation still applies. An egui viewport test covers both selection orders, Undo/reselect, BRep volume and document Undo. Windows `-j 2`: 25 workflows, 27 viewport, 6 Pipe option and one icon test passed; one native UI fixture ignored. Bends, curved or instance edges, ambiguous candidates, AutoChain, original chaining rules and actual desktop remain open. Skill update requested, awaiting consent.

## Pipe ChainEdges straight BRep edges

ChainEdges now accepts ordered BRep edge subobjects, provided the chain is connected and collinear. The selected OCCT bridge checks the exact edge curve type and oriented topological endpoints; it rejects curved edges instead of treating tessellation segments as geometry. Selection retains source stamps/publication, Undo removes the last edge and Cancel clears highlights without commit. A simulated viewport selects two edges from translated boxes, checks resulting BRep volume and Undo; a kernel fixture checks box lines and rejects cylindrical circular edges. Windows `-j 2`: 25 workflows, 26 viewport, 6 Pipe option, one icon and one kernel edge test passed; one native UI fixture ignored. Mixing curve roots and BRep edges, instances, curved/non-collinear chains, ambiguous edge disambiguation, AutoChain and actual desktop remain open. Skill update requested, awaiting consent.

## Pipe ChainEdges partial collinear curves

The recovered right-click binding now starts a Pipe chain selection session; `_Pipe _C`, `PipeChain` and the ChainEdges option reach the same owner. The current path accepts selected root Line curves in selection order, reverses segments as needed, verifies joined forward collinearity within 1e-6 mm, and constructs one straight Pipe using the common geometry builder. A simulated viewport fixture checks two reversed segments, BRep volume and atomic Undo; a controller test rejects gaps and corners without commit. An egui right-click icon test and keyboard/token/typed ChainEdges option test passed. Windows `-j 2`: 25 workflows, 25 viewport, 5 Pipe option plus targeted ChainEdges option and targeted icon tests passed; one native UI fixture ignored. BRep edge selection, non-collinear and curved chains, AutoChain, continuity/gap options, intermediate points, original solver/defaults and actual desktop remain open. Skill update requested, awaiting consent.

## Pipe Multiple selection

Multiple stages curve selections, then Enter begins shared numeric radius input. A common geometry builder is used by single and Multiple routes; outputs are prepared before one atomic transaction. A later unsupported rail leaves the document untouched. Simulated viewport tests select two rails, Undo/reselect and verify two BReps and one document Undo. Controller tests cover Global, Thick and Round on two straight rails with analytical volumes and a late incompatible curve. The option is exercised via egui keyboard, token and typed-name routes. Windows `-j 2`: 24 workflows, 24 viewport, 5 Pipe option and 56 Solid input tests passed; one native UI fixture ignored. Shared radii are an OM9 interpretation, not established original behavior. Multiple point-based radii, intermediate stations, curved variable rails, native UI and desktop remain open. Skill update requested, awaiting consent.

## Pipe intermediate Global stations on straight rails

After endpoint radii, a point picks an interior station and the next number or point gives its radius; repeated stations are sorted along the rail for commit. Undo drops the pending location or the most recently added station. Position projection to a straight rail is an OM9 interpretation, not original-code proof. Only non-Thick Global with Flat/None is exposed. A Rust station contract and OCCT ruled circular-section loft produce piecewise linear conical sections; these join at C0 and equivalence to original interpolation is not established. Tests check analytical volume for one and two stations, unsorted entry and Undo, invalid endpoints/duplicate stations, egui viewport and CMD. Windows `-j 2`: 15 kernel creation, 21 workflow (plus one later targeted multi-station test), 23 viewport and 4 Pipe option tests passed; one native UI fixture ignored. Curved rails, Thick/Round stations, Local blending, smoothness, native equivalence and actual desktop remain open. Skill update requested, awaiting consent.

## Pipe radius by point on straight rail

In Radius mode, viewport world points and comma-separated CMD coordinates enter endpoint radii on a straight Line. OM9 projects the point into the section plane perpendicular to the transformed rail, and uses its distance from the corresponding endpoint axis; this is an implementation interpretation, not a verified native rule. Thick uses the start endpoint for its first two points and the end for its next two, but that branch is not separately tested. Diameter-by-point and non-Line rails explicitly return unsupported. Simulated viewport tests cover either side of a horizontal rail and BRep volume/Undo; a CMD test uses translated geometry, points offset axially and radially, zero radius rejection and BRep volume/Undo. Windows `-j 2`: 20 workflows, 22 viewport, 4 Pipe option and 56 Solid input tests passed; one native UI fixture ignored. Other views/CPlanes, rotations, Thick, point undo, native equivalence and desktop remain open. Skill update requested, awaiting consent.

## Pipe default radius stages

The interactive default now takes start radius, end radius and Enter before creation, matching the documented sequence up to the still-missing intermediate point stage. Previously the first number committed immediately. Equal radii use the existing constant pipe path, including non-Line curves. Different radii require explicitly choosing Global because original default blending has not been established; pending values survive that choice. The direct compatibility syntax `Pipe 1` remains a one-shot constant-radius command. Menu/CMD controller tests check missing input, rejection without mutation, Global retry, BRep volume and Undo; egui viewport and options regressions were updated. Windows `-j 2`: 19 workflows, 4 Pipe option, 21 viewport, 56 Solid input and 70 Solid tests passed; one native UI fixture and one Solid fixture ignored. Intermediate radii, Local blending, point input and actual desktop remain open. Skill update requested, awaiting consent.

## Pipe Round straight Global taper and Thick

The selected OCCT bridge now builds a straight conical or cylindrical body and exact hemispheres at the endpoint planes, fusing them into one BRep. Rust validates the rail and radii. `Cap=Round` accepts distinct Global endpoint radii; Thick subtracts a similarly capped inner body. Kernel tests cover equal, increasing and decreasing radii with closedness, analytical volume and axial bounds. Menu/CMD controller tests cover translated non-Thick and Thick tapers, volume, endpoints and atomic Undo. Windows `-j 2`: 14 kernel creation, 18 workflows, 4 Pipe option, 21 viewport passed; one native UI fixture ignored. Arbitrary orientation, extreme radii/lengths, curved rails, cap/side continuity, native source topology and actual desktop remain unverified. Skill update requested, awaiting consent.

## Pipe Round caps on straight constant-radius rail

`Cap=Round` now creates a capsule from a straight Line and a constant radius, by union of the cylindrical body and endpoint spheres; a Global session with equal end radii uses the same path. Varying end radii, Thick and non-Line rails return unsupported. Menu/CMD controller tests on a translated rail verify one closed solid, analytical capsule volume and bounds, unsupported varying-radius retry and Undo. Cap cycles Flat/None/Round in simulated keyboard, token and typed-name routes. Windows `-j 2`: 17 workflows, 4 Pipe option and 21 viewport tests passed; one native UI fixture ignored. Other radii, rail lengths/placements, curved/Thick round caps, source-equivalent topology and desktop remain unverified. Skill update requested, awaiting consent.

## Pipe Thick open wall orientation

The previous two-wall result had both display triangle normals pointing outward. A new regression reproduced that defect, then passed after the existing OCCT sew bridge was extended to reverse the second input and the Pipe controller used it for the inner wall. Four straight constant/Global menu/CMD cases check opposite radial signs in tessellated triangle winding, two faces, four naked edges and area. A 5e-8 mm gap is rejected without a transaction; no successful case immediately above the 1e-7 mm threshold was established. The existing OCCT extract/unjoin/rejoin regression passed after the bridge ABI change. Windows `-j 2`: 16 workflows, 3 Pipe option, 21 viewport and one kernel rejoin test passed; one native UI fixture ignored. Exact differential normal sampling, near-threshold acceptance, general rails, source-equivalent topology and actual desktop remain open. This fixes the previously intended wall orientation, rather than adding a business rule.

## Pipe Thick open walls

`Thick=Yes, Cap=None` on a straight Line now creates a BRep containing two disconnected open wall surfaces. Outer and inner circular-section lofts are combined by the existing OCCT sewing adapter with tolerance `1e-7` mm; wall separation at each endpoint must exceed that tolerance. Four controller cases cover menu/CMD and constant/Global taper. Exact BRep measurements check two faces, four naked edges, no solid, area and atomic Undo. Windows `-j 2`: 15 workflows, 3 Pipe option and 21 egui viewport tests passed; one native UI fixture ignored. Surface orientation, near-tolerance boundary cases, general rails, round caps, desktop and source-equivalent topology remain unverified. Skill update requested, awaiting consent.

## Pipe Thick straight Global

`Thick=Yes` stages two radii at each endpoint. For a straight Line rail with flat caps and Global blending, the controller builds outer and inner tapered solids and subtracts them to create a hollow BRep. Equal radii at either endpoint are rejected, and toggling Thick after entering radii requires Undo first. A translated-rail controller test covers menu/CMD, BRep bounds and hollow volume, invalid radii, unsupported Cap=None, retry and atomic Undo. The Thick option is exercised through egui keyboard, token and typed-name routes. Windows `-j 2`: 14 workflows, 3 Pipe option, 21 viewport and 56 Solid input tests passed; one native UI fixture ignored. General rails, open/round thick caps, intermediate stations, point input and desktop acceptance remain open. Skill update requested, awaiting consent.

## Pipe command option interaction

egui simulation verifies keyboard shortcut, token click and typed-name activation of Diameter, Cap and Global. Current option values update, and resulting open pipe diameter or Global frustum volume is measured. Re-selecting Global previously discarded the entered start radius; it now retains that value. Windows `-j 2`: 25 shared option, 13 workflow and 21 Solid viewport tests passed; one native UI fixture ignored. Simulated egui interaction does not establish actual desktop behavior. Only the exposed partial options are covered; remaining Pipe modes and original defaults remain open. This corrects input retention and does not add a reusable business rule.

## Pipe Radius/Diameter input

Pipe accepts Radius or Diameter mode; values in Diameter mode are converted to radius before geometry creation. Switching modes between Global endpoints preserves the previously entered radius. The shared command option descriptor exposes its current value and shortcut; Cap also exposes its current choice. Four controller scenarios cover menu/CMD and constant/Global modes, option activation, typed mode switch, invalid input, BRep volume and Undo. Windows `-j 2`: 13 workflows, 21 egui viewport and 56 Solid input tests passed; one native UI fixture ignored. Actual option token click/keyboard, point input, desktop and the original default remain unverified. Skill update requested, awaiting consent.

## Pipe Cap None and Flat

`Cap=None` creates an open shell and `Cap=Flat` creates a capped solid in the existing constant-radius path and the straight-line Global path. `Cap=Round` returns unsupported. The prior OM9 flat default remains; the original Matrix default has not been established. Eight controller combinations cover menu/CMD, constant/Global and None/Flat, checking BRep closure, length, closed volume and Undo; invalid Round is rejected before retry. Windows `-j 2`: 12 workflows, 21 egui viewport and 56 Solid input passed; one native UI fixture ignored. Cap token clicks/shortcuts, round caps, general Global rails and actual desktop remain unverified. Skill update requested; awaiting consent.

## Pipe Global straight-rail taper

Explicit `ShapeBlending=Global` now stages positive start and end radii and commits on Enter. The current command implementation accepts straight Line rails only and creates a flat-capped BRep; additional radius stations are explicitly unavailable. Menu and CMD controller tests check a translated frustum's exact volume and bounds, invalid input retry and document Undo. A kernel test checks increasing, decreasing and equal radii, open/closed output, and invalid input. Windows OCCT SDK: 13 kernel creation, 11 workflow, 21 egui viewport and 56 Solid input tests passed; one native UI fixture ignored. The kernel also accepts polyline stations but curved/path loft accuracy was not tested and is not exposed by this command. Local blending, intermediate radii, curve selection/point entry, cap options and actual desktop remain open. Skill update requested; awaiting consent.

## Pipe overlapping curve selection

Overlapping curve picks now use the object selection popup and retain hit order. Selecting a row routes to the active Pipe session; a popup from a cancelled earlier session is rejected. An egui pointer test chooses the longer of two overlapping straight rails and verifies the resulting BRep volume; it also checks stale popup rejection after cancel/restart. Windows `-j 2`: 21 viewport, 10 workflow and 56 Solid input tests passed; one native UI fixture ignored. Choosing the actual popup row with desktop input, different views, and radius/options semantics remain unverified. Skill update requested, awaiting consent. Earlier rejection-only checkpoint below is historical.

## Pipe in-command single-curve selection

The Pipe session now asks for one curve when the existing selection is not exactly one curve, and accepts a viewport pick before numeric radius input. A valid preselection goes straight to the radius prompt. Overlapping curve hits are rejected rather than silently choosing one. Undo clears selection; Cancel and selection do not commit. Simulated egui tests invoked menu dispatch and CMD, picked a translated rail, checked actual BRep bounds, volume and document Undo; a second test checked rejection of a BRep, selection Undo and Cancel. Windows `-j 2`: 20 viewport, 10 workflow and 56 Solid input tests passed; 1 native UI fixture ignored. This is not actual desktop acceptance; overlap selection UI, Multiple/ChainEdges, start/end and intermediate radii and other documented options remain open. Skill update requested, awaiting consent.

## Pipe placed-rail checkpoint

Fixed the sampled rail using local instead of world coordinates. Translated and rotated/translated straight rails now produce BRep at the expected position through menu dispatch and CMD; four combinations checked exact bounds, volume and Undo. The full controller workflow suite passed 10 tests (one native UI fixture ignored), and 56 Solid input regressions passed. This is not desktop or viewport mouse evidence. The documented Pipe options, interactive selection and general rail behavior remain open; see `docs/validation/2026-10-28-solid-pipe-source-map.md`.

## Fillet CopyHandle checkpoint

CopyHandle enters source-handle selection, then accepts repeated destinations:
a new location on a selected open edge creates a control using the source
radius, while an existing handle receives that radius. Enter returns to the
preview stage and a later Enter commits. The implementation uses the existing
projection, topology selection and preview-rebuild paths; invalid destination
selection retains the copy session and preview. A straight box edge was tested
with source, new control, existing endpoint, BRep volume comparison and
commit/Undo. Simulated keyboard, token and typed CopyHandle activation passed.
Windows `-j 2`: 56 Solid input, 18 viewport, 9 workflow and 5 Fillet option
tests passed; one native UI fixture ignored. Curved/closed edges, actual
desktop, dragging, large-model performance and native rebuild-failure cases
remain unverified. Skill update requested, not yet approved.

## Fillet RemoveHandle checkpoint

RemoveHandle is offered only when at least one added control exists. In its
selection mode, clicking an added handle removes it and rebuilds preview;
default endpoint handles are refused. Enter returns to the preview stage and
another Enter commits. A box fixture in egui added two
controls, rejected removal of a default handle, removed both in order,
confirmed the constant-radius BRep volume and committed/undid the result.
Keyboard, token and typed option activation were exercised in a separate
simulated UI test. Windows `-j 2`: five Fillet option tests, 56 Solid input,
17 viewport and 9 workflow tests passed; one native UI fixture ignored.
The code restores the removed control when rebuilding fails, but a native
rebuild-failure case was not induced. Actual desktop interaction, closed and
general curved edges, CopyHandle and dragging remain open. Skill update
requested, not yet approved.

## Fillet AddHandle on an open edge

The application now offers AddHandle in Fillet preview. The user can choose
CurrentRadius and click selected source edges repeatedly; the picked point is
converted through the object placement and projected onto the exact OCCT edge,
then represented by oriented arc-length fraction. Each accepted point rebuilds
the preview; duplicate/end locations and failed builds keep the previous
preview. Enter exits AddHandle before the separate commit. Added handles can
be selected and edited and participate in SetAll and LinkHandles. Undoing a
source-edge pick removes controls belonging to that edge.

A translated box fixture selected a straight edge, added two handles through
egui pointer events, checked the resulting BRep against the kernel, verified
duplicate/error rollback, edited the added handle, and checked SetAll,
LinkHandles, commit and Undo. AddHandle option activation was tested via
simulated keyboard, pointer and typed routes. Windows `-j 2`: 28 kernel,
56 Solid input, 16 viewport, 9 workflow and 5 Fillet option tests passed; one
native UI fixture ignored. The original AddHandle default radius is not
verified: OM9 starts with the active handle radius or last NextRadius. Closed
edges, general curved edges, nonuniform placement, dragging, CopyHandle,
RemoveHandle, performance and actual desktop UI remain open. The native
adapter projects and constructs OCCT geometry; Rust owns control state,
budgets, preview and publication. Skill update requested, not yet approved.

## Fillet interior radius control kernel checkpoint

The OCCT fillet bridge accepts interior radius controls identified by an edge
and oriented arc-length fraction, maps them to the contour and passes the
constraint array to OCCT for interpolation. Rust rejects unselected edges,
out-of-range or duplicate positions and nonpositive radii, and limits control
counts and array bytes. A straight vertical box edge was measured with a
midpoint control and with 25% and 75% controls; cross-sections confirmed local
changes in the oriented direction. A two-segment open tangent contour was also
checked with one interior control; incomplete contour selection was rejected.
Windows `-j 2`: 28 kernel tests passed after the final bridge edit, along with
six focused Fillet regressions; 56 Solid regressions ran before the last
native overflow guard, and 15 simulated viewport and 9 workflow tests passed
after it (one native UI fixture ignored). App AddHandle/CopyHandle/RemoveHandle,
viewport point projection, closed contours, curved-control accuracy and actual
desktop UI are not implemented or verified. This is a kernel capability, not
an interactive AddHandle implementation. Skill update requested, not approved.

## Fillet shared handle viewport selection

Two coincident handles on the same topological vertex and tangent contour
previously produced an ambiguous-pick error. The controller now groups those
near-equal screen hits by native topology/contour identity before selecting a
handle; unrelated equally close hits remain ambiguous. An egui viewport test
first reproduced the failure, then passed after the fix, editing the shared
radius and committing/undoing on a two-segment prism. A controller regression
also rejected nearby but unrelated handle IDs. Windows `-j 2`: 56 Solid input,
15 viewport and 9 workflow tests passed; one native UI fixture ignored. This
does not prove actual desktop selection, curved/closed contours or large-model
latency. Skill update requested, not yet approved.

## Shell multi-face kernel checkpoint

The OCCT shell adapter accepts multiple distinct removed faces on one closed
solid, retaining at least one face and offsetting inward with positive
thickness. A 10 mm box opened at opposite faces with 1 mm walls measured
360 mm³ in either input order; invalid indices, duplicates, all-face selection,
zero thickness and open input were rejected. Windows `-j 2`: 27 kernel tests,
54 Solid input regressions and 9 workflow tests passed; one native UI fixture
was ignored. The app still exposes only its previous single numeric face path;
multi-face selection, viewport, arbitrary geometries and desktop remain open.
The derived registration note is `docs/validation/2026-10-28-solid-shell-source-map.md`.
Skill update requested, not yet approved.

## Shell staged multi-face application checkpoint

Shell now stages selected faces on one root BRep. Thickness can be supplied
before finishing selection, or after Enter; the exact original default is not
inferred. Commit replaces the source once, and failed construction preserves
the document. Tests covered two opposite box faces (360 mm³), Undo/Redo,
all-face rejection, selection Undo and Cancel, two adjacent face picks in egui
viewports, and Thickness via keyboard, token and typed assignment. Windows
`-j 2`: 56 Solid regressions, 22 command option tests, 14 simulated viewport
tests and 9 workflows passed; one native UI fixture ignored. Following the
final budget check and test edit, Shell-specific controller, viewport and
Thickness-route tests were rerun. Native UI, arbitrary topology, instance faces,
History and source defaults remain open. Skill update requested, not approved.

The M03 Solid owner `OM9-L2-03-18` names 21 L3 bindings. The separate recovered-menu inventory in `docs/analysis/matrix-command-layers.json` has 75 entries in 14 groups: Box 5, Sphere 7, root 15, Ellipsoid 2, Paraboloid 2, Extrude Planar Curve 4, Extrude Surface 4, Extrude Surface to Boundary 4, Fillet Edge 3, Boolean Tools 6, Edges 4, Faces 8, Holes 10, and Solid Edit Tools 1. These are different inventories, not 96 distinct implemented commands. The current standalone palette `Resources/menu/MainMenu.ini` lists 30 icons, including Solid Extrude All, which is not in that recovered-menu enumeration. A menu entry or source inventory is not proof of a working implementation. The reproducible [entry-by-entry source crosswalk](2026-10-28-solid-inventory.md), generated by `tools/audit_solid_inventory.py`, identifies 20 exact L3 icon matches; it does not classify unmatched variants as unsupported or complete.

The M03 owner explicitly excludes mechanical Boss, Rib, and Slab; preserve their visible disabled state and do not infer an implementation mandate from the recovered inventory. Other recovered entries and variants need individual owner, UI and runtime review. In particular, the existing 21-binding automated receipt does not establish the remaining recovered entries, their alternate scripts, pointer interaction, parameter options, or human acceptance.

For Pyramid, the bounded VB6 menu registration identifies the Pyramid icon and separate primary and star variant invocations; it does not establish the geometry algorithm. The normalized M03 L3 record preserves those variants. The existing Rust command converted a floating-point side count directly to an integer, so a fractional count could silently produce a different solid. The input path now rejects non-integral or out-of-range counts; the star variant remains unverified and is not implemented by this correction.

The common positional Solid prompt previously appended the valid prefix of an invalid submission and dropped the active prompt after an operation error. The working-tree change parses a submission atomically and retains the previous stage after a pre-commit failure. Boolean Two Objects and Text similarly retain their prompt after a pre-commit dispatch failure. The document revision prevents reopening the prompt if an error happens after a transaction was already published.

Boolean Difference previously took its operands from `Controller.selection`, a `BTreeSet` ordered by object ID, then subtracted the second BRep from the first. The older symmetric overlapping-box volume assertion could not detect reversal. The command now reads ordered selection, and selection order is maintained across replacement, single picks, additive and toggle selection, removal, pruning and reset. The two-preselected-object slice was tested first; the later grouped-selection slice and its limits are documented below.

Validation on Windows with `CARGO_TARGET_DIR=L:/om9-target` and `-j 2`: `cargo test -p om9-app --lib solid_input_regressions` passed 5/5; `cargo test -p om9-app --lib solid` passed 7/7 (includes the same 5). Public source audit passed with 0 errors. The initial integration-test build failed before running tests because an existing OM9 process held `L:/om9-target/debug/om9.exe`; the process was left running. The tests were moved to a library test module, which avoids rebuilding that executable. Pruning retained the ten newest runs plus 116 older runs referenced by records and removed none. Git blob IDs at validation: `solid.rs` 9c6590f10866b327a0573cc79935278d6e4b71f9; `solid_input_regressions.rs` deb44aa041c397d4812ff80a410a6346c121c81b; `session.rs` 931ff9cc1f4e780c6bdea2ec3e029fea77fa0a14. The tests do not verify Text dispatch failure, post-publication refresh failures, actual UI interaction, or all Solid modes.

## Boolean operand order follow-up

An asymmetric two-box regression first failed with the reversed selection returning 937.5 mm³ instead of 62.5 mm³. With deterministic IDs and the fix, it checks 32 combinations: four selection entry methods, both operand orders, and Difference or Boolean Two Objects through menu dispatch and CMD. The correct measured results are 937.5 mm³ and 62.5 mm³, with Undo/Redo checks. These are controller tests, not pointer-click or GPU evidence. A separate test covers remove/reselect, toggle, clear, replacement and selection revision on reordering.

The previously running OM9 process exited. On Windows with the external target directory and `-j 2`, `cargo test -p om9-app --lib solid_input_regressions` passed 7 tests; the Boolean test includes all 32 combinations. The integration batch `cargo test -p om9-app --test solid_workflows --test solid --test layer_controller --test layer_visible_locked --test matrix_selection_sequence --test acceptance_selection_set` passed 98 tests, with 2 native-UI fixture tests ignored. The former `solid_workflows` failure was a stale fixture submitting three numbers to the Box menu; the fixture now uses the approved default point/point/height interaction and tests Sphere through center/radius points. No product behavior was changed for that fixture. `tools/public_source_audit.py` passed with zero errors; pruning retained the ten newest and 116 cited older runs, removed none. Current Git blob IDs: `solid.rs` 91e43180b97a685c4f3a3bc904ba01d4d6416fe1; `layers.rs` 3c53f4cd0170e7c7787f9764adaaf1006bb24110; `solid_input_regressions.rs` 67adeac4b0c2ab684e3b2724f42264eae181c494; `solid_workflows.rs` aa0664f52e415637e01f62a68dc5c9649d173947.

Follow-up selection checks: `layer_select_by_layer` 3 passed, `subobject_selection` 8 passed, and the library test `plugin_selection_is_validated_before_publication_and_applied_after` passed. All 38 `plugin_native` tests were ignored because the native plugin package was not supplied; they are not counted as passes or as external SDK proof.

Next: continue the command-by-command crosswalk of the 75 recovered entries and the 30 standalone palette icons, including exact variants and disabled reasons; inspect bounded source procedures where needed. Verify actual icon/menu/CMD paths, geometry, options, cancellation, Undo and UI before promoting any status. The two native UI fixture tests remain ignored pending real saved fixtures.

## Multi-object Boolean Union follow-up

The normalized milestone requires exact union and explicitly allows disconnected results. The historical `OM9-SOLID-003` feature summary also identifies disconnected components, while the bounded recovered menu registration points to the Rhino Boolean Union command. The source address mapping and its limits are in `docs/validation/2026-10-28-solid-union-source-map.md`; neither the registration nor the local regression establishes all original Rhino input semantics.

The earlier standalone owner required exactly two BReps. At this checkpoint Union was extended to accept at least two preselected BReps and fold exact OCCT results before one host transaction; it still deleted the selected inputs. The later Union selection/DeleteInput follow-up below supersedes that input behavior. Boolean Two Objects and other Boolean commands retain their two-object limit. A failing later operation publishes no partial result.

The regression first failed because three BReps were rejected. After the change, `cargo test -j 2 -p om9-app --lib solid_input_regressions` passed 11 tests: among them, three translated boxes through menu and CMD produce one BRep with two closed solids and volume 2500 mm³ (both an initially connected and an initially disconnected operand order), with one transaction and Undo/Redo. Non-BRep and late open-BRep inputs preserve document and Redo; Boolean Two Objects rejects three inputs. The complete Boolean order test from the earlier slice is included in the 11. `cargo test -j 2 -p om9-app --test solid --test solid_workflows` passed 78 tests, with 2 UI fixture tests ignored. A subsequent `cargo test -j 2 -p om9-app --lib boolean_union_` passed 3 tests, including save/open of the compound in `.om9`, where ID, two solids, and volume are checked again. The public-source audit returned zero errors and artifact pruning removed no runs, preserving 116 cited older runs. Windows used `CARGO_TARGET_DIR=L:/om9-target`. Git blob IDs: `solid.rs` d0fc0e08478d0d6b39e9a43eec5f892df1864a07; `solid_input_regressions.rs` 9662fd78a8f6269c6ca9937d6554074636c70e42 for the 11-test run, and 1727a792de9f96f3ea12a5e4188d348817b90540 for the later 3-test run. These are controller and native-kernel tests, not pointer UI or human acceptance.

## Boolean Difference grouped selection

Without exactly two preselected objects, menu and CMD now start a target-selection stage followed by a cutter-selection stage. Enter advances each stage, Undo removes the latest selection or returns to target selection, and Cancel leaves the document unchanged. Captured target source stamps are revalidated. Each target is cut by each cutter, all outputs are prepared before a single transaction, and a failing later cutter leaves the document and Redo intact. Object picking uses the scene's selectable, revalidated hits; overlapping hits show a chooser. Chooser state is tied to the command run and stage so an old popup cannot select an object for a later run. The UI check is an egui event simulation, not an actual desktop click or human sign-off.

At this checkpoint, exactly two preselected objects still executed immediately; targets were cut independently and cutters were deleted. The later follow-up below replaces those behaviors. Supported input is selected BRep; other geometry categories, History, SDK and human M03-UI remain open. See `docs/validation/2026-10-28-solid-difference-source-map.md` for the bounded menu/decompiler mapping and distinctions.

Windows, `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: `cargo test -p om9-app --lib difference_` passed 7 tests, covering two targets/two cutters, single preselection, both menu and CMD, invalid/empty input, overlapping target/cutter refusal, pending Undo, Cancel, stale source refusal, late open-BRep failure/retry with existing Redo preserved, atomic Undo/Redo, and the earlier asymmetric operand order regression. The integration batch `cargo test -p om9-app --test solid --test solid_difference_ui --test solid_workflows` passed 80 tests and ignored 2 native-UI fixtures. `command_options_interface` separately passed 9 tests. The new UI test covers viewport selection and overlapping-candidate/stale-popup routing; its Enter submissions call the controller, so physical keyboard input and the real executable remain unverified. `cargo check -p om9-app --no-default-features --lib` failed with ten missing Sphere/Box helper errors; default-feature tests above passed, and no no-OCCT build is claimed. Public-source audit reported zero errors; pruning kept cited runs and removed none. Git blob IDs at validation: `solid.rs` bca1c322b001de69deea5d913727e2ebeacb471d, `session.rs` 79f42306889628310416eef7a7c48fde731bd593, `ui.rs` 3a7e7e1a2b0eb96c5f2e1487d662413b6045055d, `solid_input_regressions.rs` 16154ed680488c14c8fbd3488ffbd01aa1bcf2f5, and `solid_difference_ui.rs` 5251d5a1bcfb5431a4f8b31bec36a8c51ae76f8b.

## Difference group fusion and DeleteInput

Difference now always enters target and cutter stages, including when two targets were preselected; Boolean Two Objects retains its separate two-object behavior. It fuses the selected target BReps, fuses the cutter BReps, and computes their difference before one transaction. Targets are replaced; `DeleteInput=No` retains cutters and `DeleteInput=Yes` deletes them. The option resets to No on each run and uses the shared command-option owner, including keyboard, pointer and typed activation. No transaction is created merely by changing it. A direct positional dispatch cannot bypass the stages. The historical engineering record identifies No as an OM9 host decision, not proof of the original Matrix default. The user explicitly approved recording the tested OM9 rule on 2026-10-28; both repository and installed `openmatrix9-feature-port` skill copies were synchronized, without promoting the UI gate.

On Windows with `CARGO_TARGET_DIR=L:/om9-target` and `-j 2`, `cargo test -p om9-app --lib difference_` passed 10 tests on the final source. The overlapping two-target/two-cutter case verifies one closed result of 1404 mm³, both DeleteInput values, identity/presence of cutters, one transaction and Undo/Redo. It also tests invalid options, preselection, inline invocation, empty result, stale target and late open-BRep failure with Redo preserved. On the final production source, the viewport/workflow batch passed 10 tests and ignored one native-UI fixture, and the dedicated DeleteInput option test passed through keyboard, pointer and typed routes. Before the last inline-dispatch guard, the broader Solid and option batch passed 90 tests and ignored two native-UI fixtures; these counts overlap the later runs and must not be added. UI tests use egui input simulation, not a human desktop acceptance. Public-source audit passed; artifact pruning retained cited runs. The no-OCCT build error from the prior checkpoint remains unresolved.

Final source Git blob IDs: `solid.rs` adf807d5bb8a19b514d07ef250ae91e2669291e0, `command_options.rs` 65e1f582ebf6272586d2d9901c32ded7dcf23025, `solid_input_regressions.rs` ee2c7ae86418df1fa4db3faf903edffa137ae472, `solid_difference_ui.rs` 5ef9ffc5f73f1ce848f2c3de0c396cd8b2254351, `solid_workflows.rs` 924726ea1a3f1c25c157c4f7d56f4b84461bceef, and `command_options_interface.rs` 8a37336443747eb866ba90e0e36efd53e3bd60f2. Other Boolean and Solid commands, non-BRep categories, external SDK, History, actual desktop interaction and human M03-UI remain open.

After explicit user approval, the repository and installed skill `SKILL.md` files matched byte-for-byte (SHA-256 `1bed280a76fc081cd96eefe9ad4be75c25625a2f94cabea3dd2b9700e7bb8209`), as did both `solid-prompt-contract.md` copies (`be1ec416f8211c1c0f56fac5c0accb8bac0114966aa1d406c3ba4859771ed344`). `python -m pytest -q tools/tests/test_skill_reference_index.py` passed 9 tests and 7 subtests. The broader `tools/run_reference_checks.py` did not pass: 4 of 12 checks passed, 8 failed. Inspected examples include unresolved M04 reference paths, an absent M00 milestone path, and M03 merge verification reporting package/planning differences; the complete drift has not been triaged and this is not a Solid runtime failure receipt. Public-source audit remained clean.

## Boolean Union selection and DeleteInput

Union now starts one BRep-selection stage from menu or CMD; preselected objects remain in that stage. At least two objects and Enter are required. `DeleteInput=No` retains the inputs; `Yes` deletes them, with `No` reset on each run. Undo removes the most recent pending pick, Cancel leaves the document unchanged, and successful fusion publishes one transaction. The shared command-option interface supplies keyboard, pointer and typed activation. Viewport picks use the same owner as CMD, including overlap chooser and command-run validation. Boolean Two Objects keeps its own two-object protocol. The historical OM9 host record supplies the No default; it is not evidence of native Matrix's default.

Windows, `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: `cargo test -p om9-app --lib solid_input_regressions` passed 22 tests, including five Union tests for menu/CMD, preselection, input retention/deletion, Undo/Cancel, invalid input, late failure/Redo, and a three-BRep compound with archive roundtrip. The integration batch `cargo test -p om9-app --test solid --test solid_workflows --test solid_difference_ui --test command_options_interface` passed 92 tests and ignored 2 native-UI fixtures. The Union option was exercised via keyboard, pointer and typed routes; viewport selection used egui events and controller Enter, not physical keyboard or real desktop acceptance. Public-source audit passed and pruning removed no runs while retaining 116 cited older runs. Git blob IDs: `solid.rs` e42dd1578064e21cda613a5e3fde563c15c5b700, `command_options.rs` db1e14c7f1980d43892c2232bc3863ce3390b30d, `ui.rs` aef003697f78d9f11b8fac572d076bf2abf6e125, `solid_input_regressions.rs` 4c6ff1d58dc5f938c6c837a2ae742834601b905e, `solid_difference_ui.rs` ff1016cc760624f57de70c1c7fe127aa0c3d9c6e, `solid.rs` integration test df5dd736397247fd0b493e7e70ad285daec6e40f, `solid_workflows.rs` f06a441f2a24b0c2a245f74c672baf0b240e24c7, and `command_options_interface.rs` 291284db4e89e83eee4bcd65afe599eb59098492.

The user explicitly approved recording the Union rule on 2026-10-28. Repository and installed skill files matched byte-for-byte: `SKILL.md` SHA-256 `d88c8d75435e08f6d899f352ddb96d7fb067c7a6f383edd9b9413fe248c017c3` and `solid-prompt-contract.md` SHA-256 `612f889c33add5aace4e0e5771ae40432bd007586b6c0286cad6833b56b0be64`. Skill-index pytest passed 9 tests and 7 subtests. This approval and the automated checks do not close actual desktop/human UI, other input categories, History, external SDK, the no-OCCT build or the other Solid commands.

## Boolean Intersection staged sets and DeleteInput

The standalone BRep slice now stages two ordered input sets via the shared menu, CMD and viewport selection owner. Each set is fused and the common volume is calculated before any transaction. `DeleteInput=No` retains both input sets; `Yes` deletes both. Empty common results and invalid input reject without document changes. Undo and Cancel during selection do not commit. The default comes from the historical OM9 host contract, not the recovered Matrix registration. The user approved recording this exact OM9 behavior in the feature-port skill on 2026-10-28. The bounded registration mapping is in `docs/validation/2026-10-28-solid-intersection-source-map.md`.

Windows with `CARGO_TARGET_DIR=L:/om9-target` and `-j 2`: the Solid input regression batch passed 26 tests, including a four-box 700 mm³ two-set intersection for both option values, transaction/Undo/Redo, empty result and late open-BRep failure preserving Redo. The integration batch passed 94 and ignored 2 native-UI fixtures. It covers menu/CMD, keyboard/pointer/typed option activation and egui-simulated viewport picks (Enter supplied through the controller), not real desktop or human acceptance. Public-source audit passed; artifact pruning removed none and retained 116 cited runs. Skill-index pytest passed 9 tests and 7 subtests; skill quick validation passed. Both repository and installed `SKILL.md` files match SHA-256 `0c1e246a19c55edb8dd4733dd9a0695e772b70eaecf6820068f001d4e6d2886a`; both `solid-prompt-contract.md` files match `331422c75a576f8baeaa3c6c20956cbdfbc803f092c284993f3fa3496c9e1192`.

Tested source Git blob IDs: `solid.rs` 51c27ce31474fc51bb358caed4eb1bc79aa863bb, `command_options.rs` 94b486e203922cb5cddad64da585950732ba785c, `solid_input_regressions.rs` ce1b99c0d49edb244430c3d2a7570e5b8a1f0640, `solid_difference_ui.rs` 34c14bcfc3dc71574ad69833da4b8ada6125eb9b, `solid_workflows.rs` eb22da59336e514328efaa9d83eaff949f1d7c4a and `command_options_interface.rs` 77eab5833c30d9c0adbd78088e9526d95ae937f7. Other input categories, History, external SDK, real desktop and human M03-UI acceptance remain open. The earlier no-OCCT build failure and the broader reference-check failures remain unclosed; this checkpoint does not complete the Solid menu audit.

## Boolean Two Objects viewport selection — partial

The previously preselection-only prompt now accepts BRep viewport picks through
the shared Boolean selection owner. It rejects a third newly picked object;
typed `Undo` removes the last pending selection without a transaction. The
existing U/D/I behavior remains immediate and deletes inputs. This is an
intermediate implementation: the local edit contract describes five result
states, preview cycling and `DeleteInput` with an OM9 No default; these are
not yet implemented in this standalone command. Preselection of more than two
is still rejected on execution and can be corrected with Undo. The bounded
registration mapping in `docs/validation/2026-10-28-solid-boolean-two-objects-source-map.md`
proves the native command association, not its solver or preview.

Windows `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: 27 Solid input regressions
passed; viewport and workflow integration passed 13, with one native-UI fixture
ignored. The viewport test uses egui pointer simulation and controller-submitted
operation, measuring a 500 mm³ difference and Undo, not a human desktop gate.
Public-source audit passed; pruning removed zero and retained 116 cited older
runs. Scoped `git diff --check` passed. The user approved recording the selection
and Undo rule; both repository and installed skill copies match. Skill-index
pytest passed 9 tests and 7 subtests and quick validation passed. Current Git
blobs: `solid.rs` 3e24a9fe4f1e6014fe2d71232d0f29cb5f564c5e,
`solid_input_regressions.rs` f3ec74269f311ca72eed0fd005e64a06c6124528,
`solid_difference_ui.rs` 39cf432efbcb34b5463e92bd059f97c9254540e5.

## Boolean Two Objects reverse difference and symmetric difference — partial

The standalone BRep command additionally computes B−A with `R` or
`ReverseDifference`, and the symmetric difference (A−B) union (B−A) with
`X`, `Xor` or `InversionIntersection`. The shared option interface exposes
all five existing operation actions. If one difference is empty, XOR retains
the other; if both are empty, it rejects without publishing. These results
still use the preexisting immediate-commit behavior and delete both inputs;
they are not the required five-state interactive preview or the historical
OM9 `DeleteInput=No` behavior. Input geometry beyond solid BRep remains open.

On Windows, `CARGO_TARGET_DIR=L:/om9-target` and `-j 2`: 28 Solid input
regressions passed, including asymmetric overlap, containment, disjoint solids,
identical solids, and Undo/Redo. The dedicated option test passed for keyboard,
pointer and typed routes for both new operations; all interaction tests are
simulated, not native desktop acceptance. The Solid integration batch passed
83 and ignored two native-UI fixtures. An initial run found an older negative
test used `X`; the test was corrected to use a genuinely invalid operation
and the batch was rerun. Public-source audit and scoped diff check passed.
Source blobs: `solid.rs` b904743c709b95735229841cd9bcf63a3efde81b;
`solid_input_regressions.rs` 5feae86b9d1119ba06cd98214b0d2f3dc964ed85;
`command_options_interface.rs` 01ab177ccf0dd6cb4a788f1f2d8c6c29fddf4310;
`solid_workflows.rs` 410f9ac5e9391b82c46aa198b0d488f71da7ae09.

## Boolean Two Objects BRep preview and confirmation — partial

This checkpoint supersedes the immediate-commit limitation in the two preceding
Boolean Two Objects checkpoints. Two selected BReps produce a temporary
world-space BRep preview. The five operations can be selected explicitly or
cycled with Next or a viewport click; Enter confirms one transaction.
`DeleteInput=No` is the historical OM9 host default and keeps both inputs;
`Yes` deletes both. Undo exits preview to selection, Cancel discards it.
An empty result remains a selectable preview state, can be cycled past, and
cannot be committed. Captured source stamps prevent cycling or committing a
stale preview. The preview is an egui wire overlay; source objects are not yet
temporarily hidden as the historical Edit contract specifies. Non-BRep input,
dialog parity, exhaustive topology, SDK/History and human desktop UI remain
open. This is not evidence of the external Rhino solver or native default.

Windows `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: final-source Solid input
regressions 32 passed. They measure all five results, both deletion choices,
Undo/Redo, stale inputs, cancellation and cycling past an empty intersection.
Four final-source focused integration tests passed: keyboard/pointer/typed
operation and DeleteInput routes, egui-simulated viewport cycling, and the
menu workflow. A broader integration batch before the final empty-preview
and selection guard edits passed 96 and ignored two native-UI fixtures; do not
count it as a final-source full-batch receipt. Public-source audit and scoped
diff check passed. Preview drawing and click tests are simulated, not a human
desktop acceptance. Source blobs: `solid.rs`
330313c2a1405a3f5eadba97e9aa540cad4349f0; `command_options.rs`
2035cd388b86eccaf1843348210aeaf71eba0692; `ui.rs`
a428dda22cd9a38f3b36a1e62b9c81d0ad6846ba;
`solid_input_regressions.rs` 6d4e12b5d5b3f1d24e72d1846fcb298cc52f2c75;
`solid_difference_ui.rs` 9c443e1bd0c0840e9b039a0633ecc8d7a9c7a748;
`command_options_interface.rs` bf9f2b44acec5a2eeb14ae9c5f061eecdec29a7e;
`solid_workflows.rs` e51efa06f5237c642dd7fbe73fe0af17c326f6e7.

The user approved recording the BRep preview/confirmation rule. Repository and
installed skill copies match: `SKILL.md` SHA-256
`d1de21f3b908db904e2091a46f0c198e56f212f18a7ea3d106cd6b9044fce6c3`,
`solid-prompt-contract.md` SHA-256
`9e6b04a90e5122a4ac440416065e61b0fadaa6937de4a1079355349c11a04028`.
Skill index passed 9 tests and 7 subtests; quick validation passed. Artifact
pruning removed no runs and retained 116 cited older runs.

## Boolean Two Objects: transient source visibility

The preview owner exposes the two captured source IDs to the GPU viewport as a
presentation-only mask, including for empty preview. GPU scene and overlay
preparation omit those roots; resource, overlay and publication keys include
the mask, so an old queued frame cannot be accepted as the new state. Selection
frame, gumball and hover overlays are suppressed while the mask is active.
Undo, Cancel and commit clear the mask by leaving preview; persistent object
and layer visibility are never changed by this presentation.

Windows `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: three real GPU viewport
publication tests passed, including a three-object scene with two masked and
restored, and cancellation while a masked frame is in flight. The scene and
document hidden attributes stayed unchanged. Solid input regressions: 32
passed; viewport/workflow integration: 13 passed, one native-UI fixture
ignored. These GPU tests exercise a wgpu render target, not a native desktop
screenshot or human acceptance. The wgpu test build preceded only the final
UI-only suppression of hover overlays; Solid tests include that final UI edit.
Public-source audit and scoped diff check passed. Source Git blobs:
`solid.rs` 17e9458210e800bf4a4a47e765231dec9ce0fcf5,
`gpu_viewport.rs` 659a203a00f39cb304b126130f7ebc4e8cd583c5,
`ui.rs` d99420006320b4db87901263c37850d85e25fff4,
`solid_input_regressions.rs` de53bbc0e1d1d57620325907921e1adf16382b7a,
`gpu_navigation_publication.rs` 86b0afe98e792f7461bc7fe399152240faed2445.
Other geometry types, actual desktop/human acceptance, dialog parity, History,
SDK and the other Solid commands remain open.

The user approved storing the transient-visibility rule. Repository and
installed skill copies match: `SKILL.md` SHA-256
`e0fd05dd2958637a1cf71f81b6216d9539ea5e2b1cbee626ada1101dd902766c`,
`solid-prompt-contract.md` SHA-256
`e35a10da2464cdb0b05e85b2aedc12586daeb9a68933059c9211d4e614a2b934`.
Skill-index pytest passed 9 tests and 7 subtests; quick validation passed.
Artifact pruning removed no runs and retained 116 cited older runs.

## Boolean Split and Create Solid: missing palette entries

The recovered registrations and M03 parameter records identify two separate
commands, but the standalone palette omitted both. They are now present after
Boolean Two Objects, visibly disabled with a reason; CMD rejects their names
without document mutation. They currently use the generic fallback symbol,
not a reviewed authored command-specific icon. The registration mapping is
in `docs/validation/2026-10-28-solid-split-create-source-map.md`. The two
operations, their options, exact input/output contracts and matching icons
remain unimplemented; existing sewing APIs are not treated as proof.

Windows `CARGO_TARGET_DIR=L:/om9-target` `-j 2`: one Solid registry test,
14 chrome integration tests and 9 workflow tests passed; one native-UI fixture
was ignored. The chrome tests use egui pointer simulation, not a real desktop
or human UI review. Public-source audit and scoped diff check passed; pruning
removed zero runs and retained 116 cited older runs. Source Git blobs:
`menu_catalog.rs` 0c29883154bfda095b8a69ae031aa7328243d65c;
`solid_menu.rs` a6962f5a12b21f5d4ff84cd10d1d87b6124e1664;
`chrome.rs` d4561c1923ae498aa6a5f445800b4b66395f5c3c;
`solid_workflows.rs` 5537fd35bb8dd0e2e7c35c5f5c0e9820bab4ebd6;
`chrome_actions.rs` cb857976474149cd56af072fb5269f7c9b2f1631.

## Solid split kernel foundation

An OCCT-scoped adapter now partitions one closed single-solid BRep by another
into a validated compound, and extracts individual closed positive-volume
solid pieces by compound-local index. Safe Rust checks input topology, returned
piece counts and conservation of target volume; C++ only owns the synchronous
OCCT API call and snapshot bridge. This capability is not a full Boolean Split
command and is not exposed as an OM9 SDK operation. Multi-set and same-object
selection, separate document outputs, DeleteInput, undo, UI, open-surface
inputs and Create Solid remain open.

Windows `CARGO_TARGET_DIR=L:/om9-target` `-j 2`: 13 kernel editing tests
passed on the changed bridge, including overlap volumes 500/500 mm³, contained
volumes 64/936 mm³, disjoint and identical cases 1000 mm³, invalid face input,
and out-of-range piece index. Each extracted result was measured as one closed
solid and original input stayed unchanged. The application Solid workflow batch
passed 9 with one native-UI fixture ignored after the same native change.
Public Rhino 5 command help and its limits are linked in the source mapping;
it does not prove the decompiled Matrix wrapper's full semantics.

The matching SDK/package regression batches passed: SDK 12 tests plus one
compile-fail doctest, plugin host 55 unit, 3 bundle and 12 package tests.
These exercise the existing SDK boundary, not a new split SDK API. The public
source audit and scoped diff check passed; pruning removed zero runs and
retained 116 cited older runs. Source blobs: `editing.rs`
9a69d1022e397640f855c24902067ddb19c21154, `editing_bridge.cpp`
5ec8be9278c53aeaf8f43a1ec7c5c299bcdd63e9, and `l2_01_10.rs`
c5e31b824af326b8c267a413c2744cf7699c16fd.

## Solid split multiple-set kernel foundation

Safe Rust now partitions each closed single-solid target by the other targets
and cutters using the previously tested OCCT pair splitter. Equivalent cells
are detected by two directional differences and emitted once; the same BRep
may occur in both sets. Output count and retained output usage are limited.
This is a kernel helper and does not yet implement the command, selection,
document identity, DeleteInput or transaction. Surface-only and mixed input,
general topology, large-input performance and complete peak-memory accounting
of native intermediate shapes remain unverified.

Windows `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: the final-source kernel
editing batch passed 15 tests. Overlapping target boxes also used as cutters
produce three unique 500 mm³ solids in either selection order; repeated inputs
do not duplicate a cell. Two independent cutters produce five 200 mm³ solids in
either order. Invalid face input, empty target set and an insufficient output
limit reject. An application workflow batch before the final output-budget
check passed 9 tests and ignored one native-UI fixture. Public-source audit
and scoped diff check passed; pruning removed zero runs and retained 116 cited
older runs. Source blobs: `editing.rs`
716b30418ad1567a78ddf5b97119833c68555db7 and `l2_01_10.rs`
fc5ca06fa11ff1d51f8ab4df0ec12afed2ad635e. No SDK or UI capability is claimed.

## Boolean Split BRep command integration — partial

The previous disabled-only palette checkpoint is superseded for closed
single-solid BRep inputs. The original recovered key `SolidBooleanSplit`
starts the same session from menu or `BooleanSplit` CMD, without inventing
an L3 identifier. It collects targets then cutters, allows the same object in
both sets, partitions and deduplicates pieces, and inserts each piece as a
separate BRep in one transaction. `DeleteInput=No` retains both sets; Yes
deletes each unique input once. No is the existing OM9 Boolean host choice,
not a recovered Matrix default. Pending Undo/Cancel do not publish; stale
targets and invalid open cutters reject atomically. The user approved recording
this BRep rule. F6 registration, surface-only BReps, compounds, mesh, general
topology, complete native intermediate peak-memory accounting, performance,
SDK and real desktop/human UI remain open; Create Solid remains disabled.

The new authored purple-family SVG is generated from `tools/solid_icons.py`
and bound to the original key. The exporter now gives the Solid SVG catalog
priority over legacy art; generator output and manifest hash match. The new
symbol was inspected alongside the authored Boolean Two Objects and Intersection
icons at 24px on dark gray; this is not an original-icon comparison or human
recognition signoff.

Windows `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: Solid input regressions
35 passed, including same-object sets, No/Yes, distinct 500 mm³ pieces at
world x=0/5/10, one transaction, Undo/Redo, stale/open failure and pending
Undo/Cancel. Integration: chrome 15, simulated viewport 6 and workflows 9
passed, one native-UI fixture ignored. The DeleteInput option passed its
dedicated keyboard/pointer/typed test. Icon/export pytest passed 18; skill
index passed 9 and 7 subtests, and quick validation passed. Public-source
audit and scoped diff check passed; pruning removed zero and retained 116 cited
runs. These UI tests are egui simulations, not desktop acceptance.

Source Git blobs: `session.rs` beeaa060804477f242f75e5fd733dc3a22a79d69,
`solid.rs` 85f1e9a097ca8e08b0244452bec886cc64a284df,
`command_options.rs` f80b3f70e117cf3daeeb7fec43413a8c8d9a8321,
`chrome.rs` 16b69265cf77f0f8ba2ca53374215d7f00974252,
`icons.rs` b2d16096fc09bdb6ad860b2926b70df9735478e6,
`solid_input_regressions.rs` 121b4bcd93a082e4796fc92b4f8a453a0ef20eb2,
`solid_difference_ui.rs` 0ac5139b269867b801186a84d0613c64c06fdf53,
`chrome_actions.rs` 08bc8dd253b7218bd68689c81a5edfedbd96bb1f,
`command_options_interface.rs` e3b87fddae70e7a11de4c0f7254f894042117b92,
`solid_icons.py` d90e90292344107089ef1a662828244081716e26,
`export_menu_assets.py` 21a96b1ab971d60aa616b98d31ee149e64c77bb9,
and the SVG 123d9f6234ba04c76fe6a4d6860d40fce547b24a. Repository and installed
skill `SKILL.md` files match SHA-256
`93973ff2b92e3e8cd175d0c60e9ce54fe7babbd598400347edc5c9fe72810266`;
their prompt-contract files match
`5eb2a9bfbbbb82e50b8afb8d043f9b291f7bd50b3138a1101644e44bb206308d`.

## Create Solid intersecting BRep regions — partial

The earlier disabled-only checkpoint is superseded for BRep inputs. The
original `SolidCreateSolid` key now starts one selection stage via menu or
`CreateSolid` CMD; Enter constructs bounded regions and publishes separate
solid BReps in one transaction. DeleteInput No retains inputs, Yes deletes
them. No is an OM9 host choice, not a proved Matrix default. No bounded region
is a visible refusal with no mutation; pending Undo/Cancel do not publish.
Safe Rust validates source faces, retained inputs, result count and closed
positive-volume output. The selected C++ adapter only calls OCCT 8.0.1
`BOPAlgo_MakerVolume` and returns snapshots; native exceptions are caught
at the ABI and no document ownership crosses it.

Windows `CARGO_TARGET_DIR=L:/om9-target` `-j 2`: kernel editing 16 passed,
including three regions from 12 intersecting faces, empty open-face outcome and
output-limit refusal. Solid input regressions 37 passed: both DeleteInput
values, world-positioned 500 mm³ results, one transaction, Undo/Redo, open
input failure preserving Redo, pending Undo/Cancel. Chrome 15, simulated
viewport 7, workflows 9 and dedicated three-route option test 1 passed;
one native-UI fixture ignored. SDK and plugin-host test suites passed, but
the new geometry operation is not exposed through SDK. Python icon/export
18 passed; the 32-entry generator catalog, new SVG bytes and manifest hash
matched. New SVG was inspected at 24px beside authored Solid icons, not against
original imagery or by a human reviewer. Public-source audit and scoped diff
check passed; pruning removed no runs while retaining 116 cited older runs.
These tests do not prove exhaustive geometry, peak native intermediate memory,
large-model performance, mesh/other categories, History, F6, actual desktop,
or human UI acceptance.

Source Git blobs: `editing.rs` 7f95d2839cb916fe31833f26c161b963ddc7521c,
`editing_bridge.cpp` df2f8eaaf1f4be9b2a6e619e8cdf3717f580f841,
`l2_01_10.rs` 881b260a54e8fbd9be5fdf13b32719a6219e440e,
`session.rs` c3d36246c2f60bc23e923c39d30ae41b9d1000bb,
`solid.rs` 892c3ceceb9a37a342e426e53b4a6be56f481d22,
`command_options.rs` 44930e431d73d779dec7a3dcff4c1590e633e2dc,
`chrome.rs` 7bbe03d0b9c4f44e8eaaf87f9b32b6d634b45bac,
`icons.rs` f19339b2a4fb377682afe04554531cabb9a7fe2c,
`solid_input_regressions.rs` 99fe5167f5d6fa476238453192ccd8fc606d8285,
`solid_difference_ui.rs` 0f2835371751be5dc09724c511133ebca3561d6b,
`chrome_actions.rs` 01aebd8d054b6898fcb47264b880f657c515d07e,
`command_options_interface.rs` 55f8f574002b69712f0e1532da6def052869f0e0,
`solid_icons.py` b264adcea2ded5442021981582f91171e4f87d36,
`test_solid_icons.py` 1bbb8553cf10811b8ad951418874ed407df90126, and
`SolidCreateSolid.svg` 9d11c08787acab1738ebf4073082d636a7efc5ed.

## Cap Planar multi-input selection — partial

Previously Cap immediately processed exactly one preselected BRep. Menu and
CMD now stage one or more BReps, accept viewport selection, pending Undo and
Cancel, then prepare all results before a single atomic geometry transaction.
Identity and placement are preserved; a later input error leaves the document
and existing Redo intact, and the pending selection can be corrected. The
bounded registration mapping is in `2026-10-28-solid-cap-source-map.md`;
registration alone does not establish the external solver or defaults.

Windows `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: Solid input regressions 39,
chrome 15, simulated viewport 8, Solid 70 and workflows 9 passed; one Solid and
one workflow native-UI fixture ignored. The two Cap regressions were rerun
after adding a Cancel/Redo assertion and passed. The viewport test exercises
an open box shell; the two-shell tests cover 1000 mm³ closed results, world
placement, atomic failure, Undo and Redo. These checks do not prove actual
desktop or human UI acceptance. Public-source audit passed (14644 source
files, 583 icon bindings, 582 SVGs, zero errors).

The existing native adapter supports only one naked edge chain per BRep and a
closed-solid result. Multiple loops, arbitrary curved boundaries, the fixed
`1e-7` tolerance, native intermediate peak memory, other input categories,
F6, SDK/History and M03 human UI acceptance remain open. No native solver was
changed in this checkpoint. Git blobs: `solid.rs`
0b9bc48fa9d1b84fc937afe7ee50409514c85de9,
`solid_input_regressions.rs` e553c613939968be324276878608471264b9b563,
`solid_workflows.rs` 397cb9f27dbd84eebe6bd31e1aac33e4c13fa1b1,
`solid_difference_ui.rs` cda2119bd3bd2ee989d83f74737b6df9537c603c.

## Cap Planar independent loops and curved edge — partial

The later adapter partitions naked edges into closed wires and uses OCCT
`BRepBuilderAPI_FindPlane` on each complete wire rather than inferring
planarity only from vertices. It constructs non-overlapping cap faces, fixes
shell orientation and publishes a validated closed solid through the existing
bounded snapshot ABI. OCCT is necessary for topology, curve planarity, face
intersection and BRep construction; Rust retains validation, budgets,
document ownership and transaction semantics. Nested coplanar regions are
explicitly rejected until a face with holes can be built. The pairwise native
intersection work and intermediate peak memory are not bounded by the output
snapshot budget.

Red-before-fix: the two-loop box failed with “naked edges do not form one
wire”. Windows `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: kernel L2-01-10
18 passed, including two open box ends (1000 mm³), a cylinder with a circular
boundary (250π mm³), nested coplanar ring refusal and the earlier nonplanar
negative. App Cap regressions 3 passed, including two loops through CMD and
Undo; simulated viewport 8 and workflows 9 passed with one native fixture
ignored. This does not establish general curved or nested topology, partial
caps, large-input performance, actual desktop interaction or M03 human UI.
The app still supplies fixed `1e-7` tolerance; this is not a recovered Matrix
default. Git blobs: `editing_bridge.cpp`
526cd3e5384f29c0919ac0d6e726305f20582f53, `editing.rs`
6a13643d73dac2a37b7143c96e4355d0e844a83b, `l2_01_10.rs`
c4e66d3f081f507b81b4196ce60e8236659e7b08,
`solid_input_regressions.rs` 6bec1b8970bc509469436793bd9ed3bcd191e292.

## Cap Planar nested coplanar pair — partial

The earlier nested-loop refusal is superseded for the tested annular case.
The OCCT adapter compares face intersections, determines containment and
constructs a face with immediate inner wires; crossing or coincident regions
are rejected. The resulting solid remains under the existing Rust validation,
snapshot budget and atomic command transaction. The area-based containment
tolerance and quadratic native intersection cost need further validation;
native intermediate allocations are not bounded by the output budget.

Red-before-fix: an open hollow cylinder was rejected as overlapping boundary
regions. Windows `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: kernel L2-01-10
18 passed, including an annular cap preserving volume 210π mm³ and no naked
edges; app Cap 4 passed, including CMD, one transaction and Undo for the
annular cap. Simulated viewport 8 and workflows 9 passed, one native UI
fixture ignored. An attempted fixture removing both tube end faces could not
produce a valid input BRep through the current remove-face API; it is not
evidence for that case. Multiple holes, deeper nesting, intersecting loops,
arbitrary curved topology, partial caps, actual desktop/human UI and resource
limits remain unverified. The fixed app tolerance is not a proven Matrix
default. Git blobs: `editing_bridge.cpp`
d96822d736eb647f5526d3f8315744371835a0c2, `editing.rs`
051dadb918c7218edf4891b342bf36b1b185b104, `l2_01_10.rs`
6a976c42b658985e5abaa9bde2bd987bf56aa957,
`solid_input_regressions.rs` b2cece1e790a257389b589c0653cc152bab2d09a.

## Cap Planar mixed boundaries and two holes — partial

Previously one nonplanar loop rejected the whole input. The adapter now skips
nonplanar closed loops, builds the planar caps, validates and emits an open
shell when nonplanar boundaries remain, or a solid when all boundaries close.
It rejects an input with no planar loop. Rust validates that a face was added
and the number of naked edges decreased before the existing atomic document
transaction publishes the result. The OCCT bridge still owns neither document
nor history; exception handling and bounded snapshot output remain unchanged.

Red-before-fix: a stepped shell with a planar bottom opening and a nonplanar
upper loop returned “naked loop is not planar”. Windows
`CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: kernel L2-01-10 20 passed,
including the mixed shell, a planar face with two holes and 920 mm³ closed
volume, and the all-nonplanar refusal. App Cap 5 passed, including mixed-shell
CMD, one transaction and Undo. Simulated viewport 8 and workflows 9 passed;
one native UI fixture ignored. General topology, arbitrary curved edges,
large-input performance, native peak allocations, area-based containment,
fixed application tolerance, F6, SDK/History and actual desktop/human UI
acceptance remain unverified. Git blobs: `editing_bridge.cpp`
e5e18679c4a7cac61fac5592f08e0f0c835ddff3, `editing.rs`
4247d8cc143b73e22f979527fb80fcbcd67634af, `l2_01_10.rs`
76955487344ed7b32ecf8a11d04558ba3b5df292,
`solid_input_regressions.rs` eda7bcccd5c9bfbe0312f104c806e43547aaff36.

## Extract Surface numeric face path — partial

The previous numeric path always copied the face and left the source solid.
The tested default now extracts one face and replaces the source BRep with
the remaining open shell, in a single transaction. `Copy=Yes` keeps the
source. Output inherits source placement and uses the current layer; its
default layer is not inferred from Matrix registration. Native extraction and
remove-face use existing bounded OCCT APIs; Rust owns the document mutation.
Source mapping and limits are in `2026-10-28-solid-extract-source-map.md`.

Red-before-fix: the source stayed closed after extraction. Windows
`CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: Extract regressions 3 passed,
including default separation, Copy=Yes, placement, Undo/Redo and invalid
index preserving Redo. Solid suite 70 passed and one ignored; workflows 9
passed and one native UI fixture ignored. The workflow now measures the
five-face open remainder and the 100 mm² extracted face, rather than
assuming the original remains closed. Tests cover a preselected object and
numeric face index, not viewport face selection, multiple faces, interactive
options or the right-click alternate. A one-face source without copy remains
unsupported; F6, SDK/History and actual desktop/human UI remain open.
Git blobs: `solid.rs` 76f1959d0aee9517bc3bcc67226d4d28d11e8582,
`solid_input_regressions.rs` 72dc4b0d69327428100d62d46a71ba2e07c216b6,
`tests/solid.rs` 1bff96b227edb8891524395f91a5d0338ca4b9e2,
`solid_workflows.rs` b1e9fb0fea4f16ab75afe5aa1d4e62e0459d9c24.

## Extract Surface options and alternate icon — partial

The numeric session now exposes Copy and OutputLayer Current/Input through
the shared command option handler; the result layer is tested against distinct
current and source layers. The original icon's alternate starts Copy=Yes.
Current is an OM9 initial choice and is not proved to be Matrix's default.
An egui right-click on an empty document starts the variant, but an expanded
fixture with a preselected box did not start the command; this remains an
explicit ignored regression, not a passed UI gate.

Windows `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: Solid input regressions
46 passed; chrome 16 passed and one known preselection issue ignored; Copy and
OutputLayer option activation passed via keyboard, pointer and typed routes;
workflows 9 passed with one native UI fixture ignored. Viewport face picking,
multi-face extraction, single-face non-copy, actual desktop/human UI and
F6/SDK/History remain open. Git blobs: `session/solid.rs`
9b58dcc826d870ed99da57e222c3f045aa6e2f84, `session.rs`
27c9e2565e06a8d8282cae58175e59960f1d57f1, `command_options.rs`
137dda2321a12f636f41f692f08d1f1d9f0418c4, `chrome.rs`
e67f01ef0451d95536af812ee7167bf1744286e9,
`solid_input_regressions.rs` 770fda53a7ca489aaad878d564fcf70d86a67a2b,
`chrome_actions.rs` ec5f8581edb9e198a3fabb32672f30806f90287e,
`command_options_interface.rs` 6557c4053e52099ba4b952848278e218400e78e0.

## Extract Surface viewport face and right-click fixture correction — partial

The prior preselection egui failure was a fixture index error: creating a box
adds a recent-command icon before the two history buttons, so the fixed offset
clicked the wrong icon. Counting recent icons makes the regression pass; with
a preselected box right-click opens Copy=Yes, creates a face and keeps the
source in one transaction.

The viewport now picks one root BRep face and Enter invokes the same extract
owner. A source/scene/selection recheck rejects stale or changed selection;
Undo clears the pending face, and Cancel clears it without a transaction.
The exact clicked top face, five-face remainder and Undo restoration are
verified. A second click currently replaces the one face; instances and
ambiguous overlapping picks remain unhandled, as does multi-face extraction.
The numeric-index and option paths remain available.

Windows `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: simulated viewport 10,
chrome 17, Solid input regressions 46 and the three-route shared option test
passed; workflows 9 passed with one native UI fixture ignored. Following the
final source/selection guard, viewport 10 and workflows 9 were rerun. Actual
desktop/human UI, F6/SDK/History and general selection categories remain open.
Git blobs: `session/solid.rs` 65e9800b431d2357653090bce04b377417cf8071,
`session.rs` dddc7f4c968be766e6619d8cac371b6785d8964c, `ui.rs`
35618e3c453c6345b67b18cb984451fea492c053, `chrome_actions.rs`
447cf62aba5ef457e1d56e29d2fab0c25089bd18,
`solid_difference_ui.rs` 40ee9595dbbf916dda97b8f1c4b70cd829a99357.

## Extract Surface multi-face selection — partial

The former one-face selection is superseded: root BRep face picks are grouped
per source and Enter prepares all individual face outputs and source
remainders before one transaction. Undo during selection removes the last
face; Copy=Yes leaves sources unchanged. Copy=No removes each selected face
from its original source; selecting every face deletes that source once.
Output faces preserve source placement and use the selected OutputLayer.
Source stamps and selection are rechecked before preparation; duplicate
face picks are ignored.

Rust validates and budgets source indices and owns document/transaction state;
the limited OCCT adapter builds the remainder using indices in original
topology and rejects invalid resulting shells. Native intermediate allocation
peaks and general disconnected remainders are not proved. Windows
`CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: kernel L2-01-10 21 passed;
app Extract 6, viewport 11, chrome 17 and workflows 9 passed, with one
native UI fixture ignored. Three selected faces from two translated boxes,
both Copy states, output positions, Undo/Redo, all-face replacement and
pending Undo were checked. Instance faces, ambiguous overlapping picks,
general topology, F6/SDK/History and actual desktop/human UI remain open.
Git blobs: `editing.rs` 1d12f294da1b26c28acc4a561e830cf3b6cf1bd0,
`editing_bridge.cpp` 2031edc04744ed50d70622b798180b895ccc14f7,
`l2_01_10.rs` bd7503b05b621b564effaac64ca16af5fa4f7084,
`session/solid.rs` e123e7e115ca6b3c5e9643f44e9ead7494605f91,
`solid_input_regressions.rs` 7067c925ad944129b8717a45ab258ea465f31328,
`solid_difference_ui.rs` 97517ea43e0dbe5a17ffe0b3af96d1b2f08bd765.

The user approved recording the multi-face Extract Surface rule on 2026-10-28.
Repository and installed `openmatrix9-feature-port` copies were synchronized:
`SKILL.md` SHA-256
`c6811c9a95fe6f6514c69cd811da457b96444ed6a4e1411cf539adfbc21a8e6b`,
`references/solid-prompt-contract.md` SHA-256
`5e53c99752c6d4e05fbdea2e517b8bd96c7b7a30824e12164a5d1ffc9dfb690b`.
Skill reference index tests:
9 passed, 7 subtests; quick validation passed. This approval covers the
multi-face rule only and does not sign the M03 human UI gate.

## Extract Surface overlapping root-face chooser — partial

Previously picking chose the nearest hit silently. The new scene query returns
the nearest face per distinct root, and Extract opens the existing ambiguity
chooser for multiple candidates. Rows include source label, face index and
short object ID. The selected hit is revalidated against the scene epoch,
source stamp, view camera and viewport, and against the command run before
being added to the staged selection. An Extract command started from CMD now
advances its run identity so a popup from a cancelled run is not reused.

Red-before-fix: two overlapping boxes produced no choice. Windows
`CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: viewport 12, ordinary selection
sequence 2, disambiguation 5, chrome 17, Extract regressions 6 and a shared
option route test passed. The two-box test exercises selecting one specific
source, source preservation, Undo and rejection after command restart or
camera change; chooser selection was invoked through its controller-facing
test method, not a native mouse click on a popup row. Multiple faces within a
single root, instance paths, actual desktop/human UI and native resource
limits remain open. Git blobs: `scene.rs`
6d641f3b44f2e70f704ea5bfd7bd2aa47ed306a7, `ui.rs`
8b0efcde9484f4119b0e297c9ec4970e8e9f1a93, `session/solid.rs`
71c002269f823d04d6be75040e3658bdfda55e1f, `solid_difference_ui.rs`
ff37ac5c2ce74c53508098dc3ce798db3e2d1698.

## Extract Surface Ctrl removal; Fillet Edge source comparison

The normalized Extract Surface reference describes Ctrl-click removing a
selected surface. The previous multi-face session ignored Ctrl on an already
selected face. It now removes that face, without a document transaction,
including when the source is chosen through the overlap popup. The pending
selection order remains available for Undo; an unselected face is refused.
Windows `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: viewport 12, selection
sequence 2, general disambiguation 5 and Extract regressions 6 passed; three
viewport Extract tests were rerun after adding the stale-selection guard. The
overlap test uses the controller-facing chooser method; native popup clicks
and actual desktop/human UI remain unverified. Git blobs: `ui.rs`
51b3c6666d09e3e885980a2f23363e68191b09ab, `session/solid.rs`
d512db94bfc3b6b2ac6037a7e214f24da23f389c and `solid_difference_ui.rs`
d4a514ee8cb205c1097e2e4cce4802c8ae7a4e1f.

The next command audit is Fillet Edge. Its bounded source registration and
the specific difference between the normalized reference and the current
numeric solid-only path are documented in
`2026-10-28-solid-fillet-source-map.md`; no full-command acceptance is
claimed from the old numeric test.

## Fillet Edge staged constant-radius preview — partial

Menu/CMD may now set a positive `NextRadius`, select one or more edges of
one root BRep in the viewport, press Enter for a BRep preview, then Enter to
commit. The preview is a green line overlay transformed by the source
placement; the original remains visible. Undo returns from preview or removes
the last selected edge, and Cancel leaves the document unchanged. Source
stamps and selection are checked; overlapping root edge hits are refused.
The old direct numeric two-argument path still commits immediately. The app
does not assert a recovered default radius.

Windows `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: Solid input regressions
49, viewport 13, workflow 9 and the NextRadius keyboard/pointer draft test
passed; one native workflow fixture ignored. The tests measure a two-edge
closed result, placement-aware preview, no transaction until commit,
Undo/Cancel and a rejected radius change after selection. Constant-radius
OCCT remains solid-only. Per-edge radii, handle editing, chain/previous-edge,
open joined shells, ambiguity chooser, instances, actual desktop/human UI,
GPU presentation and native peak resource use remain open. Git blobs:
`session/solid.rs` d4ac5cab8b2c5ca3c4b8620357185b971e027283,
`session.rs` 1c5aaf72b0bb1ea3ab75e7c9266f12f280f208c1,
`command_options.rs` b7fb30918af8b8e98b1d36537b6be04af59b4d3e,
`ui.rs` 660c515e38d955e007c5f8670e7ef3d7d815f580,
`solid_input_regressions.rs` 50c8199048df81a29eaa0010005acd3e6429e4ef,
`solid_difference_ui.rs` 46e5e19352b4981a4233752ca5b6fad83b09122a,
`command_options_interface.rs` 1219735308abb1ae48b7ff8314f87b0a1a1e5636.

## Fillet Edge per-edge NextRadius — partial

The earlier restriction to one shared radius is superseded for independent
edges. Each pick records the current positive NextRadius; changing it affects
later picks, not previous ones. Rust validates indices/radii and retains
selection/preview; the narrow OCCT adapter builds one fillet with per-edge
constant radii and a bounded snapshot. The adapter refuses different values
inside one automatically propagated tangent contour; a dedicated tangent
fixture is still needed to validate that branch. Native intermediate
allocation peaks remain unmeasured.

Windows `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: kernel L2-01-10 22 and
Solid input regressions 50 passed. The new invalid-preview test separately
passed after the batch, confirming failure retains document and Redo. The
simulated viewport picked two translated-box edges at radii 1 and 2 mm and
measured the exact expected closed volume; session tests separately check
preview, commit and Undo. A first viewport attempt placed the second edge
outside the frame; after correcting the fixture it passed. Per-contour
variable-radius handles, open shells, ambiguity choice, instances, F6,
SDK/History and actual desktop/human UI remain open. Git blobs: `editing.rs`
d146495b917f595e94c18d213a6301f9fd8ab370, `editing_bridge.cpp`
6a63e47002312aeed6cacba1d6d4a509d8d9c306 (header-only follow-up
retested), `l2_01_10.rs`
30d9242148623dcafbabbe0dfe5dcacbf7625055, `session/solid.rs`
802eb3e930472daa3fef6051afb24063c939ee31, `solid_input_regressions.rs`
c3c8a01c467ad2acfd89277e23f8d04bbff27282, `solid_difference_ui.rs`
3cb0526c1dd831eed59fecfe092ef892dde1ce34.

## Fillet Edge joined open shell — partial

The prior solid-only restriction is superseded for the per-edge-radius
adapter and app paths. The adapter admits valid BReps with at least two
faces and rejects selected naked edges; OCCT builds the fillet and validates
the result. Rust retains document, selection, transaction and bounded
snapshot ownership. The older public uniform-radius kernel `fillet` API
still rejects open shells; both app staged and numeric paths now call the
per-edge adapter.

Red-before-fix: an open box shell's shared vertical edge was rejected as
requiring a solid. Windows `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`:
kernel L2-01-10 23, simulated viewport 13 and workflows 9 passed, with one
native UI fixture ignored; the dedicated app test passed both staged and
numeric paths with Undo. The open result remains a shell, has an added face
and matches the analytic surface area 500 − 20 + 5π − (1 − π/4) mm²; a naked
top edge is rejected. The test chooses the staged edge through controller
subobject identity, not an open-shell viewport click. General open-shell and
non-manifold topology, editable handles, per-contour variable radii, native
peak allocations, actual desktop/human UI and SDK exposure remain open.
Git blobs: `editing.rs` cf76d4b4b3019c15fa90b3255f283821f77ae0c2,
`editing_bridge.cpp` a0425be7d2f103442f3258fb274ca0ddca70297a,
`l2_01_10.rs` 8acb046c8a9e721a80478d58da0155a8bf45767e,
`session/solid.rs` d0177c2a2b068d7982d510f4e55dd49ab8e00fe9,
`solid_input_regressions.rs` b67581d71e8633ffc127db3a7e85fc923f0addf2.

## Fillet Edge sampled handle and CurrentRadius — partial

The preview now draws one selectable handle per picked edge at the midpoint of
a sampled source-edge display segment, transformed by the source placement.
Selecting it exposes `CurrentRadius`; a positive edit recomputes the
constant-radius edge preview. Source and selection are revalidated. If the
recompute or resource check fails, the old radius and preview remain. Enter
commits and Cancel drops the preview. The existing OCCT per-edge operation is
unchanged; Rust owns handle state, checks and atomic publication.

Windows `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: viewport 13, Solid input
regressions 52, workflows 9 and NextRadius keyboard/pointer draft passed; one
native workflow fixture ignored. The simulated viewport clicks a handle,
changes 1 to 1.5 mm, measures preview and committed BRep volume, and verifies
an excessive radius preserves the prior preview. The shared option interface
exposes a shortcut and draft for CurrentRadius. No actual desktop, native
pointer drag, GPU presentation, variable radius along an edge, handle
insertion/removal, general topology or M03 human UI acceptance is proved.
Sampling and handle placement at arbitrary curved edges, and per-frame
performance on large BReps, remain unverified. Git blobs: `session/solid.rs`
64daac5823a3211fc67e10484d3cea72e6038282, `command_options.rs`
be0fe508f5490dd4693d3de7f4ece575ba5a4b83, `ui.rs`
27f2a8e3253afd07a4194d5a6714ddf1ac222e4b, `solid_difference_ui.rs`
454c50ff5d0ae8bd7205e55ff9a3ddc4ab084d32.

## Fillet Edge oriented endpoints and linear radius — partial

For edges with distinct endpoints the later checkpoint replaces the single
sampled handle with two exact topological endpoint handles. Each pick captures
the selected endpoint's radius, and CurrentRadius recomputes a linear law
between the two; the old preview survives a failed recompute. A closed edge
without distinct vertices retains the prior sampled constant-radius handle.
Rust owns selection, state, validation, resource checks and transaction. The
limited OCCT bridge reads the oriented endpoints and constructs a linear
fillet on one-edge contours; variable values on multi-edge tangent contours
are refused. Native intermediate peak memory remains unmeasured.

Windows `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`: kernel L2-01-10 24,
Solid input regressions 52, viewport 13, workflows 9 and the NextRadius option
test passed; one native UI fixture ignored. A dedicated kernel test compares
the two endpoint-radius orders with constant-radius results and cuts near both
ends; it confirms the changed end corresponds to the oriented vertex. The
viewport test selects and edits both endpoint handles, checks the resulting
BRep against the kernel output, checks failure retains the old preview and
commits a changed endpoint. The initially assumed quarter-circle volume
formula did not match the varying-radius BRep and was replaced by slice and
constant-radius comparisons. Only the tested straight independent edge has
this evidence; general curves, contour chains, handle dragging/interior
handles, actual desktop/human UI and performance remain open. Git blobs:
`editing.rs` ea7f45dae5cd76bb016e3b4b11953c6b55b31529,
`editing_bridge.cpp` c9ebdf043cf4171cfc468c820bd166df3111d845,
`l2_01_10.rs` 1e105a56d8e71bf2700c8430a59e167ad564ccc5,
`session/solid.rs` 6880897af8c640294fb23f8cdeafa78431324bd5,
`solid_difference_ui.rs` c7f0ab19e82b527678d150adf30289ebf60a879b.

### Fillet endpoint classification follow-up

The endpoint bridge now distinguishes a closed edge with one topological vertex
from missing vertices and other errors. The session propagates errors before
changing selection instead of silently treating them as closed edges. A cylinder
fixture confirmed the closed-edge classification and a valid constant-radius
fillet; an out-of-range index returned an error. The app regression confirmed
one sampled handle, commit and Undo on that cylinder. Two existing staged
Fillet regressions and the simulated viewport endpoint-edit regression passed.
These checks do not validate moving the closed handle, arbitrary closed curves,
native UI, or the remaining Fillet options. The sampled point is not evidence
that the original command's movable closed-edge handle has been reproduced.

### Fillet ShowRadius (2026-10-28)

The documented ShowRadius option now controls radius text at preview handles;
the handle itself remains drawn and selectable. OM9 starts at Yes to preserve
its previous display, not as a claim about the original default. Selection and
preview stages use the shared option interface. Keyboard, pointer and typed
option activation passed simulated UI checks; a cylinder preview test checked
Yes/No, invalid input, unchanged document, commit and Undo. The existing
simulated viewport fillet test also passed. No desktop screenshot/pixel check
or original default verification was performed. The skill update is requested,
not yet approved.

### Fillet SetAll (2026-10-28)

The preview exposes `SetAll=` through the shared option interface. A positive
radius updates both endpoint values on every selected edge and rebuilds the
preview; failed validation or geometry rebuild restores the prior values and
preview. Two independent box edges were checked against an analytical volume;
an excessive radius preserved the preview. A second test committed the new
constant radii in one transaction and undid it. The simulated viewport test,
53 Solid input regressions and two existing Fillet option route tests passed;
the changed commit regression was rerun after its last edit. Only option
activation through the shared descriptor was checked for SetAll itself, not
the actual pointer/keyboard routes at that checkpoint or desktop. Added/interior handles and
multi-edge variable tangent contours remain unsupported. Skill update requested.

### Fillet LinkHandles and SetAll routes (2026-10-28)

In preview, `LinkHandles=Yes` makes a selected handle's CurrentRadius update
all existing handles through the same atomic rebuild as SetAll; `No` edits
only the selected endpoint. OM9 starts at No to preserve its prior behavior;
the original default is unknown. A two-edge box fixture checked linked volume,
independent editing after unlink, failed rebuild rollback, commit and Undo.
Simulated keyboard, pointer and typed routes were checked for LinkHandles and
SetAll; actual desktop rendering/input was not. An initial pointer retest used
a stale token rectangle after the label changed; refreshing the simulated UI
frame before the next click passed. Three Fillet option tests, 53 Solid input
regressions and the simulated viewport Fillet test passed. The skill update is
requested, not yet approved. Added handles, dragging, multi-edge tangent laws
and the rest of the documented command remain open.

### Fillet SelectEdges (2026-10-28)

SelectEdges from preview returns to edge picking without losing existing edge
selection or edited endpoint radii; NextRadius applies to subsequent picks.
The old preview is discarded, and Enter rebuilds before a separate commit.
Source stamp and selection are checked before leaving preview. Simulated
keyboard, token and typed option routes, two independent edges with retained
edits, geometric preview, commit and Undo passed. A simulated viewport test
picked the additional edge and verified the resulting BRep; 53 Solid input
regressions and four Fillet option tests passed. These are not actual desktop
acceptance; arbitrary topology, instance edges and other Fillet options remain
open. Skill update requested and not yet approved.

### Fillet variable radii on a split tangent contour (2026-10-28)

OCCT supplies relative contour abscissae and a radius law adapter. For a
nonuniform open contour the bridge now requires all contour edges and matching
radii at shared vertices, sorts the oriented endpoint constraints and lets
OCCT interpolate the law. Uniform-radius contour behavior remains supported.
A box prism with one straight top edge split into two tangent edges verified
both radius directions with BRep slices, volume bounds, incomplete-selection
and conflicting-junction rejection. An application regression edited an outer
endpoint, preserved preview when a single shared endpoint edit conflicted, and
verified commit/Undo. Windows `-j 2`: 26 kernel tests, 54 Solid input tests,
and one simulated viewport Fillet regression passed. At that checkpoint the app
could not edit coincident shared handles together; see the subsequent update.
Closed contours, curved tangent chains,
large-model native allocation and actual desktop UI remain unverified. Native
C++ is limited to the OCCT bridge; Rust owns selection, state and publication.
Skill update requested, not yet approved.

### Fillet shared-vertex batch query (2026-10-28)

The OCCT endpoint classifier now processes all candidate edges in one call,
with one native source read and one contour setup per classification rather
than repeating them for each candidate. Rust validates indices, array sizes
and outputs and the session checks temporary array budgets before the query.
A kernel fixture checked the ordered masks for the anchor, tangent neighbor
and non-tangent corner, plus empty/invalid inputs. Windows `-j 2`: 26 kernel
tests, 54 Solid input regressions, four simulated Fillet option tests and one
simulated viewport test passed. This is a behavior-preserving technical
change; timing and native peak allocation on large BReps remain unmeasured.

### Fillet shared tangent-vertex handles (2026-10-28)

CurrentRadius on a selected endpoint now updates the endpoint slots sharing
the same OCCT vertex within its fillet tangent contour, then rebuilds preview.
The bridge compares topological identity and contour membership, rather than
coordinates alone. Failure restores all changed slots and the old preview.
On the split straight-edge fixture, editing the shared vertex updates both
slots, an excessive radius restores both, and commit/Undo succeeds. A kernel
check confirms a non-tangent edge at a shared corner is not linked. Windows
`-j 2`: 26 kernel, 54 Solid input, four simulated Fillet option tests and one
simulated viewport test passed. Two coincident handles are still drawn; actual
desktop picking and curves/closed contours remain unverified. The native
query currently rereads and validates the BRep for each candidate edge, so
large-model latency and native peak allocation require measurement. C++ is
limited to OCCT topology identification; Rust owns state and publication.
Skill update requested, not yet approved.

2026-10-28 Boss oblique cylinder: polygon and reverse match independent area; finite-end miss and partial join are rejected; translated menu/CMD and simulated viewport Undo pass. Other oblique curves, original Matrix direction and actual desktop remain open. Details: 2026-10-28-solid-boss-source-map.md.

2026-10-28 Boss oblique curved profiles: circle and NURBS ellipse in both directions match independent areas, end crossing rejected; translated controller and simulated circle viewport Undo pass. General curves, near-axis geometry, original behavior and desktop remain open. See 2026-10-28-solid-boss-source-map.md.

2026-10-28 Truncated Cone staged input: see 2026-10-28-solid-truncated-cone.md. Center, base radius, signed height and top radius in captured CPlane, Diameter and _TCone alias checked; documented alternate modes, preview, original solver and desktop remain open.

2026-10-28 Truncated Cone 2Point: diameter endpoints, right-click icon, shared option routes, signed preview and captured-CPlane cross-viewport height click checked. See 2026-10-28-solid-truncated-cone.md; remaining modes and desktop are open.

2026-10-28 Truncated Cone 3Point: circumcircle in captured CPlane, collinear retry, preview, Undo and simulated viewport passed. Radius suboption after two points and other documented modes remain open. See 2026-10-28-solid-truncated-cone.md.

2026-10-28 Truncated Cone 3Point Radius: numeric/point radius, two side solutions, retry, preview, Undo, Front frame, shared options and simulated viewport passed. Point interpretation versus Matrix and other Solid modes remain open; see 2026-10-28-solid-truncated-cone.md.

2026-10-28 Truncated Cone Area: squared document units convert to base radius, with Radius/Diameter switching, invalid retry and shared option routes; other modes and desktop remain open. See 2026-10-28-solid-truncated-cone.md.

2026-10-28 Truncated Cone Solid: Yes closed, No open lateral face, equal radii cylinder; analytic area/volume, shared options and simulated viewport checked. Original Matrix open/default behavior, scale and desktop remain open; see 2026-10-28-solid-truncated-cone.md.

2026-10-28 Truncated Cone FitPoints: direct points and placed selected point/mesh/curve/surface controls, shared algebraic fit, preview, budget, Undo and simulated viewport checked. Individual control picking, general 3D fit, Matrix original solver and desktop remain open; see 2026-10-28-solid-truncated-cone.md.

2026-10-28 Truncated Cone AroundCurve: placed line/circle tangent-normal frame, point or OnCurve center, shared option routes, viewport and Undo checked. General curve accuracy, original solver and desktop remain open; see 2026-10-28-solid-truncated-cone.md.

2026-10-28 Truncated Cone Tangent + Point: single-curve contact and non-tangent point, two sides, line/circle, Undo, shared option routes and simulated viewport checked. Multi-curve tangency, original solver and desktop remain open; see 2026-10-28-solid-truncated-cone.md.
2026-10-29: Truncated Cone rejects direct dimensions that bypass a selected base mode and rejects a numeric radius before the Tangent base is resolved; see the Truncated Cone validation record for focused tests and remaining gaps.

# 2026-10-29: Cylinder height wire preview

Broad Solid regression exposed and resolved two recursive stack overflows; see `docs/validation/2026-10-29-solid-wide-regression.md`.

Cone staged base/apex construction has partial evidence in `docs/validation/2026-10-29-solid-cone-basic.md`.

Pyramid Star has partial evidence in `docs/validation/2026-10-29-solid-pyramid-star.md`.

Pyramid Edge and Flip have partial evidence in `docs/validation/2026-10-29-solid-pyramid-edge.md`.

Pyramid Circumscribed base has partial evidence in `docs/validation/2026-10-29-solid-pyramid-circumscribed.md`.

Pyramid staged Inscribed construction has partial evidence in `docs/validation/2026-10-29-solid-pyramid-inscribed.md`.

Tube base Area has partial evidence in `docs/validation/2026-10-29-solid-tube-area.md`.

Tube WallThickness has partial evidence in `docs/validation/2026-10-29-solid-tube-wall-thickness.md`.

Tube BothSides has partial evidence in `docs/validation/2026-10-29-solid-tube-both-sides.md`.

Tube Tangent + Point has partial evidence in `docs/validation/2026-10-29-solid-tube-tangent-point.md`.

Tube FitPoints and AroundCurve have partial evidence in `docs/validation/2026-10-29-solid-tube-fit-around.md`.

Tube 3Point Radius, staged wire preview and simulated right-click now have partial evidence in `docs/validation/2026-10-29-solid-tube-radius-preview.md`.

Tube Center, 2Point and 3Point staged construction is partially implemented; see `docs/validation/2026-10-29-solid-tube-basic.md`.

Cylinder now offers Center when switching back from another base mode before the first point; see `docs/validation/2026-10-29-solid-cylinder-center-return.md`.

Center and 2Point signed previews, simulated viewport and shared Truncated Cone regressions passed; original Matrix and actual GPU display remain unverified. See `docs/validation/2026-10-29-solid-cylinder-wire-preview.md`.

