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

#[test]
fn many_edge_trim_hole_renders_on_gpu_after_reopen() {
    use om9_core::rhino_semantics::{set_trimmed_surface,TrimmedSurface};
    use om9_scene::camera::{Camera,Viewport};
    if om9_render::Renderer::new().is_none() {eprintln!("skip: no wgpu adapter on this machine");return;}
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([-10.,-10.,0.],[1.,0.,0.],[0.,1.,0.],20.,20.,&limits).unwrap();
    let outer=(0..708).map(|i| {let a=std::f64::consts::TAU*i as f64/708.;[0.5+0.45*a.cos(),0.5+0.45*a.sin()]}).collect();
    let trim=TrimmedSurface {underlying:surface.clone(),outer,inners:vec![vec![[0.3,0.3],[0.7,0.3],[0.7,0.7],[0.3,0.7]]],reversed:false};
    let mut entity=Entity::new("trim",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();
    let mut doc=Document::with_template_tolerance("GPU trim");doc.entities.insert(entity.id,entity);
    let directory=std::env::temp_dir().join(format!("om9-trim-gpu-{}",uuid::Uuid::new_v4()));std::fs::create_dir(&directory).unwrap();
    struct Cleanup(std::path::PathBuf);impl Drop for Cleanup {fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
    let _cleanup=Cleanup(directory.clone());
    let input=directory.join("input.om9");let empty=directory.join("empty.om9");
    Archive::save(&input,&doc,&limits,&Cancellation::default()).unwrap();
    Archive::save(&empty,&Document::with_template_tolerance("empty"),&limits,&Cancellation::default()).unwrap();
    let actual=headless::render_project(&input,&directory.join("actual"),256,256).unwrap();
    let blank=headless::render_project(&empty,&directory.join("blank"),256,256).unwrap();
    let looking_down=|paths:&[std::path::PathBuf]|paths.iter().find(|p|p.file_name().unwrap().to_string_lossy().contains("looking-down")).unwrap().clone();
    let a=image::open(looking_down(&actual)).unwrap().to_rgba8();let b=image::open(looking_down(&blank)).unwrap().to_rgba8();
    let camera=Camera::orthographic([0.,0.,100.],[0.,0.,0.],[0.,1.,0.],41.).unwrap();
    let viewport=Viewport {origin:[0.,0.],size:[256.,256.]};
    let differences=|world|{let p=camera.project(world,viewport).unwrap().unwrap().screen;
        let (x,y)=(p[0].round() as i32,p[1].round() as i32);
        (-3..=3).flat_map(|dy|(-3..=3).map(move |dx|(x+dx,y+dy)))
            .filter(|&(x,y)|a.get_pixel(x as u32,y as u32)!=b.get_pixel(x as u32,y as u32)).count()};
    assert_eq!(differences([1.,1.,0.]),0,"hole must show the same background");
    assert!(differences([6.5,1.,0.])>30,"material must draw visible pixels");
    assert_eq!(differences([10.,1.,0.]),0,"outside must show the same background");
}

#[test]
fn many_edge_curved_trim_hole_renders_on_gpu_after_reopen() {
    use om9_core::rhino_semantics::{set_trimmed_surface,TrimmedSurface};
    use om9_scene::camera::{Camera,Viewport};
    if om9_render::Renderer::new().is_none() {eprintln!("skip: no wgpu adapter on this machine");return;}
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::cylinder([0.,0.,-10.],[1.,0.,0.],[0.,1.,0.],8.,20.,&limits).unwrap();
    let outer=(0..708).map(|i| {let a=std::f64::consts::TAU*i as f64/708.;[0.75+0.22*a.cos(),0.5+0.3*a.sin()]}).collect();
    let trim=TrimmedSurface {underlying:surface.clone(),outer,inners:vec![vec![[0.72,0.35],[0.78,0.35],[0.78,0.65],[0.72,0.65]]],reversed:false};
    let mut entity=Entity::new("curved trim",Geometry::Surface(surface.clone()),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();
    let mut doc=Document::with_template_tolerance("GPU curved trim");doc.entities.insert(entity.id,entity);
    let directory=std::env::temp_dir().join(format!("om9-curved-trim-gpu-{}",uuid::Uuid::new_v4()));std::fs::create_dir(&directory).unwrap();
    struct Cleanup(std::path::PathBuf);impl Drop for Cleanup {fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
    let _cleanup=Cleanup(directory.clone());let input=directory.join("input.om9");let empty=directory.join("empty.om9");
    Archive::save(&input,&doc,&limits,&Cancellation::default()).unwrap();
    Archive::save(&empty,&Document::with_template_tolerance("empty"),&limits,&Cancellation::default()).unwrap();
    let actual=headless::render_project(&input,&directory.join("actual"),256,256).unwrap();
    let blank=headless::render_project(&empty,&directory.join("blank"),256,256).unwrap();
    let view=|paths:&[std::path::PathBuf]|paths.iter().find(|p|p.file_name().unwrap().to_string_lossy().contains("through-finger")).unwrap().clone();
    let a=image::open(view(&actual)).unwrap().to_rgba8();let b=image::open(view(&blank)).unwrap().to_rgba8();
    let camera=Camera::orthographic([0.,-100.,0.],[0.,0.,0.],[0.,0.,1.],41.).unwrap();
    let viewport=Viewport {origin:[0.,0.],size:[256.,256.]};
    let differences=|uv:[f64;2]|{let world=om9_core::surface_kernel::evaluate(&surface,uv[0],uv[1],&limits).unwrap();
        let p=camera.project(world,viewport).unwrap().unwrap().screen;let (x,y)=(p[0].round() as i32,p[1].round() as i32);
        (-2..=2).flat_map(|dy|(-2..=2).map(move |dx|(x+dx,y+dy)))
            .filter(|&(x,y)|a.get_pixel(x as u32,y as u32)!=b.get_pixel(x as u32,y as u32)).count()};
    assert_eq!(differences([0.75,0.5]),0,"hole must show the same background");
    assert!(differences([0.88,0.5])>15,"material must draw visible pixels");
    assert_eq!(differences([0.75,0.1]),0,"outside must show the same background");
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
    let box_mesh=Mesh{vertices:box_shape.display().points.iter().map(|p|map(*p,-5.)).collect(),
        faces:box_shape.display().triangles.iter().map(|t|vec![t[0],t[1],t[2]]).collect(),normals:vec![],colors:vec![],corner_uvs:vec![]};
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
