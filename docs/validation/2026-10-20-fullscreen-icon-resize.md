# Live maximize and restore icon resizing

The subsequent [reserved slim scrollbar fix](2026-10-20-slim-sidebar-scrollbar.md)
uses six points of dock width, slightly reducing the fitted icon sizes below.
Its native maximize/restore check confirms that live resizing still works.

The preceding icon audit used fresh contexts per screen size. It did not prove
a live maximize/restore transition, and the three fixed reference palettes did
not follow screen scale. Main Menu also hit its ten-column width cap early.

This fix applies a bounded 0.9–1.0 window factor to the **actual fitted slot**
and to the three reference palettes, computed every frame from the existing
responsive scale. Main Menu retains ten columns/two rows per page. Display,
Snaps and Info retain 11/16/22 icons, their row order and horizontal group anchors.
The dock default stays 360 logical points; wide docks retain their height cap.
No command, model, startup or persistence business behavior changed.

At 100% scale, measured client-area anchors now are:

| Client area | Main Menu | Toolbar | Display / Snaps / Info |
| --- | ---: | ---: | ---: |
| 1280 × 720 | 28 pt | 23 pt | 24.81 pt |
| 1366 × 768 | 28 pt | 23 pt | 24.84 pt |
| 1280 × 900 | 29 pt | 28 pt | 26 pt |
| 1920 × 1080 | 31 pt | 32 pt | 27.59 pt |
| 2560 × 1440 | 31 pt | 32 pt | 27.59 pt |

These are slots; the drawn glyph square is three points smaller. At high DPI,
device pixels are points multiplied by pixels-per-point. Changes remain bounded
by available dock width; a window already at a size cap cannot grow indefinitely.

## Verification

- A new same-context regression first failed before the fix: Main Menu stayed
  31→31 and all three fixed palettes stayed 27.59→27.59 at 1280×900→1920×1080.
- After the fix, small → restored → large → small transitions pass at simulated
  100/125/150%, preserving slot counts and returning to the original dimensions.
- The full 12-case matrix still passes all 22 pages of twelve categories and
  twenty toolbar tabs per case. Four chrome, two fixed-palette and eight
  left-stack integration tests also pass: **16 Rust tests** in this batch.
- Native Windows title-bar maximize and restore pass on the actual test display,
  at actual reported scale 1.0. Client area 1280×720 → 2560×1369 → 1280×720;
  Main Menu 28→31→28, toolbar 23→32→23, reference palettes 24.81→27.59→24.81.
  Both native GPU screenshots were inspected.
- The first two native launcher attempts selected an untitled helper window;
  they are failed harness attempts, not successful application evidence. The
  launcher now selects only a titled, visible window with a usable client area.
- Cargo's normal integration command hit an existing running `om9.exe` lock.
  The freshly linked integration executable was run directly and passed all
  eight tests. A separate `om9-responsive-icons.exe` was built successfully.
- Full public-source audit still flags 11,740 existing tree findings. No image
  artwork/font/private reference assets were added.

Native capture and trace code is active only in a held `--ui-audit` session;
the launcher uses its own empty document/profile and never closes the user's app.
Borderless fullscreen, physical per-monitor DPI transitions and formal M01-UI
approval remain outside this batch. This verifies the Windows maximize action.

[Machine-readable evidence](../../implementation/evidence/M01/2026-10-20-fullscreen-icon-resize.json).
