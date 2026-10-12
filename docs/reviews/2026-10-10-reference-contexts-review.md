# Native reference-context review — 2026-10-10

Scope: uncommitted RCORE-01 reference-path increment in
`Gui/CoreDefinitionSnapshot.cpp`, `rust/src/reference_path.rs`,
`rust/src/reference_path_ffi.rs` and the Rust/native reference fixtures.
The reviewer was read-only and did not edit files, run hosts/builds or change git.
Runtime reproduction and confirmation are the controller's responsibility.

Initial missing-context findings covered explicit intermediate links, definition
tokens around arrays and native collapsed alias forms. Authored native path
oracles resolved the expected source while the adapter rejected them. The
bounded parser/resolver now preserves these contexts and native frames.

Subsequent findings were reproduced with real source/placement fixtures:

- Physical element labels resembling collapsed aliases incorrectly selected a
  different occurrence. RED7 showed wrong world XYZ in both LinkTransform modes.
  Rust now validates an explicit array mode; physical arrays use decimal indices
  or actual native element names/labels. Two Rust tests also failed before this
  correction and passed after it.
- Bounded collapsed aliases outside the actual count blocked native definition
  label fallback through occurrence0. RED7 rejected four direct/group cases;
  fallback4 accepted them without permitting overflowing numeric parsing.
- Direct selections with an explicit definition name lost context in all four
  physical/virtual/LinkTransform modes. RED7 failed; definition normalization
  retained the actual target and frame.
- Physical selection normalized a repeated definition prefix twice. RED8/9
  proved two native-invalid paths were accepted. Physical capture now passes
  the original suffix to the element and normalizes once; original object-path
  preflight rejects invalid traversal before returning a snapshot.
- A suppressed physical element's canonical owner path remained valid while
  its member traversal was invalid. RED9 proved the two modes were accepted.
  Original object-path identity checking rejects them.
- Final `Face1` selectors could still traverse suppressed definitions in direct
  arrays and group references. RED10 proved both native-invalid inputs were
  accepted. The shared LinkElement suppression guard rejects nonempty traversal.
- The new original-path check rejected collapsed occurrence-only selection with
  configured Face1/Face3 collections. RED10 reproduced the regression. Bounded
  occurrence-only selection now uses the existing canonical numeric identity
  check and retains the ordered native configured selectors.

Fresh final source review found **no remaining actionable defect** in this
bounded increment. It specifically accepted single normalization, the terminal
configured-occurrence exception and the shared suppression guard. No new parser
or FFI memory-safety defect was found by inspection. This does not certify
initialized pointer storage, native dependencies or global process memory safety.

Final current-runtime evidence is recorded separately in
[the validation record](../validation/2026-10-10-rhino-core-reference-contexts.md).
**Full original RCORE-01 remains in progress.** Configured topology traversal,
overlapping selection, native ownership/cycles, attributes/channels and all
original certificates/reports/allocation/worker/commit/lifecycle obligations
remain in the original closure audit.
