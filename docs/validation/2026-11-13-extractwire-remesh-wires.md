# ExtractWireframe from an attested remeshed B-rep

2026-11-13. Scoped I1 implementation; this is not completion of the full plan.

For one selected root OCCT B-rep whose published remesh scene has interior wires, ExtractWireframe now emits source topology edges and source-derived default face isocurves. The scene display and buffer prefix are checked before deciding whether interior wires are present. The source and the same OCCT isocurve operation used by the remesh worker supply exact NURBS output; the sampled scene lines are not promoted into document geometry. Both result arrays and their merge overlap are admitted; the combined output limit is checked. Failure is returned before the single transaction.

Before: this path returned topology edges only even when the remesh scene contained interior wires. After: the scoped Box fixture returns 12 edges and 12 interior curves via either CMD dispatch or menu dispatch. Each expected source isocurve has one output with matching endpoints, for display deflections 0.04 and 0.1; Undo/Redo restores the expected documents. A separate simulated managed-pressure fixture retains the published scene, document and Redo and succeeds on retry. Unit checks cover the combined output limit and admission refusal while requesting the second set of curves. The remesh worker's earlier source-to-scene test provides the relevant CPU-scene/picking evidence; none of these checks prove actual GPU presentation or physical desktop input.

## Commands and result

- Default features, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`, OCCT SDK bin on PATH: `cargo test -p om9-app -j 2 --lib extract_wireframe_ -- --nocapture`: 3 passed, 846 filtered.
- `cargo test -p om9-app -j 2 --lib source_wire_extraction_checks_combined_count_and_preserves_refusal`: 1 passed, 848 filtered.
- `cargo test -p om9-app -j 2 --test wireframe`: 89 passed, 8 ignored. `python tools/public_source_audit.py`: no errors; 15,984 source files, 594 bindings and 593 SVGs. `git diff --check` passed with CRLF warnings; artifact pruning removed zero runs.
- `cargo check -p om9-app -j 2 --no-default-features`: exit 0, 20 warnings; this fallback check does not exercise the OCCT path.
- SHA-256: `wireframe.rs` `b900fee80e731b0200745d33730c0369ab9162b6b2fed7bce6cd88156558c6a6`; `session.rs` `08fc27dec3e24f6f12c8eeae4f81c23ced5aa2d63bd5e3ac0d110fd4aaa9abac`.

Limitations: this dispatch is synchronous; only an explicitly remeshed, attested root B-rep with published interior wires uses the new path. Ordinary scene construction, multi-root and block occurrences, NativeBrep, saved/reopened proof, general trims/periodic faces, density and placements beyond the Box fixture remain open. A single native call cannot be interrupted. Native peak, physical OOM, GPU and desktop were not tested. Display still belongs to document/history and C0, I1 and C1–C5 remain open.

Next: integrate source-bound wires into ordinary scene and all affected ExtractWireframe occurrence paths, and move heavy extraction to guarded background orchestration; then verify trim, placement, cancellation, save/open and the full source/display ownership migration. Skill update requires the user's explicit consent under AGENTS.md.

## Multiple-root follow-up

The multiple-root extraction path now makes the same scene/source check for each root B-rep. In the tested mixed selection, a remeshed Box contributes 12 topology edges and 12 interior wires; a translated, not-remeshed Sphere contributes its topology edges. A combined output-count limit is enforced before allocating the result slots. Nested block leaves still use the earlier edge-only path because there is no corresponding occurrence remesh representation yet; NativeBrep and ordinary scenes are unchanged.

For both menu and CMD dispatch, the fixture checked one transaction and Undo/Redo, 24 Box curves, the Sphere edge count, sampled Box bounds and sampled Sphere radius after translation. This is controller/source evidence, not a proof of arbitrary placements or real desktop rendering. `cargo test -p om9-app -j 2 --lib extract_wireframe_`: 4 passed, 846 filtered; `cargo test -p om9-app -j 2 --test wireframe`: 89 passed, 8 ignored after the implementation change. Public-source audit reported zero errors and `git diff --check` passed with CRLF warnings. Current SHA-256: `wireframe.rs` `ab386cf57a351bf289b58eb2a40e4a03c2446ad1cec3c2b23e356a124067e77d`; `session.rs` `2021f36af44b2e790a5291680188e87a2f522656e8fecad9fc38143b471c4745`.
