# Top11 Mirror: nested mesh and BRep offscreen rendering — 2026-10-31

Status: partial automated evidence; physical M03-UI remains open.

The default-feature fixture puts an asymmetric triangle mesh and an OCCT box BRep in a shared nested block, then invokes Mirror Copy=No about the world Y axis. Two independent projected rays pick the expected nested mesh and BRep occurrence paths after reflection. The actual saved .om9 scene is rendered at 256×256 in each of the four workspace cameras and compared pixel-by-pixel with a separate document containing explicitly reflected world-space meshes derived from the original triangle and BRep display tessellation. A blank-scene comparison requires visible changed pixels in every view. This validates the tested published scene and offscreen wgpu path, including mirrored positions and selection paths; the reference does not establish independent exact BRep geometry, shading correctness under all transforms, actual window swapchain/presentation, or human UI acceptance.

The machine provided a wgpu adapter; the test did not take its no-adapter skip. The focused run reported, respectively for Looking Down, Perspective, Side View and Through Finger: 348, 312, 80 and 72 pixels different from the blank scene, and zero pixels different from the independently placed reference in every view. The permitted reference difference is at most ten pixels, and each view must have more than ten visible pixels. The full headless_render test binary passed 2 tests, 0 ignored, with default features. The fixture uses temporary .om9 and PNG files, removed after verified path containment; code and this record are the retained evidence.

Windows x64, CARGO_TARGET_DIR=L:\om9-target and -j 2. No business logic, C++ or openNURBS measurement was changed. Copy=Yes GPU rendering, other geometry/transforms, 3DM interoperability and physical desktop selection/presentation remain open.

Source SHA-256:

- `crates/om9-app/tests/headless_render.rs`: `4da6902f9a6de18e3c596e7996e7d1422c6e3eeefc080bb8692b3a4cd93adaf8`
- `crates/om9-core/src/top11.rs`: `b6b189f43fa1ff753bb3853fa01641b0232d543db096bad18a3c7d3988c25381`
- `crates/om9-app/src/session/top11.rs`: `df15d2d0a1f25e90181cb410c22aca4d4f1eb4da65fae85bc8a43e70ae7639c3`
- `crates/om9-app/src/headless.rs`: `953bcbf0e2768dbef56f2ee6ec08395778fae67ee70e600bd710ecd517a798d7`
- `crates/om9-scene/src/scene.rs`: `2e3b7a54e32ec4f9238c20a200e114b099fa60ea72e2b47924c15fc1c7970ae4`
