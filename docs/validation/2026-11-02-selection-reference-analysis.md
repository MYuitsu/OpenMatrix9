# Selection reference analysis — 2026-11-02

Scope: compare selection mechanics in the locally available Rhino 5 decompilation, Blender's public source, and OM9. This is source analysis, not a benchmark of Rhino or Blender. No reference implementation was copied.

## Observations

- The recovered `CRhinoDoc::PickObjects` procedure filters candidates and checks object bounding boxes against the clipping region before invoking the individual object's pick path. The recovered `CRhinoPickContext::PickMesh` allocates working arrays, transforms eligible mesh vertices into the clipping region, and stores clipping flags. This establishes reuse of per-vertex data within that path, not a persistent cache, GPU selection, or a triangle BVH. Decompiled types and control flow may be imperfect; this is not Rhino source code.
- Blender's [viewport selection implementation](https://github.com/blender/blender/blob/main/source/blender/editors/space_view3d/view3d_view.cc) uses GPU selection modes and has a selection cache for eligible repeated queries; its [editor selection code](https://github.com/blender/blender/blob/main/source/blender/editors/space_view3d/view3d_select.cc) includes selection-buffer contexts for edit-mode operations. These observations do not imply every Blender selection follows the same path. Consulted 2026-11-02. Blender is GPL-licensed; independently implement concepts rather than copying its code into OM9.
- OM9's `om9-scene/src/picking.rs` uses a spatial index to narrow object candidates. Its mesh face path then visits the candidate's triangles and transforms triangle vertices, potentially repeating work on shared vertices; it also has a screen-space fallback. No triangle BVH is used in that path. `om9-app/src/gpu_viewport.rs` includes selection in the GPU scene-pool signature, so a selection change invalidates entries and prepares scene resources again.

## Evidence and interpretation

The earlier [selection latency record](2026-11-02-selection-highlight-latency.md) measures the debug dice fixture, including roughly 92–107 ms for individual `pick_all` calls before the last change, and single before/after click-to-GPU-publication measurements of 1,418 ms and 304 ms. The measurements are not frame presentation latency and are not a controlled performance comparison with the reference applications. A 30 FPS target means a 33.3 ms frame interval; click latency and sustained frame rate are different measurements.

Promising independent approaches to measure are retaining immutable GPU geometry across selection changes, avoiding repeated vertex transformations, and a mesh-level acceleration structure with a screen-space fallback. An ID-buffer GPU picker is another option, but a simple frontmost-pixel result would not preserve OM9's all-candidate/overlap behavior. Any change must preserve hidden and locked object rules, stable object/face identity, preview transforms, precision, and selection results. None is established as the fastest without matched measurements.

Next: profile repeated selection on the same visible fixture, separate candidate picking from GPU resource publication and displayed-frame time, then implement and compare the highest-cost confirmed path. Source analysis alone does not establish 30 FPS or equal behavior across applications.
