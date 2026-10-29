# OM9-SOLID-011 — Pipe (partial)

The standalone controller currently consumes one selected curve and one numeric radius, samples the curve, transforms the samples by the entity placement into world coordinates, and creates a capped constant-radius BRep. The source remains in the document; Undo restores the prior document in the placement regression. Menu dispatch and CMD use the same controller path. If a single curve is not already selected, a simulated viewport click can select one while the command is active; Undo clears selection and Cancel does not commit. Overlapping curves use the existing disambiguation popup; stale command sessions are rejected in simulation. Preview, Redo, persistence, and actual desktop behavior remain unverified.

The normalized milestone reference specifies additional selection, radius and pipe options. They are not established by the present implementation and must not be presented as complete. The controller-level placement regression validates rigid translation and rotation using BRep bounds and volume. General curved rails and non-rigid transformations remain unverified. See [source mapping and validation](../validation/2026-10-28-solid-pipe-source-map.md) and [Solid audit](../validation/2026-10-28-solid-audit.md).

Explicit `ShapeBlending=Global` supports numeric start/end radii on a straight Line rail, followed by Enter; the resulting flat-capped frustum has verified bounds and volume in menu/CMD controller tests. Curved rails and intermediate stations are explicitly unsupported in this mode. The previous constant-radius path remains available. See the audit for test scope and pending desktop UI.

`Cap=None` and `Cap=Flat` are available in the active command; they produce an open shell and flat-capped solid respectively. Both were checked on straight rails with constant and Global radii, via menu/CMD controller paths. The OM9 preexisting default is Flat; the original default remains unverified. Round is explicitly unsupported. Token interaction and desktop have not been validated.

Radius and Diameter modes interpret numeric input accordingly; changing mode between Global endpoints preserves already entered geometry. Shared option snapshots expose the current Diameter flag and Cap choice. Controller activation, numeric values, BRep volume and Undo were tested; actual keyboard/token interaction and desktop remain unverified. Original Matrix defaults are unknown.

Follow-up: simulated egui token click, keyboard shortcut and typed-name activation now cover Diameter, Cap and Global with geometry checks. Re-selecting Global retains the first radius. Earlier notes marking these three option routes unverified are historical; actual desktop remains unverified.

`Thick=Yes` accepts two endpoint radii at each end and produces a hollow, flat-capped BRep on a straight Line rail. Varying endpoints require explicitly selected Global blending; equal endpoint pairs are rejected. Cap=None and Round and curved rails are unsupported in this mode. Tests cover the translated Global case, not all mode combinations.

Thick with Cap=None on a straight Line now produces two disconnected open wall surfaces in one BRep, tested for constant and Global endpoint radii. Surface orientation and near-tolerance behavior are unverified. Earlier descriptions that None is unsupported are historical.

Open Thick orientation follow-up: inner topology is reversed before combining walls. For the tested straight constant and Global fixtures, tessellated face windings have opposite radial signs; a gap below the 1e-7 mm sewing tolerance is rejected. Exact surface normal sampling, acceptance just above the threshold and actual desktop remain unverified.

`Cap=Round` now supports a straight Line with constant or equal Global endpoint radii, producing a closed capsule. Round with varying endpoints, Thick or curved rails is explicitly unsupported. Simulated Cap option cycles through Flat, None and Round; earlier notes saying Round is entirely unsupported are historical.

Round follow-up: Global on a straight Line now accepts unequal endpoint radii and Thick; the endpoint hemispheres follow each radius. The tested Thick result is a closed hollow BRep from outer-minus-inner capped bodies. Curved rails, cap/side continuity and actual desktop remain unverified; earlier Round limitations are historical.

Interactive default follow-up: enter start radius, end radius and Enter. Equal radii use the existing constant path; unequal values require explicit Global until original defaults and Local are established. `Pipe 1` remains a one-shot compatibility command. Intermediate radii and point input remain unavailable. Earlier statements about immediate interactive creation after one radius are historical.

In Radius mode, straight Line endpoint radii can be entered by viewport point or comma-separated CMD coordinate. OM9 interprets the radial distance in the perpendicular section plane; this is not verified against original code. Diameter-by-point and curved rails are unsupported. Simulated viewport and translated CMD fixtures measure output; Thick point input, other views and desktop remain unverified.

Global on straight Line rails now supports intermediate stations through a position point followed by a numeric or point radius, repeated until Enter; pending/station Undo is supported. Flat and None are available. OCCT ruled sections form piecewise linear conical segments; original interpolation and C0 joints require further validation. Thick, Round and curved-rail stations are not supported. Earlier descriptions of all intermediate stations as missing are historical.

Multiple stages curve selections and shared numeric radii; all outputs commit atomically. Simulated viewport and controller fixtures cover two rails, variants, failure and Undo. Shared radii are an OM9 interpretation, not a recovered original rule. Point and intermediate inputs for multiple rails and desktop remain unverified.

Right-click Pipe and `_Pipe _C` now start ChainEdges. The partial implementation accepts connected, forward-collinear root Line curves, reversing segment direction as needed and producing one Pipe. BRep edges, bends, curved chains, AutoChain and continuity controls remain unsupported; original defaults and desktop remain unverified.

ChainEdges follow-up: ordered connected collinear exact BRep line edges are selectable, alongside the separate root-Line chain path. The OCCT bridge checks line type and exact endpoints; selection Undo and Cancel, output volume and document Undo were checked in simulated viewport. Mixing edge and curve chains, instances, curved edges and desktop remain open. Earlier notes that all BRep edge selection is missing are historical.

Chain follow-up: root Line curves and exact BRep line edges may be mixed in selection order; selection Undo respects that combined order. Simulated viewport tests cover both orders and output volume. Earlier no-mixing notes are historical. Curves/bends/instances and desktop remain open.

For a supported straight ChainEdges chain, endpoint Radius points and Global intermediate stations now reference the full chain. A simulated mixed-geometry viewport/CMD fixture measures volume and Undo. OM9 projection and piecewise C0 interpolation are not verified original semantics.

Bent straight-segment chains now support constant radius with Flat/None via an OCCT circular sweep with mitered corners. This is an OM9 interpretation rather than verified original transition semantics. Thick, Round, variable radii and point/station input on bends, curved edges and desktop remain open. Earlier collinear-only notes are historical.

Bent straight-segment Thick chains now support constant inner/outer radii with Flat or None caps. The 90-degree simulated BRep-edge fixture checks hollow volume, open area, display winding and Undo; varying radii and Round remain unsupported. Earlier notes that Thick is entirely unsupported on bends are historical.

Bent straight-segment chains now support Round caps for constant radius, including constant Thick; endpoint segment lengths must be at least the outer radius. A 90-degree egui fixture checks thin/Thick volume and Undo. Complex chains, original semantics and desktop remain unverified; earlier Round-on-bends unsupported notes are historical.

Bent-chain endpoint Radius point input now uses the corresponding end segment tangent, including Thick. When equal radii are required, near-equal inputs within document absolute tolerance are normalized; template fallback is 0.001 mm. This projection/tolerance behavior is OM9 interpretation, not verified original behavior. Earlier notes marking all bent endpoint points unsupported are historical.

Thick equal-radius validation now matches inner and outer radii by magnitude across endpoints, regardless of input order, and normalizes corresponding near-equal values within document tolerance. Straight and bent-chain measured-volume/Undo regressions passed. Original Matrix ordering and tolerance semantics remain unverified.

Constant-radius Cap=Flat on a full analytic circle with circularity-preserving placement now creates an exact oriented torus (tube radius less than transformed ring radius). Menu/CMD controller tests measure volume, area, bounds and Undo. Other circular variants, non-circular affine placements, Matrix's original implementation and desktop remain unverified.

The same full-circle Cap=Flat branch supports constant inner/outer Thick radii via an exact hollow torus. Menu/CMD volume, area, closure and Undo passed; original Matrix behavior and other circular variants remain unverified.

For constant radii, Cap=None on a full analytic circle with circularity-preserving placement now returns exact toroidal surface walls rather than an approximate open seam; Thick has reversed inner topology. CMD area, no naked edges, no solid, display winding and Undo checked for an untransformed circle. Matrix original and desktop remain unverified.

On a full analytic circle with constant radii, Cap=Round now produces the same closed torus as Flat because the rail has no endpoints; Thick is supported. Controller and simulated viewport volume/Undo passed. This is an OM9 interpretation, not verified Matrix behavior.
