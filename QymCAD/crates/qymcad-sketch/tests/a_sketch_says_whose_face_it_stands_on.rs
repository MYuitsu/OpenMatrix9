//! A SKETCH SAYS WHOSE FACE IT STANDS ON: the properties name the part the face belongs to, and a work plane by its
//! name. Reported behaviour: they said "face of body #37" - a number nobody sees anywhere else.
use qymcad_core::feature::{FaceKey, SketchPlane};
use qymcad_core::model::Project;

#[test]
fn a_sketch_on_a_face_names_the_part() {
    qymcad_i18n::set_language("en");
    let mut p = Project::default();
    let root = p.ensure_root();
    p.set_active_component(Some(root));
    let part = p.add_part("Housing");
    p.set_active_component(Some(part));
    let body = p.add_box(20.0, 20.0, 10.0);
    let words = qymcad_sketch::sketch_plane_words(&p, SketchPlane::Face(body, FaceKey::default()));
    assert!(words.contains("Housing") && !words.contains('#'), "a sketch on a face of the part Housing is said to stand on {words:?}");
}
