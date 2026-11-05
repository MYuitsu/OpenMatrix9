# Top11 Move command snapshot — 2026-10-30

The existing command snapshot regression expected no option row for Move. This was stale after the documented Move Vertical implementation: its option is a real command option, not a placeholder. Updated the snapshot assertion to require exactly the Vertical option and corrected the outdated comment describing Top11 options. No product behavior changed.

On Windows x64 with `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`, the snapshot suite initially failed the old empty-options assertion (2 passed, 1 failed). After the correction, `cargo test -j 2 -p om9-app --test command_snapshot --test top11_options` passed all 3 snapshot and 10 option tests. The existing option tests cover activation, CPlane displacement, typed points, cancellation and Undo. Public source audit passed with zero errors. Artifact pruning removed zero runs; 116 older cited runs remain protected.

This does not establish physical desktop acceptance or complete Top11. In particular, Split currently accepts curve targets but its surface and polysurface target paths remain to be implemented and tested; the M03-UI human gate remains open.
