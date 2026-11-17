# Batched scene display-pin admission

2026-11-13. The ordinary OCCT scene path previously admitted each distinct display pin separately, repeatedly scanning retained owners. It now reserves temporary request/result/sort storage, validates every occurrence and its source/path before registration, deduplicates by actual display allocation, holds existing leases while accounting, and admits new lease metadata in one locked batch. Returned pins are installed only after the whole batch succeeds. The returned batch exclusively borrows its scratch reservation until the output vector drops. A failed stamp/path or simulated RAM admission leaves the lease registry without partial new pins. Accounted scenes remain document-backed; this is not C1 completion.

With `CARGO_TARGET_DIR=L:\om9-target`, OCCT runtime on PATH, `-j 2`: core lib 384 passed/1 ignored and 3 compile-fail doctests passed, including scratch lifetime; scene lib 47 passed; app `topology_remesh` 53 passed/1 ignored, `scene` 55 passed/1 ignored, replacement-budget group 10 passed and candidate-refusal fixture passed. Workspace check passed with 2 warnings; app no-default check passed with 21 warnings. The whole app lib suite was not rerun after batching; the earlier broad run and its five unresolved grip/hover failures remain recorded in `2026-11-13-ordinary-scene-display-pins.md`.

## Diagnostic timing

An ignored manual fixture created 1, 16 or 64 distinct OCCT box sources/displays, then measured five `Controller::new` calls per size on this machine, debug/unoptimized, one test thread. Document cloning before each call is excluded; validation and scene construction are included. Values are milliseconds, in execution order:

| B-reps | Per-pin admission | Batched admission | Pin-disabled diagnostic |
|---:|---|---|---|
| 1 | 7.426, 3.2421, 3.2193, 3.2004, 3.5207 | 5.6147, 3.2880, 3.3133, 3.2671, 3.3579 | 5.2027, 2.9184, 2.9249, 2.9094, 2.9133 |
| 16 | 36.6549, 36.0452, 35.7782, 35.6530, 35.7865 | 31.6302, 31.5801, 32.2530, 31.3683, 31.3919 | 30.7069, 30.4856, 30.3908, 30.9270, 31.7422 |
| 64 | 184.3488, 183.8810, 183.6083, 183.4949, 183.8872 | 128.1963, 127.9281, 127.9493, 128.4354, 128.2414 | 123.2410, 122.9380, 123.3393, 123.8080, 123.7075 |

For 64, the maximum is 184.3488 ms before batching and 128.4354 ms after; retained managed bytes were 596,201 in both pin-enabled runs and 590,569 in the pin-disabled diagnostic. This is a synthetic, sequential debug measurement, not a release budget, p95, interactive latency or process/native/GPU peak. The pin-disabled run used a temporary local diagnostic condition that was removed immediately afterward; it is not a production switch.

The three grip-drag failures and the two other grip/hover failures from the earlier broad run were individually rerun with the temporary pin-disabled condition; all five still failed. This isolates them from ordinary pin registration in these fixtures but does not establish their ultimate cause or prove the entire prior baseline. No temporary condition remains in the source. Public source audit: 16,004 files, 594 bindings, 593 SVGs, no errors; pruning removed 0 and retained 169 cited older runs.

Scene planning and batch admission still execute on the calling thread; registry lookup is still linear per distinct display, and large-instance and real-workload latency are unmeasured. NativeBrep, source-only Document/Undo, general representation store, eviction, cold restore and v5 remain open.

SHA-256: core `transaction.rs` `04ff9ca2adbe4baf24e0202603f07c32dba8ece5f73981ef7eb149db0581fcd0`; scene `scene.rs` `ce9ba108225971bd84ffc9621b1bbffdf7f1c819121a04716035bc9bb8055a84`; app `session.rs` `2fdf29acc66f9644568a7db298619c775778824ed9889b2feea950dc00df4d32`.
