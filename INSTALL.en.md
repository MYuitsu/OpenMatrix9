# Install OpenMatrix9 0.0.3 with official FreeCAD

This experimental package targets **Windows x64 and official FreeCAD 1.1.4**, with Python 3.11, Qt 6.8.3 and OCCT 7.8.1. You do not need the FreeCAD source, Rust or Visual Studio to use it.

## Automatic installation and Desktop icon (recommended)

Install official FreeCAD 1.1.4, then download [the setup launcher ZIP](https://github.com/MYuitsu/OpenMatrix9/releases/download/v0.0.3/OpenMatrix9-0.0.3-SetupLauncher-Windows-x64.zip). Extract the entire ZIP and double-click **Start-OM9.cmd**. It installs or updates the bundled module in `%APPDATA%\FreeCAD\v1-1\Mod\OpenMatrix9`, creates **OpenMatrix9** shortcuts with its icon on Desktop and in Start Menu, and starts the workbench. No Administrator access is needed.

Use Desktop or search OpenMatrix9 in Start Menu afterwards; it remains usable after moving or deleting the extracted folder. To update, run Start-OM9.cmd from a new bundle. Previous modules are kept outside Mod in `v1-1\OM9-backups`; an unchanged module is not recopied. Save and close the FreeCAD instance using this installation if an update is required. The launcher never closes your application or downloads a plugin. Incomplete packages and failed cache updates preserve the previous installation. Logs are in `v1-1\OM9-launcher`. Check this ZIP against **SHA256SUMS.txt** on the release page.

The launcher checks GitHub releases in the background whenever OM9 starts. New application versions or launcher revisions prompt once with a download-page link. Network errors do not prevent startup. Download/extract the new bundle and run its Start-OM9.cmd to update; it never installs updates automatically. A launcher-only update does not recopy the plugin or require closing OM9. Check status in `OM9-launcher\update-check.json`.

## Manual installation

1. Download the Windows x64 installer from the [official FreeCAD 1.1.4 release](https://github.com/FreeCAD/FreeCAD/releases/tag/1.1.4). Install and start FreeCAD once. Check Help → About FreeCAD: **1.1.4**. The installation directory may simply be named `FreeCAD 1.1`.
2. Close FreeCAD before replacing a plugin.
3. Download [OpenMatrix9-0.0.3-FreeCAD-1.1.4-Windows-x64.zip](https://github.com/MYuitsu/OpenMatrix9/releases/download/v0.0.3/OpenMatrix9-0.0.3-FreeCAD-1.1.4-Windows-x64.zip) and extract its **OpenMatrix9** folder.
4. Press **Win + R**, enter `%APPDATA%\FreeCAD\v1-1\Mod` and create the directory if needed. Place **OpenMatrix9** inside it. Move an older installation outside `Mod` as a backup first.
5. Restart your installed FreeCAD and select **OpenMatrix9** from the workbench selector.

The required layout is:

```text
%APPDATA%\FreeCAD\v1-1\Mod\OpenMatrix9\
    Init.py
    InitGui.py
    ThreeDm.py
    Resources\
    bin\OpenMatrix9Gui.pyd
    bin\OM9ThreeDmImportWorker.exe
```

Avoid nesting `OpenMatrix9\OpenMatrix9`. Keep the plugin in the user directory; do not replace files in `Program Files`. Administrator privileges are not needed to use OM9.

For a quick check, create a document, draw a Line, then Undo and Redo. The Layers panel provides the 32-color palette and supported lock/visibility controls. Disabled menu entries indicate commands still being ported.

Use Copy/Paste for selected geometry and Copy Session for the full layer table, including empty layers, colors, locks/visibility and the active layer. 3DM import/export and FreeCAD FCStd project saving are also available. Leave the system clipboard unchanged during a transfer.

The Matrix diagnostic scripts in the source are development tools, not installation steps. OM9 is a FreeCAD workbench; this package does not install a Matrix `.rhp` or a production Matrix-side receiver.

## Experimental scope

- This release adds official-host compatibility. Fresh 1.1.4 reports cover the named curve, layer, clipboard, BRep, worker and viewport cases. Earlier Rhino/Matrix development reports are historical evidence, not a full recertification of the stock build.
- Full openNURBS, all jewelry algorithms and production Matrix palette/lock handoff are not fully accepted yet.
- Finish or cancel a modeling command before changing documents. Official FreeCAD 1.1 can commit a draft in one document when a transaction begins in another; the verified workflow uses one active document.
- Advanced workflows requiring patched development FreeCAD core, including Hatch `PropertyFileIncluded` copy, are not certified on the stock host.
- The 0.0.1 native module targets development FreeCAD 27.1 and cannot be interchanged with this 0.0.3 stock module.

If the workbench is missing, check the exact host version, the versioned user directory, extracted layout and both binaries in `bin`, then restart FreeCAD. Open View → Panels → Report view for load errors. Do not copy DLLs from another FreeCAD version.

To uninstall, close FreeCAD and move OpenMatrix9 outside `Mod`; saved projects remain untouched. Compare downloaded file hashes with `SHA256SUMS.txt` on the [0.0.3 release page](https://github.com/MYuitsu/OpenMatrix9/releases/tag/v0.0.3).
