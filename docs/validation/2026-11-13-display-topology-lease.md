# B-rep display topology lease boundary

2026-11-13. This closes the specific safe-API escape identified in the earlier [remesh cache checkpoint](2026-11-13-remesh-representation-cache.md); it does not complete source/display separation.

`BrepDisplay.topology` now uses a transparent wrapper whose underlying `Arc` is private outside `om9-core`. Consumers can borrow the topology. Cloning a topology value or the whole display makes an independent deep copy; callers must admit and account for that new allocation. The core can still share topology internally with its canonical source and account for it. A compile-fail doctest rejects taking an independently clonable `Arc` from a borrowed display. An application fixture keeps topology and display copies after cache replacement and controller teardown, checks that the original document and remesh kernel reservation are released, and reads the independent copies afterwards.

The serialized display fields are unchanged. Existing v4 codec acceptance and the core legacy display serialization comparison passed; this is not a v5 codec. Distinct-object manual timing fixtures now use independently allocated topology, so their future timing results are not directly comparable to their older shared-topology workload.

## Validation

Default features, OCCT SDK runtime on PATH, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -p om9-core -j 2 --lib`: 379 passed, 1 ignored. The dedicated shared-topology accounting test also passed; the display clone test checks distinct Arc allocation and equal topology.
- `cargo test -p om9-core -j 2 --doc`: 1 compile-fail doctest passed.
- `cargo test -p om9-scene -j 2 --lib`: 47 passed. `cargo test -p om9-app -j 2 --lib remesh -- --nocapture`: 73 passed, 1 ignored. The controller-teardown topology-copy fixture also passed separately.
- `cargo test -p om9-archive -j 2 --test project_codec_acceptance`: 8 passed; `--test brep_source_admission`: 1 passed. `cargo test -p om9-kernel -j 2 --lib topology -- --nocapture`: 14 passed, 3 ignored.
- `cargo check --workspace -j 2` passed with warnings. `cargo check -p om9-core -p om9-scene -p om9-kernel -p om9-archive -p om9-exchange -j 2 --tests` passed with warnings. `cargo check -p om9-app -j 2 --no-default-features` passed with 21 warnings.
- Broader checks did **not** pass: `cargo test --workspace -j 2 --no-run` encountered duplicate addon ABI symbols while linking `om9_matrix.dll` (LNK2005/LNK1169); checking all app tests encountered unresolved `CURVE_MENU` and `controller` in `curve_shape_audit.rs`. Those checks do not establish a clean full-workspace test build.
- Public source audit: 15,991 source files, 594 icon bindings, 593 SVGs, no errors. `git diff --check` passed with CRLF notices; artifact pruning removed zero runs and retained 169 cited older runs.

The cache still covers only one previous explicitly requested attested root profile. Canonical document/Undo ownership, ordinary scene, general eviction and cold restore, NativeBrep/instances, v5 and the rest of C0–C5 remain open. Desktop/GPU, concurrency stress, native peak and physical OOM are not proved. Next: continue the C0 consumer dispositions and C1 canonical document/representation ownership, then guarded source-only restore and v5.

SHA-256: `brep.rs` `5a007ab2134f84b25e623f87178c59c5f230f68d9d401539334529ef9c754475`; `transaction.rs` `56ef927fc48a9b81e3902f416eb420a8e76fdf704400505a5eaf71d19e606c53`; `scene.rs` `b15b35b94f483678ba09576786afaf7bb70795e1f6d3b832dc4a00b297a1a0a9`; `session.rs` `f5076c8cccc4d91c3bece454279820c55ab416dc262a3a6f2b2b1aa30cdbb9ae`; kernel `brep.rs` `4cc72dfbed3e06030888919c960ab25d4397bf65a9c0dd61df7a292bcf031846`.
