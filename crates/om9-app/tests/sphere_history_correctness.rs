// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::{budget::Limits, geometry::Geometry, model::Document};

#[test]
fn b0_product_five_mm_sphere_delete_undo_redo_preserves_source_and_identity() {
    let limits = Limits::default();
    let mut c = Controller::new(Document::new("B0 correctness"),limits).unwrap();
    c.submit("Sphere 5").unwrap();
    let (&id,entity) = c.document().entities.iter().next().unwrap();
    let placement = entity.placement;
    let Geometry::Brep(brep) = &entity.geometry else { panic!("Sphere must be B-rep") };
    let source = brep.source().to_owned();
    let topology = brep.topology().clone();
    let volume = om9_kernel::creation::measure(brep,&limits).unwrap().volume_mm3;
    assert!((volume-4.*std::f64::consts::PI*125./3.).abs()<1e-6);
    let display_identity = brep.display_identity();
    let undo_count = c.coordinator().undo_len();
    assert_eq!(c.delete_objects(&[id]).unwrap(),1);
    assert!(c.document().entities.is_empty());
    assert_eq!(c.coordinator().undo_len(),undo_count+1);
    c.undo().unwrap();
    let restored = &c.document().entities[&id];
    assert_eq!(restored.placement,placement);
    let Geometry::Brep(brep) = &restored.geometry else { panic!("Undo must restore B-rep") };
    assert_eq!(brep.source(),source);
    assert_eq!(brep.topology(),&topology);
    assert!(brep.display_identity().same(&display_identity),"ordinary Undo currently restores the retained display");
    assert!((om9_kernel::creation::measure(brep,&limits).unwrap().volume_mm3-volume).abs()<1e-9);
    c.redo().unwrap();
    assert!(c.document().entities.is_empty());
    c.undo().unwrap();
    assert!(c.document().entities.contains_key(&id));
}
