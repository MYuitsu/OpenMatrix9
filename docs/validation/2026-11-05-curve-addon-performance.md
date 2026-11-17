# Curve add-on vs built-in — 2026-11-05

Source `dc2d4dfd665e14d1a68aa9c30f1fd59daca6db02` plus the local benchmark
in `crates/om9-app/tests/addon_session.rs`. Windows 10.0.26300.9818,
AMD Ryzen 9 5900X, rustc 1.98.1; default workspace features; release profile
(`opt-level=3`, thin LTO), `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`.

Command (run twice):

```powershell
$env:CARGO_TARGET_DIR = 'L:\om9-target'
rtk cargo test --release -p om9-app --test addon_session benchmark_builtin_vs_addon_curve_commands -j 2 -- --ignored --nocapture
```

Both runs passed. Each run used five warm-ups then 40 timed samples per case,
alternating implementation order, with a new empty document per sample. Timing
starts before command invocation and ends after the last input and transaction.
Setup, host loading and result assertions are outside the interval. Each sample
asserts one object and one Undo entry. The line has endpoints (0,0,0) and
(10,0,0); the circle is centered at (0,0,0), radius 5 mm. Output geometry and
dimensions are checked for every sample.

| Case | Implementation | Run 1 median / p95 (ms) | Run 2 median / p95 (ms) |
|---|---|---:|---:|
| Line | Built-in | 0.3219 / 0.4491 | 0.3078 / 0.4774 |
| Line | Add-on | 2.6014 / 3.6415 | 2.3708 / 3.1637 |
| Circle | Built-in | 0.2992 / 0.3884 | 0.2585 / 0.3617 |
| Circle | Add-on | 2.6335 / 3.4524 | 2.3991 / 3.4892 |

The second-run median is about 7.7× for add-on Line and 9.3× for add-on
Circle. Each pair uses the same input values; both measure controller, command
inputs, transaction and scene publication. The add-on uses the real Rust Matrix
command implementation and JSON session boundary in an in-process test runtime,
not a loaded production DLL. No claim is made for DLL loading, UI pointer/GPU
latency, large documents, other Curve commands, memory use or a root cause of
the measured difference. CPU frequency and other machine activity were not
controlled. The debug pilot is excluded from the results.
