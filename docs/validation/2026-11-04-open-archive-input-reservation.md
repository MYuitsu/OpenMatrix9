# Archive input reservation during Open

Open now reserves the archive input length in the coordinator's shared CPU-cache account before allocating the input buffer, using the size obtained from the already-open file handle. It samples physical memory when available and includes the current scene and path in admission. The guard remains live through restoration and is released only after the input buffer is dropped. A file that grows after its length was sampled is refused rather than read past the reserved size; refusal leaves replacement unpublished.

Two archive tests cover reservation refusal before reading and a deterministic file-growth race, including guard release. A shared-account synthetic test checks a competing peer reservation is refused while archive input is reserved; existing New/Open publication races and 20 file-lifecycle integration tests also pass. On E: with `-j 2`, seven document-snapshot tests, seven codec acceptance tests and three replacement-budget tests pass. The first attempt to compile the new peer test failed because its diagnostic required `RestoredDocument: Debug`; it was corrected and rerun. This is not a desktop or release measurement.

This reservation covers only the raw file input buffer. Allocator overhead/capacity rounding, parsing DOM, decoded document, restoration scratch, candidate controller construction, other live worlds and scene owners do not yet have a complete shared reservation through publication. The guard is released after restoration and before final replacement. If the OS sample is unavailable, this path has no fresh available-memory check. The existing 256 MiB cap therefore remains; do not treat this as proof of the P0 60% budget or a reason to raise it.

## Staged growth follow-up

External reservations can now grow without releasing their original bytes: the additional amount is admitted while the first guard remains live, then ownership is transferred into that guard. A refused increase and a foreign-account request preserve the original reservation; a successful increase is fully released on drop. A fresh physical sample covers the combined pending amount conservatively; an unsampled increase clears the sampled status.

Open uses the staged interface to reserve an additional amount equal to the bytes actually read before starting restoration. This corresponds to one of the archive preflight's input-sized terms, but does not bound all JSON bookkeeping, typed objects, restoration or controller construction. A test verifies guard release when this stage is refused; targeted core, eight archive snapshot, seven codec and three replacement-budget tests pass. The 20 file-lifecycle tests were also rerun. The guard still ends before the replacement is published, and no complete concurrency or native peak measurement is claimed.

## Typed-document preflight follow-up

For schema 4 Open, the streaming JSON structural preflight now passes its estimated total to the shared reservation before typed document deserialization. The guard is grown only by any positive difference from its already reserved input amount and is held through restoration. A valid archive test refuses this stage and confirms that the guard is released; all archive tests, three app replacement-budget tests and 20 app file-lifecycle tests passed. This admission is based on the existing structural estimate, not a measured upper bound for all allocator behavior or a certificate of actual RSS.

The streaming JSON preflight itself allocates before this new callback; chunk bookkeeping, unescape scratch, hydration and restore allocations may exceed the reservation estimate. The restored document/candidate remains outside this guard after it drops. A fresh OS sample may also be unavailable. Full peak admission and the 256 MiB interactive cap remain open.

## Streaming preflight reservation follow-up

Schema-4 JSON preflight now requests its initial input/scratch estimate before scanning, then grows the shared reservation before each estimated node/key charge. The application batches reservation growth in 64 KiB increments and retries the exact amount near the limit. A synthetic valid archive is refused on the fifth admission callback while scanning and confirms the input guard is released. The entire archive test suite and app replacement/file-lifecycle regressions pass on the external E: target with `-j 2`.

The hook covers the existing estimate, not a measured allocator upper bound. Container chunk bookkeeping occurs before this streaming hook, and resource hydration, final restoration and candidate construction still lack full shared reservations through publication. The batched callback's performance, especially under a tight budget, has not been measured. The 256 MiB cap and complete 60% RAM gate remain open.

## Chunk-table reservation follow-up

The archive decoder now admits its chunk vector and chunk-ID tree before those structures grow. For schema 4, their estimated bytes are added to the streaming JSON estimate while the structures coexist; the combined estimate is checked against the operation limit and passed to the shared reservation. A valid-archive test refuses the first chunk-table admission, before the table allocation. The full archive suite and New/Open regressions were run with `-j 2` on the external E: target.

This covers the project's accounting estimates, not verified allocator capacities. Typed payload hydration, restoration scratch and the candidate controller through publication remain incomplete. The prior note that chunk bookkeeping occurs before the reservation hook is superseded by this follow-up; performance and 256 MiB limitations remain.

## Restoration peak estimate follow-up

After typed decode and before session renewal and diagnostic/history allocations, Open now grows the existing reservation to the archive input plus the restoration routine's estimated document and scratch peak. A focused test refuses that admission before session mutation; all archive tests and app New/Open regression tests pass on the external E: target with `-j 2`.

Calculating the estimate still invokes document validation before this new callback; its scratch is not fully reserved by this step. Resource hydration, allocator overhead and the candidate controller after the archive guard drops remain incomplete. The estimated peak is not a measured RSS bound, and this does not close the 256 MiB or overall RAM-budget gates.

## Embedded-resource hydration follow-up

Before copying embedded resource payloads, the decoder now passes its existing combined estimate for archive input, chunk bookkeeping and prospective hydrated document storage to the reservation callback. A synthetic archive containing a 64 KiB embedded payload was reopened successfully, then refused at the hydration admission in a second run. The full archive test suite and app New/Open regression suites were run on the external E: target with `-j 2`.

The document manifest validation used to calculate this estimate itself runs before this callback. The estimate is not a measured allocator or process RSS bound; validation scratch and the restored candidate's lifetime through publication remain incomplete. Prior statements that resource hydration had no shared reservation are superseded for this path only. The 256 MiB limit and full P0 budget remain open.

## Document validation scratch follow-up

The core document now exposes the existing owned-storage plus validation-scratch estimate without claiming the input is valid. The archive Open path admits this estimate alongside its input/chunk owners before manifest validation and resource hydration; restoration also admits it before calling document validation. A 100-entity fixture checks that the estimate includes scratch and both estimation and validation refuse a limit just below it. The existing hydration-refusal and restore-refusal tests exercise the callbacks. Core library and full archive regressions, plus app New/Open tests, were run on the external E: target with `-j 2`.

The inventory/estimate calculation itself traverses document data and some geometry validation paths may do work or allocate before this admission; exact allocator capacity and native peaks remain unmeasured. The guard still ends before the candidate controller is published. This does not complete the managed-RAM gate or remove the 256 MiB cap.

## Atomic candidate reservation transfer foundation

The core replacement-admission lock now supports validating a candidate reservation against existing owners and competing reservations, then converting that reservation and executing a publication callback while retaining the same lock. Refusal leaves the reservation intact; the candidate estimate must fit the reservation. Tests cover insufficient reservation, competing held memory, cleanup and an actual swap of two coordinators sharing an account. On E: with `-j 2`, 275 core library, 26 atomic transaction and eight Undo-memory tests passed.

This is a foundation, not yet wired into the application Open path. It assumes the supplied candidate estimate and reservation fully cover the candidate and that the callback publishes the validated replacement without calling account operations under the lock. Constructing and sizing the candidate, moving the archive reservation into that lifetime, other live worlds and measured native peaks remain open. The current Open guard still ends before publication; the 256 MiB cap and complete P0 gate remain.

## Open candidate transfer follow-up

Open now retains the archive reservation while the restored document and candidate controller are constructed. Before publishing, it grows the guard to the candidate's currently inventoried bytes if needed, adopts the shared account and converts the guard inside the replacement-publication lock while swapping controllers. A failed Open releases its guard and retains the old document and Redo; a successful test confirms the candidate guard is consumed. The previous statements that this transfer is not wired and that the archive guard ends before publication are superseded for the Open path.

This does not prove the entire construction peak is reserved: scene construction uses a separate candidate account, and the archive guard may cover only the restored document while the candidate is being built. Candidate file bookkeeping, other peer worlds and old-controller destruction after the swap also need complete shared ownership accounting. Admission may conservatively double-count the candidate and its guard while the accounts are joined. Estimates and physical-memory samples are not measured process peaks; the 256 MiB limit and full P0/P1 acceptance remain open. Full archive, three replacement-budget and 20 file-lifecycle tests were run on the external E: target with `-j 2`.

## Candidate scene peak follow-up

Open now reserves the scene planner's estimated scratch in the old shared account before planning. Before candidate construction begins, it grows the retained archive guard by the estimated scene-construction peak; that guard remains until publication or failure. A 200-point scene test verifies the estimate covers the observed retained scene cache for that fixture. An Open regression verifies that both the archive input and scene estimate are still reserved after candidate construction and that failed replacement retains the original document and Redo. Five scene admission, three replacement-budget and 20 file-lifecycle tests passed on the external E: target with `-j 2`.

The candidate scene still performs its own admission in its separate coordinator; the added shared reservation mirrors its estimated planning/construction peak, not every allocation it performs. Candidate controller metadata, source-estimate construction, auxiliary file data, independent peer worlds and the old controller after publication are not fully accounted. Estimates can be conservative or underestimate allocator/native overhead; no large-model RSS or desktop timing is claimed. The 256 MiB cap and complete P0/P1 gates remain open.

## Candidate controller overhead follow-up

Before naming the restored document and building its controller, Open grows the shared guard by estimates for workspace startup, the controller value, the one-document map and document wrapper, plus the already allocated incoming path and diagnostics. This is deliberately conservative: some owners overlap prior estimates or live on the stack. File path and diagnostic capacities were already included by the existing `FileState::usage` within scene auxiliary accounting. An initial attempted additional charge counted them twice; a new inventory test exposed the duplication and the extra charge was removed.

The Open competition test now verifies that the archive input, candidate scene peak and fixed controller-overhead estimate are retained through construction. Four replacement-budget and 20 file-lifecycle tests pass on the external E: target with `-j 2`. This is not complete controller metadata accounting or measured RSS. Dynamic initialization/fingerprint scratch, some allocation-estimate work, independent peer worlds and old-controller lifetime still need coverage. The 256 MiB limit and full P0/P1 gates remain open.

## Replaced document lifetime follow-up

Before publishing Open, the current document is registered with the shared external-document inventory and held until the old controller has been dropped after the publication lock is released. A focused core test swaps two coordinators while retaining a 1 MiB B-rep source, confirms the new coordinator still accounts for the old document, then confirms the charge disappears when its last test capture is dropped. Four app replacement-budget and 20 file-lifecycle tests were rerun with `-j 2` on the E: target.

This specifically covers the current document allocation, not every old Undo/Redo version, scene buffer, plugin owner or independent coordinator world that may remain live. The old scene and history still need shared lifetime accounting. This is not a measured process peak or completion of P0; the 256 MiB cap remains.

## Replaced history document versions follow-up

Before Open publishes, the old coordinator now registers weak references to its unique retained document versions, including current world, scoped worlds and Undo/Redo entries. The old coordinator remains their strong owner through the swap; the registry does not copy the documents or prolong their lifetime. A core test with a 50,000-point cloud, two Undo entries and one Redo entry swaps coordinators and observes over 4 MiB charged until the old coordinator drops. Four app replacement-budget and 20 file-lifecycle tests passed with `-j 2` on the E: target.

This covers document allocations, not the old scene, Undo entry metadata, plugin resources or independent coordinator worlds. Registration uses allocation-free repeated scans and sequential registry admissions; large histories and partial registry growth on a failed Open have not been benchmarked. Weak registrations can remain while their old owners live after failure, but do not change document content or Redo. Allocator/native overhead and actual process peaks remain unverified; the 256 MiB and P0/P1 gates remain open.

## Retired controller auxiliary reservation follow-up

During Open publication, the replacement lock now converts the candidate reservation and creates a separate reservation for the previous coordinator's existing retained-metadata estimate plus the old controller's scene/auxiliary estimate. The old owner is dropped after unlocking, then that temporary reservation is released. Old document versions remain covered by the previously registered weak inventory. A core test shows a competing reservation is refused while the retired estimate is held and succeeds after it is dropped; a panic-path test checks the transfer does not deadlock or leak its reservation. On the external E: target with `-j 2`, 280 core library, 26 atomic transaction, eight Undo, four app replacement-budget and 20 file-lifecycle tests passed.

This covers the existing accounting estimates for those owners, not complete allocator/RSS, GPU or plugin memory. Some auxiliary state moves into the new controller and is conservatively counted during the brief handoff. Different live coordinator worlds remain not fully registered as a unified owner set. The prior note that old scene and journal estimates had no shared lifetime reservation is superseded for this Open handoff only; the 256 MiB and P0/P1 gates remain open.

## Published candidate peer visibility follow-up

Open validates and registers its initial candidate document in the shared external-document inventory before publication. A core test with a 50,000-point candidate and another coordinator sharing the account verifies that the peer continues charging the candidate after publication and stops when the candidate owner is dropped. Four app replacement-budget and 20 file-lifecycle tests passed with `-j 2` on the external E: target.

This covers the initially published document only. Later edits can replace its Arc with a new version which is not automatically registered for peers; other independent worlds and candidate scene/GPU/plugin owners still need continuous lifetime registration. Registration and joining can conservatively count candidate data more than once and refuse operations near the budget. Validation scratch and allocator/native peak measurements remain incomplete; this is not completion of P0 or a reason to remove the 256 MiB limit.

## Direct edit version peer visibility follow-up

Before publishing a direct, non-scoped transaction, the coordinator registers changed staged document allocations in the shared weak inventory. They are visible to a peer before publication as a conservative pending owner; a failed transaction drops staging and the weak entry expires. A 50,000-point fixture starting from a registered document verifies that the peer counts the new version after a direct move, through Undo/Redo, and stops after the owning coordinator drops. Core library, atomic transaction and Undo suites passed (282, 26 and eight tests), along with four app replacement-budget, 20 file-lifecycle and 19 B-rep picking tests on the external E: target with `-j 2`.

The earlier note that direct edited versions are never registered is superseded for this path. Initial coordinators and scoped finish, open/close, and older unregistered history are not all covered; the Undo/Redo test begins with the initial document explicitly registered. Candidate preparation itself is not fully reserved across all peers, and transient weak registrations can conservatively refuse near the limit. Registry scans and repeated admission now occur for direct commits and have not been benchmarked; these tests do not prove desktop responsiveness or full multi-world accounting. The 256 MiB cap and P0/P1 gates remain open.

## Scoped staged-version peer visibility follow-up

Changed document allocations from scoped commits are now registered in the same weak shared inventory while the staged document is live, before scope finish. A 50,000-point fixture starting from an explicitly registered document verifies that a peer charges the staged version, releases it after Abort, and continues charging a subsequent version after Finish and Undo/Redo. On the external E: target with `-j 2`, 283 core library, 26 atomic and eight Undo tests passed, as did four app replacement-budget, 20 file-lifecycle and 19 B-rep picking tests.

The earlier note that scoped staged versions are never registered is superseded for this commit path. Initial unregistered worlds, open/close and previously unregistered history remain incomplete. Scope finish still needs a fresh shared-account admission through publication; this change does not establish that race is closed. Staging preparation, GPU/plugin owners, actual memory peaks and registry performance remain unverified; the 256 MiB and P0/P1 gates remain open.

## Scope finish shared-account recheck

For a nonempty scope, Finish now rechecks retained owners, outstanding shared reservations and estimated finish scratch under the shared account locks immediately before publication; it keeps the reservation lock until the scope is removed and the new world is published. A deterministic test injects a 300 MiB peer reservation after preparing Finish while simulating 750 MiB of caller-held work. Finish refuses, preserves the scope and existing Redo, and succeeds after the competing reservation is dropped. This is a synthetic admission test, not a physical allocation or process-memory measurement.

On the external E: target with `-j 2`, 284 core library, 26 atomic, eight Undo, four app replacement-budget, 20 file-lifecycle and 19 B-rep picking tests passed. The prior scope-finish admission-race note is superseded for this tested shared-account interleaving. Initial unregistered worlds, preparation lifetime, other memory owners, actual peak behavior and desktop performance remain open; the 256 MiB limit and complete P0/P1 gates remain.

## Coordinator open-document peer inventory

The core coordinator's `open_document` path now registers the validated candidate with the shared weak document inventory before the final locked publication. Its final check excludes that exact candidate from the external pass because the separately admitted candidate/preparation peak already charges it. A 50,000-point fixture verifies that another coordinator sees the newly opened document and that the charge disappears after close. A separate 2 MiB limit fixture admits a 1.2 MB candidate that would not fit if counted twice. Existing competing-reservation and history tests also passed.

On the external E: target with `-j 2`, 286 core library, 26 atomic, eight Undo, four app replacement-budget and 20 file-lifecycle tests passed. This specifically covers coordinator `open_document`, not every initial world or independently owned scene/GPU/plugin allocation. Other forms of shared source, registration cost, transient preparation lifetime and actual process peaks remain unverified. The 256 MiB limit and full P0/P1 gates remain open.

## Source world registration on account adoption

Before a coordinator adopts another coordinator's shared account, the source now registers its already retained world and history document versions. A 50,000-point source fixture shows the joining coordinator counts the original document even before any edit, and releases its charge when the source owner drops. Open no longer repeats that same source-registration pass explicitly. On the external E: target with `-j 2`, 287 core library, 26 atomic, eight Undo, four app replacement-budget and 20 file-lifecycle tests passed.

The initial full-core run exposed an existing test assumption that no registry capacity remains after the old document drops; one 8-byte weak slot remained. The test now checks the exact reservation release and allows the small retained registry capacity. This change registers the source coordinator's documents; it does not automatically register the joining coordinator's own initial documents for the source or all peers. Registration scans, other live owners, GPU/plugin allocations and actual process peaks remain incomplete; the 256 MiB and P0/P1 gates remain open.

## Symmetric account adoption follow-up

Account adoption now registers retained documents in both directions. The joining documents are added while holding the shared registry and reservation locks; if registration or final admission fails, its newly added entries are removed and its existing snapshot is preserved. Legitimate source entries and allocated registry capacity can remain after a failed attempt. Open no longer repeats candidate registration before adoption.

Tests with 50,000-point documents exercised visibility and release in both directions and failure with two joining documents under a 3 MiB limit. On the external E: target with `-j 2`, 289 core library, 26 atomic transaction, eight Undo, four replacement-budget and 20 file-lifecycle tests passed. The earlier reverse-direction limitation is superseded for this path. Other owner metadata, scene, GPU and plugin allocations, transient preparation, real process peaks and large-model performance are not fully accounted or measured. The existing 256 MiB limit and the overall P0/P1 gates remain open.

## Retired joining account handles

An external job handle may outlive the coordinator's choice to join another account. Previously it could acquire an export or surface-snap permit after the coordinator had switched, creating an untracked job on the abandoned account. Permit acquisition now checks account retirement under the reservation lock. Adoption holds that same lock through its last permit recheck and switch, and retires the former joining account only after successful admission. A pre-existing permit or failed admission leaves the old account usable. This concerns the joining account; handles belonging to the shared source continue to work normally. Existing captures, complete scene/GPU ownership and actual process peaks are separate issues.

On E: with `-j 2`, 290 core library, 26 atomic transaction, eight Undo, four replacement-budget and 20 file-lifecycle tests passed. A focused concurrent test ran 32 adoption/export-acquisition races and verified that both operations cannot succeed together; it does not exhaust all schedules. The failure fixture also confirms both job types remain acquirable after a refused adoption. The full 60-test STL export integration suite passed serially with the native runtime. The public source audit reported no errors. These results do not close the remaining P0 memory ownership or desktop measurement gates.

## Physical headroom at Open scene planning

The temporary reservation for a newly opened document's scene-planning scratch previously passed no physical RAM sample, even though the earlier archive and later scene-construction stages sampled when available. Open now obtains a fresh sample at this allocation boundary and supplies it to the same shared-account admission; the sample is not an OS reservation and can become stale. When no sample is available this path still falls back to managed-only admission, so it does not prove physical headroom on unsupported platforms.

A controlled Open fixture injects a 1 GiB total / 256 MiB available sample only at the planning boundary after the archive has been read. The operation refuses before publication, preserves the old coordinator snapshot and Redo, and releases the temporary reservation; a retry with sufficient sampled headroom succeeds. Bypassing the injected sample made the refusal assertion fail; applying it passed. On E: with `-j 2`, all six replacement-budget unit tests, 20 file-lifecycle tests and the `om9-archive` suite passed. The fixture simulates sampled pressure rather than allocating until real OS pressure. Allocator and native peaks, complete live-scene ownership, the 256 MiB cap and the full P0 gate remain open.

## New candidate preparation reservation

New previously constructed the replacement controller and scene in a separate account before the shared account had reserved the candidate's memory. It now reserves the known fixed controller/workspace overhead, the new document's validation-peak estimate and scene-planning scratch before planning; it grows that reservation for the estimated scene peak before controller construction. Physical RAM is sampled for these admissions when available. The reservation remains live until the candidate is transferred under the existing shared-account publication guard. The former unreserved publication path was removed.

The preparation estimate is conservative for these identified owners, not a measured upper bound for all dynamic controller, allocator, native or GPU allocations. Missing physical samples still allow managed-only admission, and the existing 256 MiB aggregate limit remains. An injected pre-construction cancellation confirms a peer sees the reservation before construction, then confirms reservation release and unchanged snapshot/Redo. A competing-reservation fixture checks refusal before publication and successful retry. On E: with `-j 2`, six replacement-budget tests, 20 file-lifecycle tests, 26 atomic-transaction tests and eight Undo-memory tests passed. The first core test command used incorrect test-target names and was rerun with the correct names; the corrected run passed. No real OS pressure, RSS peak or desktop test is claimed.

## Open candidate preflight scratch

Open's candidate scene preflight first validates the restored document and then runs the scene planner. Its shared reservation formerly covered the planner estimate but not a larger validation-scratch estimate. The scene API now reports the larger of those two estimates, and Open reserves that amount alongside the already-owned archive/document reservation. The phases run sequentially, so the two temporary estimates are not summed. The document storage itself is not added again by this scratch helper.

A scene fixture checks an empty document whose validation estimate is larger than its planner estimate and checks a 200-point fixture against the maximum of both estimates. Six scene-admission tests, six replacement-budget tests and 20 file-lifecycle tests passed on E: with `-j 2`; the existing controlled Open pressure fixture checks refusal and preservation of snapshot/Redo. The estimates do not establish real allocator or native peaks, account for every shared owner, or remove the 256 MiB cap. Missing physical-RAM samples still fall back to managed-only admission. No desktop or RSS measurement was performed.
