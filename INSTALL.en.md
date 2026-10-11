# Install OpenMatrix9 with official FreeCAD

This experimental package targets **Windows x64 and official FreeCAD 1.1.4**, with Python 3.11, Qt 6.8.3 and OCCT 7.8.1. You do not need the FreeCAD source, Rust or Visual Studio to use it.

## Automatic installation and Desktop icon (recommended)

1. Install the Windows x64 build from the **[official FreeCAD 1.1.4 release](https://github.com/FreeCAD/FreeCAD/releases/tag/1.1.4)** and open FreeCAD once. Check Help → About FreeCAD for **1.1.4**; its folder may simply be named `FreeCAD 1.1`.
2. Open the **[Gumball candidate workflow](https://github.com/MYuitsu/OpenMatrix9/actions/workflows/release-windows.yml?query=branch%3Acodex%2Fgumball-release-0.3.4)**. Choose a **successful** run of `codex/gumball-release-0.3.4` for the intended commit, then download **om9-release-files** under Artifacts. GitHub may require sign-in. A queued, failed or older run is not the candidate package. Until the Action passes, its stock-host build remains unverified.
3. Extract the downloaded artifact ZIP. From it, extract **OpenMatrix9-…-SetupLauncher…-Windows-x64.zip** into a folder. Keep the bundled **OpenMatrix9** folder, **launcher-manifest.json** and **Start-OM9** files together, then double-click **Start-OM9.cmd**. The launcher reads the extracted OpenMatrix9 folder.

The launcher installs or updates OM9 in `%APPDATA%\FreeCAD\v1-1\Mod\OpenMatrix9`, creates **OpenMatrix9** shortcuts on Desktop and in Start Menu, and opens the workbench. Use the Desktop icon afterwards; the extracted folder is no longer needed. No Administrator access, FreeCAD source, Rust or Visual Studio is required.

Compare both inner ZIPs with **SHA256SUMS.txt** from the same artifact before installation. This branch produces candidate build artifacts; it is not a published release or release tag. Published builds, including prereleases, remain listed on the **[Releases page](https://github.com/MYuitsu/OpenMatrix9/releases)**.

To update, extract a new SetupLauncher bundle and run its Start-OM9.cmd. Previous modules are backed up outside `Mod` in `v1-1\OM9-backups`; an unchanged module is not recopied. Save and close the FreeCAD instance using this installation when its plugin needs replacing. Incomplete packages or failed cache updates preserve the previous installation. Logs are in `v1-1\OM9-launcher`.

The launcher checks published GitHub releases in the background at startup. It prompts once for a newer application version or launcher revision with a download-page link; network errors do not prevent startup. Candidate branch artifacts do not appear through that release checker. Download and install updates explicitly; launcher-only updates do not recopy the plugin. Status is in `OM9-launcher\update-check.json`.

## Manual installation

1. Download the Windows x64 installer from the [official FreeCAD 1.1.4 release](https://github.com/FreeCAD/FreeCAD/releases/tag/1.1.4). Install and start FreeCAD once. Check Help → About FreeCAD: **1.1.4**. The installation directory may simply be named `FreeCAD 1.1`.
2. Close FreeCAD before replacing a plugin.
3. Download [FreeCAD-1.1.4-Windows-x64 plugin ZIP from the same successful candidate artifact](https://github.com/MYuitsu/OpenMatrix9/actions/workflows/release-windows.yml?query=branch%3Acodex%2Fgumball-release-0.3.4) and extract its **OpenMatrix9** folder.
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

## Historical v0.0.3 scope and continuing host limits

The following limits describe the published v0.0.3 checkpoint. The Gumball candidate has its own native validation; consult the exact commit's Action for its stock CI result. These historical notes do not certify that candidate.

- v0.0.3 adds the public CI build and latest launcher to the official-host compatibility introduced in v0.0.2. CI scope and remaining historical tooling failures are documented in [build validation](docs/validation/2026-10-11-release003-ci.md). Earlier Rhino/Matrix development reports are historical evidence, not a full recertification of this stock build.
- Full openNURBS, all jewelry algorithms and production Matrix palette/lock handoff are not fully accepted yet.
- Finish or cancel a modeling command before changing documents. Official FreeCAD 1.1 can commit a draft in one document when a transaction begins in another; the verified workflow uses one active document.
- Advanced workflows requiring patched development FreeCAD core, including Hatch `PropertyFileIncluded` copy, are not certified on the stock host.
- The 0.0.1 native module targets development FreeCAD 27.1 and cannot be interchanged with this 0.0.3 stock module.

If the workbench is missing, check the exact host version, the versioned user directory, extracted layout and both binaries in `bin`, then restart FreeCAD. Open View → Panels → Report view for load errors. Do not copy DLLs from another FreeCAD version.

To uninstall, close FreeCAD and move OpenMatrix9 outside `Mod`; saved projects remain untouched. Compare downloaded file hashes with `SHA256SUMS.txt` from the same candidate artifact, or on the published [release page](https://github.com/MYuitsu/OpenMatrix9/releases).

Versions use `aa.bb.cc-yyyyMMddhhmm`. The launcher compares the numeric triplet first, then the build timestamp, then a launcher revision for the identical application version. Legacy `aa.bb.cc` bundles remain supported. For candidate artifacts, choose the timestamped ZIP from the intended successful branch commit; for published bundles, match the release tag. The timestamp uses Asia/Saigon time; `aa` changes only at the user’s request, each completed specification increments `bb`, and each completed task increments `cc`.
