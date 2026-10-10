# Install OpenMatrix9 0.0.1 on Windows

This is an experimental Windows x64 release. Download both assets from
[v0.0.1](https://github.com/MYuitsu/OpenMatrix9/releases/tag/v0.0.1):

1. `FreeCAD-OM9-27.1-Windows-x64.zip` — the matching portable FreeCAD host.
2. `OpenMatrix9-0.0.1-Windows-x64.zip` — the OM9 plugin.

Extract the host to a writable directory such as `C:\CAD`. Extract the plugin,
then put its **OpenMatrix9** directory inside **`FreeCAD-OM9-27.1\Mod`**.
Run **`Mod\OpenMatrix9\Start-OM9.cmd`** to launch FreeCAD and activate OM9.
The native plugin must be at `Mod\OpenMatrix9\bin\OpenMatrix9Gui.pyd`;
avoid an extra nested `OpenMatrix9` directory.

The launcher configures the bundled Python and DLL paths. No Python, Rust,
Visual Studio or Pixi installation is needed. Settings/logs are in
`%APPDATA%\OpenMatrix9\0.0.1`. Close OM9 before replacing its plugin directory.

Use the host supplied in this release: FreeCAD 27.1.0dev, Python 3.13, Qt6,
revision `21d36cfa1eb110a1d0667050ff31706298805bbd`. Ordinary FreeCAD builds are
available at [FreeCAD downloads](https://www.freecad.org/downloads.php) and
[official releases](https://github.com/FreeCAD/FreeCAD/releases), but this
native plugin has not been verified against those builds. Another host
requires an OM9 build against its matching SDK/ABI.

This release includes scoped 3DM exchange, curve/BRep workflows, OM9 clipboard,
OM9 Undo/Redo, and the layer panel. It does not implement all Matrix commands
or full openNURBS. The separately tested full-palette Matrix adapter is not
integrated into the main product UI or included in this OM9 installation;
ordinary Matrix Copy/Paste is not claimed to preserve the full palette.
Undo history is not transferred between applications. Workers default to
60% of logical CPUs, rounded down with a minimum of one.

For launch errors, check `last-launch.stdout.log` / `last-launch.stderr.log`
under the settings directory and FreeCAD's **View → Panels → Report view**.
Run `Start-OM9.cmd` instead of launching this portable host's executable directly.
Verify downloads using `SHA256SUMS.txt` and PowerShell `Get-FileHash`.

OM9 source is tagged `v0.0.1`; the matching FreeCAD source is provided as
`FreeCAD-OM9-27.1-source.zip` and in
[the host repository](https://github.com/MYuitsu/FreeCAD/tree/21d36cfa1eb110a1d0667050ff31706298805bbd).
Keep the bundled licenses and notices for OM9, FreeCAD and their dependencies.
