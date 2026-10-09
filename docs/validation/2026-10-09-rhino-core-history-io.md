# RCORE-09 / RCORE-10 implementation and evidence ledger

This ledger records the focused October 9 changes in the current working tree.
It does not close either P0 specification or certify Rhino 5 compatibility.
Native macro results must be recorded with the build and output hashes by the
runner; merely adding a macro is not runtime evidence.

## Implemented safeguards

- `rust/src/history.rs`: Record controls new relationships independently of
  Update. Existing recorded descendants continue updating when Record is off.
  Graph validation also runs while Update is off. Duplicate edges, zero IDs,
  self-links, cycles and oversized FFI lengths fail without writing outputs.
  Dependency ordering uses a bounded iterative graph walk instead of repeatedly
  cloning/scanning the remaining graph.
- `Gui/HistoryFeature.cpp`: the immediate native scheduler passes document object
  IDs with a call-local lookup map, not addresses. It rejects foreign/missing
  document links. Native links remain the persistent owner; this is not a common
  generation/revision schema.
- Join History stores `HistoryDirty` when source/frame changes or a recorded
  parameter change leaves the last shape stale. Success clears it; failure keeps
  it. A recorded Join with fewer than two parents fails and clears its output.
  This explicit dirty property currently applies to Join only.
- `rust/src/core_3dm_archive.rs`: the export manifest is bounded and validated
  before Qt JSON conversion. Wrong tuple lengths, fractional/out-of-range mesh
  indices, invalid colors/booleans/tolerance, duplicate keys, embedded NUL paths,
  mixed BRep/mesh payloads and unrecognized retained fields are rejected.
  `ThreeDmPython.cpp::write3dm` invokes it before kernel reads or output writes.
- `rust/src/retained_archive.rs`: a retained namespace must be a canonical,
  nonzero UUID, matching the existing importer. Retained dependencies must
  resolve across records and component tables without identity duplication or
  cycles. Explicit nonempty/malformed issue reports reject. The graph check runs
  before native preservation staging and again before retained replay/decoding.
  Older schema-1 manifests without a component/issue table remain readable;
  referenced missing IDs still reject.

The C++ changes are native exceptions required by App property serialization,
document links and Qt/openNURBS adapters. Rust owns portable validation/policy;
C++ resolves current native objects and performs native file/geometry work.
No native pointers are retained by the new Rust entry points. This does not
certify native libraries or the existing FFI surface as memory-safe.

## Regression evidence and behavior changes

Expected failures were observed before fixes for Record-off scheduling,
disabled-update cycle validation, unresolved dependency reports, arbitrary
namespace labels and unresolved source references. The new export validator's
first test run failed to compile because the requested API did not yet exist;
that is API test-first evidence, not a native runtime failure reproduction.

Focused Rust suites: `history_graph`, `history_commands`, `builder_history`,
`core_3dm_archive`, `core_3dm_manifest`, `retained_archive`. A subsequent full
`rtk cargo test` passed **313 tests across 49 suites**. The final subtask source
audit passed with no errors over **1,666 source files**, 511 icon bindings and
510 SVG files. These are checks of the shared working tree at this checkpoint;
later integration changes require the parent runner's final verification.

`history_smoke.FCMacro`, `builder_history_smoke.FCMacro` and
`circle_history_more_smoke.FCMacro` previously asserted that Record Off freezes
existing outputs. Those assertions now expect ongoing updates when Update is
On, per the new RCORE-09 independent policy. This intentionally supersedes the
old policy baseline; it is not represented as a regression-equivalent change.

New fixtures:

- `tests/rhino_core_history_io_smoke.FCMacro`: Record/Update separation,
  persisted Join stale state, same-process FCStd reopening, malformed recorded
  Join failure, and invalid export payload preserving the destination.
  A separate invocation with `OM9_RCORE_HISTORY_RESTORE_FILE` pointing to its
  `rhino-core-history-stale.FCStd` opens before activating the OM9 workbench.
- `tests/rhino_core_geometry_smoke.FCMacro`: point, rational arc, face with a
  trim hole, open cylindrical face, solid and real mesh remain separate native
  representations through export/import. It also checks retained namespaces,
  included archive hashes, one Undo/Redo import, preserved export refusal and
  writes `rhino-core-mixed-preserved.FCStd`. A separate invocation with
  `OM9_RCORE_GEOMETRY_RESTORE_FILE` checks cold restore before OM9 activation.
  Optional `OM9_RCORE_CYCLIC_ARCHIVE` exercises pre-mutation preservation rejection.

Both macro files passed Python syntax checks only in this subtask. No GUI or
native test execution was performed by this subtask.

The primary runner subsequently built the matching native module and ran the
History/I/O, family regressions, mixed geometry and separate-process restore
checks. Their exact results are recorded in the
[integrated P0 validation report](2026-10-09-rhino-core-p0.md); this subtask's
original execution statement is retained to distinguish who ran the evidence.

Existing native targets provide additional runnable coverage:
`ThreeDmArchiveTests` (inch/unitless policy, malformed archive, layer ancestry,
extrusion volume and invalid trim), `ThreeDmBlockTests` (nested transforms,
reflection, nonuniform scale, parent color/lock, missing/cyclic definitions),
`ThreeDmGeometryTests`, `ThreeDmInventoryTests` and `ThreeDmPreservationTests`.
These targets being present is not evidence they passed this working tree.

## P0 requirements still open

| Requirement | Current scope and remaining work |
| --- | --- |
| RCORE-09 common durable record | Native family properties exist; a common document/object/generation/revision, command/backend/schema version, typed units/frame/tolerance record and migrations are not implemented across all families. |
| RCORE-09 dirty/failed state | Join now persists explicit stale state. Surface/Circle/Builder/Cage still need a common persistent state/diagnostic contract and downstream failure fixtures. |
| RCORE-09 graph mutation | Rust scheduling rejects invalid graphs. Every native source/link mutation still needs pre-mutation DAG enforcement and rollback evidence, including placement dependencies. |
| RCORE-09 subelement identity | Native links exist; numeric `EdgeN` selectors are not proof of semantic topology identity. Changed topology must resolve with evidence or become unresolved. |
| RCORE-09 async lifecycle | This work does not add a common history worker snapshot/revision/cancellation protocol or certify stale-job rejection across all families. |
| RCORE-09 replay/metadata | Affine-template builder recipe validation exists; it is not all Matrix Builders/Styles or arbitrary Rhino History replay. Missing/versioned metadata and multiple-output failure remain family-specific. |
| RCORE-10 preservation export | Source retention and explicit geometry-only export exist. Current-state merge of retained/edited/deleted/new records, layer/material table remapping and unknown userdata is still unsupported and must fail closed. |
| RCORE-10 clipboard | Whole-module search found Qt text clipboard code in CommandConsole and tests; no OM9 Rhino geometry binary protocol adapter was found in the examined module. Rhino→OM9 and OM9→Rhino remain unvalidated separately. Host FreeCAD copy/paste is a different protocol. |
| RCORE-10 output lifecycle | The public Python exporter stages beside the target and replaces only after verification. The lower-level native writer still assumes a caller-supplied staging path; disk-full/permission/cancel failures and cleanup need runtime injection tests. |
| RCORE-10 full budget | Existing archive byte/object/depth limits plus new JSON limits do not bound every openNURBS/OCCT allocation or worker resource use. |
| RCORE-10 support matrix | A complete type × field × read/edit/retain/export matrix with runtime evidence is still required for clouds, instances, annotation, materials/lights and unknown userdata. |
| RCORE-10 target application | Rhino 5 open/edit/save/reimport, exact OS clipboard formats and per-direction compatibility require the target application and cannot be inferred from archive version or openNURBS rereads. |

RCORE-09.T01–T12 and RCORE-10.T01–T12 stay open except individual assertions
actually recorded by the current-build runner. This ledger does not promote
feature status or invent a new feature ID.
