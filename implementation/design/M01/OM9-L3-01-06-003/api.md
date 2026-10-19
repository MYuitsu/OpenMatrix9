# Tessellation cache (OM9-L3-01-06-003)

`Renderer::render(key, mesh, camera, w, h)` uploads the mesh on first use and caches the vertex/index
buffers under `key`; `Renderer::uploads()` counts the uploads; `Renderer::invalidate(key)` drops one entry
and `clear_cache()` drops all. The key is the cache identity — the caller changes it when the geometry
behind it changes.

Test: `crates/om9-render/tests/render_loop.rs::the_mesh_cache_uploads_once_per_key` —
first draw uploads (1), second reuses (still 1), invalidate then draw uploads again (2).
