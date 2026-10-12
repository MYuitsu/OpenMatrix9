# Saved Surface History admission review

Independent read-only reviewer: `/root/surface_history_admission_review`.

Initial P2 finding: persisted input.closed hints were used as actual native
rail closure in Construct policy. A valid closed Sweep could be rejected after
only its saved hint changed. Root reproduced this with portable RED and actual
native closed Sweep1/Sweep2 after valid baselines (52 checks, two failures).

Fix: Stage::Saved shares existing scalar/cardinality policy and defers closure
facts until native Stage::Construct observes the current wires. Saved metadata
is retained; no fake rail fact is substituted. FFI construction still accepts
only stages1–3. Re-review found no other important defect. The reviewer did not
build or launch a host; coordinator verification is recorded in the receipt.

Existing diagnostic regression caught lost concrete operation/source context
after early saved policy rejection. Rust now returns the kind of a complete
rejected request without writing FFI output; the native adapter adds only linked
identity to the error. Original Loft/FirstProfile.Edge1 assertions are preserved.

Limits: this review does not close original all-family results, conversions,
ownership, lifetime or RCORE-01 acceptance requirements.
