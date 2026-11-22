# Matrix9 Gem Loader size dial

The built-in library provides the nominal size table. The object metadata namespace in each `.3dm` supplies the ordered X/Y pairs and the depth ratio. X is width and Y is length for the regular visible shapes. Round lists the same diameter on both axes. The depth is the stored ratio times width. Equal lengths can have different widths, especially Baguette. Retaining record order also matters for Baguette.

## Evidence

- Original dialog `frmDialogGemSelect.Loadsizes` at VA `0169AB00`, exact Ghidra definition at export line 152509. The assembly at `0169B88A–0169BB4A` supplies the one/two/three page tests, division, VB Integer conversion and final index. The two breakpoints are 23 and 45 rows.
- Original tab click `frmDialogGemSelect.imgTabs_MouseDown` at VA `0169BD80` and tab refresh at `0169C140`. The refresh uses the adjacent entries from the boundary array as **inclusive** start and end indices. A shared endpoint must appear in both tabs. The dialog resets the tab index when loading a shape.
- Managed `Matrix.Utilities.GemBlock.GetGemBlockData_RhinoScript` identifies the relevant object metadata in the installed library. `Matrix.Interfaces.COMInterface.GetGemSizeList` supplies the ordered pairs to the UI. `Matrix.Utilities.GemBlock.Weight` and `UtilGem` identify the diamond estimate displayed by the size chooser.
- A read-only measurement of the installed Matrix90 `Library/ICGems` found 57 sets of measurements with 1,476 rows. The default shape galleries use 51 distinct cut–shape combinations. Six extra files use corner-to-corner axes on triangle or trillion cuts. The center click in `LeadGemSelect_MouseDown` (`016985BD–016985D8`) toggles the state consulted by `0169A030` when resolving those variants. Original models, code, screenshots and binary metadata stay in their local locations.

## Verified page boundaries

| Diamond cut | Unique sizes | Visible per tab | Shared endpoints |
| --- | ---: | --- | --- |
| Round | 65 | 23 / 22 / 22 | Ø 3.40 mm and Ø 6.80 mm |
| Oval | 23 | 13 / 11 | 7.20 × 5.20 mm |
| Baguette Straight | 56 | 20 / 19 / 19 | 2.50 × 2.00 mm and 3.50 × 1.75 mm |
| Marquise | 21 | 21 | Single tab |

Diamond Oval 6.00 × 4.00 mm belongs to tab 1 and has a nominal depth of 2.449197025885012 mm. The 0.612299256471253 depth ratio is relative to the 4.00 mm width.

[Gem Loader implementation and Windows checks](2026-11-22-dock-popup-theme.md#matrix9-size-catalog-and-tabs)
