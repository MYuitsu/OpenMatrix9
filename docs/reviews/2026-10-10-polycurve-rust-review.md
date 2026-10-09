# Polycurve ordered segment contract: portable Rust review

Date: 2026-10-10. Baseline: `04d64cc`.

## Scope and conclusion

Independent review of Task 2 in `docs/superpowers/plans/2026-10-10-rhino-core-polycurve-partition.md`, against the ordered segment representation contract and RCORE-01.T05 in `OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/01-object-model-kernel.md`. Reviewed the formatted safe Rust contract, copied-value FFI gates, both focused test targets, and the two module registrations in `lib.rs`.

No blocking finding in this portable scope. Safe Rust owns the numerical policy, limits, ordered copied observations, and independent report data. Native ownership, truthful occurrence traversal, curve conversion, native ABI agreement, save/reload, and working exchange remain outside this review. Portable success does not close T05, all-family polycurve preservation, History, or full P0.

## Contract observations

- `preflight` rejects zero or more than 4096 segments before row access. Checked multiplication and addition calculate the logical row/report budget, capped at 1 MiB. This budget includes the owned Rust row data and report value; allocator overhead, native containers, native copies, and worker allocations are not bounded by it.
- Length and parameter tolerances must be explicit, finite, and positive. Known domains require finite increasing endpoints and a finite span. Chained `hypot` avoids avoidable square-sum overflow; nonfinite distances and overflowed differences reject. Parameter comparisons also reject nonfinite differences.
- Parent intervals are optional and separate from child supporting domains. When the input parent domain is known, every row must have a valid interval contained in it, consecutive interval boundaries must agree within parameter tolerance, and the first/last boundaries must cover the parent endpoints within that tolerance. When the parent is unknown, row parent intervals must also be unknown. The contract does not infer a parent-to-child parameter map.
- Source rows reject missing, nested polycurve, and reference families. Ordinals start at zero and advance by one; endpoints must be finite and consecutive source joints must agree within length tolerance. Validation does not change source rows or heal endpoints.
- Native rows require a known direction and an ordinal matching the corresponding source row. Their already oriented start/end points must agree with source endpoints. Joint distance and reported shared topology are separate gates: later rows require `shared_with_previous == Some(true)`, while the first row requires no previous-joint observation. Reversed topology is accepted with correctly oriented points.
- Closure requires matching source/native closed flags, reported shared closing topology, and both source and native closing distances within tolerance. Open sequences do not make a closing-joint claim.
- `report` checks complete slice lengths and occurrence count before indexing, admits every source row, validates every source/native step, then checks finish. Checked reservations precede the three owned vectors. Allocation or validation failure returns no partial successful report; successful reports copy rows and survive later caller mutations.
- The conversion report remains `uninspected`: locus, parameterization, topology, parameter map, and deviation are `Unknown`; no inspected attribute preservation is invented. Endpoint and joint measurements are finite observations, not a continuous curve deviation certificate.

## Caller obligations and remaining limits

These are explicit integration conditions rather than blocking defects in the portable contract:

1. `validate_finish` (`polycurve_contract.rs:274`) and `om9_polycurve_finish` (`polycurve_contract_ffi.rs:91`) are count/closure fold boundaries. Callers must successfully check every ordered source/native row. Finish alone cannot certify omitted intermediate rows, their endpoints, or their source admission. The safe whole-report path performs those checks itself.
2. The native adapter must enumerate actual edge occurrences, including repeated uses and reversed wire traversal. A copied ordinal and occurrence count are claims supplied by the caller; Rust cannot establish that they correspond to every live native occurrence. Reordering or duplicating geometrically coincident occurrences can be invisible to endpoint-only data.
3. Direction and shared-vertex fields must be read from the actual native traversal. Rust validates supported direction tags and oriented endpoints, but cannot verify the truth of a copied orientation or topology flag. Shared vertices do not establish original source identity, tangent continuity, or durable `EdgeN` correspondence.
4. Admitted leaf-family tags, including `NativeOther`, are not evidence that a native converter preserves each curve's full supporting geometry, basis, weights, attributes, or ownership. The native adapter must retain each source child, apply its own owner/envelope gates before native metadata/evaluation, and reject unsupported conversion without fitting, flattening, or healing.
5. Branching/native validity, continuous geometric bounds, FCStd and working V5 partition roundtrips, source/sibling immutability, cancellation, History, and reference/nested ownership still need their respective native/host evidence. The contract supplies ordered segment observations rather than a general wire or topology certificate.

## Value ABI and failure behavior

The FFI records use `repr(C)`, scalar values, and fixed arrays. No pointer, callback, borrowed buffer, retained native owner, or allocation transfer crosses these interfaces. `usize` fields require a matching native `size_t` ABI.

The focused Rust layout tests confirm the current 64-bit sizes/alignment and selected offsets:

| Record | Size | Alignment | Checked offset |
| --- | ---: | ---: | --- |
| Input | 56 | 8 | `length_tolerance`: 24 |
| Source | 96 | 8 | `start`: 48 |
| Native | 64 | 8 | `start`: 16 |
| Summary | 40 | 8 | `metrics`: 8 |

Unknown or malformed option/flag/tag values reject with protocol status 199. Supported-but-rejected observations use typed error statuses 100–117. Absent previous or parent payloads are not consumed, including their malformed or nonfinite sentinels. False shared-topology observations reject at a required joint; unknown topology does not count as success. Every failed summary has schema zero and zero metrics; success carries the current schema and copied finite measurements.

Scalar FFI gates do not allocate and contain no input-dependent indexing or intentional panic path. They have no recovery shield for an unexpected panic; the crate's release policy is `panic = "abort"`. Native exceptions and native-side lifetime/ABI safety still require their own checks. This portable review does not certify native dependencies as memory-safe.

## Verification evidence

Independent focused run, using a separate target directory and the frozen source:

```text
rtk proxy cargo test --manifest-path H:/FreeCAD-src/Mod/OpenMatrix9/rust/Cargo.toml --target-dir H:/FreeCAD-src/build/polycurve-independent-review-target --offline --test polycurve_contract --test polycurve_contract_ffi
```

Result: exit 0; 13 core tests and eight FFI tests passed. The primary Rust target directory was not used. No native, SDK, GUI, or release build was run by the reviewer.

The final cleanup was subsequently re-reviewed without another build. It changes three nested `if let`/`near` guards into equivalent Rust 2024 let chains and adds a local `clippy::too_many_arguments` allowance on `validate_finish`, with a reason explaining the explicit fold-boundary fields. Reversing only those changes in memory reproduces the previously reviewed source SHA-256 `a82668b8be3ec2fe35b8736141a1ab4fd467cb5e1c2bc6a60ed59beefa570312`. The other four reviewed file hashes are unchanged. No API, ABI, numerical policy, or failure behavior changed; the no-blocker conclusion still applies.

Parent-run logs and exit records were independently read:

- `H:/FreeCAD-src/build/p0-polycurve-rust-rerun-red.log`: 13 failed tests; matching exit record 101.
- `H:/FreeCAD-src/build/p0-polycurve-rust-formatted-green.log`: 13 passed tests; matching exit record 0.
- `H:/FreeCAD-src/build/p0-polycurve-ffi-red.log`: two passed and six failed tests; matching exit record 101.
- `H:/FreeCAD-src/build/p0-polycurve-ffi-green.log`: eight passed tests; matching exit record 0.

These retained RED logs establish the initial missing implementation gates. They are not claimed as native exchange regressions or native acceptance evidence. Focused tests cover domain separation, malformed inputs, ordinal/count coverage, no source mutation on rejection, endpoint/joint/topology distinctions, reversal, closure, owned reports, unknown global facts, absent payload sentinels, layouts, and zeroed ABI failures.

## Reviewed source hashes

SHA-256 of the reviewed formatted sources:

| File | SHA-256 |
| --- | --- |
| `rust/src/polycurve_contract.rs` | `8c802cdcf983c13cea4820ec93f0c22fe968a7348998d7e1cbdd0f6a9fb67571` |
| `rust/src/polycurve_contract_ffi.rs` | `7b67905b52acb742ae565dfcd2dc9d7241c10f0f2bf95704ea9f54d2ac5c52ea` |
| `rust/tests/polycurve_contract.rs` | `95f5bc5f003225de16342ae0c996bb76ff9a1586f02b58e005b656b5c9117300` |
| `rust/tests/polycurve_contract_ffi.rs` | `7cacb4b51547558bff675f068277b5a3cb7441ad037e9951e81eb1dfbfd19307` |
| `rust/src/lib.rs` | `880d51c45a86264692e2558cbe31f6431d260a9b55126572cfb685731d41aabe` |

The reviewer changed only this review document; production and test sources were not edited.
