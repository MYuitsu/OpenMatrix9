# Viewport point limits: OM9, Rhino 5 and Blender

Research checked 2026-11-16 during M1-R console testing. This records evidence,
not authorization to replace the product's structural or memory limits.

OM9's `Limits::default().max_points` is 2,000,000. The physical-console High
fixture committed and presented 648 spheres, then failed Sphere 649 twice.
A direct native Controller probe independently returned `Budget("points")`
for its radius input, preserving 648 objects and 648 Undo entries:
`build/standalone-validation/ui-artifacts/m1r-receipts-20261116/native-admission-red.log`.
The GUI audit initially only reported that the command did not commit. Its
failed Close left an idle window open; that period is not a measured render
freeze or a completed 768 High test. The console wrapper now detects the
explicit audit failure and terminates its owned test process promptly.

These points include derived scene display buffers, not just NURBS control
points. In the measured High profile each original sphere display has 2,570
points and 5,008 triangles; source-derived interior wires add scene vertices.
A 768 High stress test therefore needs a larger explicitly configured scene
complexity allowance. The completed final matrix uses 4,000,000 points, while retaining
canonical validation, mesh quality, the ordinary 1,024 MiB Undo soft quota and
the managed RAM/commit checks. This does not change the application's default.

Blender's current published source declares `MESH_MAX_VERTS 2000000000L` in
[DNA_mesh_types.h](https://github.com/blender/blender/blob/main/source/blender/makesdna/DNA_mesh_types.h).
That is a two-billion vertex ceiling declaration for a mesh, not a whole-scene
FPS guarantee, available-memory guarantee or an equivalent limit on CAD
control points. Positions, edges, face corners, attributes, modifiers, Undo and
GPU resources have additional costs and constraints. It is 1,000 times the
OM9 two-million setting, but those scopes differ.

For Rhino 5, the [bounded private-decompile investigation](2026-11-16-rhino5-mesh-capacity.md)
checks the managed Count/SetVertex path, native dynamic arrays and allocator.
No two-million guard was identified in that path. Counts are signed 32-bit,
but the existing-storage reallocation wrapper narrows its byte request to
unsigned 32-bit; neither fact establishes a supported practical maximum.
A 256 MiB constant controls growth increments, and a 64,000 constant in one
command controls partitions; neither is a whole-scene cap. Native addresses,
input fingerprints and uncertainty are recorded in the investigation.

No authoritative universal scene-vertex maximum was established. McNeel's
[Rhino 5 hardware guidance](https://wiki.mcneel.com/rhino/hardwarebaseline)
says the 64-bit version can use the system's RAM and describes VRAM as relevant
to display efficiency; it does not publish a two-million scene cap. The
[archived RhinoCommon mesh implementation](https://github.com/mcneel/rhinocommon/blob/master/dotnet/opennurbs/opennurbs_mesh.cs)
uses integer vertex counts. An integer indexing range is not proof that Rhino
5 can construct, display or edit a mesh at that range. Do not promote an
inferred approximately 2^31 indexing bound into a verified Rhino 5 product cap.

Implication for the OM9 plan: a fixed whole-scene point ceiling is a different
policy from parser/per-geometry structural guards and RAM/VRAM admission.
Separate their scopes before considering a larger product default. A larger
configured test allowance alone does not improve the algorithms, remove
freezes or weaken geometric tolerances.
