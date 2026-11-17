# B-rep source lease capture timing

Date: 2026-11-05. This is a core-only synthetic microbenchmark, not an
application, GPU, STL throughput, or desktop latency measurement.

- Command: `cargo test --locked --release -p om9-core -j 2 --test brep_source_capture_bench -- --ignored --nocapture`
- `CARGO_TARGET_DIR=L:\om9-target`, `CARGO_INCREMENTAL=0`; Windows x64,
  `PROCESSOR_IDENTIFIER=AMD64 Family 25 Model 33 Stepping 0, AuthenticAMD`.
- Probe: `crates/om9-core/tests/brep_source_capture_bench.rs`, SHA-256
  `d14c1f38b90c44f325b842a4f6eb21cdd2d1bc83a765c41d3fd8ce02b8cb6aac`.
- Five alternating runs per case; values are microseconds, median and maximum.
  The output vector was preallocated outside the timed interval; no physical
  RAM sampling or STL metadata, meshing, display or disk work was included.
  Shapes are tiny synthetic core snapshots; “distinct” varies source text,
  not model size. Native OCCT validation was not part of this measurement.

| Sources | Objects | Individual median / max (µs) | Batched median / max (µs) |
| --- | ---: | ---: | ---: |
| Shared | 24 | 158 / 298 | 7 / 11 |
| Shared | 64 | 1,022 / 1,216 | 19 / 20 |
| Shared | 128 | 3,873 / 4,696 | 38 / 44 |
| Distinct | 24 | 253 / 364 | 10 / 11 |
| Distinct | 64 | 2,859 / 3,292 | 43 / 46 |
| Distinct | 128 | 18,418 / 19,721 | 129 / 130 |

All runs completed. These figures motivate batching lease admission, but do
not establish p95/p99, real-model scaling or desktop responsiveness. Later
account inventories still inspect live source owners; their cost at larger
scale and end-to-end UI impact remain to be measured.
