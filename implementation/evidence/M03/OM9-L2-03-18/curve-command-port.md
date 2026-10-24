# Curve command port status

Ported command behaviors from the FreeCAD Qt Curve/Solid controllers into the OM9 Rust session.

## Solid

- Box: all modes, options, wire preview, height drag, undo, tokens, right-click 3Point.
- Sphere: all modes, FitPoints selection, full tangent constraint matrix, previews, tokens, right-click 2Point.

## Curve

| Feature | Command | Test suite |
|---|---|---|
| OM9-CURVE-001 | Polyline | `polyline` |
| OM9-CURVE-002 | Line | `session` |
| OM9-CURVE-003 | InterpCrv | `interp` |
| OM9-CURVE-004 | Rectangle | `rect` |
| OM9-CURVE-005 | Circle | `circle` |
| OM9-CURVE-006 | Ellipse | `ellipse` |
| OM9-CURVE-007 | Arc | `arc` |
| OM9-CURVE-008 | Arc_StartPoint | `arc` |
| OM9-CURVE-009 | Rebuild | `rebuild` |
| OM9-CURVE-010 | FitCrv | `fit` |
| OM9-CURVE-011 | Blend | `blend` |
| OM9-CURVE-012 | BlendCrv | `blend` |
| OM9-CURVE-013 | Crv2View | `crv2view` |
| OM9-CURVE-015 | Pull | `pull` |
| OM9-CURVE-016 | Project | `project` |
| OM9-CURVE-017 | Intersect | `intersect` |
| OM9-CURVE-018 | Offset | `offset` |
| OM9-CURVE-019 | ExtractIsocurve | `isocurve` |
| OM9-CURVE-020 | OffsetCrvOnSrf | `offset_on_surface` |
| OM9-CURVE-021 | CurveFillet | `corner_ops` |
| OM9-CURVE-022 | CurveChamfer | `corner_ops` |
| OM9-CURVE-023 | FilletCorners | `fillet_corners` |
| OM9-CURVE-024 | ApplyCrv | `apply_crv` |
| OM9-CURVE-025 | CreateUVCrv | `uv_curves` |
| OM9-CURVE-026 | CurveThroughPt | `through_points` |
| OM9-CURVE-027 | Extend | `extend_curve` |
| OM9-CURVE-028 | Polygon | `polygon` |
| OM9-CURVE-029 | Spiral | `spiral` |
| OM9-CURVE-030 | Helix | `helix` |
| OM9-CURVE-031 | CrvSeam | `crv_seam` |
| OM9-CURVE-032 | CrvStart | `crv_ends` |
| OM9-CURVE-033 | CrvEnd | `crv_ends` |
| OM9-CURVE-035 | ContinueinterpCrv | `continue_interp` |
| OM9-CURVE-036 | Divide | `divide` |
| OM9-CURVE-037 | CurveBoolean | `curve_boolean` |
| OM9-CURVE-038 | TweenCurves | `tween_curves` |
| OM9-CURVE-039 | DupBorder | `dup_border` |
| OM9-CURVE-040 | DupEdge | `wireframe` |
| OM9-CURVE-041 | ContinueCurve | `continue_curve` |
| OM9-CURVE-042 | Curve | `control_curve` |
| OM9-CURVE-043 | CrossSectionProfiles | `cross_section` |
| OM9-CURVE-044 | AngleIsocurve | `angle_isocurve` |
| OM9-CURVE-045 | Sketch | `sketch` |
| OM9-CURVE-046 | SketchOnSrf | `sketch_on_surface` |
| OM9-CURVE-047 | InterpCrvOnSrf | `interp_on_surface` |
| OM9-CURVE-048 | PolylineOnSrf | `polyline_on_surface` |
| OM9-CURVE-049 | Mesh | `mesh_curve` |
| OM9-CURVE-050 | Convert | `to_lines` |
| OM9-CURVE-051 | Section | `section` |
| OM9-CURVE-052 | Match | `blend` |
| OM9-CURVE-053 | Silhouette | `silhouette` |
| OM9-CURVE-054 | ExtractWire | `wireframe` |
| OM9-CURVE-055 | CurveSoftEdit | `curve_soft_edit` |
| OM9-CURVE-056 | ArcBlend | `arc_blend` |
| OM9-CURVE-058 | IntersectTwoSets | `intersect_two_sets` |

## Surface

- `Revolve` (OM9-SURFACE-018): profile around the CPlane normal, full 360.
- `ExtrudeSurface` (OM9-SURFACE-002): curve extrusion with `Height=`.
- `DupBorder` also handles B-Rep boundary edges through `boundary_edges`/`duplicate_brep_edge`.
- `Heightfield` (OM9-SURFACE-031): image heightfield with `File=`, `Height=`, `Size=`.
- Depth fixes: Circle Size/Vertical/preview, Rectangle Center/3Point/Vertical/Length, Ellipse MarkFoci/Deformable/CPlane, Arc CPlane/preview, Offset/ArcBlend CPlane, Polygon tokens/preview, Helix/Spiral preview.
- Circle depth: `AroundCurve` (`OnCurve=`/path pick), `FitPoints` least-squares circle + preview, `Tangent` to two lines or line+point (+ preview), `Deformable`/`Approx`/`Degree=`/`PointCount=`, size prefixes.
- Ellipse depth: `AroundCurve` (`OnCurve=`/path pick + preview), `MarkFoci` in Center mode, `Deformable`/`Approx` toggle, radius tokens.
- Rectangle depth: numeric `Width=`/`Height=` in `3Point`/`Vertical` commits with the edge frame.
- Command options exposed as UI tokens for Divide/CrvSeam/Convert/Extend/Tween/Interp/Continue/Rebuild/Fit/Offset/Project/Blend/Pull/Intersect/Isocurve/CrossSection/FilletCorners/CornerOps/SoftEdit/ExtrudeSurface/Heightfield.
- Section/Silhouette marching: zero-crossing epsilon, saddle pairing, chain head/tail extension and merge.

## Verification

- app library: 177 passed
- solid: 67 passed, 1 ignored
- circle: 28 passed
- curve/surface command suites: all listed suites passing (see `om9-app-regression.log`)
- clippy `om9-app` and `om9-scene`: `-D warnings` clean

## Not ported yet

- Curve Boolean union handles two closed curves, disjoint pairs, and nested outer boundaries; difference/intersection and holes are not covered.
- Curve-and-surface Match that edits the source curve.
- DupBorder/DupEdge on BRep subobjects.
- Remaining surface command families outside the existing `surface_edit` owner.
- Circle tangent against circles/arcs (current Tangent solver reads line curves only).
- Arc `Tangent`/`AroundCurve`, Circle/Ellipse start/end tangent constraints.
- Section/Silhouette contour count in coarse grids can still produce multiple chains; not yet asserted as a single closed loop.
- Ellipse deformable preview still draws the analytic ring; Circle deformable preview now samples the rebuilt NURBS.
