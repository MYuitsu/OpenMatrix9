use openmatrix9_rust::modeling_exchange::{GeometryKind::*, snap_allowed};

#[test]
fn mesh_and_cloud_default_snap_reads_zero_points_policy() {
    for kind in [Mesh, PointCloud, SubD, Mixed, Retained, Unknown] {
        for mode in [2, 4, 8] {
            assert!(!snap_allowed(kind, true, false, mode));
        }
    }
}

#[test]
fn cad_with_render_mesh_remains_cad() {
    assert!(snap_allowed(CadBrep, true, false, 2));
    assert!(snap_allowed(CadCurve, true, false, 4));
    assert!(snap_allowed(CadPoint, true, false, 8));
    assert!(!snap_allowed(CadCurve, false, false, 2));
    assert!(!snap_allowed(CadCurve, true, true, 2));
    for mode in [0, 1, 16, 32] {
        assert!(!snap_allowed(CadBrep, true, false, mode));
    }
    assert!(!snap_allowed(CadPoint, true, false, 2));
}

#[test]
fn curve_cv_edit_capability_is_scoped_to_native_curve_kind() {
    use openmatrix9_rust::modeling_exchange::capabilities;
    assert!(capabilities(CadCurve).curve_edit);
    for kind in [
        CadPoint, CadBrep, Mesh, PointCloud, SubD, Mixed, Retained, Unknown,
    ] {
        assert!(!capabilities(kind).curve_edit);
    }
}
