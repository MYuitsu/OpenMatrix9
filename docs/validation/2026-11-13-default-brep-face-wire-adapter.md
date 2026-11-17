# Default B-rep face-wire adapter — implementation checkpoint

2026-11-13. I1 remains open. This checkpoint adds a synchronous OCCT adapter, not a product-visible isocurve implementation or source/display ownership separation.

The adapter reads one immutable OCCT source, visits its faces in that read, chooses interior B-spline knots or a midpoint in each face UV direction, clips each curve against its face with OCCT Common, and returns bounded Rust-owned NURBS curves. It does not return native face indices or pretend the curves are selectable topology edges. Rust retains admission/error handling and owns returned data; OCCT is used for the native face and trim operations. The caller must keep the source alive and guard publication. Request, topology and returned output limits apply; native temporary peak and runtime are not bounded by Rust's managed-byte accounting.

Default density here is an implementation candidate: exact behavior for wrapped/periodic surfaces, non-default imported density, and equivalence with the existing standalone NURBS wire parameter policy are not yet established. Failed or wholly empty clipping is refused. The function currently has no product caller: neither scene nor ExtractWireframe uses it. It does not solve topology proof, cache ownership, undo restoration, v5 migration, NativeBrep import, or background scheduling.

## Evidence

- `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`, default `om9-app` features, OCCT SDK runtime on PATH: `cargo test -p om9-app -j 2 --test isocurve` returned 54 passed, 4 ignored. After adding the quality-independence assertion, rerunning the named test returned 1 passed, 57 filtered.
- The new test checks 12 Box face wires, sampled points on a cylinder, four lines clipped around a planar hole, two Box display deflections using the same source, invalid source, output limit and refused Rust admission. Sampling does not prove a global geometric error bound. No GPU, desktop, menu, persistence or native memory peak was tested for this adapter.
- `python tools/public_source_audit.py`: 15,982 source files, 594 bindings, 593 SVG files, no errors. `git diff --check` passed for the touched source files, with Git line-ending warnings. `python tools/prune_build_artifacts.py --apply`: removed zero runs; cited runs retained by policy.
- SHA-256: `editing_bridge.cpp` `21bfc79784605e48f4ca7e1c38247eb5d59403aa83d04a2d320103f63de5dec3`; `editing.rs` `3e4e4fd28b2249ec35032301cd5afa8e5eb5aefd3d821302d48e4330f4d038e1`; `isocurve.rs` `e9eeaeb3521a8138c4b7d9c5ecd566c8fd40998dfb6f3ecacc7a3a7280a1b15b`.

Next: resolve bounded native temporary admission and wrapped/periodic/default-density policy; integrate one shared representation into worker/scene and ExtractWireframe with source binding, placement, stale/cancel, and nonselectable-isocurve checks. Then validate product paths and continue C1–C5. The full plan remains open.

## Scene preparation and cooperative checkpoint follow-up

The adapter now calls the caller's admission callback between faces and before/after each face clipping operation. A callback refusal stops traversal; one native OCCT call in progress is not interrupted. Callers that need the original error must retain it in their callback, as with other editing adapters. A trimmed B-spline fixture verifies the chosen interior knot instead of the face midpoint in one axis.

`PreparedBrepBuffers::with_interior_wires` accepts caller-owned sampled polylines, preserves the original display vertex/edge prefix, separately admits old/new buffer overlap, and adds render-only lines. Picking skips appended lines for a B-rep and still verifies the original segment mapping. The method does not itself attest that its input wires belong to the source: a future worker must bind source/profile and own/account its input through preparation. Ordinary scene creation does not call this method yet.

In a test-only source-to-scene path, 12 source-derived Box wires were sampled into the prepared buffer, and two edge-picking APIs returned no edge at the top face center. A separate scene test checked an actual topology edge remains selectable, malformed input and admission refusal fail, and old/new allocation overlap is reported. These tests do not exercise real GPU presentation, worker scheduling, trim integration, NativeBrep, or physical desktop input.

- `cargo test -p om9-scene -j 2 --lib`: 47 passed. `cargo test -p om9-app -j 2 --test isocurve`: 55 passed, 4 ignored. After adding source-to-scene assertions, the two named `default_brep_` tests both passed. Default features, `L:\om9-target`, OCCT SDK runtime on PATH.
- Public source audit returned zero errors; artifact pruning removed zero runs. `git diff --check` returned no errors, with CRLF warnings. The source hashes above identify the earlier checkpoint; current SHA-256: `editing_bridge.cpp` `c730d5b5ba72314a4bfce6cf9ef3720972c6653b85880aabab900fff27515a25`; `editing.rs` `0b6be663661aa9104bc9702fff6ee2e6503b143c96feed619800fa56021cfd2c`; `isocurve.rs` `d8245bbba94466017eb6d2469381d84700c97e18dc56805502ff72af952ffe5f`; `scene.rs` `d7231b025b07b5f83a3283a9dd51b42b4124d2e2333be71f2982cbb83e3bf8bc`; `picking.rs` `ff83ba280ba8331cbc6d316fa6d465442e65b79df6f2cb1813f0203a72533e2a`.
