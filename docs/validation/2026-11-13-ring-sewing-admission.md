# Ring sewing staging admission

2026-11-13. The selected sewing adapter has an admitted Rust entry point covering temporary index/coordinate tables and the copied B-Rep snapshot payload, with the caller's admission or cancellation error preserved. The caller accounts its input points/quads and inline receipt separately. Table sizes are overflow-checked, allocation is fallible, and actual capacities are checked; their live bytes are subtracted from the snapshot limit while they coexist. The existing `sewed_solid_with_volume` and ring-family producer use this path. The producer also checks its fixed section-table estimate and subtracts actual live section/quad capacities from the snapshot budget.

With `CARGO_TARGET_DIR=L:\om9-target`, OCCT 8.0.1 runtime, `3dm-brep`, and `-j 2`:

- Kernel `--test ring_family_brep`: 6 passed. The admitted fixture checks staging and snapshot checkpoints, final retained bytes, cancellation before allocation and during snapshot admission, and a table-budget refusal before allocation. The remaining fixtures cover four ring families, open input, source-based volume and a snapshot-limit refusal.
- App `--test ring_family_panel`: 5 passed on the producer path. Kernel lib: 33 passed, 3 ignored; it printed `File was not written with this version of the topology` after its success summary. Workspace check passed with two existing warnings.

This is a prerequisite for an admitted background Builder, not a worker implementation. The current `FeatureProducer` path still runs synchronously with default limits and does not pass a coordinator reservation into the callback. Ring section allocation is fixed-size and checked against limits but is not yet wired to an external worker lease. Native OCCT internal allocation, allocator overhead and interruption during the native call are not covered by this callback; physical OOM, process peak and desktop responsiveness were not measured. Document/Undo still own display; source-only ownership, associative source signatures, cold restore, NativeBrep and v5 remain open.

SHA-256: `creation.rs` `64a75674bd5cd323c718438c4008a60ba5773d4bd5a06d5bc3621b2b36915a47`; `associative.rs` `5d0990a15067e35ef845f985dc5ca6a6b6b2d95e8f2140af6df1d4e12a370f67`; `ring_family_brep.rs` `57e3bb2870a7ac4aa1c992dba900fd3e665f78ed94a46c6e2f7fba5c67cc5ac6`.
