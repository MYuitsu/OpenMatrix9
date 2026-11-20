# File menu reference — 2026-11-19

The File menu follows the supplied screenshot order and groups, using authored OM9 icons. Existing New/Open/Save/Save As, 3DM import/export, STL selected export and window close routes remain; unimplemented entries are disabled. Recent-file history and several reference icons are not implemented. This UI change does not alter geometry.

`CARGO_TARGET_DIR=L:/om9-target cargo check -j 2 -p om9-app --bin om9` passed. `python tools/public_source_audit.py` passed. `cargo test -j 2 -p om9-app --test chrome_actions file_menu` failed before compiling the app test: GemCutter implements a different instance of `om9_addon_abi::Interactive` in the existing addon dependency graph. No desktop or interaction validation was completed.

Next: resolve the addon ABI test build, run file_widgets menu interactions and inspect the desktop at supported window sizes.
