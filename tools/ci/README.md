# Windows release build

`.github/workflows/release-windows.yml` builds native OM9 from the checked-out
public source on a GitHub-hosted Windows 2022 runner. FreeCAD itself is never
compiled: the official 1.1.4 Windows archive is verified with its pinned SHA256,
then matching source headers/bindings and import libraries are generated from
the official DLL export tables. SDK preparation does not compile FreeCAD.

Qt 6.8.3, Python 3.11.14, OCCT 7.8.1 and other headers/import libraries are fixed
by the committed `pixi.lock` (package URLs and hashes). openNURBS uses the commit
in `cmake/OpenNURBS.cmake`; Rust dependencies use Cargo.lock and toolchain 1.98.1.
CMake builds only OpenMatrix9Gui and its module dependencies, using 60% of logical
CPUs, rounded down with a minimum of one. The native BRep oracle is a separate
test target. No OM9 binary from a previous release is a build input.

CI runs public package/source-audit and installer/updater regression, Rust regression, eight independent
analytic BRep fixtures and actual FreeCAD GUI/native/worker tests with an isolated
profile. Evidence is retained as a workflow artifact; the current test summary and
native digests are included in both ZIPs. This does not replace interactive
Matrix/Rhino user acceptance or certify every command as implemented.

For a release, set `release.json`, review/merge to main, then push its matching
version tag. Tag validation requires that exact source version and main ancestry.
Only a passing tag build can publish two ZIPs plus SHA256SUMS. Provenance is signed
using GitHub OIDC/Sigstore and associated with the ZIP digests; no signing secret
is stored in source. Actions are pinned to full commit SHAs. PRs never attest or
publish releases. Publisher verifies the same-run downloaded checksums and refuses
to overwrite an existing release. FreeCAD binaries are not bundled into OM9 ZIPs.

To verify a downloaded ZIP's public-source provenance with GitHub CLI:

```powershell
gh attestation verify OpenMatrix9-0.0.3-SetupLauncher-Windows-x64.zip --repo MYuitsu/OpenMatrix9
```

Provenance establishes build origin and integrity; it is not a malware scan,
Authenticode signature or guarantee against antivirus false positives.


The branch workflow runs on `main`, `codex/release-*` and `codex/gumball-release-*`, and supports `workflow_dispatch` without a tag. Successful branch runs retain timestamped release files as Actions artifacts; they do not publish a GitHub Release. Tagged publication requires the exact source version and actual main ancestry.

Timestamp versions follow `aa.bb.cc-yyyyMMddhhmm` in Asia/Saigon time. Validate metadata after completion gates; do not increment the user-controlled `aa`. Packaging/updater regressions retain support for old triplet-only releases and reject impossible dates.

The SDK generator imports Mesh/Points exports in addition to Gui/App/Base/Part. `HostCompatibility.h` routes only transaction and permission APIs: development SDK calls retain their document-aware behavior; the stock 1.1 SDK uses its supported older signatures. The CI Gumball image helper uses the real framebuffer when the stock host lacks `renderToImage`.

`run-gumball.ps1` runs 30 isolated official-host processes, including component input, parametric owners, associative warm/cold scaling and typed Camera/Light warm/cold restore. Every case must produce nonempty passing checks and load the selected module. The suite records its exact plugin hash, reports and images; `package_release.py` refuses a mismatched or incomplete suite. Worker integration checks the accepted `rows`/`roots` protocol, source identity/attributes and direct-versus-worker native geometry. It does not claim the separate newer Layers transfer architecture.
