# Local tests, CI build only — 2026-10-12

The user requested that tests run on the local machine and GitHub Actions only
build, to avoid repeating the slow suite on the runner. This supersedes the
earlier requirement to repeat all runtime tests on CI for this candidate.

The three approved core specifications OM9-INFO-020/021/022 retain their supported
scope and documented host limits. The native implementation is unchanged by this
workflow adjustment. Current official-stock local evidence is **30 processes /
955 checks**, including parametric Scale, Object Alignment, component input and
four-view Relocate, plus Undo/Redo and fresh-process FCStd restoration.

- Plugin SHA256: `71f8431d71f6abfb36c45e9fd2a128f1d8f232368e2e5c1faf5dd3a6ff58b987`.
- Host SHA256: `7b38c5d5aa2c74f830539c1c5c44cbf6e3e8e157292c718f9ee24ac6d3e12cf6`.
- Suite SHA256: `eb8e76a4698e17e7ffccf895d6f510bc761af48bf6a3fba3b32858c054b679f1`.
- [Local stock fixture report](2026-10-12-gumball-ci-window-fixture.md).

The prior [Action 38196279682](https://github.com/MYuitsu/OpenMatrix9/actions/runs/38196279682)
compiled successfully and failed in the Ctrl+Shift component-pointer fixture.
Its first-pointer native geometry/projection matched the passing local case;
the expected Qt mouse-press observation was missing. This remains a runner
fixture evidence gap. Removing CI tests does not convert that failed run into a
passing test result.

The revised workflow compiles only the standalone OM9 plugin and dependencies,
records the exact checkout and binary hashes, then creates two ZIPs,
SHA256SUMS and GitHub provenance. It never executes `verify-windows.ps1`,
`cargo test`, native test targets or GUI macros. The public source audit and
archive/checksum integrity checks remain part of preparing the build.

Both ZIPs include `release-validation.json` with `validation_kind: build_only`,
`tests_executed: false`, build target, commit, Action URL, version, metadata and
binary hashes. `ok` refers to build/integrity, not a runtime-test verdict.
No local passing result is relabelled as a test of the CI binaries.
Default packaging retains the older full-test gate; the workflow explicitly
uses `--build-only` and rejects runtime-test fields in that mode.

Local tooling validation: **8 packaging and 4 build-record tests pass**. These
cover explicit mode selection, rejecting test claims, changed binaries/version/
metadata, wrong checkout, missing run identity and an empty worker.

Version remains `0.3.4-202610120606`: this is completion of the existing Gumball
task, not another completed spec or task. `aa` stays user-controlled. The actual
new exact-commit Action and downloaded ZIP inspection establish build success;
this report records no result before that run completes.
