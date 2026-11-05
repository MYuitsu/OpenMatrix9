# STL soup and snapshot coexistence

## Cancellable authored-mesh preflight in the worker

The experimental STL worker now counts authored-mesh triangles and checks its reservation floor inside the worker, after the launch-time physical-memory recheck. Previously the face scan ran on the caller thread before `start` returned. The scan checks cancellation at least every 1,024 faces, including quads and repeated instances. Insufficient mesh reservation is now reported by the worker result rather than synchronously by `start`; the destination remains untouched and the export permit and reservation are released. Invalid profile, account, unsampled reservation and launch physical-memory failures remain synchronous.

Thirty-six focused cancellation/worker tests passed, including an injected cancellation during a 4,096-face scan, the asynchronously reported reservation-floor error, preservation of an existing destination, worker panic and permit release. All 60 serial STL integration cases passed in 109.50 seconds, including mesh, B-rep, NURBS, captured-source and destination-preservation fixtures. This verifies the worker foundation only: the application UI still uses synchronous export, its separate scan remains synchronous, and the background path is still gated on incomplete native/concurrent memory accounting and lifecycle integration. No desktop responsiveness or general large-model performance is claimed.

Before creating the next B-rep snapshot, STL export now subtracts the allocated capacities of its existing point and triangle arrays from the available byte limit. The same subtraction applies before converting retained NativeBrep or NURBS to a temporary kernel source; the temporary source string's capacity is also subtracted when tessellating that source. The check uses capacities rather than lengths and performs checked arithmetic. It applies to both the synchronous path and the experimental worker, whose byte limit is separately capped by its reservation.

A fixture holds a large existing point allocation and leaves only 128 bytes for the next source. It failed before the change because the snapshot was still accepted; afterwards it is refused with existing arrays untouched. The final test also holds triangle capacity and checks that both capacities are deducted. This is a necessary coexistence check, not a complete export budget: the output buffer growth after the snapshot, topology remap, diagnostics, kernel-side allocations and OCCT transients are not fully reserved together. Allocator overhead and physical process peak remain unmeasured; no UI worker admission is implied.

On the external E: target with `-j 2` and default native features, 27 STL session tests and all 60 serial STL integration tests passed. The strengthened two-capacity fixture was rerun after the full suite. NativeBrep and NURBS successful export are covered by integration fixtures, but separate memory-pressure tests for their temporary source strings remain outstanding. The public-source audit found no errors.

## Topology remap minimum

After a grouped snapshot is created, the STL path now checks the retained Rust snapshot plus the minimum storage of its three simultaneous indexed arrays: one `u32` parent, one `f64` radius and one `u32` shell value per snapshot point. Existing soup and any temporary source were already deducted from the limits supplied to this check. A real box snapshot fixture showed that a budget one byte below this combined minimum previously proceeded; the new path refuses, while the exact minimum produces the same remap as the existing algorithm. Cancellation retains priority.

This minimum does not include topology lookup maps, allocator over-allocation, output growth after remap, native memory or the cost of constructing the snapshot before the remap check. A budget equal to this minimum is not a proof that the operation fits in physical memory. The legacy unbudgeted remap entry point remains available to other callers; this change wires the STL export path only.

On the external E: target with `-j 2`, 11 kernel unit tests, 27 STL session tests and all 60 serial STL integration tests passed. The boundary fixture failed before adding the combined check and passed afterwards, and verifies the admitted mapping matches the previous algorithm. Public-source audit reported no errors.

## Topology lookup estimate

The budgeted STL remap now adds estimated BTree storage for both topology lookup maps, using at most one site entry per eligible binding and one vertex entry per vertex binding. The estimate uses the project's conservative pinned-Rust BTree inventory; duplicate bindings and unused shell entries can make it larger than actual retained storage. The loop checks cancellation periodically. A box fixture reproduces the earlier gap: a budget sufficient for the snapshot and three arrays alone no longer admits the lookup stage. At the estimated threshold, the mapping matches the pre-existing unbudgeted routine.

This supersedes the previous minimum-only admission for the STL path; the earlier boundary test remains historical. It is still an estimate, not a physical allocation limit: allocator behavior, transient BTree allocations, output growth and OCCT memory are not completely bounded, and the synchronous legacy remap entry point has not been migrated. The additional binding scan has not been benchmarked on a large model.

On E: with `-j 2`, 13 kernel unit tests, 27 STL session tests and 60 serial STL integration tests passed. The lookup boundary test was verified red with the previous arrays-only admission, then green with lookup estimation; separate tests cover cancellation during the scan and an empty binding set. No process peak or desktop measurement was taken for this change.

## B-rep output buffer growth

After topology remap and before appending a B-rep, STL now checks the capacities of existing output arrays, transient source points/triangles and the remap. If a destination vector must grow, the preflight includes the old allocation and a new allocation of the minimum required capacity at the same time. Triangle and point growth are admitted separately, and actual retained capacities are rechecked after reservation; the first source can still reuse its point buffer. A red/green fixture for a large existing triangle array and a separate point-array fixture verify refusal without changing existing arrays when the old and required new buffers cannot both fit.

This is a conservative check for these known buffers, not a complete process or worker reservation. An allocator may over-allocate before the post-allocation check, and diagnostic strings, topology maps at earlier stages, authored-mesh growth and OCCT transients require separate accounting. Real process high-water and large-model throughput have not been measured for this change.

On the external E: target with default native features and `-j 2`, 29 STL session tests and 60 serial STL integration tests passed. The triangle-growth fixture failed with preflight disabled and passed after restoring it; the point-growth fixture also passed. The fixtures exercise Rust-owned arrays and do not simulate allocator failure or prove a process-RSS ceiling.

## Authored-mesh output growth

For a user-authored mesh, the exporter now checks both sequential growth peaks before allocating: existing point/triangle capacities plus the planned new point buffer, then the planned point capacity plus existing and planned new triangle buffers. It rechecks actual capacity after each buffer allocation and before writing geometry. Normal growth uses a doubling policy to avoid reallocating for every repeated mesh; when that plan does not fit, the minimum required capacity is tried instead. The immutable input mesh is owned by the captured document and is not a new output buffer in this estimate.

A fixture with insufficient room for old and new point buffers reproduced successful growth before the check and is refused afterwards; a companion case checks triangle growth. Tests cover exact-size fallback and 256 repeated small mesh appends with bounded capacity changes and unchanged coordinates/indices. Allocator over-allocation can still temporarily exceed the planned capacity before it is rechecked, and native/output topology diagnostics, process peaks and the separate captured-document accounting remain outside this check. This does not complete the global RAM or background-export gates.

On the external E: target with default native features and `-j 2`, 32 STL session tests and 60 serial STL integration tests passed. The fallback test checks the planned capacities; the repeated-append test checks growth counts and geometry, not elapsed time. The public-source audit reported no errors. B-rep output growth still uses exact reservations and its repeated-instance performance requires separate review.

The two focused mesh-growth tests also passed with `--no-default-features`; that auxiliary build emitted 18 warnings. It does not exercise OCCT and is not a substitute for the default-feature results.

## Amortized B-rep output growth

The earlier exact-capacity B-rep change caused 64 successive box snapshots to grow both output vectors 64 times. B-rep output now plans the point and triangle growth peaks together, prefers doubling and falls back to both exact requirements when the larger plan does not fit. It rechecks actual capacities between allocations; the first B-rep may still reuse its snapshot point buffer. A threshold fixture verifies the combined exact fallback is selected when increasing the first buffer geometrically would leave too little for the second.

The 64-B-rep fixture retains coordinates and triangle indices and checks that each vector changes capacity at most ten times; this is a count, not a timing benchmark. This supersedes the earlier note that B-rep output always reserves exact capacity. Allocator over-allocation, larger repeated models, topology diagnostics and OCCT transients still need measurement and full shared-account admission.

On the external E: target with default native features and `-j 2`, 34 STL session tests and 60 serial STL integration tests passed. The repeated-B-rep fixture was red at 64 point and 64 triangle capacity changes before the fix, and passed its ten-change ceiling afterwards. The exact-fallback fixture checks planning at and below the combined threshold; allocation timing was not measured. Public-source audit reported no errors.

## Indexed-soup topology minimum

Before starting indexed topology analysis, STL export now checks the retained capacities of the completed soup plus the minimum `usize` parent and `u8` rank array for each triangle. The check is applied to authored and generated geometry, respects cancellation first, and runs before the analysis allocates those arrays. A synthetic open triangle fails one byte below this minimum and at the threshold produces the same boundary/topology result as the existing routine. The below-threshold case was accepted before the check.

This is only a lower bound. Edge/face hash tables, vertex fans, per-fan adjacency, allocator over-allocation, warnings, native transients and the later encoded-output analysis are not included; an exactly admitted minimum is not a guarantee that the full analysis fits. The work does not complete aggregate reservation or authorize UI background export.

On the external E: target with default native features and `-j 2`, 35 STL session tests and 60 serial STL integration tests passed. A separate full-export fixture then verified that a valid mesh can be built but insufficient topology headroom returns a budget error while preserving an existing destination. Public-source audit reported no errors.

## Indexed edge-stage capacity admission

The indexed topology routine now exposes an admission callback at edge-stage allocation boundaries. For the STL path it checks the live soup together with the actual capacities of the parent/rank arrays and face/edge hash tables. Before a hash table grows, the lower bound includes both the existing capacity and at least the requested new slot storage; after growth, it checks the capacity reported by the table. Existing unbudgeted callers retain the same topology interface and results. Admission checks occur on growth rather than on every triangle.

This accounts only for a known lower bound of the hash-table slots: control bytes, hidden bucket allocation, allocation rounding and temporary allocator peaks are not known through the standard API. The later vertex-fan/adjacency stage and encoded-output analysis remain outside this check. Admission happens within the current per-export limit, not a complete independently enforced process-wide allocation quota. The earlier array-only topology evidence remains historical.

On E: with `-j 2`, the full `om9-exchange` test suite, 36 STL session tests and 60 serial STL integration tests passed. Two additional exchange fixtures confirm that a face table can fit while the next edge-table allocation is refused, and that an admitted tetrahedron reports closed indexed topology. The array-only test was red before connecting the new admission callback and passed after; no performance or allocator-failure measurement was made.

## Indexed vertex-fan list admission

After dropping the edge-stage tables, the same callback now checks known storage for the fan hash table and the aggregate capacities of per-vertex link vectors. Before a table or link vector grows, it includes the existing allocation and the minimum requested replacement; after growth it checks the reported capacity. Link capacities are maintained incrementally rather than rescanning every fan for each triangle.

A fixture of 100 coincident indexed triangles fits the edge-stage lower bound but grows fan lists beyond the supplied allowance. It completed before this change and is now refused; with admission allowed, it still reports 99 duplicates, three non-manifold edges and one edge-connected component. A closed tetrahedron fixture also passes. This remains a lower-bound check: hidden hash buckets, control bytes, allocator rounding, per-fan adjacency, traversal scratch and encoded-output analysis are not covered, and it is not a process-memory guarantee.

On E: with `-j 2`, all three focused exchange fixtures, the full `om9-exchange` suite, 36 STL session tests and 60 serial STL integration tests passed. This is not evidence of desktop performance or a complete reservation for all export allocations.

## Indexed fan adjacency and traversal admission

While the fan lists remain live, indexed topology checks known adjacency table and vector capacities as well as the visited set and pending traversal vector. Checks before growth include old capacity and the minimum replacement; checks after growth include the reported new capacity. Capacity tracking is incremental. Hash-table control bytes, hidden buckets, allocator rounding and transient allocations remain outside this lower bound. Encoded-output analysis and native allocations are also not covered; this is not an RSS or process-wide guarantee.

A 100-duplicate-face fixture fit the fan-list checks at a 4,000-byte allowance but completed without accounting for the larger adjacency vectors. It failed the new expectation before this change and is now refused. The admitted tetrahedron and non-manifold result fixtures retain their topology results. This does not establish accuracy of all native or encoded-output topology paths.

On E: with `-j 2`, four focused exchange fixtures, the `om9-exchange` suite, 36 STL session tests and 60 serial STL integration tests passed. Some exchange fixtures requiring separately pinned native sources remain ignored. The public source audit passed. No desktop presentation or process peak was measured.

## Quantized-position table admission during STL writing

The streaming STL writer maintains a hash table of encoded `f32` positions to reject collisions and report distinct indices with coincident encoded positions. This table formerly grew without consulting the export byte limit. The writer now accepts an admission callback: before table growth it reports the known old slot capacity plus minimum requested replacement, and afterward the capacity reported by the table. Strict writer callers also route their topology scratch through that callback. Existing writer entrypoints retain an unrestricted wrapper; the application STL path supplies its export limit and counts the retained soup capacities alongside the table.

An exchange fixture refuses the first table allocation and checks an admitted triangle still produces the expected binary length. An application fixture forces that refusal after the temporary file is created and verifies the original destination remains unchanged and the temporary file is removed. This is a lower bound: hash control bytes and hidden buckets, allocator peaks, buffered writer memory, other export owners, native memory and later encoded-output verification are not all accounted. The per-export limit is not a complete process-wide RAM ceiling, and this does not authorize background export UI integration.

On E: with `-j 2`, all five targeted exchange admission fixtures, the `om9-exchange` suite, 37 STL session tests and 60 serial STL integration tests passed. Some unrelated exchange fixtures that require pinned native inputs were ignored. Public-source audit passed and artifact pruning removed no runs. No actual process-memory peak or desktop performance was measured.

## STL streaming and staged verification buffers

The application now includes an explicit 8 KiB writer-buffer capacity alongside the live soup and encoded-position table when checking the per-export byte limit. It checks the requested capacity before creating the temporary file and checks any larger reported capacity after creating the writer. A controlled refusal verifies the existing target is unchanged and no temporary file is left; the table-refusal fixture separately allows the writer buffer and still verifies cleanup. The final staged verification reads directly in 16,000-byte stack blocks instead of allocating an additional `BufReader` heap buffer.

The stack block, allocator rounding, writer structures, path allocations, hash metadata, other export owners and native memory are not fully accounted. The buffer-size check is a per-export estimate, not a process-RSS guarantee; directly reading the file has not been benchmarked. Verification still checks staged bytes, records and digest, not global geometric deviation, self-intersection or encoded topology certification.

On E: with `-j 2`, all 38 STL session tests and 60 serial STL integration tests passed. Public-source audit passed and artifact pruning removed no runs.

## Receipt notes and output path during STL validation

After soup creation, the STL capture deducts the actual capacities of skipped-object and warning vectors and their strings before topology validation. It also deducts the output path capacity before opening the writer, so these live allocations are considered alongside soup, the writer buffer and known topology and encoded-table slots. A fixture with 60 skipped points shows soup construction succeeding while already-created notes exceed a 10,000-byte export limit; export refuses before publication and preserves the existing target.

This is late accounting: the notes were already allocated by `stl_soup`, and their growth, allocator rounding, selection capture, extra paths, native memory and other owners are not fully admitted. It cannot guarantee peak RAM or justify worker/UI integration. Per-export limits remain partial, and the 256 MiB cap and full P0 gate remain open.

On E: with `-j 2`, 39 STL session tests and 60 serial STL integration tests passed. The public-source audit passed; artifact pruning removed no runs.

## Skipped-object note allocation

Skipped-object notes now check known existing soup and receipt capacities before growing the note vector and before allocating the formatted string. Vector reallocation includes the old and requested replacement buffers; both vector and string reported capacities are checked after allocation. Allocation failures return a budget error. A one-byte fixture verifies refusal without allocating the vector; the 60-point fixture verifies refusal preserves an existing destination.

This only covers skipped-object notes. Warning formatting still allocates before admission, and later geometry growth does not yet subtract existing receipt notes. Allocator metadata, allocator rounding at peak, native work, instance traversal and other owners remain outside this estimate. The existing per-export cap is not an aggregate or peak-RSS guarantee.

On E: with `-j 2`, 40 STL session tests and the mixed-selection STL integration test passed. Public-source audit passed and artifact pruning removed no runs.

## Receipt notes during later geometry construction

Before processing each selected source, the STL capture subtracts retained skipped and warning vector and string capacities from the limits passed to mesh append, B-rep tessellation, and NativeBrep/NURBS source conversion and append. A fixture processes a skipped point before a triangle: the note fits and the mesh fits by itself, but their known combined capacities do not, so soup construction refuses the mesh. The fixture also checks the mesh output remains empty when refused.

This accounts already-created notes in these geometry checks; warning creation itself still occurs before admission. It does not count allocator metadata, all temporary and native allocations, traversal storage, or total RSS. It does not complete P0 or justify enabling the export worker in the UI.

On E: with `-j 2`, 41 STL session tests and all 60 serial STL integration tests passed, including native B-rep and NURBS fixtures. Public-source audit passed and artifact pruning removed no runs.

## Warning note allocation

STL warning formatting now counts output bytes without allocating a formatted string, checks the known soup and receipt capacities before reserving vector and string storage, and checks their reported capacities afterward. A focused fixture verifies that a one-byte budget rejects a warning without allocating its vector and that an admitted warning retains its expected text. The same path is used for tolerance, deflection, open-surface, hidden-object and omitted-triangle warnings.

This accounts only the known soup and receipt buffers at warning creation; the temporary native source still live in some branches, allocator metadata, actual allocator peaks, native work, traversal and other export owners are not fully admitted. The limit remains partial and is not an RSS guarantee; P0 and worker UI integration remain open.

On E: with `-j 2`, 42 STL session tests and all 60 serial STL integration tests passed. Public-source audit passed and artifact pruning removed no runs.

## Converted native source lifetime

After tessellating a retained NativeBrep or converted NURBS surface, the temporary serialized OCCT source is explicitly dropped before allocating receipt warnings. The original captured document geometry remains available, and the output triangles and tolerance metadata are retained. This closes the specific overlap described in the warning allocation section; it does not demonstrate that the allocator returned pages to the OS or measure peak process memory. OCCT internals and other owners remain outside the partial budget.

On E: with `-j 2`, two retained-native B-rep and five NURBS STL integration tests passed. Public-source audit passed and artifact pruning removed no runs.

## Temporary output path during STL writing

The writer now includes the temporary path's reported capacity alongside the soup and write buffer before creating the file, and the cleanup guard owns the same path rather than a clone. A tight-budget fixture verifies rejection preserves the old destination and creates no temporary file. Cancellation and other cleanup tests still exercise the guard.

The temporary path and short formatted name are already allocated when their capacity is checked, and OS path conversion, allocator overhead and other export owners are not fully accounted. This is a partial per-export estimate, not a process-memory guarantee.

On E: with `-j 2`, all 43 STL session tests and two targeted integration tests for replacement and failure preservation passed. Public-source audit passed and artifact pruning removed no runs.

## Synchronous UI export reservation

The existing synchronous STL UI route and the Controller's selected-export wrapper now acquire the remaining managed export headroom with a physical-memory sample, deduct known scene/auxiliary usage, and retain the reservation for the duration of export. The capture's per-export byte limit is capped by that reservation; the existing authored-mesh minimum is checked before export. The controller also checks that the capture belongs to its account. A synthetic competing reservation exhausts headroom: export refuses without changing the target, releases the capture's reservation, and succeeds after the competing reservation is dropped.

This only wires the UI and Controller wrapper: direct calls to the public capture export remain outside this reservation path. The UI operation remains synchronous, and the existing 256 MiB limit, uncaptured owners, OCCT internal peak and physical-memory sampling races remain open. A reservation is not proof that native allocations or process RSS stay within it; background UI integration remains gated. No physical desktop interaction was tested.

On E: with `-j 2`, all 44 STL session tests and 60 serial STL integration tests passed. Public-source audit passed and artifact pruning removed no runs.

## Shared synchronous and worker export gate

The synchronous Controller/UI route now holds the same account-level export permit used by the background worker, from admission through publication. A focused test holds the worker permit and verifies the synchronous route reports an active export without touching the destination; it also checks a missing RAM sample releases the permit and a retry succeeds. This is a controlled permit test, not a measurement of two native exports running concurrently.

Direct public capture export still bypasses this gate. The present UI export remains synchronous and cannot yet expose background cancellation; native peak, remaining owners and desktop behavior remain open. The feature is not a complete implementation of the one-job export contract.

On E: with `-j 2`, all 45 STL session tests, four worker integration tests and one menu/CMD dispatch test passed. Public-source audit passed and artifact pruning removed no runs.

## Direct capture export gate

The public captured-source export entrypoints now acquire the same account-level permit as the Controller/UI and background worker routes. Worker and Controller code call a private implementation while holding their own permit, avoiding nested acquisition. A controlled held-permit fixture checks that both direct and synchronous routes report an active export and preserve the destination; the existing failure and retry checks cover permit release.

Direct capture export still does not obtain the Controller's managed-memory reservation or sample physical RAM. The UI remains synchronous without background cancellation, and real desktop concurrency and native peak remain unverified. This closes a gate bypass, not the full export job and memory contract.

On E: with `-j 2`, all 45 STL session tests and 60 serial STL integration tests passed. Public-source audit passed and artifact pruning removed no runs.

## Running worker and competing export routes

A controlled test now starts an actual STL worker thread with an export permit, captured triangle source and managed reservation, and pauses it before writing. While that worker is alive, both a direct captured-source export and the synchronous Controller route report an active export without changing the competing target. After release, the worker writes one triangle and exits, its reservation is released, and the previously blocked capture can export.

This exercises thread and permit lifetime rather than just holding a permit in the calling thread. The worker is deliberately paused before its actual write; it does not prove desktop event routing, cancellation UI, native-call overlap, peak memory or complete admission for the direct capture route. On E: with `-j 2`, all 46 STL session tests passed; public-source audit passed and artifact pruning removed no runs.
