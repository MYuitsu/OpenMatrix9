# OM9-SOLID-004 — Boolean2Objects

Source: `specs/04-solid/om9-solid-004-boolean-two-objects.md`; Matrix 8 Book 1, printed
p.214, PDF pp.224 verified on 2026-10-06.
Command continuation was read through the next feature heading.

Source behavior stays in the original specification. The normalized inputs,
units/tolerances/defaults, output/placement, preview/cancel/errors, dependency
guards, Undo/Redo and FCStd persistence are documented in
[native Edit contract](edit-native-contract.md).

Implemented slice: Exactly two native solid inputs; mouse/Next cycling through Union, A−B, B−A, Intersection and Inversion Intersection (XOR), DeleteInput and persistence; open-surface/mesh booleans unsupported.

Invocation: stable menu/F6 ID `OM9_SolidBooleanTwoObjects` and English CMD `Boolean2Objects`.
Native semantic/lifecycle fixtures use this exact ID in
`tests/edit_commands_smoke.FCMacro`. Evidence: `build/edit_commands_smoke-1/fd9e161d7d9847f5b93ee59c9e2b9307/results.json`;
115/115 checks passed with process exit 0.
The SDK build passed. Rust state/permission checks are in
`rust/tests/edit_session.rs` and `rust/tests/edit_commands.rs`.

Status: `partially_implemented`, `validated_supported_slice`. No full
Matrix/Rhino compatibility or live History recomputation is claimed. Continue
with the remaining options/types above and their cited original source pages.
