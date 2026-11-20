# Metal Weights source routing

The Tools add-on previously measured selection display meshes and emitted mass even
for an open mesh. The host now routes the Metal Weights add-on icon and command to
the existing BRep measurement and density registry. `gvMetalWeights` is added to
both manifests and the built-in parser; the add-on without that host service refuses
instead of reporting mass from a display mesh. The BRep adapter checks OCCT validity
and closed solids before reporting. Failed measurement leaves the previous report,
document and Undo intact.

## Evidence, 2026-11-17

- `cargo test -p om9-addon-tools --lib -j 2`: 12 passed, including refusal for
  closed and open display meshes.
- `cargo test -p om9-app --test metal_weights --test addon_session -j 2`: 7 Metal
  Weights and 13 add-on-session tests passed; 2 add-on tests were ignored (the native
  one was run separately below). [Host log](../../build/standalone-validation/ui-artifacts/metal-source-route-20261117-a/host-tests.log).
  Seven Metal Weights fixtures cover closed BRep volume/mass/DWT, a translated box,
  two material densities, missing density, open shell and assignment refusal. The
  formerly stale BRep fixtures were updated to the current reserved source and
  representation path.
- The add-on controller fixture uses a 10 × 20 × 30 mm box: 6000 mm³, 93 g at
  15.5 g/cm³ and 124.8 g at 20.8 g/cm³. It compares the add-on icon, typed alias
  and add-on-disabled fallback, Light/High detail, non-solid refusal, report
  preservation and unchanged document/Undo. An additional fixture rejects an open
  BRep through the enabled add-on route.
- Built `om9-addon-matrix --lib` in `L:/om9-target` and froze a debug DLL with
  337 exported commands. SHA-256:
  `298543e6723ae8a2dfd5e99215ce10d20d4b924cf2eb7dc9daa6afb6216c4b89`.
  The ignored native-package test was explicitly run with that DLL and passed:
  [run log](../../build/standalone-validation/ui-artifacts/metal-source-route-20261117-a/native-test.log).
  It installs and enables a temporary native package and exercises the controller;
  it does not alter the user's installed add-on or test a desktop window. A first
  attempt used a DLL overwritten by Cargo's test feature variant, which registered
  only two commands; freezing the standalone build resolved the fixture mismatch.
- The targeted Matrix package manifest/help consistency test passed. The full
  package test had 3 passes and 1 failure because its generated README uses different
  Solid-menu positions from the existing README; that unrelated drift was not
  changed. [Package check](../../build/standalone-validation/ui-artifacts/metal-source-route-20261117-a/package-check.log).

The source-routing validation Cargo commands used `CARGO_TARGET_DIR=L:/om9-target` and `-j 2`. Source commit
at start: `fea59b67f4284d0a3d4545e7b90e7e5ecd7f64fd`; the implementation and plan
were uncommitted in the working tree. The tests are scoped to the fixtures above.

The Matrix material browser/Calculate UI, object-material binding, input selection
workflow, mesh-source measurement, instance selection integration, asynchronous measurement, cancellation,
large-object budgets, real desktop/GPU and human review remain open. Existing native
and viewport paths are reused; this test does not establish general BRep validity or
performance for all shapes. Selected non-BRep objects are refused rather than
silently omitted. No manual acceptance gate is signed.

## Instance follow-up investigation

An experimental nested-instance fixture using a canonical box could not create a
controller: scene capture returned `Missing("BRep representation requires rebuild")`
before selection or measurement. The experimental implementation and fixture were
removed; this is not evidence that instance measurement works. A future fixture
must publish account-owned representations along with its definition and occurrence,
then exercise both placement composition and all-or-nothing refusal of mixed members.
The document's `RenderMaterial` stores a name and color, not mass density, so it
cannot by itself supply the requested object-material binding.

One interrupted test command accidentally used the existing checkout `target/`
before the corrected command used `L:/om9-target`; both used `-j 2`. The existing
checkout target was not deleted because it predates this attempt and may contain
user-owned files.

## Source occurrence service follow-up

The host measurement now reads canonical BRep source directly and traverses nested
definitions with composed placements. Each occurrence contributes its source volume
multiplied by the absolute affine determinant; this counts overlaps separately and
uses the material assigned to the selected root. Unsupported/open members, missing
definitions, cycles, empty instances, invalid transforms and traversal limits refuse
the entire result. The existing controller publishes a report only after every root
is measured successfully.

`cargo test -p om9-app --test metal_weights -j 2` passed 9 tests on
`L:/om9-target`. The source-service fixture uses a 100 mm³ OCCT box twice inside
nested definitions, one mirrored and scaled by 6, for 700 mm³. It exercises open
and unsupported members, cycle, node and depth refusals. A separate controller
fixture checks a mirrored/scaled root gives 600 mm³ and 9.3 g without changing the
document or Undo. The earlier add-on-session suite passed 13 tests with 2 ignored
after this implementation. The instance fixture calls the same service directly;
it does not prove selectable instance setup, UI routes, Light/High or report/Undo
behavior for an actual instance. Native operations remain synchronous and their
peak allocation and cancellation are not established by these tests. Public-source
audit passed (16,130 files, 594 bindings, 593 SVGs, zero errors).

## Controller instance routing follow-up

With the opt-in `test-fixtures` feature, a fixture-only helper inserts definitions
and root instances through normal transaction preparation and candidate-scene
publication. Its BRep comes from the real Box command and its account-owned
representation. The headless controller selects a nested instance containing two
placed occurrences, obtains 700 mm³ and 10.85 g from the menu and `gvMetalWeights`,
and obtains the same report after switching to High viewport detail. A selectable
instance containing a point is refused, preserving the report, document and Undo.
The enabled in-process Matrix add-on icon and alias also report the correct 100 mm³
for a separate selected instance without document or Undo changes.

`cargo test -p om9-app --features test-fixtures --test metal_weights --test addon_session -j 2`
passed 10 and 14 tests respectively, with 2 add-on tests ignored. The initial
fixture used a manually registered BRep with no topology attestation, so changing
detail correctly refused; replacing it with the Box command allowed the intended
Light/High check. These are headless fixtures; they do not establish a desktop
mouse click, native-package instance routing, persistence, per-member object
materials, asynchronous cancellation or general memory bounds.

## Native instance and registry comparison follow-up

The explicitly invoked native Matrix package fixture, using the frozen DLL identified
above, now also selects an admitted BRep instance. The enabled native package icon
and `gvMetalWeights` both report 6000 mm³ with no document or Undo change. It does
not exercise a desktop window or the user's installed package.

An attempted save/reopen of the nested controller fixture stopped at save with
`Unsupported("BRep archive migration to v5 is not available")`; no reopen was
claimed. This is the existing archive migration gap, not evidence of instance
measurement failure.

`MetalWeights all` and `gvMetalWeights all` compare all currently registered
densities, measuring each selected source once and retaining a per-object breakdown
for each density. The command rejects conflicting `all` and `material=` options;
an invalid selected shell leaves the previous catalog report and document/Undo
unchanged. A two-box fixture totals 300 mm³ and 3 g at an explicit test-only
10 g/cm³, with DWT checked. The registry currently mixes built-in and user entries:
this command does not implement Matrix's separate GV/User toggles, the material
browser, persistent user catalog, object-material binding or worker/cancellation.

The opt-in fixture suite passed 11 Metal Weights and 14 add-on-session tests, with
2 ignored; the native package test was explicitly run separately and passed. All
Cargo checks used `L:/om9-target` and `-j 2`.

## Partial builder panel follow-up

Menu/add-on dispatch or an unqualified command now opens a Metal Weights builder
even with an empty selection. The panel offers all registered materials or one
registry material and a Calculate button. The button and typed `Calculate` use
the same controller-owned material choice; the panel opens the existing Builders
area and scrolls the sidebar there. Closing the panel or canceling dismisses it.
This is transitional: opening with a preselection still immediately calculates
the default material, and the selector is not the Matrix GV/User/object-material
interface. Source work is still synchronous.

Headless egui input opened the builder with no selection, selected a 10 × 20 × 30
box, clicked Calculate, checked the 6000 mm³/93 g catalog result, then chose
platinum through the controller and submitted `Calculate` for 124.8 g. Document
and Undo stayed unchanged. The test exercises widget input but no desktop/GPU
presentation; it does not click the material dropdown. With `test-fixtures`, the
Metal Weights, add-on session and egui targets passed 11, 14 (2 ignored), and 1
test respectively. GV/User provenance and filters, object-material binding, panel
presentation and lifetime, full selection/report UX, worker/cancellation and
the preselection behavior remain open.

## Explicit Calculate follow-up

Opening the Metal Weights menu, enabled add-on icon, fallback icon or unqualified
command now only opens the builder, regardless of preselection. The user chooses
materials and invokes Calculate; typed `Calculate` shares the same controller
state and service. The explicit shortcuts `MetalWeights all`, `material=...` and
`MetalWeights calculate` still request a calculation directly. Opening a panel
does not discard an earlier report; invalid input is refused on calculation.

With `test-fixtures`, 11 source/command fixtures and 14 add-on-session fixtures
passed (2 ignored), together with the headless egui builder test. The latter now
clicks the material dropdown and selects Platinum 950 before typed Calculate.
The frozen native Matrix DLL package fixture was explicitly rerun and passed for
both a selected box and a selected instance using the Calculate step. The targeted
Matrix package help/manifest consistency test passed after updating its generated
help and both add-on manifests. No desktop or GPU presentation was tested.

The GV/User material origin filters, object-material density binding, full panel
behavior, source mesh policy, cancellable worker and BRep archive migration remain
open. The selected material is session state; no persistence is claimed.

## Object-material binding follow-up

The builder now offers **Use Object Materials** and explicit session-local links
from document render-material IDs to registry density IDs. Names and colors do not
imply density. Per solid occurrence, source `0` resolves the assigned layer's
material, `1` the object's material, and `3` the parent instance's material (or
the object's layer when there is no parent), consistent with the pinned openNURBS
`opennurbs_defines.h` enum. Unsupported source values, missing materials or
unbound densities refuse the entire report; prior reports and document/Undo are
preserved. Each member is measured from canonical BRep with composed placement;
the report identifies each occurrence and density. Bindings are keyed by document
and render-material ID, are bounded by configured entry/name limits, and are not
stored in the document or persisted across sessions.

The source fixture measured a three-member 100 mm³-per-member instance with
parent, object and layer sources: 300 mm³ and 5.71 g for 18k gold plus two Pt950
members. Missing and removed bindings preserved the prior report and Undo. The
headless egui fixture clicked Use Object Materials, bound a render material to
Pt950 and calculated 100 mm³ / 2.08 g. The native Matrix package fixture also
checked object-material Pt950 on a selected 6000 mm³ instance (124.8 g).

The `test-fixtures` suites passed 12 metal-weight, 14 add-on-session (2 ignored)
and 2 egui tests; the native package test was explicitly run and passed. The
no-default-features app check passed. These fixtures do not establish desktop/GPU,
persisted bindings or user catalogs, actual GV material library compatibility,
all possible instance/layer combinations, source mesh, worker/cancel or total
allocation accounting.

## Persistent user catalog follow-up

User-entered densities can now be created in the builder with an ID, name, density
in g/cm³ and source. With an application layout path configured, a validated,
versioned `metal-catalog.json` beside the layout is loaded at startup and updated
via a staged replacement; an in-memory-only app labels the entry session-only.
The catalog refuses duplicate IDs, collisions with OM9 reference IDs, test-only
entries, invalid values, excessive fields and oversized files. A failed load does
not replace the active registry, and a failed validation does not overwrite the
existing file. User entries remain separate from OM9 reference entries, including
when a reference ID is replaced in the in-memory registry.

The builder filters its comparison by **OM9 reference** and **User**. The GV
checkbox is visible but disabled because no verified Gemvision catalog is supplied;
OM9 reference values are not identified as Gemvision values. Object-material
bindings remain session-only and are not stored in this user catalog.

Twelve core metal-weight tests, twelve app metal-weight tests, one catalog test and
three headless egui tests passed. The catalog test saves twice, reloads through
normal startup, rejects duplicate/corrupt inputs and preserves the prior file and
active registry. The UI test enters a user material through the actual widgets,
saves it and reloads it in a new app instance. Another UI fixture exercises the
reference filter and preserves the previous report when no material is selected.
The app also passed `cargo check --no-default-features`; all Cargo runs used
`L:/om9-target` and `-j 2`. Desktop/GPU, genuine GV catalog import, material edit/
delete, persistent object bindings and asynchronous measurement remain open.

## Persistent bindings and user material editing

The builder now saves explicit render-material-to-density links in the validated
user catalog, keyed by document ID and material ID. Loading the catalog restores
the links for their matching document; the same material ID in a different document
does not inherit them. The prior catalog schema without bindings remains readable.
The lower-level controller binding API remains session-only. Apps with no configured
catalog path also keep changes only in memory. A failed save leaves the live binding
unchanged, and missing or invalid density references reject the entire catalog.

Users can edit a user density, name, source and uncertainty, or delete an entry.
IDs stay fixed during editing; a density still referenced by a session or persisted
binding cannot be deleted until unbound. Changes validate and save before changing
the active registry. A headless egui fixture created, edited and deleted a user
entry and verified reload. A separate egui fixture assigned a material and verified
that its binding reloaded for the same document ID. Three catalog tests covered
cross-document isolation, deletion refusal, unbinding, reload and a simulated write
failure. Twelve metal-weight tests, fourteen add-on-session tests (two ignored),
three catalog tests and three egui tests passed with `test-fixtures`; the app's
no-default-features check passed. All Cargo runs used `L:/om9-target` and `-j 2`.

The reload fixtures reconstruct a document with the same IDs; an actual BRep
archive save/reopen remains blocked by the existing v5 migration gap. The shared
catalog does not yet coordinate concurrent app processes. GV catalog import,
desktop/GPU validation, source mesh, worker/cancel and complete allocation
accounting remain open.

## Measurement worker preparation

Canonical BRep measurement now exposes a cancellation token and an admission
callback for Rust-side source validation scratch. Nested occurrence traversal
checks cancellation between members and propagates admission refusals; the
object-material collector accepts the same controls. Report calculation is now
separate from live-controller publication so a later worker can build a result
without changing the document or report. Existing UI and command execution remain
synchronous; no background worker or stale-result guard has been connected.

The kernel test measured the 6000 mm³ box and exercised pre-cancel, cancellation
from an admission callback and an explicit budget refusal. The nested instance
fixture exercised cancellation and budget refusal at the first source; the twelve
Metal Weights and three egui fixtures passed after the refactor, and the app
no-default-features check passed. The native OCCT call remains synchronous and
cannot be interrupted while inside OCCT; its internal allocations are not covered
by this callback. Cancellation after entry can only discard the result after the
native call returns. These checks do not prove an interactive worker, UI latency,
complete memory accounting or cancellation of native work.

## Background builder measurement

The interactive `MatrixApp` enables a Metal Weights worker for the builder's
Calculate action, typed `Calculate`, `MetalWeights all` and explicit material
options. The worker retains a tracked immutable document, snapshots selection,
registry, bindings and panel mode, and builds the report off-thread. Before
publication, the controller checks document epoch/identity, selection revision,
request generation and material state. A changed input, a newer report, closing
the panel or cancellation prevents publication. A single-operation permit stays
held while canceled native work drains; the panel shows calculation/cancel state.
Headless callers that do not enable the background option retain the synchronous
API, as does the non-OCCT fallback.

The worker reserves an initial 64 MiB kernel allowance plus snapshot metadata,
requests further admission for source-validation scratch and estimated report
storage, and transfers retained output to a CPU-cache reservation. This is not a
proof of the peak memory used inside OCCT or every allocator overhead. Cancellation
is checked during Rust traversal/validation and after the native call; OCCT itself
is not interruptible, and report formatting is checked only after completion.

Seventeen Metal Weights tests and three headless egui tests passed; the add-on
session suite passed fourteen with two ignored. The fixtures cover selected source,
catalog and mixed object-material results; stale selection, document edits, mode,
registry and binding changes; a newer synchronous report; cancellation; and
refusal when the initial reservation is unavailable. The native Matrix package
fixture was explicitly rerun and passed its background object-material instance.
The no-default-features app check passed. These are scoped headless checks: they
do not establish desktop/GPU responsiveness, hard latency, interruption of a live
OCCT call or complete native memory accounting. BRep archive save/reopen and the
verified GV catalog remain open.
