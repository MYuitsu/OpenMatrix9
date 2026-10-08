# RGB +5 workspace menu — 2026-10-04

Historical artwork-switch snapshot. Subsequent functional workspace adds seven resource mappings and native actions; see `2026-10-04-workspace.md` and `workspace-assets.json` for current evidence. The counts below describe the earlier commit.

The latest user instruction is to rebuild the workspace menu using the Matrix90 images already recolored by adding5 to each RGB channel, clamped255. This changes product asset priority; it does not change Rust catalog/state, Qt layout or command implementations.

`tools/export_menu_assets.py --shifted-root ref/matrix9/Matrix90/menu-images` imports the verified derivative manifest. Exact ButtonIcons `_1` and SliderIcons names, plus explicit semantic aliases, take priority over redraws and authored SVGs. The PNGs are copied byte-for-byte without another color operation. The importer checks transform metadata, safe unique keys, contained PNG paths, CRC, dimensions, current hash and original image fingerprint. Provenance is recorded in the installed manifest. No archive code or Rhino scripts execute.

All434 MainMenu names use shifted PNGs, with zero missing/category fallback. Of503 resource mappings,492 use491 derivative PNGs (one shared alias);58 are auxiliary to the main menu. The remaining11 Layer/Project chrome keys use existing authored symbols. The user's redraw folder remains unchanged. Source BMPs/PNGs/ZIP are local reference files; only selected derivative resources and tooling/evidence are committed.

The installed workbench uses relative Resources paths. CMake copied all selected assets into the matching existing FreeCAD SDK.214 obsolete installed files, declared by the previous INI, were removed after resolved-path checks. Source/installed current image bytes and INI match; no reference folder or installed Matrix90 is required at runtime.

Validation:

- Two importer tests observed RED for unsupported `shifted_root` before implementation, then GREEN. Python24 tests pass, including precedence, byte-preserving import and invalid delta/hash/path rejection; existing modern/reference tests remain green.
- MSVC x64 SDK module build/link passed; Rust release core rebuilt without source/ABI changes. Existing Rust7 semantic/fmt/Clippy checks remain the unchanged baseline.
- QtTest10 passed, including icon pixels, disabled full-color/click rejection, counts/geometry and restore/history/lifecycle. Offscreen warnings are retained in the log.
- Actual FreeCAD31 checks passed at scale1/1.25/1.5/2 with isolated profiles. Every MainMenu/quick icon loads the RGB +5 image and renders its expected scaled pixels, including disabled colors.492 image hashes match provenance. New/Open/Save/cancel/history/switch/restore and Projects reachability at1024×600 pass. Reports are in `menu-runtime.json`.
- Native dock/full-content/panel screenshots were visually inspected and copied to `docs/images`. These are real Qt widget grabs; full-content capture includes scrollable content. Four viewports/OpenGL rendering are not verified by these captures.
- Fresh read-only review checked all503 resource entries,492 selected image bytes and provenance. No unresolved findings. Structural layout is the prior verified slice; this request replaces artwork rather than implementing modeling features.

Unsupported commands and supplementary panels remain disabled. Complete resource coverage is not a claim that Matrix9 modeling behavior has been ported. Four viewports, green Command area and feature implementation remain later work. Win32 dialogs and failed Save As to an unwritable file retain their previously documented interactive test gaps.
