# Dependent History on a private candidate

The core producer registry can now rebuild the recipes affected by a set of changed
source IDs in dependency order on a caller-owned candidate. A rebuilt output feeds
the next dependent; unrelated recipes are skipped. Update-off recipes are not
rebuilt. Failure, cancellation or a returned failed outcome requires the caller to
discard the candidate and its cache. Native BRep results that need representation
handoff are refused by this path.

On 2026-11-17, a two-join chain fixture verified ordered source-to-child-to-downstream
recompute, unchanged unrelated geometry, last-good downstream geometry on a failed
draft, update-off behavior, pre-cancellation, and a traversal-budget refusal. A second
core fixture manually composes a prepared source edit, candidate recompute, and one
transaction containing both dependent updates; one Undo and Redo restore the entire
chain. The 10 producer tests, the combined transaction fixture, 4 History mutation
tests and 7 History replay tests passed with `L:/om9-target` and `-j 2`.

This is **core fixture evidence**, not automatic product behavior. The host has not
yet scheduled this work, accounted for candidate and producer allocations, guarded
stale/cancel results or published the scene. The traversal limit does not bound the
work of existing producers, and cancellation during a producer is not proved. The
test's manual candidate copy is not a production memory-ownership design. Menu,
CMD, F6, viewport and desktop validation remain open.

## Caller limits for Join History

The candidate path now passes its caller's geometry limits to the Join History
producer and topology policy, after admitting traversal and input metadata. A
fixture with a one-node ceiling refused the first join and kept its prior child;
before this change the producer silently used default limits. Producers that do not
opt in to caller limits are refused on this candidate path; a panic-on-use fixture
checks that their ordinary producer is not called. The ordinary explicit-recompute
path is unchanged.

This is not a certification of peak memory: candidate ownership, snapshot copies,
all intermediate allocations and native BRep handoff still need host admission.
Cancellation was initially checked only between producers.

## Cooperative checks in Join History

Join History's candidate producer now passes its cancellation token into the shared
canonical join. Checks occur while preparing the input curves, traversing endpoints,
inside the dense endpoint sweep (at most 256 comparisons between checks), building
the trail and healing its joints. A deterministic 300-curve fixture cancels at an
inner sweep check and verifies that the input entities remain unchanged. The core
candidate wrapper still requires the caller to discard a canceled draft.

The cancellation, producer, dependent-transaction, History mutation and History
replay targets passed; the 7 join-filtered Top 11 tests passed. A broader Top 11 run
had 42 passes and 2 failures: `circle_and_polycurve_grips_are_editable` expected 4
grips but got 3, and `grips_expose_edit_and_control_points_in_world_space` expected
8 but got 4. These grip cases are outside the changed join path and remain open;
the broad suite is not a pass. The 2 default-build host Join History fixtures passed
after the topology-check cancellation guard was added. A separate mock producer
also cancels during candidate recompute and verifies that the wrapper returns
`Canceled` without changing the child geometry; the caller must discard the draft.

The checks do not interrupt sorting, a single curve's placement conversion,
flattening, topology generation, or snapshot copying mid-operation. No latency
bound, peak-memory guarantee, worker cancellation, or UI publication is proved.

## Checked transaction assembly

A reusable Join History-only helper assembles geometry, recipe and result mutations
from a staged source edit and its rebuilt dependents. The two-level fixture commits
them in one transaction and verifies Undo/Redo. Pre-cancellation, a stale candidate,
and a source edit that disconnects the downstream join leave the live document and
Undo unchanged.

Identity, session generation, epoch, revision and source-stamp checks do not prove
candidate provenance. The caller must supply a candidate prepared from the exact
source edit against the same original document, account for its copies, and discard
it on failure. Other producer handoff and BRep representation handoff are refused.
Host worker, memory admission, stale guards, automatic propagation and desktop
routes remain open.

## Prepared source provenance

`prepare_join_history_edit` retains the exact source-only geometry or placement
request alongside the candidate it prepared, tracks the baseline and registers
the candidate in the shared memory account. Its assembly method clones that
candidate on a worker and returns the same source request with its dependent
updates. It rejects duplicate or unsupported source mutations, foreign memory
accounts, an undersized draft reservation, looser limits and pre-cancellation.
The caller retains the returned input and draft reservations until publication.

A core fixture runs assembly on a separate thread, commits two dependents with
one Undo/Redo, and verifies refusal when an intervening edit makes the request
stale. It also exercises duplicate sources, foreign account, insufficient draft
reservation and cancellation. The minimum draft reservation covers an estimated
document copy and validation scratch, not all producer, returned mutation or
native allocations. The host must admit those separately. Preparing the initial
candidate still runs on the caller thread; no host worker, scene publication or
automatic product propagation is wired by this change.

## Handoff and source-set preflight

The Join-only assembly now refuses an affected unsupported producer before
invoking it. A panic-on-call fixture confirms that refusal, and also confirms
that an update-disabled branch is skipped. General dependent recompute retains
its existing registry behavior.

Preparing a source edit admits the duplicate-detection set and uses it instead
of comparing each source with all preceding mutations. A 256-source fixture
prepares distinct edits and rejects a repeated source without publishing or
adding Undo. Draft minimum admission also includes the changed-source set.
The 2 dependent-transaction and 10 producer tests passed with the external
target and `-j 2`; the separate prior History mutation/replay targets passed.
This does not bound producer scratch, input snapshots or returned mutations,
and does not enable host scheduling or automatic propagation.

## Per-recipe snapshot admission

The prepared edit can now ask its admission peer to grow the pending draft
reservation before graph traversal and before each affected recipe takes input
and rollback snapshots. The estimate is recomputed from the current candidate:
it uses six times its duplicate-inclusive document inventory, plus its validation
estimate and explicit graph, vector and set allowances. Duplicate Join History
inputs are rejected before snapshot copying. A worker-thread fixture grows the
reservation from one byte through the peer. A separate 64-segment source fixture
shows the second join requests more allowance after the first expands its child.
Refusing that second admission keeps the live document and Undo unchanged.

This is a conservative estimate, not a measured or proven peak bound for every
snapshot layout. Producer geometry scratch, cache/outcome retention and returned
mutation copies still require complete accounting. The reservation remains pending
while these allocations exist, and the initial candidate is still prepared on the
caller thread. No product worker or automatic source propagation is enabled.

## Rebuild and update allocation follow-up

History output hashing now counts the JSON length and streams identical JSON
bytes through a fixed 4 KiB buffer into SHA-256. It no longer materializes the
entire encoded geometry; this trades a second serialization pass for bounded
encoding buffer size. A compatibility fixture compares the result with the
previous length-prefixed JSON for a polycurve and mesh. Existing source-hash
fixtures were updated to use canonical B-rep source types and pass for their
source/display distinctions. No desktop latency measurement was made.

The join trail moves each forward curve into its ordered result instead of cloning
it while retaining the input copy. Reversed curves still need a converted copy.
The admission estimate now includes update-vector metadata and is rechecked
before each rebuilt recipe and result is copied into transaction mutations.
An injected refusal at that point leaves the live document and Undo unchanged.
The existing two-join fixture and seven join-filtered Top 11 tests pass.

The estimate is still not a proven upper bound for geometry producer temporaries,
all snapshot capacity or retained outcome/cache allocations. The host worker,
scene publication, automatic propagation and desktop tests remain open.

## Explicit host worker fixture

An explicitly invoked controller worker now carries the prepared source edit and
its tracked candidate to a shared-account admission peer. On completion it checks
the document epoch, scene publication and selection/subselection revisions, then
uses the normal transaction and candidate-scene commit. Cancellation retains the
worker's scene and peer-account owners until it drains. The interactive menu and
grip paths do not invoke this experimental entry point.

In the default-build controller fixture, one parent edit rebuilt its Join History
child, published a scene object bound to the new child source stamp, and restored
both source and child with one Undo/Redo. A changed selection, cancellation, and
a disconnected join left the live document and Undo unchanged. These are headless
controller checks, not desktop/GPU evidence.

The initial source transaction is still prepared on the caller thread, and final
transaction/scene preparation also runs there. The snapshot allowance remains an
estimate; producer temporaries and all result owners are not fully bounded. The
worker is therefore not connected to user routes or automatic parent editing.

## Explicit background scene phase

The experimental host path now runs candidate-scene capture in a second worker
phase on the shared admission account. The prepared document is registered and
a conservative staging allowance remains reserved while the scene is built. The
controller checks document, scene and selection freshness again before committing
and publishing. The worker permit and memory owners remain live through both
phases and while a canceled worker drains.

The headless fixture waits until the scene phase starts, then checks that a
selection change refuses publication and cancellation leaves the document and
Undo unchanged. The source/child/scene-stamp and Undo/Redo fixture still passes.
This exercises curves; B-rep wire preparation in this path has not been tested.
The scene operation only checks cancellation before and after its work. Initial
and final transaction preparation still run on the caller thread, producer peak
memory remains unproven, and the worker is not routed from user commands.

## Background transaction preparation

An unscoped preparation peer now captures the coordinator's document, Undo/Redo
and identity state, shares its admission account, and retains an allowance for
the copied metadata. It exposes preparation without mutation access to the
originating coordinator. A core fixture prepares on a separate thread, commits
through the primary coordinator and verifies Undo/Redo; an intervening edit makes
another prepared transaction stale and leaves the new document untouched.

The explicit host worker now prepares both the initial source edit and final
source-plus-history transaction on that peer before building its candidate scene.
The primary coordinator rechecks the prepared transaction and publishes it only
after the usual context guards. Six controller fixtures pass, including stale or
canceled results after the scene phase begins. Caller-thread work still includes
request inventory, metadata capture, final retention recheck and publication;
their latency has not been measured. Producer peak accounting, full cancellation,
B-rep wire and desktop checks, and user-command routing remain open.

The peer allowance also includes copied Undo evidence owners. A canonical B-rep
fixture clears the primary history while a peer remains alive: the evidence
survives under the peer's pending allowance and is released when that peer drops.
The 4 `history_source_hash` and 3 dependent-transaction integration tests pass.
An attempted `cargo test -p om9-core --lib ...` did not compile because 108
existing unit-test errors include stale `Brep`/`CanonicalBrep` and related type
usages; it is not evidence of a library-test pass.

## MatrixApp parent grip propagation

Before this change, a parent grip edit in MatrixApp committed independently and
left the Join History child dirty until explicit recompute. Now, when an edited
grip feeds an update-enabled, attached Join History recipe, the edit reports that
History is updating and uses the background worker for the parent and affected
Join History descendants. The old scene stays visible while the worker runs;
Escape discards the pending edit. Unsupported downstream producers, failed joins,
stale context and admission refusal do not publish a partial transaction.

Seven headless egui fixtures passed: two-level propagation and one Undo/Redo,
pointer drag in the Looking Down viewport, gumball-selected grip, Escape, a
downstream join failure, a simulated memory refusal, and a parent edit after
saving and reopening the two-recipe document. The scene fixture checks child
source stamps. The 6 direct host and 4 existing grip-selection tests also passed;
the no-default-features app check and the same 7 egui fixtures passed. The pointer is an egui event simulation,
not an OS mouse or GPU/desktop run.

This covers MatrixApp grip edits only. Placement edits, other source mutation
routes, menu/CMD creation and F6 remain open. The caller still inventories and
captures metadata, producer/snapshot peak is not proven, and cancellation cannot
interrupt every geometry or scene operation. The prior document and Undo are
retained on the failure cases above. Reusable skill text for this changed behavior
is pending user consent.

## MatrixApp gumball parent placement

Before this follow-up, moving parent objects with the gumball committed their
placements without rebuilding the Join History children. MatrixApp now queues
the same background transaction when a non-copy gumball move changes parents
of an update-enabled, attached Join History recipe. A copy still inserts new
objects without changing or rebuilding the originals. The status says History
is updating until the worker publishes or reports an error.

Headless egui/controller fixtures translate three parents, verify both joined
children and their scene source stamps, and restore all changes in one Undo/Redo.
The same two-level line chain also passed a 90-degree Z rotation and a factor-2
X scale, checking transformed endpoints and one Undo.
Moving one parent so its join disconnects leaves the document and Undo unchanged;
copying a parent leaves the existing children unchanged. The full 11-fixture
Join History UI target passed in default and no-default-features builds, along
with the 6 host and 4 grip-selection fixtures. These object-gumball gestures were
invoked through the controller, not by an OS mouse or GPU desktop test.

Other rotation/scale geometries and axes, source-edit commands, menu/CMD/F6,
producer peak memory and full cancellation latency remain unverified. Recording
this changed behavior in a reusable skill awaits user consent.

## Move and Rotate source commands

Before this follow-up, the normal Move and Rotate commands committed parent
placements independently of Join History. In MatrixApp, non-copy Move and Rotate
now queue the same background source-and-descendant transaction when their
mutations affect an update-enabled Join History parent. A disconnected Move
reports the rebuild failure and keeps the previous document and Undo.

The command editor previously omitted the available Top 11 Move and Rotate
entries from suggestions; typing `Move` could therefore select `MoveUVN` instead.
Those two exact entries are now eligible suggestions. Headless egui fixtures
type Move and Rotate in the command editor and verify two-level results and Undo.
Additional fixtures dispatch Move and Rotate through their menu-origin feature
IDs, exercise point inputs and Rotate's typed-angle path, and verify one Undo.
The full 16 Join History UI fixtures passed in default and no-default-features
builds; 9 command-inline-option and 6 host fixtures passed. Menu dispatch was
called in the fixture; an OS mouse click and GPU desktop were not tested.

Mirror and other source-edit paths, producer peak memory, full cancellation and
Join History creation through its own menu/CMD/F6 remain open. The command's
pending input is consumed when the worker starts; a failed background rebuild
requires starting the transform again. Recording this behavior in a reusable
skill awaits user consent.

## Cage and Smart Flow source commands

Cage Edit now sends the cage geometry and attached source geometry through the Join History worker. Smart Flow and Smart Flow Pull Down likewise use that worker; the core private candidate accepts same-document extension updates alongside Geometry/Placement and Inserts, so flow links and dependent outputs publish with their parents. An interactive add-on mixed-reply fixture also covers a record update, source transform and inserted point together.

Headless fixtures cover Cage Edit with two dependent joins, one Undo/Redo, disconnected failure and cancellation; Smart Flow and Pull Down cover endpoint geometry, flow links and Undo/Redo, while separate Smart Flow cases verify failure and cancellation leave the links and document unchanged. Twenty-one core tests and 123 default-feature app tests passed (two ignored) across History UI/host, command options, Transform menu and add-on sessions, followed by the mixed add-on record fixture. Geometry preparation remains on the caller, and producer/native peaks, other deformation inputs, OS input and GPU are not established.

## Interactive add-on source edits

An interactive add-on reply may contain a Geometry or Placement edit of a Join History parent together with same-document Inserts. The host now sends that entire request through the History worker, and reports the add-on's own command identity as progress and then committed, refused or canceled. The core prepares the private source-and-insert candidate, rebuilds dependents and publishes it as one transaction. Replies containing unsupported mixed mutations, including a Delete of an affected parent, are refused before publication; this is not a complete policy for deleting or replacing History parents.

Headless tests exercise the installed in-process Matrix package's Scale session for success, disconnected failure and cancellation. A scripted add-on boundary exercises parent transform plus a newly inserted point, disconnected failure and parent-deletion refusal; success verifies two downstream joins and Undo/Redo. Core fixtures independently check same-document insert/edit assembly, failure, cross-document rejection and insert-only rejection. Twenty-one core fixtures and 70 default-feature app fixtures passed (two app tests ignored), followed by the scripted mixed-reply fixture; 47 no-default-feature app fixtures passed (one ignored), using `L:/om9-target` and `-j 2`. The package runs in-process through the JSON boundary, not a native DLL; exact-source replacement behavior, other mixed mutations, producer peaks and OS/GPU validation remain open.

## Transform menu native fixture migration

The prior Transform menu target compile failure is resolved without replacing its native-solid fixture with a mesh entity. Its synthetic opaque native source and display are packaged in a v4 test document and opened through the normal source/display migration, which binds the retained representation to the account before scene publication. Centre checks read that retained representation directly. The payload encodes the fixture mesh and is not a valid openNURBS model, so these tests establish Transform and legacy representation behavior only, not native topology or a real model round trip.

All 50 `transform_menu` fixtures pass with default features, and all 50 pass with no default features, using `L:/om9-target` and `-j 2`. Other Transform operations feeding Join History and broader source edits remain open.

## Transform menu source commands

The shared in-place placement path and the shared deformation geometry path for Transform commands now schedule Join History rebuilding when their mutations affect parents. Copy and array inserts retain their existing path. The command reports progress until the worker sends a committed, refused or canceled outcome. Headless typed and menu-dispatch fixtures cover Scale, Scale1D, Stretch and Shear with two-level joins and Undo/Redo; Scale1D also covers disconnected failure and cancellation. Deformation geometry itself is still prepared on the caller. Fifty-five default-feature fixtures passed across Join History UI, host and command options, and seven `transform_corrections` fixtures passed.

At this checkpoint the broader `transform_menu` target did not compile because its synthetic NativeBrep fixtures used a display field no longer present on `NativeBrepSource`. The fixture migration described above resolves those errors. Other Transform families and specialized mutation paths remain unverified.

## Numeric gumball source commands

MatrixApp numeric gumball placement changes affecting Join History parents now use the same worker transaction as drag gestures. The numeric handler reports progress while the worker runs and emits a committed, refused or canceled completion event only after its outcome is known. A headless fixture tests direct submit and simulated command-editor entry for MoveX, RotateZ and ScaleX across a two-level line Join, including exact transformed endpoints and one Undo/Redo. Another fixture tests disconnected-parent failure and one tests cancellation without publication. Fifty-two default-feature fixtures passed across UI, host and command options. Other axes, pivot/alignment modes, desktop input and GPU presentation remain unverified.

## Surface Trim source command

Interactive Trim with one partitioned surface or multiple surfaces can also pick a Join History parent curve. For a single surviving curve, both paths now schedule dependent rebuilding and preserve the old and prospective surface-partition sessions until the worker completes. Reservations conservatively include both session states, including retained BRep storage; the shared original-edge owners are counted twice. Success advances the same Trim interaction after atomic publication, while failure or cancellation restores its prior state. These paths refuse parent replacement with zero or multiple pieces.

A headless OCCT fixture exercises one and two surfaces, a crossing curve, a two-level Join History chain and each of success, disconnected failure and immediate cancellation. It verifies dependent endpoints, session state and Undo/Redo. Forty-nine default-feature fixtures passed across UI, host and command options. Surface partitioning, curve trimming and creation of prospective session data remain on the caller, and native internal allocation peaks and cancellation during OCCT remain unproven. A separate `TrimMany` state has no identified entry point in the current controller and remains unverified.

## Trim source command

Numeric Trim and interactive mutual Trim now route a single surviving parent curve through the Join History worker. A successful rebuild updates two descendant joins in the same Undo; a disconnected result leaves both document and Undo unchanged. For interactive mutual Trim, the worker retains admitted previous and prospective session state and restores the previous state on failure or cancellation; the next session state is installed only after successful publication. Another command is refused while the worker is pending. The separate multi-target Trim continuation refuses affected History parents, and mutual Trim refuses a result that would replace the parent with zero or multiple pieces. This does not establish behavior for other deletion, split, cut or source-edit routes.

Headless tests cover numeric and interactive Trim, two descendant levels, success, failure, cancellation and Undo/Redo; 47 default-feature app fixtures passed across UI, host and command options, followed by a separate multiple-piece refusal test. Twenty-seven no-default-feature UI fixtures also passed. Curve trimming and interactive intersection/continuation preparation are still on the caller thread; allocation peak, cancellation inside producer calls, OS mouse and GPU are not established.

## Mixed geometry Mirror source worker

All non-copy Mirror selections with an affected Join History parent are now dispatched to the captured worker, including the supported non-BRep geometry handled by the existing core Mirror routine. A headless mixed fixture covers a point, a triangle mesh and its face winding, a NURBS plane, a placed block instance and the parent curves; it verifies both dependent output and whole-document Undo/Redo. A disconnected-parent fixture including a point verifies no partial publication. The prior mixed OCCT box tests still apply. Forty-three default-feature fixtures passed across UI, host and command options, followed by two mixed-geometry fixtures; 23 no-default-feature UI fixtures passed. Other types supported by core Mirror have not yet been checked in this path. Caller-side source validation, estimated allocation allowance, incomplete cancellation and OCCT internal peak remain open.

## Mixed BRep Mirror source worker

At this checkpoint, non-copy Mirror of selections containing curves and canonical BReps captured the scene deflection and prepared both source types in the History worker. Each BRep is transformed from a captured source using a composed local-to-world reflection and inverse placement; cancellable Rust-side source validation and snapshot copying use admission callbacks. The source reservations remain held through dependent preparation and publication. OCCT's internal allocations are not bounded by those callbacks, and an active OCCT call cannot be interrupted. The caller still validates selected geometry and captures the worker state. The later mixed-geometry checkpoint above extends worker dispatch beyond curves and BReps.

The mixed real-OCCT fixture checks reflected box volume and bounds, both Join descendants, and one Undo/Redo. A translated box verifies world-space reflection while preserving its placement. Failure, immediate cancellation and stale selection leave the document and Undo unchanged. The default app validation passed 41 fixtures across UI, host and command options, followed by the additional stale fixture; 22 UI fixtures passed without default features. These are headless checks, not OS-input, GPU or latency measurements. The curve input allowance remains an estimate and no native or producer peak proof is claimed.

## Curve Mirror source worker

Non-copy Mirror of a curve-only selection containing Join History parents now reserves an input allowance before cloning the selection and reflects its curves in the captured History worker, before dependent evaluation. The caller still validates selected source geometry and captures worker state. The allowance uses four times validated geometry bytes plus per-object overhead; it is an estimate and does not establish the producer's true temporary allocation peak. A completed request larger than the allowance is refused, but this post-production check is not a preallocation proof.

The existing command, two-level join, Undo/Redo and Escape fixtures pass through this worker path. New fixtures confirm refusal under externally reserved memory pressure and rejection after selection changes without modifying the document or Undo. Forty default-feature fixtures passed across UI, host and command options, and 22 UI fixtures passed without default features. This does not move OCCT or mixed-selection source preparation off the caller, establish cancellation during an individual producer call, or validate OS input/GPU latency.

## Mixed BRep Mirror handoff

The prior mixed-selection refusal is replaced for non-copy Mirror: the existing source-bound native reservations and their vector allocation remain owned by the History worker until publication or disposal. Dropped, stale and undeliverable completed candidates release their owners before pruning orphaned representations. Reservation accounting remains conservative; this does not establish a complete native peak bound.

Headless fixtures use a real OCCT box and a two-level curve Join chain. Mirroring all parents and the box publishes the changed BRep and both descendants in one Undo/Redo step; mirroring just one parent with the box causes a dependent rebuild failure and leaves the document and Undo stack unchanged. A separate immediate worker-cancel fixture also leaves the document and Undo stack unchanged. The default-feature app validation passed 37 fixtures across Join History UI, host and command options plus the new cancel fixture; 20 no-default-feature UI fixtures and the public source audit also passed. Mirror source geometry, including the OCCT call, is still prepared on the caller thread; cancellation during that native call and desktop latency remain unverified. Join History creation routes remain disabled.

## Mirror source command

Before this follow-up, a non-copy Mirror committed parent geometry without
automatically rebuilding Join History children. MatrixApp now queues the
parent and affected Join History descendants together. Two-point, three-point
and YAxis forms, menu-origin dispatch and typed command-editor input were
exercised on a two-level line chain. The result and one Undo/Redo were checked;
Escape preserved the document and Undo, and a disconnected mirror also preserved
an existing Redo entry.
Copy=Yes still creates a copy without rebuilding the originals. Mirror is also
eligible as an exact command suggestion.

A mixed selection containing a Join History parent and an OCCT B-rep is refused
before either changes, because the worker cannot yet take the native replacement
reservation. A default-build fixture uses a real OCCT box for this refusal. The
21 default and 20 no-default-features Join History UI fixtures, 9 command-inline
fixtures and 6 host fixtures passed on `L:/om9-target` with `-j 2`. The broader
app `top11` test target could not compile because two existing fixtures pass
`NativeBrep` where `NativeBrepSource` is required; it is not a passing suite.

Mirror still computes its source geometry before dispatching the worker, so its
caller-thread latency and allocation peak remain open. Other source edits,
native mixed-selection handoff, full cancellation, desktop/GPU, and Join History
creation via menu/CMD/F6 remain open. Skill recording awaits user consent.


## Center Object controller source edits

`Controller::center_selected` now sends placements of Join History parents to the dependent rebuild worker, publishing the two-level chain and scene in one transaction. A MatrixApp headless fixture exercises all three parents moving together, disconnected-join refusal, cancellation, and one Undo/Redo. The test calls the controller method directly; no menu or CMD route to this method was identified, so it does not establish a user-facing Center Object command. World-bounds calculation still precedes worker dispatch. Join History creation and other source-edit paths remain open.

Validation with `L:/om9-target` and `-j 2`: default `join_history_ui` and `join_history` suites passed 50 tests; no-default `join_history_ui` passed 38. Skill consent requested.


## Top 11 parent replacement guard

The common Top 11 publication path refuses Geometry, Placement or Delete mutations on update-enabled Join History parents when they have not been routed to the dependent rebuild worker. A headless MatrixApp fixture checks typed `Split 0.5` and Top 11 Join leave parent, children, selection and Undo unchanged, while splitting an unrelated curve still works. This is an interim refusal: replacing a parent by multiple pieces and adapting its recipe are not implemented. The guard covers this Top 11 publication path, not every document mutation or external SDK route.

Default `join_history_ui`, `join_history` and `command_inline_options` passed 60 tests; no-default `join_history_ui` passed 39. A final focused default fixture also passed after adding the unrelated-curve assertion. The broader `top11` integration test does not compile: two existing synthetic fixtures construct `NativeBrep` where `NativeBrepSource` is required (lines 393 and 4058). `L:/om9-target`, `-j 2`; skill consent requested.
