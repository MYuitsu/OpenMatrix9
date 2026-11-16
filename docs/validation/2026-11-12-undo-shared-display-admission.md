# Undo shared-display admission diagnosis — 2026-11-12

The earlier B0 debug measurement observed a 100,678 ms first Undo in a
128-Sphere synthetic workload. A small number of execution samples landed in
allocation-free retained-document accounting; they do not establish how much
of that latency the accounting caused. The long measurement was not repeated.

The existing `native_display_sharing_is_accounted_across_world_capture_and_history`
core test now also asserts that, for its shared native-display fixture and tight
byte limit, the duplicate-inclusive preflight already fails before charging
temporary indexes, whereas exact retained accounting fits and the actual
validation succeeds with the same usage. Undo and Redo then succeed. This
demonstrates the mechanism in a bounded synthetic fixture; it does not measure
the Sphere fixture or establish a desktop performance improvement.

The index must be admitted alongside all retained owners and other reservations.
Simply omitting repeated displays from the preflight without accounting for the
actual retained allocations could violate the memory ceiling. The existing
allocation-free fallback and all production behavior remain unchanged. A future
optimization needs a bounded, admitted way to inventory distinct allocations,
plus comparisons against exact accounting and low-memory, concurrent-owner,
Undo/Redo and performance tests. Source-only restoration and display eviction
remain deferred by the existing Plan B gate.

Windows x64, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`: the focused test passed;
`cargo test -p om9-core --lib -j 2` passed 377 tests with one ignored. Public
source audit reported no errors. Artifact pruning removed no runs; 169 older
runs remain because existing records cite them. No desktop measurement, real
memory exhaustion test, release benchmark or application build was performed.
