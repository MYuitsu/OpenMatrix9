# Native Curve bridge review disposition — 2026-10-11T22:26:14+07:00

A fresh read-only source reviewer examined C++ capture/admission, Circle context
restore, owned Rust source pairing and FFI. It did not edit, build or run hosts.
Two concrete P2 findings were accepted after checking current sources.

1. `curve_evidence.rs` advertised a nonexistent zero-based input collection while
   source refs1..N addressed native-only snapshots. An independent Rust consumer
   failed0/1; fixed descriptor resolves two distinct owners, focused18/18. Native
   diagnostic12/12 and cold source21/21 also follow the advertised descriptor.
2. `captureCurveNativeSource` let digest/attributes/topology helpers escape without
   actual command/source. A real over-budget Label triggered raw untyped failure
   on both commands: runtime49 RED8/12. The capture boundary now adds typed actual
   context and preserves helper details/types. Runtime50 GREEN12/12 also proves
   no output/Undo on rejection, same-session repaired retry, one accepted source
   event and correct independent dimensions/Undo/Redo.

No concrete finding is deferred within this review scope. Broader missing
GeometryResult/publication, generic snapping, subset sharing/budgets, conversion
maps and lifecycle acceptance remain required work; this review does not close
them. [Checkpoint](../validation/2026-10-11-rhino-core-curve-native-bridge.md).
