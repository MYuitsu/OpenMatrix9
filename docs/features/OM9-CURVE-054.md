# OM9-CURVE-054 — Extract Wire Frame

Menu and `ExtractWireframe`/`ExtractWire` use the same dispatcher. The following standalone behavior is established only for the linked fixtures; it is not full command parity or proof for every imported geometry.

- **B-rep:** source topology edges are converted to curves on the current layer in one transaction. Fixtures cover saved/reopened Box, cylinder, Sphere and multiple roots, plus selected affine placements. OCCT-marked degenerate pole edges are skipped in the tested Sphere cases; other errors refuse the whole operation. [Source/reopen](../validation/2026-11-11-extractwire-reopened-source.md), [multiple roots](../validation/2026-11-12-extractwire-multiple-roots.md), [degenerate poles](../validation/2026-11-12-extractwire-degenerate-poles.md), [placement](../validation/2026-11-12-extractwire-affine-placement.md).
- **Standalone surface:** tested open NURBS surfaces yield four source-domain boundary curves. Tested closed cylinder and certified periodic fixtures yield the visible parameter seam once instead of twice; a doubly periodic fixture yielded two seams, one of which survived save/open and extrusion. Uncertified periodic closure is refused. A conical fixture still creates a collapsed apex curve; its handling awaits the user's decision. [Open surfaces](../validation/2026-11-12-extractwire-standalone-surface-gap.md), [closed surfaces and known issue](../validation/2026-11-12-extractwire-closed-surfaces.md).
- **Mesh and mixed selection:** a tested single triangle/quad mesh yields unique indexed face edges without the quad display diagonal. A reopened Box, triangle mesh and open surface can be processed together; invalid input or simulated memory refusal preserves the drawing and Redo. A separate actual-executable fixture also verifies a Box, triangle mesh and closed cylinder through save/open and Undo/Redo. [Mesh](../validation/2026-11-12-extractwire-mesh-edges.md), [mixed selection](../validation/2026-11-12-extractwire-mixed-sources.md).

On a recognized memory refusal, tested UI paths offer a retry after the user consents to removing oldest Undo steps as needed. A successful command uses normal Undo/Redo branching. [Memory recovery and limits](../validation/2026-11-12-memory-recovery-choice.md).

For a certified affine 2×2 planar surface, ExtractWireframe supports tested polygonal outer and inner trim loops, including one restricted non-unit parameter domain, preserving placement and one Undo/Redo. It also handles tested U/V-aligned edges on a trimmed cylinder; other trim representations remain unsupported or unverified. [Evidence and limits](../validation/2026-11-12-trimmed-surface-curve-refusal.md#curved-surface-boundaries-along-uv).

Tested self-intersecting, overlapping or nested loops are refused while preserving the document and Redo; topology checks are conservative and bounded. [Evidence](../validation/2026-11-12-trimmed-surface-curve-refusal.md#ambiguous-loop-topology).

A tested subdivided straight boundary is accepted; an adjacent doubled-back edge is refused. [Evidence](../validation/2026-11-12-trimmed-surface-curve-refusal.md#subdivided-straight-trim-boundaries).

The tested B-rep edge adapter preserves a managed-admission refusal for consent-based recovery; structural output limits remain distinct. [Evidence](../validation/2026-11-12-memory-recovery-choice.md#extractwireframe-b-rep-admission-recovery).

For a tested conical surface, a clamped boundary whose complete control row is exactly one point is omitted, while a small nonzero boundary remains. [Evidence](../validation/2026-11-12-extractwire-closed-surfaces.md#certified-collapsed-surface-boundary).

One reopened trimmed-plane fixture with UV parameters offset by 1e8 produces its correct eight boundary segments with Undo/Redo. [Evidence](../validation/2026-11-12-trimmed-surface-curve-refusal.md#large-trim-parameter-offsets).

Tested diagonal trim edges of a warped positive-weight rational bilinear surface are represented as source-derived quadratic NURBS. [Evidence](../validation/2026-11-12-trimmed-surface-curve-refusal.md#rational-bilinear-diagonal-trim).

The current scene builds ordinary B-rep wire edges from display segments and standalone surface wire edges from four sampled parameter-domain boundaries. Internal display isocurves, nested instances, every seam/import/placement, actual desktop interaction and native peak memory are not established by these fixtures. Retaining viewport display data does not by itself prove source geometry is unused. Plan A remains open; source-only restoration and display eviction are separate deferred Plan B work.
