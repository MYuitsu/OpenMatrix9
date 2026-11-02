# Rib: development session and multi-curve foundation (2026-10-29)

Status: partial, not publicly enabled. The user requested completion of Rib; this
record does not claim the command is complete. Matrix's recovered menu invokes
`_Rib` but does not establish its algorithm or defaults. The Rhino reference
describes planar curves, a boundary, two offset modes, straight/tapered mode and
draft angle; it does not establish that this implementation matches Matrix.

The videos embedded in the Rhino page show additional limitations: the
in-curve-plane example uses a closed circular curve and a box boundary, while
the normal-to-curve-plane example uses an upright curved input and a boundary
with perpendicular faces. The visible curve appears centered on the rib width
in both examples. This is an observation of those examples, not proof of all
defaults, exact distance semantics, or original Matrix behavior. The current
one-sided open-polyline and planar-face-only adapter cannot reproduce either
example. The reference videos are not included in the repository.

The hidden development session selects one or more open linear/polyline sources and one
bounded planar B-Rep boundary, then accepts a signed distance. It captures and
revalidates source stamps, handles compatible translated, reflected and scaled
placements, and publishes one boundary geometry mutation. Invalid input and
cancel do not commit; pending Undo moves back a selection stage and document
Undo/Redo reverses/reapplies the single transaction. The controller's viewport
selection hooks are exercised directly; this is not a desktop interaction test.

Separately, a hidden kernel function builds disjoint strips for multiple source
polylines and joins them to a planar boundary as one operation. A two-line
fixture has one shell, nine faces and 192 mm² area; an invalid second curve
rejects the complete operation without modifying the input boundary. This
function is connected to the hidden session; a two-curve controller fixture
verifies nine faces, 192 mm² and one Undo/Redo transaction. The one-sided strip remains an
OM9 development interpretation, not an established Rhino/Matrix result.

Validation: `cargo test -j 2 -p om9-kernel --features 3dm-brep --test
l2_01_10` passed 62 tests; `cargo test -j 2 -p om9-app --test solid solid_rib_`
passed 3 tests. Before the multi-curve controller change, the complete app Solid
suite passed 194 tests with one ignored. `python tools/public_source_audit.py` reported no errors.

Still required: support closed curves, solid or multi-face boundaries and the
centered width visible in the examples; determine
and implement both documented offset modes, tapered/draft, supported curves and
boundary surfaces; establish actual side/width and preservation behavior;
preview, option shortcuts, complete input validation, icon/menu/CMD routing and
desktop runtime checks. Defaults and original Matrix behavior remain unverified.
Do not expose `SolidRib` as supported based on these partial checks.
