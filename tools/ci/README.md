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

A separate `native-exchange` job builds the pinned openNURBS library and the native 3DM
reader/writer (`native/native_exchange.cpp`) for the standalone Rust host adapter. It uses
`tools/ci/build-native-exchange.ps1` and publishes `om9-exchange-native.exe`,
`om9-exchange-native.sha256` and `om9-exchange-native.manifest.json` as the
`om9-native-exchange` artifact. Like the plugin job, it is build-only and does not claim runtime
tests.

The native header-order regression loads the stock Windows distance-macro prelude
before the kernel headers and host zlib. It requires flat x64 FAR, no near/far
CAD-name collisions and an unprefixed system-zlib CRC32 oracle. The bundled
openNURBS zlib retains its private includes and static linkage; its header search
path is not exported into FreeCAD/zipios++ consumers.

As requested on 2026-10-12, CI only audits source, builds, records build identity,
packages and attests the ZIPs. It does not run regression tests or start FreeCAD.
`record-build.py` records the checked-out commit, actual binary SHA256 hashes,
version and release metadata digest. Both ZIPs retain this record with
`validation_kind: build_only` and `tests_executed: false`.

Run `verify-windows.ps1 -Root <build-root>` locally for package, installer/updater,
Rust, analytic BRep, native ABI, worker and Gumball regression. For an existing
selected host/plugin, run `run-gumball.ps1 -HostRoot <host> -PluginRoot <plugin>
-Output <new-evidence-folder>`. Keep the local results separately; they do not
certify the newly compiled CI binaries. [Current local proof and policy](../../docs/validation/2026-10-12-local-tests-ci-build-only.md).

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

The SDK generator imports Mesh/Points exports in addition to Gui/App/Base/Part. `HostCompatibility.h` routes only transaction and permission APIs: development SDK calls retain their document-aware behavior; the stock 1.1 SDK uses its supported older signatures. Local Gumball image fixtures use native FBO `saveImage` when the stock host lacks `renderToImage`.

`run-gumball.ps1` runs 30 isolated official-host processes, including component input, parametric owners, associative warm/cold scaling and typed Camera/Light warm/cold restore. Every case must produce nonempty passing checks and load the selected module. The suite records its exact plugin hash, reports and images. Default `package_release.py` retains the complete native-test gate; the workflow explicitly selects `--build-only`, requiring a build-only record and forbidding runtime-test claims. Both modes reject changed binary hashes or metadata. Worker integration checks the accepted `rows`/`roots` protocol, source identity/attributes and direct-versus-worker native geometry. It does not claim the separate newer Layers transfer architecture.
