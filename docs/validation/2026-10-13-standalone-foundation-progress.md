# Standalone foundation execution ledger

Plan: `docs/superpowers/plans/2026-10-13-standalone-foundation.md`.
Target: OM9Standalone, RCORE-00. Branch: `codex/om9-standalone-foundation`.

## Decisions

- Execute the supplied specification in the requested checkout on a separate branch. Existing untracked v2 specs and modified portable components are required inputs; do not commit or disturb unrelated changes.
- Initial workspace is safe Rust, with no native dependencies. QymCAD pin remains reference-only; no upstream code copied.
- Operational limits are configurable OM9 policy (initial defaults: 256 MiB accounted bytes, 100,000 objects, 2,000,000 points/nodes, depth 32, degree 32). They are not recovered Rhino defaults or measured native allocation limits.
- Positive rational weights are admitted. Periodicity metadata is retained; verified periodic continuity/evaluation remains an operation gate, not inferred from the tag. CAD trim/shell/solid/native adapters remain pending.

## Task 1 — verified initial model slice

RED: `rtk cargo test --manifest-path standalone/Cargo.toml -p om9-core --test model_admission` failed with missing budget/geometry/model modules before implementation.

GREEN: `rtk cargo test --manifest-path standalone/Cargo.toml -p om9-core` — 9 passed, 0 failed, on Windows x64, Cargo 1.98.1.

Evidence: standalone typed UUIDs/source stamps; finite/domain/weights/knots/grid/index/channel admission; distinct geometry masters; checked aggregate reservation with unchanged bookkeeping after rejection; independent visibility/selectability/editability/reference-snap permissions; nested mirrored/nonuniform instance placement applied once; dimensioned length conversion preserves weights/knots.

No executable, native adapter, UI, global transaction, archive or performance gate has passed. M01–M12 remain open. Task 1 remains partial until additional representation/family admission and operation oracles are verified.

Next: Task 2 atomic coordinator and caller scopes; first tests must compare immediate rejected world/caller/Undo/Redo and execute old Redo.

## Task 2 — verified initial transaction slice

RED: focused atomic/scope suites failed with the missing standalone transaction module.

GREEN: `rtk cargo test --manifest-path standalone/Cargo.toml -p om9-core` — 28 passed across model, global publication and scope suites.

Verified: private preparation; one accepted move/Undo/Redo; immediate unchanged rejection for invalid sources, cancellation, stale publication across commit/Undo, participant failure and overflow; successful two-document ownership transfer/Undo; mutation barrier around fallible/panicking acceptance callbacks; accepted notification failure returned as diagnostics; close invalidates prepared owners; empty requests preserve Redo; booked/active/nested scopes, first-write baseline, savepoint rollback and owner-thread enforcement.

Decisions: explicit close cancels staging and retires entries involving the closed document. A successful unrecorded mutation invalidates both stacks, because older snapshots cannot safely overwrite an unrecorded edit; an empty unrecorded scope keeps stacks. Cross-document instances with definition dependencies remain explicitly unsupported pending transfer closure. Immutable Undo entries restore only their participants, preserving unrelated documents opened afterwards.

Public source audit: 4,441 files, 511 icon bindings, 510 authored SVGs, zero errors. Formatting passed. Clippy found one collapsible conditional; corrected without changing behavior, verification pending.

Still pending: full aggregate sharing/savepoint accounting and fault injection at every prepare/teardown phase; History/resources/selection participants, source bindings and actual executable/native/UI/archive gates. No M gate marked complete.

Next: Task 3 source roles, ordered subsets, prepared results and independent derivative oracles.

## Task 3 — verified initial source/evaluation/History slice

RED: source/evaluation and History suites first failed on their missing standalone modules. Two later regressions failed on omitted aggregate selected-point copy and option bytes; repaired at reservation before copy, with overlap checks using ordered ranges.

GREEN: `rtk cargo test --manifest-path standalone/Cargo.toml --target-dir C:/Users/nguye/.codex/build/om9-foundation-target -p om9-core` — 48 passed. Cargo's first full rerun failed with MSVC LNK1318 when H: had only 13 MB free; inspected disk usage, then moved only this session's Cargo cache to the named C: directory. No existing SDK, source or validation tree was moved/deleted. The subsequent full suite passed.

Verified source roles/kinds/cardinality/unknown options, native subset indices/order, whole FitPoints, duplicate/overlapping spans, context-stale revalidation; analytic circle and rational nonperiodic NURBS D0–D3 with independent numeric fixtures, reversed fraction derivative scaling, native domain/endpoints and continuity rejection. Evaluator ports the existing authored OM9 Taylor/de Boor recurrence with provenance in `standalone/provenance.json`; no QymCAD code copied.

Verified atomically consistent output Shape/snapshot/result/source stamps/recipe; transitive dirty/stale propagation across two generations, missing-source unresolved behavior without label rebinding, child identity/revision-preserving explicit replay, graph cycle rejection, History geometry lock versus appearance/reference use, and geometry/graph/result Undo/Redo together.

Pending: periodic/trimmed/offset/derived and contact solvers; complete selector/native conversion maps; replay-module registration, typed expressions, auxiliary/topology correspondence; cross-document History/resource/selection ownership and comprehensive failure injection. No executable/native/UI/archive/performance gate has passed.

Next: Task 4 capability evidence gates, job epochs/cancellation and aggregate shared/savepoint/worker budgets.

## Task 4 — verified initial capability/job/shared-budget slice

RED: focused capability/job suites failed on missing modules. The savepoint budget fixture exposed double-accounting shared Undo allocations while the current preparation omitted caller baseline/savepoint retention. Account immutable Arc owners once and reserve each planned private clone separately, including caller and history participants.

GREEN: full standalone core suite — 57 passed. Configured Cargo target remains `C:/Users/nguye/.codex/build/om9-foundation-target` because H: is nearly full.

Verified capability source presence does not enable operations; declared Rust families/options/phases require semantic receipts; commit requires a standalone target runtime receipt; native ABI/closure/native-operation/runtime gates are distinct. Registry is trusted build/application state, never populated from archive metadata. Receipt-to-current-build fingerprint verification still belongs to the executable validation pipeline.

Verified worker floor60% with task and memory reductions and no four-worker ceiling; owned immutable inputs survive document close, canceled/duplicate/late results are discarded, and commit/Undo still invalidates old jobs even when entity stamps compare equal again. Active caller/savepoint retention shares the preparation budget and rejected clone reservation leaves immediate caller/public state unchanged.

Pending: actual scheduler execution/process containment for native operations, aggregate renderer/native peak allocation measurement and receipt pipeline. Every M01–M12 gate remains open. Next: Task 5 bounded `.om9` container and actual fresh-process executable reopen.

## Cache location restored — 2026-10-13

The user freed H: and requested the cache return. Rechecked 95.9 GB free, no active Cargo/rustc/link processes, then moved only `C:/Users/nguye/.codex/build/om9-foundation-target` back to `standalone/target` (715,845, 655 bytes, 2,315 files). File counts/bytes and sampled executable SHA-256 hashes matched. New builds use the workspace's default H: target directory; earlier C: commands remain historical receipts.

## Task 5 — verified initial archive and actual process slice

RED: archive tests failed on the missing codec; the save-validation aggregate regression subsequently reproduced a missing live-model/cold-copy reservation; restore tests failed on missing module/version/session handling; the owned audit process test failed before diagnostic lifecycle implementation.

GREEN: `rtk cargo test --locked --manifest-path standalone/Cargo.toml --workspace` — 74 passed. Workspace formatting requires `cargo fmt --all` now that multiple crates are present. Clippy with `-D warnings` passed, and the release `om9.exe` built with the Windows GUI subsystem, retaining pipe-based diagnostics without a startup console.

Verified schema-1 `.om9` container with bounded 72-byte chunk table, SHA-256, overlap/gap/truncation/overflow/trailing-data checks, streaming JSON node/string/depth/duplicate-key admission before typed decoding; point/rational-curve/mesh/cloud master metadata equality; same-volume temporary write/flush/reread/rename with cancellation, invalid input and readonly-destination preservation. Live model and private verification copies share the budget. Unknown schemas/chunks remain rejected until an explicit migration/adapter exists.

Cold runtime restore increments session/epoch, rebinds only previously verified current stamps, and checks exact solver/version/schema/command/version keys from the executable's registry. Missing modules mark recipes unresolved and last valid results stale; downstream dependencies cannot acquire false clean success. Raw codec decode is format validation, not module activation.

Actual release process receipt: `tools/validate_standalone_foundation.py` created/reopened a four-object authored document in fresh processes and an isolated environment without host paths, compared semantic identity/geometry/attributes/channels, parsed PE imports and observed the live executable. Seven loaded modules, no application children, no FreeCAD/QymCAD/Python imports or loaded modules. [Fingerprints and observations](2026-10-13-standalone-process.json).

The first live audit observed Windows `conhost.exe`: the Rust default console subsystem created an OS console host. Changed the standalone application to the Windows subsystem (appropriate for its upcoming owned GUI), verified pipe diagnostics still work and reran the live audit with zero children. No process was hidden from the audit by a whitelist.

Pending Task 5: geometry/resource/retained blobs and schema migration, structural current-state openNURBS/3DM, clipboard and full mixed CAD/topology/metadata matrix. Current executable is a headless foundation slice; no Matrix9 window/pointer, native CAD or performance gate is closed. The user-cleared build directory no longer contains old SDK/tool caches; native adapters need their own pinned standalone dependency builds.

Next: Task 6 owned scene/cameras/picking/cached affine gestures and alignment of startup Layers to the supplied Matrix9 contract, then actual UI.

## Tasks 6–7 — initial scene and actual native UI slice

RED: startup fixtures exposed eager default Layers and assignment rejection on Off layers; scene fixtures initially had no camera/picking/affine implementation. Additional regressions reproduced overflowing finite-endpoint domain spans returning a false finite Line D0, and an unchanged affine gesture writing Undo and clearing old Redo. Both were fixed. Captured evaluator/session/transcript/widget/F6/focus tests failed before the corresponding APIs and lifecycle changes.

GREEN: `rtk cargo test --locked --manifest-path standalone/Cargo.toml --workspace` — **107 passed (28 suites)**. `cargo fmt --all --check` and workspace/all-target Clippy `-D warnings` passed; actual release executable rebuilt on H:.

Startup document now has `current_layer=None` and no eager group. Unassigned curves use #008255, by-layer output follows assigned layer color, Off accepts hidden output, locked layers reject writes. Camera and CPlane are separate; four named views and maximize/restore preserve independent state. Finite 40×40 minor / 8×8 major construction grids are scene-only and excluded from picking.

Owned display buffers retain point/cloud native indices, mesh polygon face indices and polygon boundary wires. Nested/mirrored occurrence transforms apply once. Held picks reject changed source/camera/viewport/world epochs; visible, selectable, locked and SnapToLocked remain distinct. Analytic circle and nonperiodic NURBS display are bounded initial approximations; NURBS uses 256 samples, without claiming a certified arbitrary-curve chord bound. Affine gestures reuse the same immutable buffer Arc across 100 warm updates, leave the document untouched until acceptance, Cancel resets overlays, and commit is one Undo. Identity acceptance leaves Redo executable. No BVH/GPU affine performance acceptance is claimed.

CurveEvaluator binds admission to an immutable borrow and keeps captured limits. Sampling/refinement reuses this proof; interior multiplicity lookup is logarithmic and mesh triangle picking uses stack coordinates. One curve validation per hover remains; large-scene picking/index/performance work is open.

Actual native executable now opens a winit/eframe/egui 0.36.2 + wgpu window with black four-view canvases, finite grids, Matrix-green CMD, sidebar panel order and the 607 exact feature IDs visible in the catalog. CMD uses one selectable protected transcript: Unicode character ranges cannot edit history/prompt, paste changes draft only, Up/Down restore draft, Enter follows the same dispatcher as menu/F6. Line/Point sessions capture unit/CPlane/context, privately prepare scene before publication, and keep drafts outside the document. BothSides is explicit; specialized branches remain disabled. `BothSides=No` and numeric initial camera choices are labeled OM9 policies, not recovered Matrix9 defaults.

Native event replay used the application's normal widgets, then literal world endpoints [0,0,0]→[12, 6,0], one Undo, actual CMD Undo/Redo and Esc cancellation were verified. A wgpu framebuffer screenshot was inspected. Runtime: RTX 3060, Vulkan, 1280×800, **84 loaded modules, zero children**, no forbidden FreeCAD/QymCAD/Qt/Python host dependency; native root window handle observed. [UI receipt with source/binary/screenshot hashes](2026-10-13-standalone-ui/ui-receipt.json). The rebuilt executable's separate headless cold-reopen audit now observes 30 loaded Windows/UI-link modules, zero children; the earlier seven-module count belongs to the historical pre-GUI executable. Own UI screenshot is a generated build artifact at `build/standalone-validation/ui-workspace.png`, outside the public spec/package.

F6 now persists between frames, has Pin and unpinned Esc/dispatch close behavior; viewport printable input routes to CMD when no other editor owns focus. Full contextual source/selection/session restoration, panel placement/clamping, glyph/icon/docking fidelity and dialogs are still pending. Layers rows/display names are present but controls disabled; full semantic slots/Hidden/Advanced operations, shade/modes/snaps/navigation/Builder are not accepted. Initial egui painter still performs CPU projection/tessellation per frame; it does not satisfy the warm-cache GPU target. CMD's independently pinned live row while history scrolls, autocomplete/options and complete focus/IME matrix remain open.

Public-source audit: 4, 490 files, 511 icon bindings, 510 authored SVGs, **zero errors**. V1 remains immutable. All M01–M12 and feature/capability completion gates stay open for their complete scope. Next: operation-scoped standalone openNURBS build/current-state structural 3DM and remaining document/scene/UI contracts.

## Native dependency probe — selected openNURBS build

Fetched official openNURBS at `eb92af3ba1806b0a34a99aba0d3bda83e3d46083` into the excluded H: build cache. `standalone/native/CMakeLists.txt` checks the pin and builds only the authored probe, `opennurbsStatic` and prefixed zlib. No legacy OM9ThreeDm, FreeCAD, Qt, OCCT or Qym application target is added. CMake was found inside Visual Studio BuildTools, outside PATH. Actual compiler MSVC 19.44.35208.0, v143, Windows SDK 10.0.22621.0, x64 Release/MD. Compilation worker count is floor60% of the 24 logical processors (14), independently of application job policy.

RED: authored structural probe verifier failed before an executable existed. GREEN: the native probe writes a version-5 archive, reopens it in a fresh process, validates five actual native master families (point/line/rational quadratic arc/mesh/cloud), original mesh face indices, ordered cloud points/colors and native domain [2,4]. Rational arc midpoint uses independent 10/√2 oracle at 1e-10, not a screenshot; corrupt archive returns failure without success JSON. Actual PE imports have Windows/MSVC dependencies, no OCCT/Qt/host dependency. [Native probe receipt](2026-10-13-standalone-opennurbs-probe.json) records source pin/clean checkout, license/raw and worktree hashes, CMake/toolchain/profile/source/binary/fixture hashes.

This is **validated_probe_not_integrated**, not an accepted OM9 import/export command or native safety/budget gate. The process owns all native allocations and catches C++ exceptions; general hostile-record peak containment, retained/edited/deleted merge, blocks/attributes/tables/full geometry families and Rhino 5 actual open/edit remain pending. No native operation is enabled in the Rust capability registry. Upstream cloud `m_H`/PointIsHidden is explicitly runtime-only, not saved in 3DM: the eventual field matrix must reject unsupported preservation or implement a disclosed owned metadata policy. Native library version/read success does not prove Rhino 5 compatibility.

## Native owner and identity lifecycle — continued implementation

RED: process-owner tests failed before the owner crate existed. Identity regressions then reproduced reallocation of deleted/Undone UUIDs, reopening a document clone with the old stamp, and two independent restores of the same bytes sharing the same numeric owner counters. GREEN: **122 passed (33 suites)** across the complete workspace; release binaries rebuilt and Clippy passed.

`om9-native-owner` isolates Win32 handle code from safe model/archive/scene crates. Trusted caller admits an absolute executable against an expected SHA-256 and holds its image without write/delete sharing. Each native process is created suspended, assigned to a dedicated Job with committed-memory and active-process limits, then resumed. Only three explicit stdio handles are inherited; environment contains SystemRoot/TEMP/TMP. Native entry cannot precede successful job assignment. Output readers have bounded buffers, explicit stack reservations and allocation checks; Cancel/deadline/overflow terminate and reap the native process, close pipes and join readers. Native exit errors discard stdout as result data and retain bounded stderr. The eight owner tests exercise actual success, fingerprint/limit rejection, early/live Cancel, deadline, stdout/stderr flood, memory allocation refusal, descendant creation denial and native failure.

The openNURBS probe now runs through `om9-native-runner`, rather than directly. Actual write/read/corrupt-read jobs peaked around 34 MB committed memory under a 128 MB cap; each reports exited=true and active_processes=0. [Updated native receipt](2026-10-13-standalone-opennurbs-probe.json) includes runner/source fingerprints, explicit caps and job statistics. This is committed/private native memory containment, not physical-RAM measurement, an OS security sandbox, native memory-safety certification or completion of aggregate document/snapshot/job/scene budgets. The runner is an explicit developer receipt tool; archives cannot select executable paths and no import/export capability is enabled in the app.

The coordinator now owns a sorted immutable identity ledger independent of Undo. New/private candidate ledgers share unchanged allocations, use fallible exact reservation and charge prior/candidate storage before allocation. Insert cannot reallocate an identity previously accepted in public or caller state; Publish replaces existing outputs explicitly. Undo/Redo restore the original snapshots and IDs. Savepoint rollback/abort cannot make a successfully staged identity available for a different object. Rejection snapshots include this authority and old Redo remains executable.

Document/SourceStamp now include a typed random owner-session UUID in addition to monotonic counters. Reopen/cold restore issues a fresh owner and rebinds only complete stamps proved current before renewal; stale references stay stale. Two restores from the same bytes have distinct source stamps even when numeric counters agree. Schema-1 data without the new fields decodes to a legacy nil sentinel, then explicit runtime restore issues ownership and separately gates replay modules. Runtime coordinator creation/open rejects nil ownership. This tested compatibility rule does not imply full container/schema migration completion; retired identities are session authority, and full retained-source tombstones/resource/schema work remains open.

Next: retained 3DM merge and remaining model/UI/scene/budget contracts. All full M01–M12 and feature/capability gates remain open.

## Owned current-state exchange — continued implementation

Current verification: **127 workspace Rust tests passed (37 suites, 7 native lifecycle tests explicitly ignored in this invocation)**; the separate real-native invocation passed all **7 lifecycle tests**, for **134 distinct Rust tests exercised**. Clippy with warnings denied and formatting passed. Build caches, target binaries and pinned native dependencies are back on H:; no Cargo target override redirects them to C:.

`om9-exchange` captures an immutable current document, full source stamps and coordinator epoch. Its bounded schema-1 packet owns current geometry, placement, IDs, layer/definition graph, page units and Unicode labels. Geometry-only export explicitly omits History/results/runtime identity/Undo/document metadata. Before launching native work it counts retained world/Undo/Redo/scope allocations, identity authority, captured stamps, old/new serialized buffers and the native-owner reservation together. JSON responses receive duplicate-key/depth/node/string/point admission before DOM allocation. Concurrent cross-job reservation remains a separate open requirement.

The authored C++ adapter performs actual version-5 native conversion for Point, Line, Circle, NURBS curve/surface, contiguous Polycurve, Mesh triangle/quad, Cloud and shared/nested instances. Canonical geometry is millimeters, page units stay separate; expanded knots map to the compact native vector without losing accepted knot/domain data, rational weights stay homogeneous internally, surface control points remain U-major. Ordinary placement is baked once; shared/nested instance transforms remain native matrices, including mirrored/nonuniform cases. Unsupported selectable/hidden-lock/member-lock/reverse/seam/trim-domain/noncanonical-knot/circle-parameter combinations reject explicitly. Unassigned layers and all-false runtime cloud flags have disclosed native representational conversions. Native normal channels use normalized inverse-transpose, with native mesh single-precision tolerance.

RED/GREEN evidence includes real errors found by fixtures: upstream cloud transform left normals unchanged, mirrored native mesh added an extra normal sign flip, and child-first cached definition masters bypassed a simple recursion depth limit. The authored adapter now handles those cases and validates full definition longest-path depth before native masters and in cold inventory. It checks CRC/critical read errors, exclusive private output, dependency/member/layer graphs and native name normalization. [Native conversion receipt](2026-10-13-standalone-native-conversion.json): **50 bounded owned runs, 40 rejection cases**, twelve objects in the primary cold fixture, archive version 50, no active process/descendant after return; source/dependency/toolchain/binary hashes and PE imports recorded. No FreeCAD/Qym/Qt/OCCT host is linked.

Rust independently compares the cold on-disk geometry, native master type, transform, ID, attributes, definitions, layer order/units against captured typed sources. Private output is synced and hashed before/after inspection. Final publication rechecks whole-source epoch/Arc/stamps, cancellation and staged bytes, then uses same-volume rename; any preacceptance failure keeps the good destination and document/Undo/Redo. [Rust exchange receipt](2026-10-13-standalone-exchange.json): 16 real write/inspect jobs cover all current master variants, oblique/nonuniform and mirrored normal channels, periodic NURBS, nested/shared references, current edit/delete/new output, commit+Undo staleness, owner close, late cancellation, readonly destination, changed staging, unsupported fields and prelaunch budget rejection. These tests exercise the authored conversion and owning coordinator, not an echoed fixed native fixture.

The pinned JSON 3.11.3 header/license are SHA-checked by CMake, with the MIT notice under `Resources/licenses/nlohmann-json.txt`; openNURBS source remains clean at its official pin in the excluded build cache. `standalone/provenance.json` records the selected native closure and owned process boundary. Neither this library nor the developer native runner is enabled as an application import/export command. Retained unsupported records/merge/tombstones/import namespace, full `.om9` chunk migration/resources, structural clipboard, Rhino 5 actual open/edit and all complete M01–M12/UI/performance gates remain open. No feature/capability status is promoted from this slice.

## Supplied Matrix9 interface and final review fixes

Current verification supersedes the earlier invocation counts above: **158 workspace Rust tests passed (41 suites, 7 native lifecycle tests ignored here)**, plus the separately invoked **7 actual native lifecycle tests**, giving **165 distinct Rust tests exercised**. Formatting and Clippy with warnings denied passed. Release workspace/app binaries use the H: target; native dependencies and builds stay on H:.

The supplied Matrix9 screenshot governs the compact gray left sidebar, green CMD, top toolbar tabs and four black finite-grid views. Looking Down/Perspective are top-left/top-right; Through Finger/Side View are bottom-left/bottom-right. Through Finger uses global XZ, Side View global YZ; active startup view is Perspective. Only positive CPlane axes are colored from the origin. The 300-point sidebar opens with the native window maximized; the bounded replay uses 1600×1000. Authored SVG textures replace placeholder letter icons. [Icon source receipt](2026-10-13-standalone-icons.json) verifies 58 existing authored assets and pinned resvg 0.45.1 without default font/raster decoder features. No source screenshot bitmap is embedded.

CMD has one protected Transcript and one native TextEdit state. Its live row is pinned and unwrapped; history scrolls independently, while mouse selection and Ctrl+A/Copy/Cut span the same logical text document. Regression RED/GREEN reproduced near-64KiB Unicode history retirement causing stale layout byte slicing, and IME provisional Point/coordinates being executed by Enter or canceled by same-frame Preedit/Escape. The layout derives its split from the current text on each call; public IME event lifecycle prevents these command actions. Real egui pointer tests cover Unicode suffix paste, both directions of cross-region selection, actual history wheel scroll, numeric-draft Enter after viewport selection, disabled Snap actions, domain-specific tool palettes and Help-only entries. Narrow sidebar/toolbar allocation tests pass. Fresh review found no remaining concrete Important defect in this slice; actual OS IME, DPI/multi-monitor and full pixel parity remain unmeasured.

The native UI receipt replays accepted literal Line endpoints, one Undo, actual Undo/Redo and Esc through normal widgets in a real winit/wgpu window. Screenshot and source/binary fingerprints are recorded; 84 loaded modules and zero children were observed. File dialogs, semantic editable Layers/Projects/Builder, complete F6, CAD display modes and GPU scene performance remain open. These tests do not enable any additional catalog operation.

Additional native publication hardening uses VerifiedFile: open and deny write/delete sharing, verify a regular non-reparse same-volume file, hash the held handle, and publish via that same handle. Eight actual Windows ownership tests cover held-file mutation rejection, final destination replacement, cancellation, reparse and failure cleanup. Exchange capture/source/cancel checks remain in front of publication. Aggregate retained accounting now includes fixed storage, spare Vec/String capacity, conservative pinned-standard-library BTree allocation bounds and the Redo+new-Undo peak. Rejection tests retain executable old Redo. This is admission accounting, not physical-RAM measurement or completion of a concurrent global reservation pool.

The conversion batch still has **50 bounded native jobs / 40 rejection cases** and the current-state Rust lifecycle batch **7 tests / 16 native jobs**, with version50 cold correspondence and zero active processes after return. Remaining prepared/unclosed native packages are retained merge/import namespace/tombstones, full archive resources/schema migration, structural clipboard and actual Rhino5 open/edit; the total future batch count remains unknown. Next exact document: `OpenMatrix9_Codex_Spec_v2/specs/00-rhino-core/10-persistence-3dm-clipboard.md`. All complete M01–M12 and 607-feature/148-capability gates remain open; the full RCORE-00 goal is active.

## Embedded resources and explicit schema upgrade continuation

Current verification supersedes earlier invocation counts: **186 workspace Rust tests passed (44 suites, 7 native lifecycle tests explicitly ignored here)**, plus **7 actual native lifecycle tests**, for **193 distinct Rust tests exercised**. Formatting, Clippy with warnings denied and the release workspace build passed on H:. The UI replay still verifies four lifecycle checks, a native root window, 1600×1000 framebuffer, 84 live modules and zero children; the cold headless audit observes 30 modules and zero children.

Safe Rust `Document.resources` now owns ResourceIds and immutable embedded payloads. Schema 2 stores descriptors in typed metadata and bytes in checksummed binary chunks; exact ID/length/hash correspondence, empty payloads, chunk tiling, missing/orphan/duplicate resources and typed UUID canonicalization are enforced. Ordinary History option names remain ordinary data. Resource insertion/deletion participates in the global coordinator with initial batch revision checks, one Undo/Redo, world-wide live ownership and retired ResourceIds surviving Undo/savepoint rollback.

The developer `--migrate-v1` path explicitly upgrades schema 1 while preserving durable IDs and establishing empty resources. Default open rejects schema 1. Runtime restoration is admitted before the private clone or file replacement; missing modules produce unresolved diagnostics, and counter/budget failures preserve the original file. Windows save fingerprints and atomically publishes through the same held file handle. Report normalization removes only actual session owner fields, preserving recipe parameters named `epoch` and other ordinary option keys.

[Resource/schema receipt](2026-10-13-standalone-resources.json) records **35 scoped Rust tests**, including four actual cold-process receipt cases: a 32 KiB payload after removing its original file, explicit migration preserving UUIDs/point data and unresolved status, counter overflow preserving the schema-1 file with no success output, and distinct persistent fingerprints for a changed recipe parameter. The reviewer reproduced an aggregate restoration allocation defect, then verified the fix with the same 75, 545-byte limit: measured new heap peak reduced from 95,975 to 35, 575 bytes before rejection; a successful admitted case stayed within its separate 200,000-byte limit. This is a scoped allocator probe, not complete M07 certification.

This closes the implemented blob/schema substrate only. Native retained-source namespaces/bindings/tombstones/current-state merge, full material/annotation/page/camera resource families, external missing/relink behavior, structural clipboard, exact cross-snapshot payload allocation deduplication, global reservations and actual Rhino 5 interoperability remain pending. M01–M12, 607 feature and 148 capability completion statuses remain unpromoted; the full RCORE-00 goal stays active.

## Source namespace ledger and schema-3 continuation

At this source-ledger checkpoint: **205 workspace Rust tests passed (45 suites, 7 native lifecycle tests explicitly ignored here)**; the separate owned native lifecycle batch was **7 tests**, giving **212 distinct Rust tests exercised**. Clippy with warnings denied and formatting passed. The release workspace was built on H:. Later verification is recorded in the continuation below.

`SourceId` and fresh `ImportNamespaceId` now live in the owned document. A source links immutable resource/hash/version and original resolved model/page units/tolerances. Native record keys remain namespace-local provenance. Bound root objects, layers, definitions and definition members have streaming host fingerprints and sticky Unchanged/Edited/Deleted slots; retained-only source records create no fabricated geometry. Atomic import bundles publish all resources/source/namespace/geometry/layers/definitions in one coordinator entry. Source/namespace/layer/definition authority and tombstoned object IDs stay retired across Undo, caller savepoint rollback and cold reconstruction.

The final candidate reconciliation catches direct geometry/placement/attribute changes and `PreparedResult::apply`. Deleted targets remain absent at every root/member location, and duplicated retired member IDs cannot claim another parent definition. Resource deletion rejects a live source or tombstone reference; retained cross-document transfer rejects until full provenance/dependency transfer exists. Unknown unit factors require explicit resolution and canonical tolerance underflow/overflow rejects before mutation. Failed/stale/canceled/guard-rejected import retains prior state and executable Redo.

Schema 3 persists this ledger. Explicit schema-2 upgrade preserves resource bytes, hashes and durable IDs while creating empty provenance; source records cannot be injected into legacy schema 1/2. The actual cold process proves edited A/deleted B/new C and original source units survive reopening. [Resource/source/schema receipt](2026-10-13-standalone-resources.json) covers 54 scoped Rust tests and six actual cold-process receipt cases; these establish structural ownership, not native source-to-host correspondence or a merge writer.

Review found two aggregate allocation gaps and verified their fixes. A 2 MiB resource/2,048-record validation formerly accepted a 2, 450,070-byte limit at a measured 2, 663,798-byte owned heap peak; it now rejects before set growth at 2,425, 694 bytes. CLI report formerly copied a 50,000-record ledger into multiple JSON trees and buffers, exceeding the default byte budget; borrowed serializers now stream the diagnostic report and persistent hash after admission. The fingerprint algorithm is explicitly `om9-persistent-projection-v2`; only actual runtime owner fields normalize, while ordinary recipe options remain data. The hash is a diagnostic projection, separate from archive/source payload fingerprints.

The same large-ledger report probe measured 388,816,389 bytes before the streaming fix and 65, 498,470 bytes afterward. A separate 20 MiB held-diagnostic probe with an 80,706,478-byte limit formerly accepted at 86,464,182 bytes; report now admits diagnostics/buffer storage before document validation and rejects at the already-held 80, 673, 654-byte input peak. These are scoped owned-heap probes, not full global allocation or M07 acceptance.

Full native import admission, retained-only object interaction, current-state native preservation merge/remapping, full resource families, clipboard, performance, global sharing/reservations and Rhino 5 interoperability remain required. None of the complete M01–M12 or 607-feature/148-capability gates is promoted by this continuation.

## Strict source inventory, native circle correspondence and bounded owner

Latest verification: **221 workspace Rust tests passed (46 suites, 7 native lifecycle tests explicitly ignored here)**, plus **7 separately invoked actual native exchange lifecycle tests**, totaling **228 distinct Rust tests exercised**. [Workspace receipt](2026-10-13-standalone-workspace.json) fingerprints the actual test sources/log. Formatting, warning-strict workspace Clippy and the release workspace build passed on H:. The refreshed native window/framebuffer replay retains the supplied Matrix9 compact gray sidebar, green CMD, authored SVG icons and exact four-viewport order; it observed 86 loaded modules and zero children. The headless cold audit observed 30 modules and zero children.

[Strict native source inventory](2026-10-13-standalone-native-source-inventory.json) records **68 bounded jobs / 17 rejections**, with a measured peak of 81, 641,472 committed bytes. Schema-2 DTOs preserve registered component keys/kinds/relations, original model/page/definition unit contexts and tolerances, actual source geometry projections, userdata descriptors and anonymous user-table inventory. Actual fixtures include mm/inch/custom/None/Unset, 24 objects with local definition members, BRep retained-only dependency propagation, unrelated supported projection closure and nondefault native text/DimStyle. Low-level table completeness rejects unknown class/table skips, CRC/recovery errors, missing/cyclic dependencies, damaged endmarks, trailing bytes and recursive curve point budget overflow. SDK-decoded indices are explicit; original source bytes remain authoritative. Exact projection requires independent host attribute/dependency/unit admission and claims neither host readiness nor native import/edit capability.

Owned Circle now stores optional angular interval separately from its native parameter domain. Missing/null preserves radians. Native writer and cold source inventory preserve phase/custom parameter correspondence with exact full angular span; an actual near-full ON_ArcCurve for which SDK IsCircle returns true remains retained-only. Independent cold ON_ArcCurve D0–D3 samples at native 11/14/17 and angular [1,1+TAU] match analytic oracles, including uniformly scaled mirrored placement. Fractional evaluation composes the angular map before derivative powers and uses original fraction, avoiding intermediate scale overflow and phase loss in adjacent-f64 domains. Scene normalizes sample fractions before native interpolation, preserves reproducible sample/pick parameters across [0,1e308], and explicitly withholds unresolved f64 bindings with a visible precision diagnostic. Durable geometry remains intact.

Review reproduced a Windows accounting race and then an unbounded reader-join gap: a 200 ms run + 50 ms teardown waited beyond 450 ms for a suspended output reader. Native owner now uses exclusive pipe pumping with available-byte reads and one stored teardown deadline for process exit, pipe closure and actual job accounting; Drop never waits or resets that budget. **28 owner tests** include deterministic delayed/late accounting, actual pending pipe reads/overflow, failed-stop/no-extension cases and **128 barrier-started concurrent short success/error jobs**. Independent review's corrected hold probe returned at **219 ms**, exited with observed active_processes=0 and no reader thread. The resource containment scope does not claim OS scheduling guarantees, security sandboxing, native memory safety or a concurrent global reservation pool.

The refreshed original conversion batch passed **50 actual jobs / 40 rejections**, peak 34,439,168 bytes, and Rust-owned current-state export passed **7 tests / 16 native jobs**. Five-family native dependency probe and source/binary fingerprint checks passed. Resource/schema-3 evidence remains **54 scoped Rust tests / 6 cold-process receipt cases**. No native-to-host import, preservation merge, structural/OS clipboard or actual Rhino 5 acceptance is promoted. Source snapshot binding, explicit unresolved-unit re-projection, full canonical native relation mapping, attribute/default policy and pre-job target capture are the next concrete implementation tasks. Full M01–M12 and the original 607-feature/148-capability acceptance gates stay open; the RCORE-00 goal remains active.

## Actual source import, durable definition units and guarded ownership

Latest verification: **260 workspace Rust tests passed (54 suites), 7 actual native export tests and 8 actual native import tests: 275 distinct tests exercised**. Fifteen native tests are explicitly ignored only in the portable workspace run and separately executed. Resource54 and its six cold-process cases are subsets, not additional distinct tests. Formatting, warning-strict workspace Clippy and Release build passed on H:. Actual headless30/UI86 module observations still have zero children; the UI framebuffer keeps the supplied Matrix9 layout.

[Actual import receipt](2026-10-13-standalone-import.json) measures **15 source files/8 tests/0 failures**. The SDK fixture produces mm/inch/custom/None/Unset, mixed definition units, unresolved closure, complex preserved fields and parent/material color selectors. Rust captures an immutable source under held read handles, writes a private synced snapshot/resolution packet, checks the independently read native SHA-256, validates strict typed relation/projection coverage and bounded acyclic dependencies, and admits represented geometry with fresh source/namespace identities. Shared members remain local; layers retain decoded order. Default Preserve keeps uncovered records retained-only. Explicit Geometry reports omitted bound object/member/layer fields; parent/material color loss is named. Original bytes and original/resolved model/page/definition contexts survive a real fresh-process .om9 open after the source was deleted.

Target/caller capture freezes coordinator authority, epoch, caller generation/version and immutable document identity/revision. Original cancellation cannot be replaced by another token; the bounded union remains checked through commit. Plain operations reject active caller scopes; scoped imports prepare/publish only through their original caller. Private scene and external receipt storage are admitted before publication. Review reproduced foreign equal-counter coordinator publication, then verified the fix. Three additional export regressions independently reject foreign equal-counter/shared-document origins, an aborted caller roundtrip with unchanged public epoch, and an active caller bypass. Review also reproduced original export cancellation substitution with an actual native file; its refined existing lifecycle case observed RED then GREEN after binding the original token through held-file hashing and rename. Rejections preserve public state, the good destination and executable Undo/Redo.

[Native projection](2026-10-13-standalone-native-source-projection.json) has **82 owned jobs/32 expected rejections**, peak 36, 450,304 committed bytes. Schema3 does not assert host readiness: it reports actual source hash, units/resolution, numeric SDK component types, canonical placements and concrete default/nondefault classifications. Original fields/tolerances remain original metadata. Native definition bbox is its actual cache in native units, not an invented canonical box. Projection cannot serialize opaque table writers without full context; immutable source bytes own those bodies. Compatibility batches freshly pass **inventory 68/17 rejection**, peak 79,802,368, and **current conversion 50/40 rejection**, peak 34, 525,184. The existing Rust export lifecycle batch remains7/16 owned jobs. Every recorded native operation is exited with observed active_processes=0.

This is a selected synchronous import/library slice. The app import dialog, general native CAD/annotation/page/material/history/resource editing, retained-only scene interaction, full asynchronous/global reservations, current-state retained merge/remapping/export, structural/OS clipboard, exact payload sharing, GPU warm-cache performance and actual Rhino5 interoperability remain required. No complete M01–M12/607-feature/148-capability gate is promoted. Zero standalone prepared batches remain pending at this checkpoint; broader seven openNURBS packages and12 full RCORE gates remain unclosed, with total future batches unknown. Next exact work: current-state source merge with edited/deleted/new records and dependency/material/namespace remapping under owned native staging. The full RCORE-00 goal remains active.

## Original-wire/current-state preparation checkpoint — 2026-10-14

Latest verification is **313 workspace tests/61 suites plus7 native export,
8 native import and3 retained-input tests:331 distinct Rust tests exercised**.
The portable run explicitly ignores those18 native tests; all18 passed in
separate measured batches. Resource62 and its six cold-process receipt cases
are subsets. The resource recorder initially expected15 retained tests while
the suite passed16; the recorder was corrected and expanded to include the
peer-owner and JSON allocation probes. Formatting, warning-strict workspace
Clippy and Release build passed on H:. The rebuilt application again passed
headless30/UI86 loaded-module observations with zero children and the actual
1600x1000 four-view Matrix9 framebuffer/widget lifecycle checks. Its temporary
validation profiles now also stay under H:/FreeCAD-src/Mod/OpenMatrix9/build.

The [retained-input batch](2026-10-14-standalone-retained-inputs.json) verifies
three Rust tests,13 authored files,10 source scenarios and32 owned native jobs,
including nine preparations. Rust stages the exact immutable source bytes,
holds a read-only file while the native process runs, admits process/DTO/maps
with document/Undo/external owners, and checks the complete original source,
record and start/table/end hashes plus the exact decoded catalog. Model/page/
definition declarations correspond to the persisted source; the typed decoder
distinguishes known versus unknown userdata and retains copy-count-zero data.
The original operation's Cancel remains bound after preparation. Failed sharing
compatibility and late-Cancel substitution regressions were observed and fixed
without changing document state or executable Undo/Redo.

Native source preparation uses one held FILE for strict recursive scan,
reference mapping, original SDK owners and final byte validation. Decoded keys
remain separate from serialized/reference/SDK indexes: sparse Layer7/1/1,
Group5/5/0 and Light7/0/7 with no decoded Light index. V5 definitions serialize
no index and refer to their actual member UUIDs. The original legacy DimStyle
owner keeps an attachment that decoded style conversion discards. The final
[native preparation/infrastructure checkpoint](2026-10-14-standalone-retained-infrastructure.json)
records317 owned executions/123 expected rejections on one native binary.
Fresh current-runner compatibility also passed inventory 68/17, projection 82/32
and current conversion 50/40; their receipts include the new retained header.

The current Rust overlay owns durable Preserve/Geometry policy, independent
geometry/placement/attribute/label baselines, current edit/delete/new actions,
logical native UUID/decoded-index intent and final physical unit frames.
Revision-only changes that restore original fields retain native data; unknown
legacy policy or missing edited baselines refuse. Geometry mode cannot revive
retained-only records. Indexed dependency-cache propagation crosses retained-only
ancestors while unrelated definitions remain clean. Own Iref placement/child
changes also mark refresh even when its child definition is unchanged.

Measured allocator regressions reserve all peer document, Undo/Redo, external
held and incoming request owners before validation scratch. Shared JSON
admission reserves growing escape scratch, both key copies and duplicate-key
BTree storage before allocation. Near-limit escaped values, escaped keys and
tiny-map refusals now stay within their admitted peaks. Normal Arc-owner
deduplication uses an admitted index; the near-limit allocation-free fallback
can be quadratic in snapshot count and establishes no performance gate.

This checkpoint has **no production retained action writer or output wire map**.
Actual output table order/reference indexes, secondary metadata and opaque
remaps, selected cache-preserving serializers and an independent cold current
geometry/bounds oracle remain required. ON_InstanceRef.GetBBox reads its cached
box and cannot be that oracle. Full native families, resources/clipboard,
asynchronous global admission, heavy performance and actual Rhino5 acceptance
remain open. All12 complete RCORE-00 gates and the607/148 catalogs remain
unpromoted; seven broad openNURBS packages remain open and the total future
batch count is unknown. Next: closed Rust output wire/reference allocation,
followed by the schema4 current-state retained writer and cold output proof.


### Original wire additions — 2026-10-14

Native input/addition batch: 4 Rust tests, 15 authored files, 11 source scenarios, 36 bounded native runs and 10 retained preparations passed. New Layer serial8/reference2, generated native-only default serial9/reference3 and new Definition ordinal4 follow all four original definitions, including the unbound orphan. Geometry root/instance/local-member UUIDs occur once without invented component indices. Evidence: [retained inputs](2026-10-14-standalone-retained-inputs.json).

Independent wire review: 4 tests pass; generated base/suffix layer names now obey the configured byte limit before allocation, with original/fresh cancellation polling. Earlier failed addition assertions used a clean fixture that removes the sparse child and assumed three instead of four retained definitions; failure logs remain in build/standalone-validation. No native producer failure occurred in these attempts. This is output intent, not native writing or full RCORE-00 completion. Retained writer/cold output checks are next; packages 2–8 remain open and total remaining batches are unknown.


### Native retained-current adapter — 2026-10-14

Frozen adapter binary 92b71b4: selected 99/32 expected refusals (8 SDK source files, 21 successful outputs, 5 authored cache-negative files, 53 scenarios); conversion 50/40 (5 output files, 45 scenarios); inventory 68/17 (18 inputs, 6 outputs, 28 scenarios); projection 82/32 (22 inputs, 49 scenarios); preparation 65/20 (28 inputs, 32 scenarios). Total 364 owned jobs / 141 expected refusals; all children reaped. Files/scenarios/executions are distinct: selected 34 native-produced paths plus one staged copy are not 34 scenarios.

Selected writer/cold SDK tests cover A edit/B deletion/C addition, appended layers/definitions/members/mirrored instances, shared/nested cache arithmetic, typed secondary mm/inch regeneration, raw userdata/copy-count-zero chunks, decimal V5 mesh and smaller exact EOF after deletion. Cold raw_spans expose actual held output hashes/ranges for independent Rust checks. Receipts: 2026-10-14-native-retained-current-{selected,conversion,inventory,projection,preparation}.json. General opaque semantics, tight CAD bounds and Rhino application acceptance are unproved. Root Rust independent cold/current capture acceptance and atomic publication remain in progress. Full RCORE-00 and packages 2–8 remain open; total future batches unknown.


### Native retained-current adapter 2a6ca0e — 2026-10-14

Frozen adapter binary 2a6ca0e: selected 107/34 expected refusals (8 SDK source files, 23 successful outputs, 7 authored cache-negative files, 57 scenarios); conversion 50/40 (5 output files, 45 scenarios); inventory 68/17 (18 inputs, 6 outputs, 28 scenarios); projection 82/32 (22 inputs, 49 scenarios); preparation 65/20 (28 inputs, 32 scenarios). Total 372 owned jobs / 143 expected refusals; all children reaped. Files/scenarios/executions are separate counts. The selected batch grew by two successful files and two valid stale-cache negative files after actual tiny-coordinate/huge-instance and translated thin-hull regressions exposed the absolute tolerance floor.

Selected writer/cold SDK tests cover A edit/B deletion/C addition, appended layers/definitions/members/mirrored instances, shared/nested cache arithmetic, typed secondary mm/inch regeneration, raw userdata/copy-count-zero chunks, decimal V5 mesh and smaller exact EOF after deletion. Cold raw_spans expose actual held output hashes/ranges for independent Rust checks. Receipts: 2026-10-14-native-retained-current-{selected,conversion,inventory,projection,preparation}.json. General opaque semantics, tight CAD bounds and Rhino application acceptance are unproved. Root Rust independent cold/current capture acceptance and atomic publication remain in progress. Full RCORE-00 and packages 2–8 remain open; total future batches unknown.


### Rust retained-current cold/publication — 2026-10-14

Matching frozen native2a6ca0e and Rust sources passed4integrationtests,12SDKsources,21privateoutputs,21semantic scenarios and78ownednativejobs;13outputs were atomically published,7explicit publication refusals preserve oldtarget/current coordinator snapshot. Formatting, workspace Clippy-Dwarnings and releasebuild pass;342workspace tests/68suites plus23separate native tests=365distinct Rusttests exercised. Resource62/sixcold cases are subsets. Original external sources are absent before retained prep/write/cold; mm/inch/custom/mixed/None/Unset declarations, raw tables/records, current edit/delete/new, mapped frames and dirty conservative bounds have scoped correspondence. Tiny and translated thin cache regressions reject. Evidence:2026-10-14-standalone-retained-current.json. Selected codecs only; general opaque remap/native families/clipboard/global concurrent budgets/heavy GPU/Rhino5 and all M01-M12 remain open. Prepared standalone pending matrix is not fully enumerated; packages2-8 are open and total future batches unknown. Next:expand native family/field/remap admission with actual source/cold matrix.


### Six Matrix9 screenshot specification update — 2026-10-14

Six user-provided reference PNGs were analyzed as S1-S6; only authored text/hash metadata enters the repository. MATRIX9_SCREENSHOT_DETAILS_2026-10-14.md records sidebar categories/chrome, layer two-column rows, independently scrolling Job Bags and stage snapshots, CMD Box prompts/cache feedback, per-view CAD displays/active highlight, and the full title-menu label/order/separator/submenu model.15feature specs/catalog JSON-YAML/lookup/core requirements/migration ledger are linked;12M9IMG acceptance cases remain unchecked. Glyph/submenu/plugin/default/DPI unknowns remain explicit and prior approved Layers/footer/startup/Box rules persist. No images or commercial icons copied, no feature/capability/runtime status promoted. Validator:tools/validate_matrix9_screenshot_specs_20261014.py; original665v1 files remain byte/hash-identical. Full RCORE-00 goal remains active.

## Matrix9 S6 viewport title menu — 2026-10-14

360 workspace Rust tests plus 23 separately executed native integration tests (383 distinct); 17 menu tests are included in the workspace total. Six native menu replay checks pass; 1600×1000 framebuffer inspected, 84 loaded modules and no children. Maximize/restore and Wireframe are scoped; unavailable modes/submenus remain disabled. Clone/fresh-layout aliases, keyboard ownership, outside-click and stale-Escape regressions were reproduced then verified. [Details and open gates](2026-10-14-viewport-title-menu.md). Full M01–M12/607 features/148 capabilities remain unpromoted.

Historical check outside this slice: `tools/validate_rhino_foundation_specs.py` still targets the removed `ref/matrix9/OpenMatrix9_Codex_Spec_v1` location. A read-only routing probe against the immutable root v1 also found its pre-existing unquoted Box colon invalid for PyYAML (line 13097); v2 has no historical `RHINO5_FOUNDATION.md`. No baseline bytes or validator behavior were changed. The active six-image v2 validator passes, including JSON/YAML equality, references and v1 hashes; the old normalized-v1 validator is not counted as a passing check.
