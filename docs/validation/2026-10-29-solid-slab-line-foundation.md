# Slab source mapping and straight-line foundation — 2026-10-29

The recovered Solid menu maps `SolidSlab` to `! _Slab` (VB6 procedure `modMillC_Defaults.Proc_198_35_1A2F840`, bounded menu registration at `01A34463`–`01A344B4`). This is evidence of command routing, not the Slab algorithm. The normalized M03 reference lists Slab but does not describe its input workflow. The registered Matrix8 PDF path was unavailable in this environment. The [Rhino Slab command documentation](https://docs.mcneel.com/rhino/mac/help/en-us/commands/slab.htm) describes selecting a curve, choosing the offset and height, with Distance, ThroughPoint, BothSides and InCPlane options; a separately available Matrix8 manual is consistent with that broad flow. Defaults, persistence and exact original runtime behavior need direct verification.

Rust now owns a host-independent straight-line perimeter and extrusion calculation. The chosen BothSides interpretation spans equal signed height on either side of the starting plane; it is an OM9 implementation assumption requiring validation in the full command. No Slab UI or dispatcher is enabled, and general curves, miters, plane selection, option state, preview, transaction, Undo and original behavior are not implemented by this foundation.

Standalone OCCT, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -j 2 -p om9-app --test solid solid_slab_line_foundation_builds_measured_solid_without_enabling_command`: four straight-line cases passed, with volume measured from actual capped BRep, positive/negative offset and height, BothSides and invalid-input checks; the actual Slab icon binding remains unavailable.
- `python tools/public_source_audit.py`: 14728 source files, 594 icon bindings, 593 SVGs, no errors.

The derived profile is portable safe Rust; native OCCT only extrudes and measures the test fixture. No runtime publication or complete command is claimed.
