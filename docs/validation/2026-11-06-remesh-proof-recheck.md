# Remesh proof recheck: release fixture

- One OCCT sphere, radius 3 mm, creation deflection 0.005 mm; 5,968 display triangles.
- Windows, existing `L:\om9-target` release test target, `cargo test --release --locked -p om9-kernel -j 2 --lib registered_proof_validation_release_cost -- --ignored --nocapture`.
- 31 sequential samples in one process: full validation median 681,000 ns, p95 692,000 ns; immutable-owner check median 100 ns, p95 200 ns. The system timer has coarse resolution at the latter scale.
- Full validation still occurs once when the proof is adopted. The fast check requires the same account, transaction context, weak document allocation, object stamp and previously validated display fingerprint. A test verifies `Arc::make_mut` detaches the document even when only one strong owner remains and a Weak is present. Transaction/Undo changes reject the proof.
- This isolates one proof check, not whole remesh latency, peak memory, GPU time or desktop presentation. It is not a large or complex production fixture and does not establish a frame-rate improvement.
