use openmatrix9_rust::modeling_exchange::{GeometryKind, capabilities, preflight};

#[test]
fn unknown_geometry_cannot_be_silently_omitted() {
    assert!(preflight(&[GeometryKind::CadCurve, GeometryKind::Unknown]).is_err());
    assert!(preflight(&[GeometryKind::Retained]).is_err());
    assert!(
        preflight(&[
            GeometryKind::CadBrep,
            GeometryKind::Mesh,
            GeometryKind::PointCloud
        ])
        .is_ok()
    );
}

#[test]
fn display_mesh_is_not_a_reason_to_scan_cad_or_cloud_vertices() {
    let cad = capabilities(GeometryKind::CadBrep);
    assert!(cad.snap && cad.export_v5 && cad.boolean);
    for kind in [
        GeometryKind::Mesh,
        GeometryKind::PointCloud,
        GeometryKind::SubD,
    ] {
        let c = capabilities(kind);
        assert!(!c.snap && !c.curve_edit && !c.boolean);
    }
    assert!(!capabilities(GeometryKind::Unknown).select);
}

#[test]
fn geometry_only_scope_does_not_require_history_or_render() {
    assert!(preflight(&[GeometryKind::CadPoint, GeometryKind::CadCurve]).is_ok());
    assert!(!capabilities(GeometryKind::SubD).export_v5);
    assert!(!capabilities(GeometryKind::Mixed).export_v5);
}
