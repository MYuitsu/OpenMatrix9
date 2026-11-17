# Source topology signatures, scoped adapter

The later [OCCT History and Ring integration](2026-11-14-history-source-signatures.md)
supersedes this record's earlier statements that those paths still use display-derived signatures.

The OCCT adapter reads a B-rep source without creating a viewport mesh and emits
face area and center of mass, edge endpoint coordinates and curve length, and
vertex coordinates. These are source-derived numerical OCCT properties, not a
proof of exact arithmetic. Rust admits the result allocation, checks finite
signatures, and polls cancellation before parsing and between subshapes.

2026-11-14, OCCT 8 SDK, debug Windows, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo check -p om9-kernel --features 3dm-brep -j 2`: passed.
- `cargo test -p om9-kernel --features 3dm-brep -j 2 --lib source_signatures::tests -- --test-threads=1`: 3 passed. A 2×3×4 Box has six faces, twelve edges and eight vertices, total face area 52 and edge length 36, and unchanged source signatures after a remesh. A radius-5 Sphere measured source face area `100π` after dropping the B-rep display. Admission refusal, cancellation after work starts, and malformed-source refusal were exercised. OCCT prints a diagnostic for the deliberately malformed source.
- `python tools/public_source_audit.py`: no errors; artifact pruning removed no runs and retained 169 older cited runs.

This is an adapter foundation, **not History integration**: indices are local
to a source read and are not certified UUIDs or a mapping to an old display.
Current `OcctTopology` and background ring production still derive semantic
signatures from display geometry. Mapping must use an attested, unambiguous
source/topology binding and handle persisted legacy references before those
consumers can switch. Native parser, maps and OCCT integration allocations are
outside Rust admission; cancellation cannot interrupt those native stages.
Additional topology types, identity changes, worker scheduling, desktop, source-only
Document/Undo ownership, independent representation cache and v5 remain open.

## Same-root descriptor experiment

The adapter can now visit source descriptors and measure signatures while the
same parsed native root is alive. It checks a prior in-process topology proof,
matches every descriptor and oriented incidence, then maps source values to the
proof's UUIDs. A synthetic manifest exercises the mapping mechanics only; it is
not evidence that old document UUIDs were correctly bound to source geometry.

For a real Box proof produced by `capture_source_topology`, matching against
the unmeshed source read refused `topology descriptor mismatch`, even though
the source and topology binding were unchanged. The proof was captured from a
snapshot after meshing; the cause of the descriptor difference is not yet
established. Source-only capture must therefore not guess those IDs. Foreign
source/IDs, injected admission refusal and cancellation also refused safely.

`cargo test -p om9-kernel --features 3dm-brep -j 2 --lib source_signatures -- --test-threads=1`:
5 passed, including the synthetic and refusal cases. The existing topology
capture suite passed 12 tests with 3 explicitly ignored probes. No real
successful old-ID binding or History integration is claimed. Investigate the
shape-set packet's explicit references or establish an attested pre/post-mesh
mapping before connecting this adapter to product History.

## Private shape-set source measurements

`NamedTopologySource::signatures` reads its privately captured shape-set packet
without constructing a display. Native code checks that each stored reference
belongs to the reconstructed root exactly once with the captured orientation;
Rust maps those references to the packet owner's existing topology IDs. The
packet and IDs are encapsulated together; arbitrary external packets cannot be
used to construct this owner. This is an in-process path, not a persistent v5
format or general proof that arbitrary B-reps can be rebound.

The Box fixture captures a proof, remeshes and explicitly rebinds a shape-set
to that proof, then releases both displays before reading signatures. The 26
IDs match the original topology; vertex coordinates and total area 52 match the
fixture. A separate Sphere fixture releases its display and measures source
face area `100π` using the captured face ID. Injected admission refusal,
cooperative cancellation during mapping, and a corrupted private packet are
rejected. These tests do not prove physical-memory limits, arbitrary shapes,
legacy ID migration or a background product History path.

The OCCT SDK 8 debug run passed 14 topology-capture tests (3 probes ignored).
`cargo check --workspace -j 2` passed with two existing app warnings. OCCT's
parser, root validation and numerical integration allocations remain outside
Rust admission and are not interruptible within a native call. The caller
must keep the returned output reservation while holding the signatures.
