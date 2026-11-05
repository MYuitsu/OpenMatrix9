# Tiny STL triangle normal

The binary STL writer's normal calculation treated every triangle whose cross-product magnitude was at most `1e-30` as degenerate. That fixed area floor rejected some finite triangles even when their encoded `f32` vertices were distinct and met the requested quantization tolerance. The writer now computes the normal length with `hypot` and refuses zero or non-finite magnitude; the existing encoded-triangle collapse, orientation and quantization checks remain in place.

A direct writer fixture uses a right triangle with `1e-16` model-unit sides and a `1e-20` output-unit quantization limit. It checks the actual 134-byte binary STL, +Z normal and distinct encoded vertex coordinates. The zero-area control remains rejected. This is an encoder fixture near the origin, not an assertion that the entire OM9 UI/kernel pipeline accepts that feature size or a certification of STL source deviation, manifoldness or manufacturing suitability.

On E: with `-j 2`, all 40 exchange unit tests and all 52 app STL export integration tests passed. Public-source audit passed. The unrelated parallel GPU publication suite remained live without a terminal result.

The unrelated parallel GPU run was subsequently stopped because of sustained resource usage; a serial rerun passed separately. See [GPU follow-up](2026-11-03-published-scene-picking.md#gpu-suite-resource-follow-up-2026-11-04).

Follow-up adds an authored-mesh document fixture through `Controller::export_stl_selected_with_profile`, staging-file verification and publication. A `1e-16`-side triangle is exported with a requested `1e-20` mm linear tolerance, its file has one triangle with nonzero encoded edges and +Z normal, one encoded coordinate differs from the source by less than `1e-21` mm, and the document is unchanged. The focused integration test passed on E: with `-j 2`; this is one authored open surface, not a smooth B-rep tessellation or global error certificate.
