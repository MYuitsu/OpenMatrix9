# Curve native admission — fresh review disposition, 2026-10-11

Read-only reviewer: `curve_native_input_admission_review`. Scope: owned Rust
native admission, accepted log/evidence, FFI, eight initial unit tests and one
ABI test. The reviewer did not edit files, run cargo, build or launch a host.

Five concrete findings were reproduced together in the expanded real Rust tests:
8 passed/5 failed. All five passed after fixes, with 13/13 green. The following
dispositions apply to this Rust foundation only:

| Finding | Disposition and evidence |
|---|---|
| Discrete selected values unrelated to complete source | Ordered source indices and transformed XYZ are checked; translated/reordered cloud fixture rejects unrelated values and accepts independently calculated world points |
| Missing/unrelated native point selection or empty range | Single source selection must match the original point and its one-input range; rejection keeps points and log unchanged |
| Missing/singular CAD root location or invalid orientation | Required numerical root location and native orientation validated; missing/singular/unknown variants reject |
| Non-Circle stale context and Circle Undo ABA | Separate monotonic admission revision covers all six constructors/configuration changes and Undo; current resource context remains restorable |
| Polyline source range incorrectly capped at1024 | Constructor-specific range bounds preserve existing Polyline capacity; point1025 is admitted |

The reviewer correctly distinguished root-location validation from CAD/world
correspondence. A baked CAD location need not equal `source_ref.world`; no such
equality was imposed. Kernel/native correspondence remains required evidence.

Additional unit-scale mismatch was reproduced RED15/16 → GREEN16/16. The revision
getter ABI had a missing-symbol link RED before implementation; the full extern-C
admission test then passed1/1, including same-generation stale capture rejection.
Full Rust suite passed871/0/0 with238 unchanged Rust/Cargo inputs.

No review finding is deferred in this scoped Rust disposition. This does not
judge the unfinished C++ bridge, final publication barriers, full parent CAD
proof, native subset indices or cold persistence. Identical full JSON snapshots
share storage; differing selection observations can still duplicate the parent.
These are explicit remaining Task2/3 requirements, not completion claims.

[Checkpoint and remaining scope](../validation/2026-10-11-rhino-core-curve-native-admission.md).
