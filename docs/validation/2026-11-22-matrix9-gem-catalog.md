# Matrix9 Gem Loader catalog evidence

The local Matrix90 installation has a model and thumbnail for each of 57 cut/shape keys in Library/ICGems. The catalog presets live with the models.

| Cut family | 3DM samples | Size pairs |
| --- | ---: | ---: |
| Diamond | 22 | 581 |
| Simple | 14 | 323 |
| Gem | 11 | 269 |
| Cabochon | 9 | 280 |
| Pearl | 1 | 23 |
| Total | 57 | 1476 |

The decompiled managed method COMInterface.GetGemSizeList is the size-list entry point. It requests a GemBlock by type and shape and returns the X and Y columns. GemBlock.GetGemBlockData_RhinoScript reads MatrixICGem metadata from the model's Rhino object. SizeCount and indexed SizeX and SizeY encode the presets. Other fields describe type, shape, axis bounds, DepthPercent, volume factor, and optional prong parameters. UtilGem resolves the installed .3dm and .bmp paths from type and shape, with separate lookup for Custom.

The local read-only probe checked length-prefixed UTF-16LE size fields in all 57 3DM files. Every declared pair was present and positive and every type/shape key was unique. Round Diamond has 65 presets from 1.00 to 11.20 mm. Round Pearl has 23 presets from 1.00 to 12.00 mm and a depth ratio of 1. The probe verified the catalog records and did not measure their full geometry or validate pagination against the running legacy form.

Config/GemCrv.xml has 114 entries with named X/Y arrays, including Cabochon_Bullet. Those arrays are a reference for checking outlines. The recovered size-list route reads the gem model data. The role of gem.col in this workflow is unverified.

For OM9, compare catalog presets and XY orientation against the Loader input and viewport, then measure the procedural profiles and table, girdle and culet landmarks. The catalog tables, source models, decompiled code and commercial thumbnails stay in the private reference locations. The OM9 implementation and UI assets are authored in the project.
