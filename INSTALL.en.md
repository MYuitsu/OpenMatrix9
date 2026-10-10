# Install OpenMatrix9 0.0.2 with official FreeCAD

This experimental package targets **Windows x64 and official FreeCAD 1.1.4**, with Python 3.11, Qt 6.8.3 and OCCT 7.8.1. You do not need the FreeCAD source, Rust or Visual Studio to use it.

1. Download the Windows x64 installer from the [official FreeCAD 1.1.4 release](https://github.com/FreeCAD/FreeCAD/releases/tag/1.1.4). Install and start FreeCAD once. Check Help → About FreeCAD: **1.1.4**. The installation directory may simply be named `FreeCAD 1.1`.
2. Close FreeCAD before replacing a plugin.
3. Download [OpenMatrix9-0.0.2-FreeCAD-1.1.4-Windows-x64.zip](https://github.com/MYuitsu/OpenMatrix9/releases/download/v0.0.2/OpenMatrix9-0.0.2-FreeCAD-1.1.4-Windows-x64.zip) and extract its **OpenMatrix9** folder.
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
- The 0.0.1 native module targets development FreeCAD 27.1 and cannot be interchanged with this 0.0.2 stock module.

If the workbench is missing, check the exact host version, the versioned user directory, extracted layout and both binaries in `bin`, then restart FreeCAD. Open View → Panels → Report view for load errors. Do not copy DLLs from another FreeCAD version.

To uninstall, close FreeCAD and move OpenMatrix9 outside `Mod`; saved projects remain untouched. Compare downloaded file hashes with `SHA256SUMS.txt` on the [0.0.2 release page](https://github.com/MYuitsu/OpenMatrix9/releases/tag/v0.0.2).
