# Modern artwork and sidebar validation — 2026-10-04

**Historical snapshot:** the user subsequently requested the RGB +5 Matrix90 derivatives for the workspace. Current artwork policy and screenshots are documented in `2026-10-04-rgb-workspace.md`; the modern-first images and measurements below remain in Git history.

The user explicitly prioritizes their recreated icons in `OpenMatrix9_named_icon_crops`. The four supplied panel images guide the menu, compact rows, gray backgrounds, colored category dots, layer swatches and project groups. They do not authorize switching back to original artwork. This report supersedes the previous original-icon distribution measurements.

## Artwork comparison and coverage

The supplied redraws preserve recognizable functions while using a different visual style: glossy rounded light cards with blue/gold symbols, compared with the original small pixel icons on gray. Native Qt now keeps redraw colors even when a control is disabled. It does not recolor user PNGs, remove their backgrounds or modify the source folder.

`modern-assets.json` records the final package audit: 194 named PNGs byte-match the user's sources; all503 resource names have image paths; MAIN MENU has187 confirmed mappings and247 explicit category fallbacks; auxiliary resources have42 confirmed and27 category fallbacks. One named binding is excluded because its manifest identifies an original fallback. Missing dedicated artwork is supplied by newly authored SVG symbols, never reported as recovered Matrix command-specific artwork. Broad generic snap symbols are marked fallback.

461 legacy assets were removed from the current repo/resources and installed SDK:459 original BMPs, one Matrix atlas and one original fallback PNG. Original source/reference folders are unchanged. Archive decoders remain available for explicit reference export with `--allow-original`; normal distribution is modern only. Existing Git history is retained.

## Native layout comparison

| Reference | Implemented presentation |
| --- | --- |
| MAIN MENU | Three selector rows, white labels, colored category dots, compact command grid and11 quick controls. Display names SubD/Art follow screenshot; canonical Rust IDs remain Clayoo/Emboss. |
| DISPLAY and SNAPS | Six display toggles + five modes + two combos; two snap rows, numeric increments and blue master control. Auxiliary icon buttons23px fit300px dock with scrollbar. |
| INFO and LAYERS | Two rows of11 info icons;32 layer rows in two blocks, labels/swatches/arrows/locks/blue visibility controls. |
| PROJECTS | Left project list, horizontally scrollable three numbered cards with actions, five lower categories Master/Creation/Parts/Render/Output. |

These supplementary panels are presentation controls with unsupported actions disabled. Proven native menu commands remain enabled according to FreeCAD availability. Four viewports and green Command region are outside this slice.

## Verification

- Native module compiled and linked using the matching existing FreeCAD SDK; loaded the actual `OpenMatrix9Gui.pyd`. Rust catalog/state/ABI unchanged.
- Rust7 semantic tests, fmt and Clippy passed; release build passed. Windows incremental hard-link warnings fall back to copying.
- Python22 tests passed, including redraw precedence even with optional original archives, original exclusion, source preservation, corruption/path rejection and managed-output migration.
- QtTest10 passed (8 cases + init/cleanup), including color preservation, disabled click rejection, panel counts, geometry, icon pixels, restore/switch/history behavior. Offscreen font/plugin warnings do not establish native screenshot fidelity.
- Actual FreeCAD passed30 checks at scale1/1.25/1.5/2 with isolated profiles. Every menu/quick icon pixel-matches its installed modern image and disabled colors remain unchanged. Layout counts, no-wrap at startup width, New/Open/Save/cancel/history/switch/recovery and Projects reachability at1024×600 passed. `menu-runtime.json` records each fresh run.
- A native screenshot first revealed row wrapping despite the offscreen geometry case passing. Added native geometry assertion, observed failure, then fixed sizing and verified all scales. Screenshots in `docs/images` are actual widget grabs, including full content and individual panels; full-content capture includes scrollable content outside the current viewport.
- Read-only independent review confirmed all194 PNG source hashes and503 image paths. Two findings received regression failures before fixes: an unmanaged atlas must survive export, and generic snap SVGs must be reported as fallback. No unresolved blocking review findings.

Final installed-resource audit verified source and installed image bytes/INI match and no legacy BMP/atlas remains. The default export does not delete unlisted user files; it removes only obsolete file paths previously declared by its output INI, with resolved containment checks.

## Limits and next work

247 MAIN MENU names still need dedicated modern artwork. Many auxiliary controls share a generic symbol and are not behavioral ports. View rendering, Win32 file dialogs and failed Save As to an unwritable file remain outside dedicated interactive verification. Preserve this evidence when proceeding to the four-viewport spec; do not infer modeling completion from icon coverage.
