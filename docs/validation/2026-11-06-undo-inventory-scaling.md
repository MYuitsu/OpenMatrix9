# Undo inventory scaling — 2026-11-06

Undo inventory previously scanned earlier B-reps in the same document for each
source, display and topology identity. It now uses per-identity maps that
record the last document counted, so an alias in one document is subtracted
at most once while the same allocation across documents is still shared. The
larger map entries are included in the already admitted scratch bound. No RAM
or commit admission guard was disabled for either measurement.

Manual command, Windows x64, `CARGO_TARGET_DIR=L:\om9-target`,
`CARGO_INCREMENTAL=0`, `-j 2`:
`cargo test --release --locked -p om9-core --test undo_inventory_bench -j 2 -- --ignored --nocapture`.
The fixture has eight placement-history entries and tiny synthetic B-reps; it
uses either one shared source or distinct source allocations. Each number is
the median of five samples, in microseconds. Runs before and after the dedup
change were consecutive but not interleaved, so differences are indicative
rather than a controlled latency guarantee.

| B-reps | Shared before / after | Distinct before / after | Distinct retained-owner check after |
| ---: | ---: | ---: | ---: |
| 24 | 87 / 86 | 136 / 128 | 101 |
| 64 | 219 / 221 | 555 / 506 | 406 |
| 128 | 436 / 442 | 1746 / 1713 | 1353 |
| 256 | 876 / 905 | 7044 / 5962 | 4839 |
| 512 | 1957 / 1757 | 27140 / 25154 | 20682 |

Inventory byte outputs matched for each before/after case. The separate
`retained_usage()` timing includes owner validation under its own limits; it is
not an exact subtraction or an isolated cost of the admission added earlier.
The distinct-source growth remains approximately quadratic. In particular,
`Document::storage_usage` still uses allocation-free prior-owner scans; this
work does not claim to fix that broader scaling or measure desktop latency,
native geometry, RAM peak, or model sizes beyond these fixtures.

`cargo test -p om9-core --lib -j 2`: 353 passed;
`cargo test -p om9-core --test atomic_transactions -j 2`: 29 passed;
`cargo test -p om9-app --lib incremental_scene_memory_tests -j 2`: 12 passed.
The optimized `undo_memory.rs` SHA-256 is
`e024ad36d2df895c307e52c2f378178d7e19a5a31e89d6227f82b05eeaf39638`;
the benchmark SHA-256 is
`b893b234e7db5399516ed7f1875ed222cdc3b168f5aba3006b4eb75460dbb05b`.
Sphere production source/display separation and cache eviction remain open.

## Follow-up: admitted per-document storage index

The hot retained-owner and Undo-inventory paths now use a per-document B-rep
allocation index after its conservative scratch bound has been admitted. The
cross-document index also records the last document for each allocation, so
aliases in one document and shared allocations across documents are each
deduplicated correctly. The initial admission uses a nonallocating,
duplicate-inclusive upper bound. When that bound and index cannot fit, retained
owner validation falls back to the existing nonallocating exact scan; that
fallback can remain quadratic. The ordinary non-indexed document API and other
callers also retain their previous allocation-free scan.

With the same release command and five samples per case, the 512-distinct-BRep,
eight-step fixture had median inventory 25,154 µs before this follow-up and
3,225 µs afterward; the separate retained-owner check was 20,682 µs and
2,243 µs respectively. Inventory bytes remained 5,929,120 in both runs.
The 512-shared-source case was 1,757 µs before and 1,622 µs afterward.
These runs were not interleaved, use tiny synthetic B-reps and no physical or
commit sampler, and are not a desktop or real-model performance guarantee.

Validation: core unit 354 passed, including indexed-versus-allocation-free
accounting for mixed shared/independent OCCT and native B-reps; atomic
transactions 29 passed; app incremental scene/memory 12 passed; archive v4
acceptance 8 passed. Existing tight-budget tests exercise the fallback path.
SHA-256: `model.rs`
`c15199edad8f591843441ae412abda40521e7a8ee73e20a5b5b6b286ad42a81f`;
`transaction.rs`
`cdb7fa4e2a8c48f89706496cca9f4d324ebc0b1336b46671b5eb04c7938035d3`;
`transaction/undo_memory.rs`
`261f8edccee7c6b0b4bb1350bd4bcce2d52d1d30bdb61b2e4d41ee5e18ab7da7`.
The benchmark file hash is unchanged from above. Source/display separation,
per-buffer eviction, native peak and desktop interaction remain open.
