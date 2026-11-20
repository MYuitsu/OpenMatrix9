# Options View reference — 2026-11-19

The Options command and icon use the existing Options owner and open a categorized dialog with a View page matching the supplied reference structure. General retains the existing language and Undo controls. View settings and other categories are disabled pending real owners; the reference values are not represented as active settings. The UI change does not alter geometry.

`CARGO_TARGET_DIR=L:/om9-target cargo check -j 2 -p om9-app --bin om9` passed. `python tools/public_source_audit.py` passed. The targeted `addon_session` test did not compile because the existing add-on dependency graph includes duplicate `om9_addon_abi` types in the surfaces/curves add-ons. The adapted test has not run. Desktop, pointer interactions, DPI and actual View setting behavior remain unverified.

Next: resolve the dependency issue, run the Options/General interaction test and inspect the window at supported sizes; implement each View setting against its actual owner before enabling it.
