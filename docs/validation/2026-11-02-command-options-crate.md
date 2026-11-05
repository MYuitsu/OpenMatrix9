# Command option parser crate experiment — 2026-11-02

The existing parser and its four unit tests moved unchanged from `om9-app` into
`om9-command-options`, with `om9_app::option_parser` retained as a re-export.
The crate depends on `om9-core` and introduces no second command engine. A byte
comparison against the previous tracked parser matched after normalizing line
endings and excluding the new `forbid(unsafe_code)` attribute.

Environment: Windows, rustc/cargo 1.98.1, `CARGO_TARGET_DIR=L:/om9-target`,
`-j 2`, developer fallback (`--no-default-features`) for app checks. Timings are
wall clock with warm caches, not clean builds. Before extraction, one actual
comment edit followed by `cargo check -j 2 -p om9-app --no-default-features
--lib` took 4.49 s. After extraction, a comment edit followed by the same app
check took 2.18 s; an isolated crate check after a comment edit took 0.27 s.
An app check after temporarily changing an error string in the parser took
2.36 s. Source contents were restored following each measurement.

These are single observations, different incremental states, not a controlled
benchmark. Other Cargo processes shared the target: the isolated check after an
error-string edit waited for a build lock (1.19 s and 17.30 s in two attempts),
so neither is a valid clean isolated timing. No full-build speedup or percentage
is established. Next, benchmark representative real changes repeatedly in a
dedicated target with no concurrent builds, including product features and link
time, before deciding whether more crate splits are worthwhile.

Validation: `cargo test -j 2 -p om9-command-options` passed all four unit tests;
`cargo fmt -p om9-command-options --check` passed; the app fallback library
`cargo check` passed, as did the app default-feature library `cargo check`;
`cargo test -j 2 -p om9-app --test command_options_interface` passed all 74
tests with default features;
public source audit reported zero errors. The attempted
`command_options_interface` integration test with `--no-default-features` did
not compile because it references the disabled optional `om9_kernel`. This is
not evidence for or against product-runtime behavior. The fallback
`command_inline_options` suite had 7 passes and 2 failures: starting Box requires
OCCT, and the catalog sweep observed 21 commands with options against its
threshold. These failures are not attributed to the extraction without a
controlled baseline. No feature-completion or
Matrix UI acceptance status is promoted by this refactor.

## Isolated executable build comparison

On 2026-11-02, two source snapshots were built sequentially on separate initially
empty target directories on the same Windows machine. The snapshots captured the
working tree, including pre-existing changes. Before measurements, the earlier
layout was reconstructed in one copy by moving the parser back into the app and
removing the added crate dependency and re-export. A subsequent byte comparison
found no differences across 1,426 other files under crates, Resources, sdk,
launcher and native. The original workspace was not edited for the measurements.

Both builds used Rust/Cargo 1.98.1, the same existing OCCT/openNURBS SDKs,
`CARGO_PROFILE_DEV_DEBUG=0`, and `cargo build -j 2 -p om9-app --bin om9
--offline`. This is a dev-profile executable with default OCCT features and no
debug symbols, not the release profile or the default debug-symbol profile.
Each target was built from scratch once, in before/after order: 217.54 s and
201.05 s respectively. Those two single observations cannot establish a clean
build speedup; the second build also benefits from a warmer OS file cache.

For incremental measurements, a temporary error-string change in the parser and
an independent temporary app environment-variable string change were rebuilt
five times each per layout, in alternating before/after order. Each build
completed successfully, including linking the app executable. Wall-clock
seconds, in paired iteration order:

| Change | Before samples | After samples | Median before | Median after |
| --- | --- | --- | ---: | ---: |
| Parser implementation | 9.997, 8.434, 9.466, 7.889, 7.702 | 8.742, 9.736, 11.798, 8.781, 8.510 | 8.434 | 8.781 |
| App implementation | 8.093, 7.918, 7.800, 8.109, 8.233 | 8.203, 7.784, 8.163, 8.614, 8.010 | 8.093 | 8.163 |

The first app sample on each side also rebuilds the restored parser after the
preceding scenario. Median differences are small relative to observed variability:
this experiment **does not show an app-build speedup** from this extraction.
The parser change rebuilt `om9-app` on both sides, and also rebuilt
`om9-command-options` after extraction. The measurements do not rule out a
benefit when building or testing that crate independently, for other edits, or
with other profiles and hardware.

The harness sampled other `cargo.exe` and `rustc.exe` processes throughout
each build and would stop on detecting one; none were detected during the 22
successful builds. This does not control other system activity. Raw per-run
wall times are preserved in `docs/validation/2026-11-02-command-options-benchmark.json`;
the temporary targets and sources can be removed after recording the results.
