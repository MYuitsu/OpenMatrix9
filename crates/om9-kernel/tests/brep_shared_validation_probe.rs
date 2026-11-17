// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{budget::Limits, geometry::Geometry, model::{Document, Entity}};
use om9_kernel::{creation::{primitive, Primitive}, editing::extract_face};

#[test]
#[ignore = "release performance probe; run explicitly"]
fn shared_and_distinct_brep_document_validation() {
    let limits = Limits::default();
    let body = primitive(Primitive::Sphere { radius_mm: 3. }, 0.001, &limits).unwrap();
    let mut shared = Document::with_template_tolerance("shared validation");
    let mut distinct = Document::with_template_tolerance("distinct validation");
    for index in 0..22 {
        let entity = Entity::new(format!("shared {index}"), Geometry::Brep(body.clone()), None);
        shared.entities.insert(entity.id, entity);
        let face = extract_face(&body, 0, 0.001, &limits).unwrap();
        let entity = Entity::new(format!("face {index}"), Geometry::Brep(face), None);
        distinct.entities.insert(entity.id, entity);
    }
    for (name, document) in [("shared", &shared), ("distinct", &distinct)] {
        let mut elapsed = Vec::new();
        let mut expected = None;
        for _ in 0..3 {
            let start = std::time::Instant::now();
            let usage = document.validate(&limits).unwrap();
            elapsed.push(start.elapsed().as_micros());
            if let Some(previous) = expected { assert_eq!(usage, previous); } else { expected = Some(usage); }
        }
        eprintln!("{name}_validation_us={elapsed:?}");
    }
}
