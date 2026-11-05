# Sphere STL mesher memory investigation

## Scope and method

The reported high-water event was reproduced in separate Windows release test processes using OCCT 8.0.1 and a radius 12 mm sphere, requested linear tolerance 0.001 mm, adapter deflection 0.0005 mm and angle 0.3 rad. `PeakPagefileUsage` is the Windows process lifetime commit high-water counter, not bytes paged out and not GPU usage. Peaks include initialization and other native and Rust allocations. System load was not controlled; these are individual runs, not statistical performance measurements.

To isolate the native mesher, `tools/occt_mesh_memory_probe.cpp` used the exact same 895-byte OCCT B-rep input in fresh processes (SHA-256 `9F3AA08DD336382D54FF84377B7CF73A185F553B70D3BDFB3B3ADAEF09EF3A38`) with the same deflection and angle. Peak commit was 1,067,962,368 bytes with OCCT's default algorithm, 1,067,958,272 bytes with Watson explicitly selected, and 136,208,384 bytes with Delabella. Native meshes contained 241,130, 241,130 and 241,324 triangles respectively. This comparison implicates the default 2D triangulation on this fixture; it does not locate a specific allocation or demonstrate a memory leak.

## Release export comparison

In fresh release processes, the complete fine-sphere STL export had peak commit 1,101,467,648 bytes (1,050.44 MiB) before and 214,548,480 bytes (204.61 MiB) after, about 80.5% lower. Peak working set was 957,128,704 and 202,907,648 bytes respectively. The single observed elapsed times were 5,715.282 ms and 1,004.579 ms; one run per version does not establish a stable speedup. The encoded output is 12,066,184 bytes after versus 12,056,484 before.

After the change the callback-only native snapshot probe peaked at 179,163,136 bytes; the light radius 5 mm/0.05 mm export at 63,864,832 bytes; eight radius 5 mm/0.005 mm copies at 100,012,032 bytes. The corresponding baseline light and eight-copy peaks were 57,569,280 and 127,672,320 bytes. Initial process memory differed, so the small light-case increase is not attributable from these single runs. No user desktop executable was replaced or operated.

## Change and precision evidence

For STL export only, the native adapter selects Delabella when a validated root solid has exactly one spherical face. All other shapes and display meshing retain the prior algorithm. Linear deflection and angular deflection are unchanged. The OCCT toolkit's Delabella MIT notice is preserved in `Resources/licenses/delabella-LICENSE` and included by the standalone packaging script.

The same analytic sphere fixture was exported before and after. Binary STL floats were decoded and every triangle was checked for maximal radial distance to the analytic sphere, including its interior and edges. The worst mesh-to-sphere discrepancy was 0.000754229 mm before and 0.000584746 mm after against a 0.001 mm request. In the opposite direction, 48 sampled radial rays had maximum discrepancy 0.000370435 mm in both runs. Encoded output was reported watertight. There were 241,128 triangles before and 241,322 after: the files and triangulations are not identical. The first measurement covers every output triangle for this sphere, whereas source-to-mesh is sampled, not a global bidirectional certificate or proof for other shapes.

### Pole and periodic-seam probes (2026-11-04)

The 12 mm radius analytic sphere fixture now also checks 35 source radial probes at and near both poles and on both sides of the periodic seam, including repeated pole directions. All rays intersected the encoded STL; maximum sampled radial discrepancy was 0.000245094 mm against the 0.001 mm request. The same run contained 241,322 triangles, with maximum output-triangle radial discrepancy 0.000584746 mm and maximum across the existing 48 source rays 0.000370435 mm. The focused integration test passed on E: with `-j 2`. These source probes are finite samples, not a global coverage certificate or a result for other geometry, placements or platforms.

### Nonuniformly scaled sphere source probes (2026-11-04)

The existing 5 mm radius sphere with placement scale `[3, 1, 0.5]` now also probes 25 source radial directions on the resulting ellipsoid, including repeated pole directions, axes and near-seam directions. All probes intersected the encoded STL; the maximum sampled radial discrepancy was 0.000367165 mm against the 0.001 mm document tolerance. The existing test's maximum distance from each triangle center to its radial source point was 0.000489802 mm. The focused test passed on E: with `-j 2`. Both values describe this fixture and chosen sampling method; they are not a global bidirectional error certificate or an independent closest-point maximum.

The kernel integration suite passed 74 tests, the STL integration suite 42 tests, and worker cancellation unit suite 8 tests. The integration suite includes a nonuniformly scaled sphere, topology boundaries, open surfaces, winding, NURBS, quality settings and unchanged display/export independence. `tools/public_source_audit.py` passed with no errors; the standalone packaging script copies the dependency license directory. The active desktop session was not operated.

This change does not establish an upper bound on OCCT transient allocation, guarantee tolerance for every possible source, solve the process-wide RAM budget, or prove desktop responsiveness. Reproduce a user's actual model and export profile before attributing its peak to this same cause. Single-face spherical solids other than the tested fixtures and all other geometry continue to require broader corpus validation.
