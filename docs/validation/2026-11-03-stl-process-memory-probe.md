# STL process memory probe

## Scope and method

Added a Windows-only, on-demand diagnostic in `om9-native-owner` for the current process's `PROCESS_MEMORY_COUNTERS_EX` working set, peak working set, private usage and `PeakPagefileUsage`. The latter is the Windows counter name, not a measurement of bytes physically written to the paging file. A separate opt-in test executable constructs the source and controller, samples before work, then samples the process high-water counters after work. Each fixture was run in a fresh process; the high-water marks include initialization and fixture preparation, not only the measured operation. `dense_native` invokes the same grouped OCCT tessellation/snapshot adapter with local deflection half the requested output tolerance and leaves its snapshot alive for the final sample. The other cases perform actual STL export and remove the test output.

Windows, default product features, release profile from E:/om9-surface-snap-validation, Cargo `-j 2`, OCCT 8.0.1, one run per case; system load was uncontrolled. Memory values below are MiB (1,048,576 bytes), rounded to two decimals. Delta is `after PeakPagefileUsage - before PeakPagefileUsage`, both process-lifetime high-water counters. It is not a direct allocation counter for the operation and includes native and Rust memory.

| Case | Requested tolerance (mm) | Output triangles | Before peak pagefile counter (MiB) | After peak pagefile counter (MiB) | Peak increase (MiB) | Peak working set (MiB) | Wall time (ms) |
|---|---:|---:|---:|---:|---:|---:|---:|
| Full STL, one sphere radius 5 mm | 0.05 | 2,002 | 54.40 | 54.90 | 0.50 | 54.00 | 11.420 |
| Full STL, one sphere radius 12 mm | 0.001 | 241,128 | 54.31 | 1,050.44 | 996.13 | 912.79 | 5,715.282 |
| Full STL, eight separated identical B-rep copies, radius 5 mm | 0.005 | 159,600 | 54.70 | 121.76 | 67.06 | 118.17 | 1,041.984 |
| Tessellation/snapshot only, one sphere radius 12 mm | 0.001 | 241,130 | 54.22 | 1,050.37 | 996.14 | 912.97 | 5,500.300 |
| Tessellation/snapshot only, one sphere radius 12 mm | 0.002 | 120,446 | 54.56 | 433.54 | 378.98 | 378.45 | 1,809.927 |
| Tessellation/snapshot only, one sphere radius 12 mm | 0.005 | 47,984 | 54.45 | 167.82 | 113.37 | 155.41 | 477.102 |
| Tessellation/snapshot only, one sphere radius 12 mm | 0.01 | 24,040 | 54.17 | 97.95 | 43.78 | 91.72 | 149.729 |

The full and snapshot-only fine-sphere runs reached nearly equal process high-water values in separate processes. This locates the high-water event no later than completion of the grouped native snapshot path for this fixture; it does not distinguish OCCT allocations from Rust buffers or identify a single allocating function. The full STL output has two fewer triangles than the intermediate snapshot; the normal generated-triangle finalization is part of the full route. The eight objects are cloned B-reps, not shared definition occurrences: the current STL path does not yet export instance sources. The configured 256 MiB aggregate usage limit is therefore **not an RSS, commit, native transient or process memory cap**.

Raw byte counters (before peak / after peak pagefile; peak working set): light 57,044,992 / 57,569,280 / 56,623,104; dense full 56,950,784 / 1,101,467,648 / 957,128,704; repeated 57,356,288 / 127,672,320 / 123,912,192; dense snapshot 0.001 56,856,576 / 1,101,389,824 / 957,317,120; 0.002 57,212,928 / 454,598,656 / 396,836,864; 0.005 57,098,240 / 175,976,448 / 162,963,456; 0.01 56,803,328 / 102,711,296 / 96,174,080.

## Validation and limitations

Three `om9-native-owner` memory tests passed; the light diagnostic passed once in debug; all seven isolated release cases above completed. The no-default-features app check passed with 17 existing warnings. Test outputs were checked for nonzero triangle counts and STL file size. The active OM9 desktop session was not operated.

These are single-run, synthetic fixtures, not measured maxima or safe reservation formulas. Allocator reuse, source complexity, trimmed surfaces, other OCCT operations, concurrent workers, OS pressure, execution architecture and CAD model scale can change peaks. Working set is resident memory and can vary with paging; process `PeakPagefileUsage` is not a GPU metric. This does not establish input-to-display, native-vs-Rust allocation attribution, a 60% managed budget, or OOM recovery. Release measurements of boolean, fillet and loft remain required. Do not connect the STL worker on the assumption that a fixed multiple or this sphere's high-water mark is an upper bound.
