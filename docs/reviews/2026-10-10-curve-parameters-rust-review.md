# Curve parameter conversion: portable Rust review

Date: 2026-10-10. Baseline: `e5633b7`.

## Scope and conclusion

Independent review of Task 1 in `docs/superpowers/plans/2026-10-10-rhino-core-curve-parameters.md`: the safe Rust conversion query contract, its value ABI, the corresponding focused tests, and module registration in `lib.rs`. The final sources were read after formatting. This review did not inspect or execute native integration, SDK, GUI, or release builds.

No remaining blocking finding in this portable scope. The target-domain raw-reuse defect found during review is resolved. The contract keeps native form 1 and form 2 provenance distinct, validates each copied observation, and leaves global parameterization, locus, topology, and deviation facts uninspected. Passing portable tests does not close full T10, History, or global mappings.

## Resolved finding: raw reuse outside the target domain

The original form-1 raw-reuse condition allowed a source parameter just beyond the target-domain boundary when parameter and point differences remained within tolerance. That would authorize raw extrapolation even though the mapped parameter was valid.

`rust/src/curve_conversion.rs:202` now requires the raw source parameter to belong to the target domain, in addition to form 1 provenance, the raw point residual, and parameter agreement. Form 2 never authorizes raw reuse, including equal-domain or equal-endpoint observations. The regression `near_target_boundary_does_not_allow_raw_extrapolation` checks this boundary case.

Retained evidence was inspected: `p0-curve-parameters-gates-red.log` records that assertion failing with 13 passing tests and one failure; its exit record is 101. The subsequent green log records 14 core and eight FFI tests passing, including the boundary regression and mapped-output gate.

## Contract and failure handling

- Native forms 1 and 2 produce separate direct-candidate and requires-native-map actions. A form-1 action is provenance for the current query, not a global preservation statement.
- Inputs require finite, strictly increasing domains with finite spans, explicit finite tolerances, and the supported local frame and conversion method. Length and parameter tolerances must be positive. A zero conversion tolerance is retained as API provenance and does not establish exactness.
- Source requests must lie inside the source domain. Successful mapped and inverse values must lie inside their respective domains. The contract neither wraps closed parameters nor extrapolates them.
- `validate_mapped_parameter` and `om9_curve_conversion_mapped_parameter` expose a gate for native callers to check a mapping result before passing it to another native mapping or point evaluation. False success flags are rejected before output sentinels are consumed; nonfinite and outside-domain successful outputs also reject. This review confirms the portable gate and tests, not native call ordering.
- Observation validation checks mapping status, domains, inverse agreement, all point coordinates, and finite intermediate/native residuals. Chained `hypot` avoids avoidable square-sum overflow. A genuinely nonfinite distance rejects. Raw residuals are diagnostic and can exceed the acceptance tolerance.
- The query budget of 4096 is checked before allocation or row processing. The aggregate report uses checked reservation, owns copied rows, and returns no partial successful report when a row fails. Its `ConversionReport::uninspected` does not turn sampled agreement into a global map or deviation bound.

The mandatory raw diagnostic point limits the current adapter slice. Rejecting a source raw parameter outside the target domain is consistent with the documented finite full-domain query-only scope. It does not establish support for general conversions with differently bounded domains, where a mapped query might be valid but a raw diagnostic is unavailable. Such support would need an explicit optional diagnostic or a broader contract.

## ABI, ownership, and panic boundary

The exported records are `repr(C)` values containing scalar fields and fixed arrays. No native pointer, retained native handle, borrowed native array, or allocation ownership crosses these query interfaces. Successful summaries copy the validated observation; failures return an error status and a default empty summary instead of publishing failed output values. Unknown flag values, reserved values, and unsupported enum values reject.

The Rust layout tests check the current 64-bit record sizes, alignment, and selected offsets: 72-byte input, 128-byte observation, and 168-byte summary, each aligned to eight bytes. These tests are portable Rust evidence; they do not substitute for a compiled native ABI check.

The scalar FFI paths do not allocate and contain no input-dependent indexing or intentional panic path. They have no `catch_unwind` recovery shield. The crate's release policy is `panic = "abort"`; an unexpected panic is therefore not a recoverable query error. This review makes no claim that native exceptions are handled by Rust or that native exception/unwind boundaries have been exercised. The safe aggregate report handles reservation failure separately through `AllocationFailed`.

## Verification evidence

The reviewer independently ran the two focused integration targets with an isolated target directory:

```text
rtk proxy cargo test --manifest-path H:/FreeCAD-src/Mod/OpenMatrix9/rust/Cargo.toml --target-dir H:/FreeCAD-src/build/curve-parameter-independent-review-target --offline --test curve_conversion --test curve_conversion_ffi
```

Result: exit 0; 14 core tests and eight FFI tests passed. The primary Rust target directory was not used by that run. The final formatted sources were then re-read; no additional builds were run for this final review.

Parent-run evidence inspected in `H:/FreeCAD-src/build/p0-curve-parameters-checks-exits.json` records full Rust tests and Clippy at exit 0. Its initial Python runner records exit 1 and is not reported as passing. The separate corrected unittest run in `p0-curve-parameters-tooling-unittest-exit.json` records exit 0; its log reports 50 tests with one skipped, therefore 49 passed and one skipped. The reviewer did not independently rerun these broader checks.

## Reviewed source hashes

SHA-256 hashes of the final reviewed files:

| File | SHA-256 |
| --- | --- |
| `rust/src/curve_conversion.rs` | `dc302e1f759abbfbf1c05f8cb9c831a7cf337b382fcfe857cefbfa48a111963a` |
| `rust/src/curve_conversion_ffi.rs` | `9d964b66dfd767ad2532e02713b0213a284723fa3d8dedee1a54487d284fa3a7` |
| `rust/tests/curve_conversion.rs` | `48646ec56f8754352930debc0401492e7d92d50a7829221bc12630486366a590` |
| `rust/tests/curve_conversion_ffi.rs` | `e4a6d70923215455ad2654781964f7d0c8c19e82ffa927f8a8344cebfed7a665` |
| `rust/src/lib.rs` | `1cc2e87fe94a2a89d55adffa918dbc8dc4a0d42e7e178a085021d7c607294522` |

Only this review document was changed by the reviewer for this finalization. Native acceptance, full T10, History, and global mapping/preservation guarantees remain open.
