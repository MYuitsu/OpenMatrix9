# DupEdge temporary view uses the current scene profile

2026-11-13. The OCCT DupEdge source-view worker used to derive its requested deflection from the B-rep display in the document. It now reads the published scene display and caps the deflection at 0.01 mm as before. The worker still captures canonical source independently and validates the scene publication before publishing; this removes one consumer of document-owned display but does not remove that ownership.

The controller fixture starts with a sphere whose document display deflection is 0.1 mm, remeshes the scene to 0.005 mm, and starts DupEdge. The temporary scene uses 0.005 mm; the coordinator snapshot and Redo remain unchanged, and cancel restores the preceding scene publication. The existing source-view tests separately cover source edge extraction, placement, Undo/Redo, stale publication, and refusal. This is not a desktop/GPU test, an independent geometry oracle for this two-profile fixture, or evidence for NativeBrep profile selection.

Validation with OCCT 8.0.1 SDK runtime on PATH, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`: the new fixture passed; `cargo test -p om9-app -j 2 --lib dupedge_source_view -- --test-threads=2` passed 26; `--lib topology_remesh -- --test-threads=2` passed 53, ignored 1; `cargo check -p om9-app -j 2 --no-default-features` passed with 21 warnings. Public source audit before this report: 16,004 files, 594 icon bindings, 593 SVGs, no errors. Artifact pruning removed zero runs and retained 169 cited older runs.

An initial command incorrectly expanded the target environment variable and started compilation in the existing checkout `target` directory; it was interrupted promptly. All reported tests used `L:\om9-target`; no existing target directory was deleted.

Five previously observed grip/hover test failures remain unresolved. Canonical Document/Undo still own display, and general cache eviction, background cold restore, NativeBrep separation and v5 remain open. Next: migrate canonical ownership and its callers while preserving atomic worker publication.

SHA-256: `session.rs` `b2cdf8f787fc6343420b470fe1467ea1514d14de2e022b682078c31315ae1b0d`; `topology_remesh.rs` `fb4e091b95ee2ffe357a5aa7463e37d518ebcb3e7e71c6d2f8c969ad7e0d5549`.
