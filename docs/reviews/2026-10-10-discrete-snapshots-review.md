# RCORE-01 discrete snapshot review — 2026-10-10

Reviewer `/root/discrete_snapshot_review` inspected the designated increment
read-only, without builds, host launches, file/git mutations or child reviewers.
The review covers the current owned mesh/cloud model, ABI, native storage reader,
descriptor/definition integration, delayed Mesh dependency and real-host fixtures.
It does not claim complete RCORE-01 or complete C05/C06 validation.

No actionable production defect was found in this scope. Exact builtin property
checks precede native casts. Mesh additionally checks registered MeshObject type;
Points does not register PointKernel in AppPoints.cpp. Its exact PropertyPointKernel
constructs/owns that concrete kernel and returns it through getComplexData(), so
the cloud cast has an explicit native ownership obligation. Counts/retained-byte
limits precede native staging and ABI slices. Coordinates/indices validate before
Rust cloning; bounded JSON completes before any output copy. Raw points stay
separate from the native kernel transform, composed once by the shared frame path.
Native mesh/cloud owners take precedence over Group extensions. Shared Mesh.lib
and delayed Mesh.pyd resolve MSVC-exported STL readers without a second host build.

The reviewer recommended P3 coverage at rust/tests/discrete_snapshot.rs for live
ABI rejection of NaN, invalid triangle indices and serialized overflow with an
unchanged sentinel output. The controller added
`invalid_values_and_serialized_overflow_preserve_live_ffi_output`; it exercises
NaN/Inf/out-of-policy values, out-of-range/repeated/maximum indices, and a count-valid
cloud whose JSON exceeds the serialized budget. The full Rust run passed628 tests.

Native Undo diagnostics show Points recreates Placement from its stored matrix;
placed getters/rotation entries differ by one double ULP after this reconstruction.
Only the Undo restoration comparison permits1e-12; exact descriptor read-only
preservation and restored raw owned geometry comparisons remain exact.

Declined-to-judge completion claims remain required work in the original audit:

- Serialized overflow is detected after bounded native staging/Rust cloning,
  not before every allocation. Serializer growth bounds are not process/global
  allocator or peak-staging certificates.
- Cold loading/final host acceptance require runtime evidence, separate from this
  source review. The controller added a native module-loading fixture using the
  operating-system module handles before/after plugin and native module imports.
- Initialized immutable live ABI buffers, nonoverlapping output, exact native
  ownership and matching SDK/STL ABI remain caller obligations; no whole-program
  memory-safety claim follows from this review.
- Other native flags/segments/channels, archive float64 preservation, mesh/cloud
  subelement maps, instance arrays/reference contexts, full attributes, topology/
  solid certificates, reports, typed diagnostics, uniform operation integration
  and remaining allocation/worker/commit gates remain open.

Controller disposition: retain all these original requirements; source presence,
the reviewer verdict and a passing subset do not establish chapter completion.
Final runtime results and fingerprints belong to the discrete snapshot evidence.
