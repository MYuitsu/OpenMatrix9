# GPU Vulkan identity and navigation replay, 2026-11-04

## Regression found in release

Two paired hidden-window replays used the same Vulkan RTX 3060 / NVIDIA 610.88, simulated 100% DPI, four viewports, 30 Hz target, 90 scripted steps and isolated default settings. The 125- and 729-object fixtures stayed unchanged. The earlier release SHA-256 was `ac8c48503c0f299f40aa90a64877953f8dde311b6be4f7e2f481231a47b93837`; the release containing the new window inventory was `959bffd5fe6abc3198449edd11b2a45fd044ccdea87bb03f7379c0e73b15958b`.

The earlier release completed both 729-object runs (86 moving submissions each; frame-interval p50/p95/p99 33.330/34.536/43.730 ms and 33.336/34.052/49.485 ms). The newer release timed out on both 729 runs after 90 seconds; the first logged repeated viewport allocation refusals and the second did not produce usable frame records before termination. The timed-out run's frame timings are not a completed comparison. The 125-object runs completed in both versions, but their short single-run tail values varied and are insufficient to identify a performance effect.

The timed-out trace shows the actual cause of the refusal: a known live/in-flight overlay inventory of 372,727,840 logical bytes exceeded the 268,435,456-byte *unverified* Vulkan fallback. The Vulkan heap query had found an OS budget but did not establish dedicated capacity. The result was an unavailable viewport target, not evidence that the physical RTX 3060 lacked memory. Raw local artifacts: `build/standalone-validation/ui-artifacts/gpu-inventory-paired-20261104-before1`, `build/standalone-validation/ui-artifacts/gpu-inventory-paired-20261104-after1`, `build/standalone-validation/ui-artifacts/gpu-inventory-paired-20261104-before2`, and `build/standalone-validation/ui-artifacts/gpu-inventory-paired-20261104-after2`. The earlier pilot used the original preferences path and is not part of the paired comparison. Its artifact directory was deleted during cleanup because the cleanup script did not recognize Markdown backtick-delimited paths; that parser was corrected and tested afterwards. The paired and corrected-release artifacts listed here were verified to remain present.

At the sampled frame the Looking Down viewport owned 372,727,712 logical overlay-buffer bytes in 51 live buffers; the additional 128 bytes were in-flight uniforms. These are logical wgpu sizes, not measured physical VRAM or the total peak. Inspection of `build_lines` shows that its fixed batch combines construction grid, point markers, curves and mesh edges; mesh edges are generated per viewport even though their world geometry need not depend on camera. Splitting and sharing eligible world-geometry edges while keeping per-view grid, point scale, selection, preview and ordering correct is a concrete next optimization to investigate. The trace does not establish what fraction of these bytes each category owns, or prove that sharing is already safe for all inputs.

## Exact adapter match and debug verification

The Windows Vulkan path now obtains the Vulkan physical device's valid single-node LUID and resolves that LUID via DXGI, checking both the LUID and vendor/device IDs. It rejects a mismatched wgpu adapter/device and preserves the unverified fallback when identification is unavailable. The policy still limits dedicated GPU budget to 50% and the OS headroom rule; a LUID match is not full GPU accounting or batching.

On this machine the native adapter test found the same Vulkan device and DXGI identity (LUID low 48948/high 0; vendor 4318; device 9476), 12,701,401,088 dedicated bytes and 11,896,094,720 bytes reported local OS budget. The 50% configured ceiling was 6,350,700,544 bytes. A separate debug replay of the 729-object fixture completed cleanly, produced a screenshot, had zero dropped records or viewport errors and preserved the fixture. Its debug timing is not compared to release. Evidence: `build/standalone-validation/ui-artifacts/gpu-luid-debug-729-20261104` and the native Vulkan test output in the execution transcript.

The navigation harness now requests a hidden child window on Windows when `--hidden` is used, uses separate per-run layout/Gumball settings, and accepts relative or external output paths. An initial pilot exposed a relative-path reporting exception after its first child finished; the harness was corrected before the paired measurements. Hidden replay and GPU completion are not evidence of visible presentation, pointer latency, 30 FPS guarantee or desktop acceptance. Target/worker allocations are not fully reserved and complete over-budget batching remains open.

The cleanup invocation removed ten runs in total; its captured summary did not retain their names, so whether other removed runs had backtick-only citations is unknown. The parser now treats Markdown backticks as path delimiters. A temporary-directory regression test verifies that both backtick-delimited and Markdown-link citations protect older runs while an unreferenced old run is removed.

## Corrected release replay

Release rebuilt outside the checkout with `-j 2` in 7m14s; executable SHA-256 `61e199fd84a612dc6c393106cc571c6e80cbbe74344a981eef45f4cec262569b`, navigation/UI/viewport source fingerprint `643a800f607482023342c43f92845c56238837768f303b94d11f6b734c81b788`. It reported the exact-LUID DXGI sample and 6,350,700,544-byte configured ceiling on the same GPU. Both 125-object and both 729-object runs completed with no viewport errors, no dropped records, clean exit, and unchanged fixtures. This corrects the reproduced 729-object starvation on this adapter; it does not solve an over-budget scene on adapters where identity remains unknown.

Application frame-interval percentiles in milliseconds (individual runs, not pooled):

| Fixture | Earlier release p50 / p95 / p99 | Corrected release p50 / p95 / p99 |
| --- | --- | --- |
| 125, run 1 | 33.348 / 34.455 / 42.510 | 33.318 / 47.012 / 50.547 |
| 125, run 2 | 33.353 / 34.466 / 50.445 | 33.302 / 48.595 / 49.650 |
| 125, additional A/B 1 | 33.322 / 44.691 / 49.291 | 33.335 / 35.133 / 49.119 |
| 125, additional A/B 2 | 33.418 / 34.598 / 38.152 | 33.357 / 34.393 / 41.686 |
| 729, run 1 | 33.330 / 34.536 / 43.730 | 33.338 / 34.620 / 42.350 |
| 729, run 2 | 33.336 / 34.052 / 49.485 | 33.328 / 34.705 / 49.734 |

For Perspective moving/settling geometry submissions, using the log's 1 ms timestamp resolution, consecutive submission gaps above 41.7 ms numbered 4/327 before versus 10/330 after for 125 objects, and 3/167 before versus 2/154 after for 729 objects. Maximum gaps were 82 ms after for 125 and 1,052 ms before versus 149 ms after for 729; these few events are not a causal comparison or an exact count of late displayed frames. For 729, logged receive-to-submit medians were 1.773–2.035 ms before and 1.879–2.140 ms after; p95 ranged 2.997–18.550 ms before and 2.801–18.613 ms after. GPU timestamp p95 varied 16.329–17.735 ms before and 7.983–20.732 ms after. No comparable whole-thread CPU time or measured process/GPU memory peak was captured.

The 125 p95 values vary substantially across the extra interleaved runs, so these short samples do not establish a speedup or regression. The earlier and corrected binaries also contain more source differences than the LUID fix. Diagnostics were enabled in both; the result does not isolate tracking overhead with logging off. The release runs were grouped after the earlier paired runs, with additional 125 A/B runs interleaved afterwards, so they are not all adjacent pairs. The window was requested hidden and DPI simulated; no Windows display-time trace, input-to-photon metric or human desktop assessment was obtained. WPR/presentation, real user models and complete budget/batching gates remain open.

Raw retained artifacts: `build/standalone-validation/ui-artifacts/gpu-luid-release-paired-20261104`, `build/standalone-validation/ui-artifacts/gpu-inventory-extra-20261104-before3`, `build/standalone-validation/ui-artifacts/gpu-inventory-extra-20261104-fixed3`, `build/standalone-validation/ui-artifacts/gpu-inventory-extra-20261104-before4`, and `build/standalone-validation/ui-artifacts/gpu-inventory-extra-20261104-fixed4`.
