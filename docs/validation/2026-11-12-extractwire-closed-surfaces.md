# ExtractWireframe on certified closed surfaces

The standalone surface scene draws all four parameter-domain sides. For a closed axis, two of those sides coincide and depict the same seam. Previously ExtractWireframe refused a closed cylinder entirely. Unlike DupBorder, which returns only true boundaries, ExtractWireframe now returns the visible seam once alongside the other displayed domain sides, when closure is certified by the existing exact clamped-row or repeated-periodic-basis checks. A periodic axis whose closure cannot be certified is refused; unexpected coincident corners remain a refusal. A partial periodic domain is not newly supported.

A translated closed cylinder saved and reopened produced three distinct curves: two analytic radius-2 circular ends at the expected heights and one length-5 seam. Both CMD and menu dispatcher routes, sampled source-space geometry and Undo/Redo passed. A certified periodic surface produced three source-matching curves, including one seam; this comparison uses OM9 evaluation on both sides and is not an independent proof for all parameters. An uncertified periodic surface retained the document and existing Redo. A reopened closed cylinder plus a triangle mesh produced six curves in one Undo/Redo transaction. Simulated pressure on both planar and closed-cylinder surfaces retained document/Redo and succeeded after release.

Wireframe: 54 passed, four diagnostics ignored; DupBorder: 17 passed; Curve menu: 6 passed. The debug application build and no-default-features check passed, and the public-source audit reported no errors. These fixtures do not certify all closed/periodic representations, self-intersections, all placements, physical mouse interaction, physical RAM pressure or native peak memory. The original batch did not include doubly closed surfaces; a later fixture is described below. No source-only restoration or display eviction is implemented here.

## Doubly periodic surface and collapsed apex follow-up

A separately saved/reopened surface with certified periodic bases in both directions produced two distinct displayed seams. Six samples per curve match the corresponding source surface parameter lines; this is a shared-evaluator comparison, not a proof over every parameter. One extracted seam was saved and reopened, then extruded into one closed solid. Its volume agreed within 1e-5 relative with a 4,096-segment area estimate of the original surface, its height and placement were checked, and both extraction and extrusion Undo/Redo passed. This is one synthetic shape, not general certification of all doubly closed surfaces.

A separate NURBS conical surface with all upper poles exactly at its apex exposed a defect: ExtractWireframe created three output curves, one sampling to the single apex point, rather than just two noncollapsed displayed curves. At that earlier stage the desired two-output regression was ignored. The full batch then passed 55 Wireframe tests with five ignored, 17 DupBorder tests and six Curve menu tests. The subsequent fix is described below.

## Certified collapsed surface boundary

For a validated NURBS surface, when the fixed parameter lies on a fully
clamped end and every control point in that end row is exactly identical, the
boundary is a single point. ExtractWireframe now omits that boundary in both
single-surface and multi-root paths; it refuses if a surface has no remaining
curve boundaries. No sampling or tolerance-based rounding is used to declare
a boundary collapsed. Other representations are not certified by this check.

The saved/reopened translated cone fixture now returns its circular base and
one seam, verified against sampled source parameters through CMD and menu;
one Undo/Redo restores the result. A separate saved/reopened two-surface fixture
combines the pointed cone with a cone having a small nonzero top: it returns
two and three curves respectively through both invocation paths, with one
Undo/Redo. Simulated memory exhaustion on the mixed selection leaves the
document and prior Redo intact and retry succeeds. The mixed fixture checks
counts and placement, not independent samples of each output; neither case
certifies arbitrary surfaces, output roundtrip, desktop or physical RAM pressure.

On the existing `L:/om9-target` with `-j 2`, Curve menu, DupBorder, Isocurve
and Wireframe passed 6, 17, 20 and 70 tests; four Wireframe diagnostics were
ignored. The targeted tests, no-default check, debug build and public-source
audit passed. No new build directory was created.
