# Icon sizes across screen sizes

Historical measurements before the live resize fix. For current sizing and an
actual Windows maximize/restore check, see
[live resize evidence](2026-10-20-fullscreen-icon-resize.md).

Standalone OM9 measurement, 2026-10-20. This is a local layout audit, not a claim
about market share or a physical monitor acceptance test. “2K” means QHD
2560 × 1440 here. No runtime sizing or command behavior changed in this audit.

At the default **360 logical point outer dock**, the following are actual icon
slot measurements from `MatrixApp::show`, with a full client area at 100% scale:

| Client area | Main Menu slot | Icon History slot | Toolbar slot | Display / Snaps / Info slot |
| --- | ---: | ---: | ---: | ---: |
| 1280 × 720, nominal minimum | 28 px | 28 px | 23 px | 27.59 px, approximately 28 |
| 1366 × 768 | 28 px | 28 px | 23 px | 27.59 px, approximately 28 |
| 1920 × 1080, Full HD | 31 px | 40 px | 32 px | 27.59 px, approximately 28 |
| 2560 × 1440, QHD | 31 px | 40 px | 32 px | 27.59 px, approximately 28 |

The drawing area inside each slot is 3 logical points smaller than its slot:
25 / 25 / 20 / 24.59 points on the smaller frames; 28 / 37 / 29 / 24.59 on the
larger frames, in the same column order as the table. This distinguishes the
visible glyph square from its hover/click box.

These are reference client sizes, not guaranteed values for a maximized desktop
at the same resolution. Title bars, taskbar and a shorter restored window reduce
the client area; icons respond continuously to that available logical area.
1280 × 720 is the nominal minimum client contract. On a smaller usable area,
startup caps that minimum to the monitor; this does not make 800 × 400 the
recommended desktop resolution.

## Scale and dock width

Resolution alone does not determine size. The generic dock icon size follows
both dock width and logical window area. At 360 points the dock anchor is 33 points;
the screen multiplier is `min(width / 1280, height / 900)`, bounded to 0.85–1.2.
The generic slot is rounded and bounded to 23–42 points. Toolbar slots use
the same multiplier with a 27.5 point anchor and a 23–32 point bound.

Main Menu then applies an additional fit limit for ten icons per row, two rows
per page, and the page selector on the right. Its actual slot at dock width 360
is 28 points on the small frames and 31 points on the large frames. The generic
28–40 point dock metric is therefore **not** the actual Main Menu slot metric.
All 22 pages across the twelve categories were checked at every matrix case.

The three fixed palettes preserve the requested 11 / 16 / 22 icon counts,
row order and group gaps. Their slot size follows dock content width, capped
at the 360 point default. Increasing resolution alone does not change their
number of columns. On the tested default dock they measure about 27.5–27.6
points, with small pixel-grid rounding differences at 125%.

For example, at **Full HD / 125%**, measured slot sizes are Main Menu 31 points
(38.75 device px), toolbar 26 points (32.5 device px), fixed palettes 27.53 points
(34.41 device px). At **QHD / 150%**, they are 31 points (46.5 device px),
29 points (43.5 device px), and 27.59 points (41.39 device px). A saved custom
dock width changes these values. Keep Windows scaling enabled; forcing larger
icons just from physical resolution would double-count part of that scaling.

## Verification and limits

- One new Rust regression test renders 4 resolutions × 3 scales (100/125/150%).
  It sweeps all 22 pages of 12 Main Menu categories and 20 toolbar tabs: 504 selections.
  It checks sidebar slot bounds, square shape, minimum size, no overlap, fixed
  palette counts/rows, toolbar height, four views and unchanged empty document.
- The two existing fixed-palette tests and two SVG tests also passed: five Rust
  tests in this batch. Existing authored icons rasterize at 64 × 64 with linear
  sampling, larger than the painted icon squares in this tested matrix.
- All measurements in this batch are headless egui frames. Native screenshots
  from the preceding fixed-sidebar work predate a concurrent Main Menu paging
  change, so they are not claimed as current full-app acceptance evidence.
- Real per-monitor Windows DPI, a physical QHD monitor and manual M01-UI approval
  remain unverified. No claim of new native captures is made.
- The full public-source audit still reports 11,740 existing tree findings.
  No icon artwork, raster assets, fonts or private reference content were added.

Machine-readable measurements and fingerprints:
[evidence](../../implementation/evidence/M01/2026-10-20-icon-screen-sizes.json).
