# MatrixApp Boolean Union worker

The MatrixApp Boolean Union command captures canonical B-rep source leases and
placement information, then performs a certified multi-input union on a worker.
An operation-frame common affine placement is handled by the existing placement
rules. Source validation, intermediate transformed bodies and outputs use
managed admission; duplicate canonical inputs are conservatively counted per
occurrence. The UI supplies additional reservations without blocking its frame.
The worker cannot interrupt an OCCT call already in progress.

Before publication the controller checks command identity, session authority,
epoch, scene publication, selection revision, source stamps, current layer and
DeleteInput. It commits one transaction, retaining the output reservation during
the commit. CMD draft changes cancel the worker; explicit cancel drops it. Errors
leave the pending command available for retry.

Headless controller fixtures exercised three overlapping box occurrences with
different placements, DeleteInput, exact union volume 2000 mm³, Undo/Redo, stale
selection, explicit cancel and simulated managed memory refusal. The headless
MatrixApp fixture exercised both menu and CMD invocation, Undo/Redo and changed
CMD draft. `cargo test -p om9-app --lib -j 2 product_union_worker` passed three
tests; `boolean_union_ui_worker_commits_and_rejects_changed_draft` passed one.
The no-default-features app check passed with existing warnings.

The synchronous headless Controller compatibility path and other Boolean
commands, including Difference, Intersection, Split and Boolean Two Objects,
remain separate. This evidence does not establish all shapes or placements,
desktop/GPU behavior, native peak memory, physical OOM, cold Undo, v5, NativeBrep
or completion gates C1–C5. Document and Undo still retain display.
