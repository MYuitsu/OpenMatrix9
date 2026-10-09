# Hatch current-loop Rust input gate review — 2026-10-10

Reviewed baseline: `32a72a6`, with the frozen current-loop gate increment and the subsequent JSON-depth compatibility repair. Review result: no remaining portable-source blocker identified. The concrete depth compatibility finding was reproduced and repaired during review. This conclusion covers the current Hatch loop input gate; it does not close full `RCORE-01.T07`.

This was a read-only review of production and test source. The reviewer did not edit production/tests, invoke a build, run a test executable, open the GUI, or delegate further work. The test results below were read from the implementer's persisted logs and exit records. Final SDK/host artifact matching remains the integrating agent's responsibility.

## Finding and resolution

**Resolved P1 — JSON container depth rejected previously supported curve depth.** The original Hatch call used the shared parser's default JSON depth 64, while the native decoder separately permits curve-node depth below 64 (`Gui/ThreeDmHatchLoopEdit.cpp:92–105`). Each nested PolyCurve adds both an object and a `segments` array. For a root loop payload whose leaf is the portable NURBS helper, the deepest CV scalar has JSON depth `6 + 2*N`, where `N` is the number of PolyCurve parents. Thirty parents therefore exceed the original JSON ceiling while remaining within native curve depth. A depth-20 control would not reproduce this regression.

The current source calls `parse_document_at_depth(..., 256)` only for Hatch (`rust/src/hatch_loop_input.rs:200–204`). The existing helper is `pub(crate)`; all other `parse_document` callers retain depth 64 (`rust/src/builder_history.rs:305–317`). The new test supplies complete typed PolyCurve fields, accepts 63 parent curves and rejects the next parent at curve depth 64 (`rust/tests/hatch_loop_input.rs:105–118`). The stored compatibility RED has 16 passes and one failure; the focused repair GREEN has 17 passes. A further finite tiny-weight overflow test increases the final Hatch suite to 18 passes.

No further blocking arithmetic, indexing, ownership, or FFI defect was found in the reviewed portable scope. The earlier mutable Qt lookup insertion regression was independently repaired by the integrating/native reviewer; its final native regressions pass. This review does not substitute for that native review.

## Basis convention and validation

- The raw openNURBS knot cardinality is checked as `CVCount + order - 2` with checked arithmetic before extraction vectors are allocated (`hatch_loop_input.rs:84–114`). `CVCount >= order >= 2` makes the active-domain indexes `order-2` and `CVCount-1` valid. The supplied domain must equal these native active endpoints exactly, then the shared validator checks finite increasing domain and knot structure.
- The adapter restores one omitted outer observation at each end, then compresses repeated knots into distinct knots and multiplicities (`hatch_loop_input.rs:138–168`). This matches the existing `NurbsBasisAdapter.cpp:176–189` observation view. Expanded openNURBS ghost poles are checked as nonperiodic structural data; the gate does not claim that the native curve is nonperiodic, nor substitute the compact OCCT periodic cardinality rule.
- `Json::Boolean(bool)` retains the rational flag rather than erasing its value. A nonrational row has exactly two coordinates; a rational row has exactly three homogeneous values `(Xw, Yw, w)`. Finite positive weights are required before dehomogenization. Shared pole validation rejects nonfinite division results, excessive coordinates and nonfinite homogeneous reconstruction. The final test at `hatch_loop_input.rs:51–54` specifically rejects a finite `1e-320` weight whose division overflows.
- Raw knots are checked for monotonic order before compression. Shared `Axis` validation enforces finite distinct gaps, endpoint/interior multiplicity bounds, checked total cardinality and active domain (`rust/src/nurbs_basis.rs:227–292`). The raw-knot count is a conservative distinct-knot upper bound for the count-only preflight. The compressed multiplicity increment cannot overflow within the one-million stored-knot ceiling.
- The shared policy uses degree at most 25, at most 500,000 CVs, at most 1,000,000 stored knots, 64 MiB logical basis bytes and Euclidean coordinate magnitude at most `1e150`. The degree policy is the existing native/shared policy; the CV limit retains Hatch's existing limit. These are validation policy bounds, not aggregate process-memory guarantees.

## Whole-input ordering and ownership

`preflight_loops` walks every row and every nested PolyCurve segment before returning success. It rejects unrecognized curve classes. All NURBS encountered later in the payload are validated before the caller can begin native archive access/copies or native reconstruction. The Rust module contains no openNURBS/OCCT types, native constructors, solver, document references or retained handles. It uses an owned JSON DOM and temporary owned basis vectors.

The gate permits only the existing Line, Arc and Polyline families without adding portable geometry or topology checks for those families. Their fields, exact key sets, class/source compatibility, source-index correspondence, closure, intersections, containment, metadata and staged native replacement remain existing C++ responsibilities. The portable budget of two million scalar observations counts NURBS CV/knot values; the native decoder continues to count other curve-family numeric values. The portable module must not be described as a complete general loop schema or native topology validator.

There are separate bounds for input bytes (32 MiB), JSON values (three million), JSON container depth (256), curve depth (below 64), curve nodes (16,384) and aggregate NURBS scalar observations (two million). Checked count/byte preflight precedes allocation of extracted basis arrays. The owned DOM has already been parsed by that point; Qt's source DOM and compact serialization also precede the Rust call. No claim that all allocations are preceded by the basis gate is justified. Logical basis bytes do not account for DOM storage, allocator overhead, retained archives or worker aggregates.

`preflightHatchLoopFields` owns the compact UTF-8 `QByteArray` throughout the synchronous Rust call (`ThreeDmHatchLoopEdit.cpp:111–115`). Direct `applyHatchLoopFields` calls preflight first (`:117–120`). The integrating native fixture demonstrates that selected, duplicate, dependency, independent-copy, later-row and multiple-source invalid basis input reaches the gate before the absent archive path or source reconstruction. It also checks exact target sentinel bytes on rejection. Its valid control reaches the absent-source error, establishing that the error-order oracle is active.

## FFI and shared-parser compatibility

`om9_hatch_loop_input` rejects null, empty, over-budget, over-`isize::MAX` and integer-address-overflow inputs before forming a slice (`hatch_loop_input.rs:221–244`). Byte alignment is one. The caller must still supply one live, initialized, immutable readable allocation of the stated length; integer checks cannot establish allocation provenance or accessibility. A deliberately invalid pointer with a small accepted length is outside the unsafe contract and is not a valid test input.

The borrowed input slice and UTF-8 string live only for the synchronous call. The JSON DOM and extracted arrays own their data. No pointer, native allocation, global registry or callback is retained. Invalid UTF-8 is rejected. Errors are scalar status codes, translated by the native adapter into meaningful existing exceptions. The release profile has `panic = "abort"` (`rust/Cargo.toml:11–15`); an allocation failure or unforeseen panic is not translated into a validation status. This is not an all-crashes-prevented claim.

`core_3dm_archive.rs:267–273` changes its Boolean type match to `Json::Boolean(_)`, retaining acceptance of both true and false for `visible` and `locked`. Existing parser consumers otherwise keep their prior depth default. The final full Rust suite reports no failures after the shared enum/helper change.

## Evidence and remaining coverage limits

The initial portable RED is 13 failures and three passes, exit 101. The dedicated depth RED is one failure and 16 passes, exit 101. The depth GREEN is 17 passes, exit 0. The final full Rust log contains 541 passes, including all 18 current-loop gate tests, with `test_exit: 0`. The final native gate result contains 27 passing checks and an empty error; its build/gate/CTest exits are all zero. Four retained CTests passed: Merge, HatchCurrent, HatchLoops and HatchLoopEdit.

Portable unit positive helpers are degree 1. The retained native HatchLoopEdit regression supplies a positive degree-2 rational circle via `ON_Circle::GetNurbForm` (`tests/native/three_dm_hatch_loops.cpp:29`) and applies current homogeneous CV/weight/knot/domain edits (`tests/native/three_dm_hatch_loop_edit.cpp:18,27–31`). This is real native positive coverage of the omitted-outer-knot convention after the new gate. A specific expanded/unclamped ghost-pole positive control remains desirable before describing that convention as acceptance-tested by this increment. The implementation follows the existing observation adapter, but source consistency alone is not a new ghost-pole runtime proof.

The current portable later-invalid test uses a minimal PolyCurve with omitted native fields (`rust/tests/hatch_loop_input.rs:90–95`); it proves recursive portable rejection, not acceptance of that object by the native decoder. The new typed depth control and the native later-row fixture cover different parts of that claim. No exact-at-boundary JSON-node, scalar-budget or maximum-CV runtime fixture was added in this review; checked source paths and existing count-validator tests provide arithmetic evidence, not exhaustive allocation failure coverage.

Full `RCORE-01.T07` remains open: archive deserialization, every native wrapper/proxy/trim/edge/face path, aggregate memory and document/publication atomicity outside these guarded Hatch current fields are not established by this increment. It adds no Rebuild, History, query, editor or solver behavior.

## Exact reviewed hashes

SHA-256 of the frozen files read for this review:

| File | SHA-256 |
| --- | --- |
| `rust/src/hatch_loop_input.rs` | `41bed3ca4109bd8f032820253b201b469f01f291a27933575f39ae60cd07261d` |
| `rust/tests/hatch_loop_input.rs` | `99361aed9d613b3270ef7c3ab02547bcddf86c9e0c97a0aeb46829a592f09f6c` |
| `rust/src/builder_history.rs` | `836ebdfd672d8114fa421dcb2ab3173006b94a9618de7586a170b4b210d0b1a8` |
| `rust/src/core_3dm_archive.rs` | `faaada334f4f7b4d0a7e1d6c52dedf42eef8965f60dec45fd6b8befc9f60f154` |
| `rust/src/nurbs_basis.rs` | `50d4fe44e2a325ff2f614c1f06d263d42cc1f16fc3504e4240863f395e41e916` |
| `rust/Cargo.toml` | `7ad38193eca4b70ee7a421299472099d1bce1b49e89782c3b3d4a96932975f85` |
| `Gui/ThreeDmHatchLoopEdit.cpp` | `d3c038bccabbe2f93974fa553ddb8d5395d5966174473925ea29e1161480976d` |
| `tests/native/rhino_core_hatch_input_gate.cpp` | `54a9d74c14c00fe22a5bdae6e4a23e5131ec6865c65c9a74e073d052ff2788b4` |
| `docs/superpowers/plans/2026-10-10-rhino-core-hatch-input-gate.md` | `f94571ce37488c61d06ac448bbec3ab5f5478977b924f31b86be31ba0f2765df` |

Persisted evidence lives outside the repository at `H:/FreeCAD-src/build`:

| Artifact | SHA-256 |
| --- | --- |
| `p0-hatch-red-rust.log` | `b5f720a66144622d3ff8468d0a715fb2949a3b4713eaf8e4d666dfe6be046b42` |
| `p0-hatch-red-exit.json` | `40f5db64ba3684bacc320ae8f5206b1312a76e2d915be838fe480866f80cff66` |
| `p0-hatch-depth-red-rust.log` | `10a5fcb95ed9f6e23bd6eba1a6ffcdd83cc532990d4b68f56bba681aca3d95e7` |
| `p0-hatch-depth-red-exit.json` | `40f5db64ba3684bacc320ae8f5206b1312a76e2d915be838fe480866f80cff66` |
| `p0-hatch-depth-green-rust.log` | `4d4b5c2bbedb8b55728b78975fde9453e7b89cc2b488b11a660e4584db6a064e` |
| `p0-hatch-depth-green-exit.json` | `8e1c27e07362eb9798b515ea02b0dc3ef4682ff85da284264f0d210e38fd9710` |
| `p0-hatch-final-rust.log` | `5b212495df0682b0e3f3b5e62329bffcba8196ace91f7d4f367b9c4d27d52fbc` |
| `p0-hatch-final-exit.json` | `8e1c27e07362eb9798b515ea02b0dc3ef4682ff85da284264f0d210e38fd9710` |
| `p0-hatch-native-final-results.json` | `58ab92e62065354bce415f722cb2fadbc6217ea5d634840fcae866778165c100` |
| `p0-hatch-native-final-exits.json` | `11db90713accd2e22917cf047e0e5408368bbf320fdecf77e30e461480cba241` |
| `p0-hatch-native-final-ctest.log` | `b27e5c880039ed3cf5214db7c52a278ffc9b662cfb4706bfddd503c55597f705` |
