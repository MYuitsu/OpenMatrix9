# Reusable Builder History and native cage — 2026-10-09

The supported foundation is implemented and verified in the matching FreeCAD
SDK. Feature contracts remain `partially_implemented`, with
`validated_supported_slice` for the scope below. This does not enable every
original Gem Builder or reconstruct arbitrary Rhino plugin History.

## Supported behavior

- Durable native BuilderRecord stores exact feature identity, the complete raw
  versioned recipe, explicit gem-shape tag, initial frame/dimensions and independent
  template geometry. Multiple records survive deletion of every output. Explicit
  restore recreates missing slots; deletion does not automatically resurrect them.
- BuilderOutput participates in native Record/Update/Lock/Clear, independent-edit
  detach, Undo/Redo and FCStd restoration. Group frames are included once. Cold
  restore imports native types without requiring prior workbench activation.
- Programmatic Match Attributes replays all supported records onto same-tag native
  gems in one transaction. Rust rejects incompatible inputs and aggregate batches
  exceeding 4,096 records or 16,384 outputs before writes.
- Create Cage uses World BoundingBox and explicit U/V/W counts/degrees. CageEdit
  binds selected native Part/Mesh objects to a separate native 3D cage, with Global
  or Local box/falloff. ReleaseFromCage freezes selected captives. Menu/CMD routes,
  cancellation, stale-input rejection, editable CVs and Undo are verified.
- CageBinding retains frozen geometry and recovered current parameters. Recompute
  never compounds earlier deformation. Its dependency stays active with global
  History Record/Update Off. Control/captive deletion preserves the recipe for
  Undo; invalid controls clear affected geometry and repair rebuilds it.
- Retained Rhino 5 ON_NurbsCage and supported 3D ON_MorphControl become native
  editable controls. Only explicit captive UUIDs in the same verified import
  namespace are rebound. Current archive outputs are inverted in the current
  cage before subsequent edits, avoiding a second initial deformation.

Native interfaces and precise limits are documented in the
[Builder contract](../features/builder-history-native-contract.md) and
[Cage command contract](../features/cage-command-native-contract.md).

## Mathematical and ownership boundaries

Safe Rust owns bounded JSON parsing, recipe versions and evaluator validation,
shape compatibility, batch budgets, cage basis/rational evaluation, inversion,
affine proof, local attenuation, box construction, command state and retained
manifest/UUID policy. Builder JSON keeps its original 1 MiB/depth32/16,384-node
limits; retained manifests use 32 MiB/depth64/1,000,000-node limits. Invalid Unicode
escapes, duplicate JSON keys, semantic duplicate UUIDs and nonfinite data reject.

C++ is the required FreeCAD/Qt/OCCT/openNURBS adapter for persistent properties,
native links, GUI-thread transactions, selection, kernel topology and typed archive
decoding. Python is fixture/bootstrap glue. Rust owns its handles and copied data;
FFI inputs are borrowed for one call. Native pointer provenance, allocation and
dependency safety remain native obligations, not a memory-safety certification.

The archive adapter reads one bounded snapshot (at most 512 MiB), verifies its
SHA256 and decodes those exact bytes. It does not reopen the path after verification.
The SDK full-file `ON_BinaryArchiveBuffer` requires one additional bounded buffer
copy. Canonical UUID normalization is shared across selected records and captive
resolution, including equivalent braced/uppercase identities and duplicate checks.

Exact affine proof uses local cage CVs and a separate owned world pose. Transforming
CVs into rounded world coordinates before proof can lose affine classification.
Archive controls therefore retain local CVs and native Placement. Native BRep
mapping clears the input Location, composes it into the matrix, copies geometry,
then applies object Placement once: OCCT can retain Location even with copy=true.

## Geometry limits

The registered Builder evaluator is `om9.affine-template`, version1. Saved settings
are preserved metadata; original setting/cutter solvers, style selection UI and
`.mss` interchange remain unimplemented. The original Gem Match command remains
disabled; the reusable native API is available for future commands.

Affine cages support full valid native BRep topology. Nonlinear structural mapping
supports triangular meshes, single NURBS edges and verified rectangular faces
without holes/seams. General trimmed surfaces/polysurface refit, exhaustive nonlinear
inverse uniqueness, Rhino Accurate/Fast modes, 1D/2D controls, localized archive
morph variants and additional coordinate/base modes remain unsupported. They reject
or remain disabled rather than reporting successful geometry.

## Acceptance evidence

Build: `H:/FreeCAD-src/build/openmatrix9-history-cage`, matching SDK
`H:/FreeCAD-src/build/relWithDebInfo`, native module
`H:/FreeCAD-src/build/reusable-history-runtime/bin/OpenMatrix9Gui.pyd`.
The exact runtime checksum, accepted report paths and counts are recorded in
[the acceptance receipt](../../build/validation/reusable-builder-cage-20261009.json).
Reports include process exit0 as well as every individual assertion passing.

The final private-runtime suites passed **529 checks**: 57 Builder/cold/grouped
checks, 142 cage/3DM checks, and 330 Edit/History/Curve/Surface regression checks.
The installed SDK module then passed **4 cold-restore + 26 command checks**.
Rust passed **256 tests across 41 suites**, optimized targeted suites passed **49**,
and Python tools passed **23**. Both standalone native executables passed.
Clippy exited0 with 56 warnings; this is not a warning-free lint result.

The verified native module was installed into
`H:/FreeCAD-src/build/relWithDebInfo/bin/OpenMatrix9Gui.pyd`, with the prior module
backed up. Installed and private modules have SHA256
`e442e530d63a299742943f1c0e2ed3702740a3c5fc4c16a20115df9ffa21c5b7`.
The installation receipt records the backup and the installed cold report confirms
that FCStd namespace loading used the SDK's default module path.

The final verification includes full Rust tests, optimized cage/recipe/manifest
tests, Python tool tests, both standalone native cage executables, Builder storage
and cold restore, grouped output/budget cases, cage commands/deformation/lifecycle,
typed 3DM restoration and existing Edit/History/Curve/Surface regressions.

The native 3DM regression verifies translated/rotated retained controls, additional
native solid capture, local CV edits and saved pose reuse, semantic UUID ambiguity,
manifest tampering, unit/domain handling and byte-identical retained archives.
The direct cage fixture rejects empty Part captives safely before native writes.

Review-driven fixes include strict Unicode parsing, native exception containment,
canonical identity resolution, verified-byte decoding, affine local-pose proof,
immutable BRep copying and placement preservation. Relevant failing native and Rust
cases were reproduced before their fixes; failed reports remain in the local build
artifacts rather than being counted as acceptance.

Source audit still reports 290 existing private/binary reference artifacts, with no
other audit errors. Authored icon aliases give 511 bindings and 510 SVG assets.
This local workspace is not a publication-ready audit, and no publication was made.
