> Public packaging: raw forms, vendor manuals, original INI files and screenshots mentioned below require the owner's private reference archive. Use Resources/menu for the public executable menu definitions and docs/features for authored feature contracts.

# Read by UI component

Paths below are relative to the OpenMatrix9 checkout. Form filenames are inside `ref/matrix9/vb6-lite/Matrix90/`; feature IDs resolve through the workflow helper `feature ID`.

For layout, read form header/control declarations only, stopping at `Attribute VB_Name`. Locate that boundary with `rg -n 'Attribute VB_Name' <form>`, then inspect the relevant control range. These forms can contain thousands of assembly lines after the designer metadata; search one named event only when behavior remains unresolved.

For an entire small UI family, request enough bounded search results: `search OM9-SNAP --domain 01-core --limit 20` returns the current 16 Snap entries. Always compare `total` with the returned count; choose one ID and run `feature ID` before reading its spec. Default searches return only five entries, not the complete family.

| Work being done | Read next | Verify before enabling |
|---|---|---|
| Overall startup/sidebar | Accepted menu design, OM9-IFACE-001; selected screenshot; `frmMaster.frm` only for shell/control identity | Sidebar order, document editing, dock restoration |
| ICON HISTORY | `frmMenuIconHistory.frm`, accepted design | Latest successful commands only, capacity 20 in current design, host Undo/Redo availability |
| MAIN MENU | `ref/MainMenu.ini`, OM9-MAIN-001, `frmMenuTools.frm` | Current INI: 18 groups, 11 quick icons, Custom and Reset; remeasure if changed |
| DISPLAY | `frmCMenuDisplay.frm`; search `OM9-DISPLAY` within `01-core` then select one ID | Native view mode/grid command identity and document requirements |
| SNAPS | `frmMenuSnaps.frm`; search `OM9-SNAP` within `01-core` then select one ID | Actual geometric snapping integration; a toggle drawing alone is insufficient |
| INFO & SETTINGS | `frmMenuInfoSettings.frm`; specific `OM9-INFO` spec | FreeCAD property/options equivalents; no execution of Rhino macro text |
| LAYERS | `frmMenuLayers.frm`, OM9-LAYER-001 | Visibility, selection, color and save behavior; leave unsupported semantics disabled |
| PROJECTS | `frmMenuProjects.frm`, OM9-PROJECT-001; OM9-PROJECTDB-001 only when implementing database behavior | Files/database are a separate behavior slice |
| F6/context menu | OM9-F6-001, `ref/ContextMenu.xml`, `ref/RightClick.ini` | Selection-sensitive command availability |
| Four views/grid/theme | OM9-VIEWPORT-001, [accepted command/grid theme](command-grid-theme.md), packaged reference screenshot and live FreeCAD view APIs | Real camera/model interaction, black canvases, world-space major/minor grid and theme restoration |
| Command region | [accepted command/grid theme](command-grid-theme.md), OM9-IFACE-001 and CommandHistory spec | Top green transcript + inline input, shared native geometry, recall/draft/Esc and focus guards |
| Section title styling | `ctrTitleBar.123`, `.ctx` | Parent/control units, font, color encoding and runtime dimensions |
| One icon | `ref/Matrix.rui` macro/GUID/bitmap metadata; selected `.frx`/`.ctx` pointer from form | Resource identity and image dimensions match the command |

## Evidence conversion

`*.123` can be textual UserControl output. Inspect its content instead of assuming the extension is binary. `.frx`/`.ctx` contain resource blobs; an `OleObjectBlob` is not automatically a PNG. Validate offsets and format before extracting. Work on copies/derived outputs; preserve raw exports.

A VB6 form's design dimensions may use Twips or another scale and may be changed at runtime. Determine `ScaleMode`, parent origin and runtime resizing before converting. Likewise verify VB6 BGR/long colors against rendered controls; do not copy integer color values as RGB hex blindly.

INI currently uses Clayoo/Emboss/Settings; the screenshot uses SubD/Art/Setting. Keep the chosen source order and explicitly record display differences. Clayoo (`14-subd`) and T-Splines (`06-tsplines`) have separate feature catalogs; menu-name similarity does not establish functional equivalence.

## Current slice acceptance

- Seven sections in screenshot order. MAIN MENU and ICON HISTORY interactive according to accepted design.
- Sidebar starts at 300 Qt logical pixels; title/icon dimensions come from the design and are checked visually. Wrap/scroll in small windows; record tested DPI values.
- GUI-thread-only widgets, clear ownership, stable ABI string lifetimes, bounded indices, no Rust unwind across C ABI.
- Successful command acknowledgment precedes history insertion. Reset restores UI state and hidden sections.
- Three workbench switch cycles produce one sidebar and restore altered docks. Test no-document and document-open states.
- Missing icons have a consistent fallback/tooltip; distinguish verified extraction from fallback in the report.
- Use actual screenshots to state similarity. Read the accepted command/grid theme for current workspace targets and the live ledger for verified behavior; visual similarity does not complete the remaining Matrix tools.
