# M01 Viewport Grid Antialiasing (2026-10-19)

Scope: standalone Rust OM9, M01-L2-01-06 (viewport GPU draw backend), L3-006 (depth clipping and
grid overlays). User report on 2026-10-19, with a 1158x596 Perspective screenshot: "lưới này bị răng
cưa nhiều quá" (the grid is too jagged).

## Cause

The construction grid, the curves and the point markers are drawn as a wgpu `LineList`, so every line
is exactly one device pixel wide. The viewport rendered those lines into a single-sample
`Rgba8Unorm` target with no multisampling: a diagonal segment filled whole pixels and produced the
visible staircase. The same target also held the mesh silhouettes and the selection highlight, so the
quality issue was not limited to the grid.

## Implemented

- `crates/om9-render/src/scene.rs`: `SceneBatch::with_samples(device, queue, samples)` builds the face
  and overlay pipelines with that `MultisampleState::count`. `render_into_resolved` and
  `render_over_into_resolved` take a resolve view and set it as the pass `resolve_target`, so a
  multisampled frame lands in an ordinary texture. `new`, `render_into` and `render_over_into` keep one
  sample and their existing signatures, so every current caller and test is unchanged.
- `crates/om9-render/src/overlay.rs`: the same split for `OverlayBatch` (`with_samples`,
  `new`, `render`, `render_resolved`). The grid, curve and point pass now matches the multisampled
  colour+depth pair.
- `crates/om9-app/src/gpu_viewport.rs`: the document viewport creates a 4x MSAA `Rgba8Unorm` +
  `Depth32Float` pair once per target size (`SAMPLES = 4`), keeps the single-sample colour texture as
  the resolve target registered with egui, and draws the scene, drag ghost, highlight and overlay
  passes into the multisampled pair. The stress scene (`OM9_GPU_STRESS`) and the cluster path
  (`OM9_GPU_CLUSTER`) keep their one-sample targets and pipelines.

No grid geometry, extent, colour, axis colour, picking or document behaviour changed. The grid is
still the finite world-space construction plane (-20..+20 mm, minor 1 mm, major 5 mm, cyan/red axes),
unpickable and excluded from model-fit bounds.

## Measured

| Check | Result |
| --- | --- |
| `cargo test -p om9-render --test antialiasing` | 1 passed. A 45-degree one-pixel line at 256x256 resolves to **412 partial-coverage pixels with 4 samples and 0 with 1 sample**. |
| `cargo test -p om9-render` (lib) | 27 passed |
| `cargo test -p om9-render --test render_loop` | 11 passed |
| `cargo test -p om9-render --test scene_bench` | 15 passed, 4 ignored measurements |
| `cargo test -p om9-app --lib` | 98 passed |
| `cargo test -p om9-app --test viewport_title_menu` | 17 passed |
| `cargo test -p om9-app --test workbench_regions` | 8 passed |
| `cargo test -p om9-app --test shell_docking` | 10 passed |
| `cargo clippy -p om9-render -p om9-app --all-targets -- -D warnings` | exit 0 |
| `cargo build -p om9-app --release -j 2` | exit 0, `L:\om9-target\release\om9.exe` 18,904,576 bytes, 2026-10-19 09:48 |
| `om9.exe --ui-audit build\aa-audit-20261019` (debug) | exit 0, `workspace.png` SHA-256 `4FAD76F2…98F73077` |
| `om9.exe --ui-audit build\aa-release-audit-20261019` (release) | exit 0, same `workspace.png` SHA-256 |

Visual evidence:

- `build/aa-preview-20261019/grid-1x.png` and `grid-4x.png`: offscreen render of the exact document
  grid with the user's camera (perspective eye [35,-35,35], target origin, up +Z, fov 35 degrees) at
  1158x596. `crop-1x.png` and `crop-4x.png` are the same 320x160 region at 3x nearest-neighbour.
- `build/aa-audit-20261019/workspace.png` (debug) and `build/aa-release-audit-20261019/workspace.png`
  (release) are the real 1600x1000 four-view workspace audits; `perspective-zoom.png` is a 2x crop.
- `build/aa-direct-pre-20261019/workspace.png` is the pre-change `om9-pre-aa.exe` audit and
  `build/aa-direct-release-20261019/workspace.png` the new `om9.exe` audit, both launched with PATH
  reduced to `C:\Windows\System32;C:\Windows`. The same layout and camera make the grid comparison
  direct; `build/aa-script-run-20261019` is the same run through `tools/run-om9.cmd`.

## Direct launch and the OCCT runtime

`om9.exe` imports the OCCT DLLs statically, so the Windows loader needs `TKernel.dll` and the rest
before `main` runs. The 2026-10-19 09:26 session had been started by a shell/harness that already put
`build\standalone-deps\occt-8.0.1-sdk\bin`, the TBB `bin` and the jemalloc `bin` on PATH; a plain
double-click does not, which is why running `om9-pre-aa.exe` from Explorer failed with "TKernel.dll was
not found".

Two launch paths were added so this is not a trap:

- The 81 runtime DLLs (63.6 MB: 74 OCCT, 6 TBB, jemalloc) were copied beside both executables in
  `L:\om9-target\release\`, so Explorer/double-click of `om9.exe` or `om9-pre-aa.exe` now loads.
  Verified with `PATH=C:\Windows\System32;C:\Windows`: both binaries ran `--ui-audit` with exit 0 and
  wrote their `workspace.png`.
- `tools/run-om9.cmd` starts the release build with the three SDK `bin` directories prepended to PATH,
  which survives a rebuilt or cleaned target directory. Verified the same way (exit 0, audit captured).

The DLL copy is a local build-output deployment step, not source; a rebuilt target directory needs it
again (or use the script). The app does not hard-code the SDK directory.

The 1x preview shows the report exactly: broken, dashed-looking minor lines and a hard staircase on
every diagonal. The 4x preview keeps the same line colours and extents but resolves partial pixel
coverage to grey levels; the lines are continuous and the staircase is gone.

## Not done / limitations

- Only the document viewport is multisampled. The stress and cluster paths stay at one sample by
  design; they are measurement paths, not the document view.
- Four samples give four coverage levels for a sub-pixel line. A screen-space quad line with analytic
  coverage would give continuous coverage and a controllable width, but it would replace the GPU line
  primitive and its depth behaviour; it was not required for this report and is not implemented.
- The release executable could not be overwritten while the user's instance was running. Windows
  allows renaming a running executable, so the old binary was renamed to
  `L:\om9-target\release\om9-pre-aa.exe` and a new `om9.exe` was linked. The new `om9.exe` was
  started at 09:51; `om9-pre-aa.exe` is the pre-change build and can be deleted once it is not needed
  for comparison.
- Running an executable from Explorer without the OCCT runtime on PATH fails before the application
  starts. The DLL copy and `tools/run-om9.cmd` above cover both the double-click and source-tree
  launch; a packaged release still has to ship the runtime DLLs itself.
- Unrelated failures in the current working tree, reproduced with test binaries built from files this
  change does not touch: `om9-app/tests/ui_events.rs::construction_grid_colors_only_positive_axes_like_the_reference`
  and `om9-app/tests/mesh_fill.rs::a_mesh_object_fills_its_faces_after_the_grid` still expect the CPU
  egui scene painter that the GPU-only migration removed; `om9-app/tests/file_lifecycle.rs::open_candidate_must_fit_alongside_old_world_scene_and_undo`
  fails its "old alone must fit" budget check in the current tree while the 2026-10-19 03:35 test
  binary passes. None of the three exercises the changed files.
- The workspace had concurrent edits by another worker during this work (including `ui.rs` at
  09:44 and formatting changes in the touched test file); the captured release binary therefore also
  contains that state of the tree.
