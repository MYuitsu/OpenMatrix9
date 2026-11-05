# OM9 add-ons

OM9 installs commands from zip packages, the way Blender installs extensions. Commands that no
add-on provides stay visible and disabled. A command an enabled add-on provides becomes
available from the same menu entry, sidebar icon, F6 search and typed CMD name (`gvGemOnCrv`).
All three resolve to one catalog code (`BuilderGemOnCrv`).

## Using add-ons

- **Tools › Add-ons…** lists installed add-ons. From there you can enable or disable an add-on,
  grant the permissions it asks for, start its commands and uninstall it. Parameters are not
  edited there: they are set while the command runs (see *Builders* below).
- **Tools › Install Add-on from Disk…**, or drag a `.zip` onto the window, installs a package.
  Install is atomic: a failed install leaves the previous version in place.
- Add-on output goes into one transaction, so one Undo removes everything a command created.
- Gems are tagged with the `om9.gem` slot, so *Super Select: Gems* finds them.
- `OM9_SAFE_MODE=1` starts OM9 without enabling any add-on.
- `OM9_CONFIG_DIR` overrides the per-version config folder. That folder holds `userprefs.toml`
  and `extensions/user/`. An `addons/` folder beside `om9.exe` is a read-only system repository.

## Builders: parameters while the command runs

A command declared with `builder = true` does not create anything when it starts. It opens a
builder instead:

- The **command line** shows the prompt and the options as tokens, for example
  `Size=1.5  Spacing=0.2  Start=0  End=100  ZOffset=0  Layout=Dàn đều  Orient=Theo CPlane`.
  - Type `Size=2` to change a value. Option names match case-insensitively or by a unique
    prefix (`spa=0,25`).
  - A choice or flag steps to its next value when you type its bare name (`Layout`) or click it.
  - Clicking a number token puts `Size=` in the line.
- The **BUILDERS** panel below the Project Manager shows the command thumbnail (its menu icon
  and a sketch of the current diameter and gap), the live result, Accept / Cancel / Defaults,
  every parameter as a draggable or typable field, and one slider for the selected parameter.
- The **viewports** show the result as a cyan outline for the selected curves. The outline is
  not a document object. You can pick or change the curve while the builder is open.
- **Enter** or **✔ Đặt** creates the result in one transaction (one Undo). The accepted values
  become the add-on's defaults for next time.
- **Esc** or **✖ Huỷ** closes the builder; nothing is created or stored. Starting another
  command also closes it.

Manifest keys used by builders:

- `[[commands]] builder = true`.
- `[[settings]] option = "Size"`: the command-line name. It defaults to the key in PascalCase.
- `[[settings]] choice_labels = [...]`: display text for enum choices.

Commands without `builder = true` (for example Gem Count On Crv) run at once with the
remembered values.

## Interactive commands: clicks, sub-options and preview

Commands marked `interactive = true` in the manifest run step by step, like the built-in Line.
Polyline, Circle, Move and Gem Cutter work this way.

- **Starting:** the command does not create anything when it starts. It shows a prompt and its
  sub-options in the command line, for example `Start of polyline  PersistentClose=No  Length`.
- **Sub-options:** you can click an option, or type its name, `Name=value`, or a bare number.
- **Points:** you can click in the viewport or type `x,y[,z]`, `@dx,dy` (relative) or a number
  (a distance along the last direction).
- **Finishing:** Enter or a right-click finishes the command. Esc cancels it. U removes the last point.
- **Preview:** while you move the mouse the add-on draws a preview line, plus a rubber band from
  the last point. The preview is only drawn on screen and is never added to the document.
- **Undo:** the result is one transaction, so a single Undo removes it.

A command can also set `builder = true`. It then also opens the BUILDERS panel (Gem Cutter does
this). Values typed in the command line and values set in the panel are the same settings, and
the values you accept become the defaults for next time.

The session protocol is JSON over the optional `om9_addon_session` symbol. The add-on keeps no
state; the host stores it and passes it back with each step.

- Each event carries the command code, the input (`start`, `point`, `hover`, `text`, `option`,
  `enter`, `undo` or `selection`), the previous state, the CPlane, the current selection
  (meshes, curves and gem records) and the settings.
- Each reply carries:
  - the new state;
  - the prompt and option tokens;
  - what it wants next (a point, objects, a number or nothing);
  - preview geometry and a rubber band;
  - when the command is done, the geometry to create (point, line, polyline, circle, arc,
    NURBS or mesh, with an optional layer) or 4×4 transforms for the selection, with a flag to
    keep the originals.
- All values are in millimetres.
- Rust add-ons implement `session::Interactive` and call `export_interactive!` (from
  `om9-addon-abi`, feature `session`).
- The host validates every reply before it changes the document.

An add-on command is called the same way as a built-in command:

- by its catalog code or a manifest `aliases` entry in the command line (`Circle`,
  `gvAzureCutter`);
- from F6;
- from its OM9 icon in the menu palettes and toolbars. An icon whose catalog code an enabled
  add-on provides becomes active.

When OM9 itself implements a command (for example its own Polyline, Circle or the Top 11
buttons), the built-in command keeps the typed name and the icon. The add-on version still
runs by its catalog code. `docs/matrix9-commands.md` shows, for every menu command, whether OM9
or an add-on runs it today.

## Package layout

```
om9_addon.toml      manifest (schema 1.0.0): id, version, type, license, om9_version_min,
                    platforms, runtime (native | data | python | wasm), entry, permissions,
                    [[commands]] (catalog code, label, icon, shortcut), [[settings]], [[assets]]
<entry>             e.g. gem_on_curve.dll for runtime = "native"
icons/…             authored SVG
```

Native add-ons export a small C ABI (`crates/om9-addon-abi`). The ABI exports these symbols:

- `om9_addon_abi_version`
- `om9_addon_command_count`
- `om9_addon_command_name`
- `om9_addon_invoke`

The host passes the selected curves, sampled in millimetres, together with the CPlane normal and
the add-on's current settings. The add-on returns meshes and messages through host callbacks.
Rust add-ons implement `sdk::Addon` and call `export_addon!`, which also catches panics at the
boundary. The host checks every returned mesh before it touches the document.

## Building packages

- `cargo run -p om9-addons --features native --bin om9-addon -- --catalog Resources/menu/MainMenu.ini build <dir> -o <pkg>.zip`
  packages a directory deterministically. `validate`, `install`, `enable`, `list`, `menu` and
  `settings` cover the rest of the lifecycle.
- On Windows CI, `tools/ci/build-addons-windows.ps1` builds **one** native package, the OM9
  Matrix suite (`om9-addon-matrix` → `om9_matrix.dll`), and packages it as
  `om9_matrix-<ver>-windows-x64.zip` with its help pages and icons, plus
  `ring_sizes-<ver>.zip`. It then smoke-installs and enables it (which really loads the DLL),
  lists the Gems, Clayoo, Surface and Tools menus, and uploads the zips as the `om9-addons`
  artifact. Built libraries are never committed.
- The package files are generated: `OM9_UPDATE_PACKAGE=1 cargo test -p om9-addon-matrix --test package`
  writes `addons/matrix/addon/` (manifest, README, `commands/<Menu>/<Code>.md`) from the menu
  crates' manifests and checks that names and setting keys are unique and that every command
  is routed to its menu. Build `om9-addon-matrix` on its own (`cargo build -p om9-addon-matrix`):
  linked into one build with the older stand-alone add-ons, their exports would collide.

## The OM9 Matrix package

One zip with every Matrix-style command of the add-ons: **337 commands filling all 342 slots of 11 menus** (Top 11,
Curve, Surface, Solid, Transform, SubD, Builder, Tools, Gems, Settings, Cutters). While it is
enabled, its commands take the menu icons, the typed names and F6; disabling it gives the
icons back to OM9's own commands. Each menu is a crate under `addons/`; `addons/matrix` links
them into one library and routes each command to its menu.

| Menu | Crates | Notes |
|---|---|---|
| Top 11 | `transform`, `top11` | Move, Duplicate, Rotate, Mirror; edit / control points (pick, place, repeat), explode, join, split, trim, ring rail. |
| Curve | `curves`, `curve_tools` | Drawing commands; rebuild, refit, blends, offsets, fillets, chamfers, projections, intersections, sections, curve boolean, tween, sketches, curves on surfaces, soft edit … |
| Surface | `surfaces` | NURBS results: sweeps (1/2/multi rail, profile, four profile), loft, network, patch, edge curves, exact revolve, blends, fillets and chamfers (variable too), offsets, merge, match, drape, heightfield, unroll, soft edit, rebuild, refit … |
| Solid | `solid` | Primitives, booleans (mesh CSG), extrude, pipe, holes, shell, fillet edge, text … |
| Transform | `transform`, `transform_tools` | Scale, orient, shear, bend, taper, twist, flows, arrays, cages … |
| SubD | `subd` | Polygon cages with Catmull–Clark smoothing (creases stay sharp). OM9 selects whole objects, so the add-on keeps a face selection per object (Select Loop / Ring / UV / Paint / Grow / Shrink / Invert / Naked / Creased / Coplanar / Sets) that Extrude, Inset, Item, Collapse, Extract … use. |
| Builder | `builders` | Eternity band, signet, raised and SubD rings, award top, text on curve, rope, jump ring, knots, pattern, nautilus, mill work. Rings: finger rail in the Front view, axis Y. |
| Tools | `tools` | Ring rail, profiles, custom rails, smart targets, metal weights, ring resizer, findings libraries (our own parametric parts), image trace, object checker, mesh repair / reducer, 3D-print sprues … |
| Gems | `gems`, `gem_on_curve`, `gem_tools` | Loader, cuts, halo, reporter; layouts on curves and surfaces, pavé (auto, builder, sphere, prongs, azure), clusters, tapered baguettes, springs, follow, control, positioner, flow, splop. Stones remember the settings and cutters built on them (`styles` in the gem record): Match Attributes and Save / Load Style (`.om9style` files) rebuild them. |
| Settings | `settings`, `setting_tools` | Prongs, bezel, head, beads, channel border; SubD bezel, pull to rail, prong editor, prongs and beads on surfaces, metal from gems. |
| Cutters | `gem_cutter`, `cutters`, `setting_tools` | Gem cutter and its library, bezel / channel / bright-cut / micro-prong cutters, plane and cube cutters, cut to finger rail, quad flip, Boolean Builder. |

Limits worth knowing: B-rep booleans and fillets work on meshes; surface fillets and chamfers
do not trim the original surfaces; history-based Matrix behaviour (editing a result later by its
inputs) is not reproduced; vendor libraries (Stuller, Bermark) are replaced by parametric parts.

## Shipped examples

| Package | Commands | Notes |
|---|---|---|
| `addons/gem_on_curve` | Gem On Crv, Gem Count On Crv | Round brilliant stones spaced along curves.<br>Closed curve: `n = ⌊L/(d+g)⌋`, `g' = L/n − d`.<br>Open curve: `n = ⌊(L+g)/(d+g)⌋`.<br>Gem On Crv opens a builder; Gem Count On Crv reports at once.<br>Options: Size, Spacing, Start/End %, ZOffset, Layout (fit or fixed gap), Orient (CPlane or outward). |
| `addons/curves` | Polyline, Line, Interpolate/Control Point Curve, Rectangle, Circle, Ellipse, Arc (center / start-end-direction), Polygon, Spiral, Helix, Point | Interactive. Options include Close, PersistentClose, Length, Degree, Center/3Point/Rounded, 2Point/3Point/Diameter, NumSides/Circumscribed and Turns. |
| `addons/transform` | Move, Duplicate, Rotate, Mirror, Rotate 3D, Scale 3D/2D/1D, Array Polar/Linear/Rectangular | Works on the selection, or asks you to select. Shows a live preview of the moved objects. Copy option. Gem records follow the transform. |
| `addons/gem_cutter` | Gem Cutter | Builder and interactive. Makes a closed cutter mesh for each selected gem on the layer *Cutting*. Values in % of the stone diameter: Girdle Scale, Girdle Width, Top Height, Seat Size, Lower Depth, Lower Taper, Z Offset. |
| `addons/geom` | — | Shared curve maths (NURBS evaluation, interpolation, conics, arcs, spirals) linked into the add-ons above; not a package. |
| `addons/ring_sizes` | — | Data library: US and ISO 8653 ring size tables. |

Check value: a Ø17.35 mm ring rail with Ø1.5 mm stones and 0.2 mm spacing gives 32 stones with a
real gap of 0.203 mm. Stones of Ø2.0 mm give 24.

Tests:

- `crates/om9-addons/tests`
- `addons/gem_on_curve/tests/native_load.rs` (real library load)
- `crates/om9-app/tests/addon_command.rs` (controller, CMD/menu paths, single Undo)
- `crates/om9-app/tests/addon_session.rs`: interactive curves, transforms and Gem Cutter through
  the controller. It also checks that every add-on command has an OM9 icon that calls it.
- `addons/{curves,transform}/tests/native_load.rs` (real library, session symbol)
- `crates/om9-core/tests/insert_with_extension.rs`
