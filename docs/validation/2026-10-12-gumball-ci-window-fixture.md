# Native CI window fixture — 2026-10-12

The actual [Action38191927264](https://github.com/MYuitsu/OpenMatrix9/actions/runs/38191927264)
compiled the plugin and passed Rust, native host and the first seven Gumball cases.
The first core pixel check then failed. Its live viewport was 359×242, while the
fixed native FBO image was 1024×768. Local testing used a 747×378 viewport.
The stock host renders the same scene geometry into the requested FBO size;
Gumball's world geometry is sized from the live viewport's logical height.
Consequently the exported widget was larger on the smaller hosted desktop.
The actual live framebuffer colors were comparable: CI [552,581,588], local
[568,615,607]. The FBO counts were CI [2403,2534,2522], local [1341,1447,1451].

A local owned window at 1024×726 reproduced the same compact-widget assertion
with [2409,2546,2537]. Setting the owned test window to the established 1800×1000
logical client size restored [1341,1447,1451] and all 68 core checks passed.
An ordinary resize was clamped by Windows at DPI200 (1800×951 instead of
1800×1000), which the new geometry guard rejected. The owned fixture now uses
Qt's fixed-size constraint before exposure so the native minimum/maximum client
dimensions retain the requested size even on a smaller physical desktop.
The native runner now records and requires the achieved size and exposed window
before running fixtures. QT_SCALE_FACTOR remains independent and the actual
device-pixel ratio is recorded. Production widget sizing, capture resolution,
the existing 2500-pixel bound and all geometry/gesture assertions are unchanged.
Main-window setup requests in four macros are constrained to this fixture size;
their MDI viewport maximize/restore and view-lifetime checks remain active.
Only the isolated test process is resized; the user's running host is untouched.

At DPI150 the core capture also exposed an existing fragile viewport lookup:
Qt's active MDI subwindow was absent while the native source slot and four
Gumball scenes were intact. The core macro now uses the same native source-slot
and document lookup already used by the shared GumballUI fixture and retains
its PySide ancestor chain. This changes only widget lookup; all pointer, editor,
geometry, transaction and pixel assertions remain active.

The complete stock native cohort then passed all 30 processes and 955 checks,
including DPI100/150/200, parameter Scale, expressions, associative dependencies,
Object Alignment, Undo/Redo and warm/cold FCStd reopening. Every process records
the expected logical size, actual DPI, one host hash and one plugin hash. The
private exact-head artifact inspector also rejects missing geometry evidence,
incorrect size, an unexposed window or incorrect DPI; all 15 inspector guards pass.
No native source changed and the existing plugin was reused. Version counters and
the completion-task timestamp are unchanged by this CI fixture repair.

This is local evidence. The exact commit's
[GitHub Action](https://github.com/MYuitsu/OpenMatrix9/actions/workflows/release-windows.yml)
and its packaged artifacts determine the final stock CI result.

```json
{
  "local_stock_native_pass": true,
  "native_processes": 30,
  "native_checks": 955,
  "logical_window_client": [
    1800,
    1000
  ],
  "dpi_cases": [
    "1",
    "1.5",
    "2"
  ],
  "suite_sha256": "eb8e76a4698e17e7ffccf895d6f510bc761af48bf6a3fba3b32858c054b679f1",
  "plugin_sha256": "71f8431d71f6abfb36c45e9fd2a128f1d8f232368e2e5c1faf5dd3a6ff58b987",
  "host_sha256": "7b38c5d5aa2c74f830539c1c5c44cbf6e3e8e157292c718f9ee24ac6d3e12cf6",
  "actual_failed_action": "https://github.com/MYuitsu/OpenMatrix9/actions/runs/38191927264",
  "actual_failed_action_fbo_rgb_pixels": [
    2403,
    2534,
    2522
  ],
  "red_logical_window": [
    1024,
    726
  ],
  "red_fbo_rgb_pixels": [
    2409,
    2546,
    2537
  ],
  "green_fbo_rgb_pixels": [
    1341,
    1447,
    1451
  ],
  "red_report_sha256": "8034d6b267909dddf228a155231e1ba125cae4e54360d5910efa1b4fe2f6158a",
  "green_report_sha256": "b04df2fa611d4a8c4552d50c1d40da730d0d6480e7944d1a5b1c37999ee6b4bb",
  "native_runner_sha256": "d3b3834b06212284244bccfbb25fc31233ab503457792e2449a4a9f3855c1fde",
  "core_fixture_sha256": "6545639fa12e8121ed29ff371701625754453c9d309d04814dc7f6b39db6d81b",
  "stock_ci_authority": "Exact-commit GitHub Actions status and artifacts; this checkpoint records local evidence only.",
  "version": "0.3.4-202610120606"
}
```
