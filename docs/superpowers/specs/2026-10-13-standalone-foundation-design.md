# RCORE-00 standalone foundation implementation design

Authority: `OpenMatrix9_Codex_Spec_v2/specs/00-rhino-core/00-qymcad-foundation-migration.md`, CODE_GUIDE, BACKEND_POLICY and MATRIX9_UI_CONTRACT. The user requested execution of this specification on 2026-10-13. This document records implementation choices; it does not change recovered Matrix9 behavior or grant untested capabilities.

## Intended outcome

An actual standalone Rust OM9 application, with its own persistent document, geometry masters, identity, transactions, History, Undo/Redo, four-view scene, command sessions and `.om9` archive. Menu/CMD/F6 use one dispatcher. Supported slices have executable receipts; remaining commands remain visible and disabled with reasons. Existing FreeCAD reports remain historical. RCORE-00 does not require implementing all 607 commands before establishing the foundation, but its cross-cutting gates remain open until their complete coverage is demonstrated.

## Workspace and dependencies

Create `standalone/` as a separate Cargo workspace. Work on `codex/om9-standalone-foundation` in the requested checkout: current untracked v2 specifications and portable source are needed and existing modifications must be preserved. Do not commit, publish or rebuild the FreeCAD root. Keep binaries, profiles and detailed transient logs under the existing ignored build directory.

Safe Rust owns application state. Reuse existing host-free OM9 numerical routines only after checking their dependencies and tests. QymCAD remains reference-only at pinned `09a7f09ebeb87fab7c43e26774f7eb0eb88907dd`; no QymCAD source is copied by default. Native adapters are optional, operation-scoped features with explicit ABI, ownership, toolkits, license and receipts. A successful native build cannot enable untested operations. Neither FreeCAD nor QymCAD desktop is a target dependency.

## Model and publication

Typed UUID identifiers distinguish documents, entities, layers, definitions, occurrences, resources and recipes. Canonical model units are mm/radians. Entity generation, geometry and attribute revisions, document epoch and session generation form captured source stamps. Labels are presentation only. Geometry masters retain analytic/NURBS/polycurve/surface/trim/shell/solid/extrusion/mesh/cloud distinctions; display meshes belong to revision-keyed scene caches. Admission validates finite values, array dimensions, domains, topology references and aggregate limits before cloning or native calls. Unsupported families report explicit capability errors.

Documents contain immutable owned snapshots. The transaction coordinator prepares changes privately, verifies all participating document/source/context/permission stamps, validates the entire staged world, then accepts one atomic state swap. Undo entries hold before/after snapshots of every participant, including graph/resources and caller booking. Failed requests never clear executable Redo. Booked, active, nested and recording-disabled scopes retain explicit owners/savepoints. Public callbacks run after acceptance; fallible cleanup does not reverse accepted success or report a rejected commit. Reentrant mutation during preparation/publication is rejected by the coordinator state machine. Checked revision increments precede publication.

History recipes are separate from command Icon History and Undo. Recipes bind ordered typed source roles, parameters and dependencies; missing/stale/unsupported replay produces dirty/unresolved state, never stale clean output. Prepared geometry, results, reports, auxiliary records and recipe updates share source stamps and one acceptance verdict.

## Budgets and jobs

One checked aggregate budget accounts for objects, topology, poles/knots, geometry bytes, retained/resources/history bytes, depth, output counts and estimated peak copies. Allocation estimates are implementation limits, not Rhino defaults. Worker count uses `max(1,floor(0.60*N))` and is reduced by tasks and RAM. Jobs consume immutable snapshots; cancellation and document/session/job epochs discard late completions. Native concurrency is disabled until its operation-specific evidence permits it.

## Archive and exchange

Define a versioned `.om9` binary container with magic/version, bounded chunk table, SHA-256 checksums, canonical typed JSON metadata and geometry/resource/retained blobs. Validate tables, overlaps, sizes, checksums, graph and backend requirements before constructing a public document. Only explicit schema migrations are accepted. Save to a unique same-volume temporary file, flush, reread/validate, then replace atomically; failures preserve a valid existing destination. Native Undo-stack durability is not implied by model/History persistence.

Retained 3DM sources use source UUID/namespace/hash/original units and edited/deleted/new records. Current-state structural writers must respect tombstones and shared definitions. openNURBS requires a separate adapter and fresh-process fixtures. Clipboard transfer owns its data and uses the same admission/global commit path; OS formats and Rhino interoperability are separately gated.

## UI and scene

Use a Rust widget/render backend selected after inspecting its documented local/primary API. Own state renders the Matrix9 panel ordering and 300-pixel/DPI policy, four camera/CPlane/grid states, CMD transcript/options, nine recent commands, F6 Pin and Builder lifecycle. Authored OM9 assets retain command identity. Missing reference measurements stay unverified.

Scene cache keys include identity/generation/geometry/context/display revisions. Affine gestures update matrices and synchronized pick/grip proxies; original geometry is immutable until one commit. Camera navigation does not add command input or geometry Undo. Picking resolves exact object/occurrence/subelement identities and invalidates with camera/source changes. Layer visibility, editability, selectability and locked reference snapping remain separate permissions.

## Evidence and completion

Tests exercise real document state, executable Redo, source binding, numeric oracles, cold process reload, corrupt/failed writes, UI events and native adapters. Capture binary/source/fixture/hardware/backend fingerprints and linked libraries/children. Measure warm pointer, frame and commit separately; proposed p95 <=50 ms and no stall >100 ms are not claims until measured. Synchronize only tested scopes into v2, READMEs and the live ledger. Preserve v1 hashes and raw references. M01–M12 and related chapter/UI gates are not marked complete from portable tests alone.
