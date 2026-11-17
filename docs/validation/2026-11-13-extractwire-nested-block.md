# ExtractWireframe: nested block occurrences

ExtractWireframe previously refused a selected block. It now walks the captured
document definitions, composing the placement of each occurrence, and extracts
from each supported leaf source in one transaction. It does not use the display
mesh as the source for a B-rep. The occurrence list is admitted against the
existing managed budget; traversal is bounded by document depth, node and
output limits. An invisible, unselectable, unsupported or empty member causes
the whole extraction to be refused rather than silently leaving out part of the
selected block.

After `.om9` save/reopen, a two-level block with a triangle mesh and box and
two differently translated instances produced 30 distinct expected edge
midpoints under nested shear/scale. CMD and menu paths, output save/reopen and
one Undo/Redo passed. A separate nested trimmed plane produced four expected
boundary midpoints under composed affine placements and passed Undo/Redo.
Simulated managed-memory exhaustion, an unsupported curve, a hidden member and
an unselectable member preserved the document and Redo; the memory case
succeeded after the simulated pressure was released. These are headless
fixtures, not physical RAM exhaustion or a UI recovery test for blocks.

The wireframe suite passed 83 tests with six ignored before the final addition
of the unselectable-member case; the three focused nested-block tests passed
afterward. DupBorder passed 24 tests, default and no-default-feature checks
passed, and the public-source audit passed. Existing compiler warnings remain.
NativeBrep inside nested blocks, arbitrary nesting/transforms/topology,
physical desktop selection and native transient peak remain unverified. The
running desktop build was not replaced. This does not establish completion of
all ExtractWireframe modes or Plan A.

## Nested NativeBrep follow-up

One authored openNURBS quarter-cylinder 3DM fixture was imported using the
P1 Release native exchange adapter as an opaque NativeBrep. A copy of its
geometry was placed in a newly authored two-level `.om9` block definition;
two instances under affine placement produced eight edges (two arcs and two
lines each). Five samples per edge matched the quarter-cylinder oracle in
source coordinates, each output had the expected composed placement, and the
result survived save/reopen and one Undo/Redo. This is an authored `.om9`
block containing imported geometry, not a 3DM file containing an imported
block or proof of a general import-to-block operation.

The first of two runs of this test failed before extraction: the test had moved
the imported entity into a definition while leaving its retained-file binding
attached to its former root, which violates document validation. The fixture
was corrected by copying its geometry into a new document; the second run
passed. A separate generated NativeBrep with corrupt source following a valid
mesh in a block was refused and kept document and Redo unchanged. Batch
`nested-native-brep-extractwire` therefore had three test executions, two
distinct scenarios, and a final pass for each; one authored 3DM input and one
generated corrupt-source fixture were used. Independent third-party 3DM block
imports, native allocations, desktop and broad topology remain unverified.
This follow-up supersedes only the earlier statement that NativeBrep inside a
block had not been tested.

## Authored 3DM block import

An authored block containing a trimmed planar surface with a hole was exported
to 3DM with the P1 Release exchange adapter, then imported through the default
editable B-rep route. Selecting the imported block and running ExtractWireframe
produced the eight distinct expected outer and hole edges at their world-space
endpoints. One Undo/Redo restored the pre-command and post-command documents.
The existing fixture also checked hole-aware ray hits after import. A second
execution additionally imported the same block with `RetainOpaque`, confirmed
its definition member is a NativeBrep, and checked the same eight endpoints
and Undo/Redo. The named batch had two executions of one authored 3DM fixture;
both import modes passed in the final execution. It does not test an unrelated
third-party file, arbitrary nested 3DM blocks, desktop or native transient
allocations.

## Nested 3DM block import

An authored `.om9` two-level block containing a box was exported to 3DM with
two root instances and a nested shear/scale. The resulting 3DM file was
imported separately as `EditableStrict` and `RetainOpaque`; the test confirmed
the leaf type in each import, saved/reopened each `.om9` document, and ran
ExtractWireframe on both instances. Each import produced 24 distinct edges
whose endpoints matched the analytic transformed box corners; outputs survived
another save/reopen and one Undo/Redo. This batch used one authored 3DM fixture
in three successive test executions; all passed, with the final execution
including the explicit source-type and `.om9` roundtrip checks. Arbitrary
third-party nested blocks, curved B-reps, desktop and native peak remain open.

## Curved nested 3DM block

The project-authored quarter-cylinder 3DM source was imported as NativeBrep,
copied into a new two-level block with two instances and nested affine
placement, and exported as a new 3DM block. That block was imported in both
`EditableStrict` and `RetainOpaque` modes, with each leaf type verified, then
saved/reopened as `.om9` before extraction. Each instance produced two arcs
and two side edges with no duplicate edge. Five samples per edge, the arc
endpoints and side endpoints were compared with the analytic source after
independently undoing the known world placement. The output also passed
save/reopen and one Undo/Redo in both modes.

Batch `nested-curved-3dm-extractwire` ran this one authored fixture twice;
both runs passed, with the final run adding distinct-edge and endpoint checks.
This is a single sampled quarter-cylinder source, not proof of all curved
B-reps, third-party 3DM files, desktop operation or native peak. It supersedes
the earlier statement above that curved sources in nested 3DM blocks were
unverified.
