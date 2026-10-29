# Sphere 3Point/4Point retry and preview — 2026-10-29

The normalized Sphere manual describes 4Point as a sphere through the circle defined by the first three points and a fourth point. The recovered command inventory maps the menu variants to `_Sphere _3Point` and `_Sphere _4Point` (see `docs/analysis/matrix-command-layers.csv`); it does not establish the native solver's error or preview behavior.

Standalone OM9 now retains valid inputs when a 3Point or 4Point third point is collinear, or when a 4Point fourth point is coplanar. These errors create no object. The 4Point preview uses the same circle-based calculation as commit, and suppresses invalid coplanar candidates; it is a three-circle wire approximation, not a BRep preview.

Windows, `CARGO_TARGET_DIR=L:/om9-target`, `cargo test -j 2 -p om9-app --test solid solid_sphere_`: 38 passed. New controller fixtures cover invalid input retry, no document mutation, a translated preview with analytical center and radius, committed volume, and Undo. They do not verify physical pointer input, GPU presentation, original Matrix behavior, all numeric limits or every Sphere option. The previous FreeCAD record is historical and is not standalone evidence.
