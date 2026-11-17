# Boolean captured sources: scoped worker foundation

Checked OCCT Boolean now validates the canonical B-rep inputs instead of requiring
valid input display meshes. A fixture damages both boxes’ display representations
while keeping their sources valid: union, difference and intersection still have
volumes 1500, 500 and 500 mm³; certified union also has volume 1500 mm³.
The document must still validate a result representation before publication.

A new cancellable captured-source API accepts two independently retained source
leases. Admission counts both inputs, callback-time output, and final retained
output. A controller fixture drops its document and both display owners before
running the worker; source-only intersection returns volume 500 mm³. It checks
initial and output admission refusal, cancellation, and that the managed result
reservation persists until its owner is dropped. An in-flight native call itself
is not interruptible.

Validation: `cargo test -p om9-kernel --features 3dm-brep --test l2_01_10 -j 2 boolean`:
17 passed, one pre-existing nonuniform affine Boolean test ignored.
`cargo test -p om9-app --test solid -j 2 boolean`: two passed.
`cargo check -p om9-app --lib --no-default-features -j 2`: passed with warnings.

This API is not connected to product menu/CMD Boolean execution; those routes
remain synchronous. Placement transforms, transactional publication, stale
context and all enabled Boolean variants need integration and tests. Native peak
memory, physical OOM, desktop/GPU, cold Undo, v5, NativeBrep and completion
gates C1–C5 remain open.

## Live input memory for synchronous Boolean

The direct Boolean path counts the retained source, topology and display allocations of both legacy input values before validating canonical sources and allocating an output. Exact shared allocation clones are counted once; partially shared values are conservatively counted twice. A fixture verifies that two individually valid sources are rejected under a limit below their combined live input storage, and that a generous limit produces the expected intersection. This accounts managed Rust inputs and output budgets, not OCCT native peak or physical RAM.

## Captured placement preparation

A cancellable transform takes a canonical source lease, accounts the source and
candidate while copying native output, and returns a candidate whose reservation
the caller retains. A placed two-input Boolean composes this with checked source
Boolean, counting intermediate transformed bodies. Shared canonical allocations
are counted once; native work already in progress is not interruptible.

A worker fixture drops the document and display before translating a box,
checks position and volume, cancellation, admission refusal and retained result
charge. Another checks independently placed boxes (intersection 700 mm³),
shared-source placement, invalid certified operation, cancellation and refused
placement admission. The `solid` captured-filter group passed five tests; the
kernel transform-filter group passed one. Product Boolean commands do not yet
call these APIs. Multi-input/group operations, transaction integration, real
OOM, native peak, cold history and C1–C5 remain open.
