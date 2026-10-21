# Slim, reserved sidebar scrollbar

The user reported buttons hidden by the left sidebar scrollbar. The default
floating bar expanded over the content and appeared only when needed.

The left sidebar now uses a solid **4 logical point** scrollbar with a **2 point
inner gap**, always visible, including when all content fits. It reserves six
points from content width; it never expands over buttons on hover. At the 360
point outer dock, content viewport width is 349 points (355 inner minus 6).
The 240 point dock has 229 points for content. Existing menus adapt to that
viewport, retaining reference icon counts, rows and right-side page controls.

The previous code also read a scroll offset from an unwritten temporary key,
resetting wheel/drag results on subsequent frames. It now retains the actual
`ScrollAreaOutput.state.offset.y`. This repairs scrolling without changing
commands, saved layouts, geometry or the 360 point dock default. The change is
scoped to the outer left sidebar; other scroll areas keep their styles.

Verification:

- Three new Rust tests pass: actual painted bar bounds, no overlap/overflow at
  dock widths 240/360/460 and simulated DPI 100/125/150 (nine combinations), no
  hover expansion, always-visible painting without overflow, and wheel offset
  retained over idle frames.
- Two icon matrix/resize tests, four chrome tests and two fixed-palette tests
  pass, for **11 Rust tests**. The 12-case icon matrix retains all 22 Main Menu
  pages and 20 toolbar tabs; resize/restore remains reversible.
- Native GPU maximize/restore passes at 1280×720 → 2560×1369 → 1280×720.
  Receipts show a separate 4 point bar with a 2 point gap at each size.
- A separate native 240 point dock run passes the layout/CMD oracle. Its GPU
  screenshot was inspected: the far-right page buttons, Grid Snap and layer
  controls are left of the reserved scrollbar gutter.
- The whole-tree public-source audit still reports 11,740 pre-existing findings.
  No artwork, fonts or private reference content were added.

The new gutter slightly changes fitted icon sizes: at default dock and 100%,
Main Menu is 27 points at 1280×720 and 30 at 2560×1369; fixed palettes are
24.41→27.13 points. These supersede the earlier no-gutter size measurements.
Formal M01-UI approval and physical DPI transitions remain pending.

[Evidence](../../implementation/evidence/M01/2026-10-20-slim-sidebar-scrollbar.json).
