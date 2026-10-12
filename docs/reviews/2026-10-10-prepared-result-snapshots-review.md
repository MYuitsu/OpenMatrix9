# Prepared result snapshots review — 2026-10-10

One fresh read-only reviewer inspected the original chapter, prepared snapshot
plan, Rust parsing/output/ABI and unit revision handling, private native CAD
serializer, schema/capture, CMake/lib wiring and native fixture. The reviewer
did not modify source, run a host/build or spawn another reviewer.

The known optional-field issue was confirmed: non-working CAD capture reads
geometry_kind/class_name/representation even when working-only validation
does not run. geometry_kind=None set a pending Python integer error; the
transaction boundary later reported KernelFailure without current row identity.
The controller reproduced this before the fix:95 checks, two failed assertions,
actual host exit0, no publications or source/Undo/Redo changes.

Exact optional kind/string fields are now validated with the existing Rust
scalar policy before snapshot/native getters. Working-mode required fields
remain required. The matching final metadata fixture passes95 checks, including
UnsupportedRepresentation/current identity and executable Redo for that case.

No additional actionable ownership/ABI/schema regression was found. Master,
parser and output bounds, full-width revision ownership, root placement and
orientation, schema fields and build wiring were inspected. Unknown upstream
exactness/parameter maps and remaining chapter obligations stay explicit.

Minor deferred: native stream byte-budget exceptions map to KernelFailure
instead of InvalidBasis. They reject before publication; the reviewer did not
consider this a blocker for the snapshot increment. Uniform chapter-wide error
mapping remains open and this limitation is retained in the evidence ledger.

The separate cold-oracle failure was investigated against the FreeCAD source:
BoundBox consumes display tessellation. The oracle now compares detached copies
without mesh cache; cached bounding differences were not geometry loss. Final
matching evidence:665 Rust tests,3701 host checks,24 actual host exits0.

Full original RCORE-01 remains in progress; native master restoration does not
prove upstream conversion/attribute/parameter preservation or integration of other families.
