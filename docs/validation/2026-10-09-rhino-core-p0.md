# Rhino core P0 — implementation checkpoint, 2026-10-09

Scope authorized by the user: P0 foundations in
`OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core`. This checkpoint implements and
tests bounded shared services. **The complete P0 specification remains open.**
The original source audit is retained as historical evidence; the runtime
results below apply only to the current supported slices.

## Implemented scope

| Contract | Changes in this checkpoint | Remaining scope |
| --- | --- | --- |
| RCORE-01 / 07 | Rust validates the actual generated-spline publication path: degree/poles/knots/multiplicity/domain/periodic consistency. The shared Basis validator separately checks rational weights and finite homogeneous values; generated splines currently pass no weights. Native mixed geometry fixtures distinguish point, rational curve, trim hole, open periodic face, solid and mesh. | No complete type × operation × backend registry, common conversion/loss report, surface basis/trim contract or shared editor. Rational Basis validation is not yet a gate on every native rational-geometry consumer. |
| RCORE-02 | Versioned explicit model/page unit and absolute/relative/angular context, dimensional conversions, bounded encoding, native document property with Undo/Redo/FCStd. Curve construction sessions capture context; stale commit rejects. Unsuffixed construction lengths use declared model units; explicit length suffixes work. Line/Polyline/InterpCrv/Rectangle/Circle/Ellipse construction outputs retain the exact context snapshot. | Legacy documents retain existing mm behavior without an invented tolerance. Rebuild and other families still use their own tolerance/unit policies. Effective tolerance resolver, global migration, full History context and nonempty-document declared-number scaling remain unsupported. Physical-size mode leaves canonical geometry unchanged. Post-transaction failure injection for the units adapter is not yet covered. |
| RCORE-03 | Shared `w`, `r`/`R`/`@`, `wr` point grammar for Curve, Box and Sphere; finite right-handed frames. Rust owns document/view CPlane registry, bounded Previous/Next history, named identities and three-point construction. Native grid and FCStd metadata integration plus viewport World XY/XZ/YZ, Previous/Next and named save/restore menus. | View/Curve/Surface modes, interactive three-point command, universal plane mode, polar/spherical/surveyor/locale grammar and full point provenance are open. Existing Curve family latch policies remain explicit; previous/next history is in-memory. |
| RCORE-04 | Repeat tracks the last successful repeatable command separately from icon history; failed/cancelled commands and excluded view/file/history actions do not replace it. Native repeat rechecks availability. | Common typed events/options/checkpoints, common session serialization and all-family source revision guards remain open. |
| RCORE-05 | Persistent, Once/Only and suspended snap states are separate; invalid/hover/option/reference-only input does not consume Once. Accepted mouse/typed/F4 points consume the override; cancellation clears transient state. | Existing End/Mid/Point geometry coverage only; full snap/constraint composition, SmartTrack, Elevator, Grid and Locked semantics and permission/provenance unification remain open. |
| RCORE-06 | Existing native permission gates preserved; new CPlane menus check view permissions and revalidate document/view identity after dialogs. No new organization/block command is enabled. | Shared selection identity/generation and permissions across all consumers, layer/block authoring and topology identity remain open. |
| RCORE-09 | Record controls creation of new relationships independently of Update. Rust validates graphs even while updates are disabled; native scheduler uses call-local object IDs. Join persists dirty state; missing parents fail instead of leaving a successful stale result. | Common durable schema/revisions, pre-mutation graph enforcement for every family, semantic subelement mapping, async jobs and common dirty/failed state remain open. |
| RCORE-10 | Strict Rust export-manifest and retained-dependency validation before native work; namespaces must be canonical nonnil UUIDs. Retained reload revalidates dependencies. Mixed representation and separate-process FCStd fixtures supplied. | Merging retained/edited/deleted/new state into preservation export remains unsupported and fails closed. Rhino binary clipboard and Rhino 5 runtime compatibility are not certified. |

Rust owns portable state, policy and validation. New C++ code is limited to
FreeCAD document properties/signals, Qt input/menus and native geometry/scene
integration. Python additions are acceptance fixtures and local test tooling.
No private reference content, new feature IDs or unsupported command enablement
was introduced. Existing parallel Ellipse implementation was preserved; this
checkpoint only integrates its shared input units/provenance/snap paths.

## Review corrections

- Finite extreme unit values originally encoded beyond the decoder's size
  budget. A failing test reproduced this; scientific encoding now roundtrips,
  and native writes validate the encoded record before opening a transaction.
- CPlane-only changes now set the native document modified flag, including
  named saves/renames, so normal Save prompts preserve the state. No-op changes
  do not dirty the document.
- Hidden grids now receive the updated plane transform. Invalid saved CPlane
  metadata is rejected at grid, API and command boundaries without silently
  selecting a default plane or leaving a failed input session active.
- Circle/Ellipse reference selection and option changes were distinguished
  from accepted points to avoid premature one-shot consumption.
- Escape in a viewport menu or naming dialog preserves an active Curve session.
- Saved named-plane labels reject embedded NUL before the C-string boundary.
  Rust caps a document at 4096 named planes and 512 KiB total UTF-8 label bytes,
  checked atomically before save/rename/import so accepted tables fit the native
  4 MiB reload budget. These are explicit OM9 resource limits.

Independent review exercised 61 targeted Rust assertions and inspected native
ownership, error boundaries, dirty-state and one-shot integration. The primary
runner owns the native acceptance evidence below.

## Build and verification environment

- FreeCAD checkout: `H:/FreeCAD-src`; nested module checkout contains substantial
  pre-existing uncommitted work. No reset, commit, push or publication performed.
- Matching SDK: `build/relWithDebInfo`; Qt/OCCT dependencies:
  `.pixi/envs/default/Library`.
- Native target: `build/openmatrix9-history-cage`, RelWithDebInfo,
  MSVC 14.44.35207. Embedded `/Z7` debug information was used after `/Zi` failed
  with a local PDB-manager mismatch; optimization remains `/O2 /Ob1 /DNDEBUG`.
- Output: `build/reusable-history-runtime/bin/OpenMatrix9Gui.pyd`.
  Validation uses a separate FreeCAD process/profile and this explicit module
  directory. An initial run loading an old SDK module is excluded from acceptance.
- Test temporary directories were redirected inside this workspace because
  the sandbox rejected the default system TEMP path. That initial environment
  failure is not counted as a product failure.

<!-- runtime-evidence:start -->
| Native suite | Passed assertions | Evidence |
| --- | ---: | --- |
| Units/context | 20 | [results](../../build/rhino_core_units_smoke-1/0a331755d9dd46068ecae126d57f06dc/results.json) |
| Units separate-process restore | 3 | [results](../../build/rhino_core_units_smoke-1/9a3be883f97548039b6adbc2dacfe438/results.json) |
| Snap lifecycle/repeat | 10 | [results](../../build/core_snap_lifecycle_smoke-1/847799c9157641dbbf4b57172c9c1049/results.json) |
| History/I/O guards | 7 | [results](../../build/rhino_core_history_io_smoke-1/8a307e1207ad4b28bdb710a06dae57a7/results.json) |
| History regression | 83 | [results](../../build/history_smoke-1/8a834e3b2dd44d0a9e3fa507a7de916a/results.json) |
| Circle History regression | 89 | [results](../../build/circle_history_more_smoke-1/76bac59c17c24f1087078935a8f5817b/results.json) |
| Builder History regression | 43 | [results](../../build/builder_history_smoke-1/75201823902c49468fb7248d685deb56/results.json) |
| Line/Polyline regression | 27 | [results](../../build/curve_smoke-1/6f5929a5b5724818a1f24d9836ead482/results.json) |
| Rectangle regression | 37 | [results](../../build/rectangle_smoke-1/b112efe00a4d4cbf9ff9bf241f15b390/results.json) |
| Box/Sphere regression | 70 | [results](../../build/solid_commands_smoke-1/980792f9d23c43e9883319fc5da85d97/results.json) |
| Mixed geometry and retained 3DM/FCStd | 56 | [results](../../build/rhino_core_geometry_smoke-1/5c33f4f4420d40559bd854fb3e03e5c5/results.json) |
| Mixed retained geometry separate-process restore | 28 | [results](../../build/rhino_core_geometry_smoke-1/592c582abb2140f79df26f36d6142a74/results.json) |
| Invalid CPlane command boundaries | 14 | [results](../../build/core_invalid_cplane_tools_smoke-1/99da6f4a9526451c88ad39c7d386f892/results.json) |
| History stale-state separate-process restore | 2 | [results](../../build/rhino_core_history_io_smoke-1/aa1c65cec56b4b64a5a75afd3fe5e065/results.json) |
| Coordinates/CPlane native menus and persistence | 162 | [results](../../build/core_coordinates_smoke-1/1c4886e25e3d4e2aa39823a9f5e8d01c/results.json) |
| Coordinates/CPlane separate-process restore | 6 | [results](../../build/core_coordinates_smoke-1/416fb28e1c044e9297d3c12417af6d18/results.json) |

Total: **657 native GUI assertions** in 16 successful process runs.

- Rust: **321 passed across 49 suites**; Python tooling: **23 passed**.
- Rustfmt checks passed for the new units/coordinates/CPlane files and tests. Clippy exited 0 with **60 warnings remaining**, not a warning-free result.
- Public source audit passed; final file count and source fingerprints are in the [machine evidence](2026-10-09-rhino-core-p0-evidence.json). Git diff check passed with Windows CRLF recognized.
- Standalone `ThreeDmGeometryTests` passed, including adaptive ring/cylinder area, open topology and independent samples. [Kernel log](../../../../build/rcore-native/geometry-results.log).
- History/Circle initial 120-second process limits were insufficient; repeat runs with 300 seconds exited 0. An invalid-CPlane test initially left its reopened document outside its cleanup-name filter; the fixture now closes its owned documents and rerun exited 0. Those incomplete runs are excluded above.
- FreeCAD `Shape.Area` uses nonadaptive integration: the rational ring reported 285.82732078869282 instead of 91*pi. Native adaptive integration verified error 5.7e-14 for the ring and 1.4e-8 for the open cylinder. Geometry and the converter were unchanged; GUI tests independently verify boundaries, hole exclusion and dense surface samples. Repeated-import geometry is matched by namespace/source UUID, because FreeCAD can uniquify visible labels.
- GUI modified-state acceptance uses native GUI Save; calling App.saveAs alone does not clear the host GUI flag. No persistence callback was changed to clear unrelated edits.
- CPlane menu acceptance uses native Qt Home/Down/Right/Return events, checks the selected/enabled actions and resulting state, and never calls QAction.trigger or setActiveAction. This host did not deliver QtTest pointer movement to popup targets; failed pointer attempts are excluded. Pointer-hover behavior itself is not certified by the keyboard replay.
- CPlane camera comparisons preserve every serialized view field except renderer-managed nearDistance/farDistance, which are recalculated after grid scene bounds change. Raw camera snapshots remain in the results. Position, orientation, projection scale, aspect and focal distance are verified unchanged.
- Final native module SHA256: `cb399486d7ef32fd410419a310c0abe338f02470884e0731a8696f1eb1d88269`. Some earlier unaffected-family runs predate the final CPlane-only metadata hardening; captured per-run hashes are retained where available.
<!-- runtime-evidence:end -->

## Limits of acceptance

Individual native assertions do not close the full numbered RCORE fixtures
where additional options, permissions, units, instances, failure injection or
Rhino-reference behavior remain untested. P1/P2 analysis, annotation/layout and
render/resource work was outside the user's selected scope.

Follow-up must continue from the remaining-contract column above, particularly
shared object/selection identity and revision snapshots, an operation-specific
tolerance resolver, durable common History context and current-state 3DM export.
Do not infer full P0 or Rhino compatibility from the test count.

Related records: [implementation plan](../superpowers/plans/2026-10-09-rhino-core-p0.md),
[History/I/O details](2026-10-09-rhino-core-history-io.md),
[progress ledger](../openmatrix9-progress.json).
