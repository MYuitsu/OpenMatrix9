# DupEdge source adapter: process memory sample (2026-11-11)

One Windows x64 debug test process, one generated OCCT Box (4 × 5 × 6 mm, display deflection 0.01 mm), and 128 sequential calls to `source_edge_curve_at_segment_admitted` were used. Each result was checked against the analytic midpoint. Command: `cargo test -p om9-app --test wireframe dupedge_source_adapter_process_memory_probe -j 2 --target-dir L:/om9-target -- --ignored --nocapture`. One test passed; this is a diagnostic run, not a release benchmark or a product viewport/transaction test.

| Process counter | Before | After |
|---|---:|---:|
| Private bytes | 57,237,504 | 57,237,504 |
| Peak pagefile bytes | 57,257,984 | 57,257,984 |
| Working-set bytes | 56,274,944 | 57,049,088 |
| Peak working-set bytes | 56,274,944 | 57,049,088 |

Elapsed time for the 128 calls was 81.428 ms on this run. The counters describe the whole test process, including Rust, OCCT, allocations retained by their runtimes and earlier setup. Windows peak counters cover the process lifetime, and the pre-existing peak can hide a smaller transient peak; no sampling cadence or allocator attribution was used. Therefore the unchanged peak pagefile does **not** establish zero temporary allocation, an upper bound per call, OCCT-only peak, physical RAM available to other workloads, or production latency. Native OCCT allocations remain outside the Rust admission callback. Reopened product UI, larger or complex models, cold start, desktop and real workloads remain unmeasured.

At execution, SHA-256: `crates/om9-app/tests/wireframe.rs` `9e6e9db5e4071e8f66908e21c5e1084434195487dd7eb6bbee5ed1a367602703`; `crates/om9-kernel/src/editing.rs` `61dce4b48f67a6c04cab51d677da87f060231cb6abf79286e62d4154196e008b`; `crates/om9-kernel/src/editing_bridge.cpp` `13c7e066cd583540c008f0f7f921807d4ca04b91fdb55641ad0e08a76729c282`; test executable `8ea091669251331441b4d8c19a3744258d4dbc542aa80d16993ebfbb1c89274c`. The executable and build directory are not the sole evidence for the run.

After adding three interior segment projections, the same command and 128-call debug Box fixture passed again: elapsed 91.236 ms; private bytes before/after 44,109,824; peak pagefile before/after 44,130,304; working set 44,425,216 before and 45,187,072 after, with those respective process-lifetime peaks. The runs were not interleaved or controlled, so the elapsed values cannot establish a performance difference. The same process-counter limitations above apply. Updated SHA-256: `editing_bridge.cpp` `b8ea5554aa7f8be087fa3cd4d5a4cc2f0068aaa6d0545922e9e094a288ab15a2`; `session/wireframe.rs` `84dd283966401fcc4fcdedc23bed2ed45167ab8a4246178d814d05b29df98b65`; test executable `7700aedcc77221a06b92b8ffcefbbd05c76c9395820c9db56b8567076eba7c26`.
