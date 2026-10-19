# `OM9-L2-02-01` — Be Rhino Nurbs Object Semantics: API contract

Milestone `M01` · Stage `E02` · Target `standalone Rust OM9`
Audited tree: `b5a15dae9ece3ba4a01e7c26fe60b051c9837e2a` · 2026-10-19
Specification: `OM9_Milestones/M01 - CAD Shell, Viewport/L2-02-01 - Be Rhino Nurbs Object Semantics`
Status: **10 active L3, 50/50 active L4 verified with automated evidence** (`crosswalk.json`, `l4-matrix.json`).

## 1. Scope and exclusions

**In scope.** Rhino-like semantic rules over the owned geometry records:

- Point / Line / Circle / Polyline / Polycurve / NURBS curve kinds and endpoints.
- NURBS degree, pole, weight and knot semantics, checked before any kernel conversion.
- Closed and planar classification under an explicit tolerance policy.
- Surface U/V domains, periodicity, seam parameters and V orientation.
- Trimmed surface = underlying NURBS surface + outer/inner UV loops, persisted without ever
  becoming the full untrimmed surface.
- Boundary edges (pickable topology) versus display-only exact isocurves.
- Closed solid versus open surface/polysurface classification from `BrepBody`.
- Lightweight extrusion primitive and on-demand conversion to a portable B-Rep body with provenance.
- Mesh versus exact NURBS/B-Rep object-type rules.
- Block definition identity versus occurrence identity/transform.

**Out of scope.** A second geometry engine or OCCT evaluation (the module composes
`geometry`/`curve_build`/`surface_kernel`/`brep_kernel`); STEP/IGES; the 3DM reader; UI panels; the
M07 STL export command; native pointer/ABI work.

## 2. Ownership

| Concept | Owner | Decision |
| --- | --- | --- |
| `NurbsCurve`, `NurbsSurface`, `Curve`, `Geometry` | `om9-core::geometry` | REUSE |
| Curve sampling, basis, builders | `om9-core::curve_build`, `surface_kernel` | REUSE |
| B-Rep topology validation | `om9-core::brep_kernel` | REUSE |
| Semantic kinds, closed/planar, seam/orientation, trim, isocurve, solid, extrusion, instances | `om9-core::rhino_semantics` (new) | ADD |
| Known extension slots and document validation hook | `om9-core::model` | EXTEND |
| Persistence proof | `om9-archive` tests | REUSE codec |

`rhino_semantics` adds no geometry record and no native code. It is `#![forbid(unsafe_code)]` by
crate policy.

## 3. Public API

```rust
// Kinds
pub enum SemanticKind { Point, Line, Circle, Polyline, Polycurve, NurbsCurve,
                        NurbsSurface, TrimmedSurface, Extrusion, Mesh, PointCloud, Brep, Instance }
pub fn geometry_semantic_kind(&Geometry) -> SemanticKind;
pub fn curve_semantic_kind(&Curve) -> SemanticKind;
pub fn entity_semantic_kind(&Entity) -> Result<SemanticKind>;

// Curves
pub enum CurveForm { Line, Circle, Polyline, Polycurve, Nurbs }
pub struct CurveSemantics { form, closed, planar, endpoints: [Point3; 2], tolerance }
pub fn classify_curve(&Curve, tolerance: f64, limits: &Limits) -> Result<CurveSemantics>;
pub struct NurbsCurveSemantics { degree, pole_count, weight_count, knot_count,
                                 expected_knot_count, rational, closed, planar }
pub fn inspect_nurbs_curve(&NurbsCurve, tolerance, limits) -> Result<NurbsCurveSemantics>;

// Surfaces
pub struct SurfaceExpectation { periodic: [bool; 2], v_reversed: bool }
pub struct SurfaceSemantics { domain: [[f64; 2]; 2], periodic: [bool; 2],
                              seam: [Option<f64>; 2], v_reversed: bool }
pub fn classify_surface(&NurbsSurface, tolerance, limits) -> Result<SurfaceSemantics>;
impl SurfaceSemantics { pub fn require_orientation(&self, SurfaceExpectation) -> Result<()>; }

// Trim
pub struct TrimmedSurface { underlying: NurbsSurface, outer: Vec<[f64; 2]>,
                            inners: Vec<Vec<[f64; 2]>>, reversed: bool }
impl TrimmedSurface { validate, is_trimmed, underlying_surface, to_untrimmed,
                      to_extension, from_extension }
pub fn trimmed_surface_of(&Entity, &Limits) -> Result<Option<TrimmedSurface>>;
pub fn set_trimmed_surface(&mut Entity, &TrimmedSurface, &Limits) -> Result<()>;
pub fn clear_trimmed_surface(&mut Entity);

// Display/topology
pub struct DisplayIsocurve { curve, iso, value }   // display-only, not pickable
pub struct BoundaryEdge { object: ObjectId, face: Uuid, edge: Uuid }  // pickable topology

// Solid
pub enum ShapeClass { Surface, Polysurface, Solid }
pub fn classify_shell(&BrepBody, &Limits) -> Result<ShapeClass>;

// Extrusion
pub struct Extrusion { profile: Curve, direction: Point3, height: f64, capped: bool }
pub struct ExtrusionConversion { source: Extrusion, body: BrepBody }
impl Extrusion { validate, to_brep_body, convert, to_extension, from_extension }
pub fn extrusion_of(&Entity, &Limits) -> Result<Option<Extrusion>>;
pub fn set_extrusion(&mut Entity, &Extrusion, &Limits) -> Result<()>;

// Mesh vs exact
pub fn is_exact_geometry(&Geometry) -> bool;
pub fn require_exact_nurbs(&Geometry) -> Result<()>;

// Blocks
pub struct InstanceSemantic { object: ObjectId, definition: DefinitionId, transform: Transform }
pub fn block_instances(&Document, DefinitionId) -> Result<Vec<InstanceSemantic>>;
pub fn set_instance_transform(&mut Document, ObjectId, Transform) -> Result<()>;
```

## 4. Semantic rules

### 4.1 Curve kinds and endpoints

- `Geometry::Point` is `Point`; it is never a curve.
- `Curve::Line` is `Line`, `Curve::Circle` is `Circle`, `Curve::Nurbs` is `NurbsCurve`.
- A `Curve::Polycurve` whose segments are all `Line` and whose consecutive endpoints join within
  the tolerance is a `Polyline`; otherwise it remains `Polycurve` (including disconnected segments).
- `CurveSemantics.endpoints` are the sampled first/last points; they are the endpoints callers may
  display or snap.
- A line whose exact endpoints are equal is refused by `Curve::validate`
  (`Invalid("degenerate line")`). A curve whose sampled length is at or below the requested tolerance
  is refused by `classify_curve` (`Invalid("degenerate curve within tolerance")`); it is never
  reclassified as a point or polygon.

### 4.2 NURBS curve cardinality

- Required knot count is `poles + degree + 1`; multiplicity per value is at most `degree + 1`.
- Weights must be finite and strictly positive; a zero weight is refused.
- Domain must lie inside `[knots[degree], knots[poles]]`.
- These checks run in `NurbsCurve::validate` before any OCCT conversion. `inspect_nurbs_curve`
  reports the counts and whether the curve is rational.

### 4.3 Closed and planar policy

- Closed: `distance(sampled start, sampled end) <= tolerance`. The tolerance is explicit and must be
  finite, in `(0, 1e3]` mm; never raw equality.
- Planar: a Newell normal over sampled points; every sample lies within `tolerance` of the plane.
  Degenerate/collinear samples count as planar.
- The policy is documented and tested on both sides of a near-closed boundary.

### 4.4 Surface domain, seam, periodicity, orientation

- The stored `domain` is returned verbatim (no truncation) for both axes.
- A `periodic` flag must agree with the knot vector shape: a periodic record with clamped knots is
  `Invalid("surface periodic flag mismatch")`; an unclamped record with `periodic = false` is
  `Invalid("surface periodic flag missing")`.
- A periodic axis reports `seam = Some(domain.start)`. A geometrically closed but non-periodic axis
  (coincident seam rows/columns within tolerance) also reports a seam; Rhino distinguishes closed
  from periodic and so does this record.
- `require_orientation(SurfaceExpectation)` refuses a swapped U/V periodicity vector or a changed V
  orientation (`Invalid("surface U/V orientation")`).

### 4.5 Trimmed surface

- Record = underlying `NurbsSurface` + non-empty outer UV loop + optional inner loops.
- Loops need at least three finite points, no repeated consecutive point, a non-zero area, and all
  points inside the surface domain; every inner point must be inside the outer loop.
- The entity geometry remains the underlying `Geometry::Surface`; the trim is stored in the known
  extension slot `om9.trim.surface`.
- `to_untrimmed()` always fails: a trimmed patch is never promoted to the full surface.
- `Document::validate` parses and validates the slot, so an archive with a damaged trim payload is
  refused before publication.

### 4.6 Isocurves and boundary edges

- `DisplayIsocurve` wraps the exact rational isocurve from `surface_kernel::isocurve` but declares
  `is_display_only = true`, `is_model_edge = false`, `is_pickable = false`; `to_geometry()` always
  returns `Unsupported("isocurve is display-only")`, so it cannot become persistent geometry or an
  exported model edge.
- `BoundaryEdge` is the opposite: non-nil object/face/edge UUIDs, `is_model_edge = true`,
  `is_pickable = true`.

### 4.7 Solid / polysurface

- A closed shell is `Solid`; an open one-face shell is `Surface`; an open multi-face shell is
  `Polysurface`.
- A body whose `kind = Solid` but whose shell is open is refused (`Invalid("open shell is not a
  solid")`).
- A shell containing two faces over the same corner set is refused as
  `Invalid("self-overlapping shell")`.
- Geometric self-intersection of two distinct curved faces requires the OCCT kernel; this portable
  classifier refuses the topology defects it can prove and never claims watertightness for an open
  body. That boundary is recorded as a residual risk.

### 4.8 Extrusion

- Record = closed planar polyline profile + finite non-zero direction + finite non-zero height +
  `capped = true` (an open extrusion is not a solid and is refused).
- The entity geometry is the profile `Geometry::Curve`; parameters live in `om9.extrusion`.
- `to_brep_body` builds a closed portable `BrepBody`: `2n` vertices, `2n + 4n` directed edges,
  `n + 2` faces, one closed shell, `ShapeKind::Solid`. Each face owns its own directed edges; the
  kernel sews on submission.
- `convert` returns `ExtrusionConversion { source, body }`; the source record is the provenance and
  is never discarded by the conversion.
- Non-polyline or non-planar/open profiles are refused instead of approximated.

### 4.9 Mesh versus exact

- `is_exact_geometry` is true for Point/Curve/Surface/Brep/Instance and false for Mesh/Cloud.
- `require_exact_nurbs` refuses a mesh (`Unsupported("mesh is not exact NURBS")`).
- There is no implicit mesh-to-BRep/NURBS conversion API; a mesh with the same display triangles as
  a box remains `SemanticKind::Mesh`.

### 4.10 Block instances

- `block_instances` lists every live occurrence of one definition in stable object-id order.
- Each occurrence has a distinct `ObjectId` and `Transform`; the definition is shared.
- `set_instance_transform` only edits the occurrence placement and bumps its geometry revision and
  generation; the definition and sibling occurrences are untouched.

## 5. Persistence

- `om9.trim.surface` and `om9.extrusion` are registered known slots
  (`om9-core/src/model.rs:100-111`).
- `Document::validate` dispatches known slots to `rhino_semantics::validate_extension`
  (`model.rs:716`), so an invalid payload fails `Archive::encode`/`decode` before publication.
- The archive tests prove save/open equality, trim/extrusion record equality, conversion after
  reload, and refusal of a damaged trim payload.

## 6. Invariants

- One semantic owner; no duplicate geometry record and no second evaluation engine.
- Geometry kind is never inferred from a display mesh; a mesh is never reported as exact.
- A trimmed patch never becomes the untrimmed surface, and an isocurve never becomes a model edge.
- Closedness/planarity use the caller's stated tolerance, never raw floating equality.
- Open or self-overlapping topology never claims `Solid`.
- Conversion (extrusion) preserves the source record as provenance.
- All errors are typed `Error` values; invalid input leaves the document unchanged.

## 7. Residual risks / review

- Geometric face-face self-intersection needs the OCCT kernel; the portable classifier catches
  duplicate faces and open shells only. This limitation is stated in `classify_shell` docs.
- Extrusion conversion is limited to closed planar polyline profiles; NURBS-profile extrusion must
  go through the kernel with its own oracle.
- No external reviewer has signed this record; every L3 is `AUTO_ONLY` and the automated evidence is
  complete.
- The global M00 execution gate is NOT READY (pre-existing missing M00 records); no QymCAD code or
  new upstream source was imported by this work.
