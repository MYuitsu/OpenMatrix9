// SPDX-License-Identifier: LGPL-2.1-or-later
//! `OM9-L2-02-01` persistence: trim and extrusion semantics survive the project codec, and a
//! damaged semantic payload is refused before a document is published.

use om9_archive::Archive;
use om9_core::{
    Error,
    budget::Limits,
    curve_build,
    geometry::{Geometry, Point3},
    model::{Document, Entity, Extension},
    rhino_semantics::{
        Extrusion, SemanticKind, ShapeClass, TrimmedSurface, classify_shell, entity_semantic_kind,
        set_extrusion, set_trimmed_surface, trimmed_surface_of,
    },
    surface_kernel,
    transaction::Cancellation,
};

fn point(x: f64, y: f64, z: f64) -> Point3 {
    [x, y, z]
}

fn plane() -> om9_core::geometry::NurbsSurface {
    surface_kernel::plane(
        point(0.0, 0.0, 0.0),
        point(1.0, 0.0, 0.0),
        point(0.0, 1.0, 0.0),
        10.0,
        10.0,
        &Limits::default(),
    )
    .unwrap()
}

fn trimmed() -> TrimmedSurface {
    TrimmedSurface {
        underlying: plane(),
        outer: vec![[0.2, 0.2], [0.8, 0.2], [0.8, 0.8], [0.2, 0.8]],
        inners: vec![vec![[0.4, 0.4], [0.6, 0.4], [0.5, 0.6]]],
        reversed: false,
    }
}

fn extrusion() -> Extrusion {
    Extrusion {
        profile: curve_build::rectangle(
            point(0.0, 0.0, 0.0),
            point(1.0, 0.0, 0.0),
            point(0.0, 1.0, 0.0),
            10.0,
            20.0,
        )
        .unwrap(),
        direction: point(0.0, 0.0, 1.0),
        height: 30.0,
        capped: true,
    }
}

#[test]
fn trimmed_surface_semantics_survive_save_and_reopen_without_becoming_untrimmed() {
    let limits = Limits::default();
    let trimmed = trimmed();
    let mut entity = Entity::new(
        "trimmed patch",
        Geometry::Surface(trimmed.underlying.clone()),
        None,
    );
    set_trimmed_surface(&mut entity, &trimmed, &limits).unwrap();
    let id = entity.id;
    let mut document = Document::new("trim persistence");
    document.entities.insert(id, entity);

    let folder = std::env::temp_dir().join(format!("om9-trim-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&folder).unwrap();
    let path = folder.join("trim.om9");
    Archive::save(&path, &document, &limits, &Cancellation::default()).unwrap();
    let reopened = Archive::open(&path, &limits).unwrap();
    assert_eq!(reopened, document);
    let entity = &reopened.entities[&id];
    assert_eq!(
        entity_semantic_kind(entity).unwrap(),
        SemanticKind::TrimmedSurface
    );
    let restored = trimmed_surface_of(entity, &limits).unwrap().unwrap();
    assert_eq!(restored, trimmed);
    assert!(restored.is_trimmed());
    assert_eq!(
        restored.to_untrimmed(),
        Err(Error::Unsupported(
            "trimmed surface cannot be promoted to an untrimmed surface"
        ))
    );
    assert_eq!(restored.underlying.domain, [[0.0, 1.0], [0.0, 1.0]]);
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(folder).unwrap();
}

#[test]
fn larger_trim_source_roundtrips_under_default_limits() {
    let limits=Limits::default();let n=28usize;
    let mut poles=Vec::new();for u in 0..n {for v in 0..n {poles.push([10.*u as f64/(n-1) as f64,10.*v as f64/(n-1) as f64,0.]);}}
    let mut knots=vec![0.];knots.extend((0..n).map(|i|i as f64/(n-1) as f64));knots.push(1.);
    let surface=om9_core::geometry::NurbsSurface {degree:[1,1],size:[n,n],poles,weights:vec![1.;n*n],knots:[knots.clone(),knots],domain:[[0.,1.],[0.,1.]],periodic:[false,false],reversed:false};
    let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.2,0.2],[0.8,0.2],[0.8,0.8],[0.2,0.8]],
        inners:vec![vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]],reversed:false};
    let mut entity=Entity::new("large source",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();
    assert!(entity.extensions[om9_core::model::TRIM_SURFACE_EXTENSION_SLOT].payload.len()>16*1024);
    let id=entity.id;let mut document=Document::new("large trim");document.entities.insert(id,entity);
    let bytes=Archive::encode(&document,&limits).unwrap();let reopened=Archive::decode(&bytes,&limits).unwrap();
    assert_eq!(trimmed_surface_of(&reopened.entities[&id],&limits).unwrap().unwrap(),trim);
}

#[test]
fn mismatched_trim_is_not_saved_over_an_existing_file() {
    let limits=Limits::default();let trim=trimmed();
    let mut entity=Entity::new("stale trim",Geometry::Surface(trim.underlying.clone()),None);
    set_trimmed_surface(&mut entity,&trim,&limits).unwrap();
    let Geometry::Surface(surface)=&mut entity.geometry else {unreachable!()};surface.poles[3][2]=1.;
    let mut document=Document::new("mismatched trim");document.entities.insert(entity.id,entity);
    let folder=std::env::temp_dir().join(format!("om9-stale-trim-{}",uuid::Uuid::new_v4()));
    std::fs::create_dir(&folder).unwrap();let path=folder.join("drawing.om9");std::fs::write(&path,b"existing file").unwrap();
    assert!(matches!(Archive::save(&path,&document,&limits,&Cancellation::default()),Err(Error::Invalid("trimmed surface underlying mismatch"))));
    assert_eq!(std::fs::read(&path).unwrap(),b"existing file");
    std::fs::remove_file(path).unwrap();std::fs::remove_dir(folder).unwrap();
}

#[test]
fn extrusion_semantics_survive_save_and_reopen_and_still_convert() {
    let limits = Limits::default();
    let extrusion = extrusion();
    let mut entity = Entity::new(
        "extrusion",
        Geometry::Curve(extrusion.profile.clone()),
        None,
    );
    set_extrusion(&mut entity, &extrusion, &limits).unwrap();
    let id = entity.id;
    let mut document = Document::new("extrusion persistence");
    document.entities.insert(id, entity);

    let bytes = Archive::encode(&document, &limits).unwrap();
    let reopened = Archive::decode(&bytes, &limits).unwrap();
    assert_eq!(reopened, document);
    let entity = &reopened.entities[&id];
    assert_eq!(
        entity_semantic_kind(entity).unwrap(),
        SemanticKind::Extrusion
    );
    let restored = om9_core::rhino_semantics::extrusion_of(entity, &limits)
        .unwrap()
        .unwrap();
    assert_eq!(restored, extrusion);
    let conversion = restored.convert(1.0e-6, &limits).unwrap();
    assert_eq!(conversion.source, extrusion);
    assert_eq!(
        classify_shell(&conversion.body, &limits).unwrap(),
        ShapeClass::Solid
    );
    assert_eq!(
        conversion.body.kind,
        om9_core::brep_kernel::ShapeKind::Solid
    );
}

#[test]
fn a_damaged_trim_payload_is_refused_before_publication() {
    let limits = Limits::default();
    let trimmed = trimmed();
    let mut entity = Entity::new(
        "damaged trim",
        Geometry::Surface(trimmed.underlying.clone()),
        None,
    );
    set_trimmed_surface(&mut entity, &trimmed, &limits).unwrap();
    let mut value: serde_json::Value =
        serde_json::from_str(&entity.extensions["om9.trim.surface"].payload).unwrap();
    value["outer"] = serde_json::json!([[0.0, 0.0], [1.0, 0.0]]);
    entity.extensions.insert(
        "om9.trim.surface".into(),
        Extension::new(1, serde_json::to_string(&value).unwrap()).unwrap(),
    );
    let mut document = Document::new("damaged trim");
    document.entities.insert(entity.id, entity);
    assert_eq!(
        document.validate(&limits),
        Err(Error::Invalid("trim outer loop"))
    );
    assert!(Archive::encode(&document, &limits).is_err());
}

#[test]
fn mesh_and_brep_keep_distinct_kinds_across_the_codec() {
    let limits = Limits::default();
    let mut document = Document::new("mesh vs brep");
    let mesh = Entity::new(
        "mesh",
        Geometry::Mesh(om9_core::geometry::Mesh {
            vertices: vec![
                point(0.0, 0.0, 0.0),
                point(1.0, 0.0, 0.0),
                point(0.0, 1.0, 0.0),
            ],
            faces: vec![vec![0, 1, 2]],
            normals: vec![],
            colors: vec![],

            corner_uvs: Vec::new(),
        }),
        None,
    );
    let mesh_id = mesh.id;
    document.entities.insert(mesh_id, mesh);
    let bytes = Archive::encode(&document, &limits).unwrap();
    let reopened = Archive::decode(&bytes, &limits).unwrap();
    assert_eq!(
        entity_semantic_kind(&reopened.entities[&mesh_id]).unwrap(),
        SemanticKind::Mesh
    );
    assert_eq!(
        om9_core::rhino_semantics::require_exact_nurbs(&reopened.entities[&mesh_id].geometry),
        Err(Error::Unsupported("mesh is not exact NURBS"))
    );
}
