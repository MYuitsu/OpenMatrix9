# legacy3dm — ported Rhino 3DM ↔ OCCT conversion

These files are adapted from the project's own legacy `Gui/ThreeDm*.cpp` conversion
code (same repository, LGPL-2.1-or-later). They are compiled only when the
`om9-kernel` `3dm-brep` feature is enabled.

The host-only dependencies of the originals (`InputTree`/`BasisContract` numeric
policy, Qt/FreeCAD adapters and fixture tracing) are replaced by
`legacy_validation.cpp`; no FreeCAD or Qt code is linked here.

Files:

- `ThreeDmBrep.cpp`, `ThreeDmSurfaces.cpp`, `ThreeDmSolidShells.cpp`: retained
  ON_Brep ↔ OCCT face/trim/surface/shell conversion.
- `brep_curve_adapter.cpp`: exact 3D/2D curve conversion used by the above.
- `brep_3dm_bridge.cpp`: complete-operation C ABI between the 3DM ON_Brep payload
  and the OCCT B-Rep text.
- `legacy_validation.cpp`: standalone validity/basis checks replacing the host
  numeric-policy hooks.

No commercial model or third-party binary is included. The 3DM fixture used in
the verified run is a local user file and is not committed.
