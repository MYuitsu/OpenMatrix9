# Ambiguous surface selection — 2026-11-12

DupBorder and ExtractIsocurve each consumed the first selected object even if
multiple objects were selected. A saved/reopened two-plane product fixture
reproduced DupBorder creating a result for one plane without resolving the
user's selection. The same dispatch pattern existed for ExtractIsocurve; the
red test stopped on DupBorder first and did not independently run the old
Isocurve path.

At that stage both commands refused more than one selected object with a command-specific
message, before preparing a result or starting the Isocurve session. In saved/
reopened fixtures, two surfaces and a surface plus a point are refused through
CMD and menu, leaving the document and existing Redo unchanged. Selecting one
surface afterwards produces four DupBorder curves or one Isocurve, and a
single Undo/Redo restores the respective document. The test checks the output
count and complete document snapshots, not independent curve geometry for this
fixture. Other object types, desktop interaction and physical memory pressure
remain unverified. Supporting multiple surfaces in one call remains a separate
capability; refusal was not multi-object support. DupBorder's subsequent
standalone-surface extension is described below.

On the existing `L:/om9-target` with `-j 2`, Curve menu, DupBorder, Isocurve
and Wireframe passed 6, 19, 21 and 71 tests; four Wireframe diagnostics were
ignored. The no-default check, debug build and public source audit passed. No
new build directory was created. Per the user's prior decision, no Isocurve
business rule was written to the skill.

## DupBorder with multiple standalone surfaces

DupBorder now prepares boundaries for all selected standalone surfaces from a
tracked document snapshot, checks source permissions and placement, and
commits the complete set once. The path supports surface trims only where the
existing trimmed-plane helper supports them. At that stage, mixed selections
including points or B-reps were refused rather than partially processed. The
later B-rep extension is described below. ExtractIsocurve still requires one
selected surface.

In a saved/reopened fixture, a translated trimmed plane and a differently
translated untrimmed plane produce eight unique expected line segments through
CMD and menu, with placement checked and one Undo/Redo. Simulated memory
pressure and a seven-output limit preserve the drawing and existing Redo. A
separate ordered fixture prepares a plain plane before encountering an
trimmed cylinder with a diagonal trim; the entire operation refuses and retains
document and Redo. That fixture is not reopened. A separate no-OCCT controller
test verifies two untrimmed planes and Undo/Redo. Other mixed types, arbitrary
surface topology, reopening the output, desktop and physical memory pressure
remain unverified at that stage; output reopening is checked below.

Curve menu, DupBorder, Isocurve and Wireframe passed 6, 21, 21 and 71 tests,
with four Wireframe diagnostics ignored. The targeted no-OCCT test, existing
debug build and public source audit passed. No new build directory was created.

## DupBorder with B-reps and surfaces

The multi-object path now also obtains open boundary edges from each selected
OCCT B-rep's source, retains its placement, and includes the prepared curves in
the same transaction as the other selected surfaces or B-reps. Unsupported
selected objects still refuse. In a saved/reopened fixture, a standalone plane
plus an open cylindrical shell produces four planar edges and two circular
boundaries; two open shells produce four circular boundaries. The fixture uses
translation and scale/shear placements, checks curve endpoints or sampled
analytic circle positions and both cap heights, and verifies CMD/menu plus one
Undo/Redo. Simulated memory pressure and a five-output limit preserve document
and existing Redo. A separate saved/reopened fixture prepares a plane before
encountering a closed Box with no boundary and refuses the whole operation,
retaining document and Redo. At that stage the mixed fixture did not separately
reopen its output; the follow-up below does. Arbitrary imported B-reps, other topology, physical memory pressure,
native peak and desktop remain unverified.

Curve menu, DupBorder, Isocurve and Wireframe passed 6, 23, 21 and 71 tests,
with four Wireframe diagnostics ignored. The no-default check, existing debug
build and public source audit passed. No new build directory was created.

### Reopened output and downstream use

The multi-surface fixture now saves and reopens the resulting document and
checks identical entities; one boundary from each plane then measures 6 mm and
10 mm. The mixed plane/B-rep and two-B-rep fixtures repeat at source display
deflections 0.1 and 0.01 mm. After each CMD/menu extraction and Undo/Redo, the
output document is saved/reopened with identical entities. The lower circular
boundary of the translated B-rep measures 4π mm and is extruded by 5 mm into
one closed solid with volume 20π mm³ and expected world bounds. One Undo/Redo
for that extrusion also restores the respective documents. The affine shell's
boundary remains checked against its transformed analytic circle; this
downstream extrusion uses the translated shell. These are fixture-specific
checks, not a guarantee for arbitrary curves or desktop interactions.

The DupBorder integration batch passed 23 tests, including these two-profile
roundtrips and downstream operations; the public source audit passed. This
follow-up changes tests and evidence, not product behavior.
