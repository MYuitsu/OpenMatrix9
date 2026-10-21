# Windows Actions — standalone build correction, 2026-10-21

Status at local verification: product build/package and loader checks pass; a
GitHub-hosted run of the modified workflow remains pending. No commit, push, tag
or release was made during that verification checkpoint.
This is build infrastructure evidence, not a manual M01-UI or geometry acceptance.

## Failure observed

Inspected [run 39539757459](https://github.com/MYuitsu/OpenMatrix9/actions/runs/39539757459),
source `f77ec97d1c104a230a539d530077c97929c8a95f`:

- [native-exchange job](https://github.com/MYuitsu/OpenMatrix9/actions/runs/39539757459/job/119026042707):
  Ninja chose GNU 14.2.0 from `C:/mingw64/bin/c++.exe`; openNURBS failed with
  `#error do not define WIN32 for x64 builds`.
- [build job](https://github.com/MYuitsu/OpenMatrix9/actions/runs/39539757459/job/119026043043):
  the historical FreeCAD plugin target failed in `CorePreparedCad.cpp` because
  `Mod/Mesh/App/MeshPy.h` was absent. That target also did not build the active Rust product.

## Resulting build path

The active workflow now calls `tools/ci/build-standalone-windows.ps1`: MSVC x64,
root Cargo workspace, Rust 1.98.1, `--locked --release -p om9-app --bin om9 -j 2`,
default `occt-brep`, Cargo target outside the checkout. OCCT 8.0.1 archives and the
approved TKG3d patch are verified; the patched toolkit is built in a separate root.
`build-native-exchange.ps1` explicitly configures Visual Studio 2022/x64 and Release
instead of allowing Ninja to select MinGW. The native helper and product job retain
their separate artifacts. Historical plugin scripts are preserved for legacy maintenance.

`package_standalone.py` creates one portable ZIP plus `SHA256SUMS.txt`, with app-local
PE dependency closure, MSVC CRT, native adapter admission digest, licenses, patched
TKG3d source/build recipe and per-file build manifest. The pinned vendor download
contains an inner ZIP; packaging handles this explicitly and has a regression test.
Missing dependencies and mismatched build/pin identities fail rather than emitting
a ZIP that only loads on a developer machine.

CI still audits/builds/packages only, with `tests_executed: false`. Local checks
below do not become CI runtime-test claims. Tag/version/main-ancestry guards remain;
publication requires both jobs and verifies the same-run portable ZIP/checksum.
No old FreeCAD launcher package is relabeled as a standalone launcher.

## Verification

- Actionlint 1.7.12: workflow passes. Official Windows asset SHA256
  `6e7241b51e6817ea6a047693d8e6fed13b31819c9a0dd6c5a726e1592d22f6e9`.
  The check caught and corrected an unavailable workflow-level `runner` context;
  `runner.temp` is now used only in the build step's environment.
- PowerShell AST parser: all three new/modified CI scripts pass.
- `python -m unittest tools.tests.test_standalone_ci_package tools.tests.test_release_package`:
  14 tests pass (six standalone closure/integrity/vendor-archive tests, eight retained legacy tests).
- Full product script: exit 0 at
  `build/ci/standalone-local-r2-20261021`, using `CARGO_TARGET_DIR=L:/om9-target`.
  The preceding run built Rust successfully but caught the nested vendor ZIP at
  packaging; it is retained as a failed packaging checkpoint.
- Fresh Visual Studio x64 configure identifies MSVC 19.44.35208.0. Both zlib and
  the full openNURBS static target build from source, and the native 3DM executable
  links successfully against these newly compiled libraries. The product build
  reused the existing pinned libraries; this additional clean native build tests
  the path a fresh Actions runner takes. Hashes are in the JSON receipt.
- The produced ZIP was extracted to a separate directory. All 432 manifest hashes
  match. With only Windows System32 on PATH and all OM9/OCCT environment overrides
  removed, `om9 --create-fixture` and `om9 --inspect` return 0. The native executable
  loads and returns its documented usage diagnostic (expected exit 1, no geometry operation).
- Public source audit: no errors; final counts and source hashes are in the
  [machine-readable receipt](2026-10-21-standalone-github-actions.json).

Local ZIP: `OpenMatrix9-0.3.5-202610121520-Standalone-Windows-x64.zip`, 21,787,474 bytes,
SHA256 `9d61e5b4c7b20ffcff8da8fe9c93481e91dc4d1af52fb58edbe539740139efa8`.
The version was preserved from `release.json`; this task does not assign a release version.
Binary SHA256 `30f5fe6c76a75e025a9eb6f23f8d4cc41465f7288960ae699cea26fb11ab140a`.
Local manifest reports `source_dirty: true`, distinguishing this working tree from
a clean CI checkout. A CI package requires a clean checkout and matching `GITHUB_SHA`.

## Remaining verification

Run the committed changes on GitHub-hosted Windows after an authorized push. Fresh
runner downloads, permissions, artifact upload, attestation and tagged publication
are not established by local compilation. No release was created to test publication.
GPU UI startup/manual gates and native geometry conformance remain separate from
these CLI/archive/loader checks. No reusable business rules or skills were changed.
