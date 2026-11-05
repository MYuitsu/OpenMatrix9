# Boolean, Fillet and Loft process-memory probes

The opt-in Windows release fixture `crates/om9-app/tests/kernel_memory_probe.rs` runs one operation in a fresh process, samples `PROCESS_MEMORY_COUNTERS_EX` immediately before and after the call, then verifies a closed result, zero naked edges and an independently expected volume. The OCCT 8.0.1 adapter uses display deflection 0.01 mm. Setup occurs before the first sample; both input and output remain alive during the operation. The sampled peak is the process-lifetime `PeakPagefileUsage` commit high-water, not an isolated OCCT allocation or GPU memory.

The executable was built on E: with default product features and `-j 2` (SHA-256 `8A462D8ADA8570446A06A3B0D772A77CD8F5D794F9761AE7A1FCCA38007D182B`). An initial set passed; the table records a second set of fresh processes, one run per case, under uncontrolled system load.

| Fixture | Expected volume (mm³) | Peak before (MiB) | Peak after (MiB) | Difference (MiB) | Call time (ms) |
| --- | ---: | ---: | ---: | ---: | ---: |
| Union of two 10 mm boxes, 5 mm overlap | 1500 | 25.23 | 33.23 | 8.00 | 57.569 |
| One 1 mm edge fillet on a 10 mm box | 997.8539816 | 24.61 | 25.03 | 0.43 | 4.758 |
| Solid loft through three 10 × 10 mm squares | 1000 | 13.04 | 25.13 | 12.09 | 5.294 |

Raw peak-commit bytes before/after: Boolean 26,456,064 / 34,844,672; Fillet 25,800,704 / 26,247,168; Loft 13,668,352 / 26,345,472. Peak working-set bytes after were 25,268,224; 15,958,016; and 15,765,504 respectively. The analytic volume checks passed with absolute tolerance `1e-5 mm³`.

These are small synthetic cases, not upper bounds or safe reservation factors. Process high-water differences can hide allocations when setup established a higher peak; timing and working set depend on load, allocator and OS state. They do not distinguish Rust, OCCT, input, output or temporary allocations. Complex solids, many edges, dense loft sections, concurrent operations, larger fixtures and native OOM behavior remain unmeasured. Do not use these numbers alone to raise the 256 MiB ceiling or admit background export.

## More detailed Boolean and loft fixtures

The same opt-in probe was extended with a 20 × 20 × 10 mm solid drilled with 16 radius-0.7 mm holes during setup; the measured operation cuts a seventeenth hole at the center. Another fixture lofts 32 closed square sections over 10 mm. Each ran once in a fresh Windows process with OCCT 8.0.1 and deflection 0.01 mm. The current release test executable, built externally with Cargo `-j 2`, had SHA-256 `7C1A7E4D8AFF9B4449BAAC463ADD31E6313B877DB786DCE56EC9F70527C91D17`; the build took 7m41s. The user's OM9 session was not operated.

| Fixture | Output volume (mm³) | Output faces | Peak commit before (MiB) | Peak commit after (MiB) | High-water difference (MiB) | Call time (ms) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Difference on 16-hole body | 3738.305331956 | 23 | 39.5664 | 39.6328 | 0.0664 | 75.423 |
| Loft, 32 equal square sections | 1000 | 6 | 13.0195 | 25.4375 | 12.4180 | 6.119 |

Both results were closed, had zero naked edges and matched their analytic volumes within `1e-5 mm³`. Raw before/after peak-commit bytes were 41,488,384 / 41,558,016 for the Boolean and 13,651,968 / 26,673,152 for the loft. The drilled-body setup itself had already established a process high-water mark; its small later difference does **not** mean the Boolean needed only 0.0664 MiB. The loft produces just six faces for these aligned equal sections, so 32 input sections do not demonstrate complex output curvature. One run per case under uncontrolled load does not provide p95/p99, a native allocation bound, a process-memory guarantee or a safe worker admission formula. Multi-edge fillet, curved lofts, heavier real models, concurrent operations, release desktop and allocation attribution remain open.

## Twelve-edge fillet and curved 32-section loft

The same isolated probe now includes a 10 mm cube filleted on all 12 edges at radius 1 mm, and a solid loft through 32 equal 10 mm square sections whose centres follow a 2 mm sinusoidal x displacement over 10 mm height. The fillet volume is independently calculated as the rounded-box volume; the loft preserves a constant 100 mm² horizontal cross-section, so its expected volume is 1000 mm³. Both results were closed with no naked edges and matched the stated volume within `1e-5 mm³`; the loft's sampled display bounds also reached beyond x=-1.9 and x=11.9 mm. This check shows the result follows the curved path, not a full surface-accuracy certification.

Each case ran three times in separate default-feature Windows release test processes with OCCT 8.0.1, deflection 0.01 mm, external target `E:\om9-surface-snap-validation` and Cargo `-j 2`. The build took 5m32s. Executable SHA-256 `64267B34692CC7BA0D447496A9C9FF3A88077ECF379757D0801E0FC0ADCB8EBF`. System load was uncontrolled; raw values are bytes and times are milliseconds:

| Fixture | Result faces | Peak commit before → after (three runs) | Call times |
| --- | ---: | --- | --- |
| Fillet all 12 edges | 26 | 25,845,760 → 26,796,032; 32,137,216 → 33,099,776; 25,812,992 → 26,767,360 | 17.258; 16.654; 16.574 |
| Curved loft, 32 sections | 6 | 13,664,256 → 27,115,520; 13,631,488 → 33,370,112; 13,668,352 → 33,435,648 | 18.050; 17.819; 18.084 |

Median before-to-after high-water differences are approximately 0.91 MiB for fillet and 18.82 MiB for loft. These are process-lifetime high-water differences, not the memory allocated by each operation: setup may have already set a higher peak, and the second and third loft runs had a higher after-peak than the first. They include Rust, OCCT and allocator effects and are not an upper bound, safe reservation factor, p95/p99, real-model performance, or desktop result. Heavier and more complex sources, attribution, concurrent jobs and native OOM remain open. Two preliminary debug cases also passed the same geometric checks.
