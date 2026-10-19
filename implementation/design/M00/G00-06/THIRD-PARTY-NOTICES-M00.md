# Third-party notices — M00 gate additions

## OCCT 8.0.1 (modified component)

- Upstream: https://github.com/Open-Cascade-SAS/OCCT, tag `V8_0_1`, commit `b8f597c677811d1f9f4d8a97f5ae2825c0353a42`.
- License: **LGPL-2.1-or-later with the OCCT exception** (`LICENSE_LGPL_21.txt`, `OCCT_LGPL_EXCEPTION.txt`).
- Modification: `TKG3d` is rebuilt with `tools/patches/occt-8.0.1-offset-d3.patch` (corrected offset-curve
  D2/D3 coefficients). The modified source of that component is the upstream source archive
  (`build/standalone-deps/occt-V8_0_1-source.zip`, sha256 `7c033d91…f0da`) plus that patch; the build
  script is `tools/build_occt_offset_d3.py`.
- Linkage: dynamic (the application loads the released DLLs and the rebuilt `TKG3d.dll`).
- Everything else in the SDK is unmodified; the build receipt records the sha256 of every untouched
  shared output.

## openNURBS (McNeel)

- Pin `eb92af3ba1806b0a34a99aba0d3bda83e3d46083`, notice at `Resources/licenses/openNURBS.txt`.
- Used for 3DM read/write only; no modification.

## QymCAD (reference source only)

- AGPL-3.0-or-later. Not linked, not shipped: read as reference material only.
