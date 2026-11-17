# Smooth display normals and selection bounds cache

Status: implementation and focused checks passed; native 512/768 Light/High matrix
is prepared and pending an active desktop. The requested performance comparison is
not complete. The final release build passed (`-j 2`, `L:/om9-target`).

The live batched viewport now uploads area-weighted display normals and interpolates
them in the fragment shader. GPU vertices split at CAD face boundaries and at sharp
creases (45 degrees); exactly coincident seam positions can share normals inside a
face. Triangle positions remain unchanged. Normal transforms use cofactors for
nonuniform scale, shear and mirrored placements. Authored meshes without normals
retain the derivative fallback. The separate headless single-object and cluster
renderers retain flat shading; they are not the product scene path measured here.

Immutable scene buffers retain local bounds, shared across occurrences and scene
snapshots. Remesh replaces the payload; the unregistered mutation escape hatch
invalidates the cached bounds. A camera change projects eight affine box corners
per selected occurrence, with depth-slab edge clipping when necessary. Selection
frame keys also include scene publication. The decorative frame is conservative
and can be less tight than the former all-vertex rectangle. Picking, snapping,
source identity, Boolean tolerances and exact BRep storage are unchanged.

Evidence directory: `build/standalone-validation/ui-artifacts/viewport-normals-selection-20261115/`.
It contains pre-edit source snapshots, build logs and will contain native traces,
binary/source fingerprints, GPU shading readback and measured summaries.

Checks: exact-source/Boolean/Undo/selection integration passed (17); smooth
sphere/face boundary/cofactor tests passed (3); cached bounds and near-plane
projection tests passed (2); GPU shading readback passed (1). GPU scene integration
passed separately (15, with 4 previously ignored measurement cases), including a
sequential retry saved in `scene_bench-verified.log`. A wider renderer unit run passed 39 and
failed `meshlet::measurements::a_cluster_group_has_a_coarser_second_level` at its
unchecked second LOD access. That code was not modified. Full application unit
tests retain the previously recorded canonical-BRep fixture compilation blocker;
targeted integration validation is being used. An initial integration test called a
private initialization helper; it was corrected to use the existing public profile
request and the 17-test integration batch then passed. One concurrent GPU test
retry was interrupted after stalling while the desktop was disconnected; its
receipt is retained separately and is not a passed result.

The actual batched GPU readback preserved all 23,666 painted silhouette pixels.
Distinct painted brightness levels increased from 51 (flat) to 144 (smooth), with
22,421 shaded pixels changed. The figure is a controlled renderer proof, not a
native 512/768 FPS measurement:

![GPU flat/smooth readback](../../build/standalone-validation/ui-artifacts/viewport-normals-selection-20261115/shading-proof/comparison.png)

The implementation adds a 12-byte normal attribute per GPU vertex and temporary
normal-generation workspace. Cached bounds add a fixed small payload per shared
display. Reduced RAM is not assumed; peak and settled process RAM still need the
requested native measurements.

Native comparison must keep scene layout, frame clock, retained Undo and presentation
conditions explicit. GPU-encoded frame rate is not measured RDP video delivery.
UI callback gaps and intervals without a fresh GPU image must be reported separately.
The earlier High 512 exact-worker failure remains open until a new trace proves
otherwise; a successful preview or pre-Boolean viewport is not a final Boolean pass.

Native preflight on 2026-11-15 at approximately 14:36 UTC found the current Windows
session (ID 1) disconnected (`Disc`); the console is only `Conn`. The guarded runner
therefore started **zero** native profiles. No new 512/768 FPS, freeze or RAM value
is available. The four pending profiles are 512 Light, 512 High, 768 Light, 768 High.
They use the existing independent-pair layout and ordinary Sphere commands,
retaining one Undo per sphere and one more on a successful Boolean. A test-only
400,000-object cap accommodates 768 retained snapshots; production limits and the
2,000,000-point cap are unchanged. The existing user clarification about viewport-only
versus full Boolean and fixture history remains pending; the prepared default
retains the previous full Boolean/Undo scenario.

The new replay includes eight seconds of navigation with all spheres selected.
Its analyzer reports unique GPU-encoded and GPU-completed/published frames over a
fixed window including endpoint holds, plus UI gaps, creation/Boolean transitions,
RAM and Undo. It verifies that real scene geometry is rendered in each measured
phase. Error diagnostics now retain the Boolean worker's actual error text.

Next action after RDP is active: run
`rtk proxy python build/standalone-validation/ui-artifacts/viewport-normals-selection-20261115/run-native-matrix.py`.
The runner checks the current desktop, runs profiles sequentially, preserves failed
traces and can close only its own disposable process descendants. Do not reuse old
512 numbers as measurements of this implementation.

Reusable skill behavior has not been edited. After the native validation is
finished, the conservative decorative selection-frame rule should be presented
for the consent required by the root AGENTS instructions.

User stop, 2026-11-16: RDP became Active and the frozen build was verified. The full-history Boolean matrix started, but the user stopped 512 Light at 506 created spheres because viewport geometry remained invisible. Controller/source creation advanced while Perspective still reported scene_objects=0 and published_frame_id=1535. All owned test processes were terminated; Boolean and the other three profiles were not run. This is a render/presentation publication failure observation, not evidence of a GPU driver deadlock. No native FPS/freeze/RAM profile was accepted. Do not rerun until the user directs further work.
