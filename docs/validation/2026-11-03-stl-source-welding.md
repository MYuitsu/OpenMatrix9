# STL weld ownership

The STL collection path now includes the selected-source identity in each weld key. Coincident points from distinct selected objects no longer share indices, while per-object seam welding continues as before. This avoids a false closedness result for two coincident open triangles. It does not implement complete manifold verification: incidence parity, quantized weld keys, early f32 conversion, and topology within a single object remain separate open issues.

Windows default features, `CARGO_TARGET_DIR=L:/om9-target`, Cargo `-j 2`: `cargo test -p om9-app -j 2 --test stl_export` passed 12 tests. The added integration regression creates two coincident authored open triangles and verifies two encoded triangles, six reported boundary edges, and no watertight claim. Existing tests cover single B-rep/sphere/native B-rep and the prior mirror test; independent B-rep seam/cavity and coincident solid topology remain unverified. No desktop or release performance result is implied.

Next: replace parity-based closedness with explicit boundary, incidence and orientation checks, retain f64 until writing, and implement the independent export profile and captured background path described in `implementation/design/M01/viewport-export-separation-20261102/PLAN.md`.
