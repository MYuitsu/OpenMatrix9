# Owned B-rep source transfer

2026-11-13. C1 ownership prerequisite, not completion of the canonical document migration.

The candidate-to-source transfer previously returned independent `Arc<String>` and `Arc<BrepTopology>` values. A caller could clone either and let it outlive the owner expected to account for the source. The transfer now returns `OwnedBrepSource`, which retains both allocations without the display, exposes only borrowed source/topology, and is not cloneable. Its caller must still retain an appropriate reservation; the type is not itself an admission guard. The existing shape-set source uses this owner, reports the allocation bytes, and continues checking the packet against the source before restoring topology. The weak diagnostic source-liveness receipt does not expose an upgradeable owner and its weak header must be accounted for by its caller.

Validation with OCCT runtime on PATH, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -p om9-kernel -j 2 --lib shape_set -- --nocapture`: 3 passed, including source-only restore, foreign-source rejection, topology IDs and native measurements on the covered fixtures.
- `cargo test -p om9-core -j 2 --doc`: 2 compile-fail doctests passed, covering attempted independent topology Arc access from the owned source and display.
- `cargo test -p om9-core -j 2 --lib`: 379 passed, 1 ignored. `cargo test -p om9-app -j 2 --lib remesh`: 74 passed, 1 ignored, including remesh source/display lifetime and reservations.
- `cargo check --workspace -j 2`: passed, 2 warnings.

This does not prevent an unaccounted clone of an entire `Brep` candidate, and does not establish a general representation store or source-only Document/Undo. NativeBrep, cache eviction, cold restore, v5 and desktop behavior are outside this checkpoint. Next: migrate canonical Document ownership and the existing scene/transaction consumers under the main plan; do not treat this transfer as a substitute for tracked document sources.

SHA-256: `brep.rs` `0ec07092313934538136e8862c277dbe61ada2dedbee83a837d979c6ee8d094e`; `topology_capture.rs` `6882ef297fc4052c3fee57aed0a474e2f6aee68cffbca20c0a51db230389dec4`; app `session.rs` `830c169f42221550c35b345e4a5efa4600c60ac35c383485bfe29caef273cf5c`.
