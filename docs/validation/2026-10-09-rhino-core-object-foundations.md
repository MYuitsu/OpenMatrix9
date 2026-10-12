# Rhino core P0 — shared object foundations — 2026-10-09

Status: **validated bounded increment; full P0 remains in progress**.
Parent: `f5dc335e330c2ba07cb8dc724b13f4109bb8e956` on `main` (already merged/pushed).
This continuation is on `codex/rhino-core-object-foundations`.

## Result and scope

RCORE-01/04/06 now share a Rust registry for document sessions, native object IDs,
incarnations, geometry revisions and Undo/Redo epochs. C++ listens to FreeCAD
lifecycle signals and captures explicit unit context, world placement and bounded
placement-owner stamps. Capture is read-only with respect to document objects,
properties, modified flag and Undo. Tokens contain values, not retained host pointers.

Edit/Boolean, Rebuild and interactive Sweep1/Sweep2/Loft preserve the accepted
snapshot and reject stale input before their transaction. Every Chain Edges member
retains a guard. Same-name/equal-shape replacement, source edit followed by equal
geometry restoration, Undo resurrection and document close/reopen cannot revive
old work. Surface source edits now require Cancel/reselect; option-only corrections
remain supported. Built-in labels/Visibility and display changes keep identity stable.

The [implementation contract](../features/rhino-core-object-snapshots.md) records
ownership, native-language exceptions and diagnostics API boundaries. This is an
OM9 lifecycle decision implementing the written RCORE contract, not an inferred
Rhino internal API. No command was enabled or solver tolerance changed.

## Verification

- Matching FreeCAD SDK build passed (native build session 64748, exit 0).
- Full Rust: **338 passed / 51 suite result lines**; Clippy exit 0 with **66 warning
  diagnostic lines** remaining. New registry has four tests covering many lifecycle,
  invalid-handle, overflow and ABI assertions.
- Python tooling: **49 passed, 1 skipped** (50 run, exit 0); changed native macros
  also parse successfully. Rustfmt uses edition 2024 for the new Rust files.
- Native: **1182 passed assertions / 15 exit-0 runs**: 31 lifecycle,
  6 separate-process FCStd restore, and 1145 assertions in 13 regression suites.
- Every accepted run's module hash and macro hash match the tested files. Each
  result has `ok=true` and every recorded check passed. Runtime module SHA-256:
  `7ad302835feffeb9d259ef5bc87e39e69ccd46e23bbdde197b33faa50d24aa4e`.

| Native suite | Assertions | Run directory under `build/merge_replay-1/` |
| --- | ---: | --- |
| `core_object_lifecycle_smoke.FCMacro` | 31 | `bf6c99920c124700b07a70874e9ada01` |
| `core_object_lifecycle_smoke.FCMacro (cold restore)` | 6 | `dccf842fabc042a3b65446d7240caae1` |
| `edit_commands_smoke.FCMacro` | 115 | `f4d90eaecc0d4c1d855f2a84694690ca` |
| `surface_commands_smoke.FCMacro` | 97 | `6f3989f955d940f1af96783eecd704b6` |
| `surface_options_smoke.FCMacro` | 180 | `7f163be04f574799be11893dfbc32306` |
| `surface_constraints_smoke.FCMacro` | 85 | `61c8fd781deb48c395016ca36777ac94` |
| `surface_history_smoke.FCMacro` | 65 | `1f34f9e859ed481c87200e610e571bbe` |
| `curve_spline_smoke.FCMacro` | 41 | `dd281a386a524ac7a618c1eb75040eb1` |
| `history_smoke.FCMacro` | 83 | `819b1cb0d0a74f8e825832bed340209d` |
| `solid_options_smoke.FCMacro` | 252 | `e32fc702d6564adfa2cdea59c7bac752` |
| `circle_spatial_history_smoke.FCMacro` | 53 | `0576fcaa2ff241baa94d9b441db4d287` |
| `ellipse_smoke.FCMacro` | 75 | `90960adf201547e1aaca5fc6b19b5764` |
| `cage_commands_smoke.FCMacro` | 28 | `ccdbe80ce58440f89821fab41f071334` |
| `cage_deformation_smoke.FCMacro` | 27 | `dc4c3e0ea3c544da95aa0fd5186c5930` |
| `edit_3dm_special_smoke.FCMacro` | 44 | `99abea7b973142a999479a5eac057d92` |

Lifecycle checks exercise real Loft, Join, Rebuild and Chain Edges commits as well
as diagnostic captures. They cover identical replacement, deletion/Undo of the same
native wrapper/ID, edit/restore, unrelated object isolation, parent placement changes,
display-only stability, unit changes, cross-document rejection, bounded ancestry,
unchanged modified/Undo state, and cold restore. The cold process verifies preserved
UUID/native ID/geometry/context and creates a fresh valid token; it does not reuse
a capsule from the old process.

The final source review found no actionable defect after fixing visibility filtering
and checking bounded/cycle-free ancestry before native global-placement traversal.
Source review did not itself launch a build or GUI; those results are listed above.

## Failing baselines and fixture corrections

Failed attempts are excluded from pass totals and listed with hashes in the
[machine evidence](2026-10-09-rhino-core-object-foundations-evidence.json).

1. The pre-change module accepted a same-name/equal-shape Loft replacement.
   The new guard rejects it with unchanged object and Undo counts.
2. The first adapter invalidated Visibility changes; the filter now excludes built-in
   display/label notifications, including derived `_GroupTouched`, while actual Group
   membership and placement-owner changes remain guarded.
3. The Surface constraints fixture expected automatic recovery after external source
   edits. It now reselects the incompatible source to retain solver rejection coverage,
   proves restoring old bytes remains stale, then reselects valid geometry for preview.
4. The old Circle spatial fixture expected `Record=False` to pause existing History.
   The already-merged P0 policy separates Record (new relationships) from Update
   (existing relationships). It now covers both Update states with Record off.
5. The old Cage missing-archive fixture supplied a non-UUID namespace and was rejected
   earlier by the already-merged retained identity validator. It now independently
   asserts malformed identity rejection and valid-UUID missing archive rejection,
   both without mutation. Circle/History/Cage/Retained product code was unchanged here.

## Evidence boundaries and remaining P0

Tokens are process/session-local and deliberately absent from persistent History.
Persistent Surface History reconstructs from existing durable references without
interactive session guards. Subobject existence/type remains the consumer's job;
the diagnostic capsule API is not a general topology resolver.

All-family capability routing, edit/reference/visibility/lock/layer policies,
operation-specific effective tolerance, durable History/subelement remapping,
view/CPlane revisions and asynchronous cancellation/late worker rejection remain
open. Conservative App property invalidation may require reselecting after other
non-geometric edits. Rust mutexes and guarded ABI entry points do not certify native
host safety; the existing release profile still uses `panic=abort`.

The dedicated runtime is separate from any already-open user application. This
checkpoint did not rerun every standalone 3DM kernel test or establish full Rhino 5
compatibility. Original chapter capability/fixture statuses and 607 feature status
counts are unchanged; 11 directly affected specs/catalog rows carry separate follow-up
evidence. Historical P0 and merge reports remain historical.

Source audit: 2023 files, 511 bindings, 510 authored SVGs, **0 errors**. Spec manifest: **664 entries** verified; FEATURES JSON/YAML remain equivalent.
