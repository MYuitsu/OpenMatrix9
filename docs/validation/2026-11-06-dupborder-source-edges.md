# DupBorder B-rep source edges — 2026-11-06

The B-rep command previously produced polylines from display segments. It now
converts each source OCCT edge to an owned rational NURBS curve, applies the
entity placement, and prepares all curves before beginning output commits.
The native adapter owns transient OCCT operations; Rust owns the returned curve.
Unsupported or degenerate native edges return an error instead of using the
display approximation. A build without OCCT explicitly refuses B-rep DupBorder.

`cargo test --locked -p om9-app -j 2 --test dup_border`, with the configured
OCCT 8.0.1 runtime and `CARGO_TARGET_DIR=L:\om9-target`, passed 4 tests. They
cover an existing surface case, circular boundaries at two display deflections
with sampled radius and translation checks plus Undo/Redo, and a rational
NURBS/straight-edge fixture with sampled comparison to the input curve. The
first NURBS fixture used an open profile, which the extrusion adapter rejected;
the final fixture closes the profile. B-rep and surface output are now inserted
by the existing batch transaction, so one Undo removes all output curves. A
fixture constrained by the object limit confirms a failed B-rep batch preserves
the document and an existing Redo entry. `cargo check --locked -p om9-app
--no-default-features -j 2` passed, with existing warnings.

These samples do not establish global error bounds or support for every OCCT
curve type, trimmed/seam case, orientation, general affine placement, kernel
version, or legacy ID mapping. Native allocations are outside the managed Rust
reservation and synchronous per-edge source parsing has not been benchmarked.
The pressure fixture covers one admission refusal, not every native, memory or
publication failure. Desktop invocation and visual selection were not tested
in this batch. P1, v5 and display controls remain open.

Follow-up: boundary lookup and exact edge extraction validate the source and
topology without traversing the display mesh. Edge extraction no longer calls
the full area/volume measurement before every edge; OCCT still parses and
validates the source separately per edge. Four DupBorder and 31 core
curve/surface tests passed. A fixture sets the point limit below its display
mesh size but above its NURBS pole count: full display validation fails while
boundary and edge extraction succeed. No timing or large-model speedup is
claimed, and native parsing and allocation remain outside the Rust budget.

The production B-rep command now sends all requested boundary indices to one
OCCT read and topology-map traversal. Four DupBorder tests passed again; a
fixture compares reversed batch order to individual extraction and rejects an
out-of-range index. Output counts and Rust-owned curve capacities are checked;
the OCCT shape and conversion allocations are still outside this accounting.
This is a structural reduction in repeated reads, not a measured performance
result.
