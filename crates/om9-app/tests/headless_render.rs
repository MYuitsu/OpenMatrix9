// SPDX-License-Identifier: LGPL-2.1-or-later
//! Headless viewport capture: one PNG per workspace camera, with no window, egui or swapchain.
//! The test skips when the machine exposes no wgpu adapter.
use om9_app::headless;
use om9_archive::Archive;
use om9_core::{
    budget::Limits,
    geometry::{Geometry, Mesh},
    model::{Document, Entity},
    transaction::Cancellation,
};

#[test]
fn a_project_renders_one_png_per_viewport_without_a_window() {
    if om9_render::Renderer::new().is_none() {
        eprintln!("skip: no wgpu adapter on this machine");
        return;
    }
    let directory = std::env::temp_dir().join(format!("om9-headless-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("test directory");
    let project = directory.join("triangle.om9");
    let mut document = Document::new("headless");
    let mesh = Mesh {
        vertices: vec![[0., 0., 0.], [10., 0., 0.], [0., 10., 0.]],
        faces: vec![vec![0, 1, 2]],
        normals: vec![],
        colors: vec![],
        corner_uvs: vec![],
    };
    let entity = Entity::new("triangle", Geometry::Mesh(mesh), None);
    document.entities.insert(entity.id, entity);
    Archive::save(
        &project,
        &document,
        &Limits::default(),
        &Cancellation::default(),
    )
    .expect("fixture project");
    let files = headless::render_project(&project, &directory, 96, 64).expect("headless render");
    assert_eq!(files.len(), 4, "one image per workspace viewport");
    for file in &files {
        let bytes = std::fs::read(file).expect("written image");
        assert!(bytes.len() > 8, "{} is empty", file.display());
        assert_eq!(
            &bytes[..8],
            b"\x89PNG\r\n\x1a\n",
            "{} is a PNG",
            file.display()
        );
    }
    let sheet = directory.join("sheet.png");
    headless::render_sheet(&project, &sheet, 96, 64).expect("four-view sheet");
    assert_eq!(
        image::image_dimensions(&sheet).expect("sheet dimensions"),
        (192, 128),
        "2x2 sheet of 96x64 tiles"
    );
    let _ = std::fs::remove_dir_all(&directory);
}

#[cfg(feature = "occt-brep")]
#[test]
fn mirrored_nested_mesh_and_brep_render_and_pick_at_expected_positions() {
    use om9_app::session::Controller;
    use om9_core::{geometry::Transform,model::Definition,top11::Top11Action};
    use om9_scene::{camera::{Camera,Viewport},scene::PickIntent};
    use om9_kernel::creation::{self,Primitive};
    if om9_render::Renderer::new().is_none() {eprintln!("skip: no wgpu adapter on this machine");return;}
    let limits=Limits::default();
    let mesh=Mesh{vertices:vec![[0.,0.,0.],[3.,0.,0.],[0.,3.,0.]],faces:vec![vec![0,1,2]],normals:vec![],colors:vec![],corner_uvs:vec![]};
    let box_shape=creation::primitive(Primitive::Box{x_mm:2.,y_mm:3.,z_mm:1.},creation::DEFAULT_DEFLECTION_MM,&limits).unwrap();
    let mut a=Entity::new("mesh",Geometry::Mesh(mesh.clone()),None);a.placement=Transform::translation([0.,5.,0.]);let mesh_id=a.id;
    let mut b=Entity::new("brep",Geometry::Brep(box_shape.clone()),None);b.placement=Transform::translation([0.,-5.,0.]);let brep_id=b.id;
    let inner=Definition::new("inner",vec![a,b]);let inner_id=inner.id;
    let mut nested=Entity::new("nested",Geometry::Instance(inner_id),None);nested.placement=Transform::translation([4.,0.,0.]);let nested_id=nested.id;
    let outer=Definition::new("outer",vec![nested]);let outer_id=outer.id;
    let mut root=Entity::new("root",Geometry::Instance(outer_id),None);root.placement=Transform::translation([3.,0.,0.]);let root_id=root.id;
    let mut doc=Document::with_template_tolerance("mirror render");doc.definitions.insert(inner_id,inner);doc.definitions.insert(outer_id,outer);doc.entities.insert(root_id,root);
    let mut c=Controller::new(doc,limits).unwrap();c.select_objects(&[root_id]).unwrap();c.top11_invoke(Top11Action::Mirror).unwrap();c.submit("Copy=No").unwrap();c.submit("YAxis").unwrap();
    let camera=Camera::orthographic([0.,0.,100.],[0.,0.,0.],[0.,1.,0.],40.).unwrap();let viewport=Viewport{origin:[0.,0.],size:[256.,256.]};
    for (p,leaf) in [([-8.,6.,0.],mesh_id),([-8.,-3.5,1.],brep_id)] {let screen=camera.project(p,viewport).unwrap().unwrap().screen;
        let hit=c.scene().unwrap().pick(screen,&camera,viewport,4.,PickIntent::Select).unwrap().expect("mirrored occurrence pick");
        assert_eq!(hit.root,root_id);assert_eq!(hit.path,vec![nested_id,leaf]);}
    let directory=std::env::temp_dir().join(format!("om9-mirror-render-{}",uuid::Uuid::new_v4()));std::fs::create_dir_all(&directory).unwrap();
    let actual=directory.join("actual.om9");Archive::save(&actual,c.document(),&limits,&Cancellation::default()).unwrap();
    let mut reference=Document::with_template_tolerance("mirror reference");
    let map=|p:[f64;3],y:f64|[-(p[0]+7.),p[1]+y,p[2]];
    let mut reference_mesh=mesh;reference_mesh.vertices=reference_mesh.vertices.into_iter().map(|p|map(p,5.)).collect();
    let box_mesh=Mesh{vertices:box_shape.display.points.iter().map(|p|map(*p,-5.)).collect(),
        faces:box_shape.display.triangles.iter().map(|t|vec![t[0],t[1],t[2]]).collect(),normals:vec![],colors:vec![],corner_uvs:vec![]};
    for e in [Entity::new("mesh",Geometry::Mesh(reference_mesh),None),Entity::new("brep",Geometry::Mesh(box_mesh),None)] {reference.entities.insert(e.id,e);}
    let expected=directory.join("expected.om9");Archive::save(&expected,&reference,&limits,&Cancellation::default()).unwrap();
    let empty=directory.join("empty.om9");Archive::save(&empty,&Document::with_template_tolerance("empty"),&limits,&Cancellation::default()).unwrap();
    let frames=headless::render_project(&actual,&directory.join("actual"),256,256).unwrap();
    let expected_frames=headless::render_project(&expected,&directory.join("expected"),256,256).unwrap();
    let empty_frames=headless::render_project(&empty,&directory.join("empty"),256,256).unwrap();
    for ((a,b),blank) in frames.iter().zip(&expected_frames).zip(&empty_frames) {
        let actual=image::open(a).unwrap().to_rgba8();let expected=image::open(b).unwrap().to_rgba8();let blank=image::open(blank).unwrap().to_rgba8();
        let diff=actual.pixels().zip(expected.pixels()).filter(|(a,b)|a!=b).count();
        let visible=actual.pixels().zip(blank.pixels()).filter(|(a,b)|a!=b).count();
        println!("{}: visible_pixels={visible}, reference_diff_pixels={diff}",a.file_name().unwrap().to_string_lossy());
        assert!(visible>10,"no visible mirrored geometry in {}",a.display());assert!(diff<=10,"{} differs in {diff} pixels",a.display());
    }
    let target=directory.canonicalize().unwrap();let temp=std::env::temp_dir().canonicalize().unwrap();
    assert!(target.starts_with(temp) && target.file_name().is_some_and(|name|name.to_string_lossy().starts_with("om9-mirror-render-")));
    std::fs::remove_dir_all(target).unwrap();
}
