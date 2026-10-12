# RCORE-01 prepared CAD results — 2026-10-10

**Scope:** Prepared native CAD import and affine-preview admission, private native geometry/plain metadata, typed diagnostics and immutable validation reports. Full original RCORE-01 remains in progress.

The existing `commit3dm` path now prepares CAD host rows and preserved affine
preview rows before opening the FreeCAD transaction. Exact builtin metadata is
recursively copied under Rust depth/node/byte budgets; mutable containers never
reach observers as the caller's input. Custom values, key/container overrides,
cycles, nonfinite values and required binding-field errors reject first.
Required strings, positive finite tolerance, bools, colors and integer fields
are admitted by Rust scalar policy, with the existing host property ranges.
Row identity is reset before validation; safe exact names/UUIDs precede freezing.

The native InputTree gate bounds live CAD before geometry-owning OCCT copy.
The private copy is inspected by existing topology/shape validation; Rust uses
the versioned working V5 registry and admits only observed valid native CAD.
A native Solid additionally needs the complete valid-solid prerequisites.
Sheets, closed Shells and Compound wrappers keep their own representation.
No new solver, repair, mesh fallback or tolerance adjustment was introduced.
C++ remains necessary for Python/Part/OCCT ownership and GUI document APIs;
Rust retains scalar facts and policies, never a Python or native owner pointer.
Python changes only bind native-generated report strings through host APIs.

Caller-supplied result reports are discarded from the private snapshot. Each
CAD object stores `OM9GeometryResult`, and a preserved archive owner stores its
aggregate CAD/preview results in the same transaction. Reports identify the
operation/input, validation scope and private native-copy ownership. They are
explicitly **immutable import evidence, not current validity after editing**.
Upstream archive conversion is marked uninspected; output snapshot rows retain
ownership/type observations and are not yet full GeometrySnapshot payloads.
No topology map or exact conversion/parameter claim is fabricated.

Preparation errors are owned GeometryResult diagnostics before publication.
Native error handling rolls back before allocating diagnostics and does not
let C++ exceptions escape the transaction error boundary. Unexpected Python
binding errors are structured KernelFailure with their original cause and a
bounded exact string reason, without custom exception string callbacks.
Abort removes published objects; arbitrary-failure Redo restoration remains
an original requirement needing further work. Schema/basis/topology rejection
tested here preserves source, Undo and executable Redo before transaction.

**660 full Rust tests and3588 host checks passed; all22 matching host exits0.**
154 new prepared/metadata/cold checks +3434 regressions share plugin SHA256
`e6a7c5c7862310446e8aba7492c0c9fbd01cdacbecd9d15f0ec6ee522c84ee30`. Cold report-byte checks run in a second FreeCAD process.
The native fixture also verifies valid Point/Curve/Face/Shell/Solid/Compound
admission, forged-report isolation, real document-observer mutation of detached
source geometry/metadata, a single Undo/Redo, warm/cold FCStd persistence and
existing modeling/preservation import/export behavior. A controlled binding
fault after the real binder tests typed failure and object rollback.

Only standalone OpenMatrix9Gui was rebuilt against the unchanged shared SDK
and host, using the existing plugin cache and14 workers. [Review and reproduced
corrections](../reviews/2026-10-10-prepared-cad-results-review.md). [Machine
evidence](2026-10-10-rhino-core-prepared-cad-results.json) fingerprints source,
runtime, SDK, harness, native fixtures, reports, RED and successful logs.

**Full original RCORE-01 remains in progress.** Complete result snapshots and
upstream conversion/attributes/parameter maps, other enabled operation families,
non-CAD ownership/admission and all other original requirements remain in the
[closure audit](../reviews/2026-10-10-rcore01-closure-audit.md). These logical
metadata/native budgets do not certify process-wide allocation or native FFI.
