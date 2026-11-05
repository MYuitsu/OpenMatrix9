# Options by menu — Matrix reference and verified OM9 state

Use this file when a command in one of the twelve menus is changed. It is the per-command locator for the per-L3 Matrix reference texts (`OM9_Milestones/**/L3-*/reference/text`, book 1/2/addendum). Read it together with [command-option-visibility.md](command-option-visibility.md), which holds the stage rules, the workflow and the conflict policy.

The manual pages are two-column, so an automatic transcript can interleave the neighbouring command (the reference text for Circle p138, for example, starts with Rectangle's Rounded section). Rows below are therefore marked as:

- **verified** — option set and stage rules were read and exercised in OM9 (tests in `docs/validation/2026-11-02-command-option-visibility.md` and `docs/validation/2026-11-02-matrix-command-prompts.md`);
- **auto** — words were extracted automatically from the reference text and MUST be re-read in the page before a provider change;
- **pending** — no reference section was matched automatically; search the L3 refs for the quoted command name.

The OM9 start column lists the tokens the command publishes at its first prompt in a headless run; `-` means the command needs a selection or argument before it starts.

## Art (0 commands)

The Art menu icons exist in `menu_catalog::ART`, but this build maps no command-line
command for them (`assets/features.json` has no `command` for `OM9-ART-*` except
`SaveScreenAreaAsHeightfield`, which is listed under Render/Tools). Treat Art as
icon/menu-only until a command mapping exists.

## Builder (5 commands)

- `gvEternity` — pending. [ref p117, L3-001 - Eternity Band Builder] OM9 start: -.
- `gvSignet` — pending. [ref p117, L3-001 - Eternity Band Builder] OM9 start: -.
- `ClaySignetRing` — pending. [ref p24, L3-001 - Clayoo Ring Builder] OM9 start: -.
- `gvBuilderRaisedBand` — pending. [ref p123, L3-003 - Raised Band Builder] OM9 start: -.
- `gvNatBuiler` — pending. [ref p131, L3-004 - Nautilus Builder] OM9 start: -.

## Curve (55 commands)

- `Polyline` — **verified**: PersistentClose, Mode=Line/Arc, Helpers; Close after 3 points; Length only in Line mode; Direction/Center only in Arc mode. [Matrix p122-123] Stage: start: PersistentClose/Mode/Helpers; after the first point: Length/Undo; after 3 points: Close; Arc mode swaps Length for Direction/Center. OM9 start: PersistentClose, Mode, Helpers.
- `Line` — **verified**: BothSides; a bare number at the end prompt is the length. [Matrix p123-125] Stage: BothSides before the first point; after it the end point is picked or a length is typed (`Length=` stays a typed alias). Matrix also lists Normal/Angled/Vertical/FourPoint/Bisector/Perpendicular/Tangent/Extension (not implemented). OM9 start: BothSides.
- `InterpCrv` — auto: PersistentClose, Degree, Start Tangent, Uniform, Chord, End Tangent, SqrtChrd, Close, Vertical, Sharp, Undo, Center. [ref p135, L3-001 - Line Polyline Point Creation] OM9 start: Degree, Cancel.
- `Rectangle` — **verified**: 3Point, Vertical, Center; Length after the first corner; square by Shift. [Matrix p127] Stage: modes before the first corner; `Length`/`Undo` after it; holding Shift while picking the corner makes it square (no Square option; typed `Square` stays a hidden alias). Matrix `Rounded` (Arc/Conic) remains open. OM9 start: Vertical, 3Point, Center, Cancel.
- `Circle` — **verified**: Deformable, Vertical, 2Point, 3Point, Tangent, AroundCurve, FitPoints; after the center: Radius, Orientation, Diameter, Circumference, Area. [Matrix p128-130] Stage: seven mode words at the center prompt; after the center the other measures + Orientation; 3Point+Radius and Tangent FromFirstPoint still open. OM9 start: Deformable, Vertical, 2Point, 3Point, Tangent, AroundCurve, FitPoints.
- `Ellipse` — **verified**: Deformable, PointCount, Vertical, Corner, Diameter, FromFoci, MarkFoci, AroundCurve. [Matrix p130-131] Stage: modes only before the first input; current mode is the prompt; Approx is a typed alias only. OM9 start: Corner, Diameter, Vertical, FromFoci, AroundCurve, Deformable, Cancel.
- `Arc` — **verified**: 3Point, Center, StartPoint; Center adds Radius/Diameter. [Matrix p132-134] Stage: modes only before the first point; Radius/Diameter only in Center mode; Matrix: center-based default with StartPoint/Tangent/Extension (pending). OM9 start: Center, StartPoint, Cancel.
- `Arc_StartPoint` — pending. [ref p144, L3-002 - Circle Arc Ellipse Polygon Creation] OM9 start: 3Point, Center, Cancel.
- `Rebuild` — auto: Direction, ThroughPoint, PointCount, Blend Amount, Start Blend, End Blend, Ring Rail, Calculate, Point Count, Degree, Preview. [ref p144, L3-002 - Circle Arc Ellipse Polygon Creation] OM9 start: -.
- `FitCrv` — auto: DeleteInput, Perpendicular, Degree, OutputLayer, Current, Input, Angle Tolerance, AtAngle. [ref p146, L3-008 - Rebuild Refit And Knot Operations] OM9 start: -.
- `Blend` — pending. [ref p146, L3-007 - Blend Arcblend And Match Curve Continuity] OM9 start: -.
- `BlendCrv` — pending. [ref p149, L3-007 - Blend Arcblend And Match Curve Continuity] OM9 start: -.
- `Crv2View` — auto: Full=True/False, Direction. [ref p149, L3-007 - Blend Arcblend And Match Curve Continuity] OM9 start: -.
- `Pull` — pending. [ref p151, L3-009 - Project Pullback And Curve Intersections] OM9 start: -.
- `Project` — auto: Loose, DeleteInput, OutputLayer, Input, Current. [ref p151, L3-009 - Project Pullback And Curve Intersections] OM9 start: -.
- `Intersect` — auto: Distance, Chamfer, Sharp, ThroughPoint, Round, Tolerance, BothSides, Smooth, InCPlane, None, Right Click Command, Flat, Flip, Number. [ref p152, L3-007 - Geom Intersection Query Adapter] OM9 start: -.
- `Offset` — pending. [ref p152, L3-007 - Geom Intersection Query Adapter] OM9 start: -.
- `OffsetCrvOnSrf` — pending. OM9 start: -.
- `ExtractIsocurve` — pending. [ref p154, L3-006 - Surface Edge And Isocurve Semantics] OM9 start: -.
- `Fillet` — auto: Direction, Toggle, ExtractAll, Radius, No, Join, Yes, Trim. [ref p156, L3-006 - Offset Fillet Chamfer Curves] OM9 start: NextRadius, ShowRadius, Undo, Cancel.
- `FilletCorners` — auto: Distances, Join, Trim, ExtendArcsBy, Arcs, Line. [ref p156, L3-006 - Offset Fillet Chamfer Curves] OM9 start: -.
- `Chamfer` — pending. [ref p156, L3-006 - Offset Fillet Chamfer Curves] OM9 start: -.
- `CreateUVCrv` — pending. [ref p157, L3-006 - Offset Fillet Chamfer Curves] OM9 start: -.
- `ApplyCrv` — auto: Closed, Degree. [ref p158, L3-026 - Curve · Through Points - menu binding] OM9 start: -.
- `CurveThroughPt` — pending. [ref p157, L3-006 - Offset Fillet Chamfer Curves] OM9 start: -.
- `Extend` — auto: Type=Natural, Type=Arc, Uniform, Center, Chord, SqrtChrd. [ref p158, L3-026 - Curve · Through Points - menu binding] OM9 start: -.
- `Polygon` — auto: NumSides, Inscribed, Type=Smooth, ExtensionLength, Undo, Circumscribed, Vertical, AroundCurve, Edge. [ref p160, L3-002 - Circle Arc Ellipse Polygon Creation] OM9 start: Sides, Circumscribed, Inscribed, Cancel.
- `Spiral` — auto: Flat, ReverseTwist, NumPointsPerTurn, Vertical, AroundCurve. [ref p160, L3-002 - Circle Arc Ellipse Polygon Creation] OM9 start: StartRadius, EndRadius, Turns, Height, Axis, Cancel.
- `Helix` — auto: Vertical. [ref p161, L3-029 - Curve · Spiral - menu binding] OM9 start: Radius, Height, Turns, Axis, Cancel.
- `Point` — pending. [ref p162, L3-001 - Point And Polyline Semantic Types] OM9 start: none.
- `CrvStart` — pending. [ref p162, L3-001 - Point And Polyline Semantic Types] OM9 start: -.
- `CrvEnd` — pending. [ref p162, L3-001 - Point And Polyline Semantic Types] OM9 start: -.
- `CrvSeam` — auto: PersistentClose, Close, Sharp, Undo, Flip, Automatic, Natural. [ref p162, L3-001 - Point And Polyline Semantic Types] OM9 start: -.
- `ContinueinterpCrv` — auto: None, Length, Used, Split, CombineRegions, MarkEnds, GroupOutput. [ref p163, L3-003 - Interpolated And Controlpoint NURBS Curve] OM9 start: -.
- `Divide` — pending. [ref p163, L3-003 - Interpolated And Controlpoint NURBS Curve] OM9 start: -.
- `CurveBoolean` — pending. [ref p164, L3-011 - Curve Seams Directions And Divisions] OM9 start: -.
- `TweenCurves` — pending. [ref p164, L3-011 - Curve Seams Directions And Divisions] OM9 start: -.
- `DupEdge` — auto: Number, MatchMethod, None, Refit, OutputLayer, Current, Input, SamplePoints. [ref p165, L3-010 - Duplicate Edge Border Isocurve] OM9 start: -.
- `DupBorder` — pending. [ref p165, L3-010 - Duplicate Edge Border Isocurve] OM9 start: -.
- `Curve` — pending. [ref p166, L3-003 - Interpolated And Controlpoint NURBS Curve] OM9 start: Degree, Undo, Cancel.
- `ContinueCurve` — auto: Angle, PersistentClose, Close, Sharp. [ref p166, L3-003 - Interpolated And Controlpoint NURBS Curve] OM9 start: -.
- `gvExtractIsocurve` — pending. [ref p167, L3-003 - Interpolated And Controlpoint NURBS Curve] OM9 start: -.
- `CSec` — pending. [ref p167, L3-003 - Interpolated And Controlpoint NURBS Curve] OM9 start: -.
- `Sketch` — auto: Closed=No, Closed=Yes, Closed. [ref p168, L3-012 - Sketch On Surface Mesh And Two Views] OM9 start: -.
- `Sketch O` — pending. [ref p168, L3-012 - Sketch On Surface Mesh And Two Views] OM9 start: -.
- `Sketch N` — pending. [ref p169, L3-012 - Sketch On Surface Mesh And Two Views] OM9 start: -.
- `gvPolylineOnSurface` — auto: Undo. [ref p169, L3-012 - Sketch On Surface Mesh And Two Views] OM9 start: -.
- `InterpCrvOnSrf` — auto: Closed=No, Closed=Yes. [ref p169, L3-012 - Sketch On Surface Mesh And Two Views] OM9 start: -.
- `Convert` — auto: Input, Arcs, Lines. [ref p170, L3-050 - Curve · Curve to Lines - menu binding] OM9 start: -.
- `Section` — pending. [ref p170, L3-050 - Curve · Curve to Lines - menu binding] OM9 start: -.
- `Match` — auto: AssignLayersBy, CurrentLayer, InputObject, JoinCurves, ByPolysurface, ByContourPlane, None, GroupObjectsByContourPlane, SurfaceEdge, No, Yes, ExtendSection, Position, Tangency, Curvature, Perpendicular to Edge, Average Curves, Join, Merge. [ref p172, L3-007 - Blend Arcblend And Match Curve Continuity] OM9 start: -.
- `ExtractWire` — auto: OutputLayer, Current, Input, GroupOutput, Distance, SetBasePoint, Copy=Yes/No, Flip, FixEnds, No, Yes. [ref p173, L3-054 - Curve · Extract Wire Frame - menu binding] OM9 start: -.
- `Offset Normal` — pending. [ref p174, L3-006 - Offset Fillet Chamfer Curves] OM9 start: -.
- `ArcBlend` — pending. [ref p174, L3-006 - Offset Fillet Chamfer Curves] OM9 start: -.
- `IntersectTwoSets` — auto: AlternateSolution, RadiusDifference, Trim, Join. [ref p175, L3-009 - Project Pullback And Curve Intersections] OM9 start: -.

## Cutters (6 commands)

- `gvChannelBuilder` — pending. [ref p280, L3-003 - Gem Cutter Library Profiles] OM9 start: -.
- `gvMicroProngCutter` — auto: Surface to Attach, Mirror. [ref p282, L3-001 - Channel Builder Cutter] OM9 start: -.
- `gvBrightCutChannel` — pending. [ref p285, L3-003 - Bright Cut Channel Builder] OM9 start: -.
- `gvBrightCutter` — pending. [ref p285, L3-003 - Bright Cut Channel Builder] OM9 start: -.
- `gvBezelCutter` — pending. [ref p288, L3-004 - Bezel Cutter Builder] OM9 start: -.
- `gvCutRingRail` — pending. [ref p290, L3-001 - Cut To Finger Rail] OM9 start: -.

## Gems (17 commands)

- `gvLoader` — pending. [ref p177, L2-04-09 - 01-febe-gem-asset-shapes-and-loader] OM9 start: none.
- `gvGemOnCrv` — auto: Gem Tops, SelectGem, Sphere, Object, ShowProngs, Browser. [ref p177, L2-04-09 - 01-febe-gem-asset-shapes-and-loader] OM9 start: -.
- `gvGemOnCrvAdvanced` — pending. [ref p181, L3-001 - Gem On Curve Basic] OM9 start: -.
- `gvGemCountOnCrv` — pending. [ref p187, L3-002 - Gem On Curve Advanced] OM9 start: -.
- `gvGemListOnCrv` — pending. [ref p191, L3-003 - Gem Count On Curve] OM9 start: -.
- `gvGemOnCrvMulti` — pending. [ref p197, L3-005 - Gem On Curve Multi] OM9 start: -.
- `gvGemGemGuides` — pending. [ref p200, L3-008 - Gem Facet Outline Girdle Reference API] OM9 start: -.
- `gvOrientGems` — pending. [ref p209, L3-007 - Gem On Surface] OM9 start: -.
- `gvEmerald` — pending. [ref p212, L3-004 - Gem Custom Shape From Profile] OM9 start: none.
- `gvBaguette` — pending. [ref p214, L3-005 - Gem Emerald And Baguette Builders] OM9 start: none.
- `gvTaperBagChannel` — pending. [ref p217, L3-005 - Gem Emerald And Baguette Builders] OM9 start: -.
- `gvGemMap` — pending. [ref p225, L3-007 - Gem Map Viewport Overlay] OM9 start: -.
- `gvGemFollow` — pending. [ref p234, L3-008 - Gem Follow Underlying Curve] OM9 start: -.
- `gvGemControl` — pending. [ref p234, L3-008 - Gem Follow Underlying Curve] OM9 start: -.
- `gvMatchAttributes` — pending. OM9 start: -.
- `gvGemSplop` — pending. OM9 start: -.
- `gvGemPositioner` — pending. OM9 start: -.

## Render (16 commands)

- `Spotlight` — pending. OM9 start: -.
- `PointLight` — pending. OM9 start: -.
- `DirectionalLight` — pending. OM9 start: -.
- `SetSpotLightToView` — pending. OM9 start: -.
- `RectangularLight` — pending. OM9 start: -.
- `LinerLight` — pending. OM9 start: -.
- `SetRenderPlugin` — pending. OM9 start: -.
- `EMap` — pending. OM9 start: -.
- `ApplyEdgeSoftening` — pending. OM9 start: -.
- `gvAnimationBuilderAdvanced` — pending. OM9 start: -.
- `gvRenderEditor` — pending. OM9 start: -.
- `gvBatchRender` — pending. OM9 start: -.
- `gvFourViewCapture` — auto: Size, Format, Viewport, Save To, Render, Width and Height, View, from the four named viewports, Save Channel On/Off, Run time, Completing the Capture, Schedule. [ref p337, L3-004 - Four Viewport Capture And Layout] OM9 start: -.
- `gvAlphaErase` — pending. OM9 start: -.
- `SaveRenderWindowAs` — pending. OM9 start: -.
- `ApplyDisplacement` — auto: RemoveAll, On, Texture, Import From File, New, Edit, Black point, Duplicate, White point, Mapping channel, Initial quality, Max faces, Fairing, Advanced settings, Mesh memory limit (MB), Refine steps, Refine sensitivity. [ref p340, L3-003 - Render Editor Crop Watermark And Alpha] OM9 start: -.

## Setting (12 commands)

- `gvHeadBuilder` — pending. [ref p249, L2-04-14 - 01-febe-head-and-bezel-brep-settings] OM9 start: -.
- `gvBezelBuilder` — pending. OM9 start: -.
- `ClayBezel` — pending. [ref p20, L3-001 - Clayoo Pipe From Curves] OM9 start: -.
- `gvPullToRail` — pending. [ref p255, L3-006 - Scallop Bezel Builder] OM9 start: -.
- `gvBezelCutter` — pending. [ref p288, L3-004 - Bezel Cutter Builder] OM9 start: -.
- `gvProngAdder` — pending. [ref p260, L3-001 - Prong Builder Gem Line] OM9 start: -.
- `gvProngEditor` — pending. [ref p260, L3-001 - Prong Builder Gem Line] OM9 start: -.
- `gvProngOnSurface` — pending. [ref p261, L3-002 - Prong Editor Selected] OM9 start: -.
- `gvOrientOnSurface` — pending. [ref p263, L3-003 - Prong On Surface] OM9 start: -.
- `gvMetalPiece` — auto: Tweak Corners, Start Position, Distance, Corner Angle, End Position. [ref p268, L3-004 - Metal Piece From Gems] OM9 start: -.
- `gvBeadOnCrv` — auto: Mirror, Object Type, RoundGem, SelectGem, Sphere, Object, Browser, GemLineDir(Ydir/Xdir), SnaptoCorners, Side Offset, ShowProngs, End Offset. [ref p270, L3-006 - Bead On Curve Milgrain] OM9 start: -.
- `gvChannelBorder` — auto: Spacing/End/Offset, Tweak Corners, Corner Angle, Type, Line, Arc, Smooth. [ref p271, L3-005 - Channel Border Curve] OM9 start: -.

## Solid (28 commands)

- `BooleanUnion` — auto: DeleteInput. [ref p223, L3-001 - B-Rep Boolean Union] OM9 start: DeleteInput, Undo, Cancel.
- `BooleanDifference` — pending. OM9 start: DeleteInput, Undo, Cancel.
- `BooleanIntersection` — pending. OM9 start: DeleteInput, Undo, Cancel.
- `Boolean2Objects` — pending. [ref p224, L3-005 - Planar Cap And Close Solid] OM9 start: Union, Difference, ReverseDifference, Intersection, InversionIntersection, Next, DeleteInput, Undo, Cancel.
- `Cap` — pending. [ref p224, L3-005 - Planar Cap And Close Solid] OM9 start: Undo, Cancel.
- `ExtractSrf` — pending. [ref p225, L3-006 - Fillet And Chamfer B-Rep] OM9 start: Copy, OutputLayer, Undo, Cancel.
- `FilletEdge` — auto: Copy=Yes/No, AngleTolerance, Undo, Next, ShowRadius, All, NextRadius, PreviousEdgeSelection, AutoChain Yes/No, FromCurve, FromTwoPoints, AddHandle, ChainContinuity, Direction, CopyHandle, GapTolerance, TrimAndJoin. [ref p227, L3-006 - Fillet And Chamfer B-Rep] OM9 start: NextRadius, ShowRadius, Undo, Cancel.
- `TextObject` — pending. [ref p227, L3-006 - Fillet And Chamfer B-Rep] OM9 start: none.
- `ExtrudeCrv` — auto: ToPoint, AlongCurve, Draft Angle, Direction, Corners=Sharp/ Round/ Smooth, FlipAngle, BothSides, Solid, DeleteInput, ToBoundary, SplitAtTangents, SubCurve= Yes/No, AutoChain Yes/No. [ref p229, L3-002 - Curve To Solid Extrusion] OM9 start: none.
- `Pipe` — auto: Multiple. [ref p231, L3-002 - Curve To Solid Extrusion] OM9 start: ShapeBlending=Global, ShapeBlending=Local, Cap, Diameter, Thick, Multiple, ChainEdges, Undo, Cancel.
- `Box` — **verified**: Diagonal, 3Point, Vertical, Center; Cube after a Diagonal corner. [Matrix p222-223] Stage: Cube only after the diagonal first corner. OM9 start: Diagonal, 3Point, Vertical, Center.
- `Sphere` — **verified**: 2Point, 3Point, 4Point, Vertical, AroundCurve, FitPoints, Tangent; 3Point adds Radius after two points. [Matrix p223] Stage: current mode is the prompt; constraint stages keep Point/Undo/Cancel; Matrix does not document Vertical/Center as options. OM9 start: 2Point, 3Point, 4Point, Vertical, AroundCurve, FitPoints, Tangent, Cancel.
- `Ellipsoid` — auto: Right-Click Command. [ref p233, L3-001 - Analytic Solid Primitive Factories] OM9 start: Diameter, Corner, AroundCurve, FromFoci, Undo, Cancel.
- `Torus` — auto: Corner, Diameter, Vertical, From Foci, MarkFoci (From Foci Option), Around Curve, Radius, Tangent, None, Point, AroundCurve, Solid Yes/ No, FitPoints. [ref p234, L3-001 - Analytic Solid Primitive Factories] OM9 start: 2Point, 3Point, Vertical, AroundCurve, Undo, Cancel.
- `Cylinder` — pending. [ref p235, L3-001 - Analytic Solid Primitive Factories] OM9 start: Tangent, FitPoints, AroundCurve, 3Point, 2Point, Solid, Diameter, Undo, Cancel.
- `Tube` — pending. [ref p236, L3-001 - Analytic Solid Primitive Factories] OM9 start: Tangent, AroundCurve, FitPoints, 3Point, 2Point, BothSides, Diameter, Undo, Cancel.
- `Pyramid` — pending. [ref p237, L3-001 - Analytic Solid Primitive Factories] OM9 start: Circumscribed, Star, Edge, AroundCurve, NumSides, Solid, Undo, Cancel.
- `Cone` — pending. [ref p238, L3-001 - Analytic Solid Primitive Factories] OM9 start: Tangent, FitPoints, AroundCurve, 3Point, 2Point, Solid, Diameter, Undo, Cancel.
- `TCone` — pending. [ref p238, L3-001 - Analytic Solid Primitive Factories] OM9 start: Tangent, AroundCurve, FitPoints, 2Point, 3Point, Solid, Diameter, Undo, Cancel.
- `Boss` — pending. [ref p239, L3-001 - Analytic Solid Primitive Factories] OM9 start: Undo, Cancel.
- `Rib` — pending. OM9 start: -.
- `Slab` — pending. OM9 start: -.
- `MakeHole` — pending. OM9 start: -.
- `ArrayHole` — pending. OM9 start: -.
- `ArrayHolePolar` — pending. OM9 start: -.
- `MoveHole` — pending. OM9 start: -.
- `Shell` — auto: Direction Constraint=None (default), Direction Constraint=Normal, Thickness, ToBoundary. [ref p246, L3-007 - Offset Face And Thicken] OM9 start: Thickness, Undo, Cancel.
- `SolidPtOn` — pending. [ref p23, L3-018 - Transform · Symmetry - menu binding] OM9 start: -.

## SubD (44 commands)

- `tsSubdivideFaceSimple` — pending. OM9 start: -.
- `ClaySplitSides` — pending. [ref p14, L3-001 - Clayoo Divide And Split Sides] OM9 start: -.
- `ClayInset` — pending. [ref p14, L3-001 - Clayoo Divide And Split Sides] OM9 start: -.
- `ClayKnife` — pending. [ref p15, L3-002 - Clayoo Inset And Knife] OM9 start: -.
- `ClayOffset` — pending. [ref p17, L3-003 - Clayoo Extrude And Offset] OM9 start: -.
- `tsExtrude` — pending. [ref p53, L3-002 - T-Splines Select Vertex Edge Face Object Mode] OM9 start: -.
- `tsBridge` — pending. [ref p57, L3-005 - T-Spline Bridge Append Fill And Extrude] OM9 start: -.
- `ClayAppend` — pending. [ref p18, L3-004 - Clayoo Bridge And Append Face] OM9 start: -.
- `tsPipe` — auto: CurveSplitting, Hint, Segments, AddHandle, RemoveHandle, Radius (Circle) Handle, SetMultiple, Rotation Handle, ClearMultiple, Radius, Diamond, Preview, OutputType = Smooth/ Box, AlwaysIncludeAutojoint, Joints, CurrJointSet, DefromJoints=Yes/No, ExportJointSet, Square, ImportJointSet, AddJointSet, Open, Spikes, Bendy, DeleteJointSet, Difference between tsPipe and Pipe. [ref p61, L3-006 - T-Spline Skin Loft Pipe] OM9 start: -.
- `ClayBezel` — pending. [ref p20, L3-001 - Clayoo Pipe From Curves] OM9 start: -.
- `ClaySignetRing` — pending. [ref p24, L3-001 - Clayoo Ring Builder] OM9 start: -.
- `tsCrease` — pending. [ref p62, L3-004 - T-Spline Subdivide And Unweld Weld] OM9 start: -.
- `tsFlip` — pending. [ref p89, L2-09-01 - 01-febe-tsplines-legacy-hud-command] OM9 start: -.
- `ClayProjectToPlane` — pending. OM9 start: -.
- `ClayCollapse` — pending. [ref p28, L3-006 - Clayoo Collapse And Merge] OM9 start: -.
- `tsMatch` — pending. [ref p83, L2-09-01 - 01-febe-tsplines-legacy-hud-command] OM9 start: -.
- `tsUnweld` — pending. [ref p64, L3-004 - T-Spline Subdivide And Unweld Weld] OM9 start: -.
- `tsSymmetry` — auto: Segments, Weld, Tolerance. [ref p46, L3-005 - T-Splines X Y Symmetry Toggles] OM9 start: -.
- `ClayBox` — pending. [ref p32, L3-001 - Clayoo Box And Plane Generators] OM9 start: -.
- `tsSphere` — pending. [ref p66, L3-001 - T-Spline Box Sphere Cylinder Cone Torus] OM9 start: -.
- `tsCylinder` — pending. [ref p71, L3-001 - T-Spline Box Sphere Cylinder Cone Torus] OM9 start: -.
- `tsCone` — pending. [ref p71, L3-001 - T-Spline Box Sphere Cylinder Cone Torus] OM9 start: -.
- `tsTorus` — pending. OM9 start: -.
- `tsPlane` — auto: YFaces, Axial Symmetry=Yes/No, XSymmetry=Yes/No, YSymmetry=Yes/No, Vertical. [ref p74, L3-002 - T-Spline Plane And Tube From Curves] OM9 start: -.
- `ClaySweep1` — pending. [ref p38, L3-001 - Clayoo Box And Plane Generators] OM9 start: -.
- `ClaySweep2` — pending. [ref p39, L3-002 - Clayoo Sweep One Rail] OM9 start: -.
- `tsLoft` — pending. OM9 start: -.
- `ClayWrap` — pending. [ref p41, L3-004 - Clayoo Loft And Revolve] OM9 start: -.
- `ClayRevolve` — pending. [ref p44, L3-004 - Clayoo Loft And Revolve] OM9 start: -.
- `gvtsTubeFromCrvs` — pending. [ref p74, L3-002 - T-Spline Plane And Tube From Curves] OM9 start: -.
- `tsFillHole` — auto: Insertion Type = Exact/ Simple, the surface shape. Simple, RetopoSnap=(Yes/No), RetopoOffset. [ref p61, L3-006 - T-Spline Skin Loft Pipe] OM9 start: -.
- `ClayShell` — pending. [ref p47, L3-005 - Clayoo Fill And Shell] OM9 start: -.
- `gvtsExtractWireframe` — pending. OM9 start: -.
- `ClaytoNurbs` — pending. [ref p50, L3-002 - Clayoo SubD To NURBS Surfaces] OM9 start: -.
- `tsSelSet` — auto: ShowTPoints Yes/No, ShowLPoints Yes/No. [ref p84, L2-09-01 - 01-febe-tsplines-legacy-hud-command] OM9 start: -.
- `ClayRing` — pending. [ref p24, L3-001 - Clayoo Ring Builder] OM9 start: -.
- `ClaySelectUV` — pending. [ref p51, L3-001 - Clayoo Ring Edge Loop And UV Select] OM9 start: -.
- `ClaySelectAll` — pending. [ref p52, L3-001 - Clayoo Ring Edge Loop And UV Select] OM9 start: -.
- `ClaySelectNone` — pending. [ref p52, L3-001 - Clayoo Ring Edge Loop And UV Select] OM9 start: -.
- `ClayPaintSelection` — pending. [ref p53, L3-003 - Clayoo Paint Selection] OM9 start: -.
- `ClayInvert` — pending. [ref p54, L3-002 - Clayoo Select All None Invert] OM9 start: -.
- `ClaySelectNakedEdges` — pending. [ref p55, L3-005 - Clayoo Crease And Naked Edge Filters] OM9 start: -.
- `ClaySelectCoplanar` — pending. [ref p56, L3-005 - Clayoo Subdivision Level Preview] OM9 start: -.
- `gvUserLibraryCustom` — pending. [ref p56, L3-005 - Clayoo Subdivision Level Preview] OM9 start: -.

## Surface (30 commands)

- `Sweep2` — auto: Chain Edges, Point, Do Not Simplify (default), Position (G0), Preserve First/ Last Shape, Tangency (G1), Maintain Height, Curvature (G2), Preview. [ref p186, L3-007 - Sweep Single And Double Rail] OM9 start: -.
- `gvProfileSweep` — auto: Closed=Yes (default), Closed=No, Flip=Yes/No, Maintain Height=Yes/No. [ref p188, L3-001 - Sweep1 Sweep2 And Profile Sweep Workflows] OM9 start: -.
- `PlanarSrf` — pending. [ref p197, L3-003 - Open Closed And Planar Curve Rules] OM9 start: -.
- `Plane` — pending. [ref p199, L3-005 - Planar Boundary Surface Generator] OM9 start: none.
- `Rebuild` — auto: Direction, ThroughPoint, PointCount, Blend Amount, Start Blend, End Blend, Ring Rail, Calculate, Point Count, Degree, Preview. [ref p144, L3-002 - Circle Arc Ellipse Polygon Creation] OM9 start: -.
- `BlendSrf` — pending. [ref p189, L3-006 - Surface Rebuild Refit And Softedit] OM9 start: -.
- `VariableBlendSrf` — pending. [ref p192, L3-007 - Surface Match Blend And Join Continuity] OM9 start: -.
- `ShrinkTrimmedSrf` — pending. [ref p196, L3-005 - Trimmed Vs Underlying Surface Model] OM9 start: -.
- `Patch` — pending. [ref p196, L3-005 - Trimmed Vs Underlying Surface Model] OM9 start: -.
- `EdgeSrf` — pending. [ref p198, L3-005 - Planar Boundary Surface Generator] OM9 start: -.
- `Untrim` — auto: KeepTrimObjects Yes/No, BothSides, Corners Round, Corners Smooth, Solid Yes/No, DeleteInput Yes/No, DeleteInput, ToBoundary, FlipAngle, SplitAtTangents, SetBasePoint, Direction, DraftAngle, Corners Sharp (Default), SubCurve. [ref p199, L3-005 - Trimmed Vs Underlying Surface Model] OM9 start: -.
- `Revolve` — pending. [ref p201, L3-003 - Revolve Railrevolve And Extrude Workflows] OM9 start: -.
- `RailRevolve` — auto: ScaleHeight, SplitAtTangents. [ref p203, L3-003 - Revolve Profile And Rail Revolve] OM9 start: -.
- `gvfourSweep` — pending. [ref p212, L3-001 - Sweep1 Sweep2 And Profile Sweep Workflows] OM9 start: -.
- `TweenSurfaces` — auto: LinkHandles, UnlinkHandles, NumberOfSurfaces, AddHandle, MatchMethod, SideTangency, None, Refit, SamplePoints, SampleNumber, Smooth, Tolerance. [ref p209, L3-026 - Surface · Tween Surfaces - menu binding] OM9 start: -.
- `FilletSrf` — auto: Radius, Extend, Trim=Yes/No, Split, Radius/Distance Options, FromCurve, FromTwoPoints. [ref p204, L3-006 - Fillet And Chamfer B-Rep] OM9 start: -.
- `VariableFilletSrf` — pending. OM9 start: -.
- `ChamferSrf` — auto: Distances, Extend, Trim = Yes/No/Split, Radius/Distance Options. [ref p206, L3-006 - Fillet And Chamfer B-Rep] OM9 start: -.
- `VariableChamferSrf` — auto: RemoveHandle, Distance, SetAll, Corner = Round/ Sharp, Solid = Yes/ No, TrimAndJoin, Loose = Yes/ No, Preview Yes/No, Tolerance, Both Sides. [ref p206, L3-006 - Fillet And Chamfer B-Rep] OM9 start: -.
- `OffsetSrf` — pending. [ref p207, L3-007 - Offset Face And Thicken] OM9 start: -.
- `VariableOffsetSrf` — pending. [ref p207, L3-007 - Offset Face And Thicken] OM9 start: -.
- `Drape` — pending. OM9 start: -.
- `Heightfield` — pending. [ref p213, L3-001 - Sweep1 Sweep2 And Profile Sweep Workflows] OM9 start: File, Height, Size, Cancel.
- `ExtendSrf` — auto: Smooth, Line. [ref p214, L3-005 - Trim Split Untrim And Shrink Surface] OM9 start: -.
- `Smash` — pending. OM9 start: -.
- `SoftEditSrf` — auto: SrfNormal, UTangnet, VTangent. [ref p217, L3-006 - Surface Rebuild Refit And Softedit] OM9 start: -.
- `CutPlane` — pending. OM9 start: none.
- `SrfSeam` — pending. [ref p217, L3-006 - Surface Rebuild Refit And Softedit] OM9 start: -.
- `SetSurfaceTangent` — pending. [ref p218, L3-007 - Surface Match Blend And Join Continuity] OM9 start: -.
- `RefitSrf` — pending. OM9 start: -.

## Tools (14 commands)

- `gvRingRail` — pending. [ref p88, L3-005 - Top11 Ringrail Entrypoint] OM9 start: none.
- `gvProfilePlacer` — pending. [ref p139, L2-04-02 - 05-febe-ring-rail-profile-and-resize-tools] OM9 start: -.
- `gvOutsideRingRail` — pending. [ref p140, L3-002 - Outside Ring Rail Generator] OM9 start: -.
- `gvProfile` — pending. [ref p161, L3-006 - Profile Edit Merge And Endcap] OM9 start: -.
- `gvSmartTarget` — pending. [ref p146, L3-001 - Smart Target Place And Add] OM9 start: -.
- `gvSmartTargetAdd` — pending. OM9 start: -.
- `gvSmartBlend` — pending. [ref p147, L3-001 - Smart Target Place And Add] OM9 start: -.
- `gvSmartMSR` — pending. [ref p45, L3-004 - Smart MSR Control Handles] OM9 start: -.
- `gvJoinHistory` — pending. [ref p149, L3-005 - Edit Existing Builder By History] OM9 start: -.
- `gvImageTrace` — pending. [ref p149, L3-005 - Edit Existing Builder By History] OM9 start: -.
- `gvMeshRepair` — pending. [ref p151, L3-005 - Mesh Repair Before After Report] OM9 start: -.
- `gvMergeProfiles` — auto: Amount, Flip. [ref p162, L3-006 - Profile Edit Merge And Endcap] OM9 start: -.
- `gvProfileEndCap` — pending. [ref p162, L3-006 - Profile Edit Merge And Endcap] OM9 start: -.
- `gvInset` — auto: Angle, Length, ModelUnits, Delete, Reduce, Select, Undo, Weld=Yes/No, WeldAngle, SplitDisjointMeshes=Yes/No. [ref p167, L3-009 - Object On Curve Tool] OM9 start: -.

## Transform (38 commands)

- `Scale` — pending. [ref p11, L3-006 - OCCT Transform Matrix Service] OM9 start: -.
- `Scale2D` — auto: Copy=Yes/No. [ref p11, L3-006 - OCCT Transform Matrix Service] OM9 start: -.
- `Scale1D` — auto: Copy=Yes/No, WorldCoordinates=Yes/No. [ref p12, L3-003 - Nonuniform Scaling And Units] OM9 start: -.
- `ScaleNU` — pending. OM9 start: -.
- `Orient` — auto: Copy=Yes/No, Scale=NO, Scale=1D, Scale=3D. [ref p13, L3-007 - Orientation Two Three Point Solver] OM9 start: -.
- `Orient3Pt` — auto: Example 2, Copy=Yes/No. [ref p13, L3-007 - Orientation Two Three Point Solver] OM9 start: -.
- `Rotate3D` — auto: Right Click Command, Copy=Yes/No. [ref p14, L3-001 - Translation Rotation Scaling Matrices] OM9 start: -.
- `Bend` — pending. [ref p14, L3-001 - Translation Rotation Scaling Matrices] OM9 start: -.
- `Array Polar` — auto: Rigid Yes/ No, deformed as they are transformed. No, Right Click Command, LimitToSpine Yes/ No, region is bent. No, Angle, Symmetric Yes/ No, around the center. No, Example, PreserveStructure=Yes, PreserveStructure=No, Preview=Yes, Preview=No, StepAngle, Rotate, ZOffset. [ref p16, L3-004 - Polar And Linear Instance Pattern] OM9 start: -.
- `ProjectToCplane` — pending. OM9 start: -.
- `gvSmartFlow` — auto: Clear, BoundingBoxAddX, BoundingBoxAddY, Tolerance, Around=Yes/No, Around=Yes on the left, the Destination surface. Around=No Right. [ref p16, L3-004 - Polar And Linear Instance Pattern] OM9 start: -.
- `GvSmartFlowRigid` — pending. [ref p18, L3-011 - Transform · Smart Flow - menu binding] OM9 start: -.
- `gvSmartFlowBreak` — pending. OM9 start: -.
- `gvSmartFlowPullDownCurves` — pending. [ref p20, L3-006 - Surface UV Normal Placement Frames] OM9 start: -.
- `OrientOnSrf` — pending. [ref p19, L3-006 - Surface UV Normal Placement Frames] OM9 start: -.
- `OrientOnCrv` — auto: OnSurface, Copy=Yes/No, Flip, IgnoreTrims= Yes/No, XFlip/YFlip, Rotate, Perpendicular. [ref p22, L3-005 - Curve Sampled Placement Frames] OM9 start: -.
- `Symmetry` — auto: None, Position, Smooth. [ref p22, L3-005 - Curve Sampled Placement Frames] OM9 start: -.
- `SolidPtOn` — pending. [ref p23, L3-018 - Transform · Symmetry - menu binding] OM9 start: -.
- `CageEdit` — auto: Basepoint, Distance Between Items, XDegree/YDegree/ZDegree. [ref p26, L3-020 - Transform · Cage Edit - menu binding] OM9 start: -.
- `ArrayCrv` — auto: Number, Roadlike, Yes, No. [ref p27, L3-005 - Curve Sampled Placement Frames] OM9 start: -.
- `ArrayLinear` — auto: Divide, Multiple. [ref p27, L3-004 - Polar And Linear Instance Pattern] OM9 start: -.
- `ArraySrf` — pending. [ref p27, L3-004 - Polar And Linear Instance Pattern] OM9 start: -.
- `ArrayCrvOnSrf` — pending. [ref p28, L3-004 - Polar And Linear Instance Pattern] OM9 start: -.
- `Array` — pending. [ref p28, L3-004 - Polar And Linear Instance Pattern] OM9 start: -.
- `Flow` — auto: Copy=Yes/No, Yes, No, Plane. [ref p30, L3-027 - Transform · Flow Along Curve - menu binding] OM9 start: -.
- `Splop` — pending. [ref p30, L3-027 - Transform · Flow Along Curve - menu binding] OM9 start: -.
- `SetPt` — auto: Set X/Set Y/Set Z, Copy=Yes/No, Align to World/Align to C-Plane. [ref p31, L3-005 - Align And Setpoints] OM9 start: -.
- `ScaleByPlane` — pending. [ref p32, L3-031 - Transform · Scale By Plane - menu binding] OM9 start: -.
- `Stretch` — auto: Bisector, BothSides, Perpendicular, Normal, Perpendicular Options, Angled, Tangent, Tangent Options, Extension, Vertical, FourPoint, Copy=Yes/No, Rigid=Yes/No, deformed as they are transformed. No, Length. [ref p32, L3-031 - Transform · Scale By Plane - menu binding] OM9 start: -.
- `Taper` — pending. [ref p34, L3-032 - Transform · Stretch - menu binding] OM9 start: -.
- `Shear` — auto: Copy=Yes/No, Rigid=Yes/No, deformed as they are transformed. No, Flat=Yes/No, are deformed. No. [ref p35, L3-033 - Transform · Taper - menu binding] OM9 start: -.
- `Twist` — pending. [ref p36, L3-034 - Transform · Shear - menu binding] OM9 start: -.
- `Maelstrom` — pending. [ref p36, L3-034 - Transform · Shear - menu binding] OM9 start: -.
- `gvMakeDownFacing` — auto: U/V/N, Copy=Yes/No, CPlane, Scale, View, Along Tangent. [ref p37, L3-036 - Transform · Maelstrom - menu binding] OM9 start: -.
- `RemapCPlane` — pending. [ref p37, L3-036 - Transform · Maelstrom - menu binding] OM9 start: -.
- `MoveUVN` — pending. [ref p38, L3-038 - Transform · Remap to C-Plane - menu binding] OM9 start: -.
- `SoftMove` — auto: Vertical, Copy=Yes/No, LinkAll. [ref p38, L3-038 - Transform · Remap to C-Plane - menu binding] OM9 start: -.
- `Cage` — auto: BoundingBox, CoordinateSystem, CPlane, World, Diagonal, Cage points. [ref p39, L3-040 - Transform · Soft Move - menu binding] OM9 start: -.
