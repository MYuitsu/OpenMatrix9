# World frame and source attributes review — 2026-10-10

Fresh read-only reviewer examined the Rust frame/attribute model and ABI,
instance representation gate, native attribute extraction and descriptor frame/
link chain integration, with the plan and fixtures. Reviewer did not build,
run the host, edit source or review unrelated historical pending increments.

P2: the owned attribute budget used ABI header sizes192/120, undercounting
retained Rust String/Vec headers216/176 on this64-bit host. An accepted4096-
material input could exceed the declared1MiB retained-data limit by98360 bytes.
The aggregate boundary test failed before the fix. Accounting now uses actual
`size_of::<OwnedMaterial>()` and `size_of::<SourceAttributes>()`, plus UTF-8
contents, before cloning. The exported native count gate uses that same policy.
The reviewer confirmed the fix and reported no remaining actionable issue.

Main-agent verification also corrected an unknown/empty ambiguity: a native
view provider without ShapeAppearance must serialize that field as null. A
dedicated inspected bit16 distinguishes it from presence of a provider (bit8).
The Rust regression failed before this correction and passes afterward.

The expanded host fixture checks all four nested LinkTransform combinations,
mirror/nonuniform scale, compound-face and solid-instance face frames, separate
instance/definition attributes, live instance A after editing B, Undo/Redo,
source lifetime and atomic missing/member/group/singular reference rejection.
Review is source evidence; accepted runtime evidence is recorded separately in
[validation](../validation/2026-10-10-rhino-core-world-attributes.md).

Unsupported scope remains explicit: member/subelement references, group and
mesh/cloud instance definitions and arrays, arbitrary app properties and Rhino
userdata. This focused review does not establish full RCORE-01 completion or
global native memory safety.
