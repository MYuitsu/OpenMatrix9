# OCCT angular deflection adapter

Follow-up: the angular setting is now wired into the STL source paths; see [STL angular profile](2026-11-03-stl-angular-profile.md). This record describes the earlier adapter-only delivery.

P3a partial: `read_brep_with_angle` passes a finite angular deflection in `(0, pi]` radians through the synchronous OCCT snapshot adapter. Existing callers retain the 0.3 rad default. The selected native adapter remains a bounded synchronous operation; the shared C++ snapshot implementation validates its arguments.

On Windows with OCCT and openNURBS enabled, `CARGO_TARGET_DIR=L:/om9-target`, `cargo test -p om9-app -j 2 --test stl_export` passed 19 tests. A sphere with the same source and linear deflection produces more triangles with 0.1 than 0.6 radians; 0.3 matches the existing default triangulation. Invalid, infinite and NaN angles are refused. This verifies one adapter fixture and existing STL regressions, not a global angle or geometric-deviation bound.

The export profile and dialog do not yet pass an angular setting to this API. Retained 3DM B-Reps currently use a different conversion path; standalone NURBS surfaces use sampled meshing instead of the required exact kernel conversion. These paths must be unified and checked before exposing a setting as applying to every export. Background capture, resource reservations, desktop operation and the other plan gates remain open. The live OM9 session was not modified.
