> Continuation (2026-10-19): the user moved the active foundation to M01. Native BRep source/topology is now implemented and checked; [current evidence](2026-10-19-brep-selection.md) supersedes the missing-policy/native-BRep-pending statements below. Earlier checkpoint facts remain historical. External UI review is still pending.

# M02 / OM9-L2-01-07 — video review and implementation checkpoint

> Continuation: occurrence-path subselection has since been implemented and verified. See [nested occurrence checkpoint](2026-10-19-occurrence-selection.md). The limitations below describe the earlier video checkpoint. Native BRep and external review remain open.

Date: 2026-10-19. Target: the standalone Rust application in the root workspace.
This checkpoint supersedes the broad completion claims in the earlier M02 status and traceability notes.
It does **not** close native BRep topology or sign the M02-UI human review gate.

## Reference review

The supplied `20261018-2048-18.5400263.mp4` is 39.274646 seconds, 1330×1332, 30 fps.
SHA256: `e01a125b6a35e87be8d15518150018bdfd0a7b60f529ced66610d94fbdb2b2a9`.
Forty timestamped frames and four contact sheets were inspected. Eighteen unique images and the local
selection, subobject, window/crossing and Gumball text excerpts under the selected M02 L2 were reviewed.
No source PDF was reopened. Reference images and video frames remain in ignored build storage.

| Clip time (approximate) | Visible behavior | Implementation / oracle |
| --- | --- | --- |
| 0–7 s | Four views; a curve is drawn and appears in the views | Existing shared document/scene; creation is fixture setup, outside this selection module |
| 8–10 s | Selecting exposes a Gumball; deselection hides it | Click and empty-space/Esc widget tests |
| 11–16 s | Axis manipulation changes the curve and pivot | Captured press basis; native +5 mm preview, one release transaction |
| 17–28 s | Rotation arc; a yellow sector indicates the angle | Transient angle sector, no accumulated cursor trails; native 90° matrix oracle |
| 29–37 s | Curve/handle manipulations, scale and selection transitions | Square handle scale ×2; shared previews and Undo/Redo |
| 38–39 s | Unselected curve remains; Gumball is absent | Existing deselection regression |

The clip does not reveal individual modifier key presses. Alt copy toggling is taken from the local
Gumball text reference, not inferred from the command area's generic Alt hint. The sector is angle
feedback; the clip is not evidence for a timed sequence of fading Gumball copies.

## Implemented and corrected

- Crossing intersects rendered segments and clipped mesh faces, including a band entirely inside a face
  and a line whose endpoints are both outside. Window requires complete visible containment. Hidden and
  unselectable roots are excluded; instance roots are returned once.
- Mesh edge picking uses original polygon boundaries, canonical endpoint keys and perspective-correct
  world interpolation. It excludes tessellation diagonals and retains source keys on clipped edges.
  Face fallback barycentrics were corrected. Point/curve queries cannot fabricate mesh face keys.
- Subselection validates geometry kind, range, polygon edge existence, permissions and source stamps.
  Publication invalidates stale keys without rematching. Unsupported occurrence paths return an explicit
  error. Subselection storage is charged to the document owner's budget.
- Gumball hit testing starts at the press position, including a fast first movement. Samples use the
  captured frame/camera and total displacement; the final release position is included. Scaling is
  independent of sample count. Pivot placement uses displayed bounding extents rather than vertex count.
- Escape, lost focus, changed selection/document/camera and errors cancel preview. No-motion gestures
  do not create a transaction. Alt toggles copy; a second tap returns to move. A committed gesture is one
  Undo entry; cancellation preserves the pre-existing Redo branch.
- All GPU frames now pass through the common click/hover/title/widget path. Selected mesh indices are
  refreshed after a selection change. Selected curves/points have a separate line batch and preview
  model uniform; they no longer stay at the old coordinates. Static geometry is retained during drags.
- GPU highlight and ghost passes honor alpha, allow equal surface depth, and leave scene depth intact.
  CPU and GPU show the old-position ghost and current preview; rotation adds an angle sector.
- Selection-frame cache keys include publication, preview, camera and complete viewport rectangle.
  Pick tolerance is six logical points; egui performs the DPI conversion once.

## Actual verification

The executed commands, counts, log hashes and failed/repaired lint attempt are retained in
[`2026-10-19-selection-checks.json`](2026-10-19-selection-checks.json). These are suite counts, not counts
of completed implementation packages, and not a completion percentage.

| Suite | Passed tests |
| --- | ---: |
| `om9-core` Gumball | 15 |
| `om9-scene` complete crate | 69 |
| `om9-app` selected selection/Gumball widget suites | 61 |
| `om9-app --features gpu-sphere --lib` | 96 |
| `om9-render` overlay unit suite | 2 |
| Gumball archive / fresh executable reopen | 1 |

Clippy with warnings denied passed after repairing two new lint errors. Python compilation passed.
The fresh executable test preserved entity identity and placement through `.om9` save/open and a
separate `--inspect` process; session selection was not restored as document state.

Native receipts are summarized in [`2026-10-19-native-selection.json`](2026-10-19-native-selection.json).
Each replay uses two authored fixtures (curve and polygon mesh), twelve measured state/history checks,
and three image probes away from the old positions. Pixels verify the moved curve in Top/Perspective
and the moved mesh's highlight. Screenshots also capture move, scale and rotation feedback. The receipts
identify four GPU scene viewports, executable/source hashes and clean process exit.

Pointer, key and focus inputs in this replay are synthetic events through real native widgets. They
do not establish Windows foreground-focus behavior or constitute a human approval. Early attempts that
lacked native focus or used a too-small high-DPI window are failures, not acceptance evidence. The final
tool uses explicit synthetic focus, scaled window sizes and a chroma-aware line pixel check for filtered
GPU textures at fractional DPI. Production focus cancellation remains enabled and has its own test.

Builds use `CARGO_TARGET_DIR=L:/om9-target` and `-j 2`. No standalone directory, FreeCAD host build,
native adapter change, source copy from QymCAD or publication was introduced by this task.

## Scope and remaining gates

| L3 suffix | Supported standalone scope | Remaining limitation |
| --- | --- | --- |
| 001 | Screen rays, depth-ordered point/curve/mesh hits and source validation | Native BRep hit mapping absent |
| 002 | Persistent object IDs, deterministic set updates, stale/permission rejection | Human acceptance pending |
| 003 | Revision-scoped mesh face/edge/vertex keys and invalidation | Native BRep topology and occurrence-path subselection absent |
| 004 | Window/crossing, clipping, transforms, visibility/selection permissions | Curves queried against displayed samples; human acceptance pending |
| 005 | Deterministic object choices, labels/IDs, stale popup rejection, Escape preservation | Per-face ambiguity is not an object-choice feature |
| 006 | Shared selection, hover, subobject outlines and GPU highlight | Native BRep subentity overlays absent; physical input review pending |
| 007 | Axis/plane/free move, rotation, scale, pivot and animated feedback | Native BRep representation remains downstream work |
| 008 | Preview, single transaction, copy, cancel, Undo/Redo, cold `.om9` | Human acceptance pending |

`GeometryKind::from_tag` explicitly refuses BRep and the live `Geometry` enum has no native BRep owner.
Mesh face indices protected by source revisions do not prove the native BRep requirement in L3-003.
Implementing that owner also requires the v2 backend/ownership policy; the canonical directory
`OpenMatrix9_Codex_Spec_v2/` is absent from this checkout. Its AGENTS, RCORE-00, CODE_GUIDE,
BACKEND_POLICY, MATRIX9_UI_CONTRACT and MIGRATION_LEDGER could not be read or synchronized. V1 was not
substituted. The user has been asked for the relocated v2 path while independent work continues.

The checked local QymCAD revision is `b5a15dae9ece3ba4a01e7c26fe60b051c9837e2a`, different from the imported
M02 pin. The local picking/selection routines were inspected as references; this work adds independently
authored OM9 logic and does not certify the imported pinned-source gate.

The public-source audit exits 1 with 11,735 reports concerning existing reference/manual/raster paths.
None names a new output from this task. Examples include `manuals/`, `vendor-research/` and milestone
reference images. These inputs were preserved; the failed audit is not recorded as a public-release pass.

Next: restore/locate v2, resolve the native BRep model/topology owner, implement and test that remaining
scope, then obtain the separately required M02-UI review. Overall L2 status remains partial/validated
supported scope; the whole M02 milestone is not declared complete.
