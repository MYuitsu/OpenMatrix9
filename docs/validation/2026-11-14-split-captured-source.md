# Source-only split and admitted piece extraction

The kernel can split two captured OCCT source leases without retaining either
document display. It validates source bindings, checks both are closed single
solids, admits the compound snapshot, and checks the reported solid count and
preserved target volume. A companion operation splits retained intermediate
B-reps with both live input representations charged. Another extracts one
compound-local piece with the compound and output charged, and checks that the
piece is a positive-volume closed solid. The caller must separately account for
other live pieces and retain each output reservation. Cancellation is observed
between native calls and during admitted copies; an in-flight OCCT call is not
interruptible.

In one worker fixture, two overlapping boxes were split after the document and
both input displays were dropped. The compound had two solids and volume
1000 mm³; its two extracted pieces each had volume 500 mm³. The fixture also
checks pre-cancellation, source and output admission refusal, invalid piece
index, intermediate B-rep split and retained output reservation. The targeted
app test passed. The existing `l2_01_10` split filter passed eight tests with
`--features 3dm-brep`.

At this checkpoint these were low-level building blocks, before the product worker. The
multi-target/multi-cutter loop, placement handling, simultaneous piece accounting,
deduplication, UI scheduling and atomic transaction publication still need to be
integrated. The subsequent [MatrixApp Split worker](2026-11-14-boolean-split-ui-worker.md)
addresses that route in scoped fixtures. Native peak, physical OOM, desktop/GPU, Boolean Two Objects and
gates C1–C5 are not established by this evidence.
