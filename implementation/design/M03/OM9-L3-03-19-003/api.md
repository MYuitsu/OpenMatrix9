# OM9-L3-03-19-003 API

- `om9_core::utilities::bounds(&Geometry) -> Option<Bounds>`
- `om9_core::utilities::mesh_from_surface(&NurbsSurface, u, v, &Limits) -> Result<Mesh>`
- `om9_core::utilities::reduce_mesh(&Mesh, target_faces) -> Result<Mesh>`
- `om9_app::utilities::health_entries(&Document, &Limits) -> Vec<HealthEntry>`
- `om9_app::utilities::summary(&[HealthEntry]) -> String`
- `Controller::select_bad_objects`, `select_naked_edges`, `center_selected`, `group_selection`, `ungroup_selection`, `mesh_from_nurbs_selected`, `reduce_mesh_selected`, `health_summary`

Errors are typed; empty/invalid selections return `Error::Missing`/`Error::Invalid` without document changes.
