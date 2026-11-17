#![cfg(feature = "occt-brep")]
use om9_app::{display::ViewportDetail, ui::MatrixApp};
use om9_core::{geometry::Geometry, model::Document};

#[test]
fn both_display_profiles_smooth_without_changing_exact_sphere_source() {
    let mut sources = Vec::new();
    for detail in [ViewportDetail::Light, ViewportDetail::High] {
        let mut app =
            MatrixApp::new(Document::with_template_tolerance("smooth display source")).unwrap();
        app.controller.request_viewport_detail(detail).unwrap();
        for command in ["Sphere", "0,0,0", "5"] {
            app.controller.submit(command).unwrap();
        }
        let entity = app.controller.document().entities.values().next().unwrap();
        let Geometry::Brep(body) = &entity.geometry else {
            panic!("exact BRep");
        };
        let source = body.canonical_fingerprints();
        let om9_scene::scene::RenderBuffers::Mesh {
            vertices,
            triangles,
            ..
        } = app.controller.scene().unwrap().objects[0].buffers.as_ref()
        else {
            panic!("display mesh");
        };
        let mesh = om9_render::normals::smooth_faces(
            vertices.iter().map(|p| p.map(|v| v as f32)).collect(),
            triangles,
        );
        assert_eq!(mesh.indices.len(), triangles.len() * 3);
        for (p, n) in mesh.positions.iter().zip(&mesh.normals) {
            let length = p.iter().map(|v| v * v).sum::<f32>().sqrt();
            let cosine = p.iter().zip(n).map(|(p, n)| p * n).sum::<f32>().abs() / length;
            assert!(cosine > 0.97, "{detail:?} {p:?}: {n:?}");
        }
        assert_eq!(body.canonical_fingerprints(), source);
        let measures =
            om9_kernel::creation::measure(body, app.controller.coordinator().limits()).unwrap();
        assert!((measures.volume_mm3 - 4. / 3. * std::f64::consts::PI * 125.).abs() < 1e-6);
        sources.push(source.0);
    }
    assert_eq!(sources[0], sources[1]);
}
